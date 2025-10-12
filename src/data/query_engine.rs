use anyhow::{anyhow, Result};
use fxhash::FxHashSet;
use std::cmp::min;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, info};

use crate::config::config::BehaviorConfig;
use crate::config::global::get_date_notation;
use crate::data::arithmetic_evaluator::ArithmeticEvaluator;
use crate::data::data_view::DataView;
use crate::data::datatable::{DataColumn, DataRow, DataTable, DataValue};
use crate::data::evaluation_context::EvaluationContext;
use crate::data::group_by_expressions::GroupByExpressions;
use crate::data::hash_join::HashJoinExecutor;
use crate::data::recursive_where_evaluator::RecursiveWhereEvaluator;
use crate::data::row_expanders::RowExpanderRegistry;
use crate::data::subquery_executor::SubqueryExecutor;
use crate::data::temp_table_registry::TempTableRegistry;
use crate::execution_plan::{ExecutionPlan, ExecutionPlanBuilder, StepType};
use crate::sql::aggregates::{contains_aggregate, is_aggregate_compatible};
use crate::sql::parser::ast::ColumnRef;
use crate::sql::parser::ast::SetOperation;
use crate::sql::parser::ast::TableSource;
use crate::sql::recursive_parser::{
    CTEType, OrderByColumn, Parser, SelectItem, SelectStatement, SortDirection, SqlExpression,
    TableFunction,
};

/// Execution context for tracking table aliases and scope during query execution
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// Map from alias to actual table/CTE name
    /// Example: "t" -> "#tmp_trades", "a" -> "data"
    alias_map: HashMap<String, String>,
}

impl ExecutionContext {
    /// Create a new empty execution context
    pub fn new() -> Self {
        Self {
            alias_map: HashMap::new(),
        }
    }

    /// Register a table alias
    pub fn register_alias(&mut self, alias: String, table_name: String) {
        debug!("Registering alias: {} -> {}", alias, table_name);
        self.alias_map.insert(alias, table_name);
    }

    /// Resolve an alias to its actual table name
    /// Returns the alias itself if not found in the map
    pub fn resolve_alias(&self, name: &str) -> String {
        self.alias_map
            .get(name)
            .cloned()
            .unwrap_or_else(|| name.to_string())
    }

    /// Check if a name is a registered alias
    pub fn is_alias(&self, name: &str) -> bool {
        self.alias_map.contains_key(name)
    }

    /// Get a copy of all registered aliases
    pub fn get_aliases(&self) -> HashMap<String, String> {
        self.alias_map.clone()
    }

    /// Resolve a column reference to its index in the table, handling aliases
    ///
    /// This is the unified column resolution function that should be used by all
    /// SQL clauses (WHERE, SELECT, ORDER BY, GROUP BY) to ensure consistent
    /// alias resolution behavior.
    ///
    /// Resolution strategy:
    /// 1. If column_ref has a table_prefix (e.g., "t" in "t.amount"):
    ///    a. Resolve the alias: t -> actual_table_name
    ///    b. Try qualified lookup: "actual_table_name.amount"
    ///    c. Fall back to unqualified: "amount"
    /// 2. If column_ref has no prefix:
    ///    a. Try simple column name lookup: "amount"
    ///    b. Try as qualified name if it contains a dot: "table.column"
    pub fn resolve_column_index(&self, table: &DataTable, column_ref: &ColumnRef) -> Result<usize> {
        if let Some(table_prefix) = &column_ref.table_prefix {
            // Qualified column reference: resolve the alias first
            let actual_table = self.resolve_alias(table_prefix);

            // Try qualified lookup: "actual_table.column"
            let qualified_name = format!("{}.{}", actual_table, column_ref.name);
            if let Some(idx) = table.find_column_by_qualified_name(&qualified_name) {
                debug!(
                    "Resolved {}.{} -> qualified column '{}' at index {}",
                    table_prefix, column_ref.name, qualified_name, idx
                );
                return Ok(idx);
            }

            // Fall back to unqualified lookup
            if let Some(idx) = table.get_column_index(&column_ref.name) {
                debug!(
                    "Resolved {}.{} -> unqualified column '{}' at index {}",
                    table_prefix, column_ref.name, column_ref.name, idx
                );
                return Ok(idx);
            }

            // Not found with either qualified or unqualified name
            Err(anyhow!(
                "Column '{}' not found. Table '{}' may not support qualified column names",
                qualified_name,
                actual_table
            ))
        } else {
            // Unqualified column reference
            if let Some(idx) = table.get_column_index(&column_ref.name) {
                debug!(
                    "Resolved unqualified column '{}' at index {}",
                    column_ref.name, idx
                );
                return Ok(idx);
            }

            // If the column name contains a dot, try it as a qualified name
            if column_ref.name.contains('.') {
                if let Some(idx) = table.find_column_by_qualified_name(&column_ref.name) {
                    debug!(
                        "Resolved '{}' as qualified column at index {}",
                        column_ref.name, idx
                    );
                    return Ok(idx);
                }
            }

            // Column not found - provide helpful error
            let suggestion = self.find_similar_column(table, &column_ref.name);
            match suggestion {
                Some(similar) => Err(anyhow!(
                    "Column '{}' not found. Did you mean '{}'?",
                    column_ref.name,
                    similar
                )),
                None => Err(anyhow!("Column '{}' not found", column_ref.name)),
            }
        }
    }

    /// Find a similar column name using edit distance (for better error messages)
    fn find_similar_column(&self, table: &DataTable, name: &str) -> Option<String> {
        let columns = table.column_names();
        let mut best_match: Option<(String, usize)> = None;

        for col in columns {
            let distance = edit_distance(name, &col);
            if distance <= 2 {
                // Allow up to 2 character differences
                match best_match {
                    Some((_, best_dist)) if distance < best_dist => {
                        best_match = Some((col.clone(), distance));
                    }
                    None => {
                        best_match = Some((col.clone(), distance));
                    }
                    _ => {}
                }
            }
        }

        best_match.map(|(name, _)| name)
    }
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculate edit distance between two strings (Levenshtein distance)
fn edit_distance(a: &str, b: &str) -> usize {
    let len_a = a.chars().count();
    let len_b = b.chars().count();

    if len_a == 0 {
        return len_b;
    }
    if len_b == 0 {
        return len_a;
    }

    let mut matrix = vec![vec![0; len_b + 1]; len_a + 1];

    for i in 0..=len_a {
        matrix[i][0] = i;
    }
    for j in 0..=len_b {
        matrix[0][j] = j;
    }

    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();

    for i in 1..=len_a {
        for j in 1..=len_b {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            matrix[i][j] = min(
                min(matrix[i - 1][j] + 1, matrix[i][j - 1] + 1),
                matrix[i - 1][j - 1] + cost,
            );
        }
    }

    matrix[len_a][len_b]
}

/// Query engine that executes SQL directly on `DataTable`
#[derive(Clone)]
pub struct QueryEngine {
    case_insensitive: bool,
    date_notation: String,
    _behavior_config: Option<BehaviorConfig>,
}

impl Default for QueryEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl QueryEngine {
    #[must_use]
    pub fn new() -> Self {
        Self {
            case_insensitive: false,
            date_notation: get_date_notation(),
            _behavior_config: None,
        }
    }

    #[must_use]
    pub fn with_behavior_config(config: BehaviorConfig) -> Self {
        let case_insensitive = config.case_insensitive_default;
        // Use get_date_notation() to respect environment variable override
        let date_notation = get_date_notation();
        Self {
            case_insensitive,
            date_notation,
            _behavior_config: Some(config),
        }
    }

    #[must_use]
    pub fn with_date_notation(_date_notation: String) -> Self {
        Self {
            case_insensitive: false,
            date_notation: get_date_notation(), // Always use the global function
            _behavior_config: None,
        }
    }

    #[must_use]
    pub fn with_case_insensitive(case_insensitive: bool) -> Self {
        Self {
            case_insensitive,
            date_notation: get_date_notation(),
            _behavior_config: None,
        }
    }

    #[must_use]
    pub fn with_case_insensitive_and_date_notation(
        case_insensitive: bool,
        _date_notation: String, // Keep parameter for compatibility but use get_date_notation()
    ) -> Self {
        Self {
            case_insensitive,
            date_notation: get_date_notation(), // Always use the global function
            _behavior_config: None,
        }
    }

    /// Find a column name similar to the given name using edit distance
    fn find_similar_column(&self, table: &DataTable, name: &str) -> Option<String> {
        let columns = table.column_names();
        let mut best_match: Option<(String, usize)> = None;

        for col in columns {
            let distance = self.edit_distance(&col.to_lowercase(), &name.to_lowercase());
            // Only suggest if distance is small (likely a typo)
            // Allow up to 3 edits for longer names
            let max_distance = if name.len() > 10 { 3 } else { 2 };
            if distance <= max_distance {
                match &best_match {
                    None => best_match = Some((col, distance)),
                    Some((_, best_dist)) if distance < *best_dist => {
                        best_match = Some((col, distance));
                    }
                    _ => {}
                }
            }
        }

        best_match.map(|(name, _)| name)
    }

    /// Calculate Levenshtein edit distance between two strings
    fn edit_distance(&self, s1: &str, s2: &str) -> usize {
        let len1 = s1.len();
        let len2 = s2.len();
        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }

        for (i, c1) in s1.chars().enumerate() {
            for (j, c2) in s2.chars().enumerate() {
                let cost = usize::from(c1 != c2);
                matrix[i + 1][j + 1] = std::cmp::min(
                    matrix[i][j + 1] + 1, // deletion
                    std::cmp::min(
                        matrix[i + 1][j] + 1, // insertion
                        matrix[i][j] + cost,  // substitution
                    ),
                );
            }
        }

        matrix[len1][len2]
    }

    /// Check if an expression contains UNNEST function call
    fn contains_unnest(expr: &SqlExpression) -> bool {
        match expr {
            // Direct UNNEST variant
            SqlExpression::Unnest { .. } => true,
            SqlExpression::FunctionCall { name, args, .. } => {
                if name.to_uppercase() == "UNNEST" {
                    return true;
                }
                // Check recursively in function arguments
                args.iter().any(Self::contains_unnest)
            }
            SqlExpression::BinaryOp { left, right, .. } => {
                Self::contains_unnest(left) || Self::contains_unnest(right)
            }
            SqlExpression::Not { expr } => Self::contains_unnest(expr),
            SqlExpression::CaseExpression {
                when_branches,
                else_branch,
            } => {
                when_branches.iter().any(|branch| {
                    Self::contains_unnest(&branch.condition)
                        || Self::contains_unnest(&branch.result)
                }) || else_branch
                    .as_ref()
                    .map_or(false, |e| Self::contains_unnest(e))
            }
            SqlExpression::SimpleCaseExpression {
                expr,
                when_branches,
                else_branch,
            } => {
                Self::contains_unnest(expr)
                    || when_branches.iter().any(|branch| {
                        Self::contains_unnest(&branch.value)
                            || Self::contains_unnest(&branch.result)
                    })
                    || else_branch
                        .as_ref()
                        .map_or(false, |e| Self::contains_unnest(e))
            }
            SqlExpression::InList { expr, values } => {
                Self::contains_unnest(expr) || values.iter().any(Self::contains_unnest)
            }
            SqlExpression::NotInList { expr, values } => {
                Self::contains_unnest(expr) || values.iter().any(Self::contains_unnest)
            }
            SqlExpression::Between { expr, lower, upper } => {
                Self::contains_unnest(expr)
                    || Self::contains_unnest(lower)
                    || Self::contains_unnest(upper)
            }
            SqlExpression::InSubquery { expr, .. } => Self::contains_unnest(expr),
            SqlExpression::NotInSubquery { expr, .. } => Self::contains_unnest(expr),
            SqlExpression::ScalarSubquery { .. } => false, // Subqueries are handled separately
            SqlExpression::WindowFunction { args, .. } => args.iter().any(Self::contains_unnest),
            SqlExpression::MethodCall { args, .. } => args.iter().any(Self::contains_unnest),
            SqlExpression::ChainedMethodCall { base, args, .. } => {
                Self::contains_unnest(base) || args.iter().any(Self::contains_unnest)
            }
            _ => false,
        }
    }

    /// Execute a SQL query on a `DataTable` and return a `DataView` (for backward compatibility)
    pub fn execute(&self, table: Arc<DataTable>, sql: &str) -> Result<DataView> {
        let (view, _plan) = self.execute_with_plan(table, sql)?;
        Ok(view)
    }

    /// Execute a SQL query with optional temp table registry access
    pub fn execute_with_temp_tables(
        &self,
        table: Arc<DataTable>,
        sql: &str,
        temp_tables: Option<&TempTableRegistry>,
    ) -> Result<DataView> {
        let (view, _plan) = self.execute_with_plan_and_temp_tables(table, sql, temp_tables)?;
        Ok(view)
    }

    /// Execute a parsed SelectStatement on a `DataTable` and return a `DataView`
    pub fn execute_statement(
        &self,
        table: Arc<DataTable>,
        statement: SelectStatement,
    ) -> Result<DataView> {
        self.execute_statement_with_temp_tables(table, statement, None)
    }

    /// Execute a parsed SelectStatement with optional temp table access
    pub fn execute_statement_with_temp_tables(
        &self,
        table: Arc<DataTable>,
        statement: SelectStatement,
        temp_tables: Option<&TempTableRegistry>,
    ) -> Result<DataView> {
        // First process CTEs to build context
        let mut cte_context = HashMap::new();

        // Add temp tables to CTE context if provided
        if let Some(temp_registry) = temp_tables {
            for table_name in temp_registry.list_tables() {
                if let Some(temp_table) = temp_registry.get(&table_name) {
                    debug!("Adding temp table {} to CTE context", table_name);
                    let view = DataView::new(temp_table);
                    cte_context.insert(table_name, Arc::new(view));
                }
            }
        }

        for cte in &statement.ctes {
            debug!("QueryEngine: Pre-processing CTE '{}'...", cte.name);
            // Execute the CTE based on its type
            let cte_result = match &cte.cte_type {
                CTEType::Standard(query) => {
                    // Execute the CTE query (it might reference earlier CTEs)
                    let view = self.build_view_with_context(
                        table.clone(),
                        query.clone(),
                        &mut cte_context,
                    )?;

                    // Materialize the view and enrich columns with qualified names
                    let mut materialized = self.materialize_view(view)?;

                    // Enrich columns with qualified names for proper scoping
                    for column in materialized.columns_mut() {
                        column.qualified_name = Some(format!("{}.{}", cte.name, column.name));
                        column.source_table = Some(cte.name.clone());
                    }

                    DataView::new(Arc::new(materialized))
                }
                CTEType::Web(web_spec) => {
                    // Fetch data from URL
                    use crate::web::http_fetcher::WebDataFetcher;

                    let fetcher = WebDataFetcher::new()?;
                    // Pass None for query context (no full SQL available in these contexts)
                    let mut data_table = fetcher.fetch(web_spec, &cte.name, None)?;

                    // Enrich columns with qualified names for proper scoping
                    for column in data_table.columns_mut() {
                        column.qualified_name = Some(format!("{}.{}", cte.name, column.name));
                        column.source_table = Some(cte.name.clone());
                    }

                    // Convert DataTable to DataView
                    DataView::new(Arc::new(data_table))
                }
            };
            // Store the result in the context for later use
            cte_context.insert(cte.name.clone(), Arc::new(cte_result));
            debug!(
                "QueryEngine: CTE '{}' pre-processed, stored in context",
                cte.name
            );
        }

        // Now process subqueries with CTE context available
        let mut subquery_executor =
            SubqueryExecutor::with_cte_context(self.clone(), table.clone(), cte_context.clone());
        let processed_statement = subquery_executor.execute_subqueries(&statement)?;

        // Build the view with the same CTE context
        self.build_view_with_context(table, processed_statement, &mut cte_context)
    }

    /// Execute a statement with provided CTE context (for subqueries)
    pub fn execute_statement_with_cte_context(
        &self,
        table: Arc<DataTable>,
        statement: SelectStatement,
        cte_context: &HashMap<String, Arc<DataView>>,
    ) -> Result<DataView> {
        // Clone the context so we can add any CTEs from this statement
        let mut local_context = cte_context.clone();

        // Process any CTEs in this statement (they might be nested)
        for cte in &statement.ctes {
            debug!("QueryEngine: Processing nested CTE '{}'...", cte.name);
            let cte_result = match &cte.cte_type {
                CTEType::Standard(query) => {
                    let view = self.build_view_with_context(
                        table.clone(),
                        query.clone(),
                        &mut local_context,
                    )?;

                    // Materialize the view and enrich columns with qualified names
                    let mut materialized = self.materialize_view(view)?;

                    // Enrich columns with qualified names for proper scoping
                    for column in materialized.columns_mut() {
                        column.qualified_name = Some(format!("{}.{}", cte.name, column.name));
                        column.source_table = Some(cte.name.clone());
                    }

                    DataView::new(Arc::new(materialized))
                }
                CTEType::Web(web_spec) => {
                    // Fetch data from URL
                    use crate::web::http_fetcher::WebDataFetcher;

                    let fetcher = WebDataFetcher::new()?;
                    // Pass None for query context (no full SQL available in these contexts)
                    let mut data_table = fetcher.fetch(web_spec, &cte.name, None)?;

                    // Enrich columns with qualified names for proper scoping
                    for column in data_table.columns_mut() {
                        column.qualified_name = Some(format!("{}.{}", cte.name, column.name));
                        column.source_table = Some(cte.name.clone());
                    }

                    // Convert DataTable to DataView
                    DataView::new(Arc::new(data_table))
                }
            };
            local_context.insert(cte.name.clone(), Arc::new(cte_result));
        }

        // Process subqueries with the complete context
        let mut subquery_executor =
            SubqueryExecutor::with_cte_context(self.clone(), table.clone(), local_context.clone());
        let processed_statement = subquery_executor.execute_subqueries(&statement)?;

        // Build the view
        self.build_view_with_context(table, processed_statement, &mut local_context)
    }

    /// Execute a query and return both the result and the execution plan
    pub fn execute_with_plan(
        &self,
        table: Arc<DataTable>,
        sql: &str,
    ) -> Result<(DataView, ExecutionPlan)> {
        self.execute_with_plan_and_temp_tables(table, sql, None)
    }

    /// Execute a query with temp tables and return both the result and the execution plan
    pub fn execute_with_plan_and_temp_tables(
        &self,
        table: Arc<DataTable>,
        sql: &str,
        temp_tables: Option<&TempTableRegistry>,
    ) -> Result<(DataView, ExecutionPlan)> {
        let mut plan_builder = ExecutionPlanBuilder::new();
        let start_time = Instant::now();

        // Parse the SQL query
        plan_builder.begin_step(StepType::Parse, "Parse SQL query".to_string());
        plan_builder.add_detail(format!("Query: {}", sql));
        let mut parser = Parser::new(sql);
        let statement = parser
            .parse()
            .map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;
        plan_builder.add_detail(format!("Parsed successfully"));
        if let Some(from) = &statement.from_table {
            plan_builder.add_detail(format!("FROM: {}", from));
        }
        if statement.where_clause.is_some() {
            plan_builder.add_detail("WHERE clause present".to_string());
        }
        plan_builder.end_step();

        // First process CTEs to build context
        let mut cte_context = HashMap::new();

        // Add temp tables to CTE context if provided
        if let Some(temp_registry) = temp_tables {
            for table_name in temp_registry.list_tables() {
                if let Some(temp_table) = temp_registry.get(&table_name) {
                    debug!("Adding temp table {} to CTE context", table_name);
                    let view = DataView::new(temp_table);
                    cte_context.insert(table_name, Arc::new(view));
                }
            }
        }

        if !statement.ctes.is_empty() {
            plan_builder.begin_step(
                StepType::CTE,
                format!("Process {} CTEs", statement.ctes.len()),
            );

            for cte in &statement.ctes {
                let cte_start = Instant::now();
                plan_builder.begin_step(StepType::CTE, format!("CTE '{}'", cte.name));

                let cte_result = match &cte.cte_type {
                    CTEType::Standard(query) => {
                        // Add CTE query details
                        if let Some(from) = &query.from_table {
                            plan_builder.add_detail(format!("Source: {}", from));
                        }
                        if query.where_clause.is_some() {
                            plan_builder.add_detail("Has WHERE clause".to_string());
                        }
                        if query.group_by.is_some() {
                            plan_builder.add_detail("Has GROUP BY".to_string());
                        }

                        debug!(
                            "QueryEngine: Processing CTE '{}' with existing context: {:?}",
                            cte.name,
                            cte_context.keys().collect::<Vec<_>>()
                        );

                        // Process subqueries in the CTE's query FIRST
                        // This allows the subqueries to see all previously defined CTEs
                        let mut subquery_executor = SubqueryExecutor::with_cte_context(
                            self.clone(),
                            table.clone(),
                            cte_context.clone(),
                        );
                        let processed_query = subquery_executor.execute_subqueries(query)?;

                        let view = self.build_view_with_context(
                            table.clone(),
                            processed_query,
                            &mut cte_context,
                        )?;

                        // Materialize the view and enrich columns with qualified names
                        let mut materialized = self.materialize_view(view)?;

                        // Enrich columns with qualified names for proper scoping
                        for column in materialized.columns_mut() {
                            column.qualified_name = Some(format!("{}.{}", cte.name, column.name));
                            column.source_table = Some(cte.name.clone());
                        }

                        DataView::new(Arc::new(materialized))
                    }
                    CTEType::Web(web_spec) => {
                        plan_builder.add_detail(format!("URL: {}", web_spec.url));
                        if let Some(format) = &web_spec.format {
                            plan_builder.add_detail(format!("Format: {:?}", format));
                        }
                        if let Some(cache) = web_spec.cache_seconds {
                            plan_builder.add_detail(format!("Cache: {} seconds", cache));
                        }

                        // Fetch data from URL
                        use crate::web::http_fetcher::WebDataFetcher;

                        let fetcher = WebDataFetcher::new()?;
                        // Pass None for query context - each WEB CTE is independent
                        let mut data_table = fetcher.fetch(web_spec, &cte.name, None)?;

                        // Enrich columns with qualified names for proper scoping
                        for column in data_table.columns_mut() {
                            column.qualified_name = Some(format!("{}.{}", cte.name, column.name));
                            column.source_table = Some(cte.name.clone());
                        }

                        // Convert DataTable to DataView
                        DataView::new(Arc::new(data_table))
                    }
                };

                // Record CTE statistics
                plan_builder.set_rows_out(cte_result.row_count());
                plan_builder.add_detail(format!(
                    "Result: {} rows, {} columns",
                    cte_result.row_count(),
                    cte_result.column_count()
                ));
                plan_builder.add_detail(format!(
                    "Execution time: {:.3}ms",
                    cte_start.elapsed().as_secs_f64() * 1000.0
                ));

                debug!(
                    "QueryEngine: Storing CTE '{}' in context with {} rows",
                    cte.name,
                    cte_result.row_count()
                );
                cte_context.insert(cte.name.clone(), Arc::new(cte_result));
                plan_builder.end_step();
            }

            plan_builder.add_detail(format!(
                "All {} CTEs cached in context",
                statement.ctes.len()
            ));
            plan_builder.end_step();
        }

        // Process subqueries in the statement with CTE context
        plan_builder.begin_step(StepType::Subquery, "Process subqueries".to_string());
        let mut subquery_executor =
            SubqueryExecutor::with_cte_context(self.clone(), table.clone(), cte_context.clone());

        // Check if there are subqueries to process
        let has_subqueries = statement.where_clause.as_ref().map_or(false, |w| {
            // This is a simplified check - in reality we'd need to walk the AST
            format!("{:?}", w).contains("Subquery")
        });

        if has_subqueries {
            plan_builder.add_detail("Evaluating subqueries in WHERE clause".to_string());
        }

        let processed_statement = subquery_executor.execute_subqueries(&statement)?;

        if has_subqueries {
            plan_builder.add_detail("Subqueries replaced with materialized values".to_string());
        } else {
            plan_builder.add_detail("No subqueries to process".to_string());
        }

        plan_builder.end_step();
        let result = self.build_view_with_context_and_plan(
            table,
            processed_statement,
            &mut cte_context,
            &mut plan_builder,
        )?;

        let total_duration = start_time.elapsed();
        info!(
            "Query execution complete: total={:?}, rows={}",
            total_duration,
            result.row_count()
        );

        let plan = plan_builder.build();
        Ok((result, plan))
    }

    /// Build a `DataView` from a parsed SQL statement
    fn build_view(&self, table: Arc<DataTable>, statement: SelectStatement) -> Result<DataView> {
        let mut cte_context = HashMap::new();
        self.build_view_with_context(table, statement, &mut cte_context)
    }

    /// Build a DataView from a SelectStatement with CTE context
    fn build_view_with_context(
        &self,
        table: Arc<DataTable>,
        statement: SelectStatement,
        cte_context: &mut HashMap<String, Arc<DataView>>,
    ) -> Result<DataView> {
        let mut dummy_plan = ExecutionPlanBuilder::new();
        let mut exec_context = ExecutionContext::new();
        self.build_view_with_context_and_plan_and_exec(
            table,
            statement,
            cte_context,
            &mut dummy_plan,
            &mut exec_context,
        )
    }

    /// Build a DataView from a SelectStatement with CTE context and execution plan tracking
    fn build_view_with_context_and_plan(
        &self,
        table: Arc<DataTable>,
        statement: SelectStatement,
        cte_context: &mut HashMap<String, Arc<DataView>>,
        plan: &mut ExecutionPlanBuilder,
    ) -> Result<DataView> {
        let mut exec_context = ExecutionContext::new();
        self.build_view_with_context_and_plan_and_exec(
            table,
            statement,
            cte_context,
            plan,
            &mut exec_context,
        )
    }

    /// Build a DataView with CTE context, execution plan, and alias resolution context
    fn build_view_with_context_and_plan_and_exec(
        &self,
        table: Arc<DataTable>,
        statement: SelectStatement,
        cte_context: &mut HashMap<String, Arc<DataView>>,
        plan: &mut ExecutionPlanBuilder,
        exec_context: &mut ExecutionContext,
    ) -> Result<DataView> {
        // First, process any CTEs that aren't already in the context
        for cte in &statement.ctes {
            // Skip if already processed (e.g., by execute_select for WEB CTEs)
            if cte_context.contains_key(&cte.name) {
                debug!(
                    "QueryEngine: CTE '{}' already in context, skipping",
                    cte.name
                );
                continue;
            }

            debug!("QueryEngine: Processing CTE '{}'...", cte.name);
            debug!(
                "QueryEngine: Available CTEs for '{}': {:?}",
                cte.name,
                cte_context.keys().collect::<Vec<_>>()
            );

            // Execute the CTE query (it might reference earlier CTEs)
            let cte_result = match &cte.cte_type {
                CTEType::Standard(query) => {
                    let view =
                        self.build_view_with_context(table.clone(), query.clone(), cte_context)?;

                    // Materialize the view and enrich columns with qualified names
                    let mut materialized = self.materialize_view(view)?;

                    // Enrich columns with qualified names for proper scoping
                    for column in materialized.columns_mut() {
                        column.qualified_name = Some(format!("{}.{}", cte.name, column.name));
                        column.source_table = Some(cte.name.clone());
                    }

                    DataView::new(Arc::new(materialized))
                }
                CTEType::Web(_web_spec) => {
                    // Web CTEs should have been processed earlier in execute_select
                    return Err(anyhow!(
                        "Web CTEs should be processed in execute_select method"
                    ));
                }
            };

            // Store the result in the context for later use
            cte_context.insert(cte.name.clone(), Arc::new(cte_result));
            debug!(
                "QueryEngine: CTE '{}' processed, stored in context",
                cte.name
            );
        }

        // Determine the source table for the main query
        let source_table = if let Some(ref table_func) = statement.from_function {
            // Handle table functions like RANGE()
            debug!("QueryEngine: Processing table function...");
            match table_func {
                TableFunction::Generator { name, args } => {
                    // Use the generator registry to create the table
                    use crate::sql::generators::GeneratorRegistry;

                    // Create generator registry (could be cached in QueryEngine)
                    let registry = GeneratorRegistry::new();

                    if let Some(generator) = registry.get(name) {
                        // Evaluate arguments
                        let mut evaluator = ArithmeticEvaluator::with_date_notation(
                            &table,
                            self.date_notation.clone(),
                        );
                        let dummy_row = 0;

                        let mut evaluated_args = Vec::new();
                        for arg in args {
                            evaluated_args.push(evaluator.evaluate(arg, dummy_row)?);
                        }

                        // Generate the table
                        generator.generate(evaluated_args)?
                    } else {
                        return Err(anyhow!("Unknown generator function: {}", name));
                    }
                }
            }
        } else if let Some(ref subquery) = statement.from_subquery {
            // Execute the subquery and use its result as the source
            debug!("QueryEngine: Processing FROM subquery...");
            let subquery_result =
                self.build_view_with_context(table.clone(), *subquery.clone(), cte_context)?;

            // Convert the DataView to a DataTable for use as source
            // This materializes the subquery result
            let materialized = self.materialize_view(subquery_result)?;
            Arc::new(materialized)
        } else if let Some(ref table_name) = statement.from_table {
            // Check if this references a CTE
            if let Some(cte_view) = cte_context.get(table_name) {
                debug!("QueryEngine: Using CTE '{}' as source table", table_name);
                // Materialize the CTE view as a table
                let mut materialized = self.materialize_view((**cte_view).clone())?;

                // Apply alias to qualified column names if present
                if let Some(ref alias) = statement.from_alias {
                    debug!(
                        "QueryEngine: Applying alias '{}' to CTE '{}' qualified column names",
                        alias, table_name
                    );
                    for column in materialized.columns_mut() {
                        // Replace the CTE name with the alias in qualified names
                        if let Some(ref qualified_name) = column.qualified_name {
                            if qualified_name.starts_with(&format!("{}.", table_name)) {
                                column.qualified_name =
                                    Some(qualified_name.replace(
                                        &format!("{}.", table_name),
                                        &format!("{}.", alias),
                                    ));
                            }
                        }
                        // Update source table to reflect the alias
                        if column.source_table.as_ref() == Some(table_name) {
                            column.source_table = Some(alias.clone());
                        }
                    }
                }

                Arc::new(materialized)
            } else {
                // Regular table reference - use the provided table
                table.clone()
            }
        } else {
            // No FROM clause - use the provided table
            table.clone()
        };

        // Register alias in execution context if present
        if let Some(ref alias) = statement.from_alias {
            if let Some(ref table_name) = statement.from_table {
                exec_context.register_alias(alias.clone(), table_name.clone());
            }
        }

        // Process JOINs if present
        let final_table = if !statement.joins.is_empty() {
            plan.begin_step(
                StepType::Join,
                format!("Process {} JOINs", statement.joins.len()),
            );
            plan.set_rows_in(source_table.row_count());

            let join_executor = HashJoinExecutor::new(self.case_insensitive);
            let mut current_table = source_table;

            for (idx, join_clause) in statement.joins.iter().enumerate() {
                let join_start = Instant::now();
                plan.begin_step(StepType::Join, format!("JOIN #{}", idx + 1));
                plan.add_detail(format!("Type: {:?}", join_clause.join_type));
                plan.add_detail(format!("Left table: {} rows", current_table.row_count()));
                plan.add_detail(format!(
                    "Executing {:?} JOIN on {} condition(s)",
                    join_clause.join_type,
                    join_clause.condition.conditions.len()
                ));

                // Resolve the right table for the join
                let right_table = match &join_clause.table {
                    TableSource::Table(name) => {
                        // Check if it's a CTE reference
                        if let Some(cte_view) = cte_context.get(name) {
                            let mut materialized = self.materialize_view((**cte_view).clone())?;

                            // Apply alias to qualified column names if present
                            if let Some(ref alias) = join_clause.alias {
                                debug!("QueryEngine: Applying JOIN alias '{}' to CTE '{}' qualified column names", alias, name);
                                for column in materialized.columns_mut() {
                                    // Replace the CTE name with the alias in qualified names
                                    if let Some(ref qualified_name) = column.qualified_name {
                                        if qualified_name.starts_with(&format!("{}.", name)) {
                                            column.qualified_name = Some(qualified_name.replace(
                                                &format!("{}.", name),
                                                &format!("{}.", alias),
                                            ));
                                        }
                                    }
                                    // Update source table to reflect the alias
                                    if column.source_table.as_ref() == Some(name) {
                                        column.source_table = Some(alias.clone());
                                    }
                                }
                            }

                            Arc::new(materialized)
                        } else {
                            // For now, we need the actual table data
                            // In a real implementation, this would load from file
                            return Err(anyhow!("Cannot resolve table '{}' for JOIN", name));
                        }
                    }
                    TableSource::DerivedTable { query, alias: _ } => {
                        // Execute the subquery
                        let subquery_result = self.build_view_with_context(
                            table.clone(),
                            *query.clone(),
                            cte_context,
                        )?;
                        let materialized = self.materialize_view(subquery_result)?;
                        Arc::new(materialized)
                    }
                };

                // Execute the join
                let joined = join_executor.execute_join(
                    current_table.clone(),
                    join_clause,
                    right_table.clone(),
                )?;

                plan.add_detail(format!("Right table: {} rows", right_table.row_count()));
                plan.set_rows_out(joined.row_count());
                plan.add_detail(format!("Result: {} rows", joined.row_count()));
                plan.add_detail(format!(
                    "Join time: {:.3}ms",
                    join_start.elapsed().as_secs_f64() * 1000.0
                ));
                plan.end_step();

                current_table = Arc::new(joined);
            }

            plan.set_rows_out(current_table.row_count());
            plan.add_detail(format!(
                "Final result after all joins: {} rows",
                current_table.row_count()
            ));
            plan.end_step();
            current_table
        } else {
            source_table
        };

        // Continue with the existing build_view logic but using final_table
        self.build_view_internal_with_plan_and_exec(
            final_table,
            statement,
            plan,
            Some(exec_context),
        )
    }

    /// Materialize a DataView into a new DataTable
    pub fn materialize_view(&self, view: DataView) -> Result<DataTable> {
        let source = view.source();
        let mut result_table = DataTable::new("derived");

        // Get the visible columns from the view
        let visible_cols = view.visible_column_indices().to_vec();

        // Copy column definitions
        for col_idx in &visible_cols {
            let col = &source.columns[*col_idx];
            let new_col = DataColumn {
                name: col.name.clone(),
                data_type: col.data_type.clone(),
                nullable: col.nullable,
                unique_values: col.unique_values,
                null_count: col.null_count,
                metadata: col.metadata.clone(),
                qualified_name: col.qualified_name.clone(), // Preserve qualified name
                source_table: col.source_table.clone(),     // Preserve source table
            };
            result_table.add_column(new_col);
        }

        // Copy visible rows
        for row_idx in view.visible_row_indices() {
            let source_row = &source.rows[*row_idx];
            let mut new_row = DataRow { values: Vec::new() };

            for col_idx in &visible_cols {
                new_row.values.push(source_row.values[*col_idx].clone());
            }

            result_table.add_row(new_row);
        }

        Ok(result_table)
    }

    fn build_view_internal(
        &self,
        table: Arc<DataTable>,
        statement: SelectStatement,
    ) -> Result<DataView> {
        let mut dummy_plan = ExecutionPlanBuilder::new();
        self.build_view_internal_with_plan(table, statement, &mut dummy_plan)
    }

    fn build_view_internal_with_plan(
        &self,
        table: Arc<DataTable>,
        statement: SelectStatement,
        plan: &mut ExecutionPlanBuilder,
    ) -> Result<DataView> {
        self.build_view_internal_with_plan_and_exec(table, statement, plan, None)
    }

    fn build_view_internal_with_plan_and_exec(
        &self,
        table: Arc<DataTable>,
        statement: SelectStatement,
        plan: &mut ExecutionPlanBuilder,
        exec_context: Option<&ExecutionContext>,
    ) -> Result<DataView> {
        debug!(
            "QueryEngine::build_view - select_items: {:?}",
            statement.select_items
        );
        debug!(
            "QueryEngine::build_view - where_clause: {:?}",
            statement.where_clause
        );

        // Start with all rows visible
        let mut visible_rows: Vec<usize> = (0..table.row_count()).collect();

        // Apply WHERE clause filtering using recursive evaluator
        if let Some(where_clause) = &statement.where_clause {
            let total_rows = table.row_count();
            debug!("QueryEngine: Applying WHERE clause to {} rows", total_rows);
            debug!("QueryEngine: WHERE clause = {:?}", where_clause);

            plan.begin_step(StepType::Filter, "WHERE clause filtering".to_string());
            plan.set_rows_in(total_rows);
            plan.add_detail(format!("Input: {} rows", total_rows));

            // Add details about WHERE conditions
            for condition in &where_clause.conditions {
                plan.add_detail(format!("Condition: {:?}", condition.expr));
            }

            let filter_start = Instant::now();
            // Create an evaluation context for caching compiled regexes
            let mut eval_context = EvaluationContext::new(self.case_insensitive);

            // Create evaluator ONCE before the loop for performance
            let mut evaluator = if let Some(exec_ctx) = exec_context {
                // Use both contexts: exec_context for alias resolution, eval_context for regex caching
                RecursiveWhereEvaluator::with_both_contexts(&table, &mut eval_context, exec_ctx)
            } else {
                RecursiveWhereEvaluator::with_context(&table, &mut eval_context)
            };

            // Filter visible rows based on WHERE clause
            let mut filtered_rows = Vec::new();
            for row_idx in visible_rows {
                // Only log for first few rows to avoid performance impact
                if row_idx < 3 {
                    debug!("QueryEngine: Evaluating WHERE clause for row {}", row_idx);
                }

                match evaluator.evaluate(where_clause, row_idx) {
                    Ok(result) => {
                        if row_idx < 3 {
                            debug!("QueryEngine: Row {} WHERE result: {}", row_idx, result);
                        }
                        if result {
                            filtered_rows.push(row_idx);
                        }
                    }
                    Err(e) => {
                        if row_idx < 3 {
                            debug!(
                                "QueryEngine: WHERE evaluation error for row {}: {}",
                                row_idx, e
                            );
                        }
                        // Propagate WHERE clause errors instead of silently ignoring them
                        return Err(e);
                    }
                }
            }

            // Log regex cache statistics
            let (compilations, cache_hits) = eval_context.get_stats();
            if compilations > 0 || cache_hits > 0 {
                debug!(
                    "LIKE pattern cache: {} compilations, {} cache hits",
                    compilations, cache_hits
                );
            }
            visible_rows = filtered_rows;
            let filter_duration = filter_start.elapsed();
            info!(
                "WHERE clause filtering: {} rows -> {} rows in {:?}",
                total_rows,
                visible_rows.len(),
                filter_duration
            );

            plan.set_rows_out(visible_rows.len());
            plan.add_detail(format!("Output: {} rows", visible_rows.len()));
            plan.add_detail(format!(
                "Filter time: {:.3}ms",
                filter_duration.as_secs_f64() * 1000.0
            ));
            plan.end_step();
        }

        // Create initial DataView with filtered rows
        let mut view = DataView::new(table.clone());
        view = view.with_rows(visible_rows);

        // Handle GROUP BY if present
        if let Some(group_by_exprs) = &statement.group_by {
            if !group_by_exprs.is_empty() {
                debug!("QueryEngine: Processing GROUP BY: {:?}", group_by_exprs);

                plan.begin_step(
                    StepType::GroupBy,
                    format!("GROUP BY {} expressions", group_by_exprs.len()),
                );
                plan.set_rows_in(view.row_count());
                plan.add_detail(format!("Input: {} rows", view.row_count()));
                for expr in group_by_exprs {
                    plan.add_detail(format!("Group by: {:?}", expr));
                }

                let group_start = Instant::now();
                view = self.apply_group_by(
                    view,
                    group_by_exprs,
                    &statement.select_items,
                    statement.having.as_ref(),
                    plan,
                )?;

                plan.set_rows_out(view.row_count());
                plan.add_detail(format!("Output: {} groups", view.row_count()));
                plan.add_detail(format!(
                    "Overall time: {:.3}ms",
                    group_start.elapsed().as_secs_f64() * 1000.0
                ));
                plan.end_step();
            }
        } else {
            // Apply column projection or computed expressions (SELECT clause) - do this AFTER filtering
            if !statement.select_items.is_empty() {
                // Check if we have ANY non-star items (not just the first one)
                let has_non_star_items = statement
                    .select_items
                    .iter()
                    .any(|item| !matches!(item, SelectItem::Star));

                // Apply select items if:
                // 1. We have computed expressions or explicit columns
                // 2. OR we have a mix of star and other items (e.g., SELECT *, computed_col)
                if has_non_star_items || statement.select_items.len() > 1 {
                    view = self.apply_select_items(
                        view,
                        &statement.select_items,
                        &statement,
                        exec_context,
                    )?;
                }
                // If it's just a single star, no projection needed
            } else if !statement.columns.is_empty() && statement.columns[0] != "*" {
                debug!("QueryEngine: Using legacy columns path");
                // Fallback to legacy column projection for backward compatibility
                // Use the current view's source table, not the original table
                let source_table = view.source();
                let column_indices =
                    self.resolve_column_indices(source_table, &statement.columns)?;
                view = view.with_columns(column_indices);
            }
        }

        // Apply DISTINCT if specified
        if statement.distinct {
            plan.begin_step(StepType::Distinct, "Remove duplicate rows".to_string());
            plan.set_rows_in(view.row_count());
            plan.add_detail(format!("Input: {} rows", view.row_count()));

            let distinct_start = Instant::now();
            view = self.apply_distinct(view)?;

            plan.set_rows_out(view.row_count());
            plan.add_detail(format!("Output: {} unique rows", view.row_count()));
            plan.add_detail(format!(
                "Distinct time: {:.3}ms",
                distinct_start.elapsed().as_secs_f64() * 1000.0
            ));
            plan.end_step();
        }

        // Apply ORDER BY sorting
        if let Some(order_by_columns) = &statement.order_by {
            if !order_by_columns.is_empty() {
                plan.begin_step(
                    StepType::Sort,
                    format!("ORDER BY {} columns", order_by_columns.len()),
                );
                plan.set_rows_in(view.row_count());
                for col in order_by_columns {
                    plan.add_detail(format!("{} {:?}", col.column, col.direction));
                }

                let sort_start = Instant::now();
                view =
                    self.apply_multi_order_by_with_context(view, order_by_columns, exec_context)?;

                plan.add_detail(format!(
                    "Sort time: {:.3}ms",
                    sort_start.elapsed().as_secs_f64() * 1000.0
                ));
                plan.end_step();
            }
        }

        // Apply LIMIT/OFFSET
        if let Some(limit) = statement.limit {
            let offset = statement.offset.unwrap_or(0);
            plan.begin_step(StepType::Limit, format!("LIMIT {}", limit));
            plan.set_rows_in(view.row_count());
            if offset > 0 {
                plan.add_detail(format!("OFFSET: {}", offset));
            }
            view = view.with_limit(limit, offset);
            plan.set_rows_out(view.row_count());
            plan.add_detail(format!("Output: {} rows", view.row_count()));
            plan.end_step();
        }

        // Process set operations (UNION ALL, UNION, INTERSECT, EXCEPT)
        if !statement.set_operations.is_empty() {
            plan.begin_step(
                StepType::SetOperation,
                format!("Process {} set operations", statement.set_operations.len()),
            );
            plan.set_rows_in(view.row_count());

            // Materialize the first result set
            let mut combined_table = self.materialize_view(view)?;
            let first_columns = combined_table.column_names();
            let first_column_count = first_columns.len();

            // Track if any operation requires deduplication
            let mut needs_deduplication = false;

            // Process each set operation
            for (idx, (operation, next_statement)) in statement.set_operations.iter().enumerate() {
                let op_start = Instant::now();
                plan.begin_step(
                    StepType::SetOperation,
                    format!("{:?} operation #{}", operation, idx + 1),
                );

                // Execute the next SELECT statement
                // We need to pass the original table and exec_context for proper resolution
                let next_view = if let Some(exec_ctx) = exec_context {
                    self.build_view_internal_with_plan_and_exec(
                        table.clone(),
                        *next_statement.clone(),
                        plan,
                        Some(exec_ctx),
                    )?
                } else {
                    self.build_view_internal_with_plan(
                        table.clone(),
                        *next_statement.clone(),
                        plan,
                    )?
                };

                // Materialize the next result set
                let next_table = self.materialize_view(next_view)?;
                let next_columns = next_table.column_names();
                let next_column_count = next_columns.len();

                // Validate schema compatibility
                if first_column_count != next_column_count {
                    return Err(anyhow!(
                        "UNION queries must have the same number of columns: first query has {} columns, but query #{} has {} columns",
                        first_column_count,
                        idx + 2,
                        next_column_count
                    ));
                }

                // Warn if column names don't match (but allow it - some SQL dialects do)
                for (col_idx, (first_col, next_col)) in
                    first_columns.iter().zip(next_columns.iter()).enumerate()
                {
                    if !first_col.eq_ignore_ascii_case(next_col) {
                        debug!(
                            "UNION column name mismatch at position {}: '{}' vs '{}' (using first query's name)",
                            col_idx + 1,
                            first_col,
                            next_col
                        );
                    }
                }

                plan.add_detail(format!("Left: {} rows", combined_table.row_count()));
                plan.add_detail(format!("Right: {} rows", next_table.row_count()));

                // Perform the set operation
                match operation {
                    SetOperation::UnionAll => {
                        // UNION ALL: Simply concatenate all rows without deduplication
                        for row in next_table.rows.iter() {
                            combined_table.add_row(row.clone());
                        }
                        plan.add_detail(format!(
                            "Result: {} rows (no deduplication)",
                            combined_table.row_count()
                        ));
                    }
                    SetOperation::Union => {
                        // UNION: Concatenate all rows first, deduplicate at the end
                        for row in next_table.rows.iter() {
                            combined_table.add_row(row.clone());
                        }
                        needs_deduplication = true;
                        plan.add_detail(format!(
                            "Combined: {} rows (deduplication pending)",
                            combined_table.row_count()
                        ));
                    }
                    SetOperation::Intersect => {
                        // INTERSECT: Keep only rows that appear in both
                        // TODO: Implement intersection logic
                        return Err(anyhow!("INTERSECT is not yet implemented"));
                    }
                    SetOperation::Except => {
                        // EXCEPT: Keep only rows from left that don't appear in right
                        // TODO: Implement except logic
                        return Err(anyhow!("EXCEPT is not yet implemented"));
                    }
                }

                plan.add_detail(format!(
                    "Operation time: {:.3}ms",
                    op_start.elapsed().as_secs_f64() * 1000.0
                ));
                plan.set_rows_out(combined_table.row_count());
                plan.end_step();
            }

            plan.set_rows_out(combined_table.row_count());
            plan.add_detail(format!(
                "Combined result: {} rows after {} operations",
                combined_table.row_count(),
                statement.set_operations.len()
            ));
            plan.end_step();

            // Create a new view from the combined table
            view = DataView::new(Arc::new(combined_table));

            // Apply deduplication if any UNION (not UNION ALL) operation was used
            if needs_deduplication {
                plan.begin_step(
                    StepType::Distinct,
                    "UNION deduplication - remove duplicate rows".to_string(),
                );
                plan.set_rows_in(view.row_count());
                plan.add_detail(format!("Input: {} rows", view.row_count()));

                let distinct_start = Instant::now();
                view = self.apply_distinct(view)?;

                plan.set_rows_out(view.row_count());
                plan.add_detail(format!("Output: {} unique rows", view.row_count()));
                plan.add_detail(format!(
                    "Deduplication time: {:.3}ms",
                    distinct_start.elapsed().as_secs_f64() * 1000.0
                ));
                plan.end_step();
            }
        }

        Ok(view)
    }

    /// Resolve column names to indices
    fn resolve_column_indices(&self, table: &DataTable, columns: &[String]) -> Result<Vec<usize>> {
        let mut indices = Vec::new();
        let table_columns = table.column_names();

        for col_name in columns {
            let index = table_columns
                .iter()
                .position(|c| c.eq_ignore_ascii_case(col_name))
                .ok_or_else(|| {
                    let suggestion = self.find_similar_column(table, col_name);
                    match suggestion {
                        Some(similar) => anyhow::anyhow!(
                            "Column '{}' not found. Did you mean '{}'?",
                            col_name,
                            similar
                        ),
                        None => anyhow::anyhow!("Column '{}' not found", col_name),
                    }
                })?;
            indices.push(index);
        }

        Ok(indices)
    }

    /// Apply SELECT items (columns and computed expressions) to create new view
    fn apply_select_items(
        &self,
        view: DataView,
        select_items: &[SelectItem],
        _statement: &SelectStatement,
        exec_context: Option<&ExecutionContext>,
    ) -> Result<DataView> {
        debug!(
            "QueryEngine::apply_select_items - items: {:?}",
            select_items
        );
        debug!(
            "QueryEngine::apply_select_items - input view has {} rows",
            view.row_count()
        );

        // Check if any SELECT item contains UNNEST - if so, use row expansion mode
        let has_unnest = select_items.iter().any(|item| match item {
            SelectItem::Expression { expr, .. } => Self::contains_unnest(expr),
            _ => false,
        });

        if has_unnest {
            debug!("QueryEngine::apply_select_items - UNNEST detected, using row expansion");
            return self.apply_select_with_row_expansion(view, select_items);
        }

        // Check if this is an aggregate query:
        // 1. At least one aggregate function exists
        // 2. All other items are either aggregates or constants (aggregate-compatible)
        let has_aggregates = select_items.iter().any(|item| match item {
            SelectItem::Expression { expr, .. } => contains_aggregate(expr),
            SelectItem::Column(_) => false,
            SelectItem::Star => false,
        });

        let all_aggregate_compatible = select_items.iter().all(|item| match item {
            SelectItem::Expression { expr, .. } => is_aggregate_compatible(expr),
            SelectItem::Column(_) => false, // Columns are not aggregate-compatible
            SelectItem::Star => false,      // Star is not aggregate-compatible
        });

        if has_aggregates && all_aggregate_compatible && view.row_count() > 0 {
            // Special handling for aggregate queries with constants (no GROUP BY)
            // These should produce exactly one row
            debug!("QueryEngine::apply_select_items - detected aggregate query with constants");
            return self.apply_aggregate_select(view, select_items);
        }

        // Check if we need to create computed columns
        let has_computed_expressions = select_items
            .iter()
            .any(|item| matches!(item, SelectItem::Expression { .. }));

        debug!(
            "QueryEngine::apply_select_items - has_computed_expressions: {}",
            has_computed_expressions
        );

        if !has_computed_expressions {
            // Simple case: only columns, use existing projection logic
            let column_indices = self.resolve_select_columns(view.source(), select_items)?;
            return Ok(view.with_columns(column_indices));
        }

        // Complex case: we have computed expressions
        // IMPORTANT: We create a PROJECTED view, not a new table
        // This preserves the original DataTable reference

        let source_table = view.source();
        let visible_rows = view.visible_row_indices();

        // Create a temporary table just for the computed result view
        // But this table is only used for the current query result
        let mut computed_table = DataTable::new("query_result");

        // First, expand any Star selectors to actual columns
        let mut expanded_items = Vec::new();
        for item in select_items {
            match item {
                SelectItem::Star => {
                    // Expand * to all columns from source table
                    for col_name in source_table.column_names() {
                        expanded_items.push(SelectItem::Column(ColumnRef::unquoted(
                            col_name.to_string(),
                        )));
                    }
                }
                _ => expanded_items.push(item.clone()),
            }
        }

        // Add columns based on expanded SelectItems, handling duplicates
        let mut column_name_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();

        for item in &expanded_items {
            let base_name = match item {
                SelectItem::Column(col_ref) => col_ref.name.clone(),
                SelectItem::Expression { alias, .. } => alias.clone(),
                SelectItem::Star => unreachable!("Star should have been expanded"),
            };

            // Check if this column name has been used before
            let count = column_name_counts.entry(base_name.clone()).or_insert(0);
            let column_name = if *count == 0 {
                // First occurrence, use the name as-is
                base_name.clone()
            } else {
                // Duplicate, append a suffix
                format!("{base_name}_{count}")
            };
            *count += 1;

            computed_table.add_column(DataColumn::new(&column_name));
        }

        // Calculate values for each row
        let mut evaluator =
            ArithmeticEvaluator::with_date_notation(source_table, self.date_notation.clone());

        // Populate table aliases from exec_context if available
        if let Some(exec_ctx) = exec_context {
            let aliases = exec_ctx.get_aliases();
            if !aliases.is_empty() {
                debug!(
                    "Applying {} aliases to evaluator: {:?}",
                    aliases.len(),
                    aliases
                );
                evaluator = evaluator.with_table_aliases(aliases);
            }
        }

        for &row_idx in visible_rows {
            let mut row_values = Vec::new();

            for item in &expanded_items {
                let value = match item {
                    SelectItem::Column(col_ref) => {
                        // Use evaluator for column resolution (handles aliases properly)
                        match evaluator.evaluate(&SqlExpression::Column(col_ref.clone()), row_idx) {
                            Ok(val) => val,
                            Err(e) => {
                                return Err(anyhow!(
                                    "Failed to evaluate column {}: {}",
                                    col_ref.to_sql(),
                                    e
                                ));
                            }
                        }
                    }
                    SelectItem::Expression { expr, .. } => {
                        // Computed expression
                        evaluator.evaluate(expr, row_idx)?
                    }
                    SelectItem::Star => unreachable!("Star should have been expanded"),
                };
                row_values.push(value);
            }

            computed_table
                .add_row(DataRow::new(row_values))
                .map_err(|e| anyhow::anyhow!("Failed to add row: {}", e))?;
        }

        // Return a view of the computed result
        // This is a temporary view for this query only
        Ok(DataView::new(Arc::new(computed_table)))
    }

    /// Apply SELECT with row expansion (for UNNEST, EXPLODE, etc.)
    fn apply_select_with_row_expansion(
        &self,
        view: DataView,
        select_items: &[SelectItem],
    ) -> Result<DataView> {
        debug!("QueryEngine::apply_select_with_row_expansion - expanding rows");

        let source_table = view.source();
        let visible_rows = view.visible_row_indices();
        let expander_registry = RowExpanderRegistry::new();

        // Create result table
        let mut result_table = DataTable::new("unnest_result");

        // Expand * to columns and set up result columns
        let mut expanded_items = Vec::new();
        for item in select_items {
            match item {
                SelectItem::Star => {
                    for col_name in source_table.column_names() {
                        expanded_items.push(SelectItem::Column(ColumnRef::unquoted(
                            col_name.to_string(),
                        )));
                    }
                }
                _ => expanded_items.push(item.clone()),
            }
        }

        // Add columns to result table
        for item in &expanded_items {
            let column_name = match item {
                SelectItem::Column(col_ref) => col_ref.name.clone(),
                SelectItem::Expression { alias, .. } => alias.clone(),
                SelectItem::Star => unreachable!("Star should have been expanded"),
            };
            result_table.add_column(DataColumn::new(&column_name));
        }

        // Process each input row
        let mut evaluator =
            ArithmeticEvaluator::with_date_notation(source_table, self.date_notation.clone());

        for &row_idx in visible_rows {
            // First pass: identify UNNEST expressions and collect their expansion arrays
            let mut unnest_expansions = Vec::new();
            let mut unnest_indices = Vec::new();

            for (col_idx, item) in expanded_items.iter().enumerate() {
                if let SelectItem::Expression { expr, .. } = item {
                    if let Some(expansion_result) = self.try_expand_unnest(
                        expr,
                        source_table,
                        row_idx,
                        &mut evaluator,
                        &expander_registry,
                    )? {
                        unnest_expansions.push(expansion_result);
                        unnest_indices.push(col_idx);
                    }
                }
            }

            // Determine how many output rows to generate
            let expansion_count = if unnest_expansions.is_empty() {
                1 // No UNNEST, just one row
            } else {
                unnest_expansions
                    .iter()
                    .map(|exp| exp.row_count())
                    .max()
                    .unwrap_or(1)
            };

            // Generate output rows
            for output_idx in 0..expansion_count {
                let mut row_values = Vec::new();

                for (col_idx, item) in expanded_items.iter().enumerate() {
                    // Check if this column is an UNNEST column
                    let unnest_position = unnest_indices.iter().position(|&idx| idx == col_idx);

                    let value = if let Some(unnest_idx) = unnest_position {
                        // Get value from expansion array (or NULL if exhausted)
                        let expansion = &unnest_expansions[unnest_idx];
                        expansion
                            .values
                            .get(output_idx)
                            .cloned()
                            .unwrap_or(DataValue::Null)
                    } else {
                        // Regular column or non-UNNEST expression - replicate from input
                        match item {
                            SelectItem::Column(col_ref) => {
                                let col_idx =
                                    source_table.get_column_index(&col_ref.name).ok_or_else(
                                        || anyhow::anyhow!("Column '{}' not found", col_ref.name),
                                    )?;
                                let row = source_table
                                    .get_row(row_idx)
                                    .ok_or_else(|| anyhow::anyhow!("Row {} not found", row_idx))?;
                                row.get(col_idx)
                                    .ok_or_else(|| {
                                        anyhow::anyhow!("Column {} not found in row", col_idx)
                                    })?
                                    .clone()
                            }
                            SelectItem::Expression { expr, .. } => {
                                // Non-UNNEST expression - evaluate once and replicate
                                evaluator.evaluate(expr, row_idx)?
                            }
                            SelectItem::Star => unreachable!(),
                        }
                    };

                    row_values.push(value);
                }

                result_table
                    .add_row(DataRow::new(row_values))
                    .map_err(|e| anyhow::anyhow!("Failed to add expanded row: {}", e))?;
            }
        }

        debug!(
            "QueryEngine::apply_select_with_row_expansion - input rows: {}, output rows: {}",
            visible_rows.len(),
            result_table.row_count()
        );

        Ok(DataView::new(Arc::new(result_table)))
    }

    /// Try to expand an expression if it's an UNNEST call
    /// Returns Some(ExpansionResult) if successful, None if not an UNNEST
    fn try_expand_unnest(
        &self,
        expr: &SqlExpression,
        _source_table: &DataTable,
        row_idx: usize,
        evaluator: &mut ArithmeticEvaluator,
        expander_registry: &RowExpanderRegistry,
    ) -> Result<Option<crate::data::row_expanders::ExpansionResult>> {
        // Check for UNNEST variant (direct syntax)
        if let SqlExpression::Unnest { column, delimiter } = expr {
            // Evaluate the column expression
            let column_value = evaluator.evaluate(column, row_idx)?;

            // Delimiter is already a string literal
            let delimiter_value = DataValue::String(delimiter.clone());

            // Get the UNNEST expander
            let expander = expander_registry
                .get("UNNEST")
                .ok_or_else(|| anyhow::anyhow!("UNNEST expander not found"))?;

            // Expand the value
            let expansion = expander.expand(&column_value, &[delimiter_value])?;
            return Ok(Some(expansion));
        }

        // Also check for FunctionCall form (for compatibility)
        if let SqlExpression::FunctionCall { name, args, .. } = expr {
            if name.to_uppercase() == "UNNEST" {
                // UNNEST(column, delimiter)
                if args.len() != 2 {
                    return Err(anyhow::anyhow!(
                        "UNNEST requires exactly 2 arguments: UNNEST(column, delimiter)"
                    ));
                }

                // Evaluate the column expression (first arg)
                let column_value = evaluator.evaluate(&args[0], row_idx)?;

                // Evaluate the delimiter expression (second arg)
                let delimiter_value = evaluator.evaluate(&args[1], row_idx)?;

                // Get the UNNEST expander
                let expander = expander_registry
                    .get("UNNEST")
                    .ok_or_else(|| anyhow::anyhow!("UNNEST expander not found"))?;

                // Expand the value
                let expansion = expander.expand(&column_value, &[delimiter_value])?;
                return Ok(Some(expansion));
            }
        }

        Ok(None)
    }

    /// Apply aggregate-only SELECT (no GROUP BY - produces single row)
    fn apply_aggregate_select(
        &self,
        view: DataView,
        select_items: &[SelectItem],
    ) -> Result<DataView> {
        debug!("QueryEngine::apply_aggregate_select - creating single row aggregate result");

        let source_table = view.source();
        let mut result_table = DataTable::new("aggregate_result");

        // Add columns for each select item
        for item in select_items {
            let column_name = match item {
                SelectItem::Expression { alias, .. } => alias.clone(),
                _ => unreachable!("Should only have expressions in aggregate-only query"),
            };
            result_table.add_column(DataColumn::new(&column_name));
        }

        // Create evaluator with visible rows from the view (for filtered aggregates)
        let visible_rows = view.visible_row_indices().to_vec();
        let mut evaluator =
            ArithmeticEvaluator::with_date_notation(source_table, self.date_notation.clone())
                .with_visible_rows(visible_rows);

        // Evaluate each aggregate expression once (they handle all rows internally)
        let mut row_values = Vec::new();
        for item in select_items {
            match item {
                SelectItem::Expression { expr, .. } => {
                    // The evaluator will handle aggregates over all rows
                    // We pass row_index=0 but aggregates ignore it and process all rows
                    let value = evaluator.evaluate(expr, 0)?;
                    row_values.push(value);
                }
                _ => unreachable!("Should only have expressions in aggregate-only query"),
            }
        }

        // Add the single result row
        result_table
            .add_row(DataRow::new(row_values))
            .map_err(|e| anyhow::anyhow!("Failed to add aggregate result row: {}", e))?;

        Ok(DataView::new(Arc::new(result_table)))
    }

    /// Resolve `SelectItem` columns to indices (for simple column projections only)
    fn resolve_select_columns(
        &self,
        table: &DataTable,
        select_items: &[SelectItem],
    ) -> Result<Vec<usize>> {
        let mut indices = Vec::new();
        let table_columns = table.column_names();

        for item in select_items {
            match item {
                SelectItem::Column(col_ref) => {
                    // Check if this has a table prefix
                    let index = if let Some(table_prefix) = &col_ref.table_prefix {
                        // For qualified references, ONLY try qualified lookup - no fallback
                        let qualified_name = format!("{}.{}", table_prefix, col_ref.name);
                        table.find_column_by_qualified_name(&qualified_name)
                            .ok_or_else(|| {
                                // Check if any columns have qualified names for better error message
                                let has_qualified = table.columns.iter()
                                    .any(|c| c.qualified_name.is_some());
                                if !has_qualified {
                                    anyhow::anyhow!(
                                        "Column '{}' not found. Note: Table '{}' may not support qualified column names",
                                        qualified_name, table_prefix
                                    )
                                } else {
                                    anyhow::anyhow!("Column '{}' not found", qualified_name)
                                }
                            })?
                    } else {
                        // Simple column name lookup
                        table_columns
                            .iter()
                            .position(|c| c.eq_ignore_ascii_case(&col_ref.name))
                            .ok_or_else(|| {
                                let suggestion = self.find_similar_column(table, &col_ref.name);
                                match suggestion {
                                    Some(similar) => anyhow::anyhow!(
                                        "Column '{}' not found. Did you mean '{}'?",
                                        col_ref.name,
                                        similar
                                    ),
                                    None => anyhow::anyhow!("Column '{}' not found", col_ref.name),
                                }
                            })?
                    };
                    indices.push(index);
                }
                SelectItem::Star => {
                    // Expand * to all column indices
                    for i in 0..table_columns.len() {
                        indices.push(i);
                    }
                }
                SelectItem::Expression { .. } => {
                    return Err(anyhow::anyhow!(
                        "Computed expressions require new table creation"
                    ));
                }
            }
        }

        Ok(indices)
    }

    /// Apply DISTINCT to remove duplicate rows
    fn apply_distinct(&self, view: DataView) -> Result<DataView> {
        use std::collections::HashSet;

        let source = view.source();
        let visible_cols = view.visible_column_indices();
        let visible_rows = view.visible_row_indices();

        // Build a set to track unique rows
        let mut seen_rows = HashSet::new();
        let mut unique_row_indices = Vec::new();

        for &row_idx in visible_rows {
            // Build a key representing this row's visible column values
            let mut row_key = Vec::new();
            for &col_idx in visible_cols {
                let value = source
                    .get_value(row_idx, col_idx)
                    .ok_or_else(|| anyhow!("Invalid cell reference"))?;
                // Convert value to a hashable representation
                row_key.push(format!("{:?}", value));
            }

            // Check if we've seen this row before
            if seen_rows.insert(row_key) {
                // First time seeing this row combination
                unique_row_indices.push(row_idx);
            }
        }

        // Create a new view with only unique rows
        Ok(view.with_rows(unique_row_indices))
    }

    /// Apply multi-column ORDER BY sorting to the view
    fn apply_multi_order_by(
        &self,
        view: DataView,
        order_by_columns: &[OrderByColumn],
    ) -> Result<DataView> {
        self.apply_multi_order_by_with_context(view, order_by_columns, None)
    }

    /// Apply multi-column ORDER BY sorting with exec_context for alias resolution
    fn apply_multi_order_by_with_context(
        &self,
        mut view: DataView,
        order_by_columns: &[OrderByColumn],
        _exec_context: Option<&ExecutionContext>,
    ) -> Result<DataView> {
        // Build list of (source_column_index, ascending) tuples
        let mut sort_columns = Vec::new();

        for order_col in order_by_columns {
            // Try to find the column index, handling qualified column names (table.column)
            let col_index = if order_col.column.contains('.') {
                // Qualified column name - extract unqualified part
                if let Some(dot_pos) = order_col.column.rfind('.') {
                    let col_name = &order_col.column[dot_pos + 1..];

                    // After SELECT processing, columns are unqualified
                    // So just use the column name part
                    debug!(
                        "ORDER BY: Extracting unqualified column '{}' from '{}'",
                        col_name, order_col.column
                    );
                    view.source().get_column_index(col_name)
                } else {
                    view.source().get_column_index(&order_col.column)
                }
            } else {
                // Simple column name
                view.source().get_column_index(&order_col.column)
            }
            .ok_or_else(|| {
                // If not found, provide helpful error with suggestions
                let suggestion = self.find_similar_column(view.source(), &order_col.column);
                match suggestion {
                    Some(similar) => anyhow::anyhow!(
                        "Column '{}' not found. Did you mean '{}'?",
                        order_col.column,
                        similar
                    ),
                    None => {
                        // Also list available columns for debugging
                        let available_cols = view.source().column_names().join(", ");
                        anyhow::anyhow!(
                            "Column '{}' not found. Available columns: {}",
                            order_col.column,
                            available_cols
                        )
                    }
                }
            })?;

            let ascending = matches!(order_col.direction, SortDirection::Asc);
            sort_columns.push((col_index, ascending));
        }

        // Apply multi-column sorting
        view.apply_multi_sort(&sort_columns)?;
        Ok(view)
    }

    /// Apply GROUP BY to the view with optional HAVING clause
    fn apply_group_by(
        &self,
        view: DataView,
        group_by_exprs: &[SqlExpression],
        select_items: &[SelectItem],
        having: Option<&SqlExpression>,
        plan: &mut ExecutionPlanBuilder,
    ) -> Result<DataView> {
        // Use the new expression-based GROUP BY implementation
        let (result_view, phase_info) = self.apply_group_by_expressions(
            view,
            group_by_exprs,
            select_items,
            having,
            self.case_insensitive,
            self.date_notation.clone(),
        )?;

        // Add detailed phase information to the execution plan
        plan.add_detail(format!("=== GROUP BY Phase Breakdown ==="));
        plan.add_detail(format!(
            "Phase 1 - Group Building: {:.3}ms",
            phase_info.phase2_key_building.as_secs_f64() * 1000.0
        ));
        plan.add_detail(format!(
            "  • Processing {} rows into {} groups",
            phase_info.total_rows, phase_info.num_groups
        ));
        plan.add_detail(format!(
            "Phase 2 - Aggregation: {:.3}ms",
            phase_info.phase4_aggregation.as_secs_f64() * 1000.0
        ));
        if phase_info.phase4_having_evaluation > Duration::ZERO {
            plan.add_detail(format!(
                "Phase 3 - HAVING Filter: {:.3}ms",
                phase_info.phase4_having_evaluation.as_secs_f64() * 1000.0
            ));
            plan.add_detail(format!(
                "  • Filtered {} groups",
                phase_info.groups_filtered_by_having
            ));
        }
        plan.add_detail(format!(
            "Total GROUP BY time: {:.3}ms",
            phase_info.total_time.as_secs_f64() * 1000.0
        ));

        Ok(result_view)
    }

    /// Estimate the cardinality (number of unique groups) for GROUP BY operations
    /// This helps pre-size hash tables for better performance
    pub fn estimate_group_cardinality(
        &self,
        view: &DataView,
        group_by_exprs: &[SqlExpression],
    ) -> usize {
        // If we have few rows, just return the row count as upper bound
        let row_count = view.get_visible_rows().len();
        if row_count <= 100 {
            return row_count;
        }

        // Sample first 1000 rows or 10% of data, whichever is smaller
        let sample_size = min(1000, row_count / 10).max(100);
        let mut seen = FxHashSet::default();

        let visible_rows = view.get_visible_rows();
        for (i, &row_idx) in visible_rows.iter().enumerate() {
            if i >= sample_size {
                break;
            }

            // Evaluate GROUP BY expressions for this row
            let mut key_values = Vec::new();
            for expr in group_by_exprs {
                let mut evaluator = ArithmeticEvaluator::new(view.source());
                let value = evaluator.evaluate(expr, row_idx).unwrap_or(DataValue::Null);
                key_values.push(value);
            }

            seen.insert(key_values);
        }

        // Estimate total cardinality based on sample
        let sample_cardinality = seen.len();
        let estimated = (sample_cardinality * row_count) / sample_size;

        // Cap at row count and ensure minimum of sample cardinality
        estimated.min(row_count).max(sample_cardinality)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::datatable::{DataColumn, DataRow, DataValue};

    fn create_test_table() -> Arc<DataTable> {
        let mut table = DataTable::new("test");

        // Add columns
        table.add_column(DataColumn::new("id"));
        table.add_column(DataColumn::new("name"));
        table.add_column(DataColumn::new("age"));

        // Add rows
        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(1),
                DataValue::String("Alice".to_string()),
                DataValue::Integer(30),
            ]))
            .unwrap();

        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(2),
                DataValue::String("Bob".to_string()),
                DataValue::Integer(25),
            ]))
            .unwrap();

        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(3),
                DataValue::String("Charlie".to_string()),
                DataValue::Integer(35),
            ]))
            .unwrap();

        Arc::new(table)
    }

    #[test]
    fn test_select_all() {
        let table = create_test_table();
        let engine = QueryEngine::new();

        let view = engine
            .execute(table.clone(), "SELECT * FROM users")
            .unwrap();
        assert_eq!(view.row_count(), 3);
        assert_eq!(view.column_count(), 3);
    }

    #[test]
    fn test_select_columns() {
        let table = create_test_table();
        let engine = QueryEngine::new();

        let view = engine
            .execute(table.clone(), "SELECT name, age FROM users")
            .unwrap();
        assert_eq!(view.row_count(), 3);
        assert_eq!(view.column_count(), 2);
    }

    #[test]
    fn test_select_with_limit() {
        let table = create_test_table();
        let engine = QueryEngine::new();

        let view = engine
            .execute(table.clone(), "SELECT * FROM users LIMIT 2")
            .unwrap();
        assert_eq!(view.row_count(), 2);
    }

    #[test]
    fn test_type_coercion_contains() {
        // Initialize tracing for debug output
        let _ = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .try_init();

        let mut table = DataTable::new("test");
        table.add_column(DataColumn::new("id"));
        table.add_column(DataColumn::new("status"));
        table.add_column(DataColumn::new("price"));

        // Add test data with mixed types
        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(1),
                DataValue::String("Pending".to_string()),
                DataValue::Float(99.99),
            ]))
            .unwrap();

        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(2),
                DataValue::String("Confirmed".to_string()),
                DataValue::Float(150.50),
            ]))
            .unwrap();

        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(3),
                DataValue::String("Pending".to_string()),
                DataValue::Float(75.00),
            ]))
            .unwrap();

        let table = Arc::new(table);
        let engine = QueryEngine::new();

        println!("\n=== Testing WHERE clause with Contains ===");
        println!("Table has {} rows", table.row_count());
        for i in 0..table.row_count() {
            let status = table.get_value(i, 1);
            println!("Row {i}: status = {status:?}");
        }

        // Test 1: Basic string contains (should work)
        println!("\n--- Test 1: status.Contains('pend') ---");
        let result = engine.execute(
            table.clone(),
            "SELECT * FROM test WHERE status.Contains('pend')",
        );
        match result {
            Ok(view) => {
                println!("SUCCESS: Found {} matching rows", view.row_count());
                assert_eq!(view.row_count(), 2); // Should find both Pending rows
            }
            Err(e) => {
                panic!("Query failed: {e}");
            }
        }

        // Test 2: Numeric contains (should work with type coercion)
        println!("\n--- Test 2: price.Contains('9') ---");
        let result = engine.execute(
            table.clone(),
            "SELECT * FROM test WHERE price.Contains('9')",
        );
        match result {
            Ok(view) => {
                println!(
                    "SUCCESS: Found {} matching rows with price containing '9'",
                    view.row_count()
                );
                // Should find 99.99 row
                assert!(view.row_count() >= 1);
            }
            Err(e) => {
                panic!("Numeric coercion query failed: {e}");
            }
        }

        println!("\n=== All tests passed! ===");
    }

    #[test]
    fn test_not_in_clause() {
        // Initialize tracing for debug output
        let _ = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .try_init();

        let mut table = DataTable::new("test");
        table.add_column(DataColumn::new("id"));
        table.add_column(DataColumn::new("country"));

        // Add test data
        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(1),
                DataValue::String("CA".to_string()),
            ]))
            .unwrap();

        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(2),
                DataValue::String("US".to_string()),
            ]))
            .unwrap();

        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(3),
                DataValue::String("UK".to_string()),
            ]))
            .unwrap();

        let table = Arc::new(table);
        let engine = QueryEngine::new();

        println!("\n=== Testing NOT IN clause ===");
        println!("Table has {} rows", table.row_count());
        for i in 0..table.row_count() {
            let country = table.get_value(i, 1);
            println!("Row {i}: country = {country:?}");
        }

        // Test NOT IN clause - should exclude CA, return US and UK (2 rows)
        println!("\n--- Test: country NOT IN ('CA') ---");
        let result = engine.execute(
            table.clone(),
            "SELECT * FROM test WHERE country NOT IN ('CA')",
        );
        match result {
            Ok(view) => {
                println!("SUCCESS: Found {} rows not in ('CA')", view.row_count());
                assert_eq!(view.row_count(), 2); // Should find US and UK
            }
            Err(e) => {
                panic!("NOT IN query failed: {e}");
            }
        }

        println!("\n=== NOT IN test complete! ===");
    }

    #[test]
    fn test_case_insensitive_in_and_not_in() {
        // Initialize tracing for debug output
        let _ = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .try_init();

        let mut table = DataTable::new("test");
        table.add_column(DataColumn::new("id"));
        table.add_column(DataColumn::new("country"));

        // Add test data with mixed case
        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(1),
                DataValue::String("CA".to_string()), // uppercase
            ]))
            .unwrap();

        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(2),
                DataValue::String("us".to_string()), // lowercase
            ]))
            .unwrap();

        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(3),
                DataValue::String("UK".to_string()), // uppercase
            ]))
            .unwrap();

        let table = Arc::new(table);

        println!("\n=== Testing Case-Insensitive IN clause ===");
        println!("Table has {} rows", table.row_count());
        for i in 0..table.row_count() {
            let country = table.get_value(i, 1);
            println!("Row {i}: country = {country:?}");
        }

        // Test case-insensitive IN - should match 'CA' with 'ca'
        println!("\n--- Test: country IN ('ca') with case_insensitive=true ---");
        let engine = QueryEngine::with_case_insensitive(true);
        let result = engine.execute(table.clone(), "SELECT * FROM test WHERE country IN ('ca')");
        match result {
            Ok(view) => {
                println!(
                    "SUCCESS: Found {} rows matching 'ca' (case-insensitive)",
                    view.row_count()
                );
                assert_eq!(view.row_count(), 1); // Should find CA row
            }
            Err(e) => {
                panic!("Case-insensitive IN query failed: {e}");
            }
        }

        // Test case-insensitive NOT IN - should exclude 'CA' when searching for 'ca'
        println!("\n--- Test: country NOT IN ('ca') with case_insensitive=true ---");
        let result = engine.execute(
            table.clone(),
            "SELECT * FROM test WHERE country NOT IN ('ca')",
        );
        match result {
            Ok(view) => {
                println!(
                    "SUCCESS: Found {} rows not matching 'ca' (case-insensitive)",
                    view.row_count()
                );
                assert_eq!(view.row_count(), 2); // Should find us and UK rows
            }
            Err(e) => {
                panic!("Case-insensitive NOT IN query failed: {e}");
            }
        }

        // Test case-sensitive (default) - should NOT match 'CA' with 'ca'
        println!("\n--- Test: country IN ('ca') with case_insensitive=false ---");
        let engine_case_sensitive = QueryEngine::new(); // defaults to case_insensitive=false
        let result = engine_case_sensitive
            .execute(table.clone(), "SELECT * FROM test WHERE country IN ('ca')");
        match result {
            Ok(view) => {
                println!(
                    "SUCCESS: Found {} rows matching 'ca' (case-sensitive)",
                    view.row_count()
                );
                assert_eq!(view.row_count(), 0); // Should find no rows (CA != ca)
            }
            Err(e) => {
                panic!("Case-sensitive IN query failed: {e}");
            }
        }

        println!("\n=== Case-insensitive IN/NOT IN test complete! ===");
    }

    #[test]
    #[ignore = "Parentheses in WHERE clause not yet implemented"]
    fn test_parentheses_in_where_clause() {
        // Initialize tracing for debug output
        let _ = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .try_init();

        let mut table = DataTable::new("test");
        table.add_column(DataColumn::new("id"));
        table.add_column(DataColumn::new("status"));
        table.add_column(DataColumn::new("priority"));

        // Add test data
        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(1),
                DataValue::String("Pending".to_string()),
                DataValue::String("High".to_string()),
            ]))
            .unwrap();

        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(2),
                DataValue::String("Complete".to_string()),
                DataValue::String("High".to_string()),
            ]))
            .unwrap();

        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(3),
                DataValue::String("Pending".to_string()),
                DataValue::String("Low".to_string()),
            ]))
            .unwrap();

        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(4),
                DataValue::String("Complete".to_string()),
                DataValue::String("Low".to_string()),
            ]))
            .unwrap();

        let table = Arc::new(table);
        let engine = QueryEngine::new();

        println!("\n=== Testing Parentheses in WHERE clause ===");
        println!("Table has {} rows", table.row_count());
        for i in 0..table.row_count() {
            let status = table.get_value(i, 1);
            let priority = table.get_value(i, 2);
            println!("Row {i}: status = {status:?}, priority = {priority:?}");
        }

        // Test OR with parentheses - should get (Pending AND High) OR (Complete AND Low)
        println!("\n--- Test: (status = 'Pending' AND priority = 'High') OR (status = 'Complete' AND priority = 'Low') ---");
        let result = engine.execute(
            table.clone(),
            "SELECT * FROM test WHERE (status = 'Pending' AND priority = 'High') OR (status = 'Complete' AND priority = 'Low')",
        );
        match result {
            Ok(view) => {
                println!(
                    "SUCCESS: Found {} rows with parenthetical logic",
                    view.row_count()
                );
                assert_eq!(view.row_count(), 2); // Should find rows 1 and 4
            }
            Err(e) => {
                panic!("Parentheses query failed: {e}");
            }
        }

        println!("\n=== Parentheses test complete! ===");
    }

    #[test]
    #[ignore = "Numeric type coercion needs fixing"]
    fn test_numeric_type_coercion() {
        // Initialize tracing for debug output
        let _ = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .try_init();

        let mut table = DataTable::new("test");
        table.add_column(DataColumn::new("id"));
        table.add_column(DataColumn::new("price"));
        table.add_column(DataColumn::new("quantity"));

        // Add test data with different numeric types
        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(1),
                DataValue::Float(99.50), // Contains '.'
                DataValue::Integer(100),
            ]))
            .unwrap();

        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(2),
                DataValue::Float(150.0), // Contains '.' and '0'
                DataValue::Integer(200),
            ]))
            .unwrap();

        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(3),
                DataValue::Integer(75), // No decimal point
                DataValue::Integer(50),
            ]))
            .unwrap();

        let table = Arc::new(table);
        let engine = QueryEngine::new();

        println!("\n=== Testing Numeric Type Coercion ===");
        println!("Table has {} rows", table.row_count());
        for i in 0..table.row_count() {
            let price = table.get_value(i, 1);
            let quantity = table.get_value(i, 2);
            println!("Row {i}: price = {price:?}, quantity = {quantity:?}");
        }

        // Test Contains on float values - should find rows with decimal points
        println!("\n--- Test: price.Contains('.') ---");
        let result = engine.execute(
            table.clone(),
            "SELECT * FROM test WHERE price.Contains('.')",
        );
        match result {
            Ok(view) => {
                println!(
                    "SUCCESS: Found {} rows with decimal points in price",
                    view.row_count()
                );
                assert_eq!(view.row_count(), 2); // Should find 99.50 and 150.0
            }
            Err(e) => {
                panic!("Numeric Contains query failed: {e}");
            }
        }

        // Test Contains on integer values converted to string
        println!("\n--- Test: quantity.Contains('0') ---");
        let result = engine.execute(
            table.clone(),
            "SELECT * FROM test WHERE quantity.Contains('0')",
        );
        match result {
            Ok(view) => {
                println!(
                    "SUCCESS: Found {} rows with '0' in quantity",
                    view.row_count()
                );
                assert_eq!(view.row_count(), 2); // Should find 100 and 200
            }
            Err(e) => {
                panic!("Integer Contains query failed: {e}");
            }
        }

        println!("\n=== Numeric type coercion test complete! ===");
    }

    #[test]
    fn test_datetime_comparisons() {
        // Initialize tracing for debug output
        let _ = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .try_init();

        let mut table = DataTable::new("test");
        table.add_column(DataColumn::new("id"));
        table.add_column(DataColumn::new("created_date"));

        // Add test data with date strings (as they would come from CSV)
        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(1),
                DataValue::String("2024-12-15".to_string()),
            ]))
            .unwrap();

        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(2),
                DataValue::String("2025-01-15".to_string()),
            ]))
            .unwrap();

        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(3),
                DataValue::String("2025-02-15".to_string()),
            ]))
            .unwrap();

        let table = Arc::new(table);
        let engine = QueryEngine::new();

        println!("\n=== Testing DateTime Comparisons ===");
        println!("Table has {} rows", table.row_count());
        for i in 0..table.row_count() {
            let date = table.get_value(i, 1);
            println!("Row {i}: created_date = {date:?}");
        }

        // Test DateTime constructor comparison - should find dates after 2025-01-01
        println!("\n--- Test: created_date > DateTime(2025,1,1) ---");
        let result = engine.execute(
            table.clone(),
            "SELECT * FROM test WHERE created_date > DateTime(2025,1,1)",
        );
        match result {
            Ok(view) => {
                println!("SUCCESS: Found {} rows after 2025-01-01", view.row_count());
                assert_eq!(view.row_count(), 2); // Should find 2025-01-15 and 2025-02-15
            }
            Err(e) => {
                panic!("DateTime comparison query failed: {e}");
            }
        }

        println!("\n=== DateTime comparison test complete! ===");
    }

    #[test]
    fn test_not_with_method_calls() {
        // Initialize tracing for debug output
        let _ = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .try_init();

        let mut table = DataTable::new("test");
        table.add_column(DataColumn::new("id"));
        table.add_column(DataColumn::new("status"));

        // Add test data
        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(1),
                DataValue::String("Pending Review".to_string()),
            ]))
            .unwrap();

        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(2),
                DataValue::String("Complete".to_string()),
            ]))
            .unwrap();

        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(3),
                DataValue::String("Pending Approval".to_string()),
            ]))
            .unwrap();

        let table = Arc::new(table);
        let engine = QueryEngine::with_case_insensitive(true);

        println!("\n=== Testing NOT with Method Calls ===");
        println!("Table has {} rows", table.row_count());
        for i in 0..table.row_count() {
            let status = table.get_value(i, 1);
            println!("Row {i}: status = {status:?}");
        }

        // Test NOT with Contains - should exclude rows containing "pend"
        println!("\n--- Test: NOT status.Contains('pend') ---");
        let result = engine.execute(
            table.clone(),
            "SELECT * FROM test WHERE NOT status.Contains('pend')",
        );
        match result {
            Ok(view) => {
                println!(
                    "SUCCESS: Found {} rows NOT containing 'pend'",
                    view.row_count()
                );
                assert_eq!(view.row_count(), 1); // Should find only "Complete"
            }
            Err(e) => {
                panic!("NOT Contains query failed: {e}");
            }
        }

        // Test NOT with StartsWith
        println!("\n--- Test: NOT status.StartsWith('Pending') ---");
        let result = engine.execute(
            table.clone(),
            "SELECT * FROM test WHERE NOT status.StartsWith('Pending')",
        );
        match result {
            Ok(view) => {
                println!(
                    "SUCCESS: Found {} rows NOT starting with 'Pending'",
                    view.row_count()
                );
                assert_eq!(view.row_count(), 1); // Should find only "Complete"
            }
            Err(e) => {
                panic!("NOT StartsWith query failed: {e}");
            }
        }

        println!("\n=== NOT with method calls test complete! ===");
    }

    #[test]
    #[ignore = "Complex logical expressions with parentheses not yet implemented"]
    fn test_complex_logical_expressions() {
        // Initialize tracing for debug output
        let _ = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .try_init();

        let mut table = DataTable::new("test");
        table.add_column(DataColumn::new("id"));
        table.add_column(DataColumn::new("status"));
        table.add_column(DataColumn::new("priority"));
        table.add_column(DataColumn::new("assigned"));

        // Add comprehensive test data
        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(1),
                DataValue::String("Pending".to_string()),
                DataValue::String("High".to_string()),
                DataValue::String("John".to_string()),
            ]))
            .unwrap();

        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(2),
                DataValue::String("Complete".to_string()),
                DataValue::String("High".to_string()),
                DataValue::String("Jane".to_string()),
            ]))
            .unwrap();

        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(3),
                DataValue::String("Pending".to_string()),
                DataValue::String("Low".to_string()),
                DataValue::String("John".to_string()),
            ]))
            .unwrap();

        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(4),
                DataValue::String("In Progress".to_string()),
                DataValue::String("Medium".to_string()),
                DataValue::String("Jane".to_string()),
            ]))
            .unwrap();

        let table = Arc::new(table);
        let engine = QueryEngine::new();

        println!("\n=== Testing Complex Logical Expressions ===");
        println!("Table has {} rows", table.row_count());
        for i in 0..table.row_count() {
            let status = table.get_value(i, 1);
            let priority = table.get_value(i, 2);
            let assigned = table.get_value(i, 3);
            println!(
                "Row {i}: status = {status:?}, priority = {priority:?}, assigned = {assigned:?}"
            );
        }

        // Test complex AND/OR logic
        println!("\n--- Test: status = 'Pending' AND (priority = 'High' OR assigned = 'John') ---");
        let result = engine.execute(
            table.clone(),
            "SELECT * FROM test WHERE status = 'Pending' AND (priority = 'High' OR assigned = 'John')",
        );
        match result {
            Ok(view) => {
                println!(
                    "SUCCESS: Found {} rows with complex logic",
                    view.row_count()
                );
                assert_eq!(view.row_count(), 2); // Should find rows 1 and 3 (both Pending, one High priority, both assigned to John)
            }
            Err(e) => {
                panic!("Complex logic query failed: {e}");
            }
        }

        // Test NOT with complex expressions
        println!("\n--- Test: NOT (status.Contains('Complete') OR priority = 'Low') ---");
        let result = engine.execute(
            table.clone(),
            "SELECT * FROM test WHERE NOT (status.Contains('Complete') OR priority = 'Low')",
        );
        match result {
            Ok(view) => {
                println!(
                    "SUCCESS: Found {} rows with NOT complex logic",
                    view.row_count()
                );
                assert_eq!(view.row_count(), 2); // Should find rows 1 (Pending+High) and 4 (In Progress+Medium)
            }
            Err(e) => {
                panic!("NOT complex logic query failed: {e}");
            }
        }

        println!("\n=== Complex logical expressions test complete! ===");
    }

    #[test]
    fn test_mixed_data_types_and_edge_cases() {
        // Initialize tracing for debug output
        let _ = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .try_init();

        let mut table = DataTable::new("test");
        table.add_column(DataColumn::new("id"));
        table.add_column(DataColumn::new("value"));
        table.add_column(DataColumn::new("nullable_field"));

        // Add test data with mixed types and edge cases
        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(1),
                DataValue::String("123.45".to_string()),
                DataValue::String("present".to_string()),
            ]))
            .unwrap();

        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(2),
                DataValue::Float(678.90),
                DataValue::Null,
            ]))
            .unwrap();

        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(3),
                DataValue::Boolean(true),
                DataValue::String("also present".to_string()),
            ]))
            .unwrap();

        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(4),
                DataValue::String("false".to_string()),
                DataValue::Null,
            ]))
            .unwrap();

        let table = Arc::new(table);
        let engine = QueryEngine::new();

        println!("\n=== Testing Mixed Data Types and Edge Cases ===");
        println!("Table has {} rows", table.row_count());
        for i in 0..table.row_count() {
            let value = table.get_value(i, 1);
            let nullable = table.get_value(i, 2);
            println!("Row {i}: value = {value:?}, nullable_field = {nullable:?}");
        }

        // Test type coercion with boolean Contains
        println!("\n--- Test: value.Contains('true') (boolean to string coercion) ---");
        let result = engine.execute(
            table.clone(),
            "SELECT * FROM test WHERE value.Contains('true')",
        );
        match result {
            Ok(view) => {
                println!(
                    "SUCCESS: Found {} rows with boolean coercion",
                    view.row_count()
                );
                assert_eq!(view.row_count(), 1); // Should find the boolean true row
            }
            Err(e) => {
                panic!("Boolean coercion query failed: {e}");
            }
        }

        // Test multiple IN values with mixed types
        println!("\n--- Test: id IN (1, 3) ---");
        let result = engine.execute(table.clone(), "SELECT * FROM test WHERE id IN (1, 3)");
        match result {
            Ok(view) => {
                println!("SUCCESS: Found {} rows with IN clause", view.row_count());
                assert_eq!(view.row_count(), 2); // Should find rows with id 1 and 3
            }
            Err(e) => {
                panic!("Multiple IN values query failed: {e}");
            }
        }

        println!("\n=== Mixed data types test complete! ===");
    }

    /// Test that aggregate-only queries return exactly one row (regression test)
    #[test]
    fn test_aggregate_only_single_row() {
        let table = create_test_stock_data();
        let engine = QueryEngine::new();

        // Test query with multiple aggregates - should return exactly 1 row
        let result = engine
            .execute(
                table.clone(),
                "SELECT COUNT(*), MIN(close), MAX(close), AVG(close) FROM stock",
            )
            .expect("Query should succeed");

        assert_eq!(
            result.row_count(),
            1,
            "Aggregate-only query should return exactly 1 row"
        );
        assert_eq!(result.column_count(), 4, "Should have 4 aggregate columns");

        // Verify the actual values are correct
        let source = result.source();
        let row = source.get_row(0).expect("Should have first row");

        // COUNT(*) should be 5 (total rows)
        assert_eq!(row.values[0], DataValue::Integer(5));

        // MIN should be 99.5
        assert_eq!(row.values[1], DataValue::Float(99.5));

        // MAX should be 105.0
        assert_eq!(row.values[2], DataValue::Float(105.0));

        // AVG should be approximately 102.4
        if let DataValue::Float(avg) = &row.values[3] {
            assert!(
                (avg - 102.4).abs() < 0.01,
                "Average should be approximately 102.4, got {}",
                avg
            );
        } else {
            panic!("AVG should return a Float value");
        }
    }

    /// Test single aggregate function returns single row
    #[test]
    fn test_single_aggregate_single_row() {
        let table = create_test_stock_data();
        let engine = QueryEngine::new();

        let result = engine
            .execute(table.clone(), "SELECT COUNT(*) FROM stock")
            .expect("Query should succeed");

        assert_eq!(
            result.row_count(),
            1,
            "Single aggregate query should return exactly 1 row"
        );
        assert_eq!(result.column_count(), 1, "Should have 1 column");

        let source = result.source();
        let row = source.get_row(0).expect("Should have first row");
        assert_eq!(row.values[0], DataValue::Integer(5));
    }

    /// Test aggregate with WHERE clause filtering
    #[test]
    fn test_aggregate_with_where_single_row() {
        let table = create_test_stock_data();
        let engine = QueryEngine::new();

        // Filter to only high-value stocks (>= 103.0) and aggregate
        let result = engine
            .execute(
                table.clone(),
                "SELECT COUNT(*), MIN(close), MAX(close) FROM stock WHERE close >= 103.0",
            )
            .expect("Query should succeed");

        assert_eq!(
            result.row_count(),
            1,
            "Filtered aggregate query should return exactly 1 row"
        );
        assert_eq!(result.column_count(), 3, "Should have 3 aggregate columns");

        let source = result.source();
        let row = source.get_row(0).expect("Should have first row");

        // Should find 2 rows (103.5 and 105.0)
        assert_eq!(row.values[0], DataValue::Integer(2));
        assert_eq!(row.values[1], DataValue::Float(103.5)); // MIN
        assert_eq!(row.values[2], DataValue::Float(105.0)); // MAX
    }

    #[test]
    fn test_not_in_parsing() {
        use crate::sql::recursive_parser::Parser;

        let query = "SELECT * FROM test WHERE country NOT IN ('CA')";
        println!("\n=== Testing NOT IN parsing ===");
        println!("Parsing query: {query}");

        let mut parser = Parser::new(query);
        match parser.parse() {
            Ok(statement) => {
                println!("Parsed statement: {statement:#?}");
                if let Some(where_clause) = statement.where_clause {
                    println!("WHERE conditions: {:#?}", where_clause.conditions);
                    if let Some(first_condition) = where_clause.conditions.first() {
                        println!("First condition expression: {:#?}", first_condition.expr);
                    }
                }
            }
            Err(e) => {
                panic!("Parse error: {e}");
            }
        }
    }

    /// Create test stock data for aggregate testing
    fn create_test_stock_data() -> Arc<DataTable> {
        let mut table = DataTable::new("stock");

        table.add_column(DataColumn::new("symbol"));
        table.add_column(DataColumn::new("close"));
        table.add_column(DataColumn::new("volume"));

        // Add 5 rows of test data
        let test_data = vec![
            ("AAPL", 99.5, 1000),
            ("AAPL", 101.2, 1500),
            ("AAPL", 103.5, 2000),
            ("AAPL", 105.0, 1200),
            ("AAPL", 102.8, 1800),
        ];

        for (symbol, close, volume) in test_data {
            table
                .add_row(DataRow::new(vec![
                    DataValue::String(symbol.to_string()),
                    DataValue::Float(close),
                    DataValue::Integer(volume),
                ]))
                .expect("Should add row successfully");
        }

        Arc::new(table)
    }
}

#[cfg(test)]
#[path = "query_engine_tests.rs"]
mod query_engine_tests;

use anyhow::{anyhow, Result};
use fxhash::{FxHashMap, FxHashSet};
use std::cmp::min;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
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
use crate::data::subquery_executor::SubqueryExecutor;
use crate::data::virtual_table_generator::VirtualTableGenerator;
use crate::execution_plan::{ExecutionPlan, ExecutionPlanBuilder, StepType};
use crate::sql::aggregates::{contains_aggregate, is_aggregate_compatible};
use crate::sql::parser::ast::ColumnRef;
use crate::sql::parser::ast::TableSource;
use crate::sql::recursive_parser::{
    CTEType, OrderByColumn, Parser, SelectItem, SelectStatement, SortDirection, SqlExpression,
    TableFunction,
};

/// Query engine that executes SQL directly on `DataTable`
#[derive(Clone)]
pub struct QueryEngine {
    case_insensitive: bool,
    date_notation: String,
    behavior_config: Option<BehaviorConfig>,
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
            behavior_config: None,
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
            behavior_config: Some(config),
        }
    }

    #[must_use]
    pub fn with_date_notation(_date_notation: String) -> Self {
        Self {
            case_insensitive: false,
            date_notation: get_date_notation(), // Always use the global function
            behavior_config: None,
        }
    }

    #[must_use]
    pub fn with_case_insensitive(case_insensitive: bool) -> Self {
        Self {
            case_insensitive,
            date_notation: get_date_notation(),
            behavior_config: None,
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
            behavior_config: None,
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

    /// Execute a SQL query on a `DataTable` and return a `DataView` (for backward compatibility)
    pub fn execute(&self, table: Arc<DataTable>, sql: &str) -> Result<DataView> {
        let (view, _plan) = self.execute_with_plan(table, sql)?;
        Ok(view)
    }

    /// Execute a parsed SelectStatement on a `DataTable` and return a `DataView`
    pub fn execute_statement(
        &self,
        table: Arc<DataTable>,
        statement: SelectStatement,
    ) -> Result<DataView> {
        // First process CTEs to build context
        let mut cte_context = HashMap::new();
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
                    let mut data_table = fetcher.fetch(web_spec, &cte.name)?;

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
                    let mut data_table = fetcher.fetch(web_spec, &cte.name)?;

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
                        let mut data_table = fetcher.fetch(web_spec, &cte.name)?;

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
        self.build_view_with_context_and_plan(table, statement, cte_context, &mut dummy_plan)
    }

    /// Build a DataView from a SelectStatement with CTE context and execution plan tracking
    fn build_view_with_context_and_plan(
        &self,
        table: Arc<DataTable>,
        statement: SelectStatement,
        cte_context: &mut HashMap<String, Arc<DataView>>,
        plan: &mut ExecutionPlanBuilder,
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
                let materialized = self.materialize_view((**cte_view).clone())?;
                Arc::new(materialized)
            } else {
                // Regular table reference - use the provided table
                table.clone()
            }
        } else {
            // No FROM clause - use the provided table
            table.clone()
        };

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
                    "Executing {:?} JOIN on {}",
                    join_clause.join_type, join_clause.condition.left_column
                ));

                // Resolve the right table for the join
                let right_table = match &join_clause.table {
                    TableSource::Table(name) => {
                        // Check if it's a CTE reference
                        if let Some(cte_view) = cte_context.get(name) {
                            let materialized = self.materialize_view((**cte_view).clone())?;
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
        self.build_view_internal_with_plan(final_table, statement, plan)
    }

    /// Materialize a DataView into a new DataTable
    fn materialize_view(&self, view: DataView) -> Result<DataTable> {
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

            // Filter visible rows based on WHERE clause
            let mut filtered_rows = Vec::new();
            for row_idx in visible_rows {
                // Only log for first few rows to avoid performance impact
                if row_idx < 3 {
                    debug!("QueryEngine: Evaluating WHERE clause for row {}", row_idx);
                }
                let mut evaluator =
                    RecursiveWhereEvaluator::with_context(&table, &mut eval_context);
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
                )?;

                plan.set_rows_out(view.row_count());
                plan.add_detail(format!("Output: {} groups", view.row_count()));
                plan.add_detail(format!(
                    "Group time: {:.3}ms",
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
                    view = self.apply_select_items(view, &statement.select_items)?;
                } else {
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
                view = self.apply_multi_order_by(view, order_by_columns)?;

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
    fn apply_select_items(&self, view: DataView, select_items: &[SelectItem]) -> Result<DataView> {
        debug!(
            "QueryEngine::apply_select_items - items: {:?}",
            select_items
        );
        debug!(
            "QueryEngine::apply_select_items - input view has {} rows",
            view.row_count()
        );

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

        for &row_idx in visible_rows {
            let mut row_values = Vec::new();

            for item in &expanded_items {
                let value = match item {
                    SelectItem::Column(col_ref) => {
                        // Column reference with optional table prefix
                        let col_idx = if let Some(table_prefix) = &col_ref.table_prefix {
                            // For qualified references, ONLY try qualified lookup - no fallback
                            let qualified_name = format!("{}.{}", table_prefix, col_ref.name);
                            let result = source_table.find_column_by_qualified_name(&qualified_name);

                            // Debug: log what qualified names are available when lookup fails
                            if result.is_none() {
                                let available_qualified: Vec<String> = source_table.columns.iter()
                                    .filter_map(|c| c.qualified_name.clone())
                                    .collect();
                                let available_simple: Vec<String> = source_table.columns.iter()
                                    .map(|c| c.name.clone())
                                    .collect();
                                debug!(
                                    "Qualified column '{}' not found. Available qualified names: {:?}, Simple names: {:?}",
                                    qualified_name, available_qualified, available_simple
                                );
                                // This MUST return None to trigger error
                            }
                            result
                        } else {
                            // Simple column name lookup
                            source_table.get_column_index(&col_ref.name)
                        }
                        .ok_or_else(|| {
                            let display_name = if let Some(prefix) = &col_ref.table_prefix {
                                format!("{}.{}", prefix, col_ref.name)
                            } else {
                                col_ref.name.clone()
                            };
                            let suggestion = self.find_similar_column(source_table, &col_ref.name);
                            match suggestion {
                                Some(similar) => anyhow::anyhow!(
                                    "Column '{}' not found. Did you mean '{}'?",
                                    display_name,
                                    similar
                                ),
                                None => anyhow::anyhow!("Column '{}' not found", display_name),
                            }
                        })?;
                        let row = source_table
                            .get_row(row_idx)
                            .ok_or_else(|| anyhow::anyhow!("Row {} not found", row_idx))?;
                        row.get(col_idx)
                            .ok_or_else(|| anyhow::anyhow!("Column {} not found in row", col_idx))?
                            .clone()
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
        mut view: DataView,
        order_by_columns: &[OrderByColumn],
    ) -> Result<DataView> {
        // Build list of (source_column_index, ascending) tuples
        let mut sort_columns = Vec::new();

        for order_col in order_by_columns {
            // Try to find the column index
            // Check in the current view's source table (this handles both regular columns and computed columns)
            // This is especially important after GROUP BY where we have a new result table with aggregate aliases
            let col_index = view
                .source()
                .get_column_index(&order_col.column)
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
    ) -> Result<DataView> {
        // Use the new expression-based GROUP BY implementation
        self.apply_group_by_expressions(
            view,
            group_by_exprs,
            select_items,
            having,
            self.case_insensitive,
            self.date_notation.clone(),
        )
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

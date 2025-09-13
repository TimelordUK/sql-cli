// Execution Plan Module
// Provides detailed tracing and timing for query execution

use crate::data::datatable::DataValue;
use crate::sql::parser::ast::{SelectStatement, SqlExpression, WhereClause};
use std::fmt;
use std::time::{Duration, Instant};

/// Represents a single step in the execution plan
#[derive(Debug, Clone)]
pub struct ExecutionStep {
    pub step_type: StepType,
    pub description: String,
    pub details: Vec<String>,
    pub rows_in: Option<usize>,
    pub rows_out: Option<usize>,
    pub duration: Option<Duration>,
    pub children: Vec<ExecutionStep>,
}

/// Types of execution steps
#[derive(Debug, Clone)]
pub enum StepType {
    Parse,
    LoadData,
    TableScan,
    Filter,
    Sort,
    GroupBy,
    Having,
    Select,
    Limit,
    Output,
    Join,
    WindowFunction,
    Expression,
}

impl fmt::Display for StepType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StepType::Parse => write!(f, "PARSE"),
            StepType::LoadData => write!(f, "LOAD_DATA"),
            StepType::TableScan => write!(f, "TABLE_SCAN"),
            StepType::Filter => write!(f, "FILTER"),
            StepType::Sort => write!(f, "SORT"),
            StepType::GroupBy => write!(f, "GROUP_BY"),
            StepType::Having => write!(f, "HAVING"),
            StepType::Select => write!(f, "SELECT"),
            StepType::Limit => write!(f, "LIMIT"),
            StepType::Output => write!(f, "OUTPUT"),
            StepType::Join => write!(f, "JOIN"),
            StepType::WindowFunction => write!(f, "WINDOW"),
            StepType::Expression => write!(f, "EXPR"),
        }
    }
}

/// Execution plan builder that tracks all steps
pub struct ExecutionPlanBuilder {
    steps: Vec<ExecutionStep>,
    current_step: Option<ExecutionStep>,
    start_time: Option<Instant>,
}

impl ExecutionPlanBuilder {
    pub fn new() -> Self {
        Self {
            steps: Vec::new(),
            current_step: None,
            start_time: None,
        }
    }

    /// Start a new execution step
    pub fn begin_step(&mut self, step_type: StepType, description: String) {
        if let Some(current) = self.current_step.take() {
            self.steps.push(current);
        }

        self.current_step = Some(ExecutionStep {
            step_type,
            description,
            details: Vec::new(),
            rows_in: None,
            rows_out: None,
            duration: None,
            children: Vec::new(),
        });
        self.start_time = Some(Instant::now());
    }

    /// Add a detail to the current step
    pub fn add_detail(&mut self, detail: String) {
        if let Some(ref mut step) = self.current_step {
            step.details.push(detail);
        }
    }

    /// Set input row count for current step
    pub fn set_rows_in(&mut self, count: usize) {
        if let Some(ref mut step) = self.current_step {
            step.rows_in = Some(count);
        }
    }

    /// Set output row count for current step
    pub fn set_rows_out(&mut self, count: usize) {
        if let Some(ref mut step) = self.current_step {
            step.rows_out = Some(count);
        }
    }

    /// End the current step and record duration
    pub fn end_step(&mut self) {
        if let Some(ref mut step) = self.current_step {
            if let Some(start) = self.start_time {
                step.duration = Some(start.elapsed());
            }
        }

        if let Some(step) = self.current_step.take() {
            self.steps.push(step);
        }
        self.start_time = None;
    }

    /// Add a child step to the current step
    pub fn add_child_step(&mut self, child: ExecutionStep) {
        if let Some(ref mut step) = self.current_step {
            step.children.push(child);
        }
    }

    /// Build the final execution plan
    pub fn build(mut self) -> ExecutionPlan {
        if let Some(current) = self.current_step.take() {
            self.steps.push(current);
        }

        let total_duration = self.steps.iter().filter_map(|s| s.duration).sum();

        ExecutionPlan {
            steps: self.steps,
            total_duration,
        }
    }
}

/// Complete execution plan
pub struct ExecutionPlan {
    pub steps: Vec<ExecutionStep>,
    pub total_duration: Duration,
}

impl ExecutionPlan {
    /// Format the execution plan as a tree
    pub fn format_tree(&self) -> String {
        let mut output = String::new();
        output.push_str("\n");
        output
            .push_str("╔══════════════════════════════════════════════════════════════════════╗\n");
        output
            .push_str("║                        EXECUTION PLAN                               ║\n");
        output.push_str(
            "╚══════════════════════════════════════════════════════════════════════╝\n\n",
        );

        for (i, step) in self.steps.iter().enumerate() {
            self.format_step(&mut output, step, 0, i == self.steps.len() - 1);
        }

        output.push_str("\n");
        output.push_str(&format!(
            "Total Execution Time: {:.3}ms\n",
            self.total_duration.as_secs_f64() * 1000.0
        ));

        output
    }

    fn format_step(&self, output: &mut String, step: &ExecutionStep, indent: usize, is_last: bool) {
        let prefix = if indent == 0 {
            if is_last {
                "└─".to_string()
            } else {
                "├─".to_string()
            }
        } else {
            format!(
                "{}{}",
                "  ".repeat(indent),
                if is_last { "└─" } else { "├─" }
            )
        };

        // Format main step line
        let time_str = step
            .duration
            .map(|d| format!(" [{:.3}ms]", d.as_secs_f64() * 1000.0))
            .unwrap_or_default();

        let rows_str = match (step.rows_in, step.rows_out) {
            (Some(i), Some(o)) if i != o => format!(" (rows: {} → {})", i, o),
            (_, Some(o)) => format!(" (rows: {})", o),
            _ => String::new(),
        };

        output.push_str(&format!(
            "{} {} {}{}{}\n",
            prefix, step.step_type, step.description, rows_str, time_str
        ));

        // Format details
        let detail_indent = if indent == 0 {
            "  ".to_string()
        } else {
            "  ".repeat(indent + 1)
        };
        for detail in &step.details {
            output.push_str(&format!("{}  • {}\n", detail_indent, detail));
        }

        // Format children
        for (i, child) in step.children.iter().enumerate() {
            self.format_step(output, child, indent + 1, i == step.children.len() - 1);
        }
    }

    /// Format the execution plan as a table
    pub fn format_table(&self) -> String {
        let mut output = String::new();
        output.push_str("\n");
        output.push_str(
            "┌────────────────┬──────────────────────────────┬──────────┬──────────┬──────────┐\n",
        );
        output.push_str(
            "│ Step           │ Description                  │ Rows In  │ Rows Out │ Time(ms) │\n",
        );
        output.push_str(
            "├────────────────┼──────────────────────────────┼──────────┼──────────┼──────────┤\n",
        );

        for step in &self.steps {
            self.format_step_table(&mut output, step, 0);
        }

        output.push_str(
            "└────────────────┴──────────────────────────────┴──────────┴──────────┴──────────┘\n",
        );
        output.push_str(&format!(
            "\nTotal Time: {:.3}ms\n",
            self.total_duration.as_secs_f64() * 1000.0
        ));

        output
    }

    fn format_step_table(&self, output: &mut String, step: &ExecutionStep, indent: usize) {
        let step_name = format!("{}{}", "  ".repeat(indent), step.step_type);
        let desc = if step.description.len() > 28 {
            format!("{}...", &step.description[..25])
        } else {
            step.description.clone()
        };

        let rows_in = step
            .rows_in
            .map(|r| r.to_string())
            .unwrap_or_else(|| "-".to_string());

        let rows_out = step
            .rows_out
            .map(|r| r.to_string())
            .unwrap_or_else(|| "-".to_string());

        let time = step
            .duration
            .map(|d| format!("{:.3}", d.as_secs_f64() * 1000.0))
            .unwrap_or_else(|| "-".to_string());

        output.push_str(&format!(
            "│ {:<14} │ {:<28} │ {:>8} │ {:>8} │ {:>8} │\n",
            step_name, desc, rows_in, rows_out, time
        ));

        for child in &step.children {
            self.format_step_table(output, child, indent + 1);
        }
    }

    /// Get performance insights
    pub fn get_insights(&self) -> Vec<String> {
        let mut insights = Vec::new();

        // Find slowest steps
        let mut steps_with_time: Vec<_> = self
            .steps
            .iter()
            .filter_map(|s| s.duration.map(|d| (s, d)))
            .collect();
        steps_with_time.sort_by_key(|(_, d)| std::cmp::Reverse(*d));

        if let Some((slowest, duration)) = steps_with_time.first() {
            let percentage = (duration.as_secs_f64() / self.total_duration.as_secs_f64()) * 100.0;
            if percentage > 50.0 {
                insights.push(format!(
                    "⚠️  {} step took {:.1}% of total execution time ({:.3}ms)",
                    slowest.step_type,
                    percentage,
                    duration.as_secs_f64() * 1000.0
                ));
            }
        }

        // Check for filter efficiency
        for step in &self.steps {
            if matches!(step.step_type, StepType::Filter) {
                if let (Some(rows_in), Some(rows_out)) = (step.rows_in, step.rows_out) {
                    if rows_in > 0 {
                        let selectivity = (rows_out as f64 / rows_in as f64) * 100.0;
                        if selectivity < 10.0 {
                            insights.push(format!(
                                "✓ Highly selective filter: {:.1}% of rows passed ({}→{})",
                                selectivity, rows_in, rows_out
                            ));
                        } else if selectivity > 90.0 {
                            insights.push(format!(
                                "⚠️  Low selectivity filter: {:.1}% of rows passed ({}→{})",
                                selectivity, rows_in, rows_out
                            ));
                        }
                    }
                }
            }
        }

        // Check for large sorts
        for step in &self.steps {
            if matches!(step.step_type, StepType::Sort) {
                if let Some(rows) = step.rows_in {
                    if rows > 10000 {
                        insights.push(format!("⚠️  Sorting large dataset: {} rows", rows));
                    }
                }
            }
        }

        insights
    }
}

/// Helper to analyze WHERE clause complexity
pub fn analyze_where_clause(where_clause: &WhereClause) -> Vec<String> {
    let mut details = Vec::new();

    // Count conditions
    let condition_count = where_clause.conditions.len();
    details.push(format!("Conditions: {}", condition_count));

    // Analyze each condition
    for (i, condition) in where_clause.conditions.iter().enumerate() {
        let expr_detail = analyze_expression(&condition.expr);
        details.push(format!("  Condition {}: {}", i + 1, expr_detail));

        if let Some(connector) = &condition.connector {
            details.push(format!("  Connector: {:?}", connector));
        }
    }

    details
}

/// Helper to analyze SQL expressions
pub fn analyze_expression(expr: &SqlExpression) -> String {
    match expr {
        SqlExpression::Column(name) => format!("Column({})", name),
        SqlExpression::BinaryOp { left, op, right } => {
            format!(
                "BinaryOp({} {} {})",
                analyze_expression(left),
                op,
                analyze_expression(right)
            )
        }
        SqlExpression::FunctionCall { name, args, .. } => {
            format!("Function({}, {} args)", name, args.len())
        }
        SqlExpression::Between { expr, .. } => {
            format!("Between({})", analyze_expression(expr))
        }
        SqlExpression::InList { expr, values } => {
            format!(
                "InList({}, {} values)",
                analyze_expression(expr),
                values.len()
            )
        }
        SqlExpression::NotInList { expr, values } => {
            format!(
                "NotInList({}, {} values)",
                analyze_expression(expr),
                values.len()
            )
        }
        _ => format!("{:?}", expr),
    }
}

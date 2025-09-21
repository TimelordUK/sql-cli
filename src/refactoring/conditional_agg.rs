// Conditional aggregation pattern generators

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ConditionalAggregation {
    pub column: String,
    pub values: Vec<String>,
    pub aggregate_func: AggregateFunction,
    pub aggregate_column: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AggregateFunction {
    Count,
    Sum,
    Avg,
    Max,
    Min,
}

impl ConditionalAggregation {
    /// Generate pivot-like conditional aggregation
    pub fn to_sql(&self) -> Vec<String> {
        let mut statements = Vec::new();

        for value in &self.values {
            let condition = format!("{} = '{}'", self.column, value);

            let agg_expr = match &self.aggregate_func {
                AggregateFunction::Count => {
                    format!("SUM(CASE WHEN {} THEN 1 ELSE 0 END)", condition)
                }
                AggregateFunction::Sum => {
                    let default_col = "1".to_string();
                    let col = self.aggregate_column.as_ref().unwrap_or(&default_col);
                    format!("SUM(CASE WHEN {} THEN {} ELSE 0 END)", condition, col)
                }
                AggregateFunction::Avg => {
                    let default_col = "1".to_string();
                    let col = self.aggregate_column.as_ref().unwrap_or(&default_col);
                    format!("AVG(CASE WHEN {} THEN {} ELSE NULL END)", condition, col)
                }
                AggregateFunction::Max => {
                    let col = self.aggregate_column.as_ref().unwrap_or(&self.column);
                    format!("MAX(CASE WHEN {} THEN {} ELSE NULL END)", condition, col)
                }
                AggregateFunction::Min => {
                    let col = self.aggregate_column.as_ref().unwrap_or(&self.column);
                    format!("MIN(CASE WHEN {} THEN {} ELSE NULL END)", condition, col)
                }
            };

            // Generate alias from value (sanitized)
            let alias = format!(
                "{}_{}",
                value.to_lowercase().replace(' ', "_"),
                match &self.aggregate_func {
                    AggregateFunction::Count => "count",
                    AggregateFunction::Sum => "sum",
                    AggregateFunction::Avg => "avg",
                    AggregateFunction::Max => "max",
                    AggregateFunction::Min => "min",
                }
            );

            statements.push(format!("{} AS {}", agg_expr, alias));
        }

        statements
    }

    /// Generate a balance calculation pattern (e.g., debits vs credits)
    pub fn balance_pattern(
        positive_condition: &str,
        negative_condition: &str,
        column: &str,
    ) -> String {
        format!(
            r#"SUM(CASE WHEN {} THEN {} ELSE 0 END) -
SUM(CASE WHEN {} THEN {} ELSE 0 END) AS net_balance"#,
            positive_condition, column, negative_condition, column
        )
    }
}

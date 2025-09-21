// Window Function Refactoring Tools
// Helps generate complex window functions with proper syntax

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct WindowFunctionBuilder {
    pub function_type: WindowFunctionType,
    pub partition_by: Vec<String>,
    pub order_by: Vec<OrderByColumn>,
    pub window_frame: Option<WindowFrame>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum WindowFunctionType {
    RowNumber,
    Rank,
    DenseRank,
    Lead { column: String, offset: i32, default: Option<String> },
    Lag { column: String, offset: i32, default: Option<String> },
    FirstValue { column: String },
    LastValue { column: String },
    NthValue { column: String, n: i32 },
    PercentRank,
    CumeDist,
    Sum { column: String },
    Avg { column: String },
    Count { column: String },
    Max { column: String },
    Min { column: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderByColumn {
    pub column: String,
    pub direction: SortDirection,
    pub nulls: Option<NullHandling>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SortDirection {
    Asc,
    Desc,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum NullHandling {
    First,
    Last,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WindowFrame {
    pub mode: FrameMode,
    pub start: FrameBound,
    pub end: Option<FrameBound>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FrameMode {
    Rows,
    Range,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FrameBound {
    UnboundedPreceding,
    CurrentRow,
    UnboundedFollowing,
    Preceding(i32),
    Following(i32),
}

impl WindowFunctionBuilder {
    /// Create a ROW_NUMBER() window function
    pub fn row_number(partition_by: Vec<String>, order_by: Vec<String>) -> Self {
        Self {
            function_type: WindowFunctionType::RowNumber,
            partition_by,
            order_by: order_by.into_iter().map(|col| OrderByColumn {
                column: col,
                direction: SortDirection::Asc,
                nulls: None,
            }).collect(),
            window_frame: None,
        }
    }

    /// Create a LEAD window function
    pub fn lead(column: String, offset: i32, partition_by: Vec<String>, order_by: Vec<String>) -> Self {
        Self {
            function_type: WindowFunctionType::Lead {
                column: column.clone(),
                offset,
                default: None,
            },
            partition_by,
            order_by: order_by.into_iter().map(|col| OrderByColumn {
                column: col,
                direction: SortDirection::Asc,
                nulls: None,
            }).collect(),
            window_frame: None,
        }
    }

    /// Create a LAG window function
    pub fn lag(column: String, offset: i32, partition_by: Vec<String>, order_by: Vec<String>) -> Self {
        Self {
            function_type: WindowFunctionType::Lag {
                column: column.clone(),
                offset,
                default: None,
            },
            partition_by,
            order_by: order_by.into_iter().map(|col| OrderByColumn {
                column: col,
                direction: SortDirection::Asc,
                nulls: None,
            }).collect(),
            window_frame: None,
        }
    }

    /// Create a running total with SUM
    pub fn running_sum(column: String, partition_by: Vec<String>, order_by: Vec<String>) -> Self {
        Self {
            function_type: WindowFunctionType::Sum { column },
            partition_by,
            order_by: order_by.into_iter().map(|col| OrderByColumn {
                column: col,
                direction: SortDirection::Asc,
                nulls: None,
            }).collect(),
            window_frame: Some(WindowFrame {
                mode: FrameMode::Rows,
                start: FrameBound::UnboundedPreceding,
                end: Some(FrameBound::CurrentRow),
            }),
        }
    }

    /// Generate the SQL for this window function
    pub fn to_sql(&self, alias: Option<&str>) -> String {
        let mut sql = String::new();

        // Add the function part
        match &self.function_type {
            WindowFunctionType::RowNumber => sql.push_str("ROW_NUMBER()"),
            WindowFunctionType::Rank => sql.push_str("RANK()"),
            WindowFunctionType::DenseRank => sql.push_str("DENSE_RANK()"),
            WindowFunctionType::Lead { column, offset, default } => {
                sql.push_str(&format!("LEAD({}", column));
                if *offset != 1 {
                    sql.push_str(&format!(", {}", offset));
                }
                if let Some(def) = default {
                    sql.push_str(&format!(", {}", def));
                }
                sql.push(')');
            }
            WindowFunctionType::Lag { column, offset, default } => {
                sql.push_str(&format!("LAG({}", column));
                if *offset != 1 {
                    sql.push_str(&format!(", {}", offset));
                }
                if let Some(def) = default {
                    sql.push_str(&format!(", {}", def));
                }
                sql.push(')');
            }
            WindowFunctionType::FirstValue { column } => {
                sql.push_str(&format!("FIRST_VALUE({})", column));
            }
            WindowFunctionType::LastValue { column } => {
                sql.push_str(&format!("LAST_VALUE({})", column));
            }
            WindowFunctionType::NthValue { column, n } => {
                sql.push_str(&format!("NTH_VALUE({}, {})", column, n));
            }
            WindowFunctionType::PercentRank => sql.push_str("PERCENT_RANK()"),
            WindowFunctionType::CumeDist => sql.push_str("CUME_DIST()"),
            WindowFunctionType::Sum { column } => sql.push_str(&format!("SUM({})", column)),
            WindowFunctionType::Avg { column } => sql.push_str(&format!("AVG({})", column)),
            WindowFunctionType::Count { column } => sql.push_str(&format!("COUNT({})", column)),
            WindowFunctionType::Max { column } => sql.push_str(&format!("MAX({})", column)),
            WindowFunctionType::Min { column } => sql.push_str(&format!("MIN({})", column)),
        }

        sql.push_str(" OVER (");

        // Add PARTITION BY
        if !self.partition_by.is_empty() {
            sql.push_str("PARTITION BY ");
            sql.push_str(&self.partition_by.join(", "));
            if !self.order_by.is_empty() {
                sql.push(' ');
            }
        }

        // Add ORDER BY
        if !self.order_by.is_empty() {
            sql.push_str("ORDER BY ");
            let order_strs: Vec<String> = self.order_by.iter().map(|col| {
                let mut s = col.column.clone();
                match col.direction {
                    SortDirection::Desc => s.push_str(" DESC"),
                    _ => {}
                }
                if let Some(nulls) = &col.nulls {
                    match nulls {
                        NullHandling::First => s.push_str(" NULLS FIRST"),
                        NullHandling::Last => s.push_str(" NULLS LAST"),
                    }
                }
                s
            }).collect();
            sql.push_str(&order_strs.join(", "));
        }

        // Add window frame if specified
        if let Some(frame) = &self.window_frame {
            sql.push(' ');
            match frame.mode {
                FrameMode::Rows => sql.push_str("ROWS"),
                FrameMode::Range => sql.push_str("RANGE"),
            }
            sql.push_str(" BETWEEN ");
            sql.push_str(&frame_bound_to_sql(&frame.start));
            sql.push_str(" AND ");
            if let Some(end) = &frame.end {
                sql.push_str(&frame_bound_to_sql(end));
            } else {
                sql.push_str("CURRENT ROW");
            }
        }

        sql.push(')');

        // Add alias if provided
        if let Some(alias) = alias {
            sql.push_str(&format!(" AS {}", alias));
        }

        sql
    }
}

fn frame_bound_to_sql(bound: &FrameBound) -> String {
    match bound {
        FrameBound::UnboundedPreceding => "UNBOUNDED PRECEDING".to_string(),
        FrameBound::CurrentRow => "CURRENT ROW".to_string(),
        FrameBound::UnboundedFollowing => "UNBOUNDED FOLLOWING".to_string(),
        FrameBound::Preceding(n) => format!("{} PRECEDING", n),
        FrameBound::Following(n) => format!("{} FOLLOWING", n),
    }
}

/// Common window function patterns
pub struct WindowPatterns;

impl WindowPatterns {
    /// Generate ranking with ties handled
    pub fn ranking_with_ties() -> String {
        r#"-- Ranking with different tie handling
ROW_NUMBER() OVER (PARTITION BY category ORDER BY score DESC) as row_rank,
RANK() OVER (PARTITION BY category ORDER BY score DESC) as rank_with_gaps,
DENSE_RANK() OVER (PARTITION BY category ORDER BY score DESC) as dense_rank"#.to_string()
    }

    /// Generate change detection pattern
    pub fn change_detection() -> String {
        r#"-- Detect changes from previous row
LAG(value, 1) OVER (PARTITION BY id ORDER BY date) as prev_value,
value - LAG(value, 1) OVER (PARTITION BY id ORDER BY date) as change,
CASE
    WHEN value != LAG(value, 1) OVER (PARTITION BY id ORDER BY date) THEN 1
    ELSE 0
END as is_changed"#.to_string()
    }

    /// Generate running totals pattern
    pub fn running_totals() -> String {
        r#"-- Running totals and averages
SUM(amount) OVER (PARTITION BY account ORDER BY date ROWS UNBOUNDED PRECEDING) as running_total,
AVG(amount) OVER (PARTITION BY account ORDER BY date ROWS BETWEEN 6 PRECEDING AND CURRENT ROW) as moving_avg_7,
COUNT(*) OVER (PARTITION BY account ORDER BY date ROWS UNBOUNDED PRECEDING) as running_count"#.to_string()
    }

    /// Generate percentile ranking
    pub fn percentile_ranking() -> String {
        r#"-- Percentile ranking
PERCENT_RANK() OVER (ORDER BY value) as percentile,
NTILE(4) OVER (ORDER BY value) as quartile,
NTILE(10) OVER (ORDER BY value) as decile,
CUME_DIST() OVER (ORDER BY value) as cumulative_dist"#.to_string()
    }

    /// Generate gaps and islands pattern
    pub fn gaps_and_islands() -> String {
        r#"-- Gaps and Islands pattern
WITH numbered AS (
    SELECT *,
        ROW_NUMBER() OVER (ORDER BY date) as rn,
        ROW_NUMBER() OVER (PARTITION BY status ORDER BY date) as status_rn
    FROM events
),
islands AS (
    SELECT *,
        rn - status_rn as island_id
    FROM numbered
)
SELECT
    status,
    MIN(date) as island_start,
    MAX(date) as island_end,
    COUNT(*) as island_length
FROM islands
GROUP BY status, island_id
ORDER BY island_start"#.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_row_number_generation() {
        let wf = WindowFunctionBuilder::row_number(
            vec!["department".to_string()],
            vec!["salary".to_string()],
        );
        let sql = wf.to_sql(Some("rank"));
        assert_eq!(sql, "ROW_NUMBER() OVER (PARTITION BY department ORDER BY salary) AS rank");
    }

    #[test]
    fn test_lead_lag_generation() {
        let lead = WindowFunctionBuilder::lead(
            "price".to_string(),
            1,
            vec!["product".to_string()],
            vec!["date".to_string()],
        );
        let sql = lead.to_sql(Some("next_price"));
        assert!(sql.contains("LEAD(price)"));
        assert!(sql.contains("PARTITION BY product"));
    }

    #[test]
    fn test_running_sum() {
        let sum = WindowFunctionBuilder::running_sum(
            "amount".to_string(),
            vec!["account".to_string()],
            vec!["date".to_string()],
        );
        let sql = sum.to_sql(Some("running_total"));
        assert!(sql.contains("SUM(amount)"));
        assert!(sql.contains("ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW"));
    }
}
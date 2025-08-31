use anyhow::Result;

use crate::data::data_view::DataView;
use crate::data::datatable::DataValue;

/// Simple WHERE clause implementation for testing
/// This extracts the WHERE clause as a string from the original SQL
pub struct SimpleWhereFilter;

impl SimpleWhereFilter {
    /// Apply a simple WHERE clause filter from SQL string
    pub fn apply_from_sql(view: DataView, sql: &str) -> Result<DataView> {
        // Extract WHERE clause from SQL
        let upper_sql = sql.to_uppercase();
        let where_start = upper_sql.find(" WHERE ");

        if where_start.is_none() {
            return Ok(view);
        }

        let where_start = where_start.unwrap() + 7; // Skip " WHERE "
        let sql_after_where = &sql[where_start..];

        // Find the end of WHERE clause (before ORDER BY, GROUP BY, LIMIT, etc.)
        let mut where_end = sql_after_where.len();
        for keyword in &[" ORDER BY", " GROUP BY", " LIMIT", " OFFSET"] {
            if let Some(pos) = sql_after_where.to_uppercase().find(keyword) {
                where_end = where_end.min(pos);
            }
        }

        let where_clause = sql_after_where[..where_end].trim();

        // Parse simple WHERE conditions
        Self::apply_simple_where(view, where_clause)
    }

    /// Apply a simple WHERE clause (supports basic = and > operators)
    fn apply_simple_where(view: DataView, where_clause: &str) -> Result<DataView> {
        // Handle AND/OR by splitting
        if where_clause.to_uppercase().contains(" AND ") {
            let parts: Vec<&str> = where_clause.split_terminator(" AND ").collect();
            let mut result = view;
            for part in parts {
                result = Self::apply_single_condition(result, part.trim())?;
            }
            return Ok(result);
        }

        if where_clause.to_uppercase().contains(" OR ") {
            // For OR, we'd need to combine results - skip for now
            return Ok(view);
        }

        // Single condition
        Self::apply_single_condition(view, where_clause)
    }

    /// Apply a single WHERE condition
    fn apply_single_condition(view: DataView, condition: &str) -> Result<DataView> {
        // Check for LIKE operator first
        if let Some(like_pos) = condition.to_uppercase().find(" LIKE ") {
            let column = condition[..like_pos].trim();
            let pattern_str = condition[like_pos + 6..].trim();

            // Remove quotes from pattern
            let pattern = if pattern_str.starts_with('\'') && pattern_str.ends_with('\'') {
                pattern_str[1..pattern_str.len() - 1].to_string()
            } else {
                pattern_str.to_string()
            };

            // Get column index
            let col_index = view
                .source()
                .get_column_index(column)
                .ok_or_else(|| anyhow::anyhow!("Column '{}' not found", column))?;

            // Convert SQL LIKE pattern to regex
            let regex_pattern = pattern.replace('%', ".*").replace('_', ".");

            let regex = regex::RegexBuilder::new(&format!("^{}$", regex_pattern))
                .case_insensitive(true)
                .build()
                .map_err(|e| anyhow::anyhow!("Invalid LIKE pattern: {}", e))?;

            return Ok(view.filter(move |table, row_idx| {
                if let Some(DataValue::String(s)) = table.get_value(row_idx, col_index) {
                    regex.is_match(s)
                } else {
                    false
                }
            }));
        }

        // Parse condition: column operator value
        let operators = vec![">=", "<=", "!=", "<>", "=", ">", "<"];
        let mut op = "";
        let mut op_pos = 0;

        for operator in operators {
            if let Some(pos) = condition.find(operator) {
                op = operator;
                op_pos = pos;
                break;
            }
        }

        if op.is_empty() {
            return Ok(view);
        }

        let column = condition[..op_pos].trim();
        let value_str = condition[op_pos + op.len()..].trim();

        // Get column index
        let col_index = view
            .source()
            .get_column_index(column)
            .ok_or_else(|| anyhow::anyhow!("Column '{}' not found", column))?;

        // Parse value (remove quotes if string)
        let value = if value_str.starts_with('\'') && value_str.ends_with('\'') {
            // String value
            value_str[1..value_str.len() - 1].to_string()
        } else {
            // Number or other value
            value_str.to_string()
        };

        // Apply filter
        Ok(view.filter(move |table, row_idx| {
            if let Some(cell_value) = table.get_value(row_idx, col_index) {
                match (cell_value, op) {
                    (DataValue::String(s), "=") => s == &value,
                    (DataValue::String(s), "!=") | (DataValue::String(s), "<>") => s != &value,
                    (DataValue::Integer(n), "=") => {
                        if let Ok(v) = value.parse::<i64>() {
                            *n == v
                        } else {
                            false
                        }
                    }
                    (DataValue::Integer(n), ">") => {
                        if let Ok(v) = value.parse::<i64>() {
                            *n > v
                        } else {
                            false
                        }
                    }
                    (DataValue::Integer(n), "<") => {
                        if let Ok(v) = value.parse::<i64>() {
                            *n < v
                        } else {
                            false
                        }
                    }
                    (DataValue::Integer(n), ">=") => {
                        if let Ok(v) = value.parse::<i64>() {
                            *n >= v
                        } else {
                            false
                        }
                    }
                    (DataValue::Integer(n), "<=") => {
                        if let Ok(v) = value.parse::<i64>() {
                            *n <= v
                        } else {
                            false
                        }
                    }
                    (DataValue::Float(n), "=") => {
                        if let Ok(v) = value.parse::<f64>() {
                            (*n - v).abs() < f64::EPSILON
                        } else {
                            false
                        }
                    }
                    (DataValue::Float(n), ">") => {
                        if let Ok(v) = value.parse::<f64>() {
                            *n > v
                        } else {
                            false
                        }
                    }
                    _ => false,
                }
            } else {
                false
            }
        }))
    }
}

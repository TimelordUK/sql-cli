use crate::data::datatable::{DataTable, DataValue};
use crate::sql::where_ast::{ComparisonOp, WhereExpr, WhereValue};
use anyhow::Result;

/// Evaluates WHERE clause expressions against DataTable rows
pub struct WhereEvaluator<'a> {
    table: &'a DataTable,
    column_indices: Vec<usize>,
}

impl<'a> WhereEvaluator<'a> {
    pub fn new(table: &'a DataTable) -> Self {
        let column_indices = (0..table.column_count()).collect();
        Self {
            table,
            column_indices,
        }
    }

    /// Evaluate a WHERE expression for a specific row
    pub fn evaluate(&self, expr: &WhereExpr, row_index: usize) -> Result<bool> {
        match expr {
            WhereExpr::And(left, right) => {
                Ok(self.evaluate(left, row_index)? && self.evaluate(right, row_index)?)
            }
            WhereExpr::Or(left, right) => {
                Ok(self.evaluate(left, row_index)? || self.evaluate(right, row_index)?)
            }
            WhereExpr::Not(inner) => Ok(!self.evaluate(inner, row_index)?),
            WhereExpr::Equal(column, value) => {
                self.evaluate_comparison(column, value, row_index, ComparisonOp::Equal)
            }
            WhereExpr::NotEqual(column, value) => {
                self.evaluate_comparison(column, value, row_index, ComparisonOp::NotEqual)
            }
            WhereExpr::GreaterThan(column, value) => {
                self.evaluate_comparison(column, value, row_index, ComparisonOp::GreaterThan)
            }
            WhereExpr::GreaterThanOrEqual(column, value) => {
                self.evaluate_comparison(column, value, row_index, ComparisonOp::GreaterThanOrEqual)
            }
            WhereExpr::LessThan(column, value) => {
                self.evaluate_comparison(column, value, row_index, ComparisonOp::LessThan)
            }
            WhereExpr::LessThanOrEqual(column, value) => {
                self.evaluate_comparison(column, value, row_index, ComparisonOp::LessThanOrEqual)
            }
            WhereExpr::Between(column, lower, upper) => {
                self.evaluate_between(column, lower, upper, row_index)
            }
            WhereExpr::In(column, values) => self.evaluate_in(column, values, row_index, false),
            WhereExpr::NotIn(column, values) => {
                Ok(!self.evaluate_in(column, values, row_index, false)?)
            }
            WhereExpr::InIgnoreCase(column, values) => {
                self.evaluate_in(column, values, row_index, true)
            }
            WhereExpr::NotInIgnoreCase(column, values) => {
                Ok(!self.evaluate_in(column, values, row_index, true)?)
            }
            WhereExpr::Like(column, pattern) => self.evaluate_like(column, pattern, row_index),
            WhereExpr::IsNull(column) => self.evaluate_is_null(column, row_index, true),
            WhereExpr::IsNotNull(column) => self.evaluate_is_null(column, row_index, false),
            WhereExpr::Contains(column, substring) => self.evaluate_string_method(
                column,
                substring,
                row_index,
                StringMethod::Contains,
                false,
            ),
            WhereExpr::StartsWith(column, prefix) => self.evaluate_string_method(
                column,
                prefix,
                row_index,
                StringMethod::StartsWith,
                false,
            ),
            WhereExpr::EndsWith(column, suffix) => self.evaluate_string_method(
                column,
                suffix,
                row_index,
                StringMethod::EndsWith,
                false,
            ),
            WhereExpr::ContainsIgnoreCase(column, substring) => self.evaluate_string_method(
                column,
                substring,
                row_index,
                StringMethod::Contains,
                true,
            ),
            WhereExpr::StartsWithIgnoreCase(column, prefix) => self.evaluate_string_method(
                column,
                prefix,
                row_index,
                StringMethod::StartsWith,
                true,
            ),
            WhereExpr::EndsWithIgnoreCase(column, suffix) => {
                self.evaluate_string_method(column, suffix, row_index, StringMethod::EndsWith, true)
            }
            WhereExpr::ToLower(column, op, value) => {
                self.evaluate_case_conversion(column, value, row_index, op, true)
            }
            WhereExpr::ToUpper(column, op, value) => {
                self.evaluate_case_conversion(column, value, row_index, op, false)
            }
            WhereExpr::IsNullOrEmpty(column) => self.evaluate_is_null_or_empty(column, row_index),
            WhereExpr::Length(column, op, length) => {
                self.evaluate_length(column, *length, row_index, op)
            }
        }
    }

    fn get_column_index(&self, column: &str) -> Result<usize> {
        let columns = self.table.column_names();
        columns
            .iter()
            .position(|c| c.eq_ignore_ascii_case(column))
            .ok_or_else(|| anyhow::anyhow!("Column '{}' not found", column))
    }

    fn get_cell_value(&self, column: &str, row_index: usize) -> Result<Option<DataValue>> {
        let col_index = self.get_column_index(column)?;
        Ok(self.table.get_value(row_index, col_index).cloned())
    }

    fn evaluate_comparison(
        &self,
        column: &str,
        value: &WhereValue,
        row_index: usize,
        op: ComparisonOp,
    ) -> Result<bool> {
        let cell_value = self.get_cell_value(column, row_index)?;

        match cell_value {
            None | Some(DataValue::Null) => Ok(false),
            Some(data_val) => {
                let result = match (&data_val, value) {
                    // Number comparisons
                    (DataValue::Integer(a), WhereValue::Number(b)) => {
                        compare_numbers(*a as f64, *b, &op)
                    }
                    (DataValue::Float(a), WhereValue::Number(b)) => compare_numbers(*a, *b, &op),
                    // String comparisons
                    (DataValue::String(a), WhereValue::String(b)) => compare_strings(a, b, &op),
                    // String to number coercion
                    (DataValue::String(a), WhereValue::Number(b)) => {
                        if let Ok(a_num) = a.parse::<f64>() {
                            compare_numbers(a_num, *b, &op)
                        } else {
                            false
                        }
                    }
                    (DataValue::Integer(a), WhereValue::String(b)) => {
                        if let Ok(b_num) = b.parse::<f64>() {
                            compare_numbers(*a as f64, b_num, &op)
                        } else {
                            false
                        }
                    }
                    (DataValue::Float(a), WhereValue::String(b)) => {
                        if let Ok(b_num) = b.parse::<f64>() {
                            compare_numbers(*a, b_num, &op)
                        } else {
                            false
                        }
                    }
                    // Boolean comparisons
                    (DataValue::Boolean(a), WhereValue::String(b)) => {
                        let b_bool = b.eq_ignore_ascii_case("true");
                        compare_bools(*a, b_bool, &op)
                    }
                    // Null comparisons
                    (_, WhereValue::Null) => {
                        matches!(op, ComparisonOp::NotEqual)
                    }
                    _ => false,
                };
                Ok(result)
            }
        }
    }

    fn evaluate_between(
        &self,
        column: &str,
        lower: &WhereValue,
        upper: &WhereValue,
        row_index: usize,
    ) -> Result<bool> {
        let cell_value = self.get_cell_value(column, row_index)?;

        match cell_value {
            None | Some(DataValue::Null) => Ok(false),
            Some(data_val) => {
                let ge_lower =
                    self.compare_value(&data_val, lower, &ComparisonOp::GreaterThanOrEqual);
                let le_upper = self.compare_value(&data_val, upper, &ComparisonOp::LessThanOrEqual);
                Ok(ge_lower && le_upper)
            }
        }
    }

    fn evaluate_in(
        &self,
        column: &str,
        values: &[WhereValue],
        row_index: usize,
        ignore_case: bool,
    ) -> Result<bool> {
        let cell_value = self.get_cell_value(column, row_index)?;

        match cell_value {
            None | Some(DataValue::Null) => Ok(false),
            Some(data_val) => {
                for value in values {
                    if ignore_case {
                        if self.compare_ignore_case(&data_val, value) {
                            return Ok(true);
                        }
                    } else if self.compare_value(&data_val, value, &ComparisonOp::Equal) {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
        }
    }

    fn evaluate_like(&self, column: &str, pattern: &str, row_index: usize) -> Result<bool> {
        let cell_value = self.get_cell_value(column, row_index)?;

        match cell_value {
            Some(DataValue::String(s)) => {
                // Convert SQL LIKE pattern to regex
                let regex_pattern = pattern.replace('%', ".*").replace('_', ".");

                // Use case-insensitive matching
                let regex = regex::RegexBuilder::new(&format!("^{}$", regex_pattern))
                    .case_insensitive(true)
                    .build()
                    .map_err(|e| anyhow::anyhow!("Invalid LIKE pattern: {}", e))?;

                Ok(regex.is_match(&s))
            }
            _ => Ok(false),
        }
    }

    fn evaluate_is_null(&self, column: &str, row_index: usize, expect_null: bool) -> Result<bool> {
        let cell_value = self.get_cell_value(column, row_index)?;
        let is_null = matches!(cell_value, None | Some(DataValue::Null));
        Ok(is_null == expect_null)
    }

    fn evaluate_is_null_or_empty(&self, column: &str, row_index: usize) -> Result<bool> {
        let cell_value = self.get_cell_value(column, row_index)?;
        Ok(match cell_value {
            None | Some(DataValue::Null) => true,
            Some(DataValue::String(s)) => s.is_empty(),
            _ => false,
        })
    }

    fn evaluate_string_method(
        &self,
        column: &str,
        pattern: &str,
        row_index: usize,
        method: StringMethod,
        ignore_case: bool,
    ) -> Result<bool> {
        let cell_value = self.get_cell_value(column, row_index)?;

        match cell_value {
            Some(DataValue::String(s)) => {
                let (s, pattern) = if ignore_case {
                    (s.to_lowercase(), pattern.to_lowercase())
                } else {
                    (s, pattern.to_string())
                };

                Ok(match method {
                    StringMethod::Contains => s.contains(&pattern),
                    StringMethod::StartsWith => s.starts_with(&pattern),
                    StringMethod::EndsWith => s.ends_with(&pattern),
                })
            }
            _ => Ok(false),
        }
    }

    fn evaluate_case_conversion(
        &self,
        column: &str,
        value: &str,
        row_index: usize,
        op: &ComparisonOp,
        to_lower: bool,
    ) -> Result<bool> {
        let cell_value = self.get_cell_value(column, row_index)?;

        match cell_value {
            Some(DataValue::String(s)) => {
                let converted = if to_lower {
                    s.to_lowercase()
                } else {
                    s.to_uppercase()
                };
                Ok(compare_strings(&converted, value, op))
            }
            _ => Ok(false),
        }
    }

    fn evaluate_length(
        &self,
        column: &str,
        length: i64,
        row_index: usize,
        op: &ComparisonOp,
    ) -> Result<bool> {
        let cell_value = self.get_cell_value(column, row_index)?;

        match cell_value {
            Some(DataValue::String(s)) => {
                let len = s.len() as i64;
                Ok(compare_numbers(len as f64, length as f64, op))
            }
            _ => Ok(false),
        }
    }

    fn compare_value(
        &self,
        data_val: &DataValue,
        where_val: &WhereValue,
        op: &ComparisonOp,
    ) -> bool {
        match (data_val, where_val) {
            (DataValue::Integer(a), WhereValue::Number(b)) => compare_numbers(*a as f64, *b, op),
            (DataValue::Float(a), WhereValue::Number(b)) => compare_numbers(*a, *b, op),
            (DataValue::String(a), WhereValue::String(b)) => compare_strings(a, b, op),
            _ => false,
        }
    }

    fn compare_ignore_case(&self, data_val: &DataValue, where_val: &WhereValue) -> bool {
        match (data_val, where_val) {
            (DataValue::String(a), WhereValue::String(b)) => a.eq_ignore_ascii_case(b),
            _ => self.compare_value(data_val, where_val, &ComparisonOp::Equal),
        }
    }
}

enum StringMethod {
    Contains,
    StartsWith,
    EndsWith,
}

fn compare_numbers(a: f64, b: f64, op: &ComparisonOp) -> bool {
    match op {
        ComparisonOp::Equal => (a - b).abs() < f64::EPSILON,
        ComparisonOp::NotEqual => (a - b).abs() >= f64::EPSILON,
        ComparisonOp::GreaterThan => a > b,
        ComparisonOp::GreaterThanOrEqual => a >= b,
        ComparisonOp::LessThan => a < b,
        ComparisonOp::LessThanOrEqual => a <= b,
    }
}

fn compare_strings(a: &str, b: &str, op: &ComparisonOp) -> bool {
    match op {
        ComparisonOp::Equal => a == b,
        ComparisonOp::NotEqual => a != b,
        ComparisonOp::GreaterThan => a > b,
        ComparisonOp::GreaterThanOrEqual => a >= b,
        ComparisonOp::LessThan => a < b,
        ComparisonOp::LessThanOrEqual => a <= b,
    }
}

fn compare_bools(a: bool, b: bool, op: &ComparisonOp) -> bool {
    match op {
        ComparisonOp::Equal => a == b,
        ComparisonOp::NotEqual => a != b,
        _ => false,
    }
}

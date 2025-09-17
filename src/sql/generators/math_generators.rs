use super::{create_two_column_table, TableGenerator};
use crate::data::datatable::{DataColumn, DataTable, DataValue};
use anyhow::Result;
use std::sync::Arc;

/// Generate Collatz sequence (3n+1 sequence)
/// Famous unsolved problem: does every positive integer reach 1?
pub struct Collatz;

impl TableGenerator for Collatz {
    fn name(&self) -> &str {
        "COLLATZ"
    }

    fn columns(&self) -> Vec<DataColumn> {
        vec![DataColumn::new("step"), DataColumn::new("value")]
    }

    fn generate(&self, args: Vec<DataValue>) -> Result<Arc<DataTable>> {
        if args.len() != 1 {
            return Err(anyhow::anyhow!(
                "COLLATZ expects 1 argument (starting number)"
            ));
        }

        let start = match &args[0] {
            DataValue::Integer(n) => *n as u64,
            DataValue::Float(f) => *f as u64,
            _ => {
                return Err(anyhow::anyhow!(
                    "COLLATZ argument must be a positive number"
                ))
            }
        };

        if start == 0 {
            return Err(anyhow::anyhow!("COLLATZ starting number must be positive"));
        }

        let sequence = collatz_sequence(start);
        let rows: Vec<(DataValue, DataValue)> = sequence
            .into_iter()
            .enumerate()
            .map(|(i, val)| (DataValue::Integer(i as i64), DataValue::Integer(val as i64)))
            .collect();

        Ok(create_two_column_table("collatz", "step", "value", rows))
    }

    fn description(&self) -> &str {
        "Generate Collatz sequence (3n+1 sequence) until reaching 1"
    }

    fn arg_count(&self) -> usize {
        1
    }
}

/// Generate Pascal's triangle
pub struct PascalTriangle;

impl TableGenerator for PascalTriangle {
    fn name(&self) -> &str {
        "PASCAL_TRIANGLE"
    }

    fn columns(&self) -> Vec<DataColumn> {
        vec![
            DataColumn::new("row"),
            DataColumn::new("position"),
            DataColumn::new("value"),
        ]
    }

    fn generate(&self, args: Vec<DataValue>) -> Result<Arc<DataTable>> {
        if args.len() != 1 {
            return Err(anyhow::anyhow!(
                "PASCAL_TRIANGLE expects 1 argument (number of rows)"
            ));
        }

        let rows = match &args[0] {
            DataValue::Integer(n) => *n as usize,
            DataValue::Float(f) => *f as usize,
            _ => return Err(anyhow::anyhow!("PASCAL_TRIANGLE rows must be a number")),
        };

        let mut table = DataTable::new("pascal");
        table.add_column(DataColumn::new("row"));
        table.add_column(DataColumn::new("position"));
        table.add_column(DataColumn::new("value"));

        for row in 0..rows {
            let mut current_row = vec![1i64];
            for col in 1..=row {
                let val = if col == row {
                    1
                } else {
                    // Use the recurrence relation: C(n,k) = C(n-1,k-1) + C(n-1,k)
                    let prev_val = if col > 0 && col <= current_row.len() {
                        current_row[col - 1]
                    } else {
                        0
                    };
                    prev_val * (row as i64 - col as i64 + 1) / col as i64
                };
                current_row.push(val);
            }

            for (pos, &val) in current_row.iter().enumerate() {
                table
                    .add_row(crate::data::datatable::DataRow::new(vec![
                        DataValue::Integer(row as i64),
                        DataValue::Integer(pos as i64),
                        DataValue::Integer(val),
                    ]))
                    .unwrap();
            }
        }

        Ok(Arc::new(table))
    }

    fn description(&self) -> &str {
        "Generate Pascal's triangle with (row, position, value) format"
    }

    fn arg_count(&self) -> usize {
        1
    }
}

/// Generate triangular numbers: 1, 3, 6, 10, 15, ...
pub struct TriangularNumbers;

impl TableGenerator for TriangularNumbers {
    fn name(&self) -> &str {
        "TRIANGULAR"
    }

    fn columns(&self) -> Vec<DataColumn> {
        vec![DataColumn::new("n"), DataColumn::new("value")]
    }

    fn generate(&self, args: Vec<DataValue>) -> Result<Arc<DataTable>> {
        if args.len() != 1 {
            return Err(anyhow::anyhow!("TRIANGULAR expects 1 argument (count)"));
        }

        let count = match &args[0] {
            DataValue::Integer(n) => *n as usize,
            DataValue::Float(f) => *f as usize,
            _ => return Err(anyhow::anyhow!("TRIANGULAR count must be a number")),
        };

        let mut rows = Vec::new();
        for n in 1..=count {
            let value = n * (n + 1) / 2;
            rows.push((
                DataValue::Integer(n as i64),
                DataValue::Integer(value as i64),
            ));
        }

        Ok(create_two_column_table("triangular", "n", "value", rows))
    }

    fn description(&self) -> &str {
        "Generate triangular numbers (sum of first n natural numbers)"
    }

    fn arg_count(&self) -> usize {
        1
    }
}

/// Generate perfect squares
pub struct Squares;

impl TableGenerator for Squares {
    fn name(&self) -> &str {
        "SQUARES"
    }

    fn columns(&self) -> Vec<DataColumn> {
        vec![DataColumn::new("n"), DataColumn::new("square")]
    }

    fn generate(&self, args: Vec<DataValue>) -> Result<Arc<DataTable>> {
        if args.len() != 1 {
            return Err(anyhow::anyhow!("SQUARES expects 1 argument (count)"));
        }

        let count = match &args[0] {
            DataValue::Integer(n) => *n as usize,
            DataValue::Float(f) => *f as usize,
            _ => return Err(anyhow::anyhow!("SQUARES count must be a number")),
        };

        let mut rows = Vec::new();
        for n in 1..=count {
            rows.push((
                DataValue::Integer(n as i64),
                DataValue::Integer((n * n) as i64),
            ));
        }

        Ok(create_two_column_table("squares", "n", "square", rows))
    }

    fn description(&self) -> &str {
        "Generate perfect squares (n²)"
    }

    fn arg_count(&self) -> usize {
        1
    }
}

/// Generate factorial sequence
pub struct Factorials;

impl TableGenerator for Factorials {
    fn name(&self) -> &str {
        "FACTORIALS"
    }

    fn columns(&self) -> Vec<DataColumn> {
        vec![DataColumn::new("n"), DataColumn::new("factorial")]
    }

    fn generate(&self, args: Vec<DataValue>) -> Result<Arc<DataTable>> {
        if args.len() != 1 {
            return Err(anyhow::anyhow!("FACTORIALS expects 1 argument (count)"));
        }

        let count = match &args[0] {
            DataValue::Integer(n) => *n as usize,
            DataValue::Float(f) => *f as usize,
            _ => return Err(anyhow::anyhow!("FACTORIALS count must be a number")),
        };

        let mut rows = Vec::new();
        let mut factorial: i64 = 1;

        rows.push((DataValue::Integer(0), DataValue::Integer(1)));

        for n in 1..=count.min(20) {
            // Limit to 20! to avoid overflow
            factorial = factorial.saturating_mul(n as i64);
            rows.push((DataValue::Integer(n as i64), DataValue::Integer(factorial)));
        }

        Ok(create_two_column_table(
            "factorials",
            "n",
            "factorial",
            rows,
        ))
    }

    fn description(&self) -> &str {
        "Generate factorial sequence (n!)"
    }

    fn arg_count(&self) -> usize {
        1
    }
}

/// Helper function to generate Collatz sequence
fn collatz_sequence(mut n: u64) -> Vec<u64> {
    let mut sequence = vec![n];

    while n != 1 {
        if n % 2 == 0 {
            n /= 2;
        } else {
            n = n.saturating_mul(3).saturating_add(1);
        }
        sequence.push(n);

        // Safety limit to prevent infinite loops
        if sequence.len() > 1000 {
            break;
        }
    }

    sequence
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collatz_sequence() {
        assert_eq!(collatz_sequence(5), vec![5, 16, 8, 4, 2, 1]);
        assert_eq!(
            collatz_sequence(7),
            vec![7, 22, 11, 34, 17, 52, 26, 13, 40, 20, 10, 5, 16, 8, 4, 2, 1]
        );
    }
}

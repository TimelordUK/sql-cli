use anyhow::{anyhow, Result};
use std::collections::VecDeque;

use super::{ArgCount, FunctionCategory, FunctionSignature, SqlFunction};
use crate::data::datatable::DataValue;

/// DELTAS function - returns the difference from the previous value in a series
/// Similar to q/kdb's deltas function
pub struct DeltasFunction;

impl SqlFunction for DeltasFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "DELTAS",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Variadic,
            description: "Returns differences from previous values in a series",
            returns: "ARRAY",
            examples: vec![
                "SELECT DELTAS(1, 3, 6, 10, 15) -- Returns: 1, 2, 3, 4, 5",
                "SELECT DELTAS(price) as price_changes FROM stocks ORDER BY date",
                "SELECT DELTAS(100, 95, 97, 92, 98) -- Returns: 100, -5, 2, -5, 6",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.is_empty() {
            return Ok(DataValue::String("[]".to_string()));
        }

        let mut deltas = Vec::new();
        let mut prev: Option<f64> = None;

        for arg in args {
            let current = match arg {
                DataValue::Integer(i) => *i as f64,
                DataValue::Float(f) => *f,
                DataValue::Null => {
                    deltas.push("null".to_string());
                    prev = None;
                    continue;
                }
                _ => return Err(anyhow!("DELTAS expects numeric values")),
            };

            match prev {
                None => {
                    // First value is returned as-is
                    if current.fract() == 0.0 && current.abs() < 1e10 {
                        deltas.push(format!("{}", current as i64));
                    } else {
                        deltas.push(format!("{}", current));
                    }
                }
                Some(p) => {
                    let delta = current - p;
                    if delta.fract() == 0.0 && delta.abs() < 1e10 {
                        deltas.push(format!("{}", delta as i64));
                    } else {
                        deltas.push(format!("{}", delta));
                    }
                }
            }
            prev = Some(current);
        }

        // Return as array-like string for now
        Ok(DataValue::String(format!("[{}]", deltas.join(", "))))
    }
}

/// SUMS function - returns running sum of a series
/// Similar to q/kdb's sums function
pub struct SumsFunction;

impl SqlFunction for SumsFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "SUMS",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Variadic,
            description: "Returns running sum of values in a series",
            returns: "ARRAY",
            examples: vec![
                "SELECT SUMS(1, 2, 3, 4, 5) -- Returns: 1, 3, 6, 10, 15",
                "SELECT SUMS(daily_revenue) as cumulative_revenue FROM sales ORDER BY date",
                "SELECT SUMS(10, -5, 8, -3, 6) -- Returns: 10, 5, 13, 10, 16",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.is_empty() {
            return Ok(DataValue::String("[]".to_string()));
        }

        let mut sums = Vec::new();
        let mut running_sum = 0.0;

        for arg in args {
            match arg {
                DataValue::Integer(i) => {
                    running_sum += *i as f64;
                }
                DataValue::Float(f) => {
                    running_sum += f;
                }
                DataValue::Null => {
                    sums.push("null".to_string());
                    continue;
                }
                _ => return Err(anyhow!("SUMS expects numeric values")),
            }

            if running_sum.fract() == 0.0 && running_sum.abs() < 1e10 {
                sums.push(format!("{}", running_sum as i64));
            } else {
                sums.push(format!("{}", running_sum));
            }
        }

        Ok(DataValue::String(format!("[{}]", sums.join(", "))))
    }
}

/// MAVG function - returns moving average over a window
pub struct MovingAverageFunction;

impl SqlFunction for MovingAverageFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "MAVG",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Variadic,
            description: "Returns moving average with specified window size (first arg is window)",
            returns: "ARRAY",
            examples: vec![
                "SELECT MAVG(3, 10, 20, 30, 40, 50) -- 3-period moving avg",
                "SELECT MAVG(5, price) as ma5 FROM stocks ORDER BY date",
                "SELECT MAVG(2, 1, 2, 3, 4, 5) -- Returns: 1, 1.5, 2.5, 3.5, 4.5",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() < 2 {
            return Err(anyhow!("MAVG requires window size and at least one value"));
        }

        // First argument is the window size
        let window_size = match &args[0] {
            DataValue::Integer(w) if *w > 0 => *w as usize,
            _ => return Err(anyhow!("MAVG window size must be a positive integer")),
        };

        let mut averages = Vec::new();
        let mut window: VecDeque<f64> = VecDeque::with_capacity(window_size);

        // Process the remaining values
        for arg in &args[1..] {
            let value = match arg {
                DataValue::Integer(i) => *i as f64,
                DataValue::Float(f) => *f,
                DataValue::Null => {
                    averages.push("null".to_string());
                    continue;
                }
                _ => return Err(anyhow!("MAVG expects numeric values")),
            };

            window.push_back(value);
            if window.len() > window_size {
                window.pop_front();
            }

            let avg = window.iter().sum::<f64>() / window.len() as f64;
            
            if avg.fract() == 0.0 && avg.abs() < 1e10 {
                averages.push(format!("{}", avg as i64));
            } else {
                averages.push(format!("{:.2}", avg));
            }
        }

        Ok(DataValue::String(format!("[{}]", averages.join(", "))))
    }
}

/// PCT_CHANGE function - returns percentage change from previous value
pub struct PercentChangeFunction;

impl SqlFunction for PercentChangeFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "PCT_CHANGE",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Variadic,
            description: "Returns percentage change from previous values",
            returns: "ARRAY",
            examples: vec![
                "SELECT PCT_CHANGE(100, 110, 121, 108.9) -- Returns: null, 10%, 10%, -10%",
                "SELECT PCT_CHANGE(price) as price_pct_change FROM stocks ORDER BY date",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.is_empty() {
            return Ok(DataValue::String("[]".to_string()));
        }

        let mut changes = Vec::new();
        let mut prev: Option<f64> = None;

        for arg in args {
            let current = match arg {
                DataValue::Integer(i) => *i as f64,
                DataValue::Float(f) => *f,
                DataValue::Null => {
                    changes.push("null".to_string());
                    prev = None;
                    continue;
                }
                _ => return Err(anyhow!("PCT_CHANGE expects numeric values")),
            };

            match prev {
                None => {
                    changes.push("null".to_string()); // First value has no previous
                }
                Some(p) if p != 0.0 => {
                    let pct = ((current - p) / p) * 100.0;
                    changes.push(format!("{:.2}%", pct));
                }
                Some(_) => {
                    changes.push("inf".to_string()); // Division by zero
                }
            }
            prev = Some(current);
        }

        Ok(DataValue::String(format!("[{}]", changes.join(", "))))
    }
}

/// RANK function - returns rank of values (1 = smallest)
pub struct RankFunction;

impl SqlFunction for RankFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "RANK",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Variadic,
            description: "Returns rank of each value (1 = smallest)",
            returns: "ARRAY",
            examples: vec![
                "SELECT RANK(50, 20, 30, 20, 40) -- Returns: 5, 1, 3, 1, 4",
                "SELECT RANK(score) as score_rank FROM students",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.is_empty() {
            return Ok(DataValue::String("[]".to_string()));
        }

        // Collect values with their original indices
        let mut indexed_values: Vec<(usize, f64)> = Vec::new();
        
        for (i, arg) in args.iter().enumerate() {
            match arg {
                DataValue::Integer(v) => indexed_values.push((i, *v as f64)),
                DataValue::Float(v) => indexed_values.push((i, *v)),
                DataValue::Null => indexed_values.push((i, f64::NAN)),
                _ => return Err(anyhow!("RANK expects numeric values")),
            }
        }

        // Sort by value to determine ranks
        let mut sorted = indexed_values.clone();
        sorted.sort_by(|a, b| {
            if a.1.is_nan() && b.1.is_nan() {
                std::cmp::Ordering::Equal
            } else if a.1.is_nan() {
                std::cmp::Ordering::Greater  // NaN goes to end
            } else if b.1.is_nan() {
                std::cmp::Ordering::Less
            } else {
                a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal)
            }
        });

        // Assign ranks (handling ties)
        let mut ranks = vec![0; args.len()];
        let mut current_rank = 1;
        
        for i in 0..sorted.len() {
            if sorted[i].1.is_nan() {
                ranks[sorted[i].0] = 0; // NaN gets rank 0 or could be null
            } else {
                if i > 0 && !sorted[i-1].1.is_nan() && (sorted[i].1 - sorted[i-1].1).abs() > 1e-10 {
                    current_rank = i + 1;
                }
                ranks[sorted[i].0] = current_rank;
            }
        }

        let rank_strings: Vec<String> = ranks.iter()
            .map(|&r| if r == 0 { "null".to_string() } else { r.to_string() })
            .collect();

        Ok(DataValue::String(format!("[{}]", rank_strings.join(", "))))
    }
}

/// CUMMAX function - returns cumulative maximum
pub struct CumulativeMaxFunction;

impl SqlFunction for CumulativeMaxFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "CUMMAX",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Variadic,
            description: "Returns cumulative maximum of values",
            returns: "ARRAY",
            examples: vec![
                "SELECT CUMMAX(3, 1, 4, 1, 5, 9, 2) -- Returns: 3, 3, 4, 4, 5, 9, 9",
                "SELECT CUMMAX(high_price) as all_time_high FROM stocks ORDER BY date",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.is_empty() {
            return Ok(DataValue::String("[]".to_string()));
        }

        let mut results = Vec::new();
        let mut current_max: Option<f64> = None;

        for arg in args {
            let value = match arg {
                DataValue::Integer(i) => *i as f64,
                DataValue::Float(f) => *f,
                DataValue::Null => {
                    if let Some(max) = current_max {
                        if max.fract() == 0.0 && max.abs() < 1e10 {
                            results.push(format!("{}", max as i64));
                        } else {
                            results.push(format!("{}", max));
                        }
                    } else {
                        results.push("null".to_string());
                    }
                    continue;
                }
                _ => return Err(anyhow!("CUMMAX expects numeric values")),
            };

            current_max = match current_max {
                None => Some(value),
                Some(max) => Some(max.max(value)),
            };

            let max = current_max.unwrap();
            if max.fract() == 0.0 && max.abs() < 1e10 {
                results.push(format!("{}", max as i64));
            } else {
                results.push(format!("{}", max));
            }
        }

        Ok(DataValue::String(format!("[{}]", results.join(", "))))
    }
}

/// CUMMIN function - returns cumulative minimum
pub struct CumulativeMinFunction;

impl SqlFunction for CumulativeMinFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "CUMMIN",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Variadic,
            description: "Returns cumulative minimum of values",
            returns: "ARRAY",
            examples: vec![
                "SELECT CUMMIN(3, 1, 4, 1, 5, 9, 2) -- Returns: 3, 1, 1, 1, 1, 1, 1",
                "SELECT CUMMIN(low_price) as all_time_low FROM stocks ORDER BY date",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.is_empty() {
            return Ok(DataValue::String("[]".to_string()));
        }

        let mut results = Vec::new();
        let mut current_min: Option<f64> = None;

        for arg in args {
            let value = match arg {
                DataValue::Integer(i) => *i as f64,
                DataValue::Float(f) => *f,
                DataValue::Null => {
                    if let Some(min) = current_min {
                        if min.fract() == 0.0 && min.abs() < 1e10 {
                            results.push(format!("{}", min as i64));
                        } else {
                            results.push(format!("{}", min));
                        }
                    } else {
                        results.push("null".to_string());
                    }
                    continue;
                }
                _ => return Err(anyhow!("CUMMIN expects numeric values")),
            };

            current_min = match current_min {
                None => Some(value),
                Some(min) => Some(min.min(value)),
            };

            let min = current_min.unwrap();
            if min.fract() == 0.0 && min.abs() < 1e10 {
                results.push(format!("{}", min as i64));
            } else {
                results.push(format!("{}", min));
            }
        }

        Ok(DataValue::String(format!("[{}]", results.join(", "))))
    }
}

/// Register all analytics functions
pub fn register_analytics_functions(registry: &mut super::FunctionRegistry) {
    registry.register(Box::new(DeltasFunction));
    registry.register(Box::new(SumsFunction));
    registry.register(Box::new(MovingAverageFunction));
    registry.register(Box::new(PercentChangeFunction));
    registry.register(Box::new(RankFunction));
    registry.register(Box::new(CumulativeMaxFunction));
    registry.register(Box::new(CumulativeMinFunction));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deltas() {
        let func = DeltasFunction;
        let args = vec![
            DataValue::Integer(1),
            DataValue::Integer(3),
            DataValue::Integer(6),
            DataValue::Integer(10),
            DataValue::Integer(15),
        ];
        
        let result = func.evaluate(&args).unwrap();
        match result {
            DataValue::String(s) => assert_eq!(s, "[1, 2, 3, 4, 5]"),
            _ => panic!("Expected string result"),
        }
    }

    #[test]
    fn test_sums() {
        let func = SumsFunction;
        let args = vec![
            DataValue::Integer(1),
            DataValue::Integer(2),
            DataValue::Integer(3),
            DataValue::Integer(4),
            DataValue::Integer(5),
        ];
        
        let result = func.evaluate(&args).unwrap();
        match result {
            DataValue::String(s) => assert_eq!(s, "[1, 3, 6, 10, 15]"),
            _ => panic!("Expected string result"),
        }
    }

    #[test]
    fn test_moving_average() {
        let func = MovingAverageFunction;
        let args = vec![
            DataValue::Integer(2), // window size
            DataValue::Integer(1),
            DataValue::Integer(2),
            DataValue::Integer(3),
            DataValue::Integer(4),
            DataValue::Integer(5),
        ];
        
        let result = func.evaluate(&args).unwrap();
        match result {
            DataValue::String(s) => {
                assert!(s.contains("1")); // First value
                assert!(s.contains("1.5")); // Avg of 1,2
                assert!(s.contains("2.5")); // Avg of 2,3
            }
            _ => panic!("Expected string result"),
        }
    }

    #[test]
    fn test_rank() {
        let func = RankFunction;
        let args = vec![
            DataValue::Integer(50),
            DataValue::Integer(20),
            DataValue::Integer(30),
            DataValue::Integer(20),
            DataValue::Integer(40),
        ];
        
        let result = func.evaluate(&args).unwrap();
        match result {
            DataValue::String(s) => {
                assert!(s.contains("5")); // 50 is rank 5
                assert!(s.contains("1")); // 20 is rank 1 (appears twice)
                assert!(s.contains("3")); // 30 is rank 3
            }
            _ => panic!("Expected string result"),
        }
    }
}
use crate::data::datatable::DataValue;
use crate::sql::functions::{ArgCount, FunctionCategory, FunctionSignature, SqlFunction};
use anyhow::Result;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Mutex;

// Global memoization state for GROUP_NUM function
// This ensures consistency across the entire query
lazy_static! {
    static ref GROUP_NUM_MEMO: Mutex<HashMap<String, HashMap<String, i64>>> =
        Mutex::new(HashMap::new());
}

/// GROUP_NUM function - assigns a unique number (starting from 0) to each distinct value
/// Maintains consistency across the entire query execution
pub struct GroupNumFunction;

impl GroupNumFunction {
    pub fn new() -> Self {
        Self
    }

    /// Clear all memoization (should be called before each new query)
    pub fn clear_memoization() {
        let mut memo = GROUP_NUM_MEMO.lock().unwrap();
        memo.clear();
    }
}

impl SqlFunction for GroupNumFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "GROUP_NUM",
            category: FunctionCategory::Aggregate,
            arg_count: ArgCount::Fixed(1),
            description: "Assigns unique sequential numbers (starting from 0) to distinct values",
            returns: "Integer - unique number for each distinct value",
            examples: vec![
                "SELECT order_id, GROUP_NUM(order_id) as grp_num FROM orders",
                "SELECT customer, GROUP_NUM(customer) as cust_num FROM sales",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            anyhow::bail!("GROUP_NUM requires exactly 1 argument");
        }

        // Convert the value to a string for consistent hashing
        let value_str = match &args[0] {
            DataValue::Null => return Ok(DataValue::Null),
            DataValue::String(s) => s.clone(),
            DataValue::InternedString(s) => s.to_string(),
            DataValue::Integer(i) => i.to_string(),
            DataValue::Float(f) => f.to_string(),
            DataValue::Boolean(b) => b.to_string(),
            DataValue::DateTime(dt) => dt.to_string(),
            DataValue::Vector(v) => {
                let components: Vec<String> = v.iter().map(|f| f.to_string()).collect();
                format!("[{}]", components.join(","))
            }
        };

        // For now, use a default column identifier
        // In a full implementation, we'd track which column this is for
        let column_id = "_default_";

        let mut memo = GROUP_NUM_MEMO.lock().unwrap();
        let column_map = memo
            .entry(column_id.to_string())
            .or_insert_with(HashMap::new);

        // Check if we've seen this value before
        if let Some(&num) = column_map.get(&value_str) {
            Ok(DataValue::Integer(num))
        } else {
            // Assign a new number
            let new_num = column_map.len() as i64;
            column_map.insert(value_str, new_num);
            Ok(DataValue::Integer(new_num))
        }
    }
}

/// Extended version that can track column context
pub struct GroupNumWithContext;

impl GroupNumWithContext {
    pub fn new() -> Self {
        Self
    }

    /// Evaluate with column context
    pub fn evaluate_with_context(&self, value: &DataValue, column_name: &str) -> Result<DataValue> {
        if matches!(value, DataValue::Null) {
            return Ok(DataValue::Null);
        }

        let value_str = match value {
            DataValue::String(s) => s.clone(),
            DataValue::InternedString(s) => s.to_string(),
            DataValue::Integer(i) => i.to_string(),
            DataValue::Float(f) => f.to_string(),
            DataValue::Boolean(b) => b.to_string(),
            DataValue::DateTime(dt) => dt.to_string(),
            DataValue::Vector(v) => {
                let components: Vec<String> = v.iter().map(|f| f.to_string()).collect();
                format!("[{}]", components.join(","))
            }
            DataValue::Null => unreachable!(),
        };

        let mut memo = GROUP_NUM_MEMO.lock().unwrap();
        let column_map = memo
            .entry(column_name.to_string())
            .or_insert_with(HashMap::new);

        if let Some(&num) = column_map.get(&value_str) {
            Ok(DataValue::Integer(num))
        } else {
            let new_num = column_map.len() as i64;
            column_map.insert(value_str, new_num);
            Ok(DataValue::Integer(new_num))
        }
    }

    /// Clear memoization for a specific column
    pub fn clear_column(&self, column_name: &str) {
        let mut memo = GROUP_NUM_MEMO.lock().unwrap();
        memo.remove(column_name);
    }

    /// Clear all memoization
    pub fn clear_all(&self) {
        let mut memo = GROUP_NUM_MEMO.lock().unwrap();
        memo.clear();
    }
}

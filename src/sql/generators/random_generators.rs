use super::TableGenerator;
use crate::data::datatable::{DataColumn, DataRow, DataTable, DataValue};
use anyhow::Result;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::sync::Arc;

/// Generate random integers in a range
pub struct RandomIntegers;

impl TableGenerator for RandomIntegers {
    fn name(&self) -> &str {
        "RANDOM_INT"
    }

    fn columns(&self) -> Vec<DataColumn> {
        vec![DataColumn::new("id"), DataColumn::new("value")]
    }

    fn generate(&self, args: Vec<DataValue>) -> Result<Arc<DataTable>> {
        if args.len() < 3 || args.len() > 4 {
            return Err(anyhow::anyhow!(
                "RANDOM_INT expects 3-4 arguments (count, min, max, [seed])"
            ));
        }

        let count = match &args[0] {
            DataValue::Integer(n) => *n as usize,
            DataValue::Float(f) => *f as usize,
            _ => return Err(anyhow::anyhow!("RANDOM_INT count must be a number")),
        };

        let min = match &args[1] {
            DataValue::Integer(n) => *n,
            DataValue::Float(f) => *f as i64,
            _ => return Err(anyhow::anyhow!("RANDOM_INT min must be a number")),
        };

        let max = match &args[2] {
            DataValue::Integer(n) => *n,
            DataValue::Float(f) => *f as i64,
            _ => return Err(anyhow::anyhow!("RANDOM_INT max must be a number")),
        };

        if min > max {
            return Err(anyhow::anyhow!("RANDOM_INT min must be <= max"));
        }

        // Optional seed for reproducibility
        let mut rng: StdRng = if args.len() == 4 {
            let seed = match &args[3] {
                DataValue::Integer(n) => *n as u64,
                DataValue::Float(f) => *f as u64,
                _ => return Err(anyhow::anyhow!("RANDOM_INT seed must be a number")),
            };
            StdRng::seed_from_u64(seed)
        } else {
            StdRng::from_entropy()
        };

        let mut table = DataTable::new("random_int");
        table.add_column(DataColumn::new("id"));
        table.add_column(DataColumn::new("value"));

        for i in 0..count {
            let value = rng.gen_range(min..=max);
            table
                .add_row(DataRow::new(vec![
                    DataValue::Integer(i as i64),
                    DataValue::Integer(value),
                ]))
                .unwrap();
        }

        Ok(Arc::new(table))
    }

    fn description(&self) -> &str {
        "Generate random integers in range [min, max]"
    }

    fn arg_count(&self) -> usize {
        3 // or 4 with seed
    }
}

/// Generate random floats in a range
pub struct RandomFloats;

impl TableGenerator for RandomFloats {
    fn name(&self) -> &str {
        "RANDOM_FLOAT"
    }

    fn columns(&self) -> Vec<DataColumn> {
        vec![DataColumn::new("id"), DataColumn::new("value")]
    }

    fn generate(&self, args: Vec<DataValue>) -> Result<Arc<DataTable>> {
        if args.len() < 3 || args.len() > 4 {
            return Err(anyhow::anyhow!(
                "RANDOM_FLOAT expects 3-4 arguments (count, min, max, [seed])"
            ));
        }

        let count = match &args[0] {
            DataValue::Integer(n) => *n as usize,
            DataValue::Float(f) => *f as usize,
            _ => return Err(anyhow::anyhow!("RANDOM_FLOAT count must be a number")),
        };

        let min = match &args[1] {
            DataValue::Integer(n) => *n as f64,
            DataValue::Float(f) => *f,
            _ => return Err(anyhow::anyhow!("RANDOM_FLOAT min must be a number")),
        };

        let max = match &args[2] {
            DataValue::Integer(n) => *n as f64,
            DataValue::Float(f) => *f,
            _ => return Err(anyhow::anyhow!("RANDOM_FLOAT max must be a number")),
        };

        if min > max {
            return Err(anyhow::anyhow!("RANDOM_FLOAT min must be <= max"));
        }

        let mut rng: StdRng = if args.len() == 4 {
            let seed = match &args[3] {
                DataValue::Integer(n) => *n as u64,
                DataValue::Float(f) => *f as u64,
                _ => return Err(anyhow::anyhow!("RANDOM_FLOAT seed must be a number")),
            };
            StdRng::seed_from_u64(seed)
        } else {
            StdRng::from_entropy()
        };

        let mut table = DataTable::new("random_float");
        table.add_column(DataColumn::new("id"));
        table.add_column(DataColumn::new("value"));

        for i in 0..count {
            let value = rng.gen_range(min..=max);
            table
                .add_row(DataRow::new(vec![
                    DataValue::Integer(i as i64),
                    DataValue::Float(value),
                ]))
                .unwrap();
        }

        Ok(Arc::new(table))
    }

    fn description(&self) -> &str {
        "Generate random floating-point numbers in range [min, max]"
    }

    fn arg_count(&self) -> usize {
        3 // or 4 with seed
    }
}

/// Generate UUIDs
pub struct GenerateUUIDs;

impl TableGenerator for GenerateUUIDs {
    fn name(&self) -> &str {
        "GENERATE_UUID"
    }

    fn columns(&self) -> Vec<DataColumn> {
        vec![DataColumn::new("id"), DataColumn::new("uuid")]
    }

    fn generate(&self, args: Vec<DataValue>) -> Result<Arc<DataTable>> {
        if args.len() != 1 {
            return Err(anyhow::anyhow!("GENERATE_UUID expects 1 argument (count)"));
        }

        let count = match &args[0] {
            DataValue::Integer(n) => *n as usize,
            DataValue::Float(f) => *f as usize,
            _ => return Err(anyhow::anyhow!("GENERATE_UUID count must be a number")),
        };

        let mut table = DataTable::new("uuids");
        table.add_column(DataColumn::new("id"));
        table.add_column(DataColumn::new("uuid"));

        for i in 0..count {
            let uuid = uuid::Uuid::new_v4().to_string();
            table
                .add_row(DataRow::new(vec![
                    DataValue::Integer(i as i64),
                    DataValue::String(uuid),
                ]))
                .unwrap();
        }

        Ok(Arc::new(table))
    }

    fn description(&self) -> &str {
        "Generate UUID v4 strings"
    }

    fn arg_count(&self) -> usize {
        1
    }
}

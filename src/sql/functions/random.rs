use anyhow::{anyhow, Result};
use rand::Rng;

use crate::data::datatable::DataValue;
use crate::sql::functions::{ArgCount, FunctionCategory, FunctionSignature, SqlFunction};

/// RAND_RANGE function - Generate random numbers within a range
pub struct RandRangeFunction;

impl SqlFunction for RandRangeFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "RAND_RANGE",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(3),
            description: "Generate N random numbers between lower and upper bounds",
            returns: "TABLE",
            examples: vec![
                "SELECT * FROM RAND_RANGE(10, 1, 100)", // 10 random numbers between 1 and 100
                "SELECT * FROM RAND_RANGE(5, 0.0, 1.0)", // 5 random floats between 0.0 and 1.0
                "SELECT AVG(value) FROM RAND_RANGE(1000, 1, 6)", // Average of 1000 dice rolls
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let count = match &args[0] {
            DataValue::Integer(n) if *n > 0 => *n as usize,
            DataValue::Float(f) if *f > 0.0 => *f as usize,
            _ => return Err(anyhow!("RAND_RANGE count must be a positive number")),
        };

        let (lower, upper, use_float) = match (&args[1], &args[2]) {
            (DataValue::Integer(l), DataValue::Integer(u)) => (*l as f64, *u as f64, false),
            (DataValue::Float(l), DataValue::Float(u)) => (*l, *u, true),
            (DataValue::Integer(l), DataValue::Float(u)) => (*l as f64, *u, true),
            (DataValue::Float(l), DataValue::Integer(u)) => (*l, *u as f64, true),
            _ => return Err(anyhow!("RAND_RANGE bounds must be numeric")),
        };

        if lower > upper {
            return Err(anyhow!("RAND_RANGE lower bound must be <= upper bound"));
        }

        // Generate random values
        let mut rng = rand::thread_rng();
        let mut values = Vec::with_capacity(count);

        for _ in 0..count {
            if use_float {
                let value = rng.gen_range(lower..=upper);
                values.push(DataValue::Float(value));
            } else {
                let value = rng.gen_range(lower as i64..=upper as i64);
                values.push(DataValue::Integer(value));
            }
        }

        // For now, return the first value (we'll enhance this when table functions are fully supported)
        // In the future, this should return a proper table
        if values.is_empty() {
            Ok(DataValue::Null)
        } else {
            Ok(values[0].clone())
        }
    }
}

/// RANDOM function - Generate a single random number between 0 and 1
pub struct RandomFunction;

impl SqlFunction for RandomFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "RANDOM",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(0),
            description: "Generate a random float between 0 and 1",
            returns: "FLOAT",
            examples: vec![
                "SELECT RANDOM()",
                "SELECT ROUND(RANDOM() * 100, 0)", // Random integer 0-100
            ],
        }
    }

    fn evaluate(&self, _args: &[DataValue]) -> Result<DataValue> {
        let mut rng = rand::thread_rng();
        Ok(DataValue::Float(rng.gen_range(0.0..1.0)))
    }
}

/// RAND_INT function - Generate a random integer between bounds
pub struct RandIntFunction;

impl SqlFunction for RandIntFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "RAND_INT",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(2),
            description: "Generate a random integer between lower and upper bounds (inclusive)",
            returns: "INTEGER",
            examples: vec![
                "SELECT RAND_INT(1, 6)",   // Dice roll
                "SELECT RAND_INT(1, 100)", // Random percentage
                "SELECT RAND_INT(0, 255)", // Random byte value
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let lower = match &args[0] {
            DataValue::Integer(n) => *n,
            DataValue::Float(f) => *f as i64,
            _ => return Err(anyhow!("RAND_INT lower bound must be numeric")),
        };

        let upper = match &args[1] {
            DataValue::Integer(n) => *n,
            DataValue::Float(f) => *f as i64,
            _ => return Err(anyhow!("RAND_INT upper bound must be numeric")),
        };

        if lower > upper {
            return Err(anyhow!("RAND_INT lower bound must be <= upper bound"));
        }

        let mut rng = rand::thread_rng();
        Ok(DataValue::Integer(rng.gen_range(lower..=upper)))
    }
}

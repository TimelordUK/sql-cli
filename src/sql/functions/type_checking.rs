use crate::data::datatable::DataValue;
use crate::sql::functions::date_time::parse_datetime;
use crate::sql::functions::{ArgCount, FunctionCategory, FunctionSignature, SqlFunction};
use anyhow::Result;

pub struct IsDateFunction;

impl SqlFunction for IsDateFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "IS_DATE",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(1),
            description: "Check if a value can be parsed as a date/datetime",
            returns: "Boolean",
            examples: vec![
                "SELECT * FROM data WHERE IS_DATE(column)",
                "SELECT IS_DATE('2024-01-01')",
                "SELECT IS_DATE('not a date')",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Ok(DataValue::Boolean(false));
        }

        let result = match &args[0] {
            DataValue::DateTime(_) => true,
            DataValue::String(s) => parse_datetime(s).is_ok(),
            DataValue::InternedString(s) => parse_datetime(s.as_str()).is_ok(),
            DataValue::Null => false,
            _ => false,
        };

        Ok(DataValue::Boolean(result))
    }
}

pub struct IsBoolFunction;

impl SqlFunction for IsBoolFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "IS_BOOL",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(1),
            description: "Check if a value can be parsed as a boolean",
            returns: "Boolean",
            examples: vec![
                "SELECT * FROM data WHERE IS_BOOL(column)",
                "SELECT IS_BOOL('true')",
                "SELECT IS_BOOL('123')",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Ok(DataValue::Boolean(false));
        }

        let result = match &args[0] {
            DataValue::Boolean(_) => true,
            DataValue::String(s) => {
                s.eq_ignore_ascii_case("true")
                    || s.eq_ignore_ascii_case("false")
                    || s == "1"
                    || s == "0"
                    || s.eq_ignore_ascii_case("yes")
                    || s.eq_ignore_ascii_case("no")
            }
            DataValue::InternedString(s) => {
                s.eq_ignore_ascii_case("true")
                    || s.eq_ignore_ascii_case("false")
                    || s.as_str() == "1"
                    || s.as_str() == "0"
                    || s.eq_ignore_ascii_case("yes")
                    || s.eq_ignore_ascii_case("no")
            }
            DataValue::Integer(i) => *i == 0 || *i == 1,
            DataValue::Null => false,
            _ => false,
        };

        Ok(DataValue::Boolean(result))
    }
}

pub struct IsNumericFunction;

impl SqlFunction for IsNumericFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "IS_NUMERIC",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(1),
            description: "Check if a value can be parsed as a number (integer or float)",
            returns: "Boolean",
            examples: vec![
                "SELECT * FROM data WHERE IS_NUMERIC(column)",
                "SELECT IS_NUMERIC('123.45')",
                "SELECT IS_NUMERIC('not a number')",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Ok(DataValue::Boolean(false));
        }

        let result = match &args[0] {
            DataValue::Integer(_) | DataValue::Float(_) => true,
            DataValue::String(s) => s.parse::<i64>().is_ok() || s.parse::<f64>().is_ok(),
            DataValue::InternedString(s) => s.parse::<i64>().is_ok() || s.parse::<f64>().is_ok(),
            DataValue::Null => false,
            _ => false,
        };

        Ok(DataValue::Boolean(result))
    }
}

pub struct IsIntegerFunction;

impl SqlFunction for IsIntegerFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "IS_INTEGER",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(1),
            description: "Check if a value can be parsed as an integer",
            returns: "Boolean",
            examples: vec![
                "SELECT * FROM data WHERE IS_INTEGER(column)",
                "SELECT IS_INTEGER('123')",
                "SELECT IS_INTEGER('123.45')",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Ok(DataValue::Boolean(false));
        }

        let result = match &args[0] {
            DataValue::Integer(_) => true,
            DataValue::String(s) => s.parse::<i64>().is_ok(),
            DataValue::InternedString(s) => s.parse::<i64>().is_ok(),
            DataValue::Float(f) => {
                f.fract() == 0.0 && *f >= i64::MIN as f64 && *f <= i64::MAX as f64
            }
            DataValue::Null => false,
            _ => false,
        };

        Ok(DataValue::Boolean(result))
    }
}

pub struct IsFloatFunction;

impl SqlFunction for IsFloatFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "IS_FLOAT",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(1),
            description: "Check if a value can be parsed as a float",
            returns: "Boolean",
            examples: vec![
                "SELECT * FROM data WHERE IS_FLOAT(column)",
                "SELECT IS_FLOAT('123.45')",
                "SELECT IS_FLOAT('123')",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Ok(DataValue::Boolean(false));
        }

        let result = match &args[0] {
            DataValue::Float(_) => true,
            DataValue::Integer(_) => true, // Integers can be represented as floats
            DataValue::String(s) => s.parse::<f64>().is_ok(),
            DataValue::InternedString(s) => s.parse::<f64>().is_ok(),
            DataValue::Null => false,
            _ => false,
        };

        Ok(DataValue::Boolean(result))
    }
}

pub struct IsNullFunction;

impl SqlFunction for IsNullFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "IS_NULL",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(1),
            description: "Check if a value is NULL",
            returns: "Boolean",
            examples: vec![
                "SELECT * FROM data WHERE IS_NULL(column)",
                "SELECT IS_NULL(NULL)",
                "SELECT IS_NULL('value')",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Ok(DataValue::Boolean(false));
        }

        Ok(DataValue::Boolean(args[0].is_null()))
    }
}

pub struct IsNotNullFunction;

impl SqlFunction for IsNotNullFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "IS_NOT_NULL",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(1),
            description: "Check if a value is not NULL",
            returns: "Boolean",
            examples: vec![
                "SELECT * FROM data WHERE IS_NOT_NULL(column)",
                "SELECT IS_NOT_NULL('value')",
                "SELECT IS_NOT_NULL(NULL)",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Ok(DataValue::Boolean(true));
        }

        Ok(DataValue::Boolean(!args[0].is_null()))
    }
}

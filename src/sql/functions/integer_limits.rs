use crate::data::datatable::DataValue;
use crate::sql::functions::{ArgCount, FunctionCategory, FunctionSignature, SqlFunction};
use anyhow::Result;

// INT8 / BYTE limits (8-bit)
pub struct Int8Min;
impl SqlFunction for Int8Min {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "INT8_MIN",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(0),
            description: "Minimum value for signed 8-bit integer (-128)",
            returns: "Integer value",
            examples: vec!["SELECT INT8_MIN()  -- Returns -128"],
        }
    }

    fn evaluate(&self, _args: &[DataValue]) -> Result<DataValue> {
        Ok(DataValue::Integer(-128))
    }
}

pub struct Int8Max;
impl SqlFunction for Int8Max {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "INT8_MAX",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(0),
            description: "Maximum value for signed 8-bit integer (127)",
            returns: "Integer value",
            examples: vec!["SELECT INT8_MAX()  -- Returns 127"],
        }
    }

    fn evaluate(&self, _args: &[DataValue]) -> Result<DataValue> {
        Ok(DataValue::Integer(127))
    }
}

pub struct Uint8Max;
impl SqlFunction for Uint8Max {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "UINT8_MAX",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(0),
            description: "Maximum value for unsigned 8-bit integer (255)",
            returns: "Integer value",
            examples: vec!["SELECT UINT8_MAX()  -- Returns 255"],
        }
    }

    fn evaluate(&self, _args: &[DataValue]) -> Result<DataValue> {
        Ok(DataValue::Integer(255))
    }
}

// INT16 / SHORT limits (16-bit)
pub struct Int16Min;
impl SqlFunction for Int16Min {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "INT16_MIN",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(0),
            description: "Minimum value for signed 16-bit integer (-32768)",
            returns: "Integer value",
            examples: vec!["SELECT INT16_MIN()  -- Returns -32768"],
        }
    }

    fn evaluate(&self, _args: &[DataValue]) -> Result<DataValue> {
        Ok(DataValue::Integer(-32768))
    }
}

pub struct Int16Max;
impl SqlFunction for Int16Max {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "INT16_MAX",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(0),
            description: "Maximum value for signed 16-bit integer (32767)",
            returns: "Integer value",
            examples: vec!["SELECT INT16_MAX()  -- Returns 32767"],
        }
    }

    fn evaluate(&self, _args: &[DataValue]) -> Result<DataValue> {
        Ok(DataValue::Integer(32767))
    }
}

pub struct Uint16Max;
impl SqlFunction for Uint16Max {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "UINT16_MAX",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(0),
            description: "Maximum value for unsigned 16-bit integer (65535)",
            returns: "Integer value",
            examples: vec!["SELECT UINT16_MAX()  -- Returns 65535"],
        }
    }

    fn evaluate(&self, _args: &[DataValue]) -> Result<DataValue> {
        Ok(DataValue::Integer(65535))
    }
}

// INT32 limits (32-bit)
pub struct Int32Min;
impl SqlFunction for Int32Min {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "INT32_MIN",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(0),
            description: "Minimum value for signed 32-bit integer (-2147483648)",
            returns: "Integer value",
            examples: vec!["SELECT INT32_MIN()  -- Returns -2147483648"],
        }
    }

    fn evaluate(&self, _args: &[DataValue]) -> Result<DataValue> {
        Ok(DataValue::Integer(-2147483648))
    }
}

pub struct Int32Max;
impl SqlFunction for Int32Max {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "INT32_MAX",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(0),
            description: "Maximum value for signed 32-bit integer (2147483647)",
            returns: "Integer value",
            examples: vec!["SELECT INT32_MAX()  -- Returns 2147483647"],
        }
    }

    fn evaluate(&self, _args: &[DataValue]) -> Result<DataValue> {
        Ok(DataValue::Integer(2147483647))
    }
}

pub struct Uint32Max;
impl SqlFunction for Uint32Max {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "UINT32_MAX",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(0),
            description: "Maximum value for unsigned 32-bit integer (4294967295)",
            returns: "Integer value",
            examples: vec!["SELECT UINT32_MAX()  -- Returns 4294967295"],
        }
    }

    fn evaluate(&self, _args: &[DataValue]) -> Result<DataValue> {
        Ok(DataValue::Integer(4294967295))
    }
}

// INT64 limits (64-bit)
pub struct Int64Min;
impl SqlFunction for Int64Min {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "INT64_MIN",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(0),
            description: "Minimum value for signed 64-bit integer (-9223372036854775808)",
            returns: "Integer value",
            examples: vec!["SELECT INT64_MIN()  -- Returns -9223372036854775808"],
        }
    }

    fn evaluate(&self, _args: &[DataValue]) -> Result<DataValue> {
        Ok(DataValue::Integer(i64::MIN))
    }
}

pub struct Int64Max;
impl SqlFunction for Int64Max {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "INT64_MAX",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(0),
            description: "Maximum value for signed 64-bit integer (9223372036854775807)",
            returns: "Integer value",
            examples: vec!["SELECT INT64_MAX()  -- Returns 9223372036854775807"],
        }
    }

    fn evaluate(&self, _args: &[DataValue]) -> Result<DataValue> {
        Ok(DataValue::Integer(i64::MAX))
    }
}

// Alias functions for common names
pub struct ByteMin;
impl SqlFunction for ByteMin {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "BYTE_MIN",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(0),
            description: "Minimum value for unsigned byte (0)",
            returns: "Integer value",
            examples: vec!["SELECT BYTE_MIN()  -- Returns 0"],
        }
    }

    fn evaluate(&self, _args: &[DataValue]) -> Result<DataValue> {
        Ok(DataValue::Integer(0))
    }
}

pub struct ByteMax;
impl SqlFunction for ByteMax {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "BYTE_MAX",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(0),
            description: "Maximum value for unsigned byte (255)",
            returns: "Integer value",
            examples: vec!["SELECT BYTE_MAX()  -- Returns 255"],
        }
    }

    fn evaluate(&self, _args: &[DataValue]) -> Result<DataValue> {
        Ok(DataValue::Integer(255))
    }
}

pub struct CharMin;
impl SqlFunction for CharMin {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "CHAR_MIN",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(0),
            description: "Minimum value for signed char (-128)",
            returns: "Integer value",
            examples: vec!["SELECT CHAR_MIN()  -- Returns -128"],
        }
    }

    fn evaluate(&self, _args: &[DataValue]) -> Result<DataValue> {
        Ok(DataValue::Integer(-128))
    }
}

pub struct CharMax;
impl SqlFunction for CharMax {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "CHAR_MAX",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(0),
            description: "Maximum value for signed char (127)",
            returns: "Integer value",
            examples: vec!["SELECT CHAR_MAX()  -- Returns 127"],
        }
    }

    fn evaluate(&self, _args: &[DataValue]) -> Result<DataValue> {
        Ok(DataValue::Integer(127))
    }
}

pub struct ShortMin;
impl SqlFunction for ShortMin {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "SHORT_MIN",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(0),
            description: "Minimum value for signed short (-32768)",
            returns: "Integer value",
            examples: vec!["SELECT SHORT_MIN()  -- Returns -32768"],
        }
    }

    fn evaluate(&self, _args: &[DataValue]) -> Result<DataValue> {
        Ok(DataValue::Integer(-32768))
    }
}

pub struct ShortMax;
impl SqlFunction for ShortMax {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "SHORT_MAX",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(0),
            description: "Maximum value for signed short (32767)",
            returns: "Integer value",
            examples: vec!["SELECT SHORT_MAX()  -- Returns 32767"],
        }
    }

    fn evaluate(&self, _args: &[DataValue]) -> Result<DataValue> {
        Ok(DataValue::Integer(32767))
    }
}

pub struct IntMin;
impl SqlFunction for IntMin {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "INT_MIN",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(0),
            description: "Minimum value for signed 32-bit int (-2147483648)",
            returns: "Integer value",
            examples: vec!["SELECT INT_MIN()  -- Returns -2147483648"],
        }
    }

    fn evaluate(&self, _args: &[DataValue]) -> Result<DataValue> {
        Ok(DataValue::Integer(-2147483648))
    }
}

pub struct IntMax;
impl SqlFunction for IntMax {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "INT_MAX",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(0),
            description: "Maximum value for signed 32-bit int (2147483647)",
            returns: "Integer value",
            examples: vec!["SELECT INT_MAX()  -- Returns 2147483647"],
        }
    }

    fn evaluate(&self, _args: &[DataValue]) -> Result<DataValue> {
        Ok(DataValue::Integer(2147483647))
    }
}

pub struct LongMin;
impl SqlFunction for LongMin {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "LONG_MIN",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(0),
            description: "Minimum value for signed 64-bit long (-9223372036854775808)",
            returns: "Integer value",
            examples: vec!["SELECT LONG_MIN()  -- Returns -9223372036854775808"],
        }
    }

    fn evaluate(&self, _args: &[DataValue]) -> Result<DataValue> {
        Ok(DataValue::Integer(i64::MIN))
    }
}

pub struct LongMax;
impl SqlFunction for LongMax {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "LONG_MAX",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(0),
            description: "Maximum value for signed 64-bit long (9223372036854775807)",
            returns: "Integer value",
            examples: vec!["SELECT LONG_MAX()  -- Returns 9223372036854775807"],
        }
    }

    fn evaluate(&self, _args: &[DataValue]) -> Result<DataValue> {
        Ok(DataValue::Integer(i64::MAX))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_int8_limits() {
        assert_eq!(Int8Min.evaluate(&[]).unwrap(), DataValue::Integer(-128));
        assert_eq!(Int8Max.evaluate(&[]).unwrap(), DataValue::Integer(127));
        assert_eq!(Uint8Max.evaluate(&[]).unwrap(), DataValue::Integer(255));
    }

    #[test]
    fn test_int16_limits() {
        assert_eq!(Int16Min.evaluate(&[]).unwrap(), DataValue::Integer(-32768));
        assert_eq!(Int16Max.evaluate(&[]).unwrap(), DataValue::Integer(32767));
        assert_eq!(Uint16Max.evaluate(&[]).unwrap(), DataValue::Integer(65535));
    }

    #[test]
    fn test_int32_limits() {
        assert_eq!(
            Int32Min.evaluate(&[]).unwrap(),
            DataValue::Integer(-2147483648)
        );
        assert_eq!(
            Int32Max.evaluate(&[]).unwrap(),
            DataValue::Integer(2147483647)
        );
        assert_eq!(
            Uint32Max.evaluate(&[]).unwrap(),
            DataValue::Integer(4294967295)
        );
    }

    #[test]
    fn test_int64_limits() {
        assert_eq!(
            Int64Min.evaluate(&[]).unwrap(),
            DataValue::Integer(i64::MIN)
        );
        assert_eq!(
            Int64Max.evaluate(&[]).unwrap(),
            DataValue::Integer(i64::MAX)
        );
    }

    #[test]
    fn test_alias_functions() {
        assert_eq!(ByteMin.evaluate(&[]).unwrap(), DataValue::Integer(0));
        assert_eq!(ByteMax.evaluate(&[]).unwrap(), DataValue::Integer(255));
        assert_eq!(CharMin.evaluate(&[]).unwrap(), DataValue::Integer(-128));
        assert_eq!(CharMax.evaluate(&[]).unwrap(), DataValue::Integer(127));
        assert_eq!(ShortMin.evaluate(&[]).unwrap(), DataValue::Integer(-32768));
        assert_eq!(ShortMax.evaluate(&[]).unwrap(), DataValue::Integer(32767));
        assert_eq!(
            IntMin.evaluate(&[]).unwrap(),
            DataValue::Integer(-2147483648)
        );
        assert_eq!(
            IntMax.evaluate(&[]).unwrap(),
            DataValue::Integer(2147483647)
        );
        assert_eq!(LongMin.evaluate(&[]).unwrap(), DataValue::Integer(i64::MIN));
        assert_eq!(LongMax.evaluate(&[]).unwrap(), DataValue::Integer(i64::MAX));
    }
}

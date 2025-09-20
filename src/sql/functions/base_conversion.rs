use crate::data::datatable::DataValue;
use crate::sql::functions::{ArgCount, FunctionCategory, FunctionSignature, SqlFunction};
use anyhow::{anyhow, Result};

/// Convert a number to a string representation in the specified base (2-36)
pub struct ToBase;

impl SqlFunction for ToBase {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "TO_BASE",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(2),
            description: "Convert a number to a string in the specified base (2-36)",
            returns: "String representation in the specified base",
            examples: vec![
                "SELECT TO_BASE(255, 16)  -- Returns 'ff'",
                "SELECT TO_BASE(42, 2)    -- Returns '101010'",
                "SELECT TO_BASE(100, 8)   -- Returns '144'",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 2 {
            return Err(anyhow!("TO_BASE requires exactly 2 arguments"));
        }

        let number = match &args[0] {
            DataValue::Integer(i) => *i,
            DataValue::Float(f) => *f as i64,
            DataValue::Null => return Ok(DataValue::Null),
            _ => {
                return Err(anyhow!(
                    "TO_BASE requires a numeric value as first argument"
                ))
            }
        };

        let base = match &args[1] {
            DataValue::Integer(i) => *i as i32,
            DataValue::Float(f) => *f as i32,
            DataValue::Null => return Ok(DataValue::Null),
            _ => {
                return Err(anyhow!(
                    "TO_BASE requires an integer base as second argument"
                ))
            }
        };

        if base < 2 || base > 36 {
            return Err(anyhow!("Base must be between 2 and 36, got {}", base));
        }

        if number < 0 {
            // Handle negative numbers by prepending '-'
            let result = format!("-{}", to_base_string((-number) as u64, base as u32));
            Ok(DataValue::String(result))
        } else {
            let result = to_base_string(number as u64, base as u32);
            Ok(DataValue::String(result))
        }
    }
}

/// Convert a string representation in the specified base to a number
pub struct FromBase;

impl SqlFunction for FromBase {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "FROM_BASE",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(2),
            description: "Convert a string in the specified base (2-36) to a number",
            returns: "Numeric value",
            examples: vec![
                "SELECT FROM_BASE('ff', 16)     -- Returns 255",
                "SELECT FROM_BASE('101010', 2)  -- Returns 42",
                "SELECT FROM_BASE('144', 8)     -- Returns 100",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 2 {
            return Err(anyhow!("FROM_BASE requires exactly 2 arguments"));
        }

        let string = match &args[0] {
            DataValue::String(s) => s.trim(),
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("FROM_BASE requires a string as first argument")),
        };

        let base = match &args[1] {
            DataValue::Integer(i) => *i as u32,
            DataValue::Float(f) => *f as u32,
            DataValue::Null => return Ok(DataValue::Null),
            _ => {
                return Err(anyhow!(
                    "FROM_BASE requires an integer base as second argument"
                ))
            }
        };

        if base < 2 || base > 36 {
            return Err(anyhow!("Base must be between 2 and 36, got {}", base));
        }

        // Handle negative numbers
        let (is_negative, string) = if string.starts_with('-') {
            (true, &string[1..])
        } else {
            (false, string)
        };

        match i64::from_str_radix(string, base) {
            Ok(n) => {
                let result = if is_negative { -n } else { n };
                Ok(DataValue::Integer(result))
            }
            Err(e) => Err(anyhow!("Invalid {} base number '{}': {}", base, string, e)),
        }
    }
}

/// Convert to binary string
pub struct ToBinary;

impl SqlFunction for ToBinary {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "TO_BINARY",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(1),
            description: "Convert a number to binary string representation",
            returns: "Binary string",
            examples: vec![
                "SELECT TO_BINARY(42)   -- Returns '101010'",
                "SELECT TO_BINARY(255)  -- Returns '11111111'",
                "SELECT TO_BINARY(-5)   -- Returns '-101'",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Err(anyhow!("TO_BINARY requires exactly 1 argument"));
        }

        let to_base = ToBase;
        to_base.evaluate(&[args[0].clone(), DataValue::Integer(2)])
    }
}

/// Convert from binary string
pub struct FromBinary;

impl SqlFunction for FromBinary {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "FROM_BINARY",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(1),
            description: "Convert a binary string to a number",
            returns: "Numeric value",
            examples: vec![
                "SELECT FROM_BINARY('101010')   -- Returns 42",
                "SELECT FROM_BINARY('11111111') -- Returns 255",
                "SELECT FROM_BINARY('-101')     -- Returns -5",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Err(anyhow!("FROM_BINARY requires exactly 1 argument"));
        }

        let from_base = FromBase;
        from_base.evaluate(&[args[0].clone(), DataValue::Integer(2)])
    }
}

/// Convert to hexadecimal string
pub struct ToHex;

impl SqlFunction for ToHex {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "TO_HEX",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(1),
            description: "Convert a number to hexadecimal string representation",
            returns: "Hexadecimal string",
            examples: vec![
                "SELECT TO_HEX(255)    -- Returns 'ff'",
                "SELECT TO_HEX(4095)   -- Returns 'fff'",
                "SELECT TO_HEX(16777215) -- Returns 'ffffff'",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Err(anyhow!("TO_HEX requires exactly 1 argument"));
        }

        let to_base = ToBase;
        to_base.evaluate(&[args[0].clone(), DataValue::Integer(16)])
    }
}

/// Convert from hexadecimal string
pub struct FromHex;

impl SqlFunction for FromHex {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "FROM_HEX",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(1),
            description: "Convert a hexadecimal string to a number",
            returns: "Numeric value",
            examples: vec![
                "SELECT FROM_HEX('ff')     -- Returns 255",
                "SELECT FROM_HEX('fff')    -- Returns 4095",
                "SELECT FROM_HEX('ffffff') -- Returns 16777215",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Err(anyhow!("FROM_HEX requires exactly 1 argument"));
        }

        let from_base = FromBase;
        from_base.evaluate(&[args[0].clone(), DataValue::Integer(16)])
    }
}

/// Convert to octal string
pub struct ToOctal;

impl SqlFunction for ToOctal {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "TO_OCTAL",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(1),
            description: "Convert a number to octal string representation",
            returns: "Octal string",
            examples: vec![
                "SELECT TO_OCTAL(8)    -- Returns '10'",
                "SELECT TO_OCTAL(64)   -- Returns '100'",
                "SELECT TO_OCTAL(511)  -- Returns '777'",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Err(anyhow!("TO_OCTAL requires exactly 1 argument"));
        }

        let to_base = ToBase;
        to_base.evaluate(&[args[0].clone(), DataValue::Integer(8)])
    }
}

/// Convert from octal string
pub struct FromOctal;

impl SqlFunction for FromOctal {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "FROM_OCTAL",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(1),
            description: "Convert an octal string to a number",
            returns: "Numeric value",
            examples: vec![
                "SELECT FROM_OCTAL('10')   -- Returns 8",
                "SELECT FROM_OCTAL('100')  -- Returns 64",
                "SELECT FROM_OCTAL('777')  -- Returns 511",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Err(anyhow!("FROM_OCTAL requires exactly 1 argument"));
        }

        let from_base = FromBase;
        from_base.evaluate(&[args[0].clone(), DataValue::Integer(8)])
    }
}

// Helper function to convert number to string in any base
fn to_base_string(mut num: u64, base: u32) -> String {
    if num == 0 {
        return "0".to_string();
    }

    const DIGITS: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    let mut result = Vec::new();

    while num > 0 {
        let remainder = (num % base as u64) as usize;
        result.push(DIGITS[remainder]);
        num /= base as u64;
    }

    result.reverse();
    String::from_utf8(result).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_base() {
        let func = ToBase;

        // Binary
        assert_eq!(
            func.evaluate(&[DataValue::Integer(42), DataValue::Integer(2)])
                .unwrap(),
            DataValue::String("101010".to_string())
        );

        // Hexadecimal
        assert_eq!(
            func.evaluate(&[DataValue::Integer(255), DataValue::Integer(16)])
                .unwrap(),
            DataValue::String("ff".to_string())
        );

        // Octal
        assert_eq!(
            func.evaluate(&[DataValue::Integer(64), DataValue::Integer(8)])
                .unwrap(),
            DataValue::String("100".to_string())
        );

        // Base 36
        assert_eq!(
            func.evaluate(&[DataValue::Integer(1295), DataValue::Integer(36)])
                .unwrap(),
            DataValue::String("zz".to_string())
        );
    }

    #[test]
    fn test_from_base() {
        let func = FromBase;

        // Binary
        assert_eq!(
            func.evaluate(&[
                DataValue::String("101010".to_string()),
                DataValue::Integer(2)
            ])
            .unwrap(),
            DataValue::Integer(42)
        );

        // Hexadecimal
        assert_eq!(
            func.evaluate(&[DataValue::String("ff".to_string()), DataValue::Integer(16)])
                .unwrap(),
            DataValue::Integer(255)
        );

        // Octal
        assert_eq!(
            func.evaluate(&[DataValue::String("100".to_string()), DataValue::Integer(8)])
                .unwrap(),
            DataValue::Integer(64)
        );
    }

    #[test]
    fn test_binary_functions() {
        let to_bin = ToBinary;
        let from_bin = FromBinary;

        assert_eq!(
            to_bin.evaluate(&[DataValue::Integer(42)]).unwrap(),
            DataValue::String("101010".to_string())
        );

        assert_eq!(
            from_bin
                .evaluate(&[DataValue::String("101010".to_string())])
                .unwrap(),
            DataValue::Integer(42)
        );
    }

    #[test]
    fn test_hex_functions() {
        let to_hex = ToHex;
        let from_hex = FromHex;

        assert_eq!(
            to_hex.evaluate(&[DataValue::Integer(255)]).unwrap(),
            DataValue::String("ff".to_string())
        );

        assert_eq!(
            from_hex
                .evaluate(&[DataValue::String("ff".to_string())])
                .unwrap(),
            DataValue::Integer(255)
        );
    }
}

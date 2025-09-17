use crate::data::datatable::DataValue;
use crate::sql::functions::{ArgCount, FunctionCategory, FunctionSignature, SqlFunction};
use anyhow::Result;

/// REPEAT function - Repeat a string n times
pub struct RepeatFunction;

impl SqlFunction for RepeatFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "REPEAT",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(2),
            description: "Repeat a string n times",
            returns: "String containing the input repeated n times",
            examples: vec![
                "SELECT REPEAT('*', 5)  -- Returns '*****'",
                "SELECT REPEAT('ab', 3)  -- Returns 'ababab'",
                "SELECT REPEAT('=', COUNT(*) / 10) FROM table  -- Create histogram",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 2 {
            return Err(anyhow::anyhow!("REPEAT expects exactly 2 arguments"));
        }

        let string = match &args[0] {
            DataValue::String(s) => s.clone(),
            DataValue::Null => return Ok(DataValue::Null),
            _ => args[0].to_string(),
        };

        let count = match &args[1] {
            DataValue::Integer(n) => *n,
            DataValue::Float(f) => *f as i64,
            DataValue::Null => return Ok(DataValue::Null),
            _ => {
                return Err(anyhow::anyhow!(
                    "REPEAT count must be a number, got {:?}",
                    args[1]
                ))
            }
        };

        if count < 0 {
            return Err(anyhow::anyhow!("REPEAT count cannot be negative"));
        }

        if count == 0 {
            return Ok(DataValue::String(String::new()));
        }

        // Limit repetitions to prevent memory issues
        if count > 10000 {
            return Err(anyhow::anyhow!("REPEAT count too large (max 10000)"));
        }

        Ok(DataValue::String(string.repeat(count as usize)))
    }
}

/// LPAD function - Left pad a string to a certain length
pub struct LPadFunction;

impl SqlFunction for LPadFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "LPAD",
            category: FunctionCategory::String,
            arg_count: ArgCount::Range(2, 3),
            description: "Left pad a string to a certain length",
            returns: "String padded on the left to specified length",
            examples: vec![
                "SELECT LPAD('5', 3, '0')  -- Returns '005'",
                "SELECT LPAD('hello', 10, ' ')  -- Returns '     hello'",
                "SELECT LPAD('abc', 5)  -- Returns '  abc' (default pad is space)",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() < 2 || args.len() > 3 {
            return Err(anyhow::anyhow!("LPAD expects 2 or 3 arguments"));
        }

        let string = match &args[0] {
            DataValue::String(s) => s.clone(),
            DataValue::Null => return Ok(DataValue::Null),
            _ => args[0].to_string(),
        };

        let length = match &args[1] {
            DataValue::Integer(n) => *n as usize,
            DataValue::Float(f) => *f as usize,
            DataValue::Null => return Ok(DataValue::Null),
            _ => {
                return Err(anyhow::anyhow!(
                    "LPAD length must be a number, got {:?}",
                    args[1]
                ))
            }
        };

        let pad_str = if args.len() == 3 {
            match &args[2] {
                DataValue::String(s) => {
                    if s.is_empty() {
                        return Err(anyhow::anyhow!("LPAD pad string cannot be empty"));
                    }
                    s.clone()
                }
                DataValue::Null => return Ok(DataValue::Null),
                _ => args[2].to_string(),
            }
        } else {
            " ".to_string()
        };

        if string.len() >= length {
            // Truncate if string is longer than target length
            Ok(DataValue::String(string.chars().take(length).collect()))
        } else {
            let pad_needed = length - string.len();
            let pad_chars: Vec<char> = pad_str.chars().collect();
            let mut result = String::with_capacity(length);

            // Add padding
            let full_pads = pad_needed / pad_chars.len();
            let partial_pad = pad_needed % pad_chars.len();

            for _ in 0..full_pads {
                result.push_str(&pad_str);
            }
            for i in 0..partial_pad {
                result.push(pad_chars[i]);
            }

            // Add original string
            result.push_str(&string);

            Ok(DataValue::String(result))
        }
    }
}

/// RPAD function - Right pad a string to a certain length
pub struct RPadFunction;

impl SqlFunction for RPadFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "RPAD",
            category: FunctionCategory::String,
            arg_count: ArgCount::Range(2, 3),
            description: "Right pad a string to a certain length",
            returns: "String padded on the right to specified length",
            examples: vec![
                "SELECT RPAD('5', 3, '0')  -- Returns '500'",
                "SELECT RPAD('hello', 10, '.')  -- Returns 'hello.....'",
                "SELECT RPAD('abc', 5)  -- Returns 'abc  ' (default pad is space)",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() < 2 || args.len() > 3 {
            return Err(anyhow::anyhow!("RPAD expects 2 or 3 arguments"));
        }

        let string = match &args[0] {
            DataValue::String(s) => s.clone(),
            DataValue::Null => return Ok(DataValue::Null),
            _ => args[0].to_string(),
        };

        let length = match &args[1] {
            DataValue::Integer(n) => *n as usize,
            DataValue::Float(f) => *f as usize,
            DataValue::Null => return Ok(DataValue::Null),
            _ => {
                return Err(anyhow::anyhow!(
                    "RPAD length must be a number, got {:?}",
                    args[1]
                ))
            }
        };

        let pad_str = if args.len() == 3 {
            match &args[2] {
                DataValue::String(s) => {
                    if s.is_empty() {
                        return Err(anyhow::anyhow!("RPAD pad string cannot be empty"));
                    }
                    s.clone()
                }
                DataValue::Null => return Ok(DataValue::Null),
                _ => args[2].to_string(),
            }
        } else {
            " ".to_string()
        };

        if string.len() >= length {
            // Truncate if string is longer than target length
            Ok(DataValue::String(string.chars().take(length).collect()))
        } else {
            let pad_needed = length - string.len();
            let pad_chars: Vec<char> = pad_str.chars().collect();
            let mut result = String::with_capacity(length);

            // Add original string
            result.push_str(&string);

            // Add padding
            let full_pads = pad_needed / pad_chars.len();
            let partial_pad = pad_needed % pad_chars.len();

            for _ in 0..full_pads {
                result.push_str(&pad_str);
            }
            for i in 0..partial_pad {
                result.push(pad_chars[i]);
            }

            Ok(DataValue::String(result))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repeat() {
        let func = RepeatFunction;

        // Basic repetition
        assert_eq!(
            func.evaluate(&[DataValue::String("*".to_string()), DataValue::Integer(5)])
                .unwrap(),
            DataValue::String("*****".to_string())
        );

        // Multi-char repetition
        assert_eq!(
            func.evaluate(&[DataValue::String("ab".to_string()), DataValue::Integer(3)])
                .unwrap(),
            DataValue::String("ababab".to_string())
        );

        // Zero repetitions
        assert_eq!(
            func.evaluate(&[DataValue::String("x".to_string()), DataValue::Integer(0)])
                .unwrap(),
            DataValue::String("".to_string())
        );

        // Null handling
        assert_eq!(
            func.evaluate(&[DataValue::Null, DataValue::Integer(5)])
                .unwrap(),
            DataValue::Null
        );
    }

    #[test]
    fn test_lpad() {
        let func = LPadFunction;

        // Basic padding with zeros
        assert_eq!(
            func.evaluate(&[
                DataValue::String("5".to_string()),
                DataValue::Integer(3),
                DataValue::String("0".to_string())
            ])
            .unwrap(),
            DataValue::String("005".to_string())
        );

        // Default space padding
        assert_eq!(
            func.evaluate(&[DataValue::String("abc".to_string()), DataValue::Integer(5)])
                .unwrap(),
            DataValue::String("  abc".to_string())
        );

        // Multi-char padding
        assert_eq!(
            func.evaluate(&[
                DataValue::String("X".to_string()),
                DataValue::Integer(5),
                DataValue::String("ab".to_string())
            ])
            .unwrap(),
            DataValue::String("ababX".to_string())
        );
    }

    #[test]
    fn test_rpad() {
        let func = RPadFunction;

        // Basic padding with zeros
        assert_eq!(
            func.evaluate(&[
                DataValue::String("5".to_string()),
                DataValue::Integer(3),
                DataValue::String("0".to_string())
            ])
            .unwrap(),
            DataValue::String("500".to_string())
        );

        // Default space padding
        assert_eq!(
            func.evaluate(&[DataValue::String("abc".to_string()), DataValue::Integer(5)])
                .unwrap(),
            DataValue::String("abc  ".to_string())
        );

        // Dots padding
        assert_eq!(
            func.evaluate(&[
                DataValue::String("hello".to_string()),
                DataValue::Integer(10),
                DataValue::String(".".to_string())
            ])
            .unwrap(),
            DataValue::String("hello.....".to_string())
        );
    }
}

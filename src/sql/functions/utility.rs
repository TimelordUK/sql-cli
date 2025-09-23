use crate::data::datatable::DataValue;
use crate::sql::functions::{ArgCount, FunctionCategory, FunctionSignature, SqlFunction};
use anyhow::Result;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};

/// ASCII() - Get ASCII/Unicode code point of first character
pub struct AsciiFunction;

impl SqlFunction for AsciiFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "ASCII",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(1),
            description: "Get ASCII/Unicode code point of first character",
            returns: "Integer code point of the first character",
            examples: vec![
                "SELECT ASCII('A')  -- Returns 65",
                "SELECT ASCII('ABC')  -- Returns 65 (first char only)",
                "SELECT ASCII('€')  -- Returns 8364 (Euro symbol)",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Err(anyhow::anyhow!(
                "ASCII expects 1 argument, got {}",
                args.len()
            ));
        }

        let text = args[0].to_string();
        if text.is_empty() {
            return Ok(DataValue::Null);
        }

        // Get the first character and its code point
        if let Some(first_char) = text.chars().next() {
            Ok(DataValue::Integer(first_char as i64))
        } else {
            Ok(DataValue::Null)
        }
    }
}

/// ORD() - Alias for ASCII (common in some SQL dialects)
pub struct OrdFunction;

impl SqlFunction for OrdFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "ORD",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(1),
            description: "Get Unicode code point of first character (alias for ASCII)",
            returns: "Integer code point of the first character",
            examples: vec![
                "SELECT ORD('A')  -- Returns 65",
                "SELECT ORD('€')  -- Returns 8364",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        AsciiFunction.evaluate(args)
    }
}

/// CHAR() - Alias for CHR() for SQL compatibility
pub struct CharFunction;

impl SqlFunction for CharFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "CHAR",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(1),
            description: "Convert ASCII/Unicode code to character (alias for CHR)",
            returns: "Character corresponding to the code",
            examples: vec![
                "SELECT CHAR(65)  -- Returns 'A'",
                "SELECT CHAR(8364)  -- Returns '€' (Euro symbol)",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Err(anyhow::anyhow!(
                "CHAR expects 1 argument, got {}",
                args.len()
            ));
        }

        let code = match &args[0] {
            DataValue::Integer(i) => *i as u32,
            DataValue::Float(f) => f.trunc() as u32,
            DataValue::String(s) => s
                .parse::<u32>()
                .map_err(|_| anyhow::anyhow!("CHAR expects a number, got '{}'", s))?,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow::anyhow!("CHAR expects a number as argument")),
        };

        // Convert Unicode code point to character
        match char::from_u32(code) {
            Some(c) => Ok(DataValue::String(c.to_string())),
            None => Err(anyhow::anyhow!("Invalid Unicode code point: {}", code)),
        }
    }
}

/// TO_INT() - Convert string to integer
pub struct ToIntFunction;

impl SqlFunction for ToIntFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "TO_INT",
            category: FunctionCategory::Conversion,
            arg_count: ArgCount::Fixed(1),
            description: "Convert string to integer",
            returns: "Integer value or NULL if conversion fails",
            examples: vec![
                "SELECT TO_INT('123')  -- Returns 123",
                "SELECT TO_INT('45.67')  -- Returns 45 (truncates)",
                "SELECT TO_INT('abc')  -- Returns NULL",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Err(anyhow::anyhow!(
                "TO_INT expects 1 argument, got {}",
                args.len()
            ));
        }

        match &args[0] {
            DataValue::Integer(i) => Ok(DataValue::Integer(*i)),
            DataValue::Float(f) => Ok(DataValue::Integer(f.trunc() as i64)),
            DataValue::String(s) => {
                // Try to parse as float first (handles "123.45")
                if let Ok(f) = s.parse::<f64>() {
                    Ok(DataValue::Integer(f.trunc() as i64))
                } else if let Ok(i) = s.parse::<i64>() {
                    Ok(DataValue::Integer(i))
                } else {
                    Ok(DataValue::Null)
                }
            }
            DataValue::Null => Ok(DataValue::Null),
            _ => Ok(DataValue::Null),
        }
    }
}

/// TO_DECIMAL() - Convert string to decimal/float
pub struct ToDecimalFunction;

impl SqlFunction for ToDecimalFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "TO_DECIMAL",
            category: FunctionCategory::Conversion,
            arg_count: ArgCount::Fixed(1),
            description: "Convert string to decimal/float",
            returns: "Float value or NULL if conversion fails",
            examples: vec![
                "SELECT TO_DECIMAL('123.45')  -- Returns 123.45",
                "SELECT TO_DECIMAL('123')  -- Returns 123.0",
                "SELECT TO_DECIMAL('abc')  -- Returns NULL",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Err(anyhow::anyhow!(
                "TO_DECIMAL expects 1 argument, got {}",
                args.len()
            ));
        }

        match &args[0] {
            DataValue::Integer(i) => Ok(DataValue::Float(*i as f64)),
            DataValue::Float(f) => Ok(DataValue::Float(*f)),
            DataValue::String(s) => match s.parse::<f64>() {
                Ok(f) => Ok(DataValue::Float(f)),
                Err(_) => Ok(DataValue::Null),
            },
            DataValue::Null => Ok(DataValue::Null),
            _ => Ok(DataValue::Null),
        }
    }
}

/// TO_STRING() - Convert any value to string
pub struct ToStringFunction;

impl SqlFunction for ToStringFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "TO_STRING",
            category: FunctionCategory::Conversion,
            arg_count: ArgCount::Fixed(1),
            description: "Convert any value to string representation",
            returns: "String representation of the value",
            examples: vec![
                "SELECT TO_STRING(123)  -- Returns '123'",
                "SELECT TO_STRING(45.67)  -- Returns '45.67'",
                "SELECT TO_STRING(NULL)  -- Returns NULL",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Err(anyhow::anyhow!(
                "TO_STRING expects 1 argument, got {}",
                args.len()
            ));
        }

        match &args[0] {
            DataValue::Null => Ok(DataValue::Null),
            value => Ok(DataValue::String(value.to_string())),
        }
    }
}

/// ENCODE() - Encode string to base64 or hex
pub struct EncodeFunction;

impl SqlFunction for EncodeFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "ENCODE",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(2),
            description: "Encode string to base64 or hex",
            returns: "Encoded string",
            examples: vec![
                "SELECT ENCODE('Hello', 'base64')  -- Returns 'SGVsbG8='",
                "SELECT ENCODE('Hello', 'hex')  -- Returns '48656c6c6f'",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 2 {
            return Err(anyhow::anyhow!(
                "ENCODE expects 2 arguments, got {}",
                args.len()
            ));
        }

        let text = args[0].to_string();
        let encoding = args[1].to_string().to_lowercase();

        match encoding.as_str() {
            "base64" => {
                let encoded = BASE64.encode(text.as_bytes());
                Ok(DataValue::String(encoded))
            }
            "hex" => {
                let hex = hex::encode(text.as_bytes());
                Ok(DataValue::String(hex))
            }
            _ => Err(anyhow::anyhow!(
                "Unsupported encoding '{}'. Use 'base64' or 'hex'",
                encoding
            )),
        }
    }
}

/// DECODE() - Decode base64 or hex string
pub struct DecodeFunction;

impl SqlFunction for DecodeFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "DECODE",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(2),
            description: "Decode base64 or hex encoded string",
            returns: "Decoded string",
            examples: vec![
                "SELECT DECODE('SGVsbG8=', 'base64')  -- Returns 'Hello'",
                "SELECT DECODE('48656c6c6f', 'hex')  -- Returns 'Hello'",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 2 {
            return Err(anyhow::anyhow!(
                "DECODE expects 2 arguments, got {}",
                args.len()
            ));
        }

        let encoded = args[0].to_string();
        let encoding = args[1].to_string().to_lowercase();

        match encoding.as_str() {
            "base64" => match BASE64.decode(encoded.as_bytes()) {
                Ok(bytes) => match String::from_utf8(bytes) {
                    Ok(decoded) => Ok(DataValue::String(decoded)),
                    Err(_) => Err(anyhow::anyhow!("Invalid UTF-8 in decoded data")),
                },
                Err(e) => Err(anyhow::anyhow!("Base64 decode error: {}", e)),
            },
            "hex" => match hex::decode(encoded.as_bytes()) {
                Ok(bytes) => match String::from_utf8(bytes) {
                    Ok(decoded) => Ok(DataValue::String(decoded)),
                    Err(_) => Err(anyhow::anyhow!("Invalid UTF-8 in decoded data")),
                },
                Err(e) => Err(anyhow::anyhow!("Hex decode error: {}", e)),
            },
            _ => Err(anyhow::anyhow!(
                "Unsupported encoding '{}'. Use 'base64' or 'hex'",
                encoding
            )),
        }
    }
}

/// UNICODE() - Get all Unicode code points in a string
pub struct UnicodeFunction;

impl SqlFunction for UnicodeFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "UNICODE",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(1),
            description: "Get Unicode code points of all characters as comma-separated list",
            returns: "Comma-separated list of Unicode code points",
            examples: vec![
                "SELECT UNICODE('ABC')  -- Returns '65,66,67'",
                "SELECT UNICODE('€$')  -- Returns '8364,36'",
                "SELECT UNICODE('Hello')  -- Returns '72,101,108,108,111'",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Err(anyhow::anyhow!(
                "UNICODE expects 1 argument, got {}",
                args.len()
            ));
        }

        let text = args[0].to_string();
        if text.is_empty() {
            return Ok(DataValue::Null);
        }

        let codes: Vec<String> = text.chars().map(|c| (c as u32).to_string()).collect();
        Ok(DataValue::String(codes.join(",")))
    }
}

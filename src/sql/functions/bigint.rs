use crate::data::datatable::DataValue;
use crate::sql::functions::{ArgCount, FunctionCategory, FunctionSignature, SqlFunction};
use anyhow::{anyhow, Result};
use num_bigint::{BigInt, BigUint};
use num_traits::{FromPrimitive, One, Signed, ToPrimitive, Zero};

/// BIGINT - Convert to arbitrary precision integer
pub struct BigIntFunction;

impl SqlFunction for BigIntFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "BIGINT",
            category: FunctionCategory::BigNumber,
            arg_count: ArgCount::Fixed(1),
            description: "Convert value to arbitrary precision integer",
            returns: "String representation of big integer",
            examples: vec![
                "SELECT BIGINT('123456789012345678901234567890')",
                "SELECT BIGINT(2) ^ BIGINT(100)  -- 2^100",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Err(anyhow!("BIGINT expects 1 argument"));
        }

        match &args[0] {
            DataValue::String(s) => {
                match s.parse::<BigInt>() {
                    Ok(_) => Ok(DataValue::String(s.clone())), // Valid bigint string
                    Err(_) => Err(anyhow!("Invalid big integer format: {}", s)),
                }
            }
            DataValue::Integer(n) => Ok(DataValue::String(n.to_string())),
            DataValue::Float(f) => Ok(DataValue::String((*f as i64).to_string())),
            DataValue::Null => Ok(DataValue::Null),
            _ => Err(anyhow!("BIGINT expects numeric or string input")),
        }
    }
}

/// BIGADD - Add two arbitrary precision integers
pub struct BigAddFunction;

impl SqlFunction for BigAddFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "BIGADD",
            category: FunctionCategory::BigNumber,
            arg_count: ArgCount::Fixed(2),
            description: "Add two arbitrary precision integers",
            returns: "String representation of sum",
            examples: vec![
                "SELECT BIGADD('999999999999999999999', '1')",
                "SELECT BIGADD('123456789012345678901234567890', '987654321098765432109876543210')",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 2 {
            return Err(anyhow!("BIGADD expects 2 arguments"));
        }

        let a = parse_bigint(&args[0])?;
        let b = parse_bigint(&args[1])?;

        Ok(DataValue::String((a + b).to_string()))
    }
}

/// BIGMUL - Multiply two arbitrary precision integers
pub struct BigMulFunction;

impl SqlFunction for BigMulFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "BIGMUL",
            category: FunctionCategory::BigNumber,
            arg_count: ArgCount::Fixed(2),
            description: "Multiply two arbitrary precision integers",
            returns: "String representation of product",
            examples: vec![
                "SELECT BIGMUL('999999999999999999999', '999999999999999999999')",
                "SELECT BIGMUL('123456789', '987654321')",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 2 {
            return Err(anyhow!("BIGMUL expects 2 arguments"));
        }

        let a = parse_bigint(&args[0])?;
        let b = parse_bigint(&args[1])?;

        Ok(DataValue::String((a * b).to_string()))
    }
}

/// BIGPOW - Raise arbitrary precision integer to a power
pub struct BigPowFunction;

impl SqlFunction for BigPowFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "BIGPOW",
            category: FunctionCategory::BigNumber,
            arg_count: ArgCount::Fixed(2),
            description: "Raise big integer to a power (base^exponent)",
            returns: "String representation of result",
            examples: vec![
                "SELECT BIGPOW('2', 100)    -- 2^100",
                "SELECT BIGPOW('10', 50)    -- 10^50",
                "SELECT BIGPOW('99', 99)    -- 99^99",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 2 {
            return Err(anyhow!("BIGPOW expects 2 arguments"));
        }

        let base = parse_bigint(&args[0])?;
        let exp = match &args[1] {
            DataValue::Integer(n) => *n as u32,
            DataValue::String(s) => s
                .parse::<u32>()
                .map_err(|_| anyhow!("Exponent must be a non-negative integer"))?,
            _ => return Err(anyhow!("Exponent must be a non-negative integer")),
        };

        let result = base.pow(exp);
        Ok(DataValue::String(result.to_string()))
    }
}

/// BIGFACT - Calculate factorial of large numbers
pub struct BigFactorialFunction;

impl SqlFunction for BigFactorialFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "BIGFACT",
            category: FunctionCategory::BigNumber,
            arg_count: ArgCount::Fixed(1),
            description: "Calculate factorial of a number (n!)",
            returns: "String representation of factorial",
            examples: vec![
                "SELECT BIGFACT(50)     -- 50!",
                "SELECT BIGFACT(100)    -- 100!",
                "SELECT LENGTH(BIGFACT(1000))  -- How many digits in 1000!",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Err(anyhow!("BIGFACT expects 1 argument"));
        }

        let n = match &args[0] {
            DataValue::Integer(n) => *n as u32,
            DataValue::String(s) => s
                .parse::<u32>()
                .map_err(|_| anyhow!("Factorial input must be a non-negative integer"))?,
            _ => return Err(anyhow!("Factorial input must be a non-negative integer")),
        };

        if n > 10000 {
            return Err(anyhow!("Factorial input too large (max 10000)"));
        }

        let mut result = BigInt::one();
        for i in 2..=n {
            result *= i;
        }

        Ok(DataValue::String(result.to_string()))
    }
}

/// TO_BINARY - Convert number to binary string
pub struct ToBinaryFunction;

impl SqlFunction for ToBinaryFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "TO_BINARY",
            category: FunctionCategory::BigNumber,
            arg_count: ArgCount::Fixed(1),
            description: "Convert number to binary string representation",
            returns: "Binary string (e.g., '1010' for 10)",
            examples: vec![
                "SELECT TO_BINARY(42)                     -- '101010'",
                "SELECT TO_BINARY('255')                  -- '11111111'",
                "SELECT TO_BINARY(BIGPOW('2', 100))      -- Binary of 2^100",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Err(anyhow!("TO_BINARY expects 1 argument"));
        }

        let num = parse_bigint(&args[0])?;
        if num.sign() == num_bigint::Sign::Minus {
            // For negative numbers, we'll show two's complement representation
            // with a fixed bit width (64 bits for demonstration)
            return Ok(DataValue::String(format!("-{}", num.abs().to_str_radix(2))));
        }

        Ok(DataValue::String(num.to_str_radix(2)))
    }
}

/// FROM_BINARY - Convert binary string to decimal
pub struct FromBinaryFunction;

impl SqlFunction for FromBinaryFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "FROM_BINARY",
            category: FunctionCategory::BigNumber,
            arg_count: ArgCount::Fixed(1),
            description: "Convert binary string to decimal number",
            returns: "Decimal string representation",
            examples: vec![
                "SELECT FROM_BINARY('101010')             -- '42'",
                "SELECT FROM_BINARY('11111111')           -- '255'",
                "SELECT FROM_BINARY('1' || REPEAT('0', 100)) -- 2^100",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Err(anyhow!("FROM_BINARY expects 1 argument"));
        }

        let binary_str = match &args[0] {
            DataValue::String(s) => s,
            _ => return Err(anyhow!("FROM_BINARY expects a string argument")),
        };

        // Remove any spaces and validate
        let clean_binary: String = binary_str
            .chars()
            .filter(|c| *c == '0' || *c == '1')
            .collect();

        if clean_binary.is_empty() {
            return Err(anyhow!("Invalid binary string"));
        }

        match BigInt::parse_bytes(clean_binary.as_bytes(), 2) {
            Some(num) => Ok(DataValue::String(num.to_string())),
            None => Err(anyhow!("Invalid binary string: {}", binary_str)),
        }
    }
}

/// TO_HEX - Convert number to hexadecimal string
pub struct ToHexFunction;

impl SqlFunction for ToHexFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "TO_HEX",
            category: FunctionCategory::BigNumber,
            arg_count: ArgCount::Fixed(1),
            description: "Convert number to hexadecimal string",
            returns: "Hexadecimal string (e.g., '2A' for 42)",
            examples: vec![
                "SELECT TO_HEX(255)                       -- 'ff'",
                "SELECT TO_HEX(BIGPOW('16', 10))         -- '10000000000'",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Err(anyhow!("TO_HEX expects 1 argument"));
        }

        let num = parse_bigint(&args[0])?;
        if num.sign() == num_bigint::Sign::Minus {
            return Ok(DataValue::String(format!(
                "-{}",
                num.abs().to_str_radix(16)
            )));
        }

        Ok(DataValue::String(num.to_str_radix(16)))
    }
}

/// FROM_HEX - Convert hexadecimal string to decimal
pub struct FromHexFunction;

impl SqlFunction for FromHexFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "FROM_HEX",
            category: FunctionCategory::BigNumber,
            arg_count: ArgCount::Fixed(1),
            description: "Convert hexadecimal string to decimal",
            returns: "Decimal string representation",
            examples: vec![
                "SELECT FROM_HEX('FF')                    -- '255'",
                "SELECT FROM_HEX('DEADBEEF')              -- '3735928559'",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Err(anyhow!("FROM_HEX expects 1 argument"));
        }

        let hex_str = match &args[0] {
            DataValue::String(s) => s,
            _ => return Err(anyhow!("FROM_HEX expects a string argument")),
        };

        // Remove 0x prefix if present and clean
        let clean_hex = hex_str
            .trim()
            .trim_start_matches("0x")
            .trim_start_matches("0X");

        match BigInt::parse_bytes(clean_hex.as_bytes(), 16) {
            Some(num) => Ok(DataValue::String(num.to_string())),
            None => Err(anyhow!("Invalid hexadecimal string: {}", hex_str)),
        }
    }
}

/// BITAND - Bitwise AND of two numbers
pub struct BitAndFunction;

impl SqlFunction for BitAndFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "BITAND",
            category: FunctionCategory::BigNumber,
            arg_count: ArgCount::Fixed(2),
            description: "Bitwise AND of two integers",
            returns: "Result of bitwise AND",
            examples: vec![
                "SELECT BITAND(12, 10)                    -- 8 (1100 & 1010 = 1000)",
                "SELECT TO_BINARY(BITAND(255, 15))       -- '1111'",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 2 {
            return Err(anyhow!("BITAND expects 2 arguments"));
        }

        let a = parse_bigint(&args[0])?;
        let b = parse_bigint(&args[1])?;

        Ok(DataValue::String((a & b).to_string()))
    }
}

/// BITOR - Bitwise OR of two numbers
pub struct BitOrFunction;

impl SqlFunction for BitOrFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "BITOR",
            category: FunctionCategory::BigNumber,
            arg_count: ArgCount::Fixed(2),
            description: "Bitwise OR of two integers",
            returns: "Result of bitwise OR",
            examples: vec![
                "SELECT BITOR(12, 10)                     -- 14 (1100 | 1010 = 1110)",
                "SELECT TO_BINARY(BITOR(8, 4))           -- '1100'",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 2 {
            return Err(anyhow!("BITOR expects 2 arguments"));
        }

        let a = parse_bigint(&args[0])?;
        let b = parse_bigint(&args[1])?;

        Ok(DataValue::String((a | b).to_string()))
    }
}

/// BITXOR - Bitwise XOR of two numbers
pub struct BitXorFunction;

impl SqlFunction for BitXorFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "BITXOR",
            category: FunctionCategory::BigNumber,
            arg_count: ArgCount::Fixed(2),
            description: "Bitwise XOR of two integers",
            returns: "Result of bitwise XOR",
            examples: vec![
                "SELECT BITXOR(12, 10)                    -- 6 (1100 ^ 1010 = 0110)",
                "SELECT TO_BINARY(BITXOR(255, 170))      -- '01010101'",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 2 {
            return Err(anyhow!("BITXOR expects 2 arguments"));
        }

        let a = parse_bigint(&args[0])?;
        let b = parse_bigint(&args[1])?;

        Ok(DataValue::String((a ^ b).to_string()))
    }
}

/// BITSHIFT - Bit shift left (positive) or right (negative)
pub struct BitShiftFunction;

impl SqlFunction for BitShiftFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "BITSHIFT",
            category: FunctionCategory::BigNumber,
            arg_count: ArgCount::Fixed(2),
            description: "Bit shift: left for positive shift, right for negative",
            returns: "Result of bit shift",
            examples: vec![
                "SELECT BITSHIFT(1, 10)                   -- 1024 (1 << 10)",
                "SELECT BITSHIFT(1024, -10)               -- 1 (1024 >> 10)",
                "SELECT TO_BINARY(BITSHIFT(1, 100))       -- 1 followed by 100 zeros",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 2 {
            return Err(anyhow!("BITSHIFT expects 2 arguments"));
        }

        let num = parse_bigint(&args[0])?;
        let shift = match &args[1] {
            DataValue::Integer(n) => *n,
            DataValue::String(s) => s
                .parse::<i64>()
                .map_err(|_| anyhow!("Shift amount must be an integer"))?,
            _ => return Err(anyhow!("Shift amount must be an integer")),
        };

        let result = if shift >= 0 {
            num << (shift as usize)
        } else {
            num >> ((-shift) as usize)
        };

        Ok(DataValue::String(result.to_string()))
    }
}

// Helper function to parse DataValue to BigInt
fn parse_bigint(value: &DataValue) -> Result<BigInt> {
    match value {
        DataValue::String(s) => s
            .parse::<BigInt>()
            .map_err(|_| anyhow!("Invalid big integer format: {}", s)),
        DataValue::Integer(n) => Ok(BigInt::from(*n)),
        DataValue::Float(f) => Ok(BigInt::from(*f as i64)),
        DataValue::Null => Ok(BigInt::zero()),
        _ => Err(anyhow!("Expected numeric or string value for big integer")),
    }
}

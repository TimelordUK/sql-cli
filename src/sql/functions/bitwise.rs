//! Bitwise operations and binary visualization functions
//!
//! This module provides additional bitwise functions not covered by the bigint module.
//! Basic operations like BITAND, BITOR, BITXOR, TO_BINARY, FROM_BINARY are in bigint.

use anyhow::{anyhow, Result};

use crate::data::datatable::DataValue;
use crate::sql::functions::{
    ArgCount, FunctionCategory, FunctionRegistry, FunctionSignature, SqlFunction,
};

/// BITNOT(a) - Bitwise NOT operation
pub struct BitNotFunction;

impl SqlFunction for BitNotFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "BITNOT",
            category: FunctionCategory::Bitwise,
            arg_count: ArgCount::Fixed(1),
            description: "Performs bitwise NOT operation (ones' complement)",
            returns: "Integer result of ~a",
            examples: vec![
                "SELECT BITNOT(0)      -- Returns -1 (all bits set)",
                "SELECT BITNOT(255)    -- Returns -256",
                "SELECT BITNOT(-1)     -- Returns 0",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Err(anyhow!("BITNOT requires exactly 1 argument"));
        }

        match &args[0] {
            DataValue::Integer(a) => Ok(DataValue::Integer(!a)),
            DataValue::Null => Ok(DataValue::Null),
            _ => Err(anyhow!("BITNOT requires an integer argument")),
        }
    }
}

/// IS_POWER_OF_TWO(n) - Check if number is exact power of two
pub struct IsPowerOfTwoFunction;

impl SqlFunction for IsPowerOfTwoFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "IS_POWER_OF_TWO",
            category: FunctionCategory::Bitwise,
            arg_count: ArgCount::Fixed(1),
            description: "Checks if a number is an exact power of two using n & (n-1) == 0",
            returns: "Boolean true if power of two, false otherwise",
            examples: vec![
                "SELECT IS_POWER_OF_TWO(16)  -- Returns true (2^4)",
                "SELECT IS_POWER_OF_TWO(15)  -- Returns false",
                "SELECT IS_POWER_OF_TWO(1)   -- Returns true (2^0)",
                "SELECT IS_POWER_OF_TWO(0)   -- Returns false",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Err(anyhow!("IS_POWER_OF_TWO requires exactly 1 argument"));
        }

        match &args[0] {
            DataValue::Integer(n) => {
                // A number is a power of two if:
                // 1. It's positive (greater than 0)
                // 2. n & (n-1) == 0
                let is_power = *n > 0 && (n & (n - 1)) == 0;
                Ok(DataValue::Boolean(is_power))
            }
            DataValue::Null => Ok(DataValue::Null),
            _ => Err(anyhow!("IS_POWER_OF_TWO requires an integer argument")),
        }
    }
}

/// COUNT_BITS(n) - Count number of set bits (popcount)
pub struct CountBitsFunction;

impl SqlFunction for CountBitsFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "COUNT_BITS",
            category: FunctionCategory::Bitwise,
            arg_count: ArgCount::Fixed(1),
            description: "Counts the number of set bits (1s) in the binary representation",
            returns: "Integer count of set bits",
            examples: vec![
                "SELECT COUNT_BITS(7)    -- Returns 3 (111 has three 1s)",
                "SELECT COUNT_BITS(255)  -- Returns 8 (11111111 has eight 1s)",
                "SELECT COUNT_BITS(16)   -- Returns 1 (10000 has one 1)",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Err(anyhow!("COUNT_BITS requires exactly 1 argument"));
        }

        match &args[0] {
            DataValue::Integer(n) => {
                // Use built-in popcount
                let count = (*n as u64).count_ones() as i64;
                Ok(DataValue::Integer(count))
            }
            DataValue::Null => Ok(DataValue::Null),
            _ => Err(anyhow!("COUNT_BITS requires an integer argument")),
        }
    }
}

/// BINARY_FORMAT(n, separator, group_size) - Format binary with separator
pub struct BinaryFormatFunction;

impl SqlFunction for BinaryFormatFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "BINARY_FORMAT",
            category: FunctionCategory::Bitwise,
            arg_count: ArgCount::Range(1, 3),
            description: "Formats binary string with separators for readability",
            returns: "Formatted binary string",
            examples: vec![
                "SELECT BINARY_FORMAT(255)           -- Returns '11111111'",
                "SELECT BINARY_FORMAT(255, '_')      -- Returns '1111_1111' (groups of 4)",
                "SELECT BINARY_FORMAT(65535, '_', 8) -- Returns '11111111_11111111' (groups of 8)",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.is_empty() || args.len() > 3 {
            return Err(anyhow!("BINARY_FORMAT requires 1-3 arguments"));
        }

        let value = match &args[0] {
            DataValue::Integer(n) => *n,
            DataValue::Null => return Ok(DataValue::Null),
            _ => {
                return Err(anyhow!(
                    "BINARY_FORMAT requires an integer as first argument"
                ))
            }
        };

        let separator = if args.len() >= 2 {
            match &args[1] {
                DataValue::String(s) => s.clone(),
                DataValue::Null => String::new(),
                _ => return Err(anyhow!("Separator must be a string")),
            }
        } else {
            String::new()
        };

        let group_size = if args.len() == 3 {
            match &args[2] {
                DataValue::Integer(g) => {
                    if *g <= 0 {
                        return Err(anyhow!("Group size must be positive"));
                    }
                    *g as usize
                }
                DataValue::Null => 4, // Default to groups of 4
                _ => return Err(anyhow!("Group size must be an integer")),
            }
        } else {
            4 // Default to groups of 4
        };

        // Convert to binary string
        let binary = if value >= 0 {
            format!("{:b}", value)
        } else {
            format!("{:b}", value as u64)
        };

        // Add separators if requested
        let result = if !separator.is_empty() && group_size > 0 {
            let mut formatted = String::new();
            let mut chars: Vec<char> = binary.chars().collect();

            // Process from right to left for consistent grouping
            while !chars.is_empty() {
                let group_start = chars.len().saturating_sub(group_size);
                let group: String = chars.drain(group_start..).collect();

                if !formatted.is_empty() {
                    formatted = format!("{}{}{}", group, separator, formatted);
                } else {
                    formatted = group;
                }
            }
            formatted
        } else {
            binary
        };

        Ok(DataValue::String(result))
    }
}

/// NEXT_POWER_OF_TWO(n) - Returns the next power of two greater than or equal to n
pub struct NextPowerOfTwoFunction;

impl SqlFunction for NextPowerOfTwoFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "NEXT_POWER_OF_TWO",
            category: FunctionCategory::Bitwise,
            arg_count: ArgCount::Fixed(1),
            description: "Returns the next power of two greater than or equal to n",
            returns: "Integer that is the next power of two",
            examples: vec![
                "SELECT NEXT_POWER_OF_TWO(5)   -- Returns 8",
                "SELECT NEXT_POWER_OF_TWO(16)  -- Returns 16 (already power of 2)",
                "SELECT NEXT_POWER_OF_TWO(17)  -- Returns 32",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Err(anyhow!("NEXT_POWER_OF_TWO requires exactly 1 argument"));
        }

        match &args[0] {
            DataValue::Integer(n) => {
                if *n <= 0 {
                    return Ok(DataValue::Integer(1));
                }

                // Find the next power of two
                let mut power = 1i64;
                while power < *n && power < i64::MAX / 2 {
                    power <<= 1;
                }

                Ok(DataValue::Integer(power))
            }
            DataValue::Null => Ok(DataValue::Null),
            _ => Err(anyhow!("NEXT_POWER_OF_TWO requires an integer argument")),
        }
    }
}

/// HIGHEST_BIT(n) - Returns the position of the highest set bit
pub struct HighestBitFunction;

impl SqlFunction for HighestBitFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "HIGHEST_BIT",
            category: FunctionCategory::Bitwise,
            arg_count: ArgCount::Fixed(1),
            description: "Returns the position of the highest set bit (0-indexed)",
            returns: "Integer position of highest bit, or -1 if no bits set",
            examples: vec![
                "SELECT HIGHEST_BIT(8)    -- Returns 3 (bit 3 is set in 1000)",
                "SELECT HIGHEST_BIT(255)  -- Returns 7 (bit 7 is highest in 11111111)",
                "SELECT HIGHEST_BIT(0)    -- Returns -1 (no bits set)",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Err(anyhow!("HIGHEST_BIT requires exactly 1 argument"));
        }

        match &args[0] {
            DataValue::Integer(n) => {
                if *n <= 0 {
                    return Ok(DataValue::Integer(-1));
                }

                // Find the position of the highest bit
                let position = 63 - (*n as u64).leading_zeros() as i64;
                Ok(DataValue::Integer(position))
            }
            DataValue::Null => Ok(DataValue::Null),
            _ => Err(anyhow!("HIGHEST_BIT requires an integer argument")),
        }
    }
}

/// LOWEST_BIT(n) - Returns the position of the lowest set bit
pub struct LowestBitFunction;

impl SqlFunction for LowestBitFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "LOWEST_BIT",
            category: FunctionCategory::Bitwise,
            arg_count: ArgCount::Fixed(1),
            description: "Returns the position of the lowest set bit (0-indexed)",
            returns: "Integer position of lowest bit, or -1 if no bits set",
            examples: vec![
                "SELECT LOWEST_BIT(8)    -- Returns 3 (bit 3 is the only bit in 1000)",
                "SELECT LOWEST_BIT(12)   -- Returns 2 (bit 2 is lowest in 1100)",
                "SELECT LOWEST_BIT(0)    -- Returns -1 (no bits set)",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Err(anyhow!("LOWEST_BIT requires exactly 1 argument"));
        }

        match &args[0] {
            DataValue::Integer(n) => {
                if *n == 0 {
                    return Ok(DataValue::Integer(-1));
                }

                // Find the position of the lowest bit using trailing zeros
                let position = (*n as u64).trailing_zeros() as i64;
                Ok(DataValue::Integer(position))
            }
            DataValue::Null => Ok(DataValue::Null),
            _ => Err(anyhow!("LOWEST_BIT requires an integer argument")),
        }
    }
}

/// Register all bitwise functions
pub fn register_bitwise_functions(registry: &mut FunctionRegistry) {
    registry.register(Box::new(BitNotFunction));
    registry.register(Box::new(IsPowerOfTwoFunction));
    registry.register(Box::new(CountBitsFunction));
    registry.register(Box::new(BinaryFormatFunction));
    registry.register(Box::new(NextPowerOfTwoFunction));
    registry.register(Box::new(HighestBitFunction));
    registry.register(Box::new(LowestBitFunction));
}

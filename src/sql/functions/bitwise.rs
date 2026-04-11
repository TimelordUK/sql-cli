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

/// POPCOUNT(n) - Population count: number of set bits in an integer.
///
/// This is the canonical CPU/intrinsic name for what `COUNT_BITS` also does,
/// and is polymorphic with `BIT_COUNT` (which also accepts binary strings).
pub struct PopcountFunction;

impl SqlFunction for PopcountFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "POPCOUNT",
            category: FunctionCategory::Bitwise,
            arg_count: ArgCount::Fixed(1),
            description: "Population count: number of set bits (1s) in an integer",
            returns: "Integer count of set bits",
            examples: vec![
                "SELECT POPCOUNT(7)    -- Returns 3 (0b111)",
                "SELECT POPCOUNT(255)  -- Returns 8",
                "SELECT POPCOUNT(0)    -- Returns 0",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Err(anyhow!("POPCOUNT requires exactly 1 argument"));
        }

        match &args[0] {
            DataValue::Integer(n) => Ok(DataValue::Integer((*n as u64).count_ones() as i64)),
            DataValue::Null => Ok(DataValue::Null),
            _ => Err(anyhow!("POPCOUNT requires an integer argument")),
        }
    }
}

/// Shared helper: parse the optional `width` argument (8/16/32/64) for
/// LEADING_ZEROS / TRAILING_ONES style functions. Returns the width in bits.
fn parse_bit_width(arg: &DataValue, func_name: &str) -> Result<u32> {
    match arg {
        DataValue::Integer(w) => match *w {
            8 | 16 | 32 | 64 => Ok(*w as u32),
            other => Err(anyhow!(
                "{func_name}: width must be 8, 16, 32, or 64 (got {other})"
            )),
        },
        _ => Err(anyhow!("{func_name}: width must be an integer")),
    }
}

/// LEADING_ZEROS(n [, width]) - Number of leading zero bits in the binary
/// representation of `n`, viewed as an unsigned integer of the given width.
///
/// `width` defaults to 64 so the result passes through `u64::leading_zeros`
/// directly. Use an explicit width (8/16/32/64) to get the intuitive answer
/// within a smaller container (e.g. `LEADING_ZEROS(8, 8)` -> 4).
pub struct LeadingZerosFunction;

impl SqlFunction for LeadingZerosFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "LEADING_ZEROS",
            category: FunctionCategory::Bitwise,
            arg_count: ArgCount::Range(1, 2),
            description: "Number of leading zero bits (optionally within a given bit width: 8/16/32/64, default 64)",
            returns: "Integer count of leading zeros",
            examples: vec![
                "SELECT LEADING_ZEROS(1)          -- Returns 63 (64-bit view)",
                "SELECT LEADING_ZEROS(8, 8)       -- Returns 4 (00001000)",
                "SELECT LEADING_ZEROS(1, 16)      -- Returns 15",
                "SELECT LEADING_ZEROS(0, 32)      -- Returns 32",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.is_empty() || args.len() > 2 {
            return Err(anyhow!("LEADING_ZEROS requires 1 or 2 arguments"));
        }

        let n = match &args[0] {
            DataValue::Integer(n) => *n,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("LEADING_ZEROS requires an integer argument")),
        };

        let width = if args.len() == 2 {
            parse_bit_width(&args[1], "LEADING_ZEROS")?
        } else {
            64
        };

        // Mask down to `width` bits so the count is relative to that container.
        // For width=64 the mask is !0 which keeps every bit; smaller widths
        // truncate so e.g. LEADING_ZEROS(-1, 8) == 0 (all eight bits are set).
        let mask: u64 = if width == 64 {
            !0u64
        } else {
            (1u64 << width) - 1
        };
        let masked = (n as u64) & mask;

        let count = if masked == 0 {
            width as i64
        } else {
            // `u64::leading_zeros` counts against the 64-bit container, so we
            // subtract the overhead above `width` to get the result within the
            // requested window.
            (masked.leading_zeros() as i64) - (64 - width as i64)
        };

        Ok(DataValue::Integer(count))
    }
}

/// TRAILING_ZEROS(n) - Number of trailing zero bits in `n`.
///
/// Width-invariant (trailing zeros from the low end are the same regardless of
/// the container width), so no width argument is needed. For `n == 0` this
/// returns -1 to match the convention used by `LOWEST_BIT` — "no bits set".
pub struct TrailingZerosFunction;

impl SqlFunction for TrailingZerosFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "TRAILING_ZEROS",
            category: FunctionCategory::Bitwise,
            arg_count: ArgCount::Fixed(1),
            description: "Number of trailing zero bits; returns -1 if n is zero",
            returns: "Integer count of trailing zeros, or -1",
            examples: vec![
                "SELECT TRAILING_ZEROS(8)   -- Returns 3 (0b1000)",
                "SELECT TRAILING_ZEROS(12)  -- Returns 2 (0b1100)",
                "SELECT TRAILING_ZEROS(1)   -- Returns 0",
                "SELECT TRAILING_ZEROS(0)   -- Returns -1",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Err(anyhow!("TRAILING_ZEROS requires exactly 1 argument"));
        }

        match &args[0] {
            DataValue::Integer(n) => {
                if *n == 0 {
                    Ok(DataValue::Integer(-1))
                } else {
                    Ok(DataValue::Integer((*n as u64).trailing_zeros() as i64))
                }
            }
            DataValue::Null => Ok(DataValue::Null),
            _ => Err(anyhow!("TRAILING_ZEROS requires an integer argument")),
        }
    }
}

/// Register all bitwise functions
pub fn register_bitwise_functions(registry: &mut FunctionRegistry) {
    registry.register(Box::new(BitNotFunction));
    registry.register(Box::new(IsPowerOfTwoFunction));
    registry.register(Box::new(CountBitsFunction));
    registry.register(Box::new(PopcountFunction));
    registry.register(Box::new(BinaryFormatFunction));
    registry.register(Box::new(NextPowerOfTwoFunction));
    registry.register(Box::new(HighestBitFunction));
    registry.register(Box::new(LowestBitFunction));
    registry.register(Box::new(LeadingZerosFunction));
    registry.register(Box::new(TrailingZerosFunction));
}

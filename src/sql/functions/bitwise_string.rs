//! Bitwise operations on binary string representations

use crate::data::datatable::DataValue;
use crate::sql::functions::{ArgCount, FunctionCategory, FunctionSignature, SqlFunction};
use anyhow::{anyhow, Result};

/// BIT_AND_STR - Bitwise AND on binary strings
pub struct BitAndStr;

impl SqlFunction for BitAndStr {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "BIT_AND_STR",
            category: FunctionCategory::Bitwise,
            arg_count: ArgCount::Fixed(2),
            description: "Performs bitwise AND on two binary strings",
            returns: "Binary string result",
            examples: vec![
                "SELECT BIT_AND_STR('1101', '1011')",
                "SELECT BIT_AND_STR(TO_BINARY(13), TO_BINARY(11))",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let a = args[0].to_string();
        let b = args[1].to_string();

        // Pad to same length
        let max_len = a.len().max(b.len());
        let a_padded = format!("{:0>width$}", a, width = max_len);
        let b_padded = format!("{:0>width$}", b, width = max_len);

        let result: String = a_padded
            .chars()
            .zip(b_padded.chars())
            .map(|(c1, c2)| match (c1, c2) {
                ('1', '1') => '1',
                _ => '0',
            })
            .collect();

        Ok(DataValue::String(result))
    }
}

/// BIT_OR_STR - Bitwise OR on binary strings
pub struct BitOrStr;

impl SqlFunction for BitOrStr {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "BIT_OR_STR",
            category: FunctionCategory::Bitwise,
            arg_count: ArgCount::Fixed(2),
            description: "Performs bitwise OR on two binary strings",
            returns: "Binary string result",
            examples: vec![
                "SELECT BIT_OR_STR('1100', '1010')",
                "SELECT BIT_OR_STR(TO_BINARY(12), TO_BINARY(10))",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let a = args[0].to_string();
        let b = args[1].to_string();

        let max_len = a.len().max(b.len());
        let a_padded = format!("{:0>width$}", a, width = max_len);
        let b_padded = format!("{:0>width$}", b, width = max_len);

        let result: String = a_padded
            .chars()
            .zip(b_padded.chars())
            .map(|(c1, c2)| match (c1, c2) {
                ('0', '0') => '0',
                _ => '1',
            })
            .collect();

        Ok(DataValue::String(result))
    }
}

/// BIT_XOR_STR - Bitwise XOR on binary strings
pub struct BitXorStr;

impl SqlFunction for BitXorStr {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "BIT_XOR_STR",
            category: FunctionCategory::Bitwise,
            arg_count: ArgCount::Fixed(2),
            description: "Performs bitwise XOR on two binary strings",
            returns: "Binary string result",
            examples: vec![
                "SELECT BIT_XOR_STR('1100', '1010')",
                "SELECT BIT_XOR_STR(TO_BINARY(12), TO_BINARY(10))",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let a = args[0].to_string();
        let b = args[1].to_string();

        let max_len = a.len().max(b.len());
        let a_padded = format!("{:0>width$}", a, width = max_len);
        let b_padded = format!("{:0>width$}", b, width = max_len);

        let result: String = a_padded
            .chars()
            .zip(b_padded.chars())
            .map(|(c1, c2)| if c1 == c2 { '0' } else { '1' })
            .collect();

        Ok(DataValue::String(result))
    }
}

/// BIT_NOT_STR - Bitwise NOT on binary string
pub struct BitNotStr;

impl SqlFunction for BitNotStr {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "BIT_NOT_STR",
            category: FunctionCategory::Bitwise,
            arg_count: ArgCount::Fixed(1),
            description: "Performs bitwise NOT on a binary string",
            returns: "Binary string with bits flipped",
            examples: vec![
                "SELECT BIT_NOT_STR('1100')",
                "SELECT BIT_NOT_STR(TO_BINARY(12))",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let input = args[0].to_string();

        let result: String = input
            .chars()
            .map(|c| if c == '0' { '1' } else { '0' })
            .collect();

        Ok(DataValue::String(result))
    }
}

/// BIT_FLIP - Alias for BIT_NOT_STR
pub struct BitFlip;

impl SqlFunction for BitFlip {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "BIT_FLIP",
            category: FunctionCategory::Bitwise,
            arg_count: ArgCount::Fixed(1),
            description: "Flips all bits in a binary string (alias for BIT_NOT_STR)",
            returns: "Binary string with bits flipped",
            examples: vec!["SELECT BIT_FLIP('11011010')"],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        BitNotStr.evaluate(args)
    }
}

/// BIT_COUNT - Count number of 1s in binary string
pub struct BitCount;

impl SqlFunction for BitCount {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "BIT_COUNT",
            category: FunctionCategory::Bitwise,
            arg_count: ArgCount::Fixed(1),
            description: "Counts the number of 1 bits in a binary string",
            returns: "Integer count of 1 bits",
            examples: vec![
                "SELECT BIT_COUNT('11011010')",
                "SELECT BIT_COUNT(TO_BINARY(218))",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let input = args[0].to_string();
        let count = input.chars().filter(|&c| c == '1').count() as i64;
        Ok(DataValue::Integer(count))
    }
}

/// BIT_ROTATE_LEFT - Rotate binary string left by N positions
pub struct BitRotateLeft;

impl SqlFunction for BitRotateLeft {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "BIT_ROTATE_LEFT",
            category: FunctionCategory::Bitwise,
            arg_count: ArgCount::Fixed(2),
            description: "Rotates a binary string left by N positions",
            returns: "Rotated binary string",
            examples: vec![
                "SELECT BIT_ROTATE_LEFT('11011010', 2)",
                "SELECT BIT_ROTATE_LEFT(TO_BINARY(218), 3)",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let input = args[0].to_string();
        let positions = match &args[1] {
            DataValue::Integer(n) => *n as usize,
            DataValue::Float(f) => *f as usize,
            _ => return Err(anyhow!("Second argument must be a number")),
        };

        if input.is_empty() {
            return Ok(DataValue::String(String::new()));
        }

        let effective_positions = positions % input.len();
        let result = format!(
            "{}{}",
            &input[effective_positions..],
            &input[..effective_positions]
        );

        Ok(DataValue::String(result))
    }
}

/// BIT_ROTATE_RIGHT - Rotate binary string right by N positions
pub struct BitRotateRight;

impl SqlFunction for BitRotateRight {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "BIT_ROTATE_RIGHT",
            category: FunctionCategory::Bitwise,
            arg_count: ArgCount::Fixed(2),
            description: "Rotates a binary string right by N positions",
            returns: "Rotated binary string",
            examples: vec![
                "SELECT BIT_ROTATE_RIGHT('11011010', 2)",
                "SELECT BIT_ROTATE_RIGHT(TO_BINARY(218), 3)",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let input = args[0].to_string();
        let positions = match &args[1] {
            DataValue::Integer(n) => *n as usize,
            DataValue::Float(f) => *f as usize,
            _ => return Err(anyhow!("Second argument must be a number")),
        };

        if input.is_empty() {
            return Ok(DataValue::String(String::new()));
        }

        let effective_positions = positions % input.len();
        let split_point = input.len() - effective_positions;
        let result = format!("{}{}", &input[split_point..], &input[..split_point]);

        Ok(DataValue::String(result))
    }
}

/// BIT_SHIFT_LEFT - Shift binary string left, filling with zeros
pub struct BitShiftLeft;

impl SqlFunction for BitShiftLeft {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "BIT_SHIFT_LEFT",
            category: FunctionCategory::Bitwise,
            arg_count: ArgCount::Fixed(2),
            description: "Shifts a binary string left by N positions, filling with zeros",
            returns: "Shifted binary string",
            examples: vec![
                "SELECT BIT_SHIFT_LEFT('11011010', 2)",
                "SELECT BIT_SHIFT_LEFT(TO_BINARY(218), 3)",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let input = args[0].to_string();
        let positions = match &args[1] {
            DataValue::Integer(n) => *n as usize,
            DataValue::Float(f) => *f as usize,
            _ => return Err(anyhow!("Second argument must be a number")),
        };

        if positions >= input.len() {
            return Ok(DataValue::String("0".repeat(input.len())));
        }

        let result = format!("{}{}", &input[positions..], "0".repeat(positions));

        Ok(DataValue::String(result))
    }
}

/// BIT_SHIFT_RIGHT - Shift binary string right, filling with zeros
pub struct BitShiftRight;

impl SqlFunction for BitShiftRight {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "BIT_SHIFT_RIGHT",
            category: FunctionCategory::Bitwise,
            arg_count: ArgCount::Fixed(2),
            description: "Shifts a binary string right by N positions, filling with zeros",
            returns: "Shifted binary string",
            examples: vec![
                "SELECT BIT_SHIFT_RIGHT('11011010', 2)",
                "SELECT BIT_SHIFT_RIGHT(TO_BINARY(218), 3)",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let input = args[0].to_string();
        let positions = match &args[1] {
            DataValue::Integer(n) => *n as usize,
            DataValue::Float(f) => *f as usize,
            _ => return Err(anyhow!("Second argument must be a number")),
        };

        if positions >= input.len() {
            return Ok(DataValue::String("0".repeat(input.len())));
        }

        let result = format!(
            "{}{}",
            "0".repeat(positions),
            &input[..input.len() - positions]
        );

        Ok(DataValue::String(result))
    }
}

/// HAMMING_DISTANCE - Count bit differences between two binary strings
pub struct HammingDistance;

impl SqlFunction for HammingDistance {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "HAMMING_DISTANCE",
            category: FunctionCategory::Bitwise,
            arg_count: ArgCount::Fixed(2),
            description: "Counts the number of differing bits between two binary strings",
            returns: "Integer count of different bits",
            examples: vec![
                "SELECT HAMMING_DISTANCE('1101', '1011')",
                "SELECT HAMMING_DISTANCE(TO_BINARY(13), TO_BINARY(11))",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let a = args[0].to_string();
        let b = args[1].to_string();

        let max_len = a.len().max(b.len());
        let a_padded = format!("{:0>width$}", a, width = max_len);
        let b_padded = format!("{:0>width$}", b, width = max_len);

        let distance = a_padded
            .chars()
            .zip(b_padded.chars())
            .filter(|(c1, c2)| c1 != c2)
            .count() as i64;

        Ok(DataValue::Integer(distance))
    }
}

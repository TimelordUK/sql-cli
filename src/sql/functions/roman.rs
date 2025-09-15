use crate::data::datatable::DataValue;
use crate::sql::functions::{ArgCount, FunctionCategory, FunctionSignature, SqlFunction};
use anyhow::Result;

/// Convert integer to Roman numerals
pub struct ToRoman;

impl SqlFunction for ToRoman {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "TO_ROMAN",
            category: FunctionCategory::Conversion,
            arg_count: ArgCount::Fixed(1),
            description: "Convert integer to Roman numerals (1-3999)",
            returns: "String with Roman numeral representation",
            examples: vec![
                "SELECT TO_ROMAN(2024)     -- Returns 'MMXXIV'",
                "SELECT TO_ROMAN(1984)     -- Returns 'MCMLXXXIV'",
                "SELECT TO_ROMAN(49)       -- Returns 'XLIX'",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Ok(DataValue::Null);
        }

        let num = match &args[0] {
            DataValue::Float(n) => *n as i32,
            DataValue::Integer(n) => *n as i32,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Ok(DataValue::Null),
        };

        if num <= 0 || num > 3999 {
            return Ok(DataValue::String(format!("OUT_OF_RANGE({})", num)));
        }

        Ok(DataValue::String(int_to_roman(num)))
    }
}

/// Convert Roman numerals to integer
pub struct FromRoman;

impl SqlFunction for FromRoman {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "FROM_ROMAN",
            category: FunctionCategory::Conversion,
            arg_count: ArgCount::Fixed(1),
            description: "Convert Roman numerals to integer",
            returns: "Integer value of Roman numeral",
            examples: vec![
                "SELECT FROM_ROMAN('MMXXIV')     -- Returns 2024",
                "SELECT FROM_ROMAN('MCMLXXXIV')  -- Returns 1984",
                "SELECT FROM_ROMAN('XLIX')       -- Returns 49",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Ok(DataValue::Null);
        }

        let roman = match &args[0] {
            DataValue::String(s) => s.to_uppercase(),
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Ok(DataValue::Null),
        };

        match roman_to_int(&roman) {
            Some(n) => Ok(DataValue::Float(n as f64)),
            None => Ok(DataValue::Null),
        }
    }
}

/// Convert integer to Roman numerals
fn int_to_roman(mut num: i32) -> String {
    let values = [
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];

    let mut result = String::new();

    for (value, numeral) in values.iter() {
        while num >= *value {
            result.push_str(numeral);
            num -= *value;
        }
    }

    result
}

/// Convert Roman numerals to integer
fn roman_to_int(s: &str) -> Option<i32> {
    let mut result = 0;
    let mut prev_value = 0;

    for c in s.chars().rev() {
        let value = match c {
            'I' => 1,
            'V' => 5,
            'X' => 10,
            'L' => 50,
            'C' => 100,
            'D' => 500,
            'M' => 1000,
            _ => return None, // Invalid character
        };

        if value < prev_value {
            result -= value;
        } else {
            result += value;
        }

        prev_value = value;
    }

    // Validate the result by converting back
    if int_to_roman(result).to_uppercase() == s.to_uppercase() {
        Some(result)
    } else {
        None // Invalid Roman numeral
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_roman() {
        let func = ToRoman;

        // Test various numbers
        assert_eq!(
            func.evaluate(&[DataValue::Float(1.0)]).unwrap(),
            DataValue::String("I".to_string())
        );

        assert_eq!(
            func.evaluate(&[DataValue::Float(49.0)]).unwrap(),
            DataValue::String("XLIX".to_string())
        );

        assert_eq!(
            func.evaluate(&[DataValue::Float(2024.0)]).unwrap(),
            DataValue::String("MMXXIV".to_string())
        );

        assert_eq!(
            func.evaluate(&[DataValue::Float(1984.0)]).unwrap(),
            DataValue::String("MCMLXXXIV".to_string())
        );

        assert_eq!(
            func.evaluate(&[DataValue::Float(3999.0)]).unwrap(),
            DataValue::String("MMMCMXCIX".to_string())
        );
    }

    #[test]
    fn test_from_roman() {
        let func = FromRoman;

        assert_eq!(
            func.evaluate(&[DataValue::String("I".to_string())]).unwrap(),
            DataValue::Float(1.0)
        );

        assert_eq!(
            func.evaluate(&[DataValue::String("XLIX".to_string())]).unwrap(),
            DataValue::Float(49.0)
        );

        assert_eq!(
            func.evaluate(&[DataValue::String("MMXXIV".to_string())]).unwrap(),
            DataValue::Float(2024.0)
        );

        assert_eq!(
            func.evaluate(&[DataValue::String("mcmlxxxiv".to_string())]).unwrap(),
            DataValue::Float(1984.0)
        );
    }

    #[test]
    fn test_round_trip() {
        let to_roman = ToRoman;
        let from_roman = FromRoman;

        for i in 1..=3999 {
            let roman = to_roman.evaluate(&[DataValue::Float(i as f64)]).unwrap();
            let back = from_roman.evaluate(&[roman]).unwrap();
            assert_eq!(back, DataValue::Float(i as f64));
        }
    }
}
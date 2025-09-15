use crate::data::datatable::DataValue;
use crate::sql::functions::{ArgCount, FunctionCategory, FunctionSignature, SqlFunction};
use anyhow::Result;

/// Convert number to English words
pub struct ToWords;

impl SqlFunction for ToWords {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "TO_WORDS",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(1),
            description: "Convert number to English words",
            returns: "String with number spelled out in words",
            examples: vec![
                "SELECT TO_WORDS(42)        -- Returns 'forty-two'",
                "SELECT TO_WORDS(999)       -- Returns 'nine hundred ninety-nine'",
                "SELECT TO_WORDS(1234)      -- Returns 'one thousand two hundred thirty-four'",
                "SELECT TO_WORDS(1000000)   -- Returns 'one million'",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Ok(DataValue::Null);
        }

        let num = match &args[0] {
            DataValue::Float(n) => *n as i64,
            DataValue::Integer(n) => *n,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Ok(DataValue::Null),
        };

        if num < 0 {
            Ok(DataValue::String(format!(
                "negative {}",
                number_to_words(-num)
            )))
        } else {
            Ok(DataValue::String(number_to_words(num)))
        }
    }
}

/// Convert number to ordinal words (1st, 2nd, 3rd, etc.)
pub struct ToOrdinal;

impl SqlFunction for ToOrdinal {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "TO_ORDINAL",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(1),
            description: "Convert number to ordinal form (1st, 2nd, 3rd, etc.)",
            returns: "String with ordinal representation",
            examples: vec![
                "SELECT TO_ORDINAL(1)       -- Returns '1st'",
                "SELECT TO_ORDINAL(2)       -- Returns '2nd'",
                "SELECT TO_ORDINAL(21)      -- Returns '21st'",
                "SELECT TO_ORDINAL(123)     -- Returns '123rd'",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Ok(DataValue::Null);
        }

        let num = match &args[0] {
            DataValue::Float(n) => *n as i64,
            DataValue::Integer(n) => *n,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Ok(DataValue::Null),
        };

        Ok(DataValue::String(number_to_ordinal(num)))
    }
}

/// Convert number to ordinal words
pub struct ToOrdinalWords;

impl SqlFunction for ToOrdinalWords {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "TO_ORDINAL_WORDS",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(1),
            description: "Convert number to ordinal words (first, second, third, etc.)",
            returns: "String with ordinal words",
            examples: vec![
                "SELECT TO_ORDINAL_WORDS(1)   -- Returns 'first'",
                "SELECT TO_ORDINAL_WORDS(21)  -- Returns 'twenty-first'",
                "SELECT TO_ORDINAL_WORDS(100) -- Returns 'one hundredth'",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Ok(DataValue::Null);
        }

        let num = match &args[0] {
            DataValue::Float(n) => *n as i64,
            DataValue::Integer(n) => *n,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Ok(DataValue::Null),
        };

        Ok(DataValue::String(number_to_ordinal_words(num)))
    }
}

/// Convert a number to English words
fn number_to_words(n: i64) -> String {
    if n == 0 {
        return "zero".to_string();
    }

    let units = [
        "",
        "one",
        "two",
        "three",
        "four",
        "five",
        "six",
        "seven",
        "eight",
        "nine",
        "ten",
        "eleven",
        "twelve",
        "thirteen",
        "fourteen",
        "fifteen",
        "sixteen",
        "seventeen",
        "eighteen",
        "nineteen",
    ];

    let tens = [
        "", "", "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety",
    ];

    let thousands = [
        "",
        "thousand",
        "million",
        "billion",
        "trillion",
        "quadrillion",
    ];

    fn convert_hundreds(n: i64, units: &[&str], tens: &[&str]) -> String {
        let mut result = String::new();

        let hundreds = n / 100;
        let remainder = n % 100;

        if hundreds > 0 {
            result.push_str(units[hundreds as usize]);
            result.push_str(" hundred");
            if remainder > 0 {
                result.push(' ');
            }
        }

        if remainder < 20 {
            result.push_str(units[remainder as usize]);
        } else {
            let tens_digit = remainder / 10;
            let ones_digit = remainder % 10;
            result.push_str(tens[tens_digit as usize]);
            if ones_digit > 0 {
                result.push('-');
                result.push_str(units[ones_digit as usize]);
            }
        }

        result
    }

    let mut num = n;
    let mut parts = Vec::new();
    let mut thousand_index = 0;

    while num > 0 && thousand_index < thousands.len() {
        let group = num % 1000;
        if group > 0 {
            let mut part = convert_hundreds(group, &units, &tens);
            if !thousands[thousand_index].is_empty() {
                part.push(' ');
                part.push_str(thousands[thousand_index]);
            }
            parts.push(part);
        }
        num /= 1000;
        thousand_index += 1;
    }

    parts.reverse();
    parts.join(" ")
}

/// Convert number to ordinal form (1st, 2nd, 3rd, etc.)
fn number_to_ordinal(n: i64) -> String {
    let abs_n = n.abs();
    let suffix = if abs_n % 100 >= 11 && abs_n % 100 <= 13 {
        "th"
    } else {
        match abs_n % 10 {
            1 => "st",
            2 => "nd",
            3 => "rd",
            _ => "th",
        }
    };

    format!("{}{}", n, suffix)
}

/// Convert number to ordinal words
fn number_to_ordinal_words(n: i64) -> String {
    // Special cases for common ordinals
    match n {
        1 => return "first".to_string(),
        2 => return "second".to_string(),
        3 => return "third".to_string(),
        4 => return "fourth".to_string(),
        5 => return "fifth".to_string(),
        6 => return "sixth".to_string(),
        7 => return "seventh".to_string(),
        8 => return "eighth".to_string(),
        9 => return "ninth".to_string(),
        10 => return "tenth".to_string(),
        11 => return "eleventh".to_string(),
        12 => return "twelfth".to_string(),
        20 => return "twentieth".to_string(),
        30 => return "thirtieth".to_string(),
        40 => return "fortieth".to_string(),
        50 => return "fiftieth".to_string(),
        60 => return "sixtieth".to_string(),
        70 => return "seventieth".to_string(),
        80 => return "eightieth".to_string(),
        90 => return "ninetieth".to_string(),
        100 => return "one hundredth".to_string(),
        1000 => return "one thousandth".to_string(),
        _ => {}
    }

    // For compound numbers, convert the base and add ordinal to the last part
    if n < 100 && n > 20 {
        let tens = (n / 10) * 10;
        let ones = n % 10;
        if ones == 0 {
            return number_to_ordinal_words(tens);
        }
        let tens_word = number_to_words(tens);
        let ones_ordinal = number_to_ordinal_words(ones);
        return format!("{}-{}", tens_word, ones_ordinal);
    }

    // For larger numbers, convert to words and add "th"
    let words = number_to_words(n);

    // Handle special endings
    if words.ends_with("one") {
        format!(
            "{}st",
            words
                .trim_end_matches("one")
                .trim_end_matches('-')
                .trim_end_matches(' ')
        )
    } else if words.ends_with("two") {
        format!(
            "{}second",
            words
                .trim_end_matches("two")
                .trim_end_matches('-')
                .trim_end_matches(' ')
        )
    } else if words.ends_with("three") {
        format!(
            "{}third",
            words
                .trim_end_matches("three")
                .trim_end_matches('-')
                .trim_end_matches(' ')
        )
    } else if words.ends_with("y") {
        format!("{}ieth", words.trim_end_matches('y'))
    } else {
        format!("{}th", words)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_words() {
        let func = ToWords;

        assert_eq!(
            func.evaluate(&[DataValue::Float(0.0)]).unwrap(),
            DataValue::String("zero".to_string())
        );

        assert_eq!(
            func.evaluate(&[DataValue::Float(42.0)]).unwrap(),
            DataValue::String("forty-two".to_string())
        );

        assert_eq!(
            func.evaluate(&[DataValue::Float(999.0)]).unwrap(),
            DataValue::String("nine hundred ninety-nine".to_string())
        );

        assert_eq!(
            func.evaluate(&[DataValue::Float(1234.0)]).unwrap(),
            DataValue::String("one thousand two hundred thirty-four".to_string())
        );

        assert_eq!(
            func.evaluate(&[DataValue::Float(1000000.0)]).unwrap(),
            DataValue::String("one million".to_string())
        );
    }

    #[test]
    fn test_to_ordinal() {
        let func = ToOrdinal;

        assert_eq!(
            func.evaluate(&[DataValue::Float(1.0)]).unwrap(),
            DataValue::String("1st".to_string())
        );

        assert_eq!(
            func.evaluate(&[DataValue::Float(2.0)]).unwrap(),
            DataValue::String("2nd".to_string())
        );

        assert_eq!(
            func.evaluate(&[DataValue::Float(3.0)]).unwrap(),
            DataValue::String("3rd".to_string())
        );

        assert_eq!(
            func.evaluate(&[DataValue::Float(11.0)]).unwrap(),
            DataValue::String("11th".to_string())
        );

        assert_eq!(
            func.evaluate(&[DataValue::Float(21.0)]).unwrap(),
            DataValue::String("21st".to_string())
        );

        assert_eq!(
            func.evaluate(&[DataValue::Float(123.0)]).unwrap(),
            DataValue::String("123rd".to_string())
        );
    }

    #[test]
    fn test_to_ordinal_words() {
        let func = ToOrdinalWords;

        assert_eq!(
            func.evaluate(&[DataValue::Float(1.0)]).unwrap(),
            DataValue::String("first".to_string())
        );

        assert_eq!(
            func.evaluate(&[DataValue::Float(21.0)]).unwrap(),
            DataValue::String("twenty-first".to_string())
        );

        assert_eq!(
            func.evaluate(&[DataValue::Float(100.0)]).unwrap(),
            DataValue::String("one hundredth".to_string())
        );
    }
}

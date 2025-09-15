use crate::data::datatable::DataValue;
use crate::sql::functions::{ArgCount, FunctionCategory, FunctionSignature, SqlFunction};
use anyhow::Result;
use regex::Regex;

/// Strip all punctuation from text
pub struct StripPunctuation;

impl SqlFunction for StripPunctuation {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "STRIP_PUNCTUATION",
            category: FunctionCategory::String,
            arg_count: ArgCount::Range(1, 2),
            description: "Remove punctuation from text, optionally replacing with a character",
            returns: "String with punctuation removed",
            examples: vec![
                "SELECT STRIP_PUNCTUATION('Hello, world!')           -- Returns 'Hello world'",
                "SELECT STRIP_PUNCTUATION('foo.bar@baz', ' ')        -- Returns 'foo bar baz'",
                "SELECT STRIP_PUNCTUATION('data-file_v2.0', '')      -- Returns 'datafilev20'",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.is_empty() || args.len() > 2 {
            return Ok(DataValue::Null);
        }

        let text = match &args[0] {
            DataValue::String(s) => s.clone(),
            DataValue::InternedString(s) => s.to_string(),
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Ok(DataValue::Null),
        };

        let replacement = if args.len() == 2 {
            match &args[1] {
                DataValue::String(s) => s.clone(),
                DataValue::InternedString(s) => s.to_string(),
                _ => " ".to_string(),
            }
        } else {
            " ".to_string()
        };

        // Replace all punctuation with the replacement character
        let result = text
            .chars()
            .map(|c| {
                if c.is_ascii_punctuation() || !c.is_alphanumeric() && !c.is_whitespace() {
                    replacement.chars().next().unwrap_or(' ')
                } else {
                    c
                }
            })
            .collect::<String>();

        // Clean up multiple spaces if replacement was space
        let result = if replacement == " " {
            result.split_whitespace().collect::<Vec<_>>().join(" ")
        } else {
            result
        };

        Ok(DataValue::String(result))
    }
}

/// Tokenize text into words
pub struct Tokenize;

impl SqlFunction for Tokenize {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "TOKENIZE",
            category: FunctionCategory::String,
            arg_count: ArgCount::Range(1, 2),
            description: "Split text into words, removing punctuation and optionally converting case",
            returns: "Space-separated words",
            examples: vec![
                "SELECT TOKENIZE('Hello, world!')                    -- Returns 'Hello world'",
                "SELECT TOKENIZE('The quick brown fox.', 'lower')    -- Returns 'the quick brown fox'",
                "SELECT TOKENIZE('foo-bar_baz', 'upper')             -- Returns 'FOO BAR BAZ'",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.is_empty() || args.len() > 2 {
            return Ok(DataValue::Null);
        }

        let text = match &args[0] {
            DataValue::String(s) => s.clone(),
            DataValue::InternedString(s) => s.to_string(),
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Ok(DataValue::Null),
        };

        let case_option = if args.len() == 2 {
            match &args[1] {
                DataValue::String(s) => s.to_lowercase(),
                DataValue::InternedString(s) => s.to_lowercase(),
                _ => String::new(),
            }
        } else {
            String::new()
        };

        // Split on any non-alphanumeric character
        let words: Vec<String> = text
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty())
            .map(|s| match case_option.as_str() {
                "lower" | "lowercase" => s.to_lowercase(),
                "upper" | "uppercase" => s.to_uppercase(),
                _ => s.to_string(),
            })
            .collect();

        Ok(DataValue::String(words.join(" ")))
    }
}

/// Clean text by removing extra whitespace and normalizing
pub struct CleanText;

impl SqlFunction for CleanText {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "CLEAN_TEXT",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(1),
            description: "Clean text by normalizing whitespace and removing control characters",
            returns: "Cleaned string",
            examples: vec![
                "SELECT CLEAN_TEXT('  hello   world  ')              -- Returns 'hello world'",
                "SELECT CLEAN_TEXT('line1\\nline2\\tline3')            -- Returns 'line1 line2 line3'",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Ok(DataValue::Null);
        }

        let text = match &args[0] {
            DataValue::String(s) => s.clone(),
            DataValue::InternedString(s) => s.to_string(),
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Ok(DataValue::Null),
        };

        // Replace all whitespace (including newlines, tabs) with single spaces
        // Remove control characters
        let cleaned: String = text
            .chars()
            .filter(|c| !c.is_control() || c.is_whitespace())
            .map(|c| if c.is_whitespace() { ' ' } else { c })
            .collect();

        // Collapse multiple spaces and trim
        let result = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");

        Ok(DataValue::String(result))
    }
}

/// Extract words from text (returns as a single string, use with SPLIT for table)
pub struct ExtractWords;

impl SqlFunction for ExtractWords {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "EXTRACT_WORDS",
            category: FunctionCategory::String,
            arg_count: ArgCount::Range(1, 3),
            description: "Extract words from text, with optional min length and case conversion",
            returns: "Comma-separated list of words",
            examples: vec![
                "SELECT EXTRACT_WORDS('Hello, world!')               -- Returns 'Hello,world'",
                "SELECT EXTRACT_WORDS('The quick fox', 4)            -- Returns 'quick'",
                "SELECT EXTRACT_WORDS('The Fox', 2, 'lower')         -- Returns 'the,fox'",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.is_empty() || args.len() > 3 {
            return Ok(DataValue::Null);
        }

        let text = match &args[0] {
            DataValue::String(s) => s.clone(),
            DataValue::InternedString(s) => s.to_string(),
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Ok(DataValue::Null),
        };

        let min_length = if args.len() >= 2 {
            match &args[1] {
                DataValue::Integer(n) => *n as usize,
                DataValue::Float(n) => *n as usize,
                _ => 1,
            }
        } else {
            1
        };

        let case_option = if args.len() == 3 {
            match &args[2] {
                DataValue::String(s) => s.to_lowercase(),
                DataValue::InternedString(s) => s.to_lowercase(),
                _ => String::new(),
            }
        } else {
            String::new()
        };

        // Extract words
        let words: Vec<String> = text
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty() && s.len() >= min_length)
            .map(|s| match case_option.as_str() {
                "lower" | "lowercase" => s.to_lowercase(),
                "upper" | "uppercase" => s.to_uppercase(),
                _ => s.to_string(),
            })
            .collect();

        Ok(DataValue::String(words.join(",")))
    }
}

/// Count words in text
pub struct WordCount;

impl SqlFunction for WordCount {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "WORD_COUNT",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(1),
            description: "Count the number of words in text",
            returns: "Integer count of words",
            examples: vec![
                "SELECT WORD_COUNT('Hello world')                    -- Returns 2",
                "SELECT WORD_COUNT('The quick brown fox')            -- Returns 4",
                "SELECT WORD_COUNT('one,two;three')                  -- Returns 3",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Ok(DataValue::Null);
        }

        let text = match &args[0] {
            DataValue::String(s) => s.clone(),
            DataValue::InternedString(s) => s.to_string(),
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Ok(DataValue::Null),
        };

        let count = text
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty())
            .count();

        Ok(DataValue::Integer(count as i64))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_punctuation() {
        let func = StripPunctuation;

        assert_eq!(
            func.evaluate(&[DataValue::String("Hello, world!".to_string())])
                .unwrap(),
            DataValue::String("Hello world".to_string())
        );

        assert_eq!(
            func.evaluate(&[
                DataValue::String("foo.bar@baz".to_string()),
                DataValue::String("_".to_string())
            ])
            .unwrap(),
            DataValue::String("foo_bar_baz".to_string())
        );
    }

    #[test]
    fn test_tokenize() {
        let func = Tokenize;

        assert_eq!(
            func.evaluate(&[
                DataValue::String("The quick brown fox.".to_string()),
                DataValue::String("lower".to_string())
            ])
            .unwrap(),
            DataValue::String("the quick brown fox".to_string())
        );
    }

    #[test]
    fn test_word_count() {
        let func = WordCount;

        assert_eq!(
            func.evaluate(&[DataValue::String("Hello world".to_string())])
                .unwrap(),
            DataValue::Integer(2)
        );

        assert_eq!(
            func.evaluate(&[DataValue::String("one,two;three".to_string())])
                .unwrap(),
            DataValue::Integer(3)
        );
    }

    #[test]
    fn test_extract_words() {
        let func = ExtractWords;

        assert_eq!(
            func.evaluate(&[
                DataValue::String("The quick brown fox".to_string()),
                DataValue::Integer(4)
            ])
            .unwrap(),
            DataValue::String("quick,brown".to_string())
        );
    }
}

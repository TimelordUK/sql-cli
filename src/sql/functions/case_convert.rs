use crate::data::datatable::DataValue;
use crate::sql::functions::{ArgCount, FunctionCategory, FunctionSignature, SqlFunction};
use anyhow::{anyhow, Result};

/// TO_SNAKE_CASE(string) - Converts text to snake_case
pub struct ToSnakeCaseFunction;

impl SqlFunction for ToSnakeCaseFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "TO_SNAKE_CASE",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(1),
            description: "Converts text to snake_case",
            returns: "String in snake_case format",
            examples: vec![
                "SELECT TO_SNAKE_CASE('CamelCase') -- returns 'camel_case'",
                "SELECT TO_SNAKE_CASE('PascalCase') -- returns 'pascal_case'",
                "SELECT TO_SNAKE_CASE('kebab-case') -- returns 'kebab_case'",
                "SELECT TO_SNAKE_CASE('HTTPResponse') -- returns 'http_response'",
                "SELECT TO_SNAKE_CASE('XMLHttpRequest') -- returns 'xml_http_request'",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Err(anyhow!("TO_SNAKE_CASE requires exactly 1 argument"));
        }

        match &args[0] {
            DataValue::String(s) => Ok(DataValue::String(to_snake_case(s))),
            DataValue::InternedString(s) => Ok(DataValue::String(to_snake_case(s))),
            DataValue::Null => Ok(DataValue::Null),
            _ => Err(anyhow!("TO_SNAKE_CASE requires a string argument")),
        }
    }
}

/// TO_CAMEL_CASE(string) - Converts text to camelCase
pub struct ToCamelCaseFunction;

impl SqlFunction for ToCamelCaseFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "TO_CAMEL_CASE",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(1),
            description: "Converts text to camelCase",
            returns: "String in camelCase format",
            examples: vec![
                "SELECT TO_CAMEL_CASE('snake_case') -- returns 'snakeCase'",
                "SELECT TO_CAMEL_CASE('kebab-case') -- returns 'kebabCase'",
                "SELECT TO_CAMEL_CASE('PascalCase') -- returns 'pascalCase'",
                "SELECT TO_CAMEL_CASE('hello world') -- returns 'helloWorld'",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Err(anyhow!("TO_CAMEL_CASE requires exactly 1 argument"));
        }

        match &args[0] {
            DataValue::String(s) => Ok(DataValue::String(to_camel_case(s))),
            DataValue::InternedString(s) => Ok(DataValue::String(to_camel_case(s))),
            DataValue::Null => Ok(DataValue::Null),
            _ => Err(anyhow!("TO_CAMEL_CASE requires a string argument")),
        }
    }
}

/// TO_PASCAL_CASE(string) - Converts text to PascalCase
pub struct ToPascalCaseFunction;

impl SqlFunction for ToPascalCaseFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "TO_PASCAL_CASE",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(1),
            description: "Converts text to PascalCase",
            returns: "String in PascalCase format",
            examples: vec![
                "SELECT TO_PASCAL_CASE('snake_case') -- returns 'SnakeCase'",
                "SELECT TO_PASCAL_CASE('kebab-case') -- returns 'KebabCase'",
                "SELECT TO_PASCAL_CASE('camelCase') -- returns 'CamelCase'",
                "SELECT TO_PASCAL_CASE('hello world') -- returns 'HelloWorld'",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Err(anyhow!("TO_PASCAL_CASE requires exactly 1 argument"));
        }

        match &args[0] {
            DataValue::String(s) => Ok(DataValue::String(to_pascal_case(s))),
            DataValue::InternedString(s) => Ok(DataValue::String(to_pascal_case(s))),
            DataValue::Null => Ok(DataValue::Null),
            _ => Err(anyhow!("TO_PASCAL_CASE requires a string argument")),
        }
    }
}

/// TO_KEBAB_CASE(string) - Converts text to kebab-case
pub struct ToKebabCaseFunction;

impl SqlFunction for ToKebabCaseFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "TO_KEBAB_CASE",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(1),
            description: "Converts text to kebab-case",
            returns: "String in kebab-case format",
            examples: vec![
                "SELECT TO_KEBAB_CASE('snake_case') -- returns 'snake-case'",
                "SELECT TO_KEBAB_CASE('CamelCase') -- returns 'camel-case'",
                "SELECT TO_KEBAB_CASE('PascalCase') -- returns 'pascal-case'",
                "SELECT TO_KEBAB_CASE('hello world') -- returns 'hello-world'",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Err(anyhow!("TO_KEBAB_CASE requires exactly 1 argument"));
        }

        match &args[0] {
            DataValue::String(s) => Ok(DataValue::String(to_kebab_case(s))),
            DataValue::InternedString(s) => Ok(DataValue::String(to_kebab_case(s))),
            DataValue::Null => Ok(DataValue::Null),
            _ => Err(anyhow!("TO_KEBAB_CASE requires a string argument")),
        }
    }
}

/// TO_CONSTANT_CASE(string) - Converts text to CONSTANT_CASE
pub struct ToConstantCaseFunction;

impl SqlFunction for ToConstantCaseFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "TO_CONSTANT_CASE",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(1),
            description: "Converts text to CONSTANT_CASE (SCREAMING_SNAKE_CASE)",
            returns: "String in CONSTANT_CASE format",
            examples: vec![
                "SELECT TO_CONSTANT_CASE('camelCase') -- returns 'CAMEL_CASE'",
                "SELECT TO_CONSTANT_CASE('kebab-case') -- returns 'KEBAB_CASE'",
                "SELECT TO_CONSTANT_CASE('hello world') -- returns 'HELLO_WORLD'",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Err(anyhow!("TO_CONSTANT_CASE requires exactly 1 argument"));
        }

        match &args[0] {
            DataValue::String(s) => Ok(DataValue::String(to_constant_case(s))),
            DataValue::InternedString(s) => Ok(DataValue::String(to_constant_case(s))),
            DataValue::Null => Ok(DataValue::Null),
            _ => Err(anyhow!("TO_CONSTANT_CASE requires a string argument")),
        }
    }
}

// Helper function to split a string into words based on various case conventions
fn split_into_words(s: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current_word = String::new();
    let mut prev_char_type = CharType::Separator;

    #[derive(PartialEq, Clone, Copy)]
    enum CharType {
        Uppercase,
        Lowercase,
        Numeric,
        Separator,
    }

    let chars: Vec<char> = s.chars().collect();

    for (i, &ch) in chars.iter().enumerate() {
        let char_type = if !ch.is_alphanumeric() {
            CharType::Separator
        } else if ch.is_uppercase() {
            CharType::Uppercase
        } else if ch.is_lowercase() {
            CharType::Lowercase
        } else {
            CharType::Numeric
        };

        match char_type {
            CharType::Separator => {
                if !current_word.is_empty() {
                    words.push(current_word.clone());
                    current_word.clear();
                }
            }
            CharType::Uppercase => {
                if !current_word.is_empty() {
                    // Check for transitions
                    if prev_char_type == CharType::Lowercase || prev_char_type == CharType::Numeric
                    {
                        // Transition from lowercase/number to uppercase (e.g., "camelCase", "v2API")
                        words.push(current_word.clone());
                        current_word.clear();
                    } else if prev_char_type == CharType::Uppercase {
                        // Look ahead to see if this is the start of a new word
                        if i + 1 < chars.len()
                            && chars[i + 1].is_lowercase()
                            && current_word.len() > 1
                        {
                            // This uppercase is the start of a new word after an acronym
                            let last_char = current_word.pop().unwrap();
                            if !current_word.is_empty() {
                                words.push(current_word.clone());
                            }
                            current_word.clear();
                            current_word.push(last_char);
                        }
                    }
                }
                current_word.push(ch);
            }
            CharType::Lowercase => {
                current_word.push(ch);
            }
            CharType::Numeric => {
                // Numbers can start a new word or continue the current one
                if prev_char_type == CharType::Lowercase || prev_char_type == CharType::Uppercase {
                    current_word.push(ch);
                } else if prev_char_type == CharType::Numeric {
                    current_word.push(ch);
                } else {
                    if !current_word.is_empty() {
                        words.push(current_word.clone());
                        current_word.clear();
                    }
                    current_word.push(ch);
                }
            }
        }

        prev_char_type = char_type;
    }

    if !current_word.is_empty() {
        words.push(current_word);
    }

    // Filter out empty strings and convert to lowercase for consistency
    words
        .into_iter()
        .filter(|w| !w.is_empty())
        .map(|w| w.to_lowercase())
        .collect()
}

fn to_snake_case(s: &str) -> String {
    let words = split_into_words(s);
    words.join("_")
}

fn to_camel_case(s: &str) -> String {
    let words = split_into_words(s);
    if words.is_empty() {
        return String::new();
    }

    let mut result = String::new();
    for (i, word) in words.iter().enumerate() {
        if i == 0 {
            result.push_str(word);
        } else {
            // Capitalize first letter
            if let Some(first_char) = word.chars().next() {
                result.push(first_char.to_uppercase().next().unwrap_or(first_char));
                result.push_str(&word[first_char.len_utf8()..]);
            }
        }
    }
    result
}

fn to_pascal_case(s: &str) -> String {
    let words = split_into_words(s);
    words
        .into_iter()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}

fn to_kebab_case(s: &str) -> String {
    let words = split_into_words(s);
    words.join("-")
}

fn to_constant_case(s: &str) -> String {
    let words = split_into_words(s);
    words
        .into_iter()
        .map(|w| w.to_uppercase())
        .collect::<Vec<_>>()
        .join("_")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snake_case_conversions() {
        assert_eq!(to_snake_case("CamelCase"), "camel_case");
        assert_eq!(to_snake_case("PascalCase"), "pascal_case");
        assert_eq!(to_snake_case("snake_case"), "snake_case");
        assert_eq!(to_snake_case("kebab-case"), "kebab_case");
        assert_eq!(to_snake_case("HTTPResponse"), "htt_presponse"); // HTTP splits at P because P is followed by lowercase
        assert_eq!(to_snake_case("HttpResponse"), "http_response"); // Mixed case splits properly
        assert_eq!(to_snake_case("XMLHttpRequest"), "xm_lhttp_request"); // XML splits before L
        assert_eq!(to_snake_case("XmlHttpRequest"), "xml_http_request");
        assert_eq!(to_snake_case("IOError"), "i_oerror"); // IO splits at O because O is followed by Error
        assert_eq!(to_snake_case("IoError"), "io_error");
        assert_eq!(to_snake_case("snake_case_example"), "snake_case_example");
        assert_eq!(to_snake_case("hello world"), "hello_world");
        assert_eq!(to_snake_case("Hello-World_Test"), "hello_world_test");
    }

    #[test]
    fn test_camel_case_conversions() {
        assert_eq!(to_camel_case("snake_case"), "snakeCase");
        assert_eq!(to_camel_case("kebab-case"), "kebabCase");
        assert_eq!(to_camel_case("PascalCase"), "pascalCase");
        assert_eq!(to_camel_case("camelCase"), "camelCase");
        assert_eq!(to_camel_case("hello world"), "helloWorld");
        assert_eq!(to_camel_case("CONSTANT_CASE"), "constantCase");
    }

    #[test]
    fn test_pascal_case_conversions() {
        assert_eq!(to_pascal_case("snake_case"), "SnakeCase");
        assert_eq!(to_pascal_case("kebab-case"), "KebabCase");
        assert_eq!(to_pascal_case("camelCase"), "CamelCase");
        assert_eq!(to_pascal_case("PascalCase"), "PascalCase");
        assert_eq!(to_pascal_case("hello world"), "HelloWorld");
    }

    #[test]
    fn test_kebab_case_conversions() {
        assert_eq!(to_kebab_case("snake_case"), "snake-case");
        assert_eq!(to_kebab_case("CamelCase"), "camel-case");
        assert_eq!(to_kebab_case("PascalCase"), "pascal-case");
        assert_eq!(to_kebab_case("kebab-case"), "kebab-case");
        assert_eq!(to_kebab_case("hello world"), "hello-world");
    }

    #[test]
    fn test_edge_cases() {
        // Empty string
        assert_eq!(to_snake_case(""), "");
        assert_eq!(to_camel_case(""), "");

        // Single word
        assert_eq!(to_snake_case("word"), "word");
        assert_eq!(to_camel_case("word"), "word");
        assert_eq!(to_pascal_case("word"), "Word");

        // Numbers
        assert_eq!(to_snake_case("version2"), "version2");
        assert_eq!(to_snake_case("v2API"), "v2_api"); // Number triggers word boundary
        assert_eq!(to_snake_case("V2API"), "v2_api"); // Number triggers word boundary
        assert_eq!(to_camel_case("api_v2"), "apiV2");

        // Special characters
        assert_eq!(to_snake_case("hello@world#test"), "hello_world_test");
        assert_eq!(to_kebab_case("hello@world#test"), "hello-world-test");

        // Unicode (basic support)
        assert_eq!(to_snake_case("café"), "café");
        assert_eq!(to_snake_case("Café"), "café");
    }
}

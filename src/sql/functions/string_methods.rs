use anyhow::{anyhow, Result};

use super::{ArgCount, FunctionCategory, FunctionSignature, SqlFunction};
use crate::data::datatable::DataValue;

/// Trait for method-style functions that operate on a column/value
/// These are called with dot notation: column.Method(args)
pub trait MethodFunction: SqlFunction {
    /// Check if this method function handles the given method name
    fn handles_method(&self, method_name: &str) -> bool;

    /// Get the method name this function handles
    fn method_name(&self) -> &'static str;

    /// Evaluate as a method (first arg is implicit 'self')
    fn evaluate_method(&self, receiver: &DataValue, args: &[DataValue]) -> Result<DataValue> {
        // Default implementation: prepend receiver to args and call evaluate
        let mut full_args = vec![receiver.clone()];
        full_args.extend_from_slice(args);
        self.evaluate(&full_args)
    }
}

/// `ToUpper` method function
pub struct ToUpperMethod;

impl SqlFunction for ToUpperMethod {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "TOUPPER",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(1),
            description: "Converts string to uppercase",
            returns: "STRING",
            examples: vec![
                "SELECT name.ToUpper() FROM users",
                "SELECT TOUPPER(name) FROM users",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        match &args[0] {
            DataValue::String(s) => Ok(DataValue::String(s.to_uppercase())),
            DataValue::InternedString(s) => Ok(DataValue::String(s.to_uppercase())),
            DataValue::Null => Ok(DataValue::Null),
            _ => Err(anyhow!("ToUpper expects a string argument")),
        }
    }
}

impl MethodFunction for ToUpperMethod {
    fn handles_method(&self, method_name: &str) -> bool {
        method_name.eq_ignore_ascii_case("ToUpper")
            || method_name.eq_ignore_ascii_case("ToUpperCase")
    }

    fn method_name(&self) -> &'static str {
        "ToUpper"
    }
}

/// `ToLower` method function
pub struct ToLowerMethod;

impl SqlFunction for ToLowerMethod {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "TOLOWER",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(1),
            description: "Converts string to lowercase",
            returns: "STRING",
            examples: vec![
                "SELECT name.ToLower() FROM users",
                "SELECT TOLOWER(name) FROM users",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        match &args[0] {
            DataValue::String(s) => Ok(DataValue::String(s.to_lowercase())),
            DataValue::InternedString(s) => Ok(DataValue::String(s.to_lowercase())),
            DataValue::Null => Ok(DataValue::Null),
            _ => Err(anyhow!("ToLower expects a string argument")),
        }
    }
}

impl MethodFunction for ToLowerMethod {
    fn handles_method(&self, method_name: &str) -> bool {
        method_name.eq_ignore_ascii_case("ToLower")
            || method_name.eq_ignore_ascii_case("ToLowerCase")
    }

    fn method_name(&self) -> &'static str {
        "ToLower"
    }
}

/// Trim method function
pub struct TrimMethod;

impl SqlFunction for TrimMethod {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "TRIM",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(1),
            description: "Removes leading and trailing whitespace",
            returns: "STRING",
            examples: vec![
                "SELECT name.Trim() FROM users",
                "SELECT TRIM(name) FROM users",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        match &args[0] {
            DataValue::String(s) => Ok(DataValue::String(s.trim().to_string())),
            DataValue::InternedString(s) => Ok(DataValue::String(s.trim().to_string())),
            DataValue::Null => Ok(DataValue::Null),
            _ => Err(anyhow!("Trim expects a string argument")),
        }
    }
}

impl MethodFunction for TrimMethod {
    fn handles_method(&self, method_name: &str) -> bool {
        method_name.eq_ignore_ascii_case("Trim")
    }

    fn method_name(&self) -> &'static str {
        "Trim"
    }
}

/// TrimStart method function
pub struct TrimStartMethod;

impl SqlFunction for TrimStartMethod {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "TRIMSTART",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(1),
            description: "Removes leading whitespace",
            returns: "STRING",
            examples: vec![
                "SELECT name.TrimStart() FROM users",
                "SELECT TRIMSTART(name) FROM users",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        match &args[0] {
            DataValue::String(s) => Ok(DataValue::String(s.trim_start().to_string())),
            DataValue::InternedString(s) => Ok(DataValue::String(s.trim_start().to_string())),
            DataValue::Null => Ok(DataValue::Null),
            _ => Err(anyhow!("TrimStart expects a string argument")),
        }
    }
}

impl MethodFunction for TrimStartMethod {
    fn handles_method(&self, method_name: &str) -> bool {
        method_name.eq_ignore_ascii_case("TrimStart")
    }

    fn method_name(&self) -> &'static str {
        "TrimStart"
    }
}

/// TrimEnd method function
pub struct TrimEndMethod;

impl SqlFunction for TrimEndMethod {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "TRIMEND",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(1),
            description: "Removes trailing whitespace",
            returns: "STRING",
            examples: vec![
                "SELECT name.TrimEnd() FROM users",
                "SELECT TRIMEND(name) FROM users",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        match &args[0] {
            DataValue::String(s) => Ok(DataValue::String(s.trim_end().to_string())),
            DataValue::InternedString(s) => Ok(DataValue::String(s.trim_end().to_string())),
            DataValue::Null => Ok(DataValue::Null),
            _ => Err(anyhow!("TrimEnd expects a string argument")),
        }
    }
}

impl MethodFunction for TrimEndMethod {
    fn handles_method(&self, method_name: &str) -> bool {
        method_name.eq_ignore_ascii_case("TrimEnd")
    }

    fn method_name(&self) -> &'static str {
        "TrimEnd"
    }
}

/// Length method function (returns integer)
pub struct LengthMethod;

impl SqlFunction for LengthMethod {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "LENGTH",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(1),
            description: "Returns the length of a string",
            returns: "INTEGER",
            examples: vec![
                "SELECT name.Length() FROM users",
                "SELECT LENGTH(name) FROM users",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        match &args[0] {
            DataValue::String(s) => Ok(DataValue::Integer(s.len() as i64)),
            DataValue::InternedString(s) => Ok(DataValue::Integer(s.len() as i64)),
            DataValue::Null => Ok(DataValue::Null),
            _ => Err(anyhow!("Length expects a string argument")),
        }
    }
}

impl MethodFunction for LengthMethod {
    fn handles_method(&self, method_name: &str) -> bool {
        method_name.eq_ignore_ascii_case("Length") || method_name.eq_ignore_ascii_case("Len")
    }

    fn method_name(&self) -> &'static str {
        "Length"
    }
}

/// Contains method function (returns boolean)
pub struct ContainsMethod;

impl SqlFunction for ContainsMethod {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "CONTAINS",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(2),
            description: "Checks if string contains substring",
            returns: "BOOLEAN",
            examples: vec![
                "SELECT * FROM users WHERE name.Contains('john')",
                "SELECT CONTAINS(name, 'john') FROM users",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let haystack = match &args[0] {
            DataValue::String(s) => s.as_str(),
            DataValue::InternedString(s) => s.as_str(),
            DataValue::Null => return Ok(DataValue::Boolean(false)),
            _ => return Err(anyhow!("Contains expects string arguments")),
        };

        let needle = match &args[1] {
            DataValue::String(s) => s.as_str(),
            DataValue::InternedString(s) => s.as_str(),
            DataValue::Null => return Ok(DataValue::Boolean(false)),
            _ => return Err(anyhow!("Contains expects string arguments")),
        };

        Ok(DataValue::Boolean(haystack.contains(needle)))
    }
}

impl MethodFunction for ContainsMethod {
    fn handles_method(&self, method_name: &str) -> bool {
        method_name.eq_ignore_ascii_case("Contains")
    }

    fn method_name(&self) -> &'static str {
        "Contains"
    }
}

/// `StartsWith` method function
pub struct StartsWithMethod;

impl SqlFunction for StartsWithMethod {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "STARTSWITH",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(2),
            description: "Checks if string starts with prefix",
            returns: "BOOLEAN",
            examples: vec![
                "SELECT * FROM users WHERE name.StartsWith('John')",
                "SELECT STARTSWITH(name, 'John') FROM users",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let string = match &args[0] {
            DataValue::String(s) => s.as_str(),
            DataValue::InternedString(s) => s.as_str(),
            DataValue::Null => return Ok(DataValue::Boolean(false)),
            _ => return Err(anyhow!("StartsWith expects string arguments")),
        };

        let prefix = match &args[1] {
            DataValue::String(s) => s.as_str(),
            DataValue::InternedString(s) => s.as_str(),
            DataValue::Null => return Ok(DataValue::Boolean(false)),
            _ => return Err(anyhow!("StartsWith expects string arguments")),
        };

        Ok(DataValue::Boolean(string.starts_with(prefix)))
    }
}

impl MethodFunction for StartsWithMethod {
    fn handles_method(&self, method_name: &str) -> bool {
        method_name.eq_ignore_ascii_case("StartsWith")
    }

    fn method_name(&self) -> &'static str {
        "StartsWith"
    }
}

/// `EndsWith` method function
pub struct EndsWithMethod;

impl SqlFunction for EndsWithMethod {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "ENDSWITH",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(2),
            description: "Checks if string ends with suffix",
            returns: "BOOLEAN",
            examples: vec![
                "SELECT * FROM users WHERE email.EndsWith('.com')",
                "SELECT ENDSWITH(email, '.com') FROM users",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let string = match &args[0] {
            DataValue::String(s) => s.as_str(),
            DataValue::InternedString(s) => s.as_str(),
            DataValue::Null => return Ok(DataValue::Boolean(false)),
            _ => return Err(anyhow!("EndsWith expects string arguments")),
        };

        let suffix = match &args[1] {
            DataValue::String(s) => s.as_str(),
            DataValue::InternedString(s) => s.as_str(),
            DataValue::Null => return Ok(DataValue::Boolean(false)),
            _ => return Err(anyhow!("EndsWith expects string arguments")),
        };

        Ok(DataValue::Boolean(string.ends_with(suffix)))
    }
}

impl MethodFunction for EndsWithMethod {
    fn handles_method(&self, method_name: &str) -> bool {
        method_name.eq_ignore_ascii_case("EndsWith")
    }

    fn method_name(&self) -> &'static str {
        "EndsWith"
    }
}

/// Substring method function
pub struct SubstringMethod;

impl SqlFunction for SubstringMethod {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "SUBSTRING",
            category: FunctionCategory::String,
            arg_count: ArgCount::Range(2, 3),
            description: "Extracts substring from string",
            returns: "STRING",
            examples: vec![
                "SELECT name.Substring(0, 5) FROM users",
                "SELECT SUBSTRING(name, 0, 5) FROM users",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() < 2 || args.len() > 3 {
            return Err(anyhow!("Substring expects 2 or 3 arguments"));
        }

        let string = match &args[0] {
            DataValue::String(s) => s.as_str(),
            DataValue::InternedString(s) => s.as_str(),
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("Substring expects a string as first argument")),
        };

        let start = match &args[1] {
            DataValue::Integer(i) => *i as usize,
            _ => return Err(anyhow!("Substring expects integer start position")),
        };

        let result = if args.len() == 3 {
            let length = match &args[2] {
                DataValue::Integer(i) => *i as usize,
                _ => return Err(anyhow!("Substring expects integer length")),
            };

            let end = (start + length).min(string.len());
            string.chars().skip(start).take(end - start).collect()
        } else {
            string.chars().skip(start).collect()
        };

        Ok(DataValue::String(result))
    }
}

impl MethodFunction for SubstringMethod {
    fn handles_method(&self, method_name: &str) -> bool {
        method_name.eq_ignore_ascii_case("Substring") || method_name.eq_ignore_ascii_case("Substr")
    }

    fn method_name(&self) -> &'static str {
        "Substring"
    }
}

/// Replace method function
pub struct ReplaceMethod;

impl SqlFunction for ReplaceMethod {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "REPLACE",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(3),
            description: "Replaces all occurrences of a substring",
            returns: "STRING",
            examples: vec![
                "SELECT name.Replace('John', 'Jane') FROM users",
                "SELECT REPLACE(name, 'John', 'Jane') FROM users",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let string = match &args[0] {
            DataValue::String(s) => s.as_str(),
            DataValue::InternedString(s) => s.as_str(),
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("Replace expects string arguments")),
        };

        let from = match &args[1] {
            DataValue::String(s) => s.as_str(),
            DataValue::InternedString(s) => s.as_str(),
            _ => return Err(anyhow!("Replace expects string arguments")),
        };

        let to = match &args[2] {
            DataValue::String(s) => s.as_str(),
            DataValue::InternedString(s) => s.as_str(),
            _ => return Err(anyhow!("Replace expects string arguments")),
        };

        Ok(DataValue::String(string.replace(from, to)))
    }
}

impl MethodFunction for ReplaceMethod {
    fn handles_method(&self, method_name: &str) -> bool {
        method_name.eq_ignore_ascii_case("Replace")
    }

    fn method_name(&self) -> &'static str {
        "Replace"
    }
}

/// MID function - Extract substring (SQL/Excel compatible, 1-based indexing)
pub struct MidFunction;

impl SqlFunction for MidFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "MID",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(3),
            description: "Extract substring from text (1-based indexing)",
            returns: "STRING",
            examples: vec![
                "SELECT MID('Hello', 1, 3)", // Returns 'Hel'
                "SELECT MID('World', 2, 3)", // Returns 'orl'
                "SELECT MID(name, 1, 5) FROM table",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        // Get the string
        let text = match &args[0] {
            DataValue::String(s) => s.clone(),
            DataValue::InternedString(s) => s.to_string(),
            DataValue::Integer(n) => n.to_string(),
            DataValue::Float(f) => f.to_string(),
            DataValue::Null => String::new(),
            _ => return Err(anyhow!("MID first argument must be convertible to text")),
        };

        // Get start position (1-based)
        let start_pos = match &args[1] {
            DataValue::Integer(n) => *n,
            DataValue::Float(f) => *f as i64,
            _ => return Err(anyhow!("MID start position must be a number")),
        };

        // Get length
        let length = match &args[2] {
            DataValue::Integer(n) => *n,
            DataValue::Float(f) => *f as i64,
            _ => return Err(anyhow!("MID length must be a number")),
        };

        // Validate arguments
        if start_pos < 1 {
            return Err(anyhow!("MID start position must be >= 1"));
        }
        if length < 0 {
            return Err(anyhow!("MID length must be >= 0"));
        }

        // Convert to 0-based index
        let start_idx = (start_pos - 1) as usize;
        let chars: Vec<char> = text.chars().collect();

        // If start position is beyond string length, return empty string
        if start_idx >= chars.len() {
            return Ok(DataValue::String(String::new()));
        }

        // Extract substring
        let end_idx = std::cmp::min(start_idx + length as usize, chars.len());
        let result: String = chars[start_idx..end_idx].iter().collect();

        Ok(DataValue::String(result))
    }
}

/// UPPER function - Convert string to uppercase
pub struct UpperFunction;

impl SqlFunction for UpperFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "UPPER",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(1),
            description: "Convert string to uppercase",
            returns: "STRING",
            examples: vec![
                "SELECT UPPER('hello')", // Returns 'HELLO'
                "SELECT UPPER(name) FROM table",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        match &args[0] {
            DataValue::String(s) => Ok(DataValue::String(s.to_uppercase())),
            DataValue::InternedString(s) => Ok(DataValue::String(s.to_uppercase())),
            DataValue::Null => Ok(DataValue::Null),
            _ => Err(anyhow!("UPPER expects a string argument")),
        }
    }
}

/// LOWER function - Convert string to lowercase
pub struct LowerFunction;

impl SqlFunction for LowerFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "LOWER",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(1),
            description: "Convert string to lowercase",
            returns: "STRING",
            examples: vec![
                "SELECT LOWER('HELLO')", // Returns 'hello'
                "SELECT LOWER(name) FROM table",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        match &args[0] {
            DataValue::String(s) => Ok(DataValue::String(s.to_lowercase())),
            DataValue::InternedString(s) => Ok(DataValue::String(s.to_lowercase())),
            DataValue::Null => Ok(DataValue::Null),
            _ => Err(anyhow!("LOWER expects a string argument")),
        }
    }
}

/// TRIM function - Remove leading and trailing whitespace
pub struct TrimFunction;

impl SqlFunction for TrimFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "TRIM",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(1),
            description: "Remove leading and trailing whitespace",
            returns: "STRING",
            examples: vec![
                "SELECT TRIM('  hello  ')", // Returns 'hello'
                "SELECT TRIM(description) FROM table",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        match &args[0] {
            DataValue::String(s) => Ok(DataValue::String(s.trim().to_string())),
            DataValue::InternedString(s) => Ok(DataValue::String(s.trim().to_string())),
            DataValue::Null => Ok(DataValue::Null),
            _ => Err(anyhow!("TRIM expects a string argument")),
        }
    }
}

/// TEXTJOIN function - Join multiple text values with a delimiter
pub struct TextJoinFunction;

impl SqlFunction for TextJoinFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "TEXTJOIN",
            category: FunctionCategory::String,
            arg_count: ArgCount::Variadic,
            description: "Join multiple text values with a delimiter",
            returns: "STRING",
            examples: vec![
                "SELECT TEXTJOIN(',', 1, 'a', 'b', 'c')", // Returns 'a,b,c'
                "SELECT TEXTJOIN(' - ', 1, name, city) FROM table",
                "SELECT TEXTJOIN('|', 0, col1, col2, col3) FROM table",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() < 3 {
            return Err(anyhow!("TEXTJOIN requires at least 3 arguments: delimiter, ignore_empty, text1, [text2, ...]"));
        }

        // First argument: delimiter
        let delimiter = match &args[0] {
            DataValue::String(s) => s.clone(),
            DataValue::InternedString(s) => s.to_string(),
            DataValue::Integer(n) => n.to_string(),
            DataValue::Float(f) => f.to_string(),
            DataValue::Boolean(b) => b.to_string(),
            DataValue::Null => String::new(),
            _ => String::new(),
        };

        // Second argument: ignore_empty (treat as boolean - 0 is false, anything else is true)
        let ignore_empty = match &args[1] {
            DataValue::Integer(n) => *n != 0,
            DataValue::Float(f) => *f != 0.0,
            DataValue::Boolean(b) => *b,
            DataValue::String(s) => !s.is_empty() && s != "0" && s.to_lowercase() != "false",
            DataValue::InternedString(s) => {
                !s.is_empty() && s.as_str() != "0" && s.to_lowercase() != "false"
            }
            DataValue::Null => false,
            _ => true,
        };

        // Remaining arguments: values to join
        let mut values = Vec::new();
        for i in 2..args.len() {
            let string_value = match &args[i] {
                DataValue::String(s) => Some(s.clone()),
                DataValue::InternedString(s) => Some(s.to_string()),
                DataValue::Integer(n) => Some(n.to_string()),
                DataValue::Float(f) => Some(f.to_string()),
                DataValue::Boolean(b) => Some(b.to_string()),
                DataValue::DateTime(dt) => Some(dt.clone()),
                DataValue::Null => {
                    if ignore_empty {
                        None
                    } else {
                        Some(String::new())
                    }
                }
            };

            if let Some(s) = string_value {
                if !ignore_empty || !s.is_empty() {
                    values.push(s);
                }
            }
        }

        Ok(DataValue::String(values.join(&delimiter)))
    }
}

/// Edit distance (Levenshtein distance) function
pub struct EditDistanceFunction;

impl EditDistanceFunction {
    /// Calculate the Levenshtein distance between two strings
    #[must_use]
    pub fn calculate_edit_distance(s1: &str, s2: &str) -> usize {
        let len1 = s1.len();
        let len2 = s2.len();
        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }

        for (i, c1) in s1.chars().enumerate() {
            for (j, c2) in s2.chars().enumerate() {
                let cost = usize::from(c1 != c2);
                matrix[i + 1][j + 1] = std::cmp::min(
                    matrix[i][j + 1] + 1, // deletion
                    std::cmp::min(
                        matrix[i + 1][j] + 1, // insertion
                        matrix[i][j] + cost,  // substitution
                    ),
                );
            }
        }

        matrix[len1][len2]
    }
}

impl SqlFunction for EditDistanceFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "EDIT_DISTANCE",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(2),
            description: "Calculate the Levenshtein edit distance between two strings",
            returns: "INTEGER",
            examples: vec![
                "SELECT EDIT_DISTANCE('kitten', 'sitting')",
                "SELECT EDIT_DISTANCE(name, 'John') FROM users",
                "SELECT * FROM users WHERE EDIT_DISTANCE(name, 'Smith') <= 2",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let s1 = match &args[0] {
            DataValue::String(s) => s.clone(),
            DataValue::InternedString(s) => s.to_string(),
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("EDIT_DISTANCE expects string arguments")),
        };

        let s2 = match &args[1] {
            DataValue::String(s) => s.clone(),
            DataValue::InternedString(s) => s.to_string(),
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("EDIT_DISTANCE expects string arguments")),
        };

        let distance = Self::calculate_edit_distance(&s1, &s2);
        Ok(DataValue::Integer(distance as i64))
    }
}

/// FREQUENCY function - Count occurrences of a substring in a string
pub struct FrequencyFunction;

impl SqlFunction for FrequencyFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "FREQUENCY",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(2),
            description: "Count occurrences of a substring within a string",
            returns: "INTEGER",
            examples: vec![
                "SELECT FREQUENCY('hello world', 'o')",  // Returns 2
                "SELECT FREQUENCY('mississippi', 'ss')", // Returns 2
                "SELECT FREQUENCY(text_column, 'error') FROM logs",
                "SELECT name, FREQUENCY(name, 'a') as a_count FROM users",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        // Get the string to search in
        let text = match &args[0] {
            DataValue::String(s) => s.clone(),
            DataValue::InternedString(s) => s.to_string(),
            DataValue::Null => return Ok(DataValue::Integer(0)),
            _ => return Err(anyhow!("FREQUENCY expects string as first argument")),
        };

        // Get the substring to search for
        let search = match &args[1] {
            DataValue::String(s) => s.clone(),
            DataValue::InternedString(s) => s.to_string(),
            DataValue::Null => return Ok(DataValue::Integer(0)),
            _ => return Err(anyhow!("FREQUENCY expects string as second argument")),
        };

        // Empty search string returns 0
        if search.is_empty() {
            return Ok(DataValue::Integer(0));
        }

        // Count occurrences
        let count = text.matches(&search).count();
        Ok(DataValue::Integer(count as i64))
    }
}

/// IndexOf method function - finds the position of a substring
pub struct IndexOfMethod;

impl SqlFunction for IndexOfMethod {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "INDEXOF",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(2),
            description: "Returns the position of the first occurrence of a substring (0-based)",
            returns: "INTEGER",
            examples: vec![
                "SELECT email.IndexOf('@') FROM users",
                "SELECT INDEXOF(email, '@') FROM users",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let string = match &args[0] {
            DataValue::String(s) => s.as_str(),
            DataValue::InternedString(s) => s.as_str(),
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("IndexOf expects string arguments")),
        };

        let substring = match &args[1] {
            DataValue::String(s) => s.as_str(),
            DataValue::InternedString(s) => s.as_str(),
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("IndexOf expects string arguments")),
        };

        match string.find(substring) {
            Some(pos) => Ok(DataValue::Integer(pos as i64)),
            None => Ok(DataValue::Integer(-1)), // Return -1 if not found
        }
    }
}

impl MethodFunction for IndexOfMethod {
    fn handles_method(&self, method_name: &str) -> bool {
        method_name.eq_ignore_ascii_case("IndexOf")
    }

    fn method_name(&self) -> &'static str {
        "IndexOf"
    }
}

/// INSTR function - SQL standard function for finding substring position
/// Returns 1-based position for SQL compatibility
pub struct InstrFunction;

impl SqlFunction for InstrFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "INSTR",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(2),
            description: "Returns the position of the first occurrence of a substring (1-based, SQL standard)",
            returns: "INTEGER",
            examples: vec![
                "SELECT INSTR(email, '@') FROM users",
                "SELECT SUBSTRING(email, INSTR(email, '@') + 1) FROM users",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let string = match &args[0] {
            DataValue::String(s) => s.as_str(),
            DataValue::InternedString(s) => s.as_str(),
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("INSTR expects string arguments")),
        };

        let substring = match &args[1] {
            DataValue::String(s) => s.as_str(),
            DataValue::InternedString(s) => s.as_str(),
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("INSTR expects string arguments")),
        };

        match string.find(substring) {
            Some(pos) => Ok(DataValue::Integer((pos + 1) as i64)), // 1-based for SQL
            None => Ok(DataValue::Integer(0)), // Return 0 if not found (SQL standard)
        }
    }
}

/// LEFT function - extracts leftmost n characters or up to a delimiter
pub struct LeftFunction;

impl SqlFunction for LeftFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "LEFT",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(2),
            description: "Returns leftmost n characters from string",
            returns: "STRING",
            examples: vec![
                "SELECT LEFT(email, 5) FROM users",
                "SELECT LEFT('hello@world', INSTR('hello@world', '@') - 1)",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let string = match &args[0] {
            DataValue::String(s) => s.as_str(),
            DataValue::InternedString(s) => s.as_str(),
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("LEFT expects a string as first argument")),
        };

        let length = match &args[1] {
            DataValue::Integer(n) => *n as usize,
            DataValue::Float(f) => *f as usize,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("LEFT expects a number as second argument")),
        };

        let result = if length >= string.len() {
            string.to_string()
        } else {
            string.chars().take(length).collect()
        };

        Ok(DataValue::String(result))
    }
}

/// RIGHT function - extracts rightmost n characters
pub struct RightFunction;

impl SqlFunction for RightFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "RIGHT",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(2),
            description: "Returns rightmost n characters from string",
            returns: "STRING",
            examples: vec![
                "SELECT RIGHT(filename, 4) FROM files", // Get file extension
                "SELECT RIGHT(email, LENGTH(email) - INSTR(email, '@'))", // Get domain
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let string = match &args[0] {
            DataValue::String(s) => s.as_str(),
            DataValue::InternedString(s) => s.as_str(),
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("RIGHT expects a string as first argument")),
        };

        let length = match &args[1] {
            DataValue::Integer(n) => *n as usize,
            DataValue::Float(f) => *f as usize,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("RIGHT expects a number as second argument")),
        };

        let chars: Vec<char> = string.chars().collect();
        let start = if length >= chars.len() {
            0
        } else {
            chars.len() - length
        };

        let result: String = chars[start..].iter().collect();
        Ok(DataValue::String(result))
    }
}

/// SUBSTRING_BEFORE - returns substring before first/nth occurrence of delimiter
pub struct SubstringBeforeFunction;

impl SqlFunction for SubstringBeforeFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "SUBSTRING_BEFORE",
            category: FunctionCategory::String,
            arg_count: ArgCount::Range(2, 3),
            description: "Returns substring before the first (or nth) occurrence of delimiter",
            returns: "STRING",
            examples: vec![
                "SELECT SUBSTRING_BEFORE(email, '@') FROM users", // Get username
                "SELECT SUBSTRING_BEFORE('a.b.c.d', '.', 2)",     // Get 'a.b' (before 2nd dot)
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() < 2 || args.len() > 3 {
            return Err(anyhow!("SUBSTRING_BEFORE expects 2 or 3 arguments"));
        }

        let string = match &args[0] {
            DataValue::String(s) => s.as_str(),
            DataValue::InternedString(s) => s.as_str(),
            DataValue::Null => return Ok(DataValue::Null),
            _ => {
                return Err(anyhow!(
                    "SUBSTRING_BEFORE expects a string as first argument"
                ))
            }
        };

        let delimiter = match &args[1] {
            DataValue::String(s) => s.as_str(),
            DataValue::InternedString(s) => s.as_str(),
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("SUBSTRING_BEFORE expects a string delimiter")),
        };

        let occurrence = if args.len() == 3 {
            match &args[2] {
                DataValue::Integer(n) => *n as usize,
                DataValue::Float(f) => *f as usize,
                DataValue::Null => 1,
                _ => return Err(anyhow!("SUBSTRING_BEFORE expects a number for occurrence")),
            }
        } else {
            1
        };

        if occurrence == 0 {
            return Ok(DataValue::String(String::new()));
        }

        // Find the nth occurrence
        let mut count = 0;
        let mut last_pos = 0;
        for (i, _) in string.match_indices(delimiter) {
            count += 1;
            if count == occurrence {
                return Ok(DataValue::String(string[..i].to_string()));
            }
            last_pos = i;
        }

        // If we didn't find enough occurrences, return empty string
        Ok(DataValue::String(String::new()))
    }
}

/// SUBSTRING_AFTER - returns substring after first/nth occurrence of delimiter
pub struct SubstringAfterFunction;

impl SqlFunction for SubstringAfterFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "SUBSTRING_AFTER",
            category: FunctionCategory::String,
            arg_count: ArgCount::Range(2, 3),
            description: "Returns substring after the first (or nth) occurrence of delimiter",
            returns: "STRING",
            examples: vec![
                "SELECT SUBSTRING_AFTER(email, '@') FROM users", // Get domain
                "SELECT SUBSTRING_AFTER('a.b.c.d', '.', 2)",     // Get 'c.d' (after 2nd dot)
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() < 2 || args.len() > 3 {
            return Err(anyhow!("SUBSTRING_AFTER expects 2 or 3 arguments"));
        }

        let string = match &args[0] {
            DataValue::String(s) => s.as_str(),
            DataValue::InternedString(s) => s.as_str(),
            DataValue::Null => return Ok(DataValue::Null),
            _ => {
                return Err(anyhow!(
                    "SUBSTRING_AFTER expects a string as first argument"
                ))
            }
        };

        let delimiter = match &args[1] {
            DataValue::String(s) => s.as_str(),
            DataValue::InternedString(s) => s.as_str(),
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("SUBSTRING_AFTER expects a string delimiter")),
        };

        let occurrence = if args.len() == 3 {
            match &args[2] {
                DataValue::Integer(n) => *n as usize,
                DataValue::Float(f) => *f as usize,
                DataValue::Null => 1,
                _ => return Err(anyhow!("SUBSTRING_AFTER expects a number for occurrence")),
            }
        } else {
            1
        };

        if occurrence == 0 {
            return Ok(DataValue::String(string.to_string()));
        }

        // Find the nth occurrence
        let mut count = 0;
        for (i, _) in string.match_indices(delimiter) {
            count += 1;
            if count == occurrence {
                let start = i + delimiter.len();
                if start < string.len() {
                    return Ok(DataValue::String(string[start..].to_string()));
                } else {
                    return Ok(DataValue::String(String::new()));
                }
            }
        }

        // If we didn't find enough occurrences, return empty string
        Ok(DataValue::String(String::new()))
    }
}

/// SPLIT_PART - returns the nth part of a string split by delimiter (1-based)
pub struct SplitPartFunction;

impl SqlFunction for SplitPartFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "SPLIT_PART",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(3),
            description: "Returns the nth part of a string split by delimiter (1-based index)",
            returns: "STRING",
            examples: vec![
                "SELECT SPLIT_PART('a.b.c.d', '.', 2)",        // Returns 'b'
                "SELECT SPLIT_PART(email, '@', 1) FROM users", // Get username
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let string = match &args[0] {
            DataValue::String(s) => s.as_str(),
            DataValue::InternedString(s) => s.as_str(),
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("SPLIT_PART expects a string as first argument")),
        };

        let delimiter = match &args[1] {
            DataValue::String(s) => s.as_str(),
            DataValue::InternedString(s) => s.as_str(),
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("SPLIT_PART expects a string delimiter")),
        };

        let part_num = match &args[2] {
            DataValue::Integer(n) => *n as usize,
            DataValue::Float(f) => *f as usize,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("SPLIT_PART expects a number for part index")),
        };

        if part_num == 0 {
            return Err(anyhow!("SPLIT_PART part index must be >= 1"));
        }

        let parts: Vec<&str> = string.split(delimiter).collect();

        if part_num <= parts.len() {
            Ok(DataValue::String(parts[part_num - 1].to_string()))
        } else {
            Ok(DataValue::String(String::new()))
        }
    }
}

/// CHR function - Convert ASCII code to character
pub struct ChrFunction;

impl SqlFunction for ChrFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "CHR",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(1),
            description: "Convert ASCII code to character",
            returns: "STRING",
            examples: vec![
                "SELECT CHR(65)", // Returns 'A'
                "SELECT CHR(97)", // Returns 'a'
                "SELECT CHR(48)", // Returns '0'
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Err(anyhow!("CHR expects exactly 1 argument"));
        }

        let ascii_code = match &args[0] {
            DataValue::Integer(n) => *n,
            DataValue::Float(f) => *f as i64,
            DataValue::String(s) => s
                .parse::<i64>()
                .map_err(|_| anyhow!("Invalid number for CHR: {}", s))?,
            DataValue::InternedString(s) => s
                .parse::<i64>()
                .map_err(|_| anyhow!("Invalid number for CHR: {}", s))?,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("CHR expects a numeric argument")),
        };

        // ASCII printable range is 32-126, but we'll allow 0-255
        if ascii_code < 0 || ascii_code > 255 {
            return Err(anyhow!(
                "CHR argument must be between 0 and 255, got {}",
                ascii_code
            ));
        }

        let ch = ascii_code as u8 as char;
        Ok(DataValue::String(ch.to_string()))
    }
}

/// LOREM_IPSUM function - generates Lorem Ipsum placeholder text
pub struct LoremIpsumFunction;

impl SqlFunction for LoremIpsumFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "LOREM_IPSUM",
            category: FunctionCategory::String,
            arg_count: ArgCount::Range(1, 3),
            description: "Generate Lorem Ipsum placeholder text with specified number of words",
            returns: "STRING",
            examples: vec![
                "SELECT LOREM_IPSUM(10)",              // 10 random Lorem Ipsum words
                "SELECT LOREM_IPSUM(50)",              // 50 words
                "SELECT LOREM_IPSUM(20, 1)",           // 20 words, starting with 'Lorem ipsum...'
                "SELECT LOREM_IPSUM(15, 0, id)",       // 15 words, use id as seed for variation
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let num_words = match &args[0] {
            DataValue::Integer(n) if *n > 0 => *n as usize,
            DataValue::Float(f) if *f > 0.0 => *f as usize,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("LOREM_IPSUM requires a positive number of words")),
        };

        // Check if we should start with traditional "Lorem ipsum..." opening
        let start_traditional = if args.len() > 1 {
            match &args[1] {
                DataValue::Integer(n) => *n != 0,
                DataValue::Boolean(b) => *b,
                _ => false,
            }
        } else {
            false
        };

        // Get optional seed for reproducible but varied results
        let seed_value = if args.len() > 2 {
            match &args[2] {
                DataValue::Integer(n) => *n as u64,
                DataValue::Float(f) => *f as u64,
                DataValue::String(s) => {
                    // Hash the string to get a numeric seed
                    let mut hash = 0u64;
                    for byte in s.bytes() {
                        hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
                    }
                    hash
                }
                DataValue::Null => 0,
                _ => 0,
            }
        } else {
            0
        };

        // Lorem Ipsum word bank - traditional Latin placeholder text words
        const LOREM_WORDS: &[&str] = &[
            "lorem", "ipsum", "dolor", "sit", "amet", "consectetur", "adipiscing", "elit",
            "sed", "do", "eiusmod", "tempor", "incididunt", "ut", "labore", "et", "dolore",
            "magna", "aliqua", "enim", "ad", "minim", "veniam", "quis", "nostrud",
            "exercitation", "ullamco", "laboris", "nisi", "aliquip", "ex", "ea", "commodo",
            "consequat", "duis", "aute", "irure", "in", "reprehenderit", "voluptate",
            "velit", "esse", "cillum", "fugiat", "nulla", "pariatur", "excepteur", "sint",
            "occaecat", "cupidatat", "non", "proident", "sunt", "culpa", "qui", "officia",
            "deserunt", "mollit", "anim", "id", "est", "laborum", "perspiciatis", "unde",
            "omnis", "iste", "natus", "error", "voluptatem", "accusantium", "doloremque",
            "laudantium", "totam", "rem", "aperiam", "eaque", "ipsa", "quae", "ab", "illo",
            "inventore", "veritatis", "quasi", "architecto", "beatae", "vitae", "dicta",
            "explicabo", "nemo", "enim", "ipsam", "quia", "voluptas", "aspernatur", "aut",
            "odit", "fugit", "consequuntur", "magni", "dolores", "eos", "ratione", "sequi",
            "nesciunt", "neque", "porro", "quisquam", "dolorem", "adipisci", "numquam",
            "eius", "modi", "tempora", "incidunt", "magnam", "quaerat", "etiam", "minus",
            "soluta", "nobis", "eligendi", "optio", "cumque", "nihil", "impedit", "quo",
            "possimus", "suscipit", "laboriosam", "aliquid", "fuga", "distinctio", "libero",
            "tempore", "cum", "assumenda", "est", "omnis", "dolor", "repellendus",
            "temporibus", "autem", "quibusdam", "officiis", "debitis", "rerum",
            "necessitatibus", "saepe", "eveniet", "voluptates", "repudiandae", "molestiae",
            "recusandae", "itaque", "earum", "hic", "tenetur", "sapiente", "delectus",
            "reiciendis", "voluptatibus", "maiores", "alias", "consequatur", "perferendis",
            "doloribus", "asperiores", "repellat", "iusto", "odio", "dignissimos", "ducimus",
            "blanditiis", "praesentium", "voluptatum", "deleniti", "atque", "corrupti",
            "quos", "quas", "molestias", "excepturi", "occaecati", "provident", "similique",
            "mollitia", "animi", "illum", "dolorum", "fuga", "harum", "quidem", "rerum",
            "facilis", "expedita", "distinctio", "nam", "libero", "tempore", "cum", "soluta",
            "nobis", "eligendi", "optio", "cumque", "nihil", "impedit", "minus", "quod",
            "maxime", "placeat", "facere", "possimus", "omnis", "voluptas", "assumenda",
        ];

        let mut result = Vec::with_capacity(num_words);

        if start_traditional && num_words > 0 {
            // Start with traditional "Lorem ipsum dolor sit amet..."
            let traditional_start = ["lorem", "ipsum", "dolor", "sit", "amet"];
            let take_count = num_words.min(traditional_start.len());
            for i in 0..take_count {
                result.push(traditional_start[i]);
            }

            // Fill remaining with random words
            let seed = if seed_value != 0 {
                seed_value
            } else {
                use std::time::{SystemTime, UNIX_EPOCH};
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos() as u64
            };

            let mut rng = seed.wrapping_mul(num_words as u64); // Combine seed with word count
            for i in take_count..num_words {
                // Simple pseudo-random selection
                rng = (rng.wrapping_mul(1664525).wrapping_add(1013904223)) ^ (i as u64);
                let idx = (rng as usize) % LOREM_WORDS.len();
                result.push(LOREM_WORDS[idx]);
            }
        } else {
            // Generate random Lorem words
            let seed = if seed_value != 0 {
                seed_value
            } else {
                use std::time::{SystemTime, UNIX_EPOCH};
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos() as u64
            };

            let mut rng = seed.wrapping_mul(num_words as u64).wrapping_add(12345); // Combine seed with word count
            for i in 0..num_words {
                // Simple pseudo-random selection with better entropy
                rng = (rng.wrapping_mul(1664525).wrapping_add(1013904223)) ^ (i as u64);
                let idx = (rng as usize) % LOREM_WORDS.len();
                result.push(LOREM_WORDS[idx]);
            }
        }

        // Capitalize first word and add periods for readability
        let mut text = String::new();
        for (i, word) in result.iter().enumerate() {
            if i == 0 {
                // Capitalize first word
                text.push_str(&word.chars().next().unwrap().to_uppercase().to_string());
                text.push_str(&word[1..]);
            } else {
                text.push(' ');
                // Occasionally start a new sentence (roughly every 10-15 words)
                if i > 0 && ((i * 7) % 13 == 0) && i < num_words - 1 {
                    text.pop(); // Remove the space
                    text.push_str(". ");
                    // Capitalize next word
                    text.push_str(&word.chars().next().unwrap().to_uppercase().to_string());
                    text.push_str(&word[1..]);
                } else {
                    text.push_str(word);
                }
            }
        }

        // Add final period if we generated text
        if !text.is_empty() {
            text.push('.');
        }

        Ok(DataValue::String(text))
    }
}

/// Register all string method functions
pub fn register_string_methods(registry: &mut super::FunctionRegistry) {
    use std::sync::Arc;

    // Register new string functions (non-method versions)
    registry.register(Box::new(MidFunction));
    registry.register(Box::new(UpperFunction));
    registry.register(Box::new(LowerFunction));
    registry.register(Box::new(TrimFunction));
    registry.register(Box::new(TextJoinFunction));
    registry.register(Box::new(EditDistanceFunction));
    registry.register(Box::new(FrequencyFunction));

    // Register new convenient string extraction functions
    registry.register(Box::new(LeftFunction));
    registry.register(Box::new(RightFunction));
    registry.register(Box::new(SubstringBeforeFunction));
    registry.register(Box::new(SubstringAfterFunction));
    registry.register(Box::new(SplitPartFunction));

    // Register ToUpper
    let to_upper = Arc::new(ToUpperMethod);
    registry.register(Box::new(ToUpperMethod));
    registry.register_method(to_upper);

    // Register ToLower
    let to_lower = Arc::new(ToLowerMethod);
    registry.register(Box::new(ToLowerMethod));
    registry.register_method(to_lower);

    // Register Trim
    let trim = Arc::new(TrimMethod);
    registry.register(Box::new(TrimMethod));
    registry.register_method(trim);

    // Register TrimStart
    let trim_start = Arc::new(TrimStartMethod);
    registry.register(Box::new(TrimStartMethod));
    registry.register_method(trim_start);

    // Register TrimEnd
    let trim_end = Arc::new(TrimEndMethod);
    registry.register(Box::new(TrimEndMethod));
    registry.register_method(trim_end);

    // Register Length
    let length = Arc::new(LengthMethod);
    registry.register(Box::new(LengthMethod));
    registry.register_method(length);

    // Register Contains
    let contains = Arc::new(ContainsMethod);
    registry.register(Box::new(ContainsMethod));
    registry.register_method(contains);

    // Register StartsWith
    let starts_with = Arc::new(StartsWithMethod);
    registry.register(Box::new(StartsWithMethod));
    registry.register_method(starts_with);

    // Register EndsWith
    let ends_with = Arc::new(EndsWithMethod);
    registry.register(Box::new(EndsWithMethod));
    registry.register_method(ends_with);

    // Register Substring
    let substring = Arc::new(SubstringMethod);
    registry.register(Box::new(SubstringMethod));
    registry.register_method(substring);

    // Register Replace
    let replace = Arc::new(ReplaceMethod);
    registry.register(Box::new(ReplaceMethod));
    registry.register_method(replace);

    // Register IndexOf/INSTR
    let indexof = Arc::new(IndexOfMethod);
    registry.register(Box::new(IndexOfMethod));
    registry.register_method(indexof.clone());
    // Also register as INSTR for SQL compatibility
    registry.register(Box::new(InstrFunction));

    // Register CHR function
    registry.register(Box::new(ChrFunction));

    // Register LOREM_IPSUM function
    registry.register(Box::new(LoremIpsumFunction));
}

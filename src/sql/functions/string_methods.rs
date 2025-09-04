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

/// ToUpper method function
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

/// ToLower method function
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

/// StartsWith method function
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

/// EndsWith method function
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

/// Register all string method functions
pub fn register_string_methods(registry: &mut super::FunctionRegistry) {
    use std::sync::Arc;

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
}

//! Scalar lookups on the running process (S15, docs/SYSTEM_TABLES.md).
//!
//! The table generators in `src/sql/generators/system.rs` answer *list them
//! all*; these answer *just this one*, so it can sit inside an expression.

use crate::data::datatable::DataValue;
use crate::sql::functions::{ArgCount, FunctionCategory, FunctionSignature, SqlFunction};
use anyhow::{anyhow, Result};

/// ENV(name) - one environment variable of this process, or NULL if unset
pub struct EnvFunction;

impl EnvFunction {
    /// A name the OS can never hold. `var_os` makes no promise about these
    /// (older toolchains panicked on some), and the honest answer is the same
    /// as for any other variable that is not set.
    fn is_impossible(name: &str) -> bool {
        name.is_empty() || name.contains('=') || name.contains('\0')
    }
}

impl SqlFunction for EnvFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "ENV",
            category: FunctionCategory::System,
            arg_count: ArgCount::Fixed(1),
            description: "Value of one environment variable of this process, NULL if unset (case-insensitive name on Windows)",
            returns: "String, or NULL when the variable is not set",
            examples: vec![
                "SELECT ENV('PATH')",
                "SELECT COALESCE(ENV('EDITOR'), 'vi') AS editor",
                "SELECT UNNEST(ENV('PATH'), ';') AS dir  -- ':' on Unix",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let name = match &args[0] {
            DataValue::Null => return Ok(DataValue::Null),
            DataValue::String(s) => s.as_str(),
            DataValue::InternedString(s) => s.as_str(),
            other => {
                return Err(anyhow!(
                    "ENV expects a variable name as a string, got {other:?}"
                ))
            }
        };

        if Self::is_impossible(name) {
            return Ok(DataValue::Null);
        }

        // Unset is NULL, set-but-empty is '' - the same distinction
        // environment() keeps. Lossy for the same reason as there: a value
        // that is not valid UTF-8 is still a value.
        Ok(match std::env::var_os(name) {
            Some(value) => DataValue::String(value.to_string_lossy().into_owned()),
            None => DataValue::Null,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(name: &str) -> DataValue {
        EnvFunction
            .evaluate(&[DataValue::String(name.to_string())])
            .unwrap()
    }

    /// Reads a variable Cargo sets for every test binary rather than setting
    /// one: the environment is shared across test threads (see S9).
    #[test]
    fn reads_a_set_variable() {
        assert_eq!(env("CARGO_PKG_NAME"), DataValue::String("sql-cli".into()));
    }

    #[test]
    fn unset_and_impossible_names_are_null() {
        assert_eq!(env("SQL_CLI_SURELY_NOT_SET_7F3A"), DataValue::Null);
        assert_eq!(env(""), DataValue::Null);
        assert_eq!(env("A=B"), DataValue::Null);
        assert_eq!(env("A\0B"), DataValue::Null);
    }

    #[test]
    fn null_in_null_out() {
        assert_eq!(
            EnvFunction.evaluate(&[DataValue::Null]).unwrap(),
            DataValue::Null
        );
    }

    #[test]
    fn non_string_name_is_an_error() {
        assert!(EnvFunction.evaluate(&[DataValue::Integer(1)]).is_err());
    }

    /// Windows looks names up case-insensitively; Linux does not.
    #[cfg(windows)]
    #[test]
    fn windows_lookup_ignores_case() {
        assert_eq!(env("cargo_pkg_name"), env("CARGO_PKG_NAME"));
    }
}

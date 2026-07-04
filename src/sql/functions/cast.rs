use anyhow::{anyhow, Result};

use crate::data::datatable::DataValue;
use crate::sql::functions::{ArgCount, FunctionCategory, FunctionSignature, SqlFunction};

/// The set of target types CAST supports. We deliberately collapse SQL's large
/// "zoo" of character/numeric types down to the handful of types our engine
/// actually stores in `DataValue`. This keeps CAST close to DuckDB's observable
/// behaviour without dragging in fixed-width CHAR, DECIMAL precision, etc.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CastTarget {
    Integer,
    Float,
    Boolean,
    /// Any character type (VARCHAR/CHAR/TEXT/STRING/…) maps here.
    Varchar,
    /// Any temporal type (DATE/DATETIME/TIMESTAMP/…) maps here.
    DateTime,
}

impl CastTarget {
    /// Map a SQL type name (already stripped of any precision/scale) onto one of
    /// our supported target types. Returns `None` for types we cannot represent.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        let upper = name.trim().to_uppercase();
        // Collapse "DOUBLE PRECISION" and similar multi-word spellings.
        let key = upper.split_whitespace().next().unwrap_or("");
        match key {
            "INT" | "INTEGER" | "INT1" | "INT2" | "INT4" | "INT8" | "TINYINT" | "SMALLINT"
            | "BIGINT" | "HUGEINT" | "LONG" | "SHORT" | "SIGNED" | "UINTEGER" | "UBIGINT"
            | "USMALLINT" | "UTINYINT" => Some(CastTarget::Integer),
            "DOUBLE" | "FLOAT" | "FLOAT4" | "FLOAT8" | "REAL" | "DECIMAL" | "NUMERIC" | "DEC"
            | "NUMBER" => Some(CastTarget::Float),
            "BOOL" | "BOOLEAN" | "LOGICAL" => Some(CastTarget::Boolean),
            "VARCHAR" | "CHAR" | "CHARACTER" | "TEXT" | "STRING" | "NVARCHAR" | "NCHAR"
            | "BPCHAR" | "CLOB" => Some(CastTarget::Varchar),
            "DATE" | "DATETIME" | "TIMESTAMP" | "TIME" => Some(CastTarget::DateTime),
            _ => None,
        }
    }
}

/// Coerce a value to the requested target type. Errors describe why a value
/// could not be represented; callers decide whether that error is fatal (CAST)
/// or should become NULL (TRY_CAST). NULL always casts to NULL.
pub fn cast_value(value: &DataValue, target: CastTarget) -> Result<DataValue> {
    if matches!(value, DataValue::Null) {
        return Ok(DataValue::Null);
    }

    match target {
        CastTarget::Integer => cast_to_integer(value),
        CastTarget::Float => cast_to_float(value),
        CastTarget::Boolean => cast_to_boolean(value),
        CastTarget::Varchar => Ok(DataValue::String(value.to_string_optimized())),
        CastTarget::DateTime => cast_to_datetime(value),
    }
}

fn cast_to_integer(value: &DataValue) -> Result<DataValue> {
    let n = match value {
        DataValue::Integer(i) => *i,
        // Numeric-to-int rounds to nearest (matching DuckDB, which rounds rather
        // than truncates) using round-half-to-even so exact `.5` ties agree with
        // DuckDB, e.g. CAST(2.5 AS INT) = 2, CAST(3.5 AS INT) = 4.
        DataValue::Float(f) => f.round_ties_even() as i64,
        DataValue::Boolean(b) => i64::from(*b),
        DataValue::String(s) | DataValue::DateTime(s) => parse_integer(s)?,
        DataValue::InternedString(s) => parse_integer(s)?,
        other => return Err(anyhow!("cannot cast {:?} to INTEGER", other)),
    };
    Ok(DataValue::Integer(n))
}

fn cast_to_float(value: &DataValue) -> Result<DataValue> {
    let f = match value {
        DataValue::Integer(i) => *i as f64,
        DataValue::Float(f) => *f,
        DataValue::Boolean(b) => {
            if *b {
                1.0
            } else {
                0.0
            }
        }
        DataValue::String(s) | DataValue::DateTime(s) => parse_float(s)?,
        DataValue::InternedString(s) => parse_float(s)?,
        other => return Err(anyhow!("cannot cast {:?} to DOUBLE", other)),
    };
    Ok(DataValue::Float(f))
}

fn cast_to_boolean(value: &DataValue) -> Result<DataValue> {
    let b = match value {
        DataValue::Boolean(b) => *b,
        DataValue::Integer(i) => *i != 0,
        DataValue::Float(f) => *f != 0.0,
        DataValue::String(s) => parse_bool(s)?,
        DataValue::InternedString(s) => parse_bool(s)?,
        other => return Err(anyhow!("cannot cast {:?} to BOOLEAN", other)),
    };
    Ok(DataValue::Boolean(b))
}

fn cast_to_datetime(value: &DataValue) -> Result<DataValue> {
    match value {
        DataValue::DateTime(s) => Ok(DataValue::DateTime(s.clone())),
        // We store datetimes as their ISO-8601 string form; a string is taken
        // at face value rather than re-parsed, to stay within our type confines.
        DataValue::String(s) => Ok(DataValue::DateTime(s.clone())),
        DataValue::InternedString(s) => Ok(DataValue::DateTime(s.as_ref().clone())),
        other => Err(anyhow!("cannot cast {:?} to DATE/TIMESTAMP", other)),
    }
}

fn parse_integer(s: &str) -> Result<i64> {
    let trimmed = s.trim();
    trimmed
        .parse::<i64>()
        .map_err(|_| anyhow!("could not convert string '{}' to INTEGER", trimmed))
}

fn parse_float(s: &str) -> Result<f64> {
    let trimmed = s.trim();
    trimmed
        .parse::<f64>()
        .map_err(|_| anyhow!("could not convert string '{}' to DOUBLE", trimmed))
}

fn parse_bool(s: &str) -> Result<bool> {
    match s.trim().to_ascii_lowercase().as_str() {
        "true" | "t" | "yes" | "y" | "1" | "on" => Ok(true),
        "false" | "f" | "no" | "n" | "0" | "off" => Ok(false),
        other => Err(anyhow!("could not convert string '{}' to BOOLEAN", other)),
    }
}

/// SQL `CAST(expr AS type)`.
///
/// Parsed into a two-argument function call: `CAST(value, 'TYPE_NAME')`, where
/// the type name is carried as a string literal by the parser. `try_cast`
/// distinguishes `CAST` (errors on failure) from `TRY_CAST` (yields NULL).
pub struct CastFunction {
    pub try_cast: bool,
}

impl SqlFunction for CastFunction {
    fn signature(&self) -> FunctionSignature {
        if self.try_cast {
            FunctionSignature {
                name: "TRY_CAST",
                category: FunctionCategory::Conversion,
                arg_count: ArgCount::Fixed(2),
                description: "Cast a value to a target type, yielding NULL if the cast fails",
                returns: "Target type",
                examples: vec![
                    "SELECT TRY_CAST('abc' AS INTEGER)",
                    "SELECT TRY_CAST('42' AS INTEGER)",
                ],
            }
        } else {
            FunctionSignature {
                name: "CAST",
                category: FunctionCategory::Conversion,
                arg_count: ArgCount::Fixed(2),
                description: "Cast a value to a target type: CAST(expr AS type)",
                returns: "Target type",
                examples: vec![
                    "SELECT CAST('42' AS INTEGER)",
                    "SELECT CAST(price AS INTEGER) FROM trades",
                    "SELECT CAST(quantity AS DOUBLE) / 2 FROM trades",
                ],
            }
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let type_name = match &args[1] {
            DataValue::String(s) => s.as_str(),
            DataValue::InternedString(s) => s.as_str(),
            other => {
                return Err(anyhow!(
                    "CAST target type must be a type name, got {:?}",
                    other
                ))
            }
        };

        // An unknown target type is a query error, not a data error — so it
        // fails even under TRY_CAST.
        let target = CastTarget::from_name(type_name)
            .ok_or_else(|| anyhow!("unsupported CAST target type: {}", type_name))?;

        match cast_value(&args[0], target) {
            Ok(v) => Ok(v),
            Err(e) if self.try_cast => {
                let _ = e; // swallowed: TRY_CAST turns cast failures into NULL
                Ok(DataValue::Null)
            }
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cast(value: DataValue, ty: &str) -> Result<DataValue> {
        let func = CastFunction { try_cast: false };
        func.evaluate(&[value, DataValue::String(ty.to_string())])
    }

    fn try_cast(value: DataValue, ty: &str) -> DataValue {
        let func = CastFunction { try_cast: true };
        func.evaluate(&[value, DataValue::String(ty.to_string())])
            .unwrap()
    }

    #[test]
    fn string_to_integer() {
        assert_eq!(
            cast(DataValue::String("42".into()), "INTEGER").unwrap(),
            DataValue::Integer(42)
        );
    }

    #[test]
    fn float_to_integer_rounds() {
        assert_eq!(
            cast(DataValue::Float(2.9), "INT").unwrap(),
            DataValue::Integer(3)
        );
        assert_eq!(
            cast(DataValue::Float(-2.9), "BIGINT").unwrap(),
            DataValue::Integer(-3)
        );
    }

    #[test]
    fn float_to_integer_uses_banker_rounding_like_duckdb() {
        // Round-half-to-even, matching DuckDB's float->int cast.
        assert_eq!(
            cast(DataValue::Float(2.5), "INT").unwrap(),
            DataValue::Integer(2)
        );
        assert_eq!(
            cast(DataValue::Float(3.5), "INT").unwrap(),
            DataValue::Integer(4)
        );
        assert_eq!(
            cast(DataValue::Float(-2.5), "INT").unwrap(),
            DataValue::Integer(-2)
        );
    }

    #[test]
    fn integer_to_double() {
        assert_eq!(
            cast(DataValue::Integer(5), "DOUBLE").unwrap(),
            DataValue::Float(5.0)
        );
    }

    #[test]
    fn char_type_zoo_collapses_to_string() {
        for ty in [
            "VARCHAR",
            "CHAR",
            "TEXT",
            "STRING",
            "VARCHAR(50)".trim_end(),
        ] {
            let ty = ty.split('(').next().unwrap();
            assert_eq!(
                cast(DataValue::Integer(7), ty).unwrap(),
                DataValue::String("7".into())
            );
        }
    }

    #[test]
    fn to_boolean_variants() {
        assert_eq!(
            cast(DataValue::Integer(0), "BOOLEAN").unwrap(),
            DataValue::Boolean(false)
        );
        assert_eq!(
            cast(DataValue::Integer(3), "BOOL").unwrap(),
            DataValue::Boolean(true)
        );
        assert_eq!(
            cast(DataValue::String("true".into()), "BOOLEAN").unwrap(),
            DataValue::Boolean(true)
        );
    }

    #[test]
    fn null_casts_to_null() {
        assert_eq!(cast(DataValue::Null, "INTEGER").unwrap(), DataValue::Null);
    }

    #[test]
    fn invalid_cast_errors_but_try_cast_nulls() {
        assert!(cast(DataValue::String("abc".into()), "INTEGER").is_err());
        assert_eq!(
            try_cast(DataValue::String("abc".into()), "INTEGER"),
            DataValue::Null
        );
    }

    #[test]
    fn unknown_target_type_errors_even_for_try_cast() {
        let func = CastFunction { try_cast: true };
        let r = func.evaluate(&[DataValue::Integer(1), DataValue::String("BLOB".to_string())]);
        assert!(r.is_err());
    }
}

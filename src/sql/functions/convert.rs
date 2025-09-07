use anyhow::{anyhow, Result};

use crate::data::datatable::DataValue;
use crate::data::unit_converter::convert_units;
use crate::sql::functions::{ArgCount, FunctionCategory, FunctionSignature, SqlFunction};

/// CONVERT function - Convert values between different units
pub struct ConvertFunction;

impl SqlFunction for ConvertFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "CONVERT",
            category: FunctionCategory::Conversion,
            arg_count: ArgCount::Fixed(3),
            description: "Convert a value from one unit to another",
            returns: "FLOAT",
            examples: vec![
                "SELECT CONVERT(100, 'km', 'miles')",         // Distance
                "SELECT CONVERT(0, 'celsius', 'fahrenheit')", // Temperature
                "SELECT CONVERT(1, 'gallon', 'liters')",      // Volume
                "SELECT CONVERT(70, 'kg', 'pounds')",         // Weight
                "SELECT CONVERT(1, 'bar', 'psi')",            // Pressure
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        // Extract numeric value
        let value = match &args[0] {
            DataValue::Integer(n) => *n as f64,
            DataValue::Float(f) => *f,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("CONVERT first argument must be numeric")),
        };

        // Extract from_unit
        let from_unit = match &args[1] {
            DataValue::String(s) => s.as_str(),
            DataValue::InternedString(s) => s.as_str(),
            DataValue::Null => return Ok(DataValue::Null),
            _ => {
                return Err(anyhow!(
                    "CONVERT second argument must be a string (from_unit)"
                ))
            }
        };

        // Extract to_unit
        let to_unit = match &args[2] {
            DataValue::String(s) => s.as_str(),
            DataValue::InternedString(s) => s.as_str(),
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("CONVERT third argument must be a string (to_unit)")),
        };

        // Perform conversion
        match convert_units(value, from_unit, to_unit) {
            Ok(result) => Ok(DataValue::Float(result)),
            Err(e) => Err(anyhow!("Unit conversion failed: {}", e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_distance() {
        let func = ConvertFunction;

        // Test km to miles
        let args = vec![
            DataValue::Float(100.0),
            DataValue::String("km".to_string()),
            DataValue::String("miles".to_string()),
        ];
        let result = func.evaluate(&args).unwrap();
        if let DataValue::Float(val) = result {
            assert!((val - 62.137).abs() < 0.01);
        } else {
            panic!("Expected Float result");
        }
    }

    #[test]
    fn test_convert_temperature() {
        let func = ConvertFunction;

        // Test Celsius to Fahrenheit (0°C = 32°F)
        let args = vec![
            DataValue::Integer(0),
            DataValue::String("celsius".to_string()),
            DataValue::String("fahrenheit".to_string()),
        ];
        let result = func.evaluate(&args).unwrap();
        if let DataValue::Float(val) = result {
            assert!((val - 32.0).abs() < 0.01);
        } else {
            panic!("Expected Float result");
        }
    }

    #[test]
    fn test_convert_with_null() {
        let func = ConvertFunction;

        // Test with NULL value
        let args = vec![
            DataValue::Null,
            DataValue::String("km".to_string()),
            DataValue::String("miles".to_string()),
        ];
        let result = func.evaluate(&args).unwrap();
        assert!(matches!(result, DataValue::Null));
    }

    #[test]
    fn test_convert_invalid_unit() {
        let func = ConvertFunction;

        // Test with invalid unit
        let args = vec![
            DataValue::Float(100.0),
            DataValue::String("invalid_unit".to_string()),
            DataValue::String("miles".to_string()),
        ];
        let result = func.evaluate(&args);
        assert!(result.is_err());
    }
}

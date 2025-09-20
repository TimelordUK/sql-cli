use anyhow::{anyhow, Result};

use crate::data::datatable::DataValue;
use crate::sql::functions::{ArgCount, FunctionCategory, FunctionSignature, SqlFunction};

/// SIN function - Sine of angle in radians
pub struct SinFunction;

impl SqlFunction for SinFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "SIN",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(1),
            description: "Returns the sine of an angle in radians",
            returns: "FLOAT",
            examples: vec![
                "SELECT SIN(0)",           // Returns 0
                "SELECT SIN(PI()/2)",      // Returns 1
                "SELECT SIN(RADIANS(30))", // Returns 0.5
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let radians = match &args[0] {
            DataValue::Integer(n) => *n as f64,
            DataValue::Float(f) => *f,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("SIN requires a numeric argument")),
        };

        Ok(DataValue::Float(radians.sin()))
    }
}

/// COS function - Cosine of angle in radians
pub struct CosFunction;

impl SqlFunction for CosFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "COS",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(1),
            description: "Returns the cosine of an angle in radians",
            returns: "FLOAT",
            examples: vec![
                "SELECT COS(0)",           // Returns 1
                "SELECT COS(PI())",        // Returns -1
                "SELECT COS(RADIANS(60))", // Returns 0.5
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let radians = match &args[0] {
            DataValue::Integer(n) => *n as f64,
            DataValue::Float(f) => *f,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("COS requires a numeric argument")),
        };

        Ok(DataValue::Float(radians.cos()))
    }
}

/// TAN function - Tangent of angle in radians
pub struct TanFunction;

impl SqlFunction for TanFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "TAN",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(1),
            description: "Returns the tangent of an angle in radians",
            returns: "FLOAT",
            examples: vec![
                "SELECT TAN(0)",           // Returns 0
                "SELECT TAN(PI()/4)",      // Returns 1
                "SELECT TAN(RADIANS(45))", // Returns 1
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let radians = match &args[0] {
            DataValue::Integer(n) => *n as f64,
            DataValue::Float(f) => *f,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("TAN requires a numeric argument")),
        };

        Ok(DataValue::Float(radians.tan()))
    }
}

/// ASIN function - Arcsine (inverse sine) returns angle in radians
pub struct AsinFunction;

impl SqlFunction for AsinFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "ASIN",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(1),
            description:
                "Returns the arcsine (inverse sine) in radians. Input must be between -1 and 1",
            returns: "FLOAT",
            examples: vec![
                "SELECT ASIN(0)",            // Returns 0
                "SELECT ASIN(1)",            // Returns PI/2
                "SELECT DEGREES(ASIN(0.5))", // Returns 30
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let value = match &args[0] {
            DataValue::Integer(n) => *n as f64,
            DataValue::Float(f) => *f,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("ASIN requires a numeric argument")),
        };

        if value < -1.0 || value > 1.0 {
            return Err(anyhow!(
                "ASIN input must be between -1 and 1, got {}",
                value
            ));
        }

        Ok(DataValue::Float(value.asin()))
    }
}

/// ACOS function - Arccosine (inverse cosine) returns angle in radians
pub struct AcosFunction;

impl SqlFunction for AcosFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "ACOS",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(1),
            description:
                "Returns the arccosine (inverse cosine) in radians. Input must be between -1 and 1",
            returns: "FLOAT",
            examples: vec![
                "SELECT ACOS(1)",            // Returns 0
                "SELECT ACOS(0)",            // Returns PI/2
                "SELECT DEGREES(ACOS(0.5))", // Returns 60
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let value = match &args[0] {
            DataValue::Integer(n) => *n as f64,
            DataValue::Float(f) => *f,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("ACOS requires a numeric argument")),
        };

        if value < -1.0 || value > 1.0 {
            return Err(anyhow!(
                "ACOS input must be between -1 and 1, got {}",
                value
            ));
        }

        Ok(DataValue::Float(value.acos()))
    }
}

/// ATAN function - Arctangent (inverse tangent) returns angle in radians
pub struct AtanFunction;

impl SqlFunction for AtanFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "ATAN",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(1),
            description: "Returns the arctangent (inverse tangent) in radians",
            returns: "FLOAT",
            examples: vec![
                "SELECT ATAN(0)",          // Returns 0
                "SELECT ATAN(1)",          // Returns PI/4
                "SELECT DEGREES(ATAN(1))", // Returns 45
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let value = match &args[0] {
            DataValue::Integer(n) => *n as f64,
            DataValue::Float(f) => *f,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("ATAN requires a numeric argument")),
        };

        Ok(DataValue::Float(value.atan()))
    }
}

/// ATAN2 function - Two-argument arctangent returns angle in radians
pub struct Atan2Function;

impl SqlFunction for Atan2Function {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "ATAN2",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(2),
            description:
                "Returns the arctangent of y/x in radians, using signs to determine quadrant",
            returns: "FLOAT",
            examples: vec![
                "SELECT ATAN2(1, 1)",            // Returns PI/4
                "SELECT ATAN2(1, 0)",            // Returns PI/2
                "SELECT DEGREES(ATAN2(-1, -1))", // Returns -135
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let y = match &args[0] {
            DataValue::Integer(n) => *n as f64,
            DataValue::Float(f) => *f,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("ATAN2 first argument must be numeric")),
        };

        let x = match &args[1] {
            DataValue::Integer(n) => *n as f64,
            DataValue::Float(f) => *f,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("ATAN2 second argument must be numeric")),
        };

        Ok(DataValue::Float(y.atan2(x)))
    }
}

/// COT function - Cotangent of angle in radians
pub struct CotFunction;

impl SqlFunction for CotFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "COT",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(1),
            description: "Returns the cotangent of an angle in radians (1/tan)",
            returns: "FLOAT",
            examples: vec![
                "SELECT COT(PI()/4)",      // Returns 1
                "SELECT COT(PI()/2)",      // Returns ~0
                "SELECT COT(RADIANS(45))", // Returns 1
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let radians = match &args[0] {
            DataValue::Integer(n) => *n as f64,
            DataValue::Float(f) => *f,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("COT requires a numeric argument")),
        };

        let tan_val = radians.tan();
        if tan_val == 0.0 {
            return Err(anyhow!("COT undefined when tangent is zero"));
        }

        Ok(DataValue::Float(1.0 / tan_val))
    }
}

/// SINH function - Hyperbolic sine
pub struct SinhFunction;

impl SqlFunction for SinhFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "SINH",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(1),
            description: "Returns the hyperbolic sine of a number",
            returns: "FLOAT",
            examples: vec![
                "SELECT SINH(0)",  // Returns 0
                "SELECT SINH(1)",  // Returns ~1.175
                "SELECT SINH(-1)", // Returns ~-1.175
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let value = match &args[0] {
            DataValue::Integer(n) => *n as f64,
            DataValue::Float(f) => *f,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("SINH requires a numeric argument")),
        };

        Ok(DataValue::Float(value.sinh()))
    }
}

/// COSH function - Hyperbolic cosine
pub struct CoshFunction;

impl SqlFunction for CoshFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "COSH",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(1),
            description: "Returns the hyperbolic cosine of a number",
            returns: "FLOAT",
            examples: vec![
                "SELECT COSH(0)",  // Returns 1
                "SELECT COSH(1)",  // Returns ~1.543
                "SELECT COSH(-1)", // Returns ~1.543
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let value = match &args[0] {
            DataValue::Integer(n) => *n as f64,
            DataValue::Float(f) => *f,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("COSH requires a numeric argument")),
        };

        Ok(DataValue::Float(value.cosh()))
    }
}

/// TANH function - Hyperbolic tangent
pub struct TanhFunction;

impl SqlFunction for TanhFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "TANH",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(1),
            description: "Returns the hyperbolic tangent of a number",
            returns: "FLOAT",
            examples: vec![
                "SELECT TANH(0)",  // Returns 0
                "SELECT TANH(1)",  // Returns ~0.762
                "SELECT TANH(-1)", // Returns ~-0.762
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let value = match &args[0] {
            DataValue::Integer(n) => *n as f64,
            DataValue::Float(f) => *f,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("TANH requires a numeric argument")),
        };

        Ok(DataValue::Float(value.tanh()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn test_sin() {
        let func = SinFunction;

        // sin(0) = 0
        let result = func.evaluate(&[DataValue::Integer(0)]).unwrap();
        assert_eq!(result, DataValue::Float(0.0));

        // sin(π/2) = 1
        let result = func.evaluate(&[DataValue::Float(PI / 2.0)]).unwrap();
        if let DataValue::Float(val) = result {
            assert!((val - 1.0).abs() < 1e-10);
        } else {
            panic!("Expected Float result");
        }

        // sin(π) = 0
        let result = func.evaluate(&[DataValue::Float(PI)]).unwrap();
        if let DataValue::Float(val) = result {
            assert!(val.abs() < 1e-10);
        } else {
            panic!("Expected Float result");
        }
    }

    #[test]
    fn test_cos() {
        let func = CosFunction;

        // cos(0) = 1
        let result = func.evaluate(&[DataValue::Integer(0)]).unwrap();
        assert_eq!(result, DataValue::Float(1.0));

        // cos(π/2) = 0
        let result = func.evaluate(&[DataValue::Float(PI / 2.0)]).unwrap();
        if let DataValue::Float(val) = result {
            assert!(val.abs() < 1e-10);
        } else {
            panic!("Expected Float result");
        }

        // cos(π) = -1
        let result = func.evaluate(&[DataValue::Float(PI)]).unwrap();
        if let DataValue::Float(val) = result {
            assert!((val + 1.0).abs() < 1e-10);
        } else {
            panic!("Expected Float result");
        }
    }

    #[test]
    fn test_tan() {
        let func = TanFunction;

        // tan(0) = 0
        let result = func.evaluate(&[DataValue::Integer(0)]).unwrap();
        assert_eq!(result, DataValue::Float(0.0));

        // tan(π/4) = 1
        let result = func.evaluate(&[DataValue::Float(PI / 4.0)]).unwrap();
        if let DataValue::Float(val) = result {
            assert!((val - 1.0).abs() < 1e-10);
        } else {
            panic!("Expected Float result");
        }
    }

    #[test]
    fn test_asin() {
        let func = AsinFunction;

        // asin(0) = 0
        let result = func.evaluate(&[DataValue::Integer(0)]).unwrap();
        assert_eq!(result, DataValue::Float(0.0));

        // asin(1) = π/2
        let result = func.evaluate(&[DataValue::Integer(1)]).unwrap();
        if let DataValue::Float(val) = result {
            assert!((val - PI / 2.0).abs() < 1e-10);
        } else {
            panic!("Expected Float result");
        }

        // asin(value > 1) should error
        assert!(func.evaluate(&[DataValue::Float(1.5)]).is_err());
    }

    #[test]
    fn test_atan2() {
        let func = Atan2Function;

        // atan2(1, 1) = π/4
        let result = func
            .evaluate(&[DataValue::Integer(1), DataValue::Integer(1)])
            .unwrap();
        if let DataValue::Float(val) = result {
            assert!((val - PI / 4.0).abs() < 1e-10);
        } else {
            panic!("Expected Float result");
        }

        // atan2(1, 0) = π/2
        let result = func
            .evaluate(&[DataValue::Integer(1), DataValue::Integer(0)])
            .unwrap();
        if let DataValue::Float(val) = result {
            assert!((val - PI / 2.0).abs() < 1e-10);
        } else {
            panic!("Expected Float result");
        }
    }
}

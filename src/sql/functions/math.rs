use anyhow::{anyhow, Result};

use crate::data::datatable::DataValue;
use crate::sql::functions::{ArgCount, FunctionCategory, FunctionSignature, SqlFunction};

/// ROUND function - Round to specified decimal places
pub struct RoundFunction;

impl SqlFunction for RoundFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "ROUND",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Range(1, 2),
            description: "Round a number to specified decimal places",
            returns: "NUMBER",
            examples: vec![
                "SELECT ROUND(3.14159, 2)", // Returns 3.14
                "SELECT ROUND(123.456)",    // Returns 123
                "SELECT ROUND(1234.5, -2)", // Returns 1200
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let value = match &args[0] {
            DataValue::Integer(n) => *n as f64,
            DataValue::Float(f) => *f,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("ROUND requires a numeric argument")),
        };

        let decimals = if args.len() == 2 {
            match &args[1] {
                DataValue::Integer(n) => *n as i32,
                DataValue::Float(f) => *f as i32,
                _ => return Err(anyhow!("ROUND precision must be a number")),
            }
        } else {
            0
        };

        if decimals >= 0 {
            let multiplier = 10_f64.powi(decimals);
            let rounded = (value * multiplier).round() / multiplier;

            // Return integer if input was integer and result is unchanged
            if decimals == 0 {
                Ok(DataValue::Integer(rounded as i64))
            } else if matches!(&args[0], DataValue::Integer(_)) && rounded == value {
                // Input was integer and rounding didn't change it
                Ok(DataValue::Integer(value as i64))
            } else {
                Ok(DataValue::Float(rounded))
            }
        } else {
            // Negative decimals round to left of decimal point
            let divisor = 10_f64.powi(-decimals);
            let rounded = (value / divisor).round() * divisor;
            Ok(DataValue::Float(rounded))
        }
    }
}

/// ABS function - Absolute value
pub struct AbsFunction;

impl SqlFunction for AbsFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "ABS",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(1),
            description: "Returns the absolute value of a number",
            returns: "NUMBER",
            examples: vec![
                "SELECT ABS(-5)",   // Returns 5
                "SELECT ABS(3.14)", // Returns 3.14
                "SELECT ABS(price - cost) FROM products",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        match &args[0] {
            DataValue::Integer(n) => Ok(DataValue::Integer(n.abs())),
            DataValue::Float(f) => Ok(DataValue::Float(f.abs())),
            DataValue::Null => Ok(DataValue::Null),
            _ => Err(anyhow!("ABS requires a numeric argument")),
        }
    }
}

/// FLOOR function - Round down to nearest integer
pub struct FloorFunction;

impl SqlFunction for FloorFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "FLOOR",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(1),
            description: "Returns the largest integer less than or equal to the value",
            returns: "INTEGER",
            examples: vec![
                "SELECT FLOOR(3.7)",  // Returns 3
                "SELECT FLOOR(-2.3)", // Returns -3
                "SELECT FLOOR(5)",    // Returns 5
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        match &args[0] {
            DataValue::Integer(n) => Ok(DataValue::Integer(*n)),
            DataValue::Float(f) => Ok(DataValue::Integer(f.floor() as i64)),
            DataValue::Null => Ok(DataValue::Null),
            _ => Err(anyhow!("FLOOR requires a numeric argument")),
        }
    }
}

/// CEILING function - Round up to nearest integer
pub struct CeilingFunction;

impl SqlFunction for CeilingFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "CEILING",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(1),
            description: "Returns the smallest integer greater than or equal to the value",
            returns: "INTEGER",
            examples: vec![
                "SELECT CEILING(3.2)",  // Returns 4
                "SELECT CEILING(-2.7)", // Returns -2
                "SELECT CEILING(5)",    // Returns 5
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        match &args[0] {
            DataValue::Integer(n) => Ok(DataValue::Integer(*n)),
            DataValue::Float(f) => Ok(DataValue::Integer(f.ceil() as i64)),
            DataValue::Null => Ok(DataValue::Null),
            _ => Err(anyhow!("CEILING requires a numeric argument")),
        }
    }
}

/// CEIL function - Alias for CEILING function
pub struct CeilFunction;

impl SqlFunction for CeilFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "CEIL",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(1),
            description: "Alias for CEILING - Returns the smallest integer greater than or equal to the value",
            returns: "INTEGER",
            examples: vec![
                "SELECT CEIL(3.2)",  // Returns 4
                "SELECT CEIL(-2.7)", // Returns -2
                "SELECT CEIL(5)",    // Returns 5
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        // Delegate to CEILING function
        let ceiling_func = CeilingFunction;
        ceiling_func.evaluate(args)
    }
}

/// MOD function - Modulo operation
pub struct ModFunction;

impl SqlFunction for ModFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "MOD",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(2),
            description: "Returns the remainder of division",
            returns: "NUMBER",
            examples: vec![
                "SELECT MOD(10, 3)", // Returns 1
                "SELECT MOD(15, 4)", // Returns 3
                "SELECT MOD(id, 100) FROM table",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let dividend = match &args[0] {
            DataValue::Integer(n) => *n as f64,
            DataValue::Float(f) => *f,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("MOD requires numeric arguments")),
        };

        let divisor = match &args[1] {
            DataValue::Integer(n) => *n as f64,
            DataValue::Float(f) => *f,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("MOD requires numeric arguments")),
        };

        if divisor == 0.0 {
            return Err(anyhow!("Division by zero in MOD"));
        }

        // Check if both inputs were integers
        let both_integers =
            matches!(&args[0], DataValue::Integer(_)) && matches!(&args[1], DataValue::Integer(_));

        let result = dividend % divisor;

        if both_integers && result.fract() == 0.0 {
            Ok(DataValue::Integer(result as i64))
        } else {
            Ok(DataValue::Float(result))
        }
    }
}

/// QUOTIENT function - Integer division
pub struct QuotientFunction;

impl SqlFunction for QuotientFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "QUOTIENT",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(2),
            description: "Returns the integer portion of division",
            returns: "INTEGER",
            examples: vec![
                "SELECT QUOTIENT(10, 3)",  // Returns 3
                "SELECT QUOTIENT(15, 4)",  // Returns 3
                "SELECT QUOTIENT(100, 7)", // Returns 14
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let numerator = match &args[0] {
            DataValue::Integer(n) => *n as f64,
            DataValue::Float(f) => *f,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("QUOTIENT requires numeric arguments")),
        };

        let denominator = match &args[1] {
            DataValue::Integer(n) => *n as f64,
            DataValue::Float(f) => *f,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("QUOTIENT requires numeric arguments")),
        };

        if denominator == 0.0 {
            return Err(anyhow!("Division by zero in QUOTIENT"));
        }

        Ok(DataValue::Integer((numerator / denominator).trunc() as i64))
    }
}

/// SQRT function - Square root
pub struct SqrtFunction;

impl SqlFunction for SqrtFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "SQRT",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(1),
            description: "Returns the square root of a number",
            returns: "FLOAT",
            examples: vec![
                "SELECT SQRT(16)", // Returns 4.0
                "SELECT SQRT(2)",  // Returns 1.414...
                "SELECT SQRT(area) FROM squares",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let value = match &args[0] {
            DataValue::Integer(n) => *n as f64,
            DataValue::Float(f) => *f,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("SQRT requires a numeric argument")),
        };

        if value < 0.0 {
            return Err(anyhow!("SQRT of negative number"));
        }

        Ok(DataValue::Float(value.sqrt()))
    }
}

/// EXP function - e raised to the power
pub struct ExpFunction;

impl SqlFunction for ExpFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "EXP",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(1),
            description: "Returns e raised to the power of the given number",
            returns: "FLOAT",
            examples: vec![
                "SELECT EXP(1)",      // Returns e (2.718...)
                "SELECT EXP(0)",      // Returns 1
                "SELECT EXP(LN(10))", // Returns 10
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let value = match &args[0] {
            DataValue::Integer(n) => *n as f64,
            DataValue::Float(f) => *f,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("EXP requires a numeric argument")),
        };

        Ok(DataValue::Float(value.exp()))
    }
}

/// LN function - Natural logarithm
pub struct LnFunction;

impl SqlFunction for LnFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "LN",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(1),
            description: "Returns the natural logarithm (base e) of a number",
            returns: "FLOAT",
            examples: vec![
                "SELECT LN(2.718282)", // Returns ~1
                "SELECT LN(10)",       // Returns 2.302...
                "SELECT LN(1)",        // Returns 0
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let value = match &args[0] {
            DataValue::Integer(n) => *n as f64,
            DataValue::Float(f) => *f,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("LN requires a numeric argument")),
        };

        if value <= 0.0 {
            return Err(anyhow!("LN of non-positive number"));
        }

        Ok(DataValue::Float(value.ln()))
    }
}

/// LOG function - Logarithm with specified base
pub struct LogFunction;

impl SqlFunction for LogFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "LOG",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Range(1, 2),
            description:
                "Returns the logarithm of a number (base 10 by default, or specified base)",
            returns: "FLOAT",
            examples: vec![
                "SELECT LOG(100)",      // Returns 2 (base 10)
                "SELECT LOG(8, 2)",     // Returns 3 (log base 2 of 8)
                "SELECT LOG(1000, 10)", // Returns 3
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let value = match &args[0] {
            DataValue::Integer(n) => *n as f64,
            DataValue::Float(f) => *f,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("LOG requires numeric arguments")),
        };

        if value <= 0.0 {
            return Err(anyhow!("LOG of non-positive number"));
        }

        let base = if args.len() == 2 {
            match &args[1] {
                DataValue::Integer(n) => *n as f64,
                DataValue::Float(f) => *f,
                _ => return Err(anyhow!("LOG base must be a number")),
            }
        } else {
            10.0
        };

        if base <= 0.0 || base == 1.0 {
            return Err(anyhow!("Invalid logarithm base"));
        }

        Ok(DataValue::Float(value.log(base)))
    }
}

/// LOG10 function - Base-10 logarithm
pub struct Log10Function;

impl SqlFunction for Log10Function {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "LOG10",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(1),
            description: "Returns the base-10 logarithm of a number",
            returns: "FLOAT",
            examples: vec![
                "SELECT LOG10(100)",  // Returns 2
                "SELECT LOG10(1000)", // Returns 3
                "SELECT LOG10(0.1)",  // Returns -1
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let value = match &args[0] {
            DataValue::Integer(n) => *n as f64,
            DataValue::Float(f) => *f,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("LOG10 requires a numeric argument")),
        };

        if value <= 0.0 {
            return Err(anyhow!("LOG10 of non-positive number"));
        }

        Ok(DataValue::Float(value.log10()))
    }
}

/// POWER function - Raise to power
pub struct PowerFunction;

impl SqlFunction for PowerFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "POWER",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(2),
            description: "Returns a number raised to a power",
            returns: "NUMBER",
            examples: vec![
                "SELECT POWER(2, 3)",   // Returns 8
                "SELECT POWER(10, -2)", // Returns 0.01
                "SELECT POWER(9, 0.5)", // Returns 3
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let base = match &args[0] {
            DataValue::Integer(n) => *n as f64,
            DataValue::Float(f) => *f,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("POWER requires numeric arguments")),
        };

        let exponent = match &args[1] {
            DataValue::Integer(n) => *n as f64,
            DataValue::Float(f) => *f,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("POWER requires numeric arguments")),
        };

        let result = base.powf(exponent);

        // Always return float for POWER function to match SQL standards
        Ok(DataValue::Float(result))
    }
}

/// POW function - Alias for POWER function
pub struct PowFunction;

impl SqlFunction for PowFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "POW",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(2),
            description: "Returns a number raised to a power (alias for POWER)",
            returns: "FLOAT",
            examples: vec![
                "SELECT POW(2, 3)",   // Returns 8.0
                "SELECT POW(10, -2)", // Returns 0.01
                "SELECT POW(9, 0.5)", // Returns 3.0
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        // Delegate to POWER function
        let power_func = PowerFunction;
        power_func.evaluate(args)
    }
}

/// DEGREES function - Convert radians to degrees
pub struct DegreesFunction;

impl SqlFunction for DegreesFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "DEGREES",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(1),
            description: "Convert radians to degrees",
            returns: "FLOAT",
            examples: vec![
                "SELECT DEGREES(PI())",   // Returns 180
                "SELECT DEGREES(PI()/2)", // Returns 90
                "SELECT DEGREES(1)",      // Returns 57.2958...
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let radians = match &args[0] {
            DataValue::Integer(n) => *n as f64,
            DataValue::Float(f) => *f,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("DEGREES requires a numeric argument")),
        };

        Ok(DataValue::Float(radians * 180.0 / std::f64::consts::PI))
    }
}

/// RADIANS function - Convert degrees to radians
pub struct RadiansFunction;

impl SqlFunction for RadiansFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "RADIANS",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(1),
            description: "Convert degrees to radians",
            returns: "FLOAT",
            examples: vec![
                "SELECT RADIANS(180)", // Returns PI
                "SELECT RADIANS(90)",  // Returns PI/2
                "SELECT RADIANS(45)",  // Returns PI/4
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let degrees = match &args[0] {
            DataValue::Integer(n) => *n as f64,
            DataValue::Float(f) => *f,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("RADIANS requires a numeric argument")),
        };

        Ok(DataValue::Float(degrees * std::f64::consts::PI / 180.0))
    }
}

/// Register all math functions
pub fn register_math_functions(registry: &mut super::FunctionRegistry) {
    registry.register(Box::new(RoundFunction));
    registry.register(Box::new(AbsFunction));
    registry.register(Box::new(FloorFunction));
    registry.register(Box::new(CeilingFunction));
    registry.register(Box::new(CeilFunction)); // Add CEIL alias
    registry.register(Box::new(ModFunction));
    registry.register(Box::new(QuotientFunction));
    registry.register(Box::new(SqrtFunction));
    registry.register(Box::new(ExpFunction));
    registry.register(Box::new(LnFunction));
    registry.register(Box::new(LogFunction));
    registry.register(Box::new(Log10Function));
    registry.register(Box::new(PowerFunction));
    registry.register(Box::new(PowFunction)); // Add POW alias
    registry.register(Box::new(DegreesFunction));
    registry.register(Box::new(RadiansFunction));
}

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

/// FACTORIAL function - Calculate factorial of a number
pub struct FactorialFunction;

impl FactorialFunction {
    /// Precomputed factorials for 0! through 20!
    /// Beyond 20!, the values exceed u64 max
    const FACTORIAL_TABLE: [u64; 21] = [
        1,                         // 0!
        1,                         // 1!
        2,                         // 2!
        6,                         // 3!
        24,                        // 4!
        120,                       // 5!
        720,                       // 6!
        5_040,                     // 7!
        40_320,                    // 8!
        362_880,                   // 9!
        3_628_800,                 // 10!
        39_916_800,                // 11!
        479_001_600,               // 12!
        6_227_020_800,             // 13!
        87_178_291_200,            // 14!
        1_307_674_368_000,         // 15!
        20_922_789_888_000,        // 16!
        355_687_428_096_000,       // 17!
        6_402_373_705_728_000,     // 18!
        121_645_100_408_832_000,   // 19!
        2_432_902_008_176_640_000, // 20!
    ];

    fn factorial(n: u64) -> Result<u64> {
        if n <= 20 {
            Ok(Self::FACTORIAL_TABLE[n as usize])
        } else {
            Err(anyhow!("FACTORIAL: argument {} too large (max is 20)", n))
        }
    }
}

impl SqlFunction for FactorialFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "FACTORIAL",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(1),
            description: "Returns the factorial of a non-negative integer (n!)",
            returns: "INTEGER",
            examples: vec![
                "SELECT FACTORIAL(5)",  // Returns 120
                "SELECT FACTORIAL(10)", // Returns 3628800
                "SELECT FACTORIAL(0)",  // Returns 1
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let n = match &args[0] {
            DataValue::Integer(i) if *i >= 0 && *i <= 20 => *i as u64,
            DataValue::Integer(i) if *i < 0 => {
                return Err(anyhow!("FACTORIAL requires a non-negative integer"))
            }
            DataValue::Integer(i) => {
                return Err(anyhow!("FACTORIAL: argument {} too large (max is 20)", i))
            }
            DataValue::Float(f) if f.is_finite() && *f >= 0.0 && f.floor() == *f => {
                let n = *f as i64;
                if n > 20 {
                    return Err(anyhow!("FACTORIAL: argument {} too large (max is 20)", n));
                }
                n as u64
            }
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("FACTORIAL requires a non-negative integer")),
        };

        Ok(DataValue::Integer(Self::factorial(n)? as i64))
    }
}

/// SUM_N function - Sum of first n natural numbers (triangular number)
pub struct SumNFunction;

impl SqlFunction for SumNFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "SUM_N",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(1),
            description: "Calculate sum of first n natural numbers (triangular number)",
            returns: "INTEGER",
            examples: vec![
                "SELECT SUM_N(10)",  // Returns 55 (1+2+3+...+10)
                "SELECT SUM_N(100)", // Returns 5050
                "SELECT SUM_N(5)",   // Returns 15 (1+2+3+4+5)
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let n = match &args[0] {
            DataValue::Integer(n) if *n >= 0 => *n,
            DataValue::Float(f) if f.is_finite() && *f >= 0.0 && f.floor() == *f => *f as i64,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("SUM_N requires a non-negative integer")),
        };

        // Calculate n * (n + 1) / 2
        // Check for overflow
        if n > 3037000499 {
            // sqrt(i64::MAX * 2) approximately
            return Err(anyhow!("SUM_N: argument {} too large (would overflow)", n));
        }

        let result = n * (n + 1) / 2;
        Ok(DataValue::Integer(result))
    }
}

/// SUM_N_SQR function - sum of squares from 1 to n
pub struct SumNSqrFunction;

impl SqlFunction for SumNSqrFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "SUM_N_SQR",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(1),
            description: "Sum of squares from 1 to n: 1² + 2² + ... + n²",
            returns: "Integer - sum of squares (n(n+1)(2n+1)/6)",
            examples: vec![
                "SELECT SUM_N_SQR(5) -- Returns 55 (1+4+9+16+25)",
                "SELECT SUM_N_SQR(10) -- Returns 385",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Err(anyhow!("SUM_N_SQR requires exactly 1 argument"));
        }

        let n = match &args[0] {
            DataValue::Integer(i) => *i,
            DataValue::Float(f) => *f as i64,
            _ => return Err(anyhow!("SUM_N_SQR requires a numeric argument")),
        };

        if n < 0 {
            return Err(anyhow!("SUM_N_SQR requires a non-negative number"));
        }

        // Formula: n(n+1)(2n+1)/6
        let result = n * (n + 1) * (2 * n + 1) / 6;
        Ok(DataValue::Integer(result))
    }
}

/// SUM_N_CUBE function - sum of cubes from 1 to n
pub struct SumNCubeFunction;

impl SqlFunction for SumNCubeFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "SUM_N_CUBE",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(1),
            description: "Sum of cubes from 1 to n: 1³ + 2³ + ... + n³",
            returns: "Integer - sum of cubes ([n(n+1)/2]²)",
            examples: vec![
                "SELECT SUM_N_CUBE(4) -- Returns 100 (1+8+27+64)",
                "SELECT SUM_N_CUBE(5) -- Returns 225",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Err(anyhow!("SUM_N_CUBE requires exactly 1 argument"));
        }

        let n = match &args[0] {
            DataValue::Integer(i) => *i,
            DataValue::Float(f) => *f as i64,
            _ => return Err(anyhow!("SUM_N_CUBE requires a numeric argument")),
        };

        if n < 0 {
            return Err(anyhow!("SUM_N_CUBE requires a non-negative number"));
        }

        // Formula: [n(n+1)/2]² - which is the square of sum_n!
        let sum_n = n * (n + 1) / 2;
        let result = sum_n * sum_n;
        Ok(DataValue::Integer(result))
    }
}

/// HARMONIC function - harmonic series sum 1 + 1/2 + 1/3 + ... + 1/n
pub struct HarmonicFunction;

impl SqlFunction for HarmonicFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "HARMONIC",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(1),
            description: "Harmonic series: 1 + 1/2 + 1/3 + ... + 1/n",
            returns: "Float - sum of harmonic series",
            examples: vec![
                "SELECT HARMONIC(4) -- Returns 2.0833... (1 + 0.5 + 0.333... + 0.25)",
                "SELECT HARMONIC(10) -- Returns 2.9290...",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Err(anyhow!("HARMONIC requires exactly 1 argument"));
        }

        let n = match &args[0] {
            DataValue::Integer(i) => *i,
            DataValue::Float(f) => *f as i64,
            _ => return Err(anyhow!("HARMONIC requires a numeric argument")),
        };

        if n <= 0 {
            return Err(anyhow!("HARMONIC requires a positive number"));
        }

        let mut sum = 0.0;
        for i in 1..=n {
            sum += 1.0 / i as f64;
        }

        Ok(DataValue::Float(sum))
    }
}

/// FIBONACCI function - returns the nth Fibonacci number
pub struct FibonacciFunction;

impl SqlFunction for FibonacciFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "FIBONACCI",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(1),
            description: "Returns the nth Fibonacci number (0, 1, 1, 2, 3, 5, 8, ...)",
            returns: "Integer - nth Fibonacci number",
            examples: vec![
                "SELECT FIBONACCI(7) -- Returns 13",
                "SELECT FIBONACCI(10) -- Returns 55",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Err(anyhow!("FIBONACCI requires exactly 1 argument"));
        }

        let n = match &args[0] {
            DataValue::Integer(i) => *i,
            DataValue::Float(f) => *f as i64,
            _ => return Err(anyhow!("FIBONACCI requires a numeric argument")),
        };

        if n < 0 {
            return Err(anyhow!("FIBONACCI requires a non-negative number"));
        }

        if n == 0 {
            return Ok(DataValue::Integer(0));
        }
        if n == 1 {
            return Ok(DataValue::Integer(1));
        }

        let mut a = 0i64;
        let mut b = 1i64;

        for _ in 2..=n {
            let temp = a + b;
            a = b;
            b = temp;
        }

        Ok(DataValue::Integer(b))
    }
}

/// GEOMETRIC function - geometric series sum: a + ar + ar² + ... + ar^(n-1)
pub struct GeometricFunction;

impl SqlFunction for GeometricFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "GEOMETRIC",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(3),
            description: "Geometric series sum: a + ar + ar² + ... + ar^(n-1)",
            returns: "Float - sum of geometric series",
            examples: vec![
                "SELECT GEOMETRIC(1, 2, 5) -- Returns 31 (1 + 2 + 4 + 8 + 16)",
                "SELECT GEOMETRIC(1, 0.5, 10) -- Returns 1.998... (converging series)",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 3 {
            return Err(anyhow!("GEOMETRIC requires exactly 3 arguments (a, r, n)"));
        }

        let a = match &args[0] {
            DataValue::Integer(i) => *i as f64,
            DataValue::Float(f) => *f,
            _ => return Err(anyhow!("GEOMETRIC first argument (a) must be numeric")),
        };

        let r = match &args[1] {
            DataValue::Integer(i) => *i as f64,
            DataValue::Float(f) => *f,
            _ => return Err(anyhow!("GEOMETRIC second argument (r) must be numeric")),
        };

        let n = match &args[2] {
            DataValue::Integer(i) => *i,
            DataValue::Float(f) => *f as i64,
            _ => return Err(anyhow!("GEOMETRIC third argument (n) must be numeric")),
        };

        if n < 0 {
            return Err(anyhow!("GEOMETRIC requires non-negative n"));
        }

        if n == 0 {
            return Ok(DataValue::Float(0.0));
        }

        // Formula: a * (1 - r^n) / (1 - r) for r != 1
        // If r = 1, sum = a * n
        let sum = if (r - 1.0).abs() < 1e-10 {
            a * n as f64
        } else {
            a * (1.0 - r.powi(n as i32)) / (1.0 - r)
        };

        Ok(DataValue::Float(sum))
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
    registry.register(Box::new(FactorialFunction));
    registry.register(Box::new(SumNFunction));
    registry.register(Box::new(SumNSqrFunction));
    registry.register(Box::new(SumNCubeFunction));
    registry.register(Box::new(HarmonicFunction));
    registry.register(Box::new(FibonacciFunction));
    registry.register(Box::new(GeometricFunction));
}

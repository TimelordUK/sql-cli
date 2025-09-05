use anyhow::Result;

use super::{ArgCount, FunctionCategory, FunctionSignature, SqlFunction};
use crate::data::datatable::DataValue;

/// PI constant function
pub struct PiFunction;

impl SqlFunction for PiFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "PI",
            category: FunctionCategory::Constant,
            arg_count: ArgCount::Fixed(0),
            description: "Returns the value of π (pi)",
            returns: "FLOAT",
            examples: vec!["SELECT PI()", "SELECT radius * 2 * PI() AS circumference"],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        Ok(DataValue::Float(std::f64::consts::PI))
    }
}

/// E (Euler's number) constant function
pub struct EFunction;

impl SqlFunction for EFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "E",
            category: FunctionCategory::Constant,
            arg_count: ArgCount::Fixed(0),
            description: "Returns Euler's number (e ≈ 2.71828)",
            returns: "FLOAT",
            examples: vec!["SELECT E()", "SELECT POW(E(), x) AS exp_x"],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        Ok(DataValue::Float(std::f64::consts::E))
    }
}

/// Mass of electron in kg
pub struct MeFunction;

/// Alias for ME function
pub struct MassElectronFunction;

impl SqlFunction for MeFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "ME",
            category: FunctionCategory::Constant,
            arg_count: ArgCount::Fixed(0),
            description: "Returns the mass of an electron in kg (9.10938356 × 10^-31)",
            returns: "FLOAT",
            examples: vec!["SELECT ME()", "SELECT mass / ME() AS electron_masses"],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        Ok(DataValue::Float(9.1093837015e-31))
    }
}

impl SqlFunction for MassElectronFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "MASS_ELECTRON",
            category: FunctionCategory::Constant,
            arg_count: ArgCount::Fixed(0),
            description: "Alias for ME() - Returns the mass of an electron in kg",
            returns: "FLOAT",
            examples: vec!["SELECT MASS_ELECTRON()"],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        Ok(DataValue::Float(9.1093837015e-31))
    }
}

/// TAU constant (2π)
pub struct TauFunction;

impl SqlFunction for TauFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "TAU",
            category: FunctionCategory::Constant,
            arg_count: ArgCount::Fixed(0),
            description: "Returns tau (τ = 2π = 6.28318...)",
            returns: "FLOAT",
            examples: vec!["SELECT TAU()", "SELECT radius * TAU() as circumference"],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        Ok(DataValue::Float(2.0 * std::f64::consts::PI))
    }
}

/// PHI constant (Golden ratio)
pub struct PhiFunction;

impl SqlFunction for PhiFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "PHI",
            category: FunctionCategory::Constant,
            arg_count: ArgCount::Fixed(0),
            description: "Returns the golden ratio φ (1.61803...)",
            returns: "FLOAT",
            examples: vec!["SELECT PHI()", "SELECT width * PHI() as golden_height"],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        Ok(DataValue::Float(1.618033988749895)) // (1 + sqrt(5)) / 2
    }
}

/// HBAR constant (Reduced Planck constant)
pub struct HbarFunction;

impl SqlFunction for HbarFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "HBAR",
            category: FunctionCategory::Constant,
            arg_count: ArgCount::Fixed(0),
            description: "Returns the reduced Planck constant ħ in J⋅s (1.055 × 10⁻³⁴)",
            returns: "FLOAT",
            examples: vec!["SELECT HBAR()", "SELECT HBAR() / (2 * PI()) as h_bar"],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        Ok(DataValue::Float(1.054571817e-34))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pi_function() {
        let func = PiFunction;
        let result = func.evaluate(&[]).unwrap();
        match result {
            DataValue::Float(val) => assert!((val - std::f64::consts::PI).abs() < 1e-10),
            _ => panic!("Expected Float"),
        }
    }

    #[test]
    fn test_pi_with_args_fails() {
        let func = PiFunction;
        let result = func.evaluate(&[DataValue::Float(1.0)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_e_function() {
        let func = EFunction;
        let result = func.evaluate(&[]).unwrap();
        match result {
            DataValue::Float(val) => assert!((val - std::f64::consts::E).abs() < 1e-10),
            _ => panic!("Expected Float"),
        }
    }

    #[test]
    fn test_me_function() {
        let func = MeFunction;
        let result = func.evaluate(&[]).unwrap();
        match result {
            DataValue::Float(val) => assert!((val - 9.1093837015e-31).abs() < 1e-40),
            _ => panic!("Expected Float"),
        }
    }
}

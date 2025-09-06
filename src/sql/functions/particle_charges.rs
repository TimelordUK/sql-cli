use anyhow::Result;

use super::{ArgCount, FunctionCategory, FunctionSignature, SqlFunction};
use crate::data::datatable::DataValue;

/// Electron charge (-e)
pub struct ChargeElectronFunction;

impl SqlFunction for ChargeElectronFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "CHARGE_ELECTRON",
            category: FunctionCategory::Constant,
            arg_count: ArgCount::Fixed(0),
            description: "Returns the electron charge in coulombs (-1.602176634 × 10^-19 C)",
            returns: "FLOAT",
            examples: vec![
                "SELECT CHARGE_ELECTRON()",
                "SELECT n_electrons * CHARGE_ELECTRON() AS total_charge",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        Ok(DataValue::Float(-1.602176634e-19)) // Negative charge
    }
}

/// Proton charge (+e)
pub struct ChargeProtonFunction;

impl SqlFunction for ChargeProtonFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "CHARGE_PROTON",
            category: FunctionCategory::Constant,
            arg_count: ArgCount::Fixed(0),
            description: "Returns the proton charge in coulombs (+1.602176634 × 10^-19 C)",
            returns: "FLOAT",
            examples: vec![
                "SELECT CHARGE_PROTON()",
                "SELECT n_protons * CHARGE_PROTON() AS nuclear_charge",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        Ok(DataValue::Float(1.602176634e-19)) // Positive charge
    }
}

/// Neutron charge (0)
pub struct ChargeNeutronFunction;

impl SqlFunction for ChargeNeutronFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "CHARGE_NEUTRON",
            category: FunctionCategory::Constant,
            arg_count: ArgCount::Fixed(0),
            description: "Returns the neutron charge in coulombs (0 C)",
            returns: "FLOAT",
            examples: vec![
                "SELECT CHARGE_NEUTRON()",
                "SELECT CHARGE_NEUTRON() + CHARGE_PROTON() AS net_nucleon_charge",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        Ok(DataValue::Float(0.0)) // Neutral (no charge)
    }
}

/// Up quark charge (+2/3 e)
pub struct ChargeUpQuarkFunction;

impl SqlFunction for ChargeUpQuarkFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "CHARGE_UP_QUARK",
            category: FunctionCategory::Constant,
            arg_count: ArgCount::Fixed(0),
            description: "Returns the up quark charge in coulombs (+2/3 × 1.602176634 × 10^-19 C)",
            returns: "FLOAT",
            examples: vec![
                "SELECT CHARGE_UP_QUARK()",
                "SELECT 2 * CHARGE_UP_QUARK() + CHARGE_DOWN_QUARK() AS proton_charge",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        Ok(DataValue::Float(2.0 / 3.0 * 1.602176634e-19)) // +2/3 e
    }
}

/// Down quark charge (-1/3 e)
pub struct ChargeDownQuarkFunction;

impl SqlFunction for ChargeDownQuarkFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "CHARGE_DOWN_QUARK",
            category: FunctionCategory::Constant,
            arg_count: ArgCount::Fixed(0),
            description:
                "Returns the down quark charge in coulombs (-1/3 × 1.602176634 × 10^-19 C)",
            returns: "FLOAT",
            examples: vec![
                "SELECT CHARGE_DOWN_QUARK()",
                "SELECT CHARGE_UP_QUARK() + 2 * CHARGE_DOWN_QUARK() AS neutron_charge",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        Ok(DataValue::Float(-1.0 / 3.0 * 1.602176634e-19)) // -1/3 e
    }
}

/// Positron charge (+e) - antiparticle of electron
pub struct ChargePositronFunction;

impl SqlFunction for ChargePositronFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "CHARGE_POSITRON",
            category: FunctionCategory::Constant,
            arg_count: ArgCount::Fixed(0),
            description: "Returns the positron charge in coulombs (+1.602176634 × 10^-19 C)",
            returns: "FLOAT",
            examples: vec![
                "SELECT CHARGE_POSITRON()",
                "SELECT CHARGE_ELECTRON() + CHARGE_POSITRON() AS annihilation_charge",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        Ok(DataValue::Float(1.602176634e-19)) // Positive charge (antielectron)
    }
}

/// Muon charge (-e)
pub struct ChargeMuonFunction;

impl SqlFunction for ChargeMuonFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "CHARGE_MUON",
            category: FunctionCategory::Constant,
            arg_count: ArgCount::Fixed(0),
            description: "Returns the muon charge in coulombs (-1.602176634 × 10^-19 C)",
            returns: "FLOAT",
            examples: vec![
                "SELECT CHARGE_MUON()",
                "SELECT CHARGE_MUON() / CHARGE_ELECTRON() AS charge_ratio",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        Ok(DataValue::Float(-1.602176634e-19)) // Same charge as electron
    }
}

/// Tau charge (-e)
pub struct ChargeTauFunction;

impl SqlFunction for ChargeTauFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "CHARGE_TAU",
            category: FunctionCategory::Constant,
            arg_count: ArgCount::Fixed(0),
            description: "Returns the tau lepton charge in coulombs (-1.602176634 × 10^-19 C)",
            returns: "FLOAT",
            examples: vec![
                "SELECT CHARGE_TAU()",
                "SELECT CHARGE_TAU() = CHARGE_ELECTRON() AS same_charge",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        Ok(DataValue::Float(-1.602176634e-19)) // Same charge as electron
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_electron_charge() {
        let func = ChargeElectronFunction;
        let result = func.evaluate(&[]).unwrap();
        match result {
            DataValue::Float(val) => {
                assert_eq!(val, -1.602176634e-19);
                assert!(val < 0.0, "Electron charge should be negative");
            }
            _ => panic!("Expected Float"),
        }
    }

    #[test]
    fn test_proton_charge() {
        let func = ChargeProtonFunction;
        let result = func.evaluate(&[]).unwrap();
        match result {
            DataValue::Float(val) => {
                assert_eq!(val, 1.602176634e-19);
                assert!(val > 0.0, "Proton charge should be positive");
            }
            _ => panic!("Expected Float"),
        }
    }

    #[test]
    fn test_neutron_charge() {
        let func = ChargeNeutronFunction;
        let result = func.evaluate(&[]).unwrap();
        match result {
            DataValue::Float(val) => {
                assert_eq!(val, 0.0);
                assert_eq!(val, 0.0, "Neutron charge should be zero");
            }
            _ => panic!("Expected Float"),
        }
    }

    #[test]
    fn test_charge_conservation() {
        let electron = ChargeElectronFunction.evaluate(&[]).unwrap();
        let proton = ChargeProtonFunction.evaluate(&[]).unwrap();

        if let (DataValue::Float(e), DataValue::Float(p)) = (electron, proton) {
            assert_eq!(e + p, 0.0, "Electron and proton charges should cancel");
        } else {
            panic!("Expected Float values");
        }
    }

    #[test]
    fn test_quark_charges() {
        let up = ChargeUpQuarkFunction.evaluate(&[]).unwrap();
        let down = ChargeDownQuarkFunction.evaluate(&[]).unwrap();

        if let (DataValue::Float(u), DataValue::Float(d)) = (up, down) {
            // Proton = 2 up + 1 down
            let proton_charge = 2.0 * u + d;
            assert!(
                (proton_charge - 1.602176634e-19).abs() < 1e-30,
                "Proton charge from quarks"
            );

            // Neutron = 1 up + 2 down
            let neutron_charge = u + 2.0 * d;
            assert!(neutron_charge.abs() < 1e-30, "Neutron charge from quarks");
        } else {
            panic!("Expected Float values");
        }
    }
}

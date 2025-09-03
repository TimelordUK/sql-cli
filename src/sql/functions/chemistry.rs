use anyhow::{anyhow, Result};
use std::collections::HashMap;

use super::{ArgCount, FunctionCategory, FunctionSignature, SqlFunction};
use crate::data::datatable::DataValue;

/// Avogadro's number
pub struct AvogadroFunction;

impl SqlFunction for AvogadroFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "AVOGADRO",
            category: FunctionCategory::Chemical,
            arg_count: ArgCount::Fixed(0),
            description: "Returns Avogadro's number (6.022 × 10^23)",
            returns: "FLOAT",
            examples: vec![
                "SELECT AVOGADRO()",
                "SELECT molecules / AVOGADRO() AS moles",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        Ok(DataValue::Float(6.022140857e23))
    }
}

/// Atomic mass function - returns atomic mass for an element
pub struct AtomicMassFunction;

impl AtomicMassFunction {
    fn get_atomic_mass(element: &str) -> Option<f64> {
        let masses: HashMap<&str, f64> = [
            // First 20 elements
            ("H", 1.008),
            ("HYDROGEN", 1.008),
            ("HE", 4.003),
            ("HELIUM", 4.003),
            ("LI", 6.941),
            ("LITHIUM", 6.941),
            ("BE", 9.012),
            ("BERYLLIUM", 9.012),
            ("B", 10.81),
            ("BORON", 10.81),
            ("C", 12.01),
            ("CARBON", 12.01),
            ("N", 14.01),
            ("NITROGEN", 14.01),
            ("O", 16.00),
            ("OXYGEN", 16.00),
            ("F", 19.00),
            ("FLUORINE", 19.00),
            ("NE", 20.18),
            ("NEON", 20.18),
            ("NA", 22.99),
            ("SODIUM", 22.99),
            ("MG", 24.31),
            ("MAGNESIUM", 24.31),
            ("AL", 26.98),
            ("ALUMINUM", 26.98),
            ("ALUMINIUM", 26.98),
            ("SI", 28.09),
            ("SILICON", 28.09),
            ("P", 30.97),
            ("PHOSPHORUS", 30.97),
            ("S", 32.07),
            ("SULFUR", 32.07),
            ("SULPHUR", 32.07),
            ("CL", 35.45),
            ("CHLORINE", 35.45),
            ("AR", 39.95),
            ("ARGON", 39.95),
            ("K", 39.10),
            ("POTASSIUM", 39.10),
            ("CA", 40.08),
            ("CALCIUM", 40.08),
            // Common elements beyond first 20
            ("FE", 55.85),
            ("IRON", 55.85),
            ("CU", 63.55),
            ("COPPER", 63.55),
            ("ZN", 65.39),
            ("ZINC", 65.39),
            ("AG", 107.87),
            ("SILVER", 107.87),
            ("AU", 196.97),
            ("GOLD", 196.97),
            ("HG", 200.59),
            ("MERCURY", 200.59),
            ("PB", 207.2),
            ("LEAD", 207.2),
            ("U", 238.03),
            ("URANIUM", 238.03),
        ]
        .iter()
        .cloned()
        .collect();

        masses.get(element.to_uppercase().as_str()).copied()
    }
}

impl SqlFunction for AtomicMassFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "ATOMIC_MASS",
            category: FunctionCategory::Chemical,
            arg_count: ArgCount::Fixed(1),
            description: "Returns the atomic mass of an element in amu",
            returns: "FLOAT",
            examples: vec![
                "SELECT ATOMIC_MASS('H')",
                "SELECT ATOMIC_MASS('Carbon')",
                "SELECT ATOMIC_MASS('Au') AS gold_mass",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        match &args[0] {
            DataValue::String(element) => match Self::get_atomic_mass(element) {
                Some(mass) => Ok(DataValue::Float(mass)),
                None => Err(anyhow!("Unknown element: {}", element)),
            },
            DataValue::InternedString(element) => match Self::get_atomic_mass(element) {
                Some(mass) => Ok(DataValue::Float(mass)),
                None => Err(anyhow!("Unknown element: {}", element)),
            },
            _ => Err(anyhow!("ATOMIC_MASS() requires a string argument")),
        }
    }
}

/// Future: Atomic number function
pub struct AtomicNumberFunction;

impl AtomicNumberFunction {
    fn get_atomic_number(element: &str) -> Option<i64> {
        let numbers: HashMap<&str, i64> = [
            ("H", 1),
            ("HYDROGEN", 1),
            ("HE", 2),
            ("HELIUM", 2),
            ("LI", 3),
            ("LITHIUM", 3),
            ("BE", 4),
            ("BERYLLIUM", 4),
            ("B", 5),
            ("BORON", 5),
            ("C", 6),
            ("CARBON", 6),
            ("N", 7),
            ("NITROGEN", 7),
            ("O", 8),
            ("OXYGEN", 8),
            ("F", 9),
            ("FLUORINE", 9),
            ("NE", 10),
            ("NEON", 10),
            ("NA", 11),
            ("SODIUM", 11),
            ("MG", 12),
            ("MAGNESIUM", 12),
            ("AL", 13),
            ("ALUMINUM", 13),
            ("ALUMINIUM", 13),
            ("SI", 14),
            ("SILICON", 14),
            ("P", 15),
            ("PHOSPHORUS", 15),
            ("S", 16),
            ("SULFUR", 16),
            ("SULPHUR", 16),
            ("CL", 17),
            ("CHLORINE", 17),
            ("AR", 18),
            ("ARGON", 18),
            ("K", 19),
            ("POTASSIUM", 19),
            ("CA", 20),
            ("CALCIUM", 20),
            // Common elements
            ("FE", 26),
            ("IRON", 26),
            ("CU", 29),
            ("COPPER", 29),
            ("ZN", 30),
            ("ZINC", 30),
            ("AG", 47),
            ("SILVER", 47),
            ("AU", 79),
            ("GOLD", 79),
            ("HG", 80),
            ("MERCURY", 80),
            ("PB", 82),
            ("LEAD", 82),
            ("U", 92),
            ("URANIUM", 92),
        ]
        .iter()
        .cloned()
        .collect();

        numbers.get(element.to_uppercase().as_str()).copied()
    }
}

impl SqlFunction for AtomicNumberFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "ATOMIC_NUMBER",
            category: FunctionCategory::Chemical,
            arg_count: ArgCount::Fixed(1),
            description: "Returns the atomic number of an element",
            returns: "INTEGER",
            examples: vec![
                "SELECT ATOMIC_NUMBER('H')",
                "SELECT ATOMIC_NUMBER('Carbon')",
                "SELECT ATOMIC_NUMBER('Au') AS gold_number",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        match &args[0] {
            DataValue::String(element) => match Self::get_atomic_number(element) {
                Some(number) => Ok(DataValue::Integer(number)),
                None => Err(anyhow!("Unknown element: {}", element)),
            },
            DataValue::InternedString(element) => match Self::get_atomic_number(element) {
                Some(number) => Ok(DataValue::Integer(number)),
                None => Err(anyhow!("Unknown element: {}", element)),
            },
            _ => Err(anyhow!("ATOMIC_NUMBER() requires a string argument")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_avogadro() {
        let func = AvogadroFunction;
        let result = func.evaluate(&[]).unwrap();
        match result {
            DataValue::Float(val) => assert!((val - 6.022140857e23).abs() < 1e20),
            _ => panic!("Expected Float"),
        }
    }

    #[test]
    fn test_atomic_mass_hydrogen() {
        let func = AtomicMassFunction;
        let result = func
            .evaluate(&[DataValue::String("H".to_string())])
            .unwrap();
        match result {
            DataValue::Float(val) => assert!((val - 1.008).abs() < 0.001),
            _ => panic!("Expected Float"),
        }
    }

    #[test]
    fn test_atomic_mass_carbon() {
        let func = AtomicMassFunction;
        let result = func
            .evaluate(&[DataValue::String("Carbon".to_string())])
            .unwrap();
        match result {
            DataValue::Float(val) => assert!((val - 12.01).abs() < 0.01),
            _ => panic!("Expected Float"),
        }
    }

    #[test]
    fn test_atomic_mass_gold() {
        let func = AtomicMassFunction;
        let result = func
            .evaluate(&[DataValue::String("Au".to_string())])
            .unwrap();
        match result {
            DataValue::Float(val) => assert!((val - 196.97).abs() < 0.01),
            _ => panic!("Expected Float"),
        }
    }

    #[test]
    fn test_atomic_mass_unknown_element() {
        let func = AtomicMassFunction;
        let result = func.evaluate(&[DataValue::String("Xyz".to_string())]);
        assert!(result.is_err());
    }

    #[test]
    fn test_atomic_number_carbon() {
        let func = AtomicNumberFunction;
        let result = func
            .evaluate(&[DataValue::String("C".to_string())])
            .unwrap();
        match result {
            DataValue::Integer(val) => assert_eq!(val, 6),
            _ => panic!("Expected Integer"),
        }
    }
}

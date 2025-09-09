use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use std::collections::HashMap;

use super::{ArgCount, FunctionCategory, FunctionSignature, SqlFunction};
use crate::data::datatable::DataValue;

/// Represents a known molecule with its formula and common names
#[derive(Debug, Clone)]
struct Molecule {
    formula: &'static str,
    names: &'static [&'static str],
    category: &'static str,
}

lazy_static! {
    /// Table of known molecules and their formulas
    static ref MOLECULE_TABLE: Vec<Molecule> = vec![
        // Water and related
        Molecule {
            formula: "H2O",
            names: &["WATER"],
            category: "Inorganic"
        },
        Molecule {
            formula: "H2O2",
            names: &["HYDROGEN PEROXIDE"],
            category: "Inorganic"
        },

        // Common gases
        Molecule {
            formula: "NH3",
            names: &["AMMONIA"],
            category: "Inorganic"
        },
        Molecule {
            formula: "CO2",
            names: &["CARBON DIOXIDE", "CO2"],
            category: "Inorganic"
        },
        Molecule {
            formula: "CO",
            names: &["CARBON MONOXIDE", "CO"],
            category: "Inorganic"
        },
        Molecule {
            formula: "O2",
            names: &["OXYGEN", "DIOXYGEN"],
            category: "Inorganic"
        },
        Molecule {
            formula: "N2",
            names: &["NITROGEN", "DINITROGEN"],
            category: "Inorganic"
        },
        Molecule {
            formula: "O3",
            names: &["OZONE"],
            category: "Inorganic"
        },

        // Hydrocarbons
        Molecule {
            formula: "CH4",
            names: &["METHANE"],
            category: "Hydrocarbon"
        },
        Molecule {
            formula: "C2H6",
            names: &["ETHANE"],
            category: "Hydrocarbon"
        },
        Molecule {
            formula: "C3H8",
            names: &["PROPANE"],
            category: "Hydrocarbon"
        },
        Molecule {
            formula: "C4H10",
            names: &["BUTANE"],
            category: "Hydrocarbon"
        },
        Molecule {
            formula: "C5H12",
            names: &["PENTANE"],
            category: "Hydrocarbon"
        },
        Molecule {
            formula: "C6H14",
            names: &["HEXANE"],
            category: "Hydrocarbon"
        },
        Molecule {
            formula: "C2H4",
            names: &["ETHENE", "ETHYLENE"],
            category: "Hydrocarbon"
        },
        Molecule {
            formula: "C2H2",
            names: &["ETHYNE", "ACETYLENE"],
            category: "Hydrocarbon"
        },
        Molecule {
            formula: "C6H6",
            names: &["BENZENE"],
            category: "Hydrocarbon"
        },

        // Sugars
        Molecule {
            formula: "C6H12O6",
            names: &["GLUCOSE", "DEXTROSE"],
            category: "Sugar"
        },
        Molecule {
            formula: "C6H12O6",
            names: &["FRUCTOSE"],
            category: "Sugar"
        },
        Molecule {
            formula: "C12H22O11",
            names: &["SUCROSE", "TABLE SUGAR"],
            category: "Sugar"
        },
        Molecule {
            formula: "C12H22O11",
            names: &["LACTOSE", "MILK SUGAR"],
            category: "Sugar"
        },

        // Salts and minerals
        Molecule {
            formula: "NaCl",
            names: &["SALT", "TABLE SALT", "SODIUM CHLORIDE"],
            category: "Salt"
        },
        Molecule {
            formula: "NaHCO3",
            names: &["BAKING SODA", "SODIUM BICARBONATE"],
            category: "Salt"
        },
        Molecule {
            formula: "CaCO3",
            names: &["CALCIUM CARBONATE", "LIMESTONE", "CHALK"],
            category: "Mineral"
        },
        Molecule {
            formula: "CaSO4",
            names: &["CALCIUM SULFATE", "GYPSUM"],
            category: "Mineral"
        },

        // Acids
        Molecule {
            formula: "HCl",
            names: &["HYDROCHLORIC ACID"],
            category: "Acid"
        },
        Molecule {
            formula: "H2SO4",
            names: &["SULFURIC ACID"],
            category: "Acid"
        },
        Molecule {
            formula: "HNO3",
            names: &["NITRIC ACID"],
            category: "Acid"
        },
        Molecule {
            formula: "H3PO4",
            names: &["PHOSPHORIC ACID"],
            category: "Acid"
        },
        Molecule {
            formula: "CH3COOH",
            names: &["ACETIC ACID", "VINEGAR"],
            category: "Acid"
        },

        // Organic compounds
        Molecule {
            formula: "C2H5OH",
            names: &["ETHANOL", "ALCOHOL", "ETHYL ALCOHOL"],
            category: "Alcohol"
        },
        Molecule {
            formula: "CH3OH",
            names: &["METHANOL", "METHYL ALCOHOL"],
            category: "Alcohol"
        },
        Molecule {
            formula: "C3H8O",
            names: &["ISOPROPANOL", "ISOPROPYL ALCOHOL", "RUBBING ALCOHOL"],
            category: "Alcohol"
        },
        Molecule {
            formula: "CH3COCH3",
            names: &["ACETONE"],
            category: "Organic"
        },
        Molecule {
            formula: "C8H10N4O2",
            names: &["CAFFEINE"],
            category: "Organic"
        },
        Molecule {
            formula: "C9H8O4",
            names: &["ASPIRIN", "ACETYLSALICYLIC ACID"],
            category: "Organic"
        },

        // Vitamins
        Molecule {
            formula: "C6H8O6",
            names: &["VITAMIN C", "ASCORBIC ACID"],
            category: "Vitamin"
        },
    ];

    /// HashMap for fast lookup of formulas by name
    static ref MOLECULE_LOOKUP: HashMap<String, &'static str> = {
        let mut map = HashMap::new();
        for molecule in MOLECULE_TABLE.iter() {
            for name in molecule.names {
                map.insert((*name).to_string(), molecule.formula);
            }
        }
        map
    };
}

/// Represents a parsed molecular formula
#[derive(Debug, Clone)]
struct MolecularFormula {
    elements: Vec<(String, usize)>, // (element symbol, count)
}

impl MolecularFormula {
    /// Parse a molecular formula like "H2O", "Ca(OH)2", "C6H12O6"
    fn parse(formula: &str) -> Result<Self> {
        let formula = formula.trim();

        // Check for common compound aliases first
        if let Some(expanded) = Self::get_compound_alias(formula) {
            return Self::parse_formula(expanded);
        }

        Self::parse_formula(formula)
    }

    /// Get common compound aliases from the molecule table
    fn get_compound_alias(name: &str) -> Option<&'static str> {
        let name_upper = name.to_uppercase();
        MOLECULE_LOOKUP.get(&name_upper).copied()
    }

    /// Parse the actual formula string
    fn parse_formula(formula: &str) -> Result<Self> {
        let mut elements = Vec::new();
        let mut chars = formula.chars().peekable();

        while chars.peek().is_some() {
            // Handle parentheses groups like (OH)2
            if chars.peek() == Some(&'(') {
                chars.next(); // consume '('
                let mut group = String::new();
                let mut depth = 1;

                for ch in chars.by_ref() {
                    if ch == '(' {
                        depth += 1;
                        group.push(ch);
                    } else if ch == ')' {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                        group.push(ch);
                    } else {
                        group.push(ch);
                    }
                }

                // Get multiplier for the group
                let multiplier = Self::parse_number(&mut chars).unwrap_or(1);

                // Recursively parse the group
                let group_formula = Self::parse_formula(&group)?;
                for (elem, count) in group_formula.elements {
                    elements.push((elem, count * multiplier));
                }
            } else {
                // Parse element symbol
                let element = Self::parse_element(&mut chars)?;
                let count = Self::parse_number(&mut chars).unwrap_or(1);

                // Merge with existing element if present
                if let Some((_, existing_count)) = elements.iter_mut().find(|(e, _)| e == &element)
                {
                    *existing_count += count;
                } else {
                    elements.push((element, count));
                }
            }
        }

        Ok(MolecularFormula { elements })
    }

    /// Parse an element symbol (1 uppercase + optional lowercase letters)
    fn parse_element(chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<String> {
        let mut element = String::new();

        // First character must be uppercase
        if let Some(ch) = chars.peek() {
            if ch.is_uppercase() {
                element.push(chars.next().unwrap());
            } else {
                return Err(anyhow!("Expected uppercase letter for element symbol"));
            }
        } else {
            return Err(anyhow!("Unexpected end of formula"));
        }

        // Following characters are lowercase
        while let Some(&ch) = chars.peek() {
            if ch.is_lowercase() {
                element.push(chars.next().unwrap());
            } else {
                break;
            }
        }

        Ok(element)
    }

    /// Parse a number from the formula
    fn parse_number(chars: &mut std::iter::Peekable<std::str::Chars>) -> Option<usize> {
        let mut num_str = String::new();

        while let Some(&ch) = chars.peek() {
            if ch.is_ascii_digit() {
                num_str.push(chars.next().unwrap());
            } else {
                break;
            }
        }

        if num_str.is_empty() {
            None
        } else {
            num_str.parse().ok()
        }
    }

    /// Calculate the total molecular mass
    fn calculate_mass(&self) -> Result<f64> {
        let mut total_mass = 0.0;

        for (element, count) in &self.elements {
            let atomic_mass = AtomicMassFunction::get_atomic_mass(element)
                .ok_or_else(|| anyhow!("Unknown element: {}", element))?;
            total_mass += atomic_mass * (*count as f64);
        }

        Ok(total_mass)
    }
}

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
        Ok(DataValue::Float(6.02214076e23))
    }
}

/// Atomic mass function - returns atomic mass for an element
pub struct AtomicMassFunction;

impl AtomicMassFunction {
    fn get_atomic_mass(element: &str) -> Option<f64> {
        let masses: HashMap<&str, f64> = [
            // First 20 elements - uppercase and proper case for formula parsing
            ("H", 1.008),
            ("HYDROGEN", 1.008),
            ("He", 4.003),
            ("HE", 4.003),
            ("HELIUM", 4.003),
            ("Li", 6.941),
            ("LI", 6.941),
            ("LITHIUM", 6.941),
            ("Be", 9.012),
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
            ("Ne", 20.18),
            ("NE", 20.18),
            ("NEON", 20.18),
            ("Na", 22.99),
            ("NA", 22.99),
            ("SODIUM", 22.99),
            ("Mg", 24.31),
            ("MG", 24.31),
            ("MAGNESIUM", 24.31),
            ("Al", 26.98),
            ("AL", 26.98),
            ("ALUMINUM", 26.98),
            ("ALUMINIUM", 26.98),
            ("Si", 28.09),
            ("SI", 28.09),
            ("SILICON", 28.09),
            ("P", 30.97),
            ("PHOSPHORUS", 30.97),
            ("S", 32.07),
            ("SULFUR", 32.07),
            ("SULPHUR", 32.07),
            ("Cl", 35.45),
            ("CL", 35.45),
            ("CHLORINE", 35.45),
            ("Ar", 39.95),
            ("AR", 39.95),
            ("ARGON", 39.95),
            ("K", 39.10),
            ("POTASSIUM", 39.10),
            ("Ca", 40.08),
            ("CA", 40.08),
            ("CALCIUM", 40.08),
            // Common elements beyond first 20
            ("FE", 55.85),
            ("Fe", 55.85),
            ("IRON", 55.85),
            ("CU", 63.55),
            ("Cu", 63.55),
            ("COPPER", 63.55),
            ("Zn", 65.39),
            ("ZN", 65.39),
            ("ZINC", 65.39),
            ("Ag", 107.87),
            ("AG", 107.87),
            ("SILVER", 107.87),
            ("Au", 196.97),
            ("AU", 196.97),
            ("GOLD", 196.97),
            ("Hg", 200.59),
            ("HG", 200.59),
            ("MERCURY", 200.59),
            ("Pb", 207.2),
            ("PB", 207.2),
            ("LEAD", 207.2),
            ("U", 238.03),
            ("URANIUM", 238.03),
        ]
        .iter()
        .copied()
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
            description: "Returns the atomic mass of an element or molecular formula in amu",
            returns: "FLOAT",
            examples: vec![
                "SELECT ATOMIC_MASS('H')",
                "SELECT ATOMIC_MASS('Carbon')",
                "SELECT ATOMIC_MASS('H2O') AS water_mass",
                "SELECT ATOMIC_MASS('Ca(OH)2') AS calcium_hydroxide",
                "SELECT ATOMIC_MASS('water') AS water_mass",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        match &args[0] {
            DataValue::String(input) => {
                // First try as a simple element
                if let Some(mass) = Self::get_atomic_mass(input) {
                    return Ok(DataValue::Float(mass));
                }

                // Try parsing as molecular formula
                match MolecularFormula::parse(input) {
                    Ok(formula) => {
                        let mass = formula.calculate_mass()?;
                        Ok(DataValue::Float(mass))
                    }
                    Err(_) => Err(anyhow!(
                        "Unknown element or invalid molecular formula: {}",
                        input
                    )),
                }
            }
            DataValue::InternedString(input) => {
                // First try as a simple element
                if let Some(mass) = Self::get_atomic_mass(input) {
                    return Ok(DataValue::Float(mass));
                }

                // Try parsing as molecular formula
                match MolecularFormula::parse(input) {
                    Ok(formula) => {
                        let mass = formula.calculate_mass()?;
                        Ok(DataValue::Float(mass))
                    }
                    Err(_) => Err(anyhow!(
                        "Unknown element or invalid molecular formula: {}",
                        input
                    )),
                }
            }
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
        .copied()
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

/// Returns the number of neutrons in the most common isotope of an element
pub struct NeutronsFunction;

impl SqlFunction for NeutronsFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "NEUTRONS",
            category: FunctionCategory::Chemical,
            arg_count: ArgCount::Fixed(1),
            description: "Returns the number of neutrons in the most common isotope",
            returns: "INTEGER",
            examples: vec![
                "SELECT NEUTRONS('C')",      // Carbon-12 has 6 neutrons
                "SELECT NEUTRONS('U')",      // Uranium-238 has 146 neutrons
                "SELECT NEUTRONS('Gold')",   // Gold-197 has 118 neutrons
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let element = match &args[0] {
            DataValue::String(s) => s.as_str(),
            DataValue::InternedString(s) => s.as_str(),
            _ => return Err(anyhow!("NEUTRONS() requires a string argument")),
        };

        // Get atomic number (protons)
        let protons = AtomicNumberFunction::get_atomic_number(element)
            .ok_or_else(|| anyhow!("Unknown element: {}", element))?;

        // Get atomic mass
        let atomic_mass = AtomicMassFunction::get_atomic_mass(element)
            .ok_or_else(|| anyhow!("Unknown element: {}", element))?;

        // Mass number is the rounded atomic mass (most common isotope)
        let mass_number = atomic_mass.round() as i64;

        // Neutrons = mass_number - protons
        let neutrons = mass_number - protons;

        Ok(DataValue::Integer(neutrons))
    }
}

/// Returns the molecular formula for a given compound name
pub struct MoleculeFormulaFunction;

impl SqlFunction for MoleculeFormulaFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "MOLECULE_FORMULA",
            category: FunctionCategory::Chemical,
            arg_count: ArgCount::Fixed(1),
            description: "Returns the molecular formula for a compound name",
            returns: "STRING",
            examples: vec![
                "SELECT MOLECULE_FORMULA('water')",
                "SELECT MOLECULE_FORMULA('glucose')",
                "SELECT MOLECULE_FORMULA('caffeine')",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let input = match &args[0] {
            DataValue::String(s) => s.clone(),
            DataValue::InternedString(s) => s.to_string(),
            _ => return Err(anyhow!("MOLECULE_FORMULA expects a string")),
        };

        let upper_input = input.to_uppercase();

        // Check if it's a known molecule name
        if let Some(formula) = MOLECULE_LOOKUP.get(&upper_input) {
            return Ok(DataValue::String((*formula).to_string()));
        }

        // If not found, return an error
        Err(anyhow!("Unknown molecule: {}", input))
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

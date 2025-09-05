use anyhow::Result;

use super::{ArgCount, FunctionCategory, FunctionSignature, SqlFunction};
use crate::data::datatable::DataValue;

/// Speed of light in vacuum
pub struct SpeedOfLightFunction;

impl SqlFunction for SpeedOfLightFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "C",
            category: FunctionCategory::Constant,
            arg_count: ArgCount::Fixed(0),
            description: "Returns the speed of light in vacuum (m/s)",
            returns: "FLOAT",
            examples: vec!["SELECT C()", "SELECT distance / C() as light_time"],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        Ok(DataValue::Float(299792458.0))
    }
}

/// Gravitational constant
pub struct GravitationalConstantFunction;

impl SqlFunction for GravitationalConstantFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "G",
            category: FunctionCategory::Constant,
            arg_count: ArgCount::Fixed(0),
            description: "Returns the gravitational constant (m³ kg⁻¹ s⁻²)",
            returns: "FLOAT",
            examples: vec![
                "SELECT G()",
                "SELECT G() * mass1 * mass2 / (r * r) as force",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        Ok(DataValue::Float(6.67430e-11))
    }
}

/// Planck constant
pub struct PlanckFunction;

impl SqlFunction for PlanckFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "H",
            category: FunctionCategory::Constant,
            arg_count: ArgCount::Fixed(0),
            description: "Returns the Planck constant (J⋅s)",
            returns: "FLOAT",
            examples: vec!["SELECT H()", "SELECT H() * frequency as energy"],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        Ok(DataValue::Float(6.62607015e-34))
    }
}

/// Boltzmann constant
pub struct BoltzmannFunction;

impl SqlFunction for BoltzmannFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "K",
            category: FunctionCategory::Constant,
            arg_count: ArgCount::Fixed(0),
            description: "Returns the Boltzmann constant (J/K)",
            returns: "FLOAT",
            examples: vec!["SELECT K()", "SELECT K() * temperature as thermal_energy"],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        Ok(DataValue::Float(1.380649e-23))
    }
}

/// Universal gas constant
pub struct GasConstantFunction;

impl SqlFunction for GasConstantFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "R",
            category: FunctionCategory::Constant,
            arg_count: ArgCount::Fixed(0),
            description: "Returns the universal gas constant (J mol⁻¹ K⁻¹)",
            returns: "FLOAT",
            examples: vec![
                "SELECT R()",
                "SELECT pressure * volume / (R() * temperature) as moles",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        Ok(DataValue::Float(8.314462618))
    }
}

/// Electric permittivity of vacuum
pub struct PermittivityFunction;

impl SqlFunction for PermittivityFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "E0",
            category: FunctionCategory::Constant,
            arg_count: ArgCount::Fixed(0),
            description: "Returns the electric permittivity of vacuum (F/m)",
            returns: "FLOAT",
            examples: vec![
                "SELECT E0()",
                "SELECT 1 / (4 * PI() * E0()) as coulomb_constant",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        Ok(DataValue::Float(8.8541878128e-12))
    }
}

/// Magnetic permeability of vacuum
pub struct PermeabilityFunction;

impl SqlFunction for PermeabilityFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "MU0",
            category: FunctionCategory::Constant,
            arg_count: ArgCount::Fixed(0),
            description: "Returns the magnetic permeability of vacuum (N/A²)",
            returns: "FLOAT",
            examples: vec!["SELECT MU0()", "SELECT SQRT(MU0() * E0()) as impedance"],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        Ok(DataValue::Float(1.25663706212e-6))
    }
}

/// Elementary charge
pub struct ElementaryChargeFunction;

impl SqlFunction for ElementaryChargeFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "QE",
            category: FunctionCategory::Constant,
            arg_count: ArgCount::Fixed(0),
            description: "Returns the elementary charge (C)",
            returns: "FLOAT",
            examples: vec!["SELECT QE()", "SELECT n_electrons * QE() as total_charge"],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        Ok(DataValue::Float(1.602176634e-19))
    }
}

/// Proton mass
pub struct ProtonMassFunction;

impl SqlFunction for ProtonMassFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "MP",
            category: FunctionCategory::Constant,
            arg_count: ArgCount::Fixed(0),
            description: "Returns the proton mass (kg)",
            returns: "FLOAT",
            examples: vec!["SELECT MP()", "SELECT mass / MP() as proton_masses"],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        Ok(DataValue::Float(1.67262192369e-27))
    }
}

/// Neutron mass
pub struct NeutronMassFunction;

impl SqlFunction for NeutronMassFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "MN",
            category: FunctionCategory::Constant,
            arg_count: ArgCount::Fixed(0),
            description: "Returns the neutron mass (kg)",
            returns: "FLOAT",
            examples: vec!["SELECT MN()", "SELECT mass / MN() as neutron_masses"],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        Ok(DataValue::Float(1.67492749804e-27))
    }
}

/// Atomic mass unit
pub struct AtomicMassUnitFunction;

impl SqlFunction for AtomicMassUnitFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "AMU",
            category: FunctionCategory::Constant,
            arg_count: ArgCount::Fixed(0),
            description: "Returns the atomic mass unit (kg)",
            returns: "FLOAT",
            examples: vec!["SELECT AMU()", "SELECT molecular_weight * AMU() as mass_kg"],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        Ok(DataValue::Float(1.66053906660e-27))
    }
}

/// Fine structure constant
pub struct FineStructureFunction;

impl SqlFunction for FineStructureFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "ALPHA",
            category: FunctionCategory::Constant,
            arg_count: ArgCount::Fixed(0),
            description: "Returns the fine structure constant (dimensionless)",
            returns: "FLOAT",
            examples: vec![
                "SELECT ALPHA()",
                "SELECT 1 / ALPHA() as inverse_fine_structure",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        Ok(DataValue::Float(7.2973525693e-3))
    }
}

/// Rydberg constant
pub struct RydbergFunction;

impl SqlFunction for RydbergFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "RY",
            category: FunctionCategory::Constant,
            arg_count: ArgCount::Fixed(0),
            description: "Returns the Rydberg constant (m⁻¹)",
            returns: "FLOAT",
            examples: vec!["SELECT RY()", "SELECT 1 / RY() as rydberg_wavelength"],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        Ok(DataValue::Float(10973731.568160))
    }
}

/// Stefan-Boltzmann constant
pub struct StefanBoltzmannFunction;

impl SqlFunction for StefanBoltzmannFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "SIGMA",
            category: FunctionCategory::Constant,
            arg_count: ArgCount::Fixed(0),
            description: "Returns the Stefan-Boltzmann constant (W m⁻² K⁻⁴)",
            returns: "FLOAT",
            examples: vec![
                "SELECT SIGMA()",
                "SELECT SIGMA() * area * POWER(temp, 4) as radiated_power",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        Ok(DataValue::Float(5.670374419e-8))
    }
}

/// Classical electron radius
pub struct ElectronRadiusFunction;

impl SqlFunction for ElectronRadiusFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "RE",
            category: FunctionCategory::Constant,
            arg_count: ArgCount::Fixed(0),
            description: "Returns the classical electron radius (m)",
            returns: "FLOAT",
            examples: vec!["SELECT RE()", "SELECT radius / RE() as electron_radii"],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        Ok(DataValue::Float(2.8179403262e-15))
    }
}

/// Proton radius
pub struct ProtonRadiusFunction;

impl SqlFunction for ProtonRadiusFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "RP",
            category: FunctionCategory::Constant,
            arg_count: ArgCount::Fixed(0),
            description: "Returns the proton radius (m)",
            returns: "FLOAT",
            examples: vec!["SELECT RP()", "SELECT radius / RP() as proton_radii"],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        Ok(DataValue::Float(8.414e-16))
    }
}

/// Neutron radius
pub struct NeutronRadiusFunction;

impl SqlFunction for NeutronRadiusFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "RN",
            category: FunctionCategory::Constant,
            arg_count: ArgCount::Fixed(0),
            description: "Returns the neutron radius (m)",
            returns: "FLOAT",
            examples: vec!["SELECT RN()", "SELECT radius / RN() as neutron_radii"],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        Ok(DataValue::Float(8.4e-16))
    }
}

/// Register all physics functions
pub fn register_physics_functions(registry: &mut super::FunctionRegistry) {
    // Fundamental constants
    registry.register(Box::new(SpeedOfLightFunction));
    registry.register(Box::new(GravitationalConstantFunction));
    registry.register(Box::new(PlanckFunction));
    registry.register(Box::new(BoltzmannFunction));
    registry.register(Box::new(GasConstantFunction));

    // Electromagnetic constants
    registry.register(Box::new(PermittivityFunction));
    registry.register(Box::new(PermeabilityFunction));
    registry.register(Box::new(ElementaryChargeFunction));

    // Particle masses
    registry.register(Box::new(ProtonMassFunction));
    registry.register(Box::new(NeutronMassFunction));
    registry.register(Box::new(AtomicMassUnitFunction));

    // Other physics constants
    registry.register(Box::new(FineStructureFunction));
    registry.register(Box::new(RydbergFunction));
    registry.register(Box::new(StefanBoltzmannFunction));

    // Particle radii
    registry.register(Box::new(ElectronRadiusFunction));
    registry.register(Box::new(ProtonRadiusFunction));
    registry.register(Box::new(NeutronRadiusFunction));
}

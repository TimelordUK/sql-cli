use anyhow::Result;

use super::{ArgCount, FunctionCategory, FunctionSignature, SqlFunction};
use crate::data::datatable::DataValue;

// Solar System Masses (in kg)

/// Earth mass function
pub struct MassEarthFunction;

impl SqlFunction for MassEarthFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "MASS_EARTH",
            category: FunctionCategory::Astronomical,
            arg_count: ArgCount::Fixed(0),
            description: "Returns Earth's mass in kg (5.972 × 10^24)",
            returns: "FLOAT",
            examples: vec![
                "SELECT MASS_EARTH()",
                "SELECT asteroid_mass / MASS_EARTH() AS earth_masses",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        Ok(DataValue::Float(5.972e24))
    }
}

/// Sun mass function
pub struct MassSunFunction;

impl SqlFunction for MassSunFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "MASS_SUN",
            category: FunctionCategory::Astronomical,
            arg_count: ArgCount::Fixed(0),
            description: "Returns the Sun's mass in kg (1.989 × 10^30)",
            returns: "FLOAT",
            examples: vec![
                "SELECT MASS_SUN()",
                "SELECT star_mass / MASS_SUN() AS solar_masses",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        Ok(DataValue::Float(1.989e30))
    }
}

/// Moon mass function
pub struct MassMoonFunction;

impl SqlFunction for MassMoonFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "MASS_MOON",
            category: FunctionCategory::Astronomical,
            arg_count: ArgCount::Fixed(0),
            description: "Returns the Moon's mass in kg (7.342 × 10^22)",
            returns: "FLOAT",
            examples: vec![
                "SELECT MASS_MOON()",
                "SELECT satellite_mass / MASS_MOON() AS lunar_masses",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        Ok(DataValue::Float(7.342e22))
    }
}

// Planetary Masses

/// Mercury mass function
pub struct MassMercuryFunction;

impl SqlFunction for MassMercuryFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "MASS_MERCURY",
            category: FunctionCategory::Astronomical,
            arg_count: ArgCount::Fixed(0),
            description: "Returns Mercury's mass in kg (3.301 × 10^23)",
            returns: "FLOAT",
            examples: vec!["SELECT MASS_MERCURY()"],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        Ok(DataValue::Float(3.301e23))
    }
}

/// Venus mass function
pub struct MassVenusFunction;

impl SqlFunction for MassVenusFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "MASS_VENUS",
            category: FunctionCategory::Astronomical,
            arg_count: ArgCount::Fixed(0),
            description: "Returns Venus's mass in kg (4.867 × 10^24)",
            returns: "FLOAT",
            examples: vec!["SELECT MASS_VENUS()"],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        Ok(DataValue::Float(4.867e24))
    }
}

/// Mars mass function
pub struct MassMarsFunction;

impl SqlFunction for MassMarsFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "MASS_MARS",
            category: FunctionCategory::Astronomical,
            arg_count: ArgCount::Fixed(0),
            description: "Returns Mars's mass in kg (6.417 × 10^23)",
            returns: "FLOAT",
            examples: vec!["SELECT MASS_MARS()"],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        Ok(DataValue::Float(6.417e23))
    }
}

/// Jupiter mass function
pub struct MassJupiterFunction;

impl SqlFunction for MassJupiterFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "MASS_JUPITER",
            category: FunctionCategory::Astronomical,
            arg_count: ArgCount::Fixed(0),
            description: "Returns Jupiter's mass in kg (1.898 × 10^27)",
            returns: "FLOAT",
            examples: vec![
                "SELECT MASS_JUPITER()",
                "SELECT exoplanet_mass / MASS_JUPITER() AS jupiter_masses",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        Ok(DataValue::Float(1.898e27))
    }
}

/// Saturn mass function
pub struct MassSaturnFunction;

impl SqlFunction for MassSaturnFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "MASS_SATURN",
            category: FunctionCategory::Astronomical,
            arg_count: ArgCount::Fixed(0),
            description: "Returns Saturn's mass in kg (5.683 × 10^26)",
            returns: "FLOAT",
            examples: vec!["SELECT MASS_SATURN()"],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        Ok(DataValue::Float(5.683e26))
    }
}

/// Uranus mass function
pub struct MassUranusFunction;

impl SqlFunction for MassUranusFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "MASS_URANUS",
            category: FunctionCategory::Astronomical,
            arg_count: ArgCount::Fixed(0),
            description: "Returns Uranus's mass in kg (8.681 × 10^25)",
            returns: "FLOAT",
            examples: vec!["SELECT MASS_URANUS()"],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        Ok(DataValue::Float(8.681e25))
    }
}

/// Neptune mass function
pub struct MassNeptuneFunction;

impl SqlFunction for MassNeptuneFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "MASS_NEPTUNE",
            category: FunctionCategory::Astronomical,
            arg_count: ArgCount::Fixed(0),
            description: "Returns Neptune's mass in kg (1.024 × 10^26)",
            returns: "FLOAT",
            examples: vec!["SELECT MASS_NEPTUNE()"],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        Ok(DataValue::Float(1.024e26))
    }
}

// Radius Functions

/// Sun radius function
pub struct RadiusSunFunction;

impl SqlFunction for RadiusSunFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "RADIUS_SUN",
            category: FunctionCategory::Astronomical,
            arg_count: ArgCount::Fixed(0),
            description: "Returns the Sun's radius in meters (6.96 × 10^8)",
            returns: "FLOAT",
            examples: vec![
                "SELECT RADIUS_SUN()",
                "SELECT star_radius / RADIUS_SUN() AS solar_radii",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        Ok(DataValue::Float(6.96e8))
    }
}

/// Earth radius function
pub struct RadiusEarthFunction;

impl SqlFunction for RadiusEarthFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "RADIUS_EARTH",
            category: FunctionCategory::Astronomical,
            arg_count: ArgCount::Fixed(0),
            description: "Returns Earth's radius in meters (6.371 × 10^6)",
            returns: "FLOAT",
            examples: vec![
                "SELECT RADIUS_EARTH()",
                "SELECT planet_radius / RADIUS_EARTH() AS earth_radii",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        Ok(DataValue::Float(6.371e6))
    }
}

/// Moon radius function
pub struct RadiusMoonFunction;

impl SqlFunction for RadiusMoonFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "RADIUS_MOON",
            category: FunctionCategory::Astronomical,
            arg_count: ArgCount::Fixed(0),
            description: "Returns the Moon's radius in meters (1.737 × 10^6)",
            returns: "FLOAT",
            examples: vec![
                "SELECT RADIUS_MOON()",
                "SELECT satellite_radius / RADIUS_MOON() AS lunar_radii",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        Ok(DataValue::Float(1.737e6))
    }
}

/// Mercury radius function
pub struct RadiusMercuryFunction;

impl SqlFunction for RadiusMercuryFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "RADIUS_MERCURY",
            category: FunctionCategory::Astronomical,
            arg_count: ArgCount::Fixed(0),
            description: "Returns Mercury's radius in meters (2.440 × 10^6)",
            returns: "FLOAT",
            examples: vec!["SELECT RADIUS_MERCURY()"],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        Ok(DataValue::Float(2.440e6))
    }
}

/// Venus radius function
pub struct RadiusVenusFunction;

impl SqlFunction for RadiusVenusFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "RADIUS_VENUS",
            category: FunctionCategory::Astronomical,
            arg_count: ArgCount::Fixed(0),
            description: "Returns Venus's radius in meters (6.052 × 10^6)",
            returns: "FLOAT",
            examples: vec!["SELECT RADIUS_VENUS()"],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        Ok(DataValue::Float(6.052e6))
    }
}

/// Mars radius function
pub struct RadiusMarsFunction;

impl SqlFunction for RadiusMarsFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "RADIUS_MARS",
            category: FunctionCategory::Astronomical,
            arg_count: ArgCount::Fixed(0),
            description: "Returns Mars's radius in meters (3.390 × 10^6)",
            returns: "FLOAT",
            examples: vec!["SELECT RADIUS_MARS()"],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        Ok(DataValue::Float(3.390e6))
    }
}

/// Jupiter radius function
pub struct RadiusJupiterFunction;

impl SqlFunction for RadiusJupiterFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "RADIUS_JUPITER",
            category: FunctionCategory::Astronomical,
            arg_count: ArgCount::Fixed(0),
            description: "Returns Jupiter's radius in meters (6.991 × 10^7)",
            returns: "FLOAT",
            examples: vec![
                "SELECT RADIUS_JUPITER()",
                "SELECT exoplanet_radius / RADIUS_JUPITER() AS jupiter_radii",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        Ok(DataValue::Float(6.991e7))
    }
}

/// Saturn radius function
pub struct RadiusSaturnFunction;

impl SqlFunction for RadiusSaturnFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "RADIUS_SATURN",
            category: FunctionCategory::Astronomical,
            arg_count: ArgCount::Fixed(0),
            description: "Returns Saturn's radius in meters (5.823 × 10^7)",
            returns: "FLOAT",
            examples: vec!["SELECT RADIUS_SATURN()"],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        Ok(DataValue::Float(5.823e7))
    }
}

/// Uranus radius function
pub struct RadiusUranusFunction;

impl SqlFunction for RadiusUranusFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "RADIUS_URANUS",
            category: FunctionCategory::Astronomical,
            arg_count: ArgCount::Fixed(0),
            description: "Returns Uranus's radius in meters (2.536 × 10^7)",
            returns: "FLOAT",
            examples: vec!["SELECT RADIUS_URANUS()"],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        Ok(DataValue::Float(2.536e7))
    }
}

/// Neptune radius function
pub struct RadiusNeptuneFunction;

impl SqlFunction for RadiusNeptuneFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "RADIUS_NEPTUNE",
            category: FunctionCategory::Astronomical,
            arg_count: ArgCount::Fixed(0),
            description: "Returns Neptune's radius in meters (2.462 × 10^7)",
            returns: "FLOAT",
            examples: vec!["SELECT RADIUS_NEPTUNE()"],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        Ok(DataValue::Float(2.462e7))
    }
}

// Distance Units

/// Astronomical Unit function
pub struct AuFunction;

impl SqlFunction for AuFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "AU",
            category: FunctionCategory::Astronomical,
            arg_count: ArgCount::Fixed(0),
            description: "Returns one Astronomical Unit in meters (1.496 × 10^11)",
            returns: "FLOAT",
            examples: vec!["SELECT AU()", "SELECT distance_m / AU() AS distance_au"],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        Ok(DataValue::Float(1.496e11))
    }
}

/// Light Year function
pub struct LightYearFunction;

impl SqlFunction for LightYearFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "LIGHT_YEAR",
            category: FunctionCategory::Astronomical,
            arg_count: ArgCount::Fixed(0),
            description: "Returns one light year in meters (9.461 × 10^15)",
            returns: "FLOAT",
            examples: vec![
                "SELECT LIGHT_YEAR()",
                "SELECT star_distance / LIGHT_YEAR() AS distance_ly",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        Ok(DataValue::Float(9.461e15))
    }
}

/// Parsec function
pub struct ParsecFunction;

impl SqlFunction for ParsecFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "PARSEC",
            category: FunctionCategory::Astronomical,
            arg_count: ArgCount::Fixed(0),
            description: "Returns one parsec in meters (3.086 × 10^16)",
            returns: "FLOAT",
            examples: vec![
                "SELECT PARSEC()",
                "SELECT galaxy_distance / PARSEC() AS distance_pc",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        Ok(DataValue::Float(3.086e16))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mass_earth() {
        let func = MassEarthFunction;
        let result = func.evaluate(&[]).unwrap();
        match result {
            DataValue::Float(val) => assert_eq!(val, 5.972e24),
            _ => panic!("Expected Float"),
        }
    }

    #[test]
    fn test_mass_sun() {
        let func = MassSunFunction;
        let result = func.evaluate(&[]).unwrap();
        match result {
            DataValue::Float(val) => assert_eq!(val, 1.989e30),
            _ => panic!("Expected Float"),
        }
    }

    #[test]
    fn test_au() {
        let func = AuFunction;
        let result = func.evaluate(&[]).unwrap();
        match result {
            DataValue::Float(val) => assert_eq!(val, 1.496e11),
            _ => panic!("Expected Float"),
        }
    }

    #[test]
    fn test_light_year() {
        let func = LightYearFunction;
        let result = func.evaluate(&[]).unwrap();
        match result {
            DataValue::Float(val) => assert_eq!(val, 9.461e15),
            _ => panic!("Expected Float"),
        }
    }
}

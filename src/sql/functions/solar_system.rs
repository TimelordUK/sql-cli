use anyhow::{anyhow, Result};
use std::collections::HashMap;

use super::{ArgCount, FunctionCategory, FunctionSignature, SqlFunction};
use crate::data::datatable::DataValue;

// Solar system data structure
struct SolarBody {
    mass_kg: f64,
    radius_m: f64,
    distance_from_sun_m: f64,
    orbital_period_days: f64,
    gravity_ms2: f64, // Surface gravity
    density_kgm3: f64,
    escape_velocity_ms: f64,
    rotation_period_hours: f64,
    moons: i32,
}

lazy_static::lazy_static! {
    static ref SOLAR_BODIES: HashMap<&'static str, SolarBody> = {
        let mut m = HashMap::new();

        // Sun
        m.insert("sun", SolarBody {
            mass_kg: 1.989e30,
            radius_m: 6.96e8,
            distance_from_sun_m: 0.0,
            orbital_period_days: 0.0,
            gravity_ms2: 274.0,
            density_kgm3: 1408.0,
            escape_velocity_ms: 617500.0,
            rotation_period_hours: 609.12, // 25.38 days at equator
            moons: 0,
        });

        // Mercury
        m.insert("mercury", SolarBody {
            mass_kg: 3.3011e23,
            radius_m: 2.4397e6,
            distance_from_sun_m: 5.791e10,
            orbital_period_days: 87.969,
            gravity_ms2: 3.7,
            density_kgm3: 5427.0,
            escape_velocity_ms: 4250.0,
            rotation_period_hours: 1407.6, // 58.65 days
            moons: 0,
        });

        // Venus
        m.insert("venus", SolarBody {
            mass_kg: 4.8675e24,
            radius_m: 6.0518e6,
            distance_from_sun_m: 1.082e11,
            orbital_period_days: 224.701,
            gravity_ms2: 8.87,
            density_kgm3: 5243.0,
            escape_velocity_ms: 10360.0,
            rotation_period_hours: 5832.5, // 243.02 days (retrograde)
            moons: 0,
        });

        // Earth
        m.insert("earth", SolarBody {
            mass_kg: 5.97237e24,
            radius_m: 6.371e6,
            distance_from_sun_m: 1.496e11,
            orbital_period_days: 365.256,
            gravity_ms2: 9.807,
            density_kgm3: 5514.0,
            escape_velocity_ms: 11186.0,
            rotation_period_hours: 23.9345,
            moons: 1,
        });

        // Moon
        m.insert("moon", SolarBody {
            mass_kg: 7.342e22,
            radius_m: 1.7374e6,
            distance_from_sun_m: 1.496e11, // Same as Earth
            orbital_period_days: 27.322, // Around Earth
            gravity_ms2: 1.62,
            density_kgm3: 3344.0,
            escape_velocity_ms: 2380.0,
            rotation_period_hours: 655.73, // 27.32 days (tidally locked)
            moons: 0,
        });

        // Mars
        m.insert("mars", SolarBody {
            mass_kg: 6.4171e23,
            radius_m: 3.3895e6,
            distance_from_sun_m: 2.279e11,
            orbital_period_days: 686.971,
            gravity_ms2: 3.71,
            density_kgm3: 3933.0,
            escape_velocity_ms: 5030.0,
            rotation_period_hours: 24.6229,
            moons: 2,
        });

        // Jupiter
        m.insert("jupiter", SolarBody {
            mass_kg: 1.8982e27,
            radius_m: 6.9911e7,
            distance_from_sun_m: 7.786e11,
            orbital_period_days: 4332.59,
            gravity_ms2: 24.79,
            density_kgm3: 1326.0,
            escape_velocity_ms: 59500.0,
            rotation_period_hours: 9.9250,
            moons: 95, // As of 2023
        });

        // Saturn
        m.insert("saturn", SolarBody {
            mass_kg: 5.6834e26,
            radius_m: 5.8232e7,
            distance_from_sun_m: 1.4335e12,
            orbital_period_days: 10759.22,
            gravity_ms2: 10.44,
            density_kgm3: 687.0,
            escape_velocity_ms: 35500.0,
            rotation_period_hours: 10.656,
            moons: 146, // As of 2023
        });

        // Uranus
        m.insert("uranus", SolarBody {
            mass_kg: 8.6810e25,
            radius_m: 2.5362e7,
            distance_from_sun_m: 2.8725e12,
            orbital_period_days: 30688.5,
            gravity_ms2: 8.87,
            density_kgm3: 1271.0,
            escape_velocity_ms: 21300.0,
            rotation_period_hours: 17.24, // Retrograde
            moons: 27,
        });

        // Neptune
        m.insert("neptune", SolarBody {
            mass_kg: 1.02413e26,
            radius_m: 2.4622e7,
            distance_from_sun_m: 4.4951e12,
            orbital_period_days: 60195.0,
            gravity_ms2: 11.15,
            density_kgm3: 1638.0,
            escape_velocity_ms: 23500.0,
            rotation_period_hours: 16.11,
            moons: 16,
        });

        // Pluto (dwarf planet)
        m.insert("pluto", SolarBody {
            mass_kg: 1.303e22,
            radius_m: 1.1883e6,
            distance_from_sun_m: 5.9064e12,
            orbital_period_days: 90560.0,
            gravity_ms2: 0.62,
            density_kgm3: 1854.0,
            escape_velocity_ms: 1210.0,
            rotation_period_hours: 153.29, // 6.387 days
            moons: 5,
        });

        // Ceres (dwarf planet)
        m.insert("ceres", SolarBody {
            mass_kg: 9.393e20,
            radius_m: 4.73e5,
            distance_from_sun_m: 4.14e11,
            orbital_period_days: 1682.0,
            gravity_ms2: 0.27,
            density_kgm3: 2162.0,
            escape_velocity_ms: 510.0,
            rotation_period_hours: 9.07,
            moons: 0,
        });

        // Eris (dwarf planet)
        m.insert("eris", SolarBody {
            mass_kg: 1.66e22,
            radius_m: 1.163e6,
            distance_from_sun_m: 1.0123e13,
            orbital_period_days: 203830.0,
            gravity_ms2: 0.82,
            density_kgm3: 2520.0,
            escape_velocity_ms: 1380.0,
            rotation_period_hours: 25.92,
            moons: 1,
        });

        m
    };
}

// Mass lookup function
pub struct MassSolarBodyFunction;

impl SqlFunction for MassSolarBodyFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "MASS_SOLAR_BODY",
            category: FunctionCategory::Astronomical,
            arg_count: ArgCount::Fixed(1),
            description: "Returns the mass of a solar system body in kg",
            returns: "FLOAT",
            examples: vec![
                "SELECT MASS_SOLAR_BODY('earth')",
                "SELECT MASS_SOLAR_BODY('pluto')",
                "SELECT body_name, MASS_SOLAR_BODY(body_name) FROM solar_system",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let body_name = match &args[0] {
            DataValue::String(s) => s.to_lowercase(),
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("MASS_SOLAR_BODY requires a string argument")),
        };

        match SOLAR_BODIES.get(body_name.as_str()) {
            Some(body) => Ok(DataValue::Float(body.mass_kg)),
            None => Err(anyhow!("Unknown solar body: {}", body_name)),
        }
    }
}

// Radius lookup function
pub struct RadiusSolarBodyFunction;

impl SqlFunction for RadiusSolarBodyFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "RADIUS_SOLAR_BODY",
            category: FunctionCategory::Astronomical,
            arg_count: ArgCount::Fixed(1),
            description: "Returns the radius of a solar system body in meters",
            returns: "FLOAT",
            examples: vec![
                "SELECT RADIUS_SOLAR_BODY('earth')",
                "SELECT RADIUS_SOLAR_BODY('jupiter')",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let body_name = match &args[0] {
            DataValue::String(s) => s.to_lowercase(),
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("RADIUS_SOLAR_BODY requires a string argument")),
        };

        match SOLAR_BODIES.get(body_name.as_str()) {
            Some(body) => Ok(DataValue::Float(body.radius_m)),
            None => Err(anyhow!("Unknown solar body: {}", body_name)),
        }
    }
}

// Distance from Sun lookup function
pub struct DistanceSolarBodyFunction;

impl SqlFunction for DistanceSolarBodyFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "DISTANCE_SOLAR_BODY",
            category: FunctionCategory::Astronomical,
            arg_count: ArgCount::Fixed(1),
            description: "Returns the mean distance from the Sun in meters",
            returns: "FLOAT",
            examples: vec![
                "SELECT DISTANCE_SOLAR_BODY('mars')",
                "SELECT DISTANCE_SOLAR_BODY('neptune') / AU() AS neptune_au",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let body_name = match &args[0] {
            DataValue::String(s) => s.to_lowercase(),
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("DISTANCE_SOLAR_BODY requires a string argument")),
        };

        match SOLAR_BODIES.get(body_name.as_str()) {
            Some(body) => Ok(DataValue::Float(body.distance_from_sun_m)),
            None => Err(anyhow!("Unknown solar body: {}", body_name)),
        }
    }
}

// Orbital period lookup function
pub struct OrbitalPeriodSolarBodyFunction;

impl SqlFunction for OrbitalPeriodSolarBodyFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "ORBITAL_PERIOD_SOLAR_BODY",
            category: FunctionCategory::Astronomical,
            arg_count: ArgCount::Fixed(1),
            description: "Returns the orbital period in days",
            returns: "FLOAT",
            examples: vec![
                "SELECT ORBITAL_PERIOD_SOLAR_BODY('earth')",
                "SELECT ORBITAL_PERIOD_SOLAR_BODY('pluto') / 365.256 AS pluto_years",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let body_name = match &args[0] {
            DataValue::String(s) => s.to_lowercase(),
            DataValue::Null => return Ok(DataValue::Null),
            _ => {
                return Err(anyhow!(
                    "ORBITAL_PERIOD_SOLAR_BODY requires a string argument"
                ))
            }
        };

        match SOLAR_BODIES.get(body_name.as_str()) {
            Some(body) => Ok(DataValue::Float(body.orbital_period_days)),
            None => Err(anyhow!("Unknown solar body: {}", body_name)),
        }
    }
}

// Surface gravity lookup function
pub struct GravitySolarBodyFunction;

impl SqlFunction for GravitySolarBodyFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "GRAVITY_SOLAR_BODY",
            category: FunctionCategory::Astronomical,
            arg_count: ArgCount::Fixed(1),
            description: "Returns the surface gravity in m/s²",
            returns: "FLOAT",
            examples: vec![
                "SELECT GRAVITY_SOLAR_BODY('earth')",
                "SELECT GRAVITY_SOLAR_BODY('moon')",
                "SELECT body, GRAVITY_SOLAR_BODY(body) / 9.807 AS earth_g FROM planets",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let body_name = match &args[0] {
            DataValue::String(s) => s.to_lowercase(),
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("GRAVITY_SOLAR_BODY requires a string argument")),
        };

        match SOLAR_BODIES.get(body_name.as_str()) {
            Some(body) => Ok(DataValue::Float(body.gravity_ms2)),
            None => Err(anyhow!("Unknown solar body: {}", body_name)),
        }
    }
}

// Density lookup function
pub struct DensitySolarBodyFunction;

impl SqlFunction for DensitySolarBodyFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "DENSITY_SOLAR_BODY",
            category: FunctionCategory::Astronomical,
            arg_count: ArgCount::Fixed(1),
            description: "Returns the density in kg/m³",
            returns: "FLOAT",
            examples: vec![
                "SELECT DENSITY_SOLAR_BODY('saturn')",
                "SELECT body, DENSITY_SOLAR_BODY(body) FROM planets ORDER BY DENSITY_SOLAR_BODY(body)",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let body_name = match &args[0] {
            DataValue::String(s) => s.to_lowercase(),
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("DENSITY_SOLAR_BODY requires a string argument")),
        };

        match SOLAR_BODIES.get(body_name.as_str()) {
            Some(body) => Ok(DataValue::Float(body.density_kgm3)),
            None => Err(anyhow!("Unknown solar body: {}", body_name)),
        }
    }
}

// Escape velocity lookup function
pub struct EscapeVelocitySolarBodyFunction;

impl SqlFunction for EscapeVelocitySolarBodyFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "ESCAPE_VELOCITY_SOLAR_BODY",
            category: FunctionCategory::Astronomical,
            arg_count: ArgCount::Fixed(1),
            description: "Returns the escape velocity in m/s",
            returns: "FLOAT",
            examples: vec![
                "SELECT ESCAPE_VELOCITY_SOLAR_BODY('earth')",
                "SELECT body, ESCAPE_VELOCITY_SOLAR_BODY(body) / 1000 AS km_per_s FROM planets",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let body_name = match &args[0] {
            DataValue::String(s) => s.to_lowercase(),
            DataValue::Null => return Ok(DataValue::Null),
            _ => {
                return Err(anyhow!(
                    "ESCAPE_VELOCITY_SOLAR_BODY requires a string argument"
                ))
            }
        };

        match SOLAR_BODIES.get(body_name.as_str()) {
            Some(body) => Ok(DataValue::Float(body.escape_velocity_ms)),
            None => Err(anyhow!("Unknown solar body: {}", body_name)),
        }
    }
}

// Rotation period lookup function
pub struct RotationPeriodSolarBodyFunction;

impl SqlFunction for RotationPeriodSolarBodyFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "ROTATION_PERIOD_SOLAR_BODY",
            category: FunctionCategory::Astronomical,
            arg_count: ArgCount::Fixed(1),
            description: "Returns the rotation period in hours",
            returns: "FLOAT",
            examples: vec![
                "SELECT ROTATION_PERIOD_SOLAR_BODY('earth')",
                "SELECT body, ROTATION_PERIOD_SOLAR_BODY(body) / 24 AS days FROM planets",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let body_name = match &args[0] {
            DataValue::String(s) => s.to_lowercase(),
            DataValue::Null => return Ok(DataValue::Null),
            _ => {
                return Err(anyhow!(
                    "ROTATION_PERIOD_SOLAR_BODY requires a string argument"
                ))
            }
        };

        match SOLAR_BODIES.get(body_name.as_str()) {
            Some(body) => Ok(DataValue::Float(body.rotation_period_hours)),
            None => Err(anyhow!("Unknown solar body: {}", body_name)),
        }
    }
}

// Number of moons lookup function
pub struct MoonsSolarBodyFunction;

impl SqlFunction for MoonsSolarBodyFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "MOONS_SOLAR_BODY",
            category: FunctionCategory::Astronomical,
            arg_count: ArgCount::Fixed(1),
            description: "Returns the number of known moons",
            returns: "INTEGER",
            examples: vec![
                "SELECT MOONS_SOLAR_BODY('jupiter')",
                "SELECT body, MOONS_SOLAR_BODY(body) FROM planets ORDER BY MOONS_SOLAR_BODY(body) DESC",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let body_name = match &args[0] {
            DataValue::String(s) => s.to_lowercase(),
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("MOONS_SOLAR_BODY requires a string argument")),
        };

        match SOLAR_BODIES.get(body_name.as_str()) {
            Some(body) => Ok(DataValue::Integer(body.moons as i64)),
            None => Err(anyhow!("Unknown solar body: {}", body_name)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mass_solar_body() {
        let func = MassSolarBodyFunction;

        // Test Earth
        let result = func
            .evaluate(&[DataValue::String("earth".to_string())])
            .unwrap();
        match result {
            DataValue::Float(val) => assert_eq!(val, 5.97237e24),
            _ => panic!("Expected Float"),
        }

        // Test Pluto
        let result = func
            .evaluate(&[DataValue::String("pluto".to_string())])
            .unwrap();
        match result {
            DataValue::Float(val) => assert_eq!(val, 1.303e22),
            _ => panic!("Expected Float"),
        }
    }

    #[test]
    fn test_gravity_solar_body() {
        let func = GravitySolarBodyFunction;

        // Test Earth
        let result = func
            .evaluate(&[DataValue::String("earth".to_string())])
            .unwrap();
        match result {
            DataValue::Float(val) => assert_eq!(val, 9.807),
            _ => panic!("Expected Float"),
        }

        // Test Moon
        let result = func
            .evaluate(&[DataValue::String("moon".to_string())])
            .unwrap();
        match result {
            DataValue::Float(val) => assert_eq!(val, 1.62),
            _ => panic!("Expected Float"),
        }
    }
}

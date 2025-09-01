use anyhow::{anyhow, Result};
use std::collections::HashMap;

/// Unit categories for conversion
#[derive(Debug, Clone, PartialEq)]
enum UnitCategory {
    Length,
    Mass,
    Temperature,
    Volume,
    Time,
    Area,
    Speed,
    Pressure,
}

/// Unit definition with conversion factor to SI base unit
#[derive(Debug, Clone)]
struct Unit {
    category: UnitCategory,
    to_si_factor: f64,   // Multiply by this to get SI unit
    to_si_offset: f64,   // Add this after multiplication (for temperature)
    from_si_factor: f64, // Multiply SI by this to get unit
    from_si_offset: f64, // Add this after multiplication (for temperature)
}

impl Unit {
    /// Create a simple unit with just a conversion factor
    fn simple(category: UnitCategory, to_si_factor: f64) -> Self {
        Unit {
            category,
            to_si_factor,
            to_si_offset: 0.0,
            from_si_factor: 1.0 / to_si_factor,
            from_si_offset: 0.0,
        }
    }

    /// Create a temperature unit with custom conversion
    fn temperature(
        to_si_factor: f64,
        to_si_offset: f64,
        from_si_factor: f64,
        from_si_offset: f64,
    ) -> Self {
        Unit {
            category: UnitCategory::Temperature,
            to_si_factor,
            to_si_offset,
            from_si_factor,
            from_si_offset,
        }
    }
}

/// Main unit converter
pub struct UnitConverter {
    units: HashMap<String, Unit>,
}

impl UnitConverter {
    pub fn new() -> Self {
        let mut units = HashMap::new();

        // LENGTH - Base unit: meter (m)
        units.insert("m".to_string(), Unit::simple(UnitCategory::Length, 1.0));
        units.insert("meter".to_string(), Unit::simple(UnitCategory::Length, 1.0));
        units.insert(
            "meters".to_string(),
            Unit::simple(UnitCategory::Length, 1.0),
        );
        units.insert("metre".to_string(), Unit::simple(UnitCategory::Length, 1.0));
        units.insert(
            "metres".to_string(),
            Unit::simple(UnitCategory::Length, 1.0),
        );

        // Metric length
        units.insert("km".to_string(), Unit::simple(UnitCategory::Length, 1000.0));
        units.insert(
            "kilometer".to_string(),
            Unit::simple(UnitCategory::Length, 1000.0),
        );
        units.insert(
            "kilometers".to_string(),
            Unit::simple(UnitCategory::Length, 1000.0),
        );
        units.insert("cm".to_string(), Unit::simple(UnitCategory::Length, 0.01));
        units.insert(
            "centimeter".to_string(),
            Unit::simple(UnitCategory::Length, 0.01),
        );
        units.insert(
            "centimeters".to_string(),
            Unit::simple(UnitCategory::Length, 0.01),
        );
        units.insert("mm".to_string(), Unit::simple(UnitCategory::Length, 0.001));
        units.insert(
            "millimeter".to_string(),
            Unit::simple(UnitCategory::Length, 0.001),
        );
        units.insert(
            "millimeters".to_string(),
            Unit::simple(UnitCategory::Length, 0.001),
        );
        units.insert("nm".to_string(), Unit::simple(UnitCategory::Length, 1e-9));
        units.insert(
            "nanometer".to_string(),
            Unit::simple(UnitCategory::Length, 1e-9),
        );
        units.insert("um".to_string(), Unit::simple(UnitCategory::Length, 1e-6));
        units.insert(
            "micrometer".to_string(),
            Unit::simple(UnitCategory::Length, 1e-6),
        );

        // Imperial length
        units.insert(
            "mi".to_string(),
            Unit::simple(UnitCategory::Length, 1609.344),
        );
        units.insert(
            "mile".to_string(),
            Unit::simple(UnitCategory::Length, 1609.344),
        );
        units.insert(
            "miles".to_string(),
            Unit::simple(UnitCategory::Length, 1609.344),
        );
        units.insert("yd".to_string(), Unit::simple(UnitCategory::Length, 0.9144));
        units.insert(
            "yard".to_string(),
            Unit::simple(UnitCategory::Length, 0.9144),
        );
        units.insert(
            "yards".to_string(),
            Unit::simple(UnitCategory::Length, 0.9144),
        );
        units.insert("ft".to_string(), Unit::simple(UnitCategory::Length, 0.3048));
        units.insert(
            "foot".to_string(),
            Unit::simple(UnitCategory::Length, 0.3048),
        );
        units.insert(
            "feet".to_string(),
            Unit::simple(UnitCategory::Length, 0.3048),
        );
        units.insert("in".to_string(), Unit::simple(UnitCategory::Length, 0.0254));
        units.insert(
            "inch".to_string(),
            Unit::simple(UnitCategory::Length, 0.0254),
        );
        units.insert(
            "inches".to_string(),
            Unit::simple(UnitCategory::Length, 0.0254),
        );

        // Nautical
        units.insert(
            "nmi".to_string(),
            Unit::simple(UnitCategory::Length, 1852.0),
        );
        units.insert(
            "nautical_mile".to_string(),
            Unit::simple(UnitCategory::Length, 1852.0),
        );

        // MASS - Base unit: kilogram (kg)
        units.insert("kg".to_string(), Unit::simple(UnitCategory::Mass, 1.0));
        units.insert(
            "kilogram".to_string(),
            Unit::simple(UnitCategory::Mass, 1.0),
        );
        units.insert(
            "kilograms".to_string(),
            Unit::simple(UnitCategory::Mass, 1.0),
        );

        // Metric mass
        units.insert("g".to_string(), Unit::simple(UnitCategory::Mass, 0.001));
        units.insert("gram".to_string(), Unit::simple(UnitCategory::Mass, 0.001));
        units.insert("grams".to_string(), Unit::simple(UnitCategory::Mass, 0.001));
        units.insert("mg".to_string(), Unit::simple(UnitCategory::Mass, 0.000001));
        units.insert(
            "milligram".to_string(),
            Unit::simple(UnitCategory::Mass, 0.000001),
        );
        units.insert(
            "milligrams".to_string(),
            Unit::simple(UnitCategory::Mass, 0.000001),
        );
        units.insert("ug".to_string(), Unit::simple(UnitCategory::Mass, 1e-9));
        units.insert(
            "microgram".to_string(),
            Unit::simple(UnitCategory::Mass, 1e-9),
        );
        units.insert("t".to_string(), Unit::simple(UnitCategory::Mass, 1000.0));
        units.insert(
            "tonne".to_string(),
            Unit::simple(UnitCategory::Mass, 1000.0),
        );
        units.insert(
            "metric_ton".to_string(),
            Unit::simple(UnitCategory::Mass, 1000.0),
        );

        // Imperial mass
        units.insert(
            "lb".to_string(),
            Unit::simple(UnitCategory::Mass, 0.45359237),
        );
        units.insert(
            "lbs".to_string(),
            Unit::simple(UnitCategory::Mass, 0.45359237),
        );
        units.insert(
            "pound".to_string(),
            Unit::simple(UnitCategory::Mass, 0.45359237),
        );
        units.insert(
            "pounds".to_string(),
            Unit::simple(UnitCategory::Mass, 0.45359237),
        );
        units.insert(
            "oz".to_string(),
            Unit::simple(UnitCategory::Mass, 0.028349523125),
        );
        units.insert(
            "ounce".to_string(),
            Unit::simple(UnitCategory::Mass, 0.028349523125),
        );
        units.insert(
            "ounces".to_string(),
            Unit::simple(UnitCategory::Mass, 0.028349523125),
        );
        units.insert(
            "ton".to_string(),
            Unit::simple(UnitCategory::Mass, 907.18474),
        );
        units.insert(
            "short_ton".to_string(),
            Unit::simple(UnitCategory::Mass, 907.18474),
        );
        units.insert(
            "long_ton".to_string(),
            Unit::simple(UnitCategory::Mass, 1016.0469088),
        );
        units.insert(
            "stone".to_string(),
            Unit::simple(UnitCategory::Mass, 6.35029318),
        );

        // TEMPERATURE - Base unit: Kelvin (K)
        units.insert(
            "k".to_string(),
            Unit::simple(UnitCategory::Temperature, 1.0),
        );
        units.insert(
            "kelvin".to_string(),
            Unit::simple(UnitCategory::Temperature, 1.0),
        );

        // Celsius: K = C + 273.15
        units.insert(
            "c".to_string(),
            Unit::temperature(1.0, 273.15, 1.0, -273.15),
        );
        units.insert(
            "celsius".to_string(),
            Unit::temperature(1.0, 273.15, 1.0, -273.15),
        );
        units.insert(
            "centigrade".to_string(),
            Unit::temperature(1.0, 273.15, 1.0, -273.15),
        );

        // Fahrenheit: K = (F + 459.67) × 5/9
        units.insert(
            "f".to_string(),
            Unit::temperature(5.0 / 9.0, 459.67 * 5.0 / 9.0, 9.0 / 5.0, -459.67),
        );
        units.insert(
            "fahrenheit".to_string(),
            Unit::temperature(5.0 / 9.0, 459.67 * 5.0 / 9.0, 9.0 / 5.0, -459.67),
        );

        // VOLUME - Base unit: liter (L)
        units.insert("l".to_string(), Unit::simple(UnitCategory::Volume, 1.0));
        units.insert("liter".to_string(), Unit::simple(UnitCategory::Volume, 1.0));
        units.insert(
            "liters".to_string(),
            Unit::simple(UnitCategory::Volume, 1.0),
        );
        units.insert("litre".to_string(), Unit::simple(UnitCategory::Volume, 1.0));
        units.insert(
            "litres".to_string(),
            Unit::simple(UnitCategory::Volume, 1.0),
        );

        // Metric volume
        units.insert("ml".to_string(), Unit::simple(UnitCategory::Volume, 0.001));
        units.insert(
            "milliliter".to_string(),
            Unit::simple(UnitCategory::Volume, 0.001),
        );
        units.insert(
            "milliliters".to_string(),
            Unit::simple(UnitCategory::Volume, 0.001),
        );
        units.insert("m3".to_string(), Unit::simple(UnitCategory::Volume, 1000.0));
        units.insert(
            "cubic_meter".to_string(),
            Unit::simple(UnitCategory::Volume, 1000.0),
        );
        units.insert("cm3".to_string(), Unit::simple(UnitCategory::Volume, 0.001));
        units.insert(
            "cubic_centimeter".to_string(),
            Unit::simple(UnitCategory::Volume, 0.001),
        );
        units.insert("cc".to_string(), Unit::simple(UnitCategory::Volume, 0.001));

        // Imperial volume
        units.insert(
            "gal".to_string(),
            Unit::simple(UnitCategory::Volume, 3.785411784),
        );
        units.insert(
            "gallon".to_string(),
            Unit::simple(UnitCategory::Volume, 3.785411784),
        );
        units.insert(
            "gallons".to_string(),
            Unit::simple(UnitCategory::Volume, 3.785411784),
        );
        units.insert(
            "qt".to_string(),
            Unit::simple(UnitCategory::Volume, 0.946352946),
        );
        units.insert(
            "quart".to_string(),
            Unit::simple(UnitCategory::Volume, 0.946352946),
        );
        units.insert(
            "quarts".to_string(),
            Unit::simple(UnitCategory::Volume, 0.946352946),
        );
        units.insert(
            "pt".to_string(),
            Unit::simple(UnitCategory::Volume, 0.473176473),
        );
        units.insert(
            "pint".to_string(),
            Unit::simple(UnitCategory::Volume, 0.473176473),
        );
        units.insert(
            "pints".to_string(),
            Unit::simple(UnitCategory::Volume, 0.473176473),
        );
        units.insert(
            "cup".to_string(),
            Unit::simple(UnitCategory::Volume, 0.2365882365),
        );
        units.insert(
            "cups".to_string(),
            Unit::simple(UnitCategory::Volume, 0.2365882365),
        );
        units.insert(
            "fl_oz".to_string(),
            Unit::simple(UnitCategory::Volume, 0.0295735295625),
        );
        units.insert(
            "fluid_ounce".to_string(),
            Unit::simple(UnitCategory::Volume, 0.0295735295625),
        );
        units.insert(
            "tbsp".to_string(),
            Unit::simple(UnitCategory::Volume, 0.01478676478125),
        );
        units.insert(
            "tablespoon".to_string(),
            Unit::simple(UnitCategory::Volume, 0.01478676478125),
        );
        units.insert(
            "tsp".to_string(),
            Unit::simple(UnitCategory::Volume, 0.00492892159375),
        );
        units.insert(
            "teaspoon".to_string(),
            Unit::simple(UnitCategory::Volume, 0.00492892159375),
        );

        // Imperial UK volume
        units.insert(
            "imperial_gal".to_string(),
            Unit::simple(UnitCategory::Volume, 4.54609),
        );
        units.insert(
            "imperial_gallon".to_string(),
            Unit::simple(UnitCategory::Volume, 4.54609),
        );
        units.insert(
            "imperial_qt".to_string(),
            Unit::simple(UnitCategory::Volume, 1.1365225),
        );
        units.insert(
            "imperial_pt".to_string(),
            Unit::simple(UnitCategory::Volume, 0.56826125),
        );

        // TIME - Base unit: second (s)
        units.insert("s".to_string(), Unit::simple(UnitCategory::Time, 1.0));
        units.insert("sec".to_string(), Unit::simple(UnitCategory::Time, 1.0));
        units.insert("second".to_string(), Unit::simple(UnitCategory::Time, 1.0));
        units.insert("seconds".to_string(), Unit::simple(UnitCategory::Time, 1.0));

        units.insert("ms".to_string(), Unit::simple(UnitCategory::Time, 0.001));
        units.insert(
            "millisecond".to_string(),
            Unit::simple(UnitCategory::Time, 0.001),
        );
        units.insert(
            "milliseconds".to_string(),
            Unit::simple(UnitCategory::Time, 0.001),
        );
        units.insert("us".to_string(), Unit::simple(UnitCategory::Time, 0.000001));
        units.insert(
            "microsecond".to_string(),
            Unit::simple(UnitCategory::Time, 0.000001),
        );
        units.insert(
            "microseconds".to_string(),
            Unit::simple(UnitCategory::Time, 0.000001),
        );
        units.insert(
            "ns".to_string(),
            Unit::simple(UnitCategory::Time, 0.000000001),
        );
        units.insert(
            "nanosecond".to_string(),
            Unit::simple(UnitCategory::Time, 0.000000001),
        );
        units.insert(
            "nanoseconds".to_string(),
            Unit::simple(UnitCategory::Time, 0.000000001),
        );

        units.insert("min".to_string(), Unit::simple(UnitCategory::Time, 60.0));
        units.insert("minute".to_string(), Unit::simple(UnitCategory::Time, 60.0));
        units.insert(
            "minutes".to_string(),
            Unit::simple(UnitCategory::Time, 60.0),
        );
        units.insert("hr".to_string(), Unit::simple(UnitCategory::Time, 3600.0));
        units.insert("hour".to_string(), Unit::simple(UnitCategory::Time, 3600.0));
        units.insert(
            "hours".to_string(),
            Unit::simple(UnitCategory::Time, 3600.0),
        );
        units.insert("day".to_string(), Unit::simple(UnitCategory::Time, 86400.0));
        units.insert(
            "days".to_string(),
            Unit::simple(UnitCategory::Time, 86400.0),
        );
        units.insert(
            "week".to_string(),
            Unit::simple(UnitCategory::Time, 604800.0),
        );
        units.insert(
            "weeks".to_string(),
            Unit::simple(UnitCategory::Time, 604800.0),
        );
        units.insert(
            "month".to_string(),
            Unit::simple(UnitCategory::Time, 2629746.0),
        ); // Average month
        units.insert(
            "months".to_string(),
            Unit::simple(UnitCategory::Time, 2629746.0),
        );
        units.insert(
            "year".to_string(),
            Unit::simple(UnitCategory::Time, 31557600.0),
        ); // 365.25 days
        units.insert(
            "years".to_string(),
            Unit::simple(UnitCategory::Time, 31557600.0),
        );

        // AREA - Base unit: square meter (m²)
        units.insert("m2".to_string(), Unit::simple(UnitCategory::Area, 1.0));
        units.insert("sqm".to_string(), Unit::simple(UnitCategory::Area, 1.0));
        units.insert(
            "square_meter".to_string(),
            Unit::simple(UnitCategory::Area, 1.0),
        );
        units.insert(
            "square_meters".to_string(),
            Unit::simple(UnitCategory::Area, 1.0),
        );

        units.insert(
            "km2".to_string(),
            Unit::simple(UnitCategory::Area, 1000000.0),
        );
        units.insert(
            "square_kilometer".to_string(),
            Unit::simple(UnitCategory::Area, 1000000.0),
        );
        units.insert("cm2".to_string(), Unit::simple(UnitCategory::Area, 0.0001));
        units.insert(
            "square_centimeter".to_string(),
            Unit::simple(UnitCategory::Area, 0.0001),
        );

        units.insert(
            "sq_ft".to_string(),
            Unit::simple(UnitCategory::Area, 0.09290304),
        );
        units.insert(
            "square_foot".to_string(),
            Unit::simple(UnitCategory::Area, 0.09290304),
        );
        units.insert(
            "square_feet".to_string(),
            Unit::simple(UnitCategory::Area, 0.09290304),
        );
        units.insert(
            "sq_in".to_string(),
            Unit::simple(UnitCategory::Area, 0.00064516),
        );
        units.insert(
            "square_inch".to_string(),
            Unit::simple(UnitCategory::Area, 0.00064516),
        );
        units.insert(
            "sq_mi".to_string(),
            Unit::simple(UnitCategory::Area, 2589988.110336),
        );
        units.insert(
            "square_mile".to_string(),
            Unit::simple(UnitCategory::Area, 2589988.110336),
        );
        units.insert(
            "acre".to_string(),
            Unit::simple(UnitCategory::Area, 4046.8564224),
        );
        units.insert(
            "acres".to_string(),
            Unit::simple(UnitCategory::Area, 4046.8564224),
        );
        units.insert(
            "hectare".to_string(),
            Unit::simple(UnitCategory::Area, 10000.0),
        );
        units.insert(
            "hectares".to_string(),
            Unit::simple(UnitCategory::Area, 10000.0),
        );

        // SPEED - Base unit: meters per second (m/s)
        units.insert("mps".to_string(), Unit::simple(UnitCategory::Speed, 1.0));
        units.insert("m/s".to_string(), Unit::simple(UnitCategory::Speed, 1.0));
        units.insert(
            "meters_per_second".to_string(),
            Unit::simple(UnitCategory::Speed, 1.0),
        );

        units.insert(
            "kph".to_string(),
            Unit::simple(UnitCategory::Speed, 0.277777778),
        );
        units.insert(
            "kmh".to_string(),
            Unit::simple(UnitCategory::Speed, 0.277777778),
        );
        units.insert(
            "km/h".to_string(),
            Unit::simple(UnitCategory::Speed, 0.277777778),
        );
        units.insert(
            "kilometers_per_hour".to_string(),
            Unit::simple(UnitCategory::Speed, 0.277777778),
        );

        units.insert(
            "mph".to_string(),
            Unit::simple(UnitCategory::Speed, 0.44704),
        );
        units.insert(
            "mi/h".to_string(),
            Unit::simple(UnitCategory::Speed, 0.44704),
        );
        units.insert(
            "miles_per_hour".to_string(),
            Unit::simple(UnitCategory::Speed, 0.44704),
        );

        units.insert(
            "knot".to_string(),
            Unit::simple(UnitCategory::Speed, 0.514444444),
        );
        units.insert(
            "knots".to_string(),
            Unit::simple(UnitCategory::Speed, 0.514444444),
        );
        units.insert(
            "kt".to_string(),
            Unit::simple(UnitCategory::Speed, 0.514444444),
        );

        units.insert("fps".to_string(), Unit::simple(UnitCategory::Speed, 0.3048));
        units.insert(
            "ft/s".to_string(),
            Unit::simple(UnitCategory::Speed, 0.3048),
        );
        units.insert(
            "feet_per_second".to_string(),
            Unit::simple(UnitCategory::Speed, 0.3048),
        );

        // Speed of light (for physics calculations)
        // Note: "c" conflicts with Celsius, so we only use "light_speed"
        units.insert(
            "light_speed".to_string(),
            Unit::simple(UnitCategory::Speed, 299792458.0),
        );

        // PRESSURE - Base unit: Pascal (Pa)
        units.insert("pa".to_string(), Unit::simple(UnitCategory::Pressure, 1.0));
        units.insert(
            "pascal".to_string(),
            Unit::simple(UnitCategory::Pressure, 1.0),
        );
        units.insert(
            "pascals".to_string(),
            Unit::simple(UnitCategory::Pressure, 1.0),
        );

        units.insert(
            "kpa".to_string(),
            Unit::simple(UnitCategory::Pressure, 1000.0),
        );
        units.insert(
            "kilopascal".to_string(),
            Unit::simple(UnitCategory::Pressure, 1000.0),
        );
        units.insert(
            "mpa".to_string(),
            Unit::simple(UnitCategory::Pressure, 1000000.0),
        );
        units.insert(
            "megapascal".to_string(),
            Unit::simple(UnitCategory::Pressure, 1000000.0),
        );
        units.insert(
            "gpa".to_string(),
            Unit::simple(UnitCategory::Pressure, 1000000000.0),
        );
        units.insert(
            "gigapascal".to_string(),
            Unit::simple(UnitCategory::Pressure, 1000000000.0),
        );

        units.insert(
            "bar".to_string(),
            Unit::simple(UnitCategory::Pressure, 100000.0),
        );
        units.insert(
            "bars".to_string(),
            Unit::simple(UnitCategory::Pressure, 100000.0),
        );
        units.insert(
            "mbar".to_string(),
            Unit::simple(UnitCategory::Pressure, 100.0),
        );
        units.insert(
            "millibar".to_string(),
            Unit::simple(UnitCategory::Pressure, 100.0),
        );

        units.insert(
            "atm".to_string(),
            Unit::simple(UnitCategory::Pressure, 101325.0),
        );
        units.insert(
            "atmosphere".to_string(),
            Unit::simple(UnitCategory::Pressure, 101325.0),
        );
        units.insert(
            "atmospheres".to_string(),
            Unit::simple(UnitCategory::Pressure, 101325.0),
        );

        units.insert(
            "psi".to_string(),
            Unit::simple(UnitCategory::Pressure, 6894.757293168),
        );
        units.insert(
            "pounds_per_square_inch".to_string(),
            Unit::simple(UnitCategory::Pressure, 6894.757293168),
        );

        units.insert(
            "torr".to_string(),
            Unit::simple(UnitCategory::Pressure, 133.322368421),
        );
        units.insert(
            "mmhg".to_string(),
            Unit::simple(UnitCategory::Pressure, 133.322368421),
        );
        units.insert(
            "mm_hg".to_string(),
            Unit::simple(UnitCategory::Pressure, 133.322368421),
        );

        UnitConverter { units }
    }

    /// Convert a value from one unit to another
    pub fn convert(&self, value: f64, from_unit: &str, to_unit: &str) -> Result<f64> {
        // Normalize unit names to lowercase for lookup
        let from_key = from_unit.to_lowercase();
        let to_key = to_unit.to_lowercase();

        // Get unit definitions
        let from = self
            .units
            .get(&from_key)
            .ok_or_else(|| anyhow!("Unknown unit: {}", from_unit))?;
        let to = self
            .units
            .get(&to_key)
            .ok_or_else(|| anyhow!("Unknown unit: {}", to_unit))?;

        // Check that units are in the same category
        if from.category != to.category {
            return Err(anyhow!(
                "Cannot convert between different unit types: {} ({:?}) to {} ({:?})",
                from_unit,
                from.category,
                to_unit,
                to.category
            ));
        }

        // Convert to SI base unit
        let si_value = value * from.to_si_factor + from.to_si_offset;

        // Convert from SI base unit to target unit
        let result = si_value * to.from_si_factor + to.from_si_offset;

        Ok(result)
    }

    /// Get list of supported units for a category
    pub fn get_units_for_category(&self, category: &str) -> Vec<String> {
        let cat = match category.to_lowercase().as_str() {
            "length" => Some(UnitCategory::Length),
            "mass" | "weight" => Some(UnitCategory::Mass),
            "temperature" | "temp" => Some(UnitCategory::Temperature),
            "volume" => Some(UnitCategory::Volume),
            "time" => Some(UnitCategory::Time),
            "area" => Some(UnitCategory::Area),
            "speed" | "velocity" => Some(UnitCategory::Speed),
            "pressure" => Some(UnitCategory::Pressure),
            _ => None,
        };

        if let Some(category) = cat {
            self.units
                .iter()
                .filter(|(_, unit)| unit.category == category)
                .map(|(name, _)| name.clone())
                .collect()
        } else {
            Vec::new()
        }
    }
}

// Singleton instance
lazy_static::lazy_static! {
    static ref CONVERTER: UnitConverter = UnitConverter::new();
}

/// Public API for unit conversion
pub fn convert_units(value: f64, from_unit: &str, to_unit: &str) -> Result<f64> {
    CONVERTER.convert(value, from_unit, to_unit)
}

/// Get list of units for a category
pub fn list_units(category: &str) -> Vec<String> {
    CONVERTER.get_units_for_category(category)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_length_conversions() {
        let converter = UnitConverter::new();

        // Kilometers to miles
        let result = converter.convert(100.0, "km", "miles").unwrap();
        assert!((result - 62.137).abs() < 0.01);

        // Feet to meters
        let result = converter.convert(100.0, "ft", "m").unwrap();
        assert!((result - 30.48).abs() < 0.01);

        // Inches to centimeters
        let result = converter.convert(12.0, "in", "cm").unwrap();
        assert!((result - 30.48).abs() < 0.01);
    }

    #[test]
    fn test_mass_conversions() {
        let converter = UnitConverter::new();

        // Pounds to kilograms
        let result = converter.convert(100.0, "lb", "kg").unwrap();
        assert!((result - 45.359).abs() < 0.01);

        // Ounces to grams
        let result = converter.convert(16.0, "oz", "g").unwrap();
        assert!((result - 453.592).abs() < 0.01);
    }

    #[test]
    fn test_temperature_conversions() {
        let converter = UnitConverter::new();

        // Fahrenheit to Celsius
        let result = converter.convert(32.0, "F", "C").unwrap();
        assert!((result - 0.0).abs() < 0.01);

        let result = converter.convert(212.0, "F", "C").unwrap();
        assert!((result - 100.0).abs() < 0.01);

        // Celsius to Kelvin
        let result = converter.convert(0.0, "C", "K").unwrap();
        assert!((result - 273.15).abs() < 0.01);
    }

    #[test]
    fn test_volume_conversions() {
        let converter = UnitConverter::new();

        // Gallons to liters
        let result = converter.convert(1.0, "gal", "L").unwrap();
        assert!((result - 3.785).abs() < 0.01);

        // Cups to milliliters
        let result = converter.convert(1.0, "cup", "ml").unwrap();
        assert!((result - 236.588).abs() < 0.01);
    }

    #[test]
    fn test_invalid_conversion() {
        let converter = UnitConverter::new();

        // Cannot convert between different categories
        let result = converter.convert(100.0, "km", "kg");
        assert!(result.is_err());

        // Unknown unit
        let result = converter.convert(100.0, "xyz", "m");
        assert!(result.is_err());
    }

    #[test]
    fn test_case_insensitive() {
        let converter = UnitConverter::new();

        // Should handle various cases
        let result1 = converter.convert(100.0, "KM", "MILES").unwrap();
        let result2 = converter.convert(100.0, "km", "miles").unwrap();
        let result3 = converter.convert(100.0, "Km", "Miles").unwrap();

        assert!((result1 - result2).abs() < 0.001);
        assert!((result2 - result3).abs() < 0.001);
    }
}

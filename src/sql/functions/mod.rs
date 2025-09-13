use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use crate::data::datatable::DataValue;

pub mod astronomy;
pub mod chemistry;
pub mod comparison;
pub mod constants;
pub mod convert;
pub mod date_time;
pub mod format;
pub mod format_number;
pub mod geometry;
pub mod group_num;
pub mod hash;
pub mod math;
pub mod mathematics;
pub mod particle_charges;
pub mod physics;
pub mod random;
pub mod solar_system;
pub mod string_methods;
pub mod type_checking;

// Re-export MethodFunction trait
pub use string_methods::MethodFunction;

/// Category of SQL functions for organization and discovery
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FunctionCategory {
    Constant,     // Mathematical and physical constants
    Mathematical, // Mathematical operations
    Astronomical, // Astronomical constants and calculations
    Chemical,     // Chemical elements and properties
    Date,         // Date/time operations
    String,       // String manipulation
    Aggregate,    // Aggregation functions
    Conversion,   // Unit conversion functions
}

impl fmt::Display for FunctionCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FunctionCategory::Constant => write!(f, "Constant"),
            FunctionCategory::Mathematical => write!(f, "Mathematical"),
            FunctionCategory::Astronomical => write!(f, "Astronomical"),
            FunctionCategory::Chemical => write!(f, "Chemical"),
            FunctionCategory::Date => write!(f, "Date"),
            FunctionCategory::String => write!(f, "String"),
            FunctionCategory::Aggregate => write!(f, "Aggregate"),
            FunctionCategory::Conversion => write!(f, "Conversion"),
        }
    }
}

/// Describes the number of arguments a function accepts
#[derive(Debug, Clone)]
pub enum ArgCount {
    /// Exactly n arguments
    Fixed(usize),
    /// Between min and max arguments (inclusive)
    Range(usize, usize),
    /// Any number of arguments
    Variadic,
}

impl ArgCount {
    #[must_use]
    pub fn is_valid(&self, count: usize) -> bool {
        match self {
            ArgCount::Fixed(n) => count == *n,
            ArgCount::Range(min, max) => count >= *min && count <= *max,
            ArgCount::Variadic => true,
        }
    }

    #[must_use]
    pub fn description(&self) -> String {
        match self {
            ArgCount::Fixed(0) => "no arguments".to_string(),
            ArgCount::Fixed(1) => "1 argument".to_string(),
            ArgCount::Fixed(n) => format!("{n} arguments"),
            ArgCount::Range(min, max) => format!("{min} to {max} arguments"),
            ArgCount::Variadic => "any number of arguments".to_string(),
        }
    }
}

/// Signature of a SQL function including metadata
#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub name: &'static str,
    pub category: FunctionCategory,
    pub arg_count: ArgCount,
    pub description: &'static str,
    pub returns: &'static str,
    pub examples: Vec<&'static str>,
}

/// Trait that all SQL functions must implement
pub trait SqlFunction: Send + Sync {
    /// Get the function's signature and metadata
    fn signature(&self) -> FunctionSignature;

    /// Evaluate the function with the given arguments
    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue>;

    /// Validate arguments before evaluation (default implementation checks count)
    fn validate_args(&self, args: &[DataValue]) -> Result<()> {
        let sig = self.signature();
        if !sig.arg_count.is_valid(args.len()) {
            return Err(anyhow!(
                "{}() expects {}, got {}",
                sig.name,
                sig.arg_count.description(),
                args.len()
            ));
        }
        Ok(())
    }
}

/// Registry for all SQL functions
pub struct FunctionRegistry {
    functions: HashMap<String, Box<dyn SqlFunction>>,
    by_category: HashMap<FunctionCategory, Vec<String>>,
    methods: HashMap<String, Arc<dyn MethodFunction>>,
}

impl FunctionRegistry {
    /// Create a new registry with all built-in functions
    #[must_use]
    pub fn new() -> Self {
        let mut registry = Self {
            functions: HashMap::new(),
            by_category: HashMap::new(),
            methods: HashMap::new(),
        };

        // Register all built-in functions
        registry.register_constants();
        registry.register_astronomical_functions();
        registry.register_chemical_functions();
        registry.register_mathematical_functions();
        registry.register_geometry_functions();
        registry.register_physics_functions();
        registry.register_date_time_functions();
        registry.register_string_methods();
        registry.register_conversion_functions();
        registry.register_hash_functions();
        registry.register_comparison_functions();
        registry.register_aggregate_functions();
        registry.register_random_functions();
        registry.register_format_functions();
        registry.register_type_checking_functions();

        registry
    }

    /// Register a function in the registry
    pub fn register(&mut self, func: Box<dyn SqlFunction>) {
        let sig = func.signature();
        let name = sig.name.to_uppercase();
        let category = sig.category;

        // Add to main registry
        self.functions.insert(name.clone(), func);

        // Add to category index
        self.by_category.entry(category).or_default().push(name);
    }

    /// Get a function by name (case-insensitive)
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&dyn SqlFunction> {
        self.functions
            .get(&name.to_uppercase())
            .map(std::convert::AsRef::as_ref)
    }

    /// Check if a function exists
    #[must_use]
    pub fn contains(&self, name: &str) -> bool {
        self.functions.contains_key(&name.to_uppercase())
    }

    /// Get all functions matching a prefix (for autocomplete)
    #[must_use]
    pub fn autocomplete(&self, prefix: &str) -> Vec<FunctionSignature> {
        let prefix_upper = prefix.to_uppercase();
        self.functions
            .iter()
            .filter(|(name, _)| name.starts_with(&prefix_upper))
            .map(|(_, func)| func.signature())
            .collect()
    }

    /// Get all functions in a category
    #[must_use]
    pub fn get_by_category(&self, category: FunctionCategory) -> Vec<FunctionSignature> {
        self.by_category
            .get(&category)
            .map(|names| {
                names
                    .iter()
                    .filter_map(|name| self.functions.get(name))
                    .map(|func| func.signature())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all available functions
    #[must_use]
    pub fn all_functions(&self) -> Vec<FunctionSignature> {
        self.functions
            .values()
            .map(|func| func.signature())
            .collect()
    }

    /// Register a method function
    pub fn register_method(&mut self, method: Arc<dyn MethodFunction>) {
        let method_name = method.method_name().to_uppercase();
        self.methods.insert(method_name, method);
    }

    /// Get a method function by name
    #[must_use]
    pub fn get_method(&self, name: &str) -> Option<Arc<dyn MethodFunction>> {
        // Try exact match first
        if let Some(method) = self.methods.get(&name.to_uppercase()) {
            return Some(Arc::clone(method));
        }

        // Try to find a method that handles this name
        for method in self.methods.values() {
            if method.handles_method(name) {
                return Some(Arc::clone(method));
            }
        }

        None
    }

    /// Check if a method exists
    #[must_use]
    pub fn has_method(&self, name: &str) -> bool {
        self.get_method(name).is_some()
    }

    /// Generate markdown documentation for all functions
    #[must_use]
    pub fn generate_markdown_docs(&self) -> String {
        use std::fmt::Write;
        let mut doc = String::new();

        writeln!(&mut doc, "# SQL CLI Function Reference\n").unwrap();
        writeln!(
            &mut doc,
            "This document is auto-generated from the function registry.\n"
        )
        .unwrap();

        // Get all categories in a deterministic order
        let mut categories: Vec<FunctionCategory> = self.by_category.keys().copied().collect();
        categories.sort_by_key(|c| format!("{c:?}"));

        for category in categories {
            let functions = self.get_by_category(category);
            if functions.is_empty() {
                continue;
            }

            writeln!(&mut doc, "## {category} Functions\n").unwrap();

            // Sort functions by name for consistent output
            let mut functions = functions;
            functions.sort_by_key(|f| f.name);

            for func in functions {
                writeln!(&mut doc, "### {}()\n", func.name).unwrap();
                writeln!(&mut doc, "**Description:** {}\n", func.description).unwrap();
                writeln!(
                    &mut doc,
                    "**Arguments:** {}\n",
                    func.arg_count.description()
                )
                .unwrap();
                writeln!(&mut doc, "**Returns:** {}\n", func.returns).unwrap();

                if !func.examples.is_empty() {
                    writeln!(&mut doc, "**Examples:**").unwrap();
                    writeln!(&mut doc, "```sql").unwrap();
                    for example in &func.examples {
                        writeln!(&mut doc, "{example}").unwrap();
                    }
                    writeln!(&mut doc, "```\n").unwrap();
                }
            }
        }

        doc
    }

    /// Generate help text for a specific function
    #[must_use]
    pub fn generate_function_help(&self, name: &str) -> Option<String> {
        self.get(name).map(|func| {
            let sig = func.signature();
            let mut help = String::new();
            use std::fmt::Write;

            writeln!(&mut help, "Function: {}()", sig.name).unwrap();
            writeln!(&mut help, "Category: {}", sig.category).unwrap();
            writeln!(&mut help, "Description: {}", sig.description).unwrap();
            writeln!(&mut help, "Arguments: {}", sig.arg_count.description()).unwrap();
            writeln!(&mut help, "Returns: {}", sig.returns).unwrap();

            if !sig.examples.is_empty() {
                writeln!(&mut help, "\nExamples:").unwrap();
                for example in &sig.examples {
                    writeln!(&mut help, "  {example}").unwrap();
                }
            }

            help
        })
    }

    /// List all available functions with brief descriptions
    #[must_use]
    pub fn list_functions(&self) -> String {
        use std::fmt::Write;
        let mut list = String::new();

        writeln!(&mut list, "Available SQL Functions:\n").unwrap();

        let mut categories: Vec<FunctionCategory> = self.by_category.keys().copied().collect();
        categories.sort_by_key(|c| format!("{c:?}"));

        for category in categories {
            let functions = self.get_by_category(category);
            if functions.is_empty() {
                continue;
            }

            writeln!(&mut list, "{category} Functions:").unwrap();

            let mut functions = functions;
            functions.sort_by_key(|f| f.name);

            for func in functions {
                writeln!(
                    &mut list,
                    "  {:20} - {}",
                    format!("{}()", func.name),
                    func.description
                )
                .unwrap();
            }
            writeln!(&mut list).unwrap();
        }

        list
    }

    /// Register constant functions
    fn register_constants(&mut self) {
        use constants::{
            EFunction, HbarFunction, MassElectronFunction, MeFunction, PhiFunction, PiFunction,
            TauFunction,
        };

        self.register(Box::new(PiFunction));
        self.register(Box::new(EFunction));
        self.register(Box::new(MeFunction)); // Mass of electron
        self.register(Box::new(MassElectronFunction)); // Alias for ME
        self.register(Box::new(TauFunction));
        self.register(Box::new(PhiFunction));
        self.register(Box::new(HbarFunction));
    }

    /// Register astronomical functions
    fn register_astronomical_functions(&mut self) {
        use astronomy::{
            AuFunction, DistJupiterFunction, DistMarsFunction, DistMercuryFunction,
            DistNeptuneFunction, DistSaturnFunction, DistUranusFunction, DistVenusFunction,
            LightYearFunction, MassEarthFunction, MassJupiterFunction, MassMarsFunction,
            MassMercuryFunction, MassMoonFunction, MassNeptuneFunction, MassSaturnFunction,
            MassSunFunction, MassUranusFunction, MassVenusFunction, ParsecFunction,
            RadiusEarthFunction, RadiusJupiterFunction, RadiusMarsFunction, RadiusMercuryFunction,
            RadiusMoonFunction, RadiusNeptuneFunction, RadiusSaturnFunction, RadiusSunFunction,
            RadiusUranusFunction, RadiusVenusFunction,
        };

        use solar_system::{
            DensitySolarBodyFunction, DistanceSolarBodyFunction, EscapeVelocitySolarBodyFunction,
            GravitySolarBodyFunction, MassSolarBodyFunction, MoonsSolarBodyFunction,
            OrbitalPeriodSolarBodyFunction, RadiusSolarBodyFunction,
            RotationPeriodSolarBodyFunction,
        };

        self.register(Box::new(MassEarthFunction));
        self.register(Box::new(MassSunFunction));
        self.register(Box::new(MassMoonFunction));
        self.register(Box::new(AuFunction)); // Astronomical unit
        self.register(Box::new(LightYearFunction));
        self.register(Box::new(ParsecFunction));

        // Planetary masses
        self.register(Box::new(MassMercuryFunction));
        self.register(Box::new(MassVenusFunction));
        self.register(Box::new(MassMarsFunction));
        self.register(Box::new(MassJupiterFunction));
        self.register(Box::new(MassSaturnFunction));
        self.register(Box::new(MassUranusFunction));
        self.register(Box::new(MassNeptuneFunction));

        // Solar body radius functions
        self.register(Box::new(RadiusSunFunction));
        self.register(Box::new(RadiusEarthFunction));
        self.register(Box::new(RadiusMoonFunction));
        self.register(Box::new(RadiusMercuryFunction));
        self.register(Box::new(RadiusVenusFunction));
        self.register(Box::new(RadiusMarsFunction));
        self.register(Box::new(RadiusJupiterFunction));
        self.register(Box::new(RadiusSaturnFunction));
        self.register(Box::new(RadiusUranusFunction));
        self.register(Box::new(RadiusNeptuneFunction));

        // Planetary distances from the Sun
        self.register(Box::new(DistMercuryFunction));
        self.register(Box::new(DistVenusFunction));
        self.register(Box::new(DistMarsFunction));
        self.register(Box::new(DistJupiterFunction));
        self.register(Box::new(DistSaturnFunction));
        self.register(Box::new(DistUranusFunction));
        self.register(Box::new(DistNeptuneFunction));

        // Solar system lookup functions
        self.register(Box::new(MassSolarBodyFunction));
        self.register(Box::new(RadiusSolarBodyFunction));
        self.register(Box::new(DistanceSolarBodyFunction));
        self.register(Box::new(OrbitalPeriodSolarBodyFunction));
        self.register(Box::new(GravitySolarBodyFunction));
        self.register(Box::new(DensitySolarBodyFunction));
        self.register(Box::new(EscapeVelocitySolarBodyFunction));
        self.register(Box::new(RotationPeriodSolarBodyFunction));
        self.register(Box::new(MoonsSolarBodyFunction));
    }

    /// Register chemical functions
    fn register_chemical_functions(&mut self) {
        use chemistry::{
            AtomicMassFunction, AtomicNumberFunction, AvogadroFunction, MoleculeFormulaFunction,
            NeutronsFunction,
        };

        self.register(Box::new(AvogadroFunction));
        self.register(Box::new(AtomicMassFunction));
        self.register(Box::new(AtomicNumberFunction));
        self.register(Box::new(NeutronsFunction));
        self.register(Box::new(MoleculeFormulaFunction));
    }

    /// Register string method functions
    fn register_string_methods(&mut self) {
        string_methods::register_string_methods(self);
    }

    /// Register geometry functions
    fn register_geometry_functions(&mut self) {
        use geometry::{
            CircleAreaFunction, CircleCircumferenceFunction, Distance2DFunction,
            PythagorasFunction, SphereSurfaceAreaFunction, SphereVolumeFunction,
            TriangleAreaFunction,
        };

        self.register(Box::new(PythagorasFunction));
        self.register(Box::new(CircleAreaFunction));
        self.register(Box::new(CircleCircumferenceFunction));
        self.register(Box::new(SphereVolumeFunction));
        self.register(Box::new(SphereSurfaceAreaFunction));
        self.register(Box::new(TriangleAreaFunction));
        self.register(Box::new(Distance2DFunction));
    }

    /// Register hash functions
    fn register_hash_functions(&mut self) {
        use hash::{Md5Function, Sha1Function, Sha256Function, Sha512Function};

        self.register(Box::new(Md5Function));
        self.register(Box::new(Sha1Function));
        self.register(Box::new(Sha256Function));
        self.register(Box::new(Sha512Function));
    }

    /// Register comparison functions
    fn register_comparison_functions(&mut self) {
        comparison::register_comparison_functions(self);
    }

    /// Register mathematical functions
    fn register_mathematical_functions(&mut self) {
        use mathematics::{
            IsPrimeFunction, NextPrimeFunction, NthPrimeFunction, PrevPrimeFunction,
            PrimeCountFunction, PrimeFunction, PrimePiFunction,
        };

        // Prime number functions
        self.register(Box::new(PrimeFunction));
        self.register(Box::new(NthPrimeFunction)); // Alias for PRIME
        self.register(Box::new(IsPrimeFunction));
        self.register(Box::new(PrimeCountFunction));
        self.register(Box::new(PrimePiFunction)); // Alias for PRIME_COUNT
        self.register(Box::new(NextPrimeFunction));
        self.register(Box::new(PrevPrimeFunction));

        // General math functions
        math::register_math_functions(self);
    }

    /// Register physics constants
    fn register_physics_functions(&mut self) {
        physics::register_physics_functions(self);

        // Register particle charge functions
        use particle_charges::{
            ChargeDownQuarkFunction, ChargeElectronFunction, ChargeMuonFunction,
            ChargeNeutronFunction, ChargePositronFunction, ChargeProtonFunction, ChargeTauFunction,
            ChargeUpQuarkFunction,
        };

        self.register(Box::new(ChargeElectronFunction));
        self.register(Box::new(ChargeProtonFunction));
        self.register(Box::new(ChargeNeutronFunction));
        self.register(Box::new(ChargeUpQuarkFunction));
        self.register(Box::new(ChargeDownQuarkFunction));
        self.register(Box::new(ChargePositronFunction));
        self.register(Box::new(ChargeMuonFunction));
        self.register(Box::new(ChargeTauFunction));
    }

    /// Register date/time functions
    fn register_date_time_functions(&mut self) {
        date_time::register_date_time_functions(self);
    }

    /// Register conversion functions
    fn register_conversion_functions(&mut self) {
        use convert::ConvertFunction;

        self.register(Box::new(ConvertFunction));
    }

    /// Register aggregate and analytic functions
    fn register_aggregate_functions(&mut self) {
        use group_num::GroupNumFunction;

        // Register GROUP_NUM function
        // Note: We create a new instance per query to ensure clean memoization
        self.register(Box::new(GroupNumFunction::new()));
    }

    /// Register random number generation functions
    fn register_random_functions(&mut self) {
        use random::{RandIntFunction, RandRangeFunction, RandomFunction};

        self.register(Box::new(RandomFunction));
        self.register(Box::new(RandIntFunction));
        self.register(Box::new(RandRangeFunction));
    }

    /// Register formatting functions
    fn register_format_functions(&mut self) {
        use format::{
            CenterFunction, FormatDateFunction, FormatNumberFunction, LPadFunction, RPadFunction,
        };
        use format_number::{FormatCurrencyFunction, RenderNumberFunction};

        self.register(Box::new(FormatNumberFunction));
        self.register(Box::new(FormatDateFunction));
        self.register(Box::new(LPadFunction));
        self.register(Box::new(RPadFunction));
        self.register(Box::new(CenterFunction));
        self.register(Box::new(RenderNumberFunction));
        self.register(Box::new(FormatCurrencyFunction));
    }

    /// Register type checking functions
    fn register_type_checking_functions(&mut self) {
        use type_checking::{
            IsBoolFunction, IsDateFunction, IsFloatFunction, IsIntegerFunction, IsNotNullFunction,
            IsNullFunction, IsNumericFunction,
        };

        self.register(Box::new(IsDateFunction));
        self.register(Box::new(IsBoolFunction));
        self.register(Box::new(IsNumericFunction));
        self.register(Box::new(IsIntegerFunction));
        self.register(Box::new(IsFloatFunction));
        self.register(Box::new(IsNullFunction));
        self.register(Box::new(IsNotNullFunction));
    }
}

impl Default for FunctionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = FunctionRegistry::new();

        // Check that some known functions exist
        assert!(registry.contains("PI"));
        assert!(registry.contains("MASS_EARTH"));
        assert!(registry.contains("ME"));
    }

    #[test]
    fn test_case_insensitive_lookup() {
        let registry = FunctionRegistry::new();

        assert!(registry.get("pi").is_some());
        assert!(registry.get("PI").is_some());
        assert!(registry.get("Pi").is_some());
    }

    #[test]
    fn test_autocomplete() {
        let registry = FunctionRegistry::new();

        let mass_functions = registry.autocomplete("MASS");
        assert!(!mass_functions.is_empty());

        // Should include MASS_EARTH, MASS_SUN, etc.
        let names: Vec<&str> = mass_functions.iter().map(|sig| sig.name).collect();
        assert!(names.contains(&"MASS_EARTH"));
        assert!(names.contains(&"MASS_SUN"));
    }
}

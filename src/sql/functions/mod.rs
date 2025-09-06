use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use crate::data::datatable::DataValue;

pub mod astronomy;
pub mod chemistry;
pub mod comparison;
pub mod constants;
pub mod date_time;
pub mod math;
pub mod mathematics;
pub mod physics;
pub mod string_methods;

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
    pub fn is_valid(&self, count: usize) -> bool {
        match self {
            ArgCount::Fixed(n) => count == *n,
            ArgCount::Range(min, max) => count >= *min && count <= *max,
            ArgCount::Variadic => true,
        }
    }

    pub fn description(&self) -> String {
        match self {
            ArgCount::Fixed(0) => "no arguments".to_string(),
            ArgCount::Fixed(1) => "1 argument".to_string(),
            ArgCount::Fixed(n) => format!("{} arguments", n),
            ArgCount::Range(min, max) => format!("{} to {} arguments", min, max),
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
        registry.register_physics_functions();
        registry.register_date_time_functions();
        registry.register_string_methods();
        registry.register_comparison_functions();

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
        self.by_category
            .entry(category)
            .or_insert_with(Vec::new)
            .push(name);
    }

    /// Get a function by name (case-insensitive)
    pub fn get(&self, name: &str) -> Option<&dyn SqlFunction> {
        self.functions.get(&name.to_uppercase()).map(|b| b.as_ref())
    }

    /// Check if a function exists
    pub fn contains(&self, name: &str) -> bool {
        self.functions.contains_key(&name.to_uppercase())
    }

    /// Get all functions matching a prefix (for autocomplete)
    pub fn autocomplete(&self, prefix: &str) -> Vec<FunctionSignature> {
        let prefix_upper = prefix.to_uppercase();
        self.functions
            .iter()
            .filter(|(name, _)| name.starts_with(&prefix_upper))
            .map(|(_, func)| func.signature())
            .collect()
    }

    /// Get all functions in a category
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
    pub fn has_method(&self, name: &str) -> bool {
        self.get_method(name).is_some()
    }

    /// Generate markdown documentation for all functions
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
        categories.sort_by_key(|c| format!("{:?}", c));

        for category in categories {
            let functions = self.get_by_category(category);
            if functions.is_empty() {
                continue;
            }

            writeln!(&mut doc, "## {} Functions\n", category).unwrap();

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
                        writeln!(&mut doc, "{}", example).unwrap();
                    }
                    writeln!(&mut doc, "```\n").unwrap();
                }
            }
        }

        doc
    }

    /// Generate help text for a specific function
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
                    writeln!(&mut help, "  {}", example).unwrap();
                }
            }

            help
        })
    }

    /// List all available functions with brief descriptions
    pub fn list_functions(&self) -> String {
        use std::fmt::Write;
        let mut list = String::new();

        writeln!(&mut list, "Available SQL Functions:\n").unwrap();

        let mut categories: Vec<FunctionCategory> = self.by_category.keys().copied().collect();
        categories.sort_by_key(|c| format!("{:?}", c));

        for category in categories {
            let functions = self.get_by_category(category);
            if functions.is_empty() {
                continue;
            }

            writeln!(&mut list, "{} Functions:", category).unwrap();

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
        use constants::*;

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
        use astronomy::*;

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
    }

    /// Register chemical functions
    fn register_chemical_functions(&mut self) {
        use chemistry::*;

        self.register(Box::new(AvogadroFunction));
        self.register(Box::new(AtomicMassFunction));
        self.register(Box::new(AtomicNumberFunction));
        self.register(Box::new(MoleculeFormulaFunction));
    }

    /// Register string method functions
    fn register_string_methods(&mut self) {
        string_methods::register_string_methods(self);
    }

    /// Register comparison functions
    fn register_comparison_functions(&mut self) {
        comparison::register_comparison_functions(self);
    }

    /// Register mathematical functions
    fn register_mathematical_functions(&mut self) {
        use mathematics::*;

        // Prime number functions
        self.register(Box::new(PrimeFunction));
        self.register(Box::new(IsPrimeFunction));
        self.register(Box::new(PrimeCountFunction));
        self.register(Box::new(NextPrimeFunction));
        self.register(Box::new(PrevPrimeFunction));

        // General math functions
        math::register_math_functions(self);
    }

    /// Register physics constants
    fn register_physics_functions(&mut self) {
        physics::register_physics_functions(self);
    }

    /// Register date/time functions
    fn register_date_time_functions(&mut self) {
        date_time::register_date_time_functions(self);
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

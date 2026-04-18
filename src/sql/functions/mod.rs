use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use crate::data::datatable::DataValue;

pub mod ansi;
pub mod astronomy;
pub mod base_conversion;
pub mod bigint;
pub mod bitwise;
pub mod bitwise_string;
pub mod case_convert;
pub mod chemistry;
pub mod comparison;
pub mod constants;
pub mod convert;
pub mod date_time;
pub mod financial;
pub mod format;
pub mod format_number;
pub mod geometry;
pub mod group_num;
pub mod hash;
pub mod integer_limits;
pub mod math;
pub mod mathematics;
pub mod number_words;
pub mod particle_charges;
pub mod path;
pub mod physics;
pub mod random;
pub mod roman;
pub mod solar_system;
pub mod statistics;
pub mod string_fun;
pub mod string_methods;
pub mod string_utils;
pub mod text_processing;
pub mod trigonometry;
pub mod type_checking;
pub mod utility;
pub mod vector;

// Re-export MethodFunction trait
pub use string_methods::MethodFunction;

/// Category of SQL functions for organization and discovery
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FunctionCategory {
    Constant,      // Mathematical and physical constants
    Mathematical,  // Mathematical operations
    Statistical,   // Statistical functions
    Astronomical,  // Astronomical constants and calculations
    Chemical,      // Chemical elements and properties
    Date,          // Date/time operations
    String,        // String manipulation
    Aggregate,     // Aggregation functions
    Conversion,    // Unit conversion functions
    BigNumber,     // Arbitrary precision arithmetic
    TableFunction, // Table-generating functions
    Bitwise,       // Bitwise operations and binary visualization
    Terminal,      // Terminal formatting (ANSI colors, styles)
}

impl fmt::Display for FunctionCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FunctionCategory::Constant => write!(f, "Constant"),
            FunctionCategory::Mathematical => write!(f, "Mathematical"),
            FunctionCategory::Statistical => write!(f, "Statistical"),
            FunctionCategory::Astronomical => write!(f, "Astronomical"),
            FunctionCategory::Chemical => write!(f, "Chemical"),
            FunctionCategory::Date => write!(f, "Date"),
            FunctionCategory::String => write!(f, "String"),
            FunctionCategory::Aggregate => write!(f, "Aggregate"),
            FunctionCategory::Conversion => write!(f, "Conversion"),
            FunctionCategory::BigNumber => write!(f, "BigNumber"),
            FunctionCategory::TableFunction => write!(f, "TableFunction"),
            FunctionCategory::Bitwise => write!(f, "Bitwise"),
            FunctionCategory::Terminal => write!(f, "Terminal"),
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
        registry.register_statistical_functions();
        registry.register_geometry_functions();
        registry.register_physics_functions();
        registry.register_date_time_functions();
        registry.register_string_methods();
        registry.register_financial_functions();
        registry.register_bigint_functions();
        registry.register_conversion_functions();
        registry.register_hash_functions();
        registry.register_comparison_functions();
        registry.register_aggregate_functions();
        registry.register_random_functions();
        registry.register_format_functions();
        registry.register_type_checking_functions();
        registry.register_utility_functions();
        registry.register_bitwise_functions();
        registry.register_ansi_functions();
        registry.register_vector_functions();

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
            EFunction, HbarFunction, MassElectronFunction, MeFunction, PhiFunction,
            PiDigitFunction, PiDigitsFunction, PiFunction, TauFunction,
        };

        self.register(Box::new(PiFunction));
        self.register(Box::new(PiDigitsFunction)); // Arbitrary precision pi
        self.register(Box::new(PiDigitFunction)); // Single digit lookup
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
        use case_convert::{
            ToCamelCaseFunction, ToConstantCaseFunction, ToKebabCaseFunction, ToPascalCaseFunction,
            ToSnakeCaseFunction,
        };
        use number_words::{ToOrdinal, ToOrdinalWords, ToWords};
        use path::{
            BasenameFunction, DirnameFunction, ExtensionFunction, PathDepthFunction,
            PathPartFunction, StemFunction,
        };
        use string_fun::{
            InitCapFunction, MorseCodeFunction, PigLatinFunction, ProperFunction, ReverseFunction,
            Rot13Function, ScrambleFunction, SoundexFunction,
        };
        use string_utils::{LPadFunction, RPadFunction, RepeatFunction};
        use text_processing::{CleanText, ExtractWords, StripPunctuation, Tokenize, WordCount};

        string_methods::register_string_methods(self);

        // String utility functions
        self.register(Box::new(RepeatFunction));
        self.register(Box::new(LPadFunction));
        self.register(Box::new(RPadFunction));

        // Path / filename functions (POSIX-style)
        self.register(Box::new(BasenameFunction));
        self.register(Box::new(DirnameFunction));
        self.register(Box::new(ExtensionFunction));
        self.register(Box::new(StemFunction));
        self.register(Box::new(PathDepthFunction));
        self.register(Box::new(PathPartFunction));

        // Case conversion functions
        self.register(Box::new(ToSnakeCaseFunction));
        self.register(Box::new(ToCamelCaseFunction));
        self.register(Box::new(ToPascalCaseFunction));
        self.register(Box::new(ToKebabCaseFunction));
        self.register(Box::new(ToConstantCaseFunction));

        // String fun & transformation functions
        self.register(Box::new(ReverseFunction));
        self.register(Box::new(InitCapFunction));
        self.register(Box::new(ProperFunction));
        self.register(Box::new(Rot13Function));
        self.register(Box::new(SoundexFunction));
        self.register(Box::new(PigLatinFunction));
        self.register(Box::new(MorseCodeFunction));
        self.register(Box::new(ScrambleFunction));

        // Number to words functions
        self.register(Box::new(ToWords));
        self.register(Box::new(ToOrdinal));
        self.register(Box::new(ToOrdinalWords));

        // Text processing functions
        self.register(Box::new(StripPunctuation));
        self.register(Box::new(Tokenize));
        self.register(Box::new(CleanText));
        self.register(Box::new(ExtractWords));
        self.register(Box::new(WordCount));
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
        use base_conversion::{
            FromBase, FromBinary, FromHex, FromOctal, ToBase, ToBinary, ToHex, ToOctal,
        };
        use integer_limits::{
            ByteMax, ByteMin, CharMax, CharMin, Int16Max, Int16Min, Int32Max, Int32Min, Int64Max,
            Int64Min, Int8Max, Int8Min, IntMax, IntMin, LongMax, LongMin, ShortMax, ShortMin,
            Uint16Max, Uint32Max, Uint8Max,
        };
        use mathematics::{
            IsPrimeFunction, NextPrimeFunction, NthPrimeFunction, PrevPrimeFunction,
            PrimeCountFunction, PrimeFunction, PrimePiFunction,
        };
        use trigonometry::{
            AcosFunction, AsinFunction, Atan2Function, AtanFunction, CosFunction, CoshFunction,
            CotFunction, SinFunction, SinhFunction, TanFunction, TanhFunction,
        };

        // Prime number functions
        self.register(Box::new(PrimeFunction));
        self.register(Box::new(NthPrimeFunction)); // Alias for PRIME
        self.register(Box::new(IsPrimeFunction));
        self.register(Box::new(PrimeCountFunction));
        self.register(Box::new(PrimePiFunction)); // Alias for PRIME_COUNT
        self.register(Box::new(NextPrimeFunction));
        self.register(Box::new(PrevPrimeFunction));

        // Trigonometric functions
        self.register(Box::new(SinFunction));
        self.register(Box::new(CosFunction));
        self.register(Box::new(TanFunction));
        self.register(Box::new(CotFunction));
        self.register(Box::new(AsinFunction));
        self.register(Box::new(AcosFunction));
        self.register(Box::new(AtanFunction));
        self.register(Box::new(Atan2Function));

        // Hyperbolic functions
        self.register(Box::new(SinhFunction));
        self.register(Box::new(CoshFunction));
        self.register(Box::new(TanhFunction));

        // Base conversion functions
        self.register(Box::new(ToBase));
        self.register(Box::new(FromBase));
        self.register(Box::new(ToBinary));
        self.register(Box::new(FromBinary));
        self.register(Box::new(ToHex));
        self.register(Box::new(FromHex));
        self.register(Box::new(ToOctal));
        self.register(Box::new(FromOctal));

        // Integer limit functions
        self.register(Box::new(Int8Min));
        self.register(Box::new(Int8Max));
        self.register(Box::new(Uint8Max));
        self.register(Box::new(Int16Min));
        self.register(Box::new(Int16Max));
        self.register(Box::new(Uint16Max));
        self.register(Box::new(Int32Min));
        self.register(Box::new(Int32Max));
        self.register(Box::new(Uint32Max));
        self.register(Box::new(Int64Min));
        self.register(Box::new(Int64Max));

        // Alias functions for common names
        self.register(Box::new(ByteMin));
        self.register(Box::new(ByteMax));
        self.register(Box::new(CharMin));
        self.register(Box::new(CharMax));
        self.register(Box::new(ShortMin));
        self.register(Box::new(ShortMax));
        self.register(Box::new(IntMin));
        self.register(Box::new(IntMax));
        self.register(Box::new(LongMin));
        self.register(Box::new(LongMax));

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

    /// Register financial functions
    fn register_financial_functions(&mut self) {
        financial::register_financial_functions(self);
    }

    /// Register conversion functions
    fn register_conversion_functions(&mut self) {
        use convert::ConvertFunction;
        use roman::{FromRoman, ToRoman};

        self.register(Box::new(ConvertFunction));
        self.register(Box::new(ToRoman));
        self.register(Box::new(FromRoman));
    }

    /// Register statistical functions
    fn register_statistical_functions(&mut self) {
        use statistics::{
            CorrelationFunction, KurtosisFunction, MedianFunction, ModeFunction,
            PercentileFunction, SkewFunction, VarPopFunction, VarSampFunction, VarianceFunction,
        };

        self.register(Box::new(MedianFunction));
        self.register(Box::new(PercentileFunction));
        self.register(Box::new(ModeFunction));
        self.register(Box::new(VarianceFunction));
        self.register(Box::new(VarSampFunction));
        self.register(Box::new(VarPopFunction));
        self.register(Box::new(CorrelationFunction));
        self.register(Box::new(SkewFunction));
        self.register(Box::new(KurtosisFunction));
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
            CenterFunction, DateFormatFunction, FormatDateFunction, FormatNumberFunction,
            LPadFunction, RPadFunction,
        };
        use format_number::{FormatBytesFunction, FormatCurrencyFunction, RenderNumberFunction};

        self.register(Box::new(FormatNumberFunction));
        self.register(Box::new(FormatDateFunction));
        self.register(Box::new(DateFormatFunction));
        self.register(Box::new(LPadFunction));
        self.register(Box::new(RPadFunction));
        self.register(Box::new(CenterFunction));
        self.register(Box::new(RenderNumberFunction));
        self.register(Box::new(FormatCurrencyFunction));
        self.register(Box::new(FormatBytesFunction));
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

    /// Register utility functions
    fn register_utility_functions(&mut self) {
        use utility::{
            AsciiFunction, CharFunction, DecodeFunction, EncodeFunction, OrdFunction,
            ToDecimalFunction, ToIntFunction, ToStringFunction, UnicodeFunction,
        };

        self.register(Box::new(AsciiFunction));
        self.register(Box::new(OrdFunction));
        self.register(Box::new(CharFunction));
        self.register(Box::new(ToIntFunction));
        self.register(Box::new(ToDecimalFunction));
        self.register(Box::new(ToStringFunction));
        self.register(Box::new(EncodeFunction));
        self.register(Box::new(DecodeFunction));
        self.register(Box::new(UnicodeFunction));
    }

    /// Register big integer and bit manipulation functions
    fn register_bigint_functions(&mut self) {
        use bigint::{
            BigAddFunction, BigFactorialFunction, BigIntFunction, BigMulFunction, BigPowFunction,
            BitAndFunction, BitOrFunction, BitShiftFunction, BitXorFunction, FromBinaryFunction,
            FromHexFunction, ToBinaryFunction, ToHexFunction,
        };

        // Arbitrary precision arithmetic
        self.register(Box::new(BigIntFunction));
        self.register(Box::new(BigAddFunction));
        self.register(Box::new(BigMulFunction));
        self.register(Box::new(BigPowFunction));
        self.register(Box::new(BigFactorialFunction));

        // Bit manipulation
        self.register(Box::new(BitAndFunction));
        self.register(Box::new(BitOrFunction));
        self.register(Box::new(BitXorFunction));
        self.register(Box::new(BitShiftFunction));

        // Base conversions
        self.register(Box::new(ToBinaryFunction));
        self.register(Box::new(FromBinaryFunction));
        self.register(Box::new(ToHexFunction));
        self.register(Box::new(FromHexFunction));
    }

    /// Register bitwise functions (additional bit operations not in bigint)
    fn register_bitwise_functions(&mut self) {
        bitwise::register_bitwise_functions(self);

        // Register string-based bitwise operations
        self.register(Box::new(bitwise_string::BitAndStr));
        self.register(Box::new(bitwise_string::BitOrStr));
        self.register(Box::new(bitwise_string::BitXorStr));
        self.register(Box::new(bitwise_string::BitNotStr));
        self.register(Box::new(bitwise_string::BitFlip));
        self.register(Box::new(bitwise_string::BitCount));
        self.register(Box::new(bitwise_string::BitRotateLeft));
        self.register(Box::new(bitwise_string::BitRotateRight));
        self.register(Box::new(bitwise_string::BitShiftLeft));
        self.register(Box::new(bitwise_string::BitShiftRight));
        self.register(Box::new(bitwise_string::HammingDistance));
    }

    /// Register ANSI terminal formatting functions
    fn register_ansi_functions(&mut self) {
        use ansi::{
            AnsiBgFunction, AnsiBlinkFunction, AnsiBoldFunction, AnsiColorFunction,
            AnsiItalicFunction, AnsiReverseFunction, AnsiRgbBgFunction, AnsiRgbFunction,
            AnsiStrikethroughFunction, AnsiUnderlineFunction,
        };

        // Color functions
        self.register(Box::new(AnsiColorFunction));
        self.register(Box::new(AnsiBgFunction));
        self.register(Box::new(AnsiRgbFunction));
        self.register(Box::new(AnsiRgbBgFunction));

        // Formatting functions
        self.register(Box::new(AnsiBoldFunction));
        self.register(Box::new(AnsiItalicFunction));
        self.register(Box::new(AnsiUnderlineFunction));
        self.register(Box::new(AnsiBlinkFunction));
        self.register(Box::new(AnsiReverseFunction));
        self.register(Box::new(AnsiStrikethroughFunction));
    }

    /// Register vector mathematics functions
    fn register_vector_functions(&mut self) {
        use vector::{
            ClosestPointOnLineFunction, LineIntersectFunction, LineReflectPointFunction,
            PointLineDistanceFunction, SegmentIntersectFunction, VecAddFunction, VecAngleFunction,
            VecCrossFunction, VecDistanceFunction, VecDotFunction, VecFunction, VecMagFunction,
            VecNormalizeFunction, VecScaleFunction, VecSubFunction,
        };

        // Vector construction
        self.register(Box::new(VecFunction));

        // Vector operations
        self.register(Box::new(VecAddFunction));
        self.register(Box::new(VecSubFunction));
        self.register(Box::new(VecScaleFunction));

        // Vector analysis
        self.register(Box::new(VecDotFunction));
        self.register(Box::new(VecMagFunction));
        self.register(Box::new(VecNormalizeFunction));
        self.register(Box::new(VecDistanceFunction));
        self.register(Box::new(VecCrossFunction));
        self.register(Box::new(VecAngleFunction));

        // Line geometry
        self.register(Box::new(LineIntersectFunction));
        self.register(Box::new(SegmentIntersectFunction));
        self.register(Box::new(ClosestPointOnLineFunction));
        self.register(Box::new(PointLineDistanceFunction));
        self.register(Box::new(LineReflectPointFunction));
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

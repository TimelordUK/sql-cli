use crate::data::datatable::{DataColumn, DataRow, DataTable, DataValue};
use anyhow::Result;
use std::sync::Arc;

pub mod literal_generators;
pub mod math_generators;
pub mod prime_generators;
pub mod random_generators;
pub mod sequence_generators;
pub mod string_generators;

/// Trait for table-generating functions that produce rows dynamically
pub trait TableGenerator: Send + Sync {
    /// Get the name of the generator function (e.g., "GENERATE_PRIMES")
    fn name(&self) -> &str;

    /// Get the column definitions for the generated table
    fn columns(&self) -> Vec<DataColumn>;

    /// Generate the table based on the provided arguments
    /// Arguments are evaluated expressions from the SQL query
    fn generate(&self, args: Vec<DataValue>) -> Result<Arc<DataTable>>;

    /// Get a description of what this generator does
    fn description(&self) -> &str;

    /// Get the expected number of arguments
    fn arg_count(&self) -> usize;
}

/// Registry for table generator functions
pub struct GeneratorRegistry {
    generators: std::collections::HashMap<String, Box<dyn TableGenerator>>,
}

impl GeneratorRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            generators: std::collections::HashMap::new(),
        };
        registry.register_default_generators();
        registry
    }

    fn register_default_generators(&mut self) {
        use literal_generators::{Array, Values};
        use math_generators::{Collatz, Factorials, PascalTriangle, Squares, TriangularNumbers};
        use prime_generators::{Fibonacci, GeneratePrimes, PrimeFactors};
        use random_generators::{GenerateUUIDs, RandomFloats, RandomIntegers};
        use sequence_generators::{Dates, Range, Series};
        use string_generators::{Chars, Lines, Split, Tokenize};

        // Literal value generators
        self.register(Box::new(Values));
        self.register(Box::new(Array));

        // Sequence generators
        self.register(Box::new(Range));
        self.register(Box::new(Series));
        self.register(Box::new(Dates));

        // String generators
        self.register(Box::new(Split));
        self.register(Box::new(Tokenize));
        self.register(Box::new(Chars));
        self.register(Box::new(Lines));

        // Prime and number theory generators
        self.register(Box::new(GeneratePrimes));
        self.register(Box::new(PrimeFactors));
        self.register(Box::new(Fibonacci));

        // Mathematical sequence generators
        self.register(Box::new(Collatz));
        self.register(Box::new(PascalTriangle));
        self.register(Box::new(TriangularNumbers));
        self.register(Box::new(Squares));
        self.register(Box::new(Factorials));

        // Random generators
        self.register(Box::new(RandomIntegers));
        self.register(Box::new(RandomFloats));
        self.register(Box::new(GenerateUUIDs));
    }

    pub fn register(&mut self, generator: Box<dyn TableGenerator>) {
        self.generators
            .insert(generator.name().to_uppercase(), generator);
    }

    pub fn get(&self, name: &str) -> Option<&Box<dyn TableGenerator>> {
        self.generators.get(&name.to_uppercase())
    }

    pub fn list(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.generators.keys().map(|s| s.as_str()).collect();
        names.sort();
        names
    }

    pub fn list_generators_formatted(&self) -> String {
        let mut output = String::new();
        output.push_str("=== Available Generator Functions ===\n\n");

        // Group generators by category
        let mut sequence_gens = Vec::new();
        let mut string_gens = Vec::new();
        let mut math_gens = Vec::new();
        let mut random_gens = Vec::new();
        let mut utility_gens = Vec::new();

        for (name, gen) in &self.generators {
            let entry = format!("  {} - {}", name, gen.description());

            if name == "RANGE" || name == "SERIES" || name == "DATES" {
                sequence_gens.push(entry);
            } else if name == "SPLIT" || name == "TOKENIZE" || name == "CHARS" || name == "LINES" {
                string_gens.push(entry);
            } else if name.starts_with("RANDOM_") {
                random_gens.push(entry);
            } else if name == "GENERATE_UUID" {
                utility_gens.push(entry);
            } else {
                math_gens.push(entry);
            }
        }

        if !sequence_gens.is_empty() {
            sequence_gens.sort();
            output.push_str("Sequence Generators:\n");
            for entry in sequence_gens {
                output.push_str(&format!("{}\n", entry));
            }
            output.push('\n');
        }

        if !string_gens.is_empty() {
            string_gens.sort();
            output.push_str("String Generators:\n");
            for entry in string_gens {
                output.push_str(&format!("{}\n", entry));
            }
            output.push('\n');
        }

        if !math_gens.is_empty() {
            math_gens.sort();
            output.push_str("Mathematical Generators:\n");
            for entry in math_gens {
                output.push_str(&format!("{}\n", entry));
            }
            output.push('\n');
        }

        if !random_gens.is_empty() {
            random_gens.sort();
            output.push_str("Random Generators:\n");
            for entry in random_gens {
                output.push_str(&format!("{}\n", entry));
            }
            output.push('\n');
        }

        if !utility_gens.is_empty() {
            utility_gens.sort();
            output.push_str("Utility Generators:\n");
            for entry in utility_gens {
                output.push_str(&format!("{}\n", entry));
            }
            output.push('\n');
        }

        output.push_str("Use: SELECT * FROM <generator>(<args>)\n");
        output.push_str("Example: SELECT * FROM GENERATE_PRIMES(100)\n");
        output
    }

    pub fn get_generator_help(&self, name: &str) -> Option<String> {
        self.generators.get(&name.to_uppercase()).map(|gen| {
            let mut help = String::new();
            help.push_str(&format!("=== {} ===\n\n", name.to_uppercase()));
            help.push_str(&format!("Description: {}\n", gen.description()));
            help.push_str(&format!(
                "Arguments: {} argument(s) expected\n",
                gen.arg_count()
            ));
            help.push_str("\nColumns:\n");
            for col in gen.columns() {
                help.push_str(&format!("  - {}\n", col.name));
            }
            help.push_str("\nExample:\n");
            help.push_str(&format!("  SELECT * FROM {}(", name.to_uppercase()));

            // Add example arguments based on the generator
            match name.to_uppercase().as_str() {
                "GENERATE_PRIMES" => help.push_str("100"),
                "FIBONACCI" => help.push_str("20"),
                "PRIME_FACTORS" => help.push_str("1260"),
                "COLLATZ" => help.push_str("7"),
                "PASCAL_TRIANGLE" => help.push_str("5"),
                "TRIANGULAR" | "SQUARES" | "FACTORIALS" => help.push_str("10"),
                "RANDOM_INT" => help.push_str("10, 1, 100, 42"),
                "RANDOM_FLOAT" => help.push_str("10, 0, 1, 42"),
                "GENERATE_UUID" => help.push_str("5"),
                _ => help.push_str("..."),
            }
            help.push_str(");\n");
            help
        })
    }
}

/// Helper function to create a single-column table
pub fn create_single_column_table(
    name: &str,
    column_name: &str,
    values: Vec<DataValue>,
) -> Arc<DataTable> {
    let mut table = DataTable::new(name);
    table.add_column(DataColumn::new(column_name));

    for value in values {
        table.add_row(DataRow::new(vec![value])).unwrap();
    }

    Arc::new(table)
}

/// Helper function to create a two-column table
pub fn create_two_column_table(
    name: &str,
    col1_name: &str,
    col2_name: &str,
    rows: Vec<(DataValue, DataValue)>,
) -> Arc<DataTable> {
    let mut table = DataTable::new(name);
    table.add_column(DataColumn::new(col1_name));
    table.add_column(DataColumn::new(col2_name));

    for (val1, val2) in rows {
        table.add_row(DataRow::new(vec![val1, val2])).unwrap();
    }

    Arc::new(table)
}

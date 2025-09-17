use crate::data::datatable::{DataColumn, DataRow, DataTable, DataValue};
use anyhow::Result;
use std::sync::Arc;

pub mod math_generators;
pub mod prime_generators;
pub mod random_generators;

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
        use math_generators::{Collatz, Factorials, PascalTriangle, Squares, TriangularNumbers};
        use prime_generators::{Fibonacci, GeneratePrimes, PrimeFactors};
        use random_generators::{GenerateUUIDs, RandomFloats, RandomIntegers};

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
        self.generators.keys().map(|s| s.as_str()).collect()
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

use crate::data::DataValue;
use crate::sql::functions::{ArgCount, FunctionCategory, FunctionSignature, SqlFunction};
use anyhow::{anyhow, Result};

/// RANGE - Generate numeric sequence (table function)
/// Note: This is a special table function that returns a table, not a scalar value
pub struct RangeFunction;

impl SqlFunction for RangeFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "RANGE",
            category: FunctionCategory::TableFunction,
            arg_count: ArgCount::Range(1, 3),
            description: "Generate a numeric sequence as a table",
            returns: "Table with 'value' column containing integers",
            examples: vec![
                "SELECT * FROM RANGE(10)                       -- 1 to 10",
                "SELECT * FROM RANGE(5, 10)                    -- 5 to 10",
                "SELECT * FROM RANGE(10, 1, -1)                -- 10 down to 1",
                "SELECT * FROM RANGE(0, 100, 5)                -- 0, 5, 10, ..., 100",
                "SELECT value * 2 FROM RANGE(1, 5) AS r        -- 2, 4, 6, 8, 10",
            ],
        }
    }

    fn evaluate(&self, _args: &[DataValue]) -> Result<DataValue> {
        // This is a table function - it's handled specially by the parser/executor
        // This evaluate method won't actually be called
        Err(anyhow!("RANGE is a table function - use it in FROM clause: SELECT * FROM RANGE(...) "))
    }
}

/// SPLIT - Split string into rows (table function)
/// Note: This is a special table function that returns a table, not a scalar value
pub struct SplitFunction;

impl SqlFunction for SplitFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "SPLIT",
            category: FunctionCategory::TableFunction,
            arg_count: ArgCount::Range(1, 2),
            description: "Split a string into rows based on delimiter",
            returns: "Table with 'value' and 'index' columns",
            examples: vec![
                "SELECT * FROM SPLIT('hello world')             -- Split on spaces",
                "SELECT * FROM SPLIT('a,b,c', ',')              -- Split on comma",
                "SELECT * FROM SPLIT('one-two-three', '-')      -- Split on dash",
                "SELECT value FROM SPLIT('foo bar baz')         -- Just the values",
                "SELECT UPPER(value) FROM SPLIT('hello world')  -- Transform values",
            ],
        }
    }

    fn evaluate(&self, _args: &[DataValue]) -> Result<DataValue> {
        // This is a table function - it's handled specially by the parser/executor
        // This evaluate method won't actually be called
        Err(anyhow!("SPLIT is a table function - use it in FROM clause: SELECT * FROM SPLIT(...) "))
    }
}

/// VALUES - Create table from literal values (table function)
/// Note: This is a special table function that returns a table, not a scalar value
pub struct ValuesFunction;

impl SqlFunction for ValuesFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "VALUES",
            category: FunctionCategory::TableFunction,
            arg_count: ArgCount::Variadic,
            description: "Create a table from literal values",
            returns: "Table with columns from the provided values",
            examples: vec![
                "SELECT * FROM (VALUES (1, 'a'), (2, 'b'), (3, 'c')) AS t(id, name)",
                "SELECT * FROM (VALUES (1), (2), (3)) AS t(n)",
                "SELECT column1, column2 FROM (VALUES (10, 20), (30, 40))",
            ],
        }
    }

    fn evaluate(&self, _args: &[DataValue]) -> Result<DataValue> {
        // This is a table function - it's handled specially by the parser/executor
        Err(anyhow!("VALUES is a table function - use it in FROM clause"))
    }
}
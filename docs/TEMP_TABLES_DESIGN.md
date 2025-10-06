# Temporary Tables Design (#tmp Tables)

## Overview
This document outlines the design for temporary tables that persist across query batches within a script execution, enabling query result reuse and dynamic data injection into subsequent WEB CTEs.

## Use Cases

### Phase 1: Basic Persistence
```sql
-- Fetch trade data once, reuse multiple times
WITH WEB trades AS (
    URL 'https://api.example.com/trades'
    BODY { "date": "2024-01-15" }
    FORMAT JSON
)
SELECT * FROM trades INTO #tmp_trades;
GO

-- Later queries can reference #tmp_trades without refetching
SELECT COUNT(*) FROM #tmp_trades WHERE status = 'completed';
GO

SELECT symbol, AVG(price) FROM #tmp_trades GROUP BY symbol;
GO
```

### Phase 2: Dynamic Query Injection
```sql
-- Extract trade IDs from first query
WITH WEB trades AS (...)
SELECT trade_id FROM trades INTO #trade_ids;
GO

-- Use those IDs in next query's body
WITH WEB allocations AS (
    URL 'https://api.example.com/allocations'
    BODY {
        "trade_ids": #trade_ids  -- Expanded to array
    }
    FORMAT JSON
)
SELECT * FROM allocations;
GO
```

## Architecture

### 1. Current Script Execution Flow

```
┌─────────────────────────────────────────┐
│  Script File                             │
│  --------------------------------        │
│  SELECT ... FROM web_cte INTO #tmp;      │
│  GO                                      │
│  SELECT * FROM #tmp;                     │
│  GO                                      │
└─────────────────┬───────────────────────┘
                  ▼
┌─────────────────────────────────────────┐
│  ScriptParser::parse_statements()       │
│  Splits on "GO" separator                │
└─────────────────┬───────────────────────┘
                  ▼
┌─────────────────────────────────────────┐
│  non_interactive::execute_script()       │
│  Loop: for each statement                │
│    1. Parse SQL                          │
│    2. Execute query                      │
│    3. Display results                    │
│    4. **Context is lost**                │
└─────────────────────────────────────────┘
```

**Problem**: Each statement executes independently - no shared state between queries.

### 2. New Architecture with Temp Tables

```
┌─────────────────────────────────────────┐
│  TempTableRegistry (New)                 │
│  --------------------------------        │
│  - HashMap<String, Arc<DataTable>>       │
│  - Lives for script duration             │
│  - Not persisted to Redis                │
│  - Dropped after script completes        │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│  Enhanced execute_script()               │
│  --------------------------------        │
│  let mut temp_tables = TempTableRegistry │
│                                          │
│  for statement in statements:            │
│    1. Parse SQL                          │
│    2. Check for INTO #tmp                │
│    3. Execute with temp_tables context   │
│    4. If INTO: store result              │
│    5. If FROM #tmp: resolve from registry│
└─────────────────┬───────────────────────┘
                  ▼
┌─────────────────────────────────────────┐
│  QueryEngine (Enhanced)                  │
│  --------------------------------        │
│  - Accept TempTableRegistry              │
│  - Resolve #tmp in FROM clause           │
│  - Treat #tmp like CTE                   │
└─────────────────────────────────────────┘
```

## Implementation Plan

### Phase 1: Basic Persistence (Immediate Value)

#### 1.1 Add TempTableRegistry

```rust
// src/data/temp_table_registry.rs
use std::collections::HashMap;
use std::sync::Arc;
use crate::data::datatable::DataTable;

pub struct TempTableRegistry {
    tables: HashMap<String, Arc<DataTable>>,
}

impl TempTableRegistry {
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
        }
    }

    /// Store a temporary table
    /// Returns error if table already exists
    pub fn insert(&mut self, name: String, table: Arc<DataTable>) -> Result<()> {
        if self.tables.contains_key(&name) {
            // Option 1: Error on duplicate
            return Err(anyhow!("Temporary table {} already exists", name));

            // Option 2: Drop and replace (SQL Server behavior)
            // self.tables.insert(name, table);
        } else {
            self.tables.insert(name, table);
        }
        Ok(())
    }

    /// Get a temporary table by name
    pub fn get(&self, name: &str) -> Option<Arc<DataTable>> {
        self.tables.get(name).cloned()
    }

    /// Check if a temporary table exists
    pub fn contains(&self, name: &str) -> bool {
        self.tables.contains_key(name)
    }

    /// Drop a temporary table
    pub fn drop(&mut self, name: &str) -> bool {
        self.tables.remove(name).is_some()
    }

    /// Get count of temp tables
    pub fn count(&self) -> usize {
        self.tables.len()
    }

    /// Clear all temp tables
    pub fn clear(&mut self) {
        self.tables.clear();
    }
}
```

#### 1.2 Extend AST for SELECT INTO

```rust
// In src/sql/parser/ast.rs

#[derive(Debug, Clone, PartialEq)]
pub struct SelectStatement {
    // ... existing fields ...

    /// Optional INTO clause for creating temporary tables
    pub into_table: Option<IntoTable>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IntoTable {
    /// Name of the temporary table (with # prefix)
    pub name: String,
    /// Whether to replace if exists
    pub replace_if_exists: bool,
}
```

#### 1.3 Parser Enhancement

```rust
// In src/sql/parser/mod.rs or recursive_parser.rs

impl Parser {
    fn parse_select_statement(&mut self) -> Result<SelectStatement> {
        // Parse SELECT ...
        let select_items = self.parse_select_items()?;

        // Parse FROM ...
        let from = self.parse_from_clause()?;

        // NEW: Check for INTO clause
        let into_table = if self.current_token_is_keyword("INTO") {
            self.consume_keyword("INTO")?;
            Some(self.parse_into_clause()?)
        } else {
            None
        };

        // Continue with WHERE, GROUP BY, etc.
        // ...

        Ok(SelectStatement {
            // ... existing fields ...
            into_table,
        })
    }

    fn parse_into_clause(&mut self) -> Result<IntoTable> {
        // Expect identifier starting with #
        let name = self.expect_identifier()?;

        if !name.starts_with('#') {
            return Err(anyhow!("Temporary table must start with #"));
        }

        // Future: Handle "OR REPLACE" if needed
        Ok(IntoTable {
            name,
            replace_if_exists: false,
        })
    }
}
```

#### 1.4 Script Execution Enhancement

```rust
// In src/non_interactive.rs

pub fn execute_script(config: NonInteractiveConfig) -> Result<()> {
    let parser = ScriptParser::new(&config.query);
    let statements = parser.parse_and_validate()?;

    // NEW: Create temp table registry for script duration
    let mut temp_tables = TempTableRegistry::new();
    let mut script_result = ScriptResult::new();

    // Load initial data file if provided
    let mut base_table = if !config.data_file.is_empty() {
        Some(load_data_file(&config.data_file)?)
    } else {
        None
    };

    for (idx, sql) in statements.iter().enumerate() {
        let start = Instant::now();

        // Parse the statement
        let mut parser = Parser::new(sql);
        let statement = parser.parse_select()?;

        // Execute with temp table context
        let result = execute_with_temp_tables(
            &statement,
            &base_table,
            &mut temp_tables,
            &config
        );

        match result {
            Ok(data_view) => {
                let rows = data_view.row_count();

                // If this is an INTO statement, store the result
                if let Some(into) = &statement.into_table {
                    temp_tables.insert(
                        into.name.clone(),
                        Arc::new(data_view.to_datatable())
                    )?;

                    println!("({} rows affected) -> {}", rows, into.name);
                } else {
                    // Regular output
                    output_result(&data_view, &config)?;
                }

                script_result.add_success(idx + 1, sql.clone(), rows, start.elapsed());
            }
            Err(e) => {
                script_result.add_failure(idx + 1, sql.clone(), e.to_string(), start.elapsed());
                if !config.continue_on_error {
                    break;
                }
            }
        }
    }

    // Cleanup: temp_tables dropped when going out of scope
    Ok(())
}

fn execute_with_temp_tables(
    statement: &SelectStatement,
    base_table: &Option<DataTable>,
    temp_tables: &mut TempTableRegistry,
    config: &NonInteractiveConfig
) -> Result<DataView> {
    // Check if FROM clause references a temp table
    let table = if let Some(from_table_name) = &statement.from_table {
        if from_table_name.starts_with('#') {
            // Resolve from temp table registry
            let temp_table = temp_tables.get(from_table_name)
                .ok_or_else(|| anyhow!("Temporary table {} not found", from_table_name))?;
            Some((*temp_table).clone())
        } else {
            base_table.clone()
        }
    } else {
        base_table.clone()
    };

    // Execute query with resolved table
    let engine = QueryEngine::new();
    engine.execute(&table, statement)
}
```

#### 1.5 Query Engine Enhancement

Currently query engine receives a `table: Option<DataTable>`. Need to enhance to resolve temp tables in CTE context:

```rust
// In src/data/query_engine.rs

impl QueryEngine {
    pub fn execute_with_temp_tables(
        &self,
        table: &Option<DataTable>,
        statement: &SelectStatement,
        temp_tables: &TempTableRegistry,
    ) -> Result<DataView> {
        // When processing CTEs and FROM clauses, check temp_tables first
        // Treat #tmp tables like CTEs - they're already materialized

        // ... existing execution logic ...
    }
}
```

## Phase 2: Dynamic Query Injection (Future Enhancement)

### Use Case Expanded

```sql
-- Step 1: Extract subset of data
WITH WEB trades AS (
    URL 'https://api.example.com/trades'
    BODY { "date": "2024-01-15", "status": "pending" }
    FORMAT JSON
)
SELECT trade_id FROM trades INTO #pending_ids;
GO

-- Step 2: Use those IDs to fetch allocations
WITH WEB allocations AS (
    URL 'https://api.example.com/allocations'
    BODY {
        "trade_ids": #pending_ids,          -- Array injection
        "mode": "detailed"
    }
    FORMAT JSON
)
SELECT * FROM allocations;
GO
```

### Body Template Expansion

```rust
// In web_cte_parser.rs or http_fetcher.rs

fn expand_body_with_temp_tables(
    body: &str,
    temp_tables: &TempTableRegistry
) -> Result<String> {
    // Parse body as JSON/template
    let mut body_value: serde_json::Value = serde_json::from_str(body)?;

    // Find references to #table_name
    expand_references(&mut body_value, temp_tables)?;

    Ok(serde_json::to_string(&body_value)?)
}

fn expand_references(
    value: &mut serde_json::Value,
    temp_tables: &TempTableRegistry
) -> Result<()> {
    match value {
        Value::String(s) if s.starts_with('#') => {
            // Replace with table data
            let table = temp_tables.get(s)
                .ok_or_else(|| anyhow!("Temp table {} not found", s))?;

            // Convert first column to array
            let array = table.rows.iter()
                .map(|row| row.values[0].clone())
                .collect::<Vec<_>>();

            *value = serde_json::to_value(array)?;
        }
        Value::Object(map) => {
            for (_, v) in map.iter_mut() {
                expand_references(v, temp_tables)?;
            }
        }
        Value::Array(arr) => {
            for v in arr.iter_mut() {
                expand_references(v, temp_tables)?;
            }
        }
        _ => {}
    }
    Ok(())
}
```

### Advanced Templating (Future)

```sql
-- More flexible with explicit column selection
WITH WEB orders AS (
    URL 'https://api.example.com/orders'
    BODY {
        "customer_ids": #customers.customer_id,    -- Specific column
        "include_details": true,
        "filters": {
            "regions": #active_regions.region_code  -- Another column
        }
    }
    FORMAT JSON
)
SELECT * FROM orders;
```

## Benefits

### Phase 1 Benefits (Immediate)
1. **Code Reuse**: Write WEB CTE once, query results multiple ways
2. **Performance**: Avoid redundant HTTP calls
3. **Clarity**: Separate data fetching from analysis
4. **Debugging**: Inspect intermediate results

### Phase 2 Benefits (Future)
1. **Dynamic Queries**: Build queries based on previous results
2. **Data Pipelines**: Multi-stage ETL-style workflows
3. **Batch Operations**: Process data in chunks
4. **Complex Integrations**: Chain multiple API calls with dependencies

## Implementation Complexity

### Phase 1: **Medium** (1-2 days)
- ✅ Clear scope and boundaries
- ✅ Existing patterns to follow (CTEs)
- ✅ No Redis integration needed
- ✅ Minimal parser changes

**Risks**: Low
- Parser changes are straightforward (INTO clause)
- TempTableRegistry is simple HashMap wrapper
- Script execution already loops over statements

### Phase 2: **High** (3-5 days)
- ⚠️ JSON template parsing complexity
- ⚠️ Need robust error handling
- ⚠️ Multiple expansion strategies needed
- ⚠️ Type coercion challenges

**Risks**: Medium
- Need to handle various data types (strings, numbers, nulls)
- Template syntax needs careful design
- Error messages must be clear

## Recommended Approach

### Step 1: Phase 1 MVP (Start Here)
1. Implement TempTableRegistry
2. Add INTO clause parsing
3. Enhance execute_script() to maintain registry
4. Add temp table resolution in FROM clauses
5. Test with basic scripts

**Success Criteria**:
- Can save WEB CTE results to #tmp
- Can query #tmp in subsequent statements
- Temp tables cleared after script ends

### Step 2: Evaluate Phase 1 (After Testing)
- Gather feedback on syntax and UX
- Identify real-world use cases
- Determine if Phase 2 is needed
- Refine Phase 2 design based on usage patterns

### Step 3: Phase 2 Design Refinement
- Design template expansion syntax
- Decide on column selection syntax
- Plan type handling strategy
- Build prototype with limited scope

## Alternative Syntaxes Considered

### SQL Server Style
```sql
SELECT * INTO #tmp FROM web_trades;  -- SQL Server
SELECT * FROM web_trades INTO #tmp;  -- Our proposed syntax
```
**Decision**: Use "INTO #tmp" after FROM for consistency with CTEs

### Explicit Declaration
```sql
DECLARE @tmp AS TEMP TABLE;
SELECT * FROM web_trades INTO @tmp;
```
**Decision**: Too verbose, # prefix is sufficient

### Hash vs At
```sql
#tmp  -- Unix/SQL Server style (chosen)
@tmp  -- Variable style
$tmp  -- Shell style
```
**Decision**: # prefix is most SQL-like

## Testing Strategy

### Unit Tests
- TempTableRegistry operations
- Parser handles INTO clause
- AST includes into_table field

### Integration Tests
```sql
-- Test 1: Basic persistence
SELECT * FROM trades INTO #t1;
GO
SELECT COUNT(*) FROM #t1;
GO

-- Test 2: Error on duplicate
SELECT * FROM trades INTO #t1;
GO
SELECT * FROM orders INTO #t1;  -- Should error
GO

-- Test 3: Multiple temp tables
SELECT * FROM trades INTO #t1;
GO
SELECT * FROM orders INTO #t2;
GO
SELECT * FROM #t1 JOIN #t2 ON ...;
GO
```

### Performance Tests
- Memory usage with large temp tables
- Impact on script execution time
- Cleanup verification

## Documentation Needs

1. **User Guide**: How to use temp tables in scripts
2. **Examples**: Real-world use cases
3. **API Docs**: TempTableRegistry methods
4. **Migration**: From multiple WEB CTEs to temp tables

## Future Enhancements (Beyond Phase 2)

1. **Indexing**: `CREATE INDEX ON #tmp (column)`
2. **Explicit Drops**: `DROP TABLE #tmp`
3. **Table Variables**: `DECLARE @t TABLE (...)`
4. **Cross-Script Persistence**: Named temp tables in ~/.sql-cli/temp/
5. **Memory Limits**: Max size for temp tables

## Conclusion

**Phase 1 is highly feasible and provides immediate value** with:
- Clear scope and implementation path
- Low risk and complexity
- Leverages existing architecture
- Solves real pain point (redundant WEB CTEs)

**Phase 2 requires careful design** but is architecturally sound:
- Can build incrementally on Phase 1
- Template expansion has clear patterns
- Provides powerful data pipeline capabilities

**Recommendation**: Proceed with Phase 1 implementation, evaluate adoption, then refine Phase 2 design based on real-world usage patterns.

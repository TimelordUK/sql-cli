# Column Scoping Implementation Design

## Overview
Add proper table-qualified column references (e.g., `messages.field_name`, `fields.description`) to enable unambiguous column resolution in joins and complex queries.

## Current Limitations
- Cannot use table prefixes: `SELECT messages.field_name` fails
- Cannot use aliases: `SELECT m.field_name FROM messages m` fails
- Ambiguous columns in joins require workarounds
- Table prefixes are lost during query execution

## Implementation Plan

### Phase 1: Column Metadata Enrichment (Non-breaking)

#### 1.1 Extend DataColumn Structure
```rust
// In src/data/datatable.rs
pub struct DataColumn {
    pub name: String,
    pub data_type: DataType,
    pub nullable: bool,
    // NEW: Qualified name with table prefix
    pub qualified_name: Option<String>,
    // NEW: Source table/CTE name
    pub source_table: Option<String>,
}
```

#### 1.2 Enrich During Data Loading
```rust
// In src/data/query_engine.rs - when loading CTEs
fn load_cte(&self, cte_name: &str, spec: &WebCTESpec) -> DataTable {
    let mut table = fetcher.fetch(spec, cte_name)?;

    // NEW: Enrich columns with qualified names
    for column in table.columns_mut() {
        column.qualified_name = Some(format!("{}.{}", cte_name, column.name));
        column.source_table = Some(cte_name.to_string());
    }

    table
}
```

#### 1.3 Preserve Through Operations
- DataView operations should maintain qualified names
- Join operations should preserve source table information
- Group by and aggregates need to handle qualified names

### Phase 2: Parser Enhancement

#### 2.1 Extend ColumnRef in AST
```rust
// In src/sql/parser/ast.rs
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ColumnRef {
    pub name: String,
    pub quote_style: QuoteStyle,
    // NEW: Optional table/alias prefix
    pub table_prefix: Option<String>,
}

impl ColumnRef {
    pub fn qualified(table: String, name: String) -> Self {
        Self {
            name,
            quote_style: QuoteStyle::None,
            table_prefix: Some(table),
        }
    }

    pub fn to_qualified_string(&self) -> String {
        match &self.table_prefix {
            Some(table) => format!("{}.{}", table, self.name),
            None => self.name.clone(),
        }
    }
}
```

#### 2.2 Parser Updates
```rust
// In src/sql/recursive_parser.rs
fn parse_column_ref(&mut self) -> Result<ColumnRef, String> {
    let first_part = self.parse_identifier()?;

    // Check for dot notation
    if self.current_token == Token::Dot {
        self.advance(); // consume dot
        let column_name = self.parse_identifier()?;

        Ok(ColumnRef::qualified(first_part, column_name))
    } else {
        Ok(ColumnRef::unquoted(first_part))
    }
}
```

### Phase 3: Column Resolution Logic

#### 3.1 Resolution Strategy
```rust
// In src/data/arithmetic_evaluator.rs
fn resolve_column(&self, col_ref: &ColumnRef, table: &DataTable) -> Result<usize> {
    // 1. Try exact qualified match
    if let Some(prefix) = &col_ref.table_prefix {
        let qualified = format!("{}.{}", prefix, col_ref.name);
        if let Some(idx) = table.find_column_by_qualified_name(&qualified) {
            return Ok(idx);
        }
    }

    // 2. Try unqualified match (backward compatibility)
    if let Some(idx) = table.find_column(&col_ref.name) {
        return Ok(idx);
    }

    // 3. Error with helpful message
    Err(format!("Column '{}' not found", col_ref.to_qualified_string()))
}
```

#### 3.2 Join Resolution
```rust
// In src/data/query_executor.rs
fn execute_join(&self, left: DataView, right: DataView, condition: &JoinCondition)
    -> Result<DataView> {

    // Preserve qualified names in joined result
    let mut result_columns = Vec::new();

    // Add left table columns with qualification
    for col in left.columns() {
        let mut new_col = col.clone();
        if new_col.qualified_name.is_none() {
            new_col.qualified_name = Some(format!("{}.{}", left_table_name, col.name));
        }
        result_columns.push(new_col);
    }

    // Add right table columns with qualification
    for col in right.columns() {
        let mut new_col = col.clone();
        if new_col.qualified_name.is_none() {
            new_col.qualified_name = Some(format!("{}.{}", right_table_name, col.name));
        }
        result_columns.push(new_col);
    }

    // ... rest of join logic
}
```

### Phase 4: Alias Support

#### 4.1 Track Table Aliases
```rust
// In src/sql/parser/ast.rs
#[derive(Debug, Clone)]
pub struct TableSource {
    Table(String),
    DerivedTable {
        query: Box<SelectStatement>,
        alias: String,
    },
    // NEW: Track alias for regular tables too
    TableWithAlias {
        name: String,
        alias: String,
    },
}
```

#### 4.2 Alias Resolution
```rust
// In query_executor.rs - maintain alias map during execution
struct ExecutionContext {
    // Map from alias to actual table name
    alias_map: HashMap<String, String>,
    // Tables by their actual names
    tables: HashMap<String, Arc<DataView>>,
}

fn resolve_table_reference(&self, name: &str) -> Option<&str> {
    // Check if it's an alias first
    self.alias_map.get(name)
        .map(|s| s.as_str())
        .or(Some(name))
}
```

## Testing Strategy

### Test Cases
1. **Basic Qualified Columns**
   ```sql
   WITH WEB messages AS (URL 'file://data.csv')
   SELECT messages.field_name FROM messages
   ```

2. **Join with Qualified Columns**
   ```sql
   WITH
     WEB msg AS (URL 'file://messages.csv'),
     WEB fld AS (URL 'file://fields.csv')
   SELECT msg.name, fld.description
   FROM msg
   JOIN fld ON msg.field_number = fld.number
   ```

3. **Alias Support**
   ```sql
   WITH WEB messages AS (URL 'file://data.csv')
   SELECT m.field_name
   FROM messages m
   WHERE m.required = 'Y'
   ```

4. **Mixed Qualified/Unqualified** (backward compatibility)
   ```sql
   SELECT
     messages.field_name,  -- qualified
     required              -- unqualified
   FROM messages
   ```

## Implementation Order

1. **Day 1: Foundation**
   - Extend DataColumn structure
   - Add qualified names during CTE loading
   - Test backward compatibility

2. **Day 2: Parser**
   - Add table_prefix to ColumnRef
   - Parse dot notation
   - Update AST formatter

3. **Day 3: Resolution**
   - Implement column resolution logic
   - Handle joins with qualification
   - Add alias support

4. **Day 4: Testing & Polish**
   - Comprehensive test suite
   - Update examples
   - Documentation

## Backward Compatibility

- All existing queries must continue to work
- Unqualified column names remain the default
- Qualified names are optional enhancements
- No breaking changes to public APIs

## Future Enhancements

1. **Star Expansion with Qualification**
   ```sql
   SELECT messages.*, fields.description
   FROM messages JOIN fields ON ...
   ```

2. **Schema/Database Qualification**
   ```sql
   SELECT db1.messages.field_name
   FROM db1.messages
   ```

3. **CTE Column Lists**
   ```sql
   WITH msg(id, name, type) AS (
     URL 'file://data.csv'
   )
   SELECT msg.id FROM msg
   ```

## Success Criteria

- [x] FIX protocol joins work with proper scoping
- [x] No existing queries break
- [x] Clear error messages for ambiguous columns
- [x] Performance remains unchanged
- [x] Examples demonstrate the new capability
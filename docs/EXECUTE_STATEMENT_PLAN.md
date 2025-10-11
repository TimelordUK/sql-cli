# Execute Statement Feature Plan

## Context

The nvim plugin sends SQL scripts to sql-cli, and we want to be able to execute a specific statement within a multi-statement script while automatically handling dependencies (especially temporary tables created by earlier statements).

## User Story

As a user in nvim, I want to:
1. Have a SQL script with multiple statements (some creating temp tables, CTEs, etc.)
2. Position my cursor on statement #3
3. Run `\sx` (execute statement)
4. Have sql-cli automatically:
   - Parse the entire script
   - Compute dependencies for statement #3
   - Execute only the minimal subset of statements needed (e.g., if statement #3 needs a temp table from statement #1, execute statements 1 and 3, but skip statement #2 if it's unrelated)

## Feature: `--execute-statement <N>`

### Usage
```bash
sql-cli -f script.sql --execute-statement 3
```

This will:
1. Parse the entire script
2. Build a dependency graph
3. Identify which statements statement #3 depends on
4. Execute the minimal subset in the correct order
5. Return results from statement #3

### Example Script
```sql
-- Statement 1: Create temp table
CREATE TEMP TABLE raw_data AS
SELECT * FROM sales WHERE year = 2024;
GO

-- Statement 2: Unrelated query (should be skipped)
SELECT COUNT(*) FROM customers;
GO

-- Statement 3: Uses temp table from statement 1
SELECT product, SUM(amount) as total
FROM raw_data
GROUP BY product
ORDER BY total DESC;
GO

-- Statement 4: Another temp table
CREATE TEMP TABLE summary AS
SELECT * FROM raw_data WHERE amount > 1000;
GO
```

Running `--execute-statement 3` should:
- Execute statement 1 (creates `raw_data` temp table)
- Skip statement 2 (unrelated)
- Execute statement 3 (target statement, uses `raw_data`)
- Skip statement 4 (not needed)
- Return results from statement 3

## Implementation Plan

### Phase 1: Dependency Analysis (NEXT SESSION)
**Goal**: Compute the minimal subset of statements needed

1. **Parse script into statements**
   - Already have: `sql_cli::sql::script_parser` with GO separator support
   - Split script into individual statements with statement numbers

2. **Extract table references from each statement**
   - For each statement, identify:
     - Tables READ: `FROM`, `JOIN` clauses
     - Tables WRITTEN: `CREATE TABLE`, `CREATE TEMP TABLE`, `INSERT INTO`, etc.
   - Build a map: `statement_id -> (reads: Vec<String>, writes: Vec<String>)`

3. **Build dependency graph**
   - For each statement, find which earlier statements it depends on
   - Statement B depends on A if: B reads a table that A writes
   - Use a topological sort approach

4. **Compute minimal execution set**
   - Given target statement N
   - Find all statements that N transitively depends on
   - Return ordered list of statement IDs to execute

**Deliverable**: Function signature:
```rust
fn compute_execution_plan(
    statements: &[ParsedStatement],
    target_statement_id: usize
) -> Result<Vec<usize>, Error>
```

### Phase 2: Execution Engine (LATER)
Once we have the execution plan:

1. Execute statements in order from the plan
2. Collect results from the target statement
3. Return results to user
4. Handle errors gracefully (e.g., if statement 1 fails, stop execution)

### Phase 3: Integration (LATER)
1. Add `--execute-statement` flag to CLI
2. Wire up with nvim plugin's `\sx` command
3. Testing with real-world scripts

## Technical Considerations

### Temp Table Detection
- Patterns to detect:
  - `CREATE TEMP TABLE name AS ...`
  - `CREATE TEMPORARY TABLE name AS ...`
  - CTEs are statement-local (don't create dependencies across statements)

### Edge Cases
- Circular dependencies (should error)
- Statement references non-existent table (should error)
- Multiple statements write to same table (use latest)
- Temp tables vs permanent tables (scope considerations)

### Parser Enhancements Needed
- May need to enhance recursive_parser to extract:
  - Table names from CREATE statements
  - Table names from FROM/JOIN clauses
  - Distinguish between temp and permanent tables

## Why Main.rs Refactoring Was Needed

Before implementing `--execute-statement`, we needed to:
1. Clean up main.rs to make adding new flags easier
2. Create proper handler structure following established patterns
3. Ensure argument parsing is clean and maintainable

**Status**: ✅ Main.rs refactored (1967 → 984 lines, 49% reduction)

## Next Steps (Tomorrow)

1. **Start with Phase 1**: Implement dependency analysis
   - Create `src/analysis/statement_dependencies.rs`
   - Write tests with example scripts
   - Build the dependency graph algorithm

2. **Test thoroughly**:
   - Simple case: Linear dependencies (1→2→3)
   - Complex case: Multiple branches (1→3, 2→3, 4 independent)
   - Edge case: Circular dependencies (should error)

3. **Once dependency analysis works**, move to Phase 2: execution

## Open Questions
- Should we cache temp tables between `\sx` calls in the same nvim session?
- How to handle statements that modify data (INSERT/UPDATE/DELETE)?
- Should we show which statements are being executed? (verbose mode?)

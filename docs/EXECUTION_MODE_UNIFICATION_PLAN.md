# Execution Mode Unification Plan

## Executive Summary

**Problem:** SQL CLI has two distinct execution modes with significant code duplication and feature divergence:
- **Script mode** (`-f` flag): Multiple statements with GO separators
- **Single query mode** (`-q` flag): Single statement execution

These modes have evolved separately, causing:
- Feature inconsistencies (temp tables, templates, expansions work differently)
- Code duplication (data loading, preprocessing, execution logic)
- Maintenance burden (changes must be made in multiple places)
- Subtle bugs (double-preprocessing, re-parsing issues)

**Goal:** Unify these execution paths to share as much code as possible while preserving distinct semantics where necessary.

---

## Current State Analysis

### Script Mode (`execute_script()` in src/non_interactive.rs:706)

**Flow:**
1. Parse script into multiple SQL statements (ScriptParser)
2. Load data file (or use DUAL)
3. Create TempTableRegistry
4. **For each statement:**
   a. Expand templates (TemplateExpander)
   b. Parse SQL to AST (Parser)
   c. Determine source table (handle temp tables)
   d. Apply preprocessing pipeline if FROM clause exists
   e. Execute via `QueryEngine.execute_statement_with_temp_tables()` (AST execution)
   f. Handle INTO clause (store in temp table)
   g. Format and output results

**Features:**
- ✅ Temp tables (#table_name)
- ✅ Template expansion ({{table.column}})
- ✅ Preprocessing pipeline (alias expansion, etc.)
- ✅ GO separators
- ✅ EXIT statements
- ✅ [SKIP] directives
- ✅ Data file hints (-- #!)
- ✅ INTO clause

**Execution:** Direct AST execution via QueryEngine

### Single Query Mode (`execute_non_interactive()` in src/non_interactive.rs:227)

**Flow:**
1. Check for temp table usage (error if found)
2. Load data file (or use DUAL)
3. Parse SQL to AST (Parser)
4. Apply preprocessing pipeline if FROM clause exists
5. Execute via `QueryExecutionService.execute()` (re-parses SQL string!)
6. Format and output results

**Features:**
- ❌ NO temp tables (explicitly blocked)
- ❌ NO template expansion
- ✅ Preprocessing pipeline (alias expansion, etc.)
- ❌ NO GO separators (single statement only)
- ❌ NO INTO clause support
- ✅ Execution plan visualization (--execution-plan)
- ✅ Query plan AST display (--query-plan)
- ✅ Work units analysis (--show-work-units)

**Execution:** String execution via QueryExecutionService (re-parses)

### Key Differences

| Feature | Script Mode | Single Query Mode |
|---------|-------------|-------------------|
| Temp tables | ✅ Full support | ❌ Blocked |
| Template expansion | ✅ Yes | ❌ No |
| Preprocessing | ✅ Applied once | ✅ Applied (but may re-parse) |
| Execution path | Direct AST via QueryEngine | String via QueryExecutionService |
| Multi-statement | ✅ Via GO | ❌ Single only |
| INTO clause | ✅ Supported | ❌ No |
| Data file hints | ✅ Yes | ❌ No |

---

## Code Duplication Analysis

### Duplicated Logic

1. **Data Loading** (lines ~720-770 in script, ~233-246 in single)
   - File existence checking
   - CSV/JSON detection
   - DUAL table creation
   - Identical logic, duplicated

2. **Preprocessing Pipeline** (lines ~943-993 in script, ~427-474 in single)
   - Check for FROM clause
   - Apply transformer config
   - Create pipeline with config
   - Process AST
   - Nearly identical, slight variations

3. **Output Formatting** (scattered throughout)
   - CSV/TSV/JSON/Table formatting
   - Column width calculation
   - Row iteration
   - Shared functions, but invoked differently

4. **Parser Invocation** (multiple places)
   - Error handling patterns
   - AST validation
   - Similar but not shared

### The Re-parsing Problem

**Critical Issue:** Script mode applies preprocessing to AST, then executes AST directly. Single query mode applies preprocessing to AST, but then:

```rust
// Single query mode (line ~487)
let executable_sql = ast_formatter::format_select_statement(&stmt);
let query_service = QueryExecutionService::with_behavior_config(behavior_config);
query_service.execute(&executable_sql, ...) // RE-PARSES the formatted SQL!
```

This means:
- Preprocessing transformations are formatted back to SQL
- SQL is re-parsed (second parse!)
- If preprocessing is enabled in QueryExecutionService, it may apply AGAIN
- Performance overhead
- Potential for bugs (double transformation)

---

## Root Causes

### 1. Historical Evolution
Script mode was added later to support temp tables and multi-statement execution. Features were bolted on without refactoring the core execution path.

### 2. Different Entry Points
Two separate functions (`execute_script()` vs `execute_non_interactive()`) with no shared foundation.

### 3. Service Layer Confusion
- `QueryExecutionService`: High-level service that re-parses SQL
- `QueryEngine`: Low-level engine that executes AST directly
- Script mode uses QueryEngine directly
- Single mode uses QueryExecutionService (which wraps QueryEngine)

### 4. Feature Creep
New features (temp tables, templates, preprocessing) added to one mode but not the other.

---

## Proposed Solution: Unified Execution Core

### Architecture Principle

**Single Responsibility:** Separate concerns into distinct layers:
1. **Parsing Layer:** SQL string → AST (once!)
2. **Preprocessing Layer:** AST transformations
3. **Execution Layer:** AST → DataView
4. **Output Layer:** DataView → formatted output

### New Structure

```
┌─────────────────────────────────────────────────────────────┐
│                  Execution Coordinator                       │
│  (Orchestrates all execution regardless of mode)            │
└──────────────────────┬──────────────────────────────────────┘
                       │
       ┌───────────────┴────────────────┐
       ▼                                ▼
┌──────────────┐              ┌──────────────────┐
│ Script Mode  │              │ Single Mode      │
│ - Parse GO   │              │ - Parse single   │
│ - Directives │              │ - Flags          │
└──────┬───────┘              └────────┬─────────┘
       │                               │
       └───────────┬───────────────────┘
                   ▼
       ┌────────────────────────┐
       │   Statement Executor   │
       │ (Shared by both modes) │
       └────────────┬───────────┘
                    │
         ┌──────────┼──────────┐
         ▼          ▼          ▼
    ┌────────┐ ┌────────┐ ┌─────────┐
    │ Parse  │ │Preproc │ │ Execute │
    │ (once!)│ │ (once!)│ │ (AST)   │
    └────────┘ └────────┘ └─────────┘
                    │
                    ▼
            ┌──────────────┐
            │ Format Output│
            └──────────────┘
```

---

## Refactoring Plan

### Phase 0: Preparation (Week 1)

**Goal:** Create foundation without breaking existing code

#### 0.1: Create StatementExecutor Core
**File:** `src/execution/statement_executor.rs` (new module)

```rust
/// Core execution logic shared by all modes
pub struct StatementExecutor {
    case_insensitive: bool,
    auto_hide_empty: bool,
    show_preprocessing: bool,
}

impl StatementExecutor {
    /// Execute a single SQL statement (already parsed)
    pub fn execute_statement(
        &self,
        stmt: SelectStatement,
        source_table: Arc<DataTable>,
        temp_tables: &TempTableRegistry,
        config: &ExecutionConfig,
    ) -> Result<ExecutionResult> {
        // 1. Apply preprocessing pipeline (if needed)
        let transformed = self.apply_preprocessing(stmt, config)?;

        // 2. Execute via QueryEngine (AST execution)
        let engine = QueryEngine::with_case_insensitive(self.case_insensitive);
        let result_view = engine.execute_statement_with_temp_tables(
            source_table,
            transformed,
            Some(temp_tables),
        )?;

        // 3. Return result
        Ok(ExecutionResult {
            dataview: result_view,
            stats: ExecutionStats::new(),
        })
    }

    fn apply_preprocessing(
        &self,
        stmt: SelectStatement,
        config: &ExecutionConfig,
    ) -> Result<SelectStatement> {
        // Apply preprocessing once, controlled by config
        if !self.should_preprocess(&stmt) {
            return Ok(stmt);
        }

        let mut pipeline = create_pipeline_with_config(
            self.show_preprocessing,
            config.transformer_config.clone(),
        );

        pipeline.process(stmt)
    }
}
```

**Benefits:**
- Single place for statement execution
- No re-parsing
- Preprocessing applied once
- Shared by all modes

#### 0.2: Create ExecutionContext
**File:** `src/execution/context.rs` (new)

```rust
/// Context for statement execution (replaces scattered state)
pub struct ExecutionContext {
    pub source_tables: Arc<DataTable>,
    pub temp_tables: TempTableRegistry,
    pub variables: HashMap<String, String>,
    pub config: ExecutionConfig,
}

impl ExecutionContext {
    pub fn new(source_table: Arc<DataTable>) -> Self {
        Self {
            source_tables: source_table,
            temp_tables: TempTableRegistry::new(),
            variables: HashMap::new(),
            config: ExecutionConfig::default(),
        }
    }

    pub fn resolve_table(&self, name: &str) -> Arc<DataTable> {
        if name.starts_with('#') {
            // Temp table
            self.temp_tables.get(name).unwrap_or_else(|| {
                panic!("Temp table not found: {}", name)
            })
        } else {
            // Base table
            self.source_tables.clone()
        }
    }
}
```

#### 0.3: Add Comprehensive Tests
**File:** `tests/execution_unification_tests.rs`

```rust
#[test]
fn test_single_statement_execution() {
    // Test that statement executor works for simple case
}

#[test]
fn test_preprocessing_applied_once() {
    // Verify preprocessing runs exactly once
}

#[test]
fn test_temp_table_execution() {
    // Test temp table resolution
}
```

**Deliverable:** New modules with tests, existing code unchanged

---

### Phase 1: Refactor Script Mode (Week 2)

**Goal:** Convert script mode to use StatementExecutor

#### 1.1: Update execute_script()

```rust
pub fn execute_script(config: NonInteractiveConfig) -> Result<()> {
    // Existing: Parse script, load data
    // ...

    // NEW: Create execution context
    let mut context = ExecutionContext::new(arc_data_table);

    // NEW: Create statement executor
    let executor = StatementExecutor::new(
        config.case_insensitive,
        config.auto_hide_empty,
        config.show_preprocessing,
    );

    // For each statement
    for (idx, script_stmt) in script_statements.iter().enumerate() {
        // Existing: Handle EXIT, SKIP
        // ...

        // NEW: Expand templates (now part of context)
        let expanded_sql = context.expand_templates(statement)?;

        // NEW: Parse once
        let stmt = Parser::new(&expanded_sql).parse()?;

        // NEW: Determine source table
        let source_table = context.resolve_table(
            stmt.from_table.as_deref().unwrap_or("base")
        );

        // NEW: Execute via shared executor
        let result = executor.execute_statement(
            stmt.clone(),
            source_table,
            &context.temp_tables,
            &make_execution_config(&config),
        )?;

        // Existing: Handle INTO clause
        if let Some(into_table) = &stmt.into_table {
            context.temp_tables.insert(
                into_table.name.clone(),
                result.dataview.source_arc(),
            )?;
        }

        // Existing: Format output
        // ...
    }
}
```

**Changes:**
- Use StatementExecutor for execution
- Centralize context management
- No more direct QueryEngine calls
- Template expansion part of context

#### 1.2: Test Script Mode Changes

```bash
# Run all script tests
./run_python_tests.sh
uv run python tests/integration/test_examples.py

# Verify temp tables still work
./target/release/sql-cli -f tests/sql_examples/temp_tables_test.sql

# Verify templates still work
./target/release/sql-cli -f examples/with_templates.sql
```

**Deliverable:** Script mode refactored, all tests passing

---

### Phase 2: Refactor Single Query Mode (Week 3)

**Goal:** Convert single query mode to use StatementExecutor

#### 2.1: Update execute_non_interactive()

```rust
pub fn execute_non_interactive(config: NonInteractiveConfig) -> Result<()> {
    // Existing: Validation
    // NOTE: Remove temp table blocking! Allow temp tables everywhere!
    // check_temp_table_usage(&config.query)?; // DELETE THIS

    // Existing: Load data
    // ...

    // NEW: Create execution context (just like script mode!)
    let mut context = ExecutionContext::new(arc_data_table);

    // NEW: Create statement executor (same as script mode!)
    let executor = StatementExecutor::new(
        config.case_insensitive,
        config.auto_hide_empty,
        config.show_preprocessing,
    );

    // Existing: Handle special flags (--query-plan, --show-work-units, etc.)
    // ...

    // NEW: Parse once
    let stmt = Parser::new(&config.query).parse()?;

    // NEW: Determine source table (now supports temp tables!)
    let source_table = context.resolve_table(
        stmt.from_table.as_deref().unwrap_or("base")
    );

    // NEW: Execute via shared executor (SAME CODE AS SCRIPT MODE!)
    let result = executor.execute_statement(
        stmt,
        source_table,
        &context.temp_tables,
        &make_execution_config(&config),
    )?;

    // Existing: Apply limit if requested
    // ...

    // Existing: Format output
    // ...
}
```

**Key Changes:**
- Remove temp table blocking (feature parity!)
- Use StatementExecutor (same as script mode)
- Remove QueryExecutionService call (no more re-parsing!)
- Support temp tables in single query mode

#### 2.2: Deprecate QueryExecutionService String Path

**File:** `src/services/query_execution_service.rs`

```rust
impl QueryExecutionService {
    /// Execute a query from SQL string
    /// DEPRECATED: Use StatementExecutor with pre-parsed AST instead
    #[deprecated(note = "Use StatementExecutor for direct AST execution")]
    pub fn execute(&self, sql: &str, ...) -> Result<QueryExecutionResult> {
        // Keep for TUI compatibility (for now)
        // Eventually TUI should also use StatementExecutor
    }
}
```

#### 2.3: Test Single Query Mode Changes

```bash
# Test basic queries still work
./target/release/sql-cli -q "SELECT 1+1"
./target/release/sql-cli data/test.csv -q "SELECT * FROM test"

# Test preprocessing still works
./target/release/sql-cli data/test.csv -q "SELECT a FROM test WHERE COUNT(*) > 0" --show-preprocessing

# Test that we don't re-parse (check logs)
RUST_LOG=debug ./target/release/sql-cli -q "SELECT 1+1" 2>&1 | grep -i parse
```

**Deliverable:** Single query mode refactored, tests passing, no re-parsing

---

### Phase 3: Add Missing Features (Week 4)

**Goal:** Achieve feature parity between modes

#### 3.1: Template Expansion in Single Query Mode

Since single query mode now uses ExecutionContext, templates automatically work:

```bash
# This should now work (currently doesn't):
./target/release/sql-cli data/test.csv -q "SELECT {{test.column_name}}"
```

#### 3.2: Temp Tables in Single Query Mode

Remove the blocking check:

```rust
// DELETE THIS FUNCTION:
// fn check_temp_table_usage(query: &str) -> Result<()> { ... }

// Now this works:
./target/release/sql-cli -q "SELECT * FROM #my_temp_table"
```

**Note:** User needs to populate temp table first (in script mode), but at least it doesn't error!

#### 3.3: Data File Hints in Single Query Mode

```rust
// In execute_non_interactive(), before loading data:
let parser = ScriptParser::new(&config.query);
if let Some(hint) = parser.data_file_hint() {
    // Use hint if no data file specified
}
```

Now this works:
```sql
-- #! ../data/test.csv
SELECT * FROM test
```

#### 3.4: Test Feature Parity

```bash
# Test templates in single query
echo "SELECT {{test.a}}" | ./target/release/sql-cli data/test.csv -q -

# Test data file hints
./target/release/sql-cli -q "-- #! data/test.csv
SELECT * FROM test"

# Test unified behavior
./run_all_tests.sh
```

**Deliverable:** Feature parity achieved

---

### Phase 4: TUI Integration (Week 5)

**Goal:** Update TUI to use StatementExecutor

#### 4.1: Update EnhancedTuiApp

```rust
impl EnhancedTuiApp {
    pub fn execute_query_v2(&mut self, query: &str) -> Result<()> {
        // Parse once
        let stmt = Parser::new(query).parse()?;

        // Use StatementExecutor
        let result = self.statement_executor.execute_statement(
            stmt,
            self.context.resolve_table("base"),
            &self.context.temp_tables,
            &self.execution_config,
        )?;

        // Update UI with result
        self.update_dataview(result.dataview);
        Ok(())
    }
}
```

**Benefit:** TUI gets temp table support, templates, and unified behavior!

---

### Phase 5: Cleanup & Optimization (Week 6)

**Goal:** Remove duplication, optimize performance

#### 5.1: Remove Duplicated Code

- **Delete:** Duplicated data loading logic
- **Delete:** Duplicated preprocessing setup
- **Delete:** Re-parsing code paths
- **Centralize:** Output formatting logic

#### 5.2: Performance Audit

```bash
# Measure parsing overhead
hyperfine './target/release/sql-cli -q "SELECT * FROM test" data/test.csv'

# Before refactor: ~X ms (includes re-parsing)
# After refactor: ~Y ms (single parse)
```

#### 5.3: Documentation Update

Update CLAUDE.md:
```markdown
## Execution Modes

SQL CLI has two execution modes, both sharing the same core execution engine:

1. **Single Query Mode** (`-q` flag): Execute one SQL statement
2. **Script Mode** (`-f` flag): Execute multiple statements with GO separators

Both modes support:
- Temp tables
- Template expansion
- Preprocessing pipeline
- All SQL features

The only difference is multi-statement support.
```

---

## Implementation Order

### Week 1: Foundation
- [ ] Create StatementExecutor
- [ ] Create ExecutionContext
- [ ] Add tests for new modules
- [ ] No changes to existing code

### Week 2: Script Mode
- [ ] Refactor execute_script() to use StatementExecutor
- [ ] Test all script features
- [ ] Verify examples still work

### Week 3: Single Query Mode
- [ ] Refactor execute_non_interactive() to use StatementExecutor
- [ ] Remove re-parsing
- [ ] Test all single query features

### Week 4: Feature Parity
- [ ] Enable templates in single mode
- [ ] Enable temp tables in single mode
- [ ] Add data file hints to single mode
- [ ] Comprehensive testing

### Week 5: TUI Integration
- [ ] Update TUI to use StatementExecutor
- [ ] Test interactive mode
- [ ] Verify all keybindings work

### Week 6: Cleanup
- [ ] Remove deprecated code
- [ ] Optimize performance
- [ ] Update documentation
- [ ] Final regression testing

---

## Success Metrics

### Code Quality
- [ ] Lines of code reduced by ~30%
- [ ] Duplication eliminated in data loading
- [ ] Duplication eliminated in preprocessing
- [ ] Single parse per statement (no re-parsing)

### Feature Parity
- [ ] Temp tables work in all modes
- [ ] Templates work in all modes
- [ ] Preprocessing works identically
- [ ] Output formatting consistent

### Performance
- [ ] Single query mode faster (no re-parse)
- [ ] Script mode same or faster
- [ ] TUI responsiveness unchanged

### Testing
- [ ] All existing tests pass
- [ ] New tests for unified execution
- [ ] Example scripts still work
- [ ] Python tests pass

---

## Risk Mitigation

### Risk 1: Breaking Changes
**Impact:** High
**Probability:** Medium
**Mitigation:**
- Comprehensive test suite before starting
- Refactor incrementally (one mode at a time)
- Keep old code paths until new ones proven
- Extensive manual testing

### Risk 2: Performance Regression
**Impact:** Medium
**Probability:** Low
**Mitigation:**
- Benchmark before refactor
- Benchmark after each phase
- Profile hot paths
- Optimize if needed

### Risk 3: Feature Divergence During Refactor
**Impact:** Medium
**Probability:** Medium
**Mitigation:**
- Feature freeze during refactor
- Document all edge cases
- Test matrix for all feature combinations

---

## Alternatives Considered

### Alternative 1: Keep Separate Modes
**Pros:** No refactor risk
**Cons:** Technical debt grows, maintenance burden, bugs

**Rejected:** Unsustainable long-term

### Alternative 2: Rewrite Everything
**Pros:** Clean slate
**Cons:** High risk, long timeline, breaks everything

**Rejected:** Too risky

### Alternative 3: Incremental Unification (Chosen)
**Pros:** Low risk, testable, incremental value
**Cons:** Takes longer (6 weeks)

**Chosen:** Best balance of risk and benefit

---

## Appendix: File Map

### New Files
```
src/execution/
├── mod.rs                  # New module
├── statement_executor.rs   # Core executor
├── context.rs             # Execution context
└── config.rs              # Execution configuration

tests/
└── execution_unification_tests.rs  # Test suite
```

### Modified Files
```
src/non_interactive.rs     # Both execute_* functions
src/services/query_execution_service.rs  # Deprecation
src/ui/enhanced_tui.rs     # TUI integration
docs/CLAUDE.md             # Documentation update
```

### Files to Delete Eventually
```
# After Phase 5:
- Duplicated data loading logic in non_interactive.rs
- Re-parsing code paths in QueryExecutionService
```

---

## Next Steps

1. **Review this plan** with team/maintainers
2. **Get approval** for the approach
3. **Create GitHub issue** tracking each phase
4. **Start Phase 0** (foundation)
5. **Weekly check-ins** to assess progress

---

## Questions for Discussion

1. Should we support temp tables in single query mode? (Proposed: Yes)
2. Should we deprecate QueryExecutionService entirely? (Proposed: Eventually)
3. Timeline acceptable? (Proposed: 6 weeks)
4. Any features we're missing in this analysis?
5. Should we maintain backward compatibility for old behavior? (Proposed: Yes, via flags)

---

## Conclusion

This refactoring will:
- **Eliminate code duplication** (~30% reduction)
- **Fix subtle bugs** (re-parsing, double preprocessing)
- **Enable feature parity** (temp tables, templates everywhere)
- **Improve performance** (single parse)
- **Simplify maintenance** (one execution path)
- **Enable future features** (easier to add once)

The incremental approach minimizes risk while delivering value at each phase.

**Estimated effort:** 6 weeks, one developer
**Risk level:** Medium (mitigated by incremental approach)
**Value:** High (technical debt reduction + feature parity)

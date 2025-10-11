# Code CTE Design - Programmable Data Transformations

> **⚠️ DECISION: NOT PROCEEDING WITH THIS FEATURE (2025-01-11)**
>
> After investigation, we decided NOT to implement CODE CTEs due to Python environment management complexity (venvs, versions, dependencies). The same capability can be achieved more simply by:
>
> 1. **Enhancing WEB CTEs** to support POST with query result as body
> 2. **User runs their own Flask/Node/Go server** for transformations
> 3. User maintains full control of their Python environment
>
> This document is preserved for historical reference only.
>
> See `docs/SESSION_SUMMARY_2025-01-11.md` for full analysis.

---

## Executive Summary

This document outlines a strategic design for adding programmable CTEs to SQL-CLI, enabling users to write code-based data transformations that integrate seamlessly with the SQL execution pipeline. This is a long-term, multi-phase project that will be developed iteratively.

## Vision

Enable users to write testable, reusable code blocks that act as CTEs in the SQL pipeline:

```sql
-- External code file: ./scripts/enrich_trades.py
-- Testable independently with pytest

WITH base_trades AS (
    SELECT * FROM trades WHERE date = '2024-01-15'
),
CODE enriched AS (
    LANGUAGE python
    SOURCE './scripts/enrich_trades.py'
    FUNCTION enrich_trades
    INPUT base_trades
)
SELECT * FROM enriched WHERE risk_score > 50;
```

The code receives DataTable(s) as input, performs transformations, and returns a new DataTable that flows back into the SQL pipeline.

## Current State Analysis

### What We Have

1. **Temp Tables (#tmp)** ✅
   - Persist data across GO-delimited queries
   - TempTableRegistry for script-scoped storage
   - Already working well

2. **Template System** ✅
   - `@{VAR:name}`, `@{INPUT:prompt}`, `@{MACRO:name}` syntax
   - External template files with SQL syntax highlighting
   - Template expansion before query execution

3. **WEB CTEs** ✅
   - Fetch data from HTTP APIs
   - JSON/CSV format support
   - Template injection for dynamic queries

4. **Python Test Infrastructure** ✅
   - 40+ Python test files
   - Python already installed and working
   - subprocess integration proven

5. **Lua in Nvim Plugin** ✅
   - Client-side Lua for editor integration
   - Not embedded in CLI (yet)

### Architectural Foundation

```
Current Pipeline:
SQL Text → Parser → AST → Query Executor → DataTable → Results

Proposed Pipeline:
SQL Text → Parser → AST → Query Executor → [Code CTE Executor] → DataTable → Results
                              ↓
                    CTE Context (read-only)
                    Temp Tables (read-only)
```

## Technology Options Analysis

### Option 1: Python (via subprocess) ⭐ **RECOMMENDED FOR PHASE 1**

**Pros:**
- ✅ Already used extensively in test suite
- ✅ Rich ecosystem (pandas, numpy, polars)
- ✅ Most developers know Python
- ✅ Easy debugging - run scripts independently
- ✅ No new Rust dependencies
- ✅ Proven subprocess integration

**Cons:**
- ⚠️ Slower startup (subprocess overhead)
- ⚠️ Data serialization cost (JSON/CSV between processes)
- ⚠️ Not embedded (external Python required)

**Implementation Complexity:** Low
**Risk:** Low
**Performance:** Medium (acceptable for data transformation)

**Architecture:**
```rust
// Rust side
pub struct PythonCodeCTE {
    script_path: PathBuf,
    function_name: String,
}

impl PythonCodeCTE {
    fn execute(&self, input: &DataTable) -> Result<DataTable> {
        // 1. Serialize DataTable to JSON
        let input_json = serde_json::to_string(&input)?;

        // 2. Create temp file with input data
        let temp_input = NamedTempFile::new()?;
        write!(temp_input, "{}", input_json)?;

        // 3. Execute Python script
        let output = Command::new("python3")
            .arg(&self.script_path)
            .arg("--function")
            .arg(&self.function_name)
            .arg("--input")
            .arg(temp_input.path())
            .output()?;

        // 4. Parse output JSON back to DataTable
        let output_json = String::from_utf8(output.stdout)?;
        let result_table = DataTable::from_json(&output_json)?;

        Ok(result_table)
    }
}
```

```python
# Python side (user-written script)
import json
import sys
from typing import Dict, List, Any

def enrich_trades(input_table: Dict[str, Any]) -> Dict[str, Any]:
    """
    Process trade data and add risk scores.

    Args:
        input_table: {
            "columns": ["trade_id", "symbol", "amount"],
            "rows": [[1, "AAPL", 1000], [2, "GOOGL", 2000]]
        }

    Returns:
        Same structure with additional columns
    """
    # Add risk_score column
    rows = input_table["rows"]
    enriched_rows = []

    for row in rows:
        amount = row[2]
        risk_score = calculate_risk(amount)  # User logic
        enriched_rows.append(row + [risk_score])

    return {
        "columns": input_table["columns"] + ["risk_score"],
        "rows": enriched_rows
    }

# CLI framework code (provided by sql-cli)
if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument("--function", required=True)
    parser.add_argument("--input", required=True)
    args = parser.parse_args()

    # Load input
    with open(args.input) as f:
        input_data = json.load(f)

    # Execute user function
    func = globals()[args.function]
    result = func(input_data)

    # Output result
    print(json.dumps(result))
```

### Option 2: Embedded Lua (mlua crate)

**Pros:**
- ✅ Lightweight and fast
- ✅ Small runtime overhead
- ✅ Good Rust integration (mlua crate)
- ✅ Lua already familiar to Nvim users

**Cons:**
- ⚠️ Limited ecosystem compared to Python
- ⚠️ New dependency (mlua ~300KB)
- ⚠️ Separate language to learn
- ⚠️ Harder debugging than Python

**Implementation Complexity:** Medium
**Risk:** Medium
**Performance:** High

### Option 3: JavaScript (Rhai or boa)

**Pros:**
- ✅ Rhai designed for Rust embedding
- ✅ Familiar syntax for many developers
- ✅ Good performance

**Cons:**
- ⚠️ Another language to support
- ⚠️ Smaller ecosystem than Python/Lua
- ⚠️ Less proven in data processing

**Implementation Complexity:** Medium
**Risk:** Medium
**Performance:** High

### Option 4: WebAssembly (wasmtime)

**Pros:**
- ✅ Language agnostic (Python, Rust, C++, Go)
- ✅ Sandboxed execution
- ✅ Great performance potential

**Cons:**
- ❌ Complex compilation workflow
- ❌ Steep learning curve
- ❌ Hard to debug
- ❌ Overkill for current use case

**Implementation Complexity:** Very High
**Risk:** High
**Performance:** Very High

## Recommended Approach: Phased Implementation

### Phase 1: Python Subprocess (6-8 sessions)

**Goal:** Prove the concept with minimal risk

**Deliverables:**
1. JSON DataTable serialization format
2. Python helper library (`sql_cli_helper.py`)
3. CODE CTE parser syntax
4. Python subprocess executor
5. Error handling and validation
6. Documentation and examples
7. Test suite

**SQL Syntax:**
```sql
CODE result AS (
    LANGUAGE python
    SOURCE './transform.py'
    FUNCTION transform_data
    INPUT source_cte
)
```

**Success Criteria:**
- Can execute Python code from SQL
- Input/output DataTable serialization works
- Error messages are clear
- Performance acceptable for <100K rows

### Phase 2: Multiple Inputs & Optimization (3-4 sessions)

**Enhancements:**
- Multiple input CTEs
- Caching of Python interpreter
- Streaming for large datasets
- CSV serialization option (faster than JSON)

```sql
CODE joined AS (
    LANGUAGE python
    SOURCE './join_logic.py'
    FUNCTION custom_join
    INPUT trades, allocations  -- Multiple inputs
)
```

### Phase 3: Embedded Lua (Optional, 4-6 sessions)

**If Python proves successful and users want faster embedded option:**
- Add mlua dependency
- Implement Lua executor
- Provide Lua API for DataTable access
- Performance comparison

```sql
CODE filtered AS (
    LANGUAGE lua
    INLINE "
        return input:filter(function(row)
            return row.amount > 1000
        end)
    "
    INPUT trades
)
```

### Phase 4: Advanced Features (Future)

- Async/concurrent execution
- Streaming transformations
- Type system integration
- WASM support for compiled languages

## API Design

### DataTable JSON Format

```json
{
    "name": "trades",
    "columns": [
        {
            "name": "trade_id",
            "data_type": "Integer",
            "nullable": false
        },
        {
            "name": "symbol",
            "data_type": "String",
            "nullable": false
        },
        {
            "name": "amount",
            "data_type": "Float",
            "nullable": true
        }
    ],
    "rows": [
        [1, "AAPL", 1000.50],
        [2, "GOOGL", 2000.75],
        [3, "MSFT", null]
    ]
}
```

### Python Helper Library

```python
# sql_cli_helper.py (provided with installation)
from typing import List, Dict, Any, Optional
from dataclasses import dataclass

@dataclass
class Column:
    name: str
    data_type: str
    nullable: bool

class DataTable:
    """Wrapper around sql-cli DataTable JSON format"""

    def __init__(self, data: Dict[str, Any]):
        self.name = data["name"]
        self.columns = [Column(**c) for c in data["columns"]]
        self.rows = data["rows"]

    def to_dict(self) -> Dict[str, Any]:
        return {
            "name": self.name,
            "columns": [{"name": c.name, "data_type": c.data_type, "nullable": c.nullable}
                       for c in self.columns],
            "rows": self.rows
        }

    def filter(self, predicate):
        """Filter rows based on predicate"""
        filtered = [row for row in self.rows if predicate(row)]
        return DataTable({
            "name": self.name,
            "columns": [c.__dict__ for c in self.columns],
            "rows": filtered
        })

    def add_column(self, name: str, data_type: str, values: List[Any]):
        """Add a new column"""
        if len(values) != len(self.rows):
            raise ValueError(f"Expected {len(self.rows)} values, got {len(values)}")

        self.columns.append(Column(name, data_type, nullable=True))
        for i, row in enumerate(self.rows):
            row.append(values[i])

    def to_pandas(self):
        """Convert to pandas DataFrame (if pandas installed)"""
        import pandas as pd
        return pd.DataFrame(self.rows, columns=[c.name for c in self.columns])

    @staticmethod
    def from_pandas(df, name="result"):
        """Create from pandas DataFrame"""
        # Type inference logic
        pass
```

### User Script Template

```python
#!/usr/bin/env python3
"""
Trade enrichment script for sql-cli CODE CTE

This script can be tested independently:
    python enrich_trades.py --test
    pytest test_enrich_trades.py
"""
from sql_cli_helper import DataTable

def enrich_trades(trades: DataTable) -> DataTable:
    """
    Add risk scores and categories to trades

    Args:
        trades: DataTable with columns [trade_id, symbol, amount]

    Returns:
        DataTable with additional columns [risk_score, category]
    """
    risk_scores = []
    categories = []

    for row in trades.rows:
        trade_id, symbol, amount = row

        # Calculate risk score
        risk = calculate_risk(amount, symbol)
        risk_scores.append(risk)

        # Categorize
        category = categorize_trade(amount)
        categories.append(category)

    # Add new columns
    trades.add_column("risk_score", "Float", risk_scores)
    trades.add_column("category", "String", categories)

    return trades

def calculate_risk(amount: float, symbol: str) -> float:
    """Business logic for risk calculation"""
    base_risk = amount / 10000
    if symbol in ["TSLA", "GME"]:
        return base_risk * 2.0
    return base_risk

def categorize_trade(amount: float) -> str:
    """Categorize trade size"""
    if amount < 1000:
        return "small"
    elif amount < 10000:
        return "medium"
    else:
        return "large"

# Testing interface
if __name__ == "__main__":
    import sys
    import json

    if "--test" in sys.argv:
        # Run with test data
        test_data = DataTable({
            "name": "test_trades",
            "columns": [
                {"name": "trade_id", "data_type": "Integer", "nullable": False},
                {"name": "symbol", "data_type": "String", "nullable": False},
                {"name": "amount", "data_type": "Float", "nullable": False}
            ],
            "rows": [
                [1, "AAPL", 5000.0],
                [2, "TSLA", 15000.0],
                [3, "GOOGL", 500.0]
            ]
        })

        result = enrich_trades(test_data)
        print(json.dumps(result.to_dict(), indent=2))

    else:
        # SQL-CLI integration mode
        from sql_cli_helper import run_code_cte
        run_code_cte(enrich_trades)
```

## File Organization

```
sql-cli/
├── src/
│   ├── sql/
│   │   └── code_cte/
│   │       ├── mod.rs           # CODE CTE parser
│   │       ├── executor.rs      # Abstract executor trait
│   │       ├── python.rs        # Python subprocess executor
│   │       └── lua.rs           # Lua embedded executor (Phase 3)
│   └── data/
│       └── serialization/
│           ├── json_table.rs    # DataTable <-> JSON
│           └── csv_table.rs     # DataTable <-> CSV (Phase 2)
├── python/
│   ├── sql_cli_helper.py        # Helper library
│   ├── setup.py                 # Pip package
│   └── examples/
│       ├── enrich_trades.py
│       ├── custom_join.py
│       └── test_*.py
├── examples/
│   └── code_cte_examples.sql    # SQL examples
└── docs/
    ├── CODE_CTE_DESIGN.md       # This document
    ├── CODE_CTE_TUTORIAL.md     # User guide
    └── CODE_CTE_API.md          # Python API reference
```

## Security Considerations

1. **Code Execution Risk:**
   - Python scripts run with user's permissions
   - No sandboxing in Phase 1 (subprocess inherits permissions)
   - Users must trust their own code

2. **Path Handling:**
   - Validate SOURCE paths are readable
   - No automatic execution of code
   - Explicit SOURCE required (no implicit includes)

3. **Error Handling:**
   - Timeout for long-running scripts
   - Capture stderr for debugging
   - Clear error messages for invalid Python

4. **Future (Phase 3+):**
   - Consider Lua sandboxing
   - Resource limits (memory, CPU)
   - WASM for true sandboxing

## Testing Strategy

### Unit Tests (Rust)
- Parser for CODE CTE syntax
- JSON serialization/deserialization
- Error handling

### Integration Tests (Python)
- End-to-end CODE CTE execution
- Multiple input CTEs
- Error propagation
- Performance benchmarks

### Example Tests
```python
def test_code_cte_basic():
    """Test basic CODE CTE execution"""
    result = run_sql("""
        WITH data AS (SELECT value as n FROM RANGE(1, 5))
        CODE doubled AS (
            LANGUAGE python
            SOURCE './tests/scripts/double_values.py'
            FUNCTION double_values
            INPUT data
        )
        SELECT * FROM doubled;
    """)

    assert len(result) == 5
    assert result[0]["n"] == 2
    assert result[4]["n"] == 10
```

## Performance Expectations

### Phase 1 (Python Subprocess)

| Rows    | Serialization | Python Exec | Total      | Acceptable? |
|---------|---------------|-------------|------------|-------------|
| 100     | 1ms           | 50ms        | ~51ms      | ✅ Yes      |
| 1,000   | 5ms           | 55ms        | ~60ms      | ✅ Yes      |
| 10,000  | 50ms          | 100ms       | ~150ms     | ✅ Yes      |
| 100,000 | 500ms         | 500ms       | ~1s        | ⚠️ Acceptable |
| 1M+     | 5s+           | Variable    | 10s+       | ❌ Use streaming |

**Optimization paths:**
- CSV serialization (faster than JSON)
- Persistent Python process (avoid startup overhead)
- Streaming mode (Phase 2+)

## Migration Path for Existing Users

This is a **purely additive feature**:

1. No breaking changes to existing SQL syntax
2. CODE CTE is opt-in
3. Works alongside all existing features
4. Python scripts are external - no forced dependencies
5. Clear error if Python not available

## Success Metrics

### Phase 1 Success:
- [ ] Can execute Python function from SQL
- [ ] INPUT CTE data passed correctly
- [ ] OUTPUT DataTable integrates with SQL
- [ ] 10+ example scripts provided
- [ ] Documentation complete
- [ ] 5 users test and provide feedback
- [ ] Performance acceptable for 10K rows

### Long-term Vision:
- Stored procedure-like workflow
- Library of reusable transformations
- Community-contributed scripts
- Tight editor integration (LSP, debugging)

## Open Questions for Discussion

1. **Naming:** `CODE` vs `SCRIPT` vs `TRANSFORM` vs `PROC`?
2. **Error handling:** What happens if Python script fails mid-stream?
3. **Type system:** Should we enforce schema compatibility?
4. **Caching:** Cache Python interpreter process between queries?
5. **Async:** Should CODE CTEs run concurrently when independent?

## Next Steps

1. **Review this design** with user feedback
2. **Create Phase 1 task breakdown** (6-8 sessions)
3. **Prototype JSON serialization** (1 session)
4. **Prototype Python executor** (1-2 sessions)
5. **Iterate based on real-world usage**

## Conclusion

Starting with Python via subprocess is the **lowest-risk, highest-value** approach:
- Leverages existing infrastructure
- Familiar to most users
- Easy to test and debug
- Minimal new dependencies
- Can optimize later if needed

This provides a solid foundation for stored procedure-like functionality while maintaining sql-cli's philosophy of simplicity and transparency.

---

**Document Status:** Draft for discussion
**Author:** Claude with user guidance
**Date:** 2025-10-09
**Next Review:** After initial feedback

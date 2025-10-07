# Phase 2: Scripting Enhancements - Design Document

## Overview

This document outlines the roadmap for Phase 2 enhancements to sql-cli's scripting capabilities. The goal is to create a powerful data pipeline that seamlessly integrates SQL queries, temporary tables, web API calls, and potentially embedded scripting languages.

## Current State (Phase 1 Complete ✅)

### Implemented Features
- **Temporary Tables (#tmp)**: Store query results across script statements
- **Script Directives**: EXIT and [SKIP] for flow control
- **WEB CTE**: Query REST APIs and treat responses as tables
- **GO Separators**: Multi-statement script execution
- **Data File Hints**: `-- #! path/to/file.csv`

### Current Workflow Example
```sql
-- Get high-value trades
SELECT * FROM trades WHERE amount > 100000 INTO #high_value;
GO

-- Aggregate by trader
SELECT trader, SUM(amount) FROM #high_value GROUP BY trader;
GO

-- Query external API (static parameters)
WITH market_data AS (
    WEB(
        URL 'https://api.market.com/quotes',
        BODY {"symbols": ["AAPL", "MSFT"]}
    )
)
SELECT * FROM market_data;
GO
```

### The Gap 🔍

**Problem:** Cannot dynamically pass data from temp tables to WEB CTEs

**Current Limitation:**
```sql
-- ❌ This doesn't work yet
SELECT DISTINCT symbol FROM #high_value INTO #symbols;
GO

WITH quotes AS (
    WEB(
        URL 'https://api.market.com/quotes',
        BODY {"symbols": #symbols}  -- Can't inject temp table data!
    )
)
SELECT * FROM quotes;
GO
```

**Impact:** Forces over-fetching or manual workarounds
- Must fetch ALL securities instead of just needed ones
- Cannot chain queries across multiple systems efficiently
- Limits the power of the temp table + WEB CTE combination

---

## Phase 2A: Template Injection in WEB CTEs 🎯

### Priority: **HIGHEST** (Start Tomorrow)

### Goal
Enable dynamic data injection from temporary tables into WEB CTE request bodies and URLs.

### Use Cases

#### Use Case 1: Dynamic Instrument Lists
```sql
-- Get instruments we care about from trade data
SELECT DISTINCT instrument FROM trades
WHERE date = TODAY()
INTO #instruments;
GO

-- Fetch quotes ONLY for those instruments
WITH market_data AS (
    WEB(
        URL 'https://api.secmaster.com/quotes',
        METHOD 'POST',
        BODY {
            "instruments": ${#instruments},  -- Template injection!
            "fields": ["last", "bid", "ask"]
        }
    )
)
SELECT * FROM market_data;
GO
```

#### Use Case 2: Multi-System Data Correlation
```sql
-- Parse FIX logs for unique order IDs
WITH fix_orders AS (
    WEB(
        URL 'https://fix-engine/parse',
        BODY {"tag": 11, "date": "2025-10-07"}
    )
)
SELECT DISTINCT order_id FROM fix_orders INTO #orders;
GO

-- Query Front Arena risk system for those specific orders
WITH risk_data AS (
    WEB(
        URL 'https://front-arena/api/risk',
        BODY {"order_ids": ${#orders}}
    )
)
SELECT * FROM risk_data;
GO

-- Cross-reference with internal trade DB
SELECT
    t.order_id,
    t.trader,
    r.var,
    r.credit_exposure
FROM trades t
JOIN risk_data r ON t.order_id = r.order_id
WHERE t.order_id IN (SELECT * FROM #orders);
GO
```

#### Use Case 3: Dynamic URL Construction
```sql
-- Get list of counterparties
SELECT DISTINCT counterparty_id FROM trades INTO #counterparties;
GO

-- Fetch ratings for each (URL injection)
WITH ratings AS (
    WEB(
        URL 'https://ratings.com/api/${#counterparties}/rating',
        METHOD 'GET'
    )
)
SELECT * FROM ratings;
GO
```

### Syntax Design

**Template Variables:**
- `${#table_name}` - Inject entire table as JSON array
- `${#table_name.column}` - Inject single column as array
- `${#table_name[0].column}` - Inject single value (first row)

**Examples:**
```sql
-- Full table injection (array of objects)
BODY {"data": ${#instruments}}
→ {"data": [{"symbol": "AAPL"}, {"symbol": "MSFT"}]}

-- Column array injection
BODY {"symbols": ${#instruments.symbol}}
→ {"symbols": ["AAPL", "MSFT"]}

-- Single value injection
URL 'https://api.com/trades/${#config[0].region}'
→ 'https://api.com/trades/APAC'
```

### Implementation Plan

#### Step 1: Extend AST
```rust
// src/sql/parser/ast.rs
pub struct WebCTESpec {
    pub url: String,
    pub method: HttpMethod,
    pub body: Option<String>,
    pub template_vars: Vec<TemplateVar>,  // NEW
}

pub struct TemplateVar {
    pub placeholder: String,      // e.g., "${#instruments}"
    pub table_name: String,        // e.g., "#instruments"
    pub column: Option<String>,    // e.g., Some("symbol")
    pub index: Option<usize>,      // e.g., Some(0)
}
```

#### Step 2: Template Parser
```rust
// src/sql/template_expander.rs (NEW FILE)

pub struct TemplateExpander;

impl TemplateExpander {
    /// Find all template variables in a string
    pub fn parse_templates(input: &str) -> Vec<TemplateVar> {
        // Regex: \$\{#(\w+)(?:\.(\w+))?(?:\[(\d+)\])?\}
        // Matches: ${#table}, ${#table.col}, ${#table[0].col}
    }

    /// Expand templates with data from temp tables
    pub fn expand(
        template: &str,
        temp_tables: &TempTableRegistry
    ) -> Result<String> {
        let vars = Self::parse_templates(template);
        let mut result = template.to_string();

        for var in vars {
            let data = Self::extract_data(&var, temp_tables)?;
            let json = serde_json::to_string(&data)?;
            result = result.replace(&var.placeholder, &json);
        }

        Ok(result)
    }

    /// Extract data from temp table based on template spec
    fn extract_data(
        var: &TemplateVar,
        registry: &TempTableRegistry
    ) -> Result<serde_json::Value> {
        let table = registry.get(&var.table_name)?;

        match (&var.column, var.index) {
            // ${#table} - full table as array of objects
            (None, None) => {
                Self::table_to_json_array(table)
            }
            // ${#table.col} - column as array
            (Some(col), None) => {
                Self::column_to_json_array(table, col)
            }
            // ${#table[0].col} - single value
            (Some(col), Some(idx)) => {
                Self::cell_to_json(table, idx, col)
            }
            // ${#table[0]} - single row as object
            (None, Some(idx)) => {
                Self::row_to_json(table, idx)
            }
        }
    }
}
```

#### Step 3: Integration Point
```rust
// src/non_interactive.rs - execute_script()

// When executing a statement with WEB CTEs:
for cte in &parsed_stmt.ctes {
    if let CTEType::Web(web_spec) = &cte.cte_type {
        // Expand templates in URL
        let expanded_url = TemplateExpander::expand(
            &web_spec.url,
            &temp_tables
        )?;

        // Expand templates in BODY
        let expanded_body = if let Some(body) = &web_spec.body {
            Some(TemplateExpander::expand(body, &temp_tables)?)
        } else {
            None
        };

        // Execute WEB request with expanded values
        execute_web_cte(expanded_url, expanded_body, ...)?;
    }
}
```

#### Step 4: Testing
```bash
# Create test script: tests/integration/test_template_injection.sql
-- #! ../data/sales_data.csv

-- Get distinct regions
SELECT DISTINCT region FROM sales_data INTO #regions;
GO

-- Mock API call with injected data
WITH regional_data AS (
    WEB(
        URL 'http://localhost:8080/api/regions',
        METHOD 'POST',
        BODY {"regions": ${#regions.region}}
    )
)
SELECT * FROM regional_data;
GO
```

### Error Handling

**Validation:**
- Template references non-existent table → Error with clear message
- Template references non-existent column → Error with column list
- Empty table injection → Warning, inject empty array `[]`
- Type mismatch in single-value injection → Error

**Examples:**
```
Error: Template variable ${#missing} references temporary table '#missing'
       which does not exist. Available tables: #instruments, #trades

Error: Template variable ${#instruments.invalid_col} references column
       'invalid_col' which does not exist in table '#instruments'.
       Available columns: symbol, exchange, price

Warning: Template variable ${#results} injecting empty array -
         table '#results' has 0 rows
```

### Performance Considerations

- **Large Tables:** Warn if injecting >1000 rows
- **Serialization:** Stream JSON encoding for large datasets
- **Caching:** Cache serialized JSON if same table used multiple times

---

## Phase 2B: Python Integration 🐍

### Priority: **HIGH** (After template injection)

### Goal
Embed Python interpreter for complex transformations, statistical analysis, and ML within SQL scripts.

### Architecture

```
sql-cli (Rust)
    │
    ├─ SQL Parser & Engine
    ├─ Temporary Tables (DataTable)
    │
    └─ Python Bridge (pyo3)
           │
           ├─ PythonContext
           │   ├─ get_table(name) → pandas.DataFrame
           │   ├─ set_table(name, df) → #tmp table
           │   └─ execute(code) → Result
           │
           └─ Python Runtime
               ├─ pandas
               ├─ numpy
               ├─ scipy
               └─ User code
```

### Syntax Design

#### Option 1: PYTHON Statement Block
```sql
SELECT * FROM trades WHERE date = TODAY() INTO #trades;
GO

PYTHON """
import pandas as pd
import numpy as np

# Get data from temp table
df = ctx.table('#trades')

# Complex analysis
df['returns'] = df['pnl'].pct_change()
df['volatility'] = df['returns'].rolling(20).std()
df['zscore'] = (df['pnl'] - df['pnl'].mean()) / df['pnl'].std()

# Filter and return
result = df[df['zscore'].abs() > 2.0]
ctx.return_table(result, '#outliers')
""";
GO

-- Continue with SQL
SELECT trader, COUNT(*) as outlier_count
FROM #outliers
GROUP BY trader;
GO
```

#### Option 2: PYTHON Function (Stored Procedures)
```sql
-- Define reusable Python function
CREATE PYTHON FUNCTION calc_sharpe(
    returns_table TEXT
) RETURNS TABLE AS """
import pandas as pd
import numpy as np

df = ctx.table(returns_table)
sharpe = df['returns'].mean() / df['returns'].std() * np.sqrt(252)

return pd.DataFrame({'sharpe_ratio': [sharpe]})
""";
GO

-- Use like a regular function
WITH returns AS (
    SELECT trader, (pnl - LAG(pnl) OVER (ORDER BY date)) as returns
    FROM trades
)
SELECT trader, calc_sharpe('#returns') as sharpe
FROM returns
GROUP BY trader;
GO
```

### Use Cases

#### Use Case 1: Statistical Analysis
```sql
-- Get price series
SELECT date, close FROM prices INTO #prices;
GO

PYTHON """
from scipy import stats
import pandas as pd

df = ctx.table('#prices')

# Augmented Dickey-Fuller test for stationarity
result = stats.adfuller(df['close'])

ctx.return_table(pd.DataFrame({
    'adf_statistic': [result[0]],
    'p_value': [result[1]],
    'is_stationary': [result[1] < 0.05]
}), '#stationarity_test')
""";
GO

SELECT * FROM #stationarity_test;
GO
```

#### Use Case 2: Machine Learning
```sql
-- Get features
SELECT * FROM trade_features INTO #features;
GO

PYTHON """
from sklearn.ensemble import RandomForestClassifier
import pandas as pd

df = ctx.table('#features')

X = df[['volume', 'volatility', 'spread']]
y = df['profitable']

model = RandomForestClassifier()
model.fit(X, y)

df['prediction'] = model.predict(X)
df['probability'] = model.predict_proba(X)[:, 1]

ctx.return_table(df, '#predictions')
""";
GO

SELECT * FROM #predictions WHERE probability > 0.8;
GO
```

#### Use Case 3: Custom Business Logic
```sql
PYTHON """
# Complex P&L calculation with Python
import pandas as pd

trades = ctx.table('#trades')
positions = ctx.table('#positions')

# Custom calculation logic
pnl = calculate_complex_pnl(trades, positions)  # Your function

ctx.return_table(pnl, '#pnl_results')
""";
GO
```

### Implementation Components

#### Component 1: Python Runtime (pyo3)
```rust
// Cargo.toml
[dependencies]
pyo3 = { version = "0.20", features = ["auto-initialize"] }

// src/python/runtime.rs
use pyo3::prelude::*;

pub struct PythonRuntime {
    interpreter: Python,
}

impl PythonRuntime {
    pub fn new() -> Result<Self> {
        pyo3::prepare_freethreaded_python();
        Ok(Self {
            interpreter: Python::acquire_gil()
        })
    }

    pub fn execute(
        &self,
        code: &str,
        context: &PythonContext
    ) -> Result<()> {
        Python::with_gil(|py| {
            // Inject context
            let locals = PyDict::new(py);
            locals.set_item("ctx", context.to_py(py))?;

            // Execute code
            py.run(code, None, Some(locals))?;
            Ok(())
        })
    }
}
```

#### Component 2: Python Context Bridge
```rust
// src/python/context.rs

#[pyclass]
pub struct PythonContext {
    temp_tables: Arc<Mutex<TempTableRegistry>>,
}

#[pymethods]
impl PythonContext {
    /// Get a temp table as pandas DataFrame
    fn table(&self, name: &str) -> PyResult<PyObject> {
        let tables = self.temp_tables.lock().unwrap();
        let table = tables.get(name)?;

        // Convert DataTable → pandas DataFrame
        datatable_to_dataframe(table)
    }

    /// Store pandas DataFrame as temp table
    fn return_table(
        &self,
        df: PyObject,
        name: &str
    ) -> PyResult<()> {
        // Convert pandas DataFrame → DataTable
        let table = dataframe_to_datatable(df)?;

        let mut tables = self.temp_tables.lock().unwrap();
        tables.insert(name.to_string(), Arc::new(table))?;
        Ok(())
    }

    /// Execute SQL query from Python
    fn query(&self, sql: &str) -> PyResult<PyObject> {
        // Execute SQL and return as DataFrame
        todo!()
    }
}
```

#### Component 3: DataTable ↔ DataFrame Conversion
```rust
// src/python/conversion.rs

/// Convert DataTable to pandas DataFrame
fn datatable_to_dataframe(table: &DataTable) -> PyResult<PyObject> {
    Python::with_gil(|py| {
        let pandas = py.import("pandas")?;

        // Build dict of columns
        let mut data = HashMap::new();
        for (i, col_name) in table.column_names().iter().enumerate() {
            let values: Vec<PyObject> = table
                .get_column(i)
                .iter()
                .map(|val| datavalue_to_py(val, py))
                .collect();
            data.insert(col_name.clone(), values);
        }

        // Create DataFrame
        pandas.call_method1("DataFrame", (data,))
    })
}

/// Convert pandas DataFrame to DataTable
fn dataframe_to_datatable(df: PyObject) -> Result<DataTable> {
    Python::with_gil(|py| {
        let df = df.as_ref(py);

        // Extract columns
        let columns: Vec<String> = df
            .getattr("columns")?
            .call_method0("tolist")?
            .extract()?;

        // Extract rows
        let values_array = df.call_method0("values")?;
        // ... convert to DataTable format

        DataTable::new(columns, rows)
    })
}
```

### Python Package Management

**Option 1: Fixed Standard Library Only**
- Pros: Simple, no dependencies, secure
- Cons: Limited functionality
- Use: Development/testing phase

**Option 2: Pre-bundled Packages**
- Ship with pandas, numpy, scipy
- Pros: Predictable, secure
- Cons: Fixed versions, larger binary

**Option 3: User-managed Virtual Env**
- Allow `.venv` in project directory
- User runs `pip install pandas`
- Pros: Flexible, full ecosystem
- Cons: Security concerns, version conflicts

**Recommendation: Start with Option 2, add Option 3 later**

### Security Considerations

**Sandboxing:**
- Restrict file system access (read-only)
- No network access from Python (only through WEB CTE)
- Memory limits
- Execution timeout

**Code Review:**
- Scripts are code - treat with caution
- Consider signed scripts
- Audit logging of Python execution

---

## Phase 2C: Lua Scripting 🌙

### Priority: **MEDIUM** (Alternative to Python)

### Why Lua?

**Pros:**
- Lightweight (~200KB vs Python's MBs)
- Already used in many embedded systems (Redis, Nginx, etc.)
- Easy to sandbox
- Fast execution
- Simple integration with Rust

**Cons:**
- Smaller ecosystem than Python
- Fewer data science libraries
- Less familiar to most users

### Use Cases

Lua is better suited for:
- **Simple transformations** (data munging)
- **Custom business rules** (policy enforcement)
- **Lightweight calculations** (not heavy stats)

### Example Syntax
```sql
SELECT * FROM trades INTO #trades;
GO

LUA """
local trades = ctx:get_table('#trades')
local results = {}

for i, trade in ipairs(trades) do
    if trade.amount > 100000 then
        table.insert(results, {
            trade_id = trade.id,
            risk_score = calculate_risk(trade),
            alert = trade.amount > 500000
        })
    end
end

ctx:set_table('#risk_scores', results)
""";
GO
```

### Implementation

**Lua Crate:**
```rust
// Cargo.toml
[dependencies]
mlua = "0.9"

// src/lua/runtime.rs
use mlua::prelude::*;

pub struct LuaRuntime {
    lua: Lua,
}

impl LuaRuntime {
    pub fn new() -> Result<Self> {
        let lua = Lua::new();
        Ok(Self { lua })
    }

    pub fn execute(
        &self,
        code: &str,
        context: &LuaContext
    ) -> Result<()> {
        let globals = self.lua.globals();
        globals.set("ctx", context)?;
        self.lua.load(code).exec()?;
        Ok(())
    }
}
```

**Decision Point:** Implement Lua if Python is too heavy, or if specific use cases demand it.

---

## Comparison Matrix

| Feature | Template Injection | Python | Lua |
|---------|-------------------|--------|-----|
| **Complexity** | Low | High | Medium |
| **Dev Time** | 1-2 days | 3-5 days | 2-3 days |
| **Runtime Overhead** | None | ~50MB + startup | ~200KB |
| **Use Cases** | API data injection | Stats, ML, complex logic | Simple transforms |
| **Ecosystem** | N/A | Massive (pandas, numpy, sklearn) | Limited |
| **Security** | Safe | Needs sandboxing | Easy to sandbox |
| **Debugging** | Easy | Medium | Medium |
| **Familiarity** | High (SQL users) | High (general) | Low |

---

## Recommended Roadmap

### **Phase 2A: Template Injection** (Week 1) ⭐
**Goal:** Enable dynamic WEB CTE queries with temp table data

**Deliverables:**
1. Template parser for `${#table}` syntax
2. JSON serialization of temp tables
3. Integration with WEB CTE execution
4. Test suite with mock APIs
5. Documentation + examples

**Success Metrics:**
- ✅ Can inject full tables as JSON arrays
- ✅ Can inject single columns as arrays
- ✅ Can inject single values
- ✅ Works in both BODY and URL
- ✅ Clear error messages for invalid templates

### **Phase 2B: Python Integration** (Week 2-3) 🐍
**Goal:** Embed Python for complex analysis

**Deliverables:**
1. Python runtime integration (pyo3)
2. DataTable ↔ DataFrame bridge
3. PythonContext API (get_table, return_table)
4. Basic package support (pandas, numpy)
5. Error handling and sandboxing

**Success Metrics:**
- ✅ Can execute Python code blocks
- ✅ Can pass data to/from Python
- ✅ Can use pandas for transformations
- ✅ Proper error propagation
- ✅ Memory/timeout limits enforced

### **Phase 2C: Advanced Features** (Ongoing)
**Future enhancements:**
- Python stored procedures (CREATE PYTHON FUNCTION)
- Lua support (if needed)
- Python package management
- Streaming large datasets
- Async execution
- Python ↔ SQL type system improvements

---

## Open Questions

### 1. Python Package Management
**Q:** Allow users to install packages, or ship with fixed set?
**Options:**
- A: Ship with pandas/numpy/scipy only (simple, secure)
- B: Allow .venv with pip install (flexible, complex)
- C: Hybrid - allow allowlist of packages

**Decision:** Start with A, revisit based on user needs

### 2. Python Code Storage
**Q:** Where to store reusable Python functions?
**Options:**
- A: Inline in SQL scripts only
- B: CREATE PYTHON FUNCTION (stored in metadata)
- C: External .py files that can be imported
- D: All of the above

**Decision:** Start with A (inline), add B (stored) in Phase 2C

### 3. Execution Model
**Q:** How to handle Python execution time?
**Options:**
- A: Synchronous (block until Python returns)
- B: Async (continue processing, wait when results needed)
- C: Background jobs

**Decision:** Start with A, add B if performance issues

### 4. Type System
**Q:** How to handle SQL ↔ Python type mismatches?
**Options:**
- A: Best-effort conversion (int → float, etc.)
- B: Strict typing with errors
- C: Explicit type hints in context

**Decision:** Start with A, provide clear error messages

---

## Example Workflows

### Workflow 1: FIX Log Analysis → Trade DB → Risk System
```sql
-- Parse FIX logs for trades
WITH fix_trades AS (
    WEB(
        URL 'https://fix-engine/parse',
        BODY {"date": "2025-10-07", "msg_type": "ExecutionReport"}
    )
)
SELECT
    order_id,
    symbol,
    qty,
    price,
    DATETIME_PARSE(transact_time, 'FIX') as timestamp
FROM fix_trades
INTO #fix_trades;
GO

-- Get timing analysis with Python
PYTHON """
import pandas as pd

df = ctx.table('#fix_trades')
df = df.sort_values('timestamp')

# Calculate latencies
df['latency'] = df.groupby('order_id')['timestamp'].diff()

# Flag outliers
df['outlier'] = df['latency'] > df['latency'].quantile(0.95)

ctx.return_table(df[df['outlier']], '#latency_outliers')
""";
GO

-- Get order IDs with high latency
SELECT DISTINCT order_id FROM #latency_outliers INTO #problem_orders;
GO

-- Query internal trade DB for those orders
SELECT * FROM trades
WHERE order_id IN (SELECT * FROM #problem_orders)
INTO #trade_details;
GO

-- Fetch risk metrics from Front Arena for problem trades
WITH risk_metrics AS (
    WEB(
        URL 'https://front-arena/api/risk/orders',
        METHOD 'POST',
        BODY {
            "order_ids": ${#problem_orders.order_id},
            "metrics": ["var", "delta", "credit_exposure"]
        }
    )
)
SELECT
    t.*,
    r.var,
    r.delta,
    r.credit_exposure
FROM #trade_details t
JOIN risk_metrics r ON t.order_id = r.order_id
WHERE r.var > 1000000;
GO
```

### Workflow 2: Multi-System Data Enrichment
```sql
-- Get distinct instruments from trades
SELECT DISTINCT instrument
FROM trades
WHERE date = TODAY()
INTO #instruments;
GO

-- Fetch security master data
WITH sec_master AS (
    WEB(
        URL 'https://sec-master/api/securities',
        BODY {"symbols": ${#instruments.instrument}}
    )
)
SELECT * FROM sec_master INTO #security_info;
GO

-- Get market data
WITH market_data AS (
    WEB(
        URL 'https://market-data/quotes',
        BODY {"symbols": ${#instruments.instrument}}
    )
)
SELECT * FROM market_data INTO #quotes;
GO

-- Python: Calculate Greeks
PYTHON """
import pandas as pd
from scipy.stats import norm
import numpy as np

sec = ctx.table('#security_info')
quotes = ctx.table('#quotes')

# Merge data
df = sec.merge(quotes, on='symbol')

# Black-Scholes calculations
# (simplified example)
df['delta'] = norm.cdf(
    (np.log(df['spot'] / df['strike']) +
     (0.05 + 0.5 * df['vol']**2) * df['tte']) /
    (df['vol'] * np.sqrt(df['tte']))
)

ctx.return_table(df, '#greeks')
""";
GO

-- Final report
SELECT
    t.instrument,
    t.position,
    g.delta,
    g.delta * t.position as portfolio_delta
FROM trades t
JOIN #greeks g ON t.instrument = g.symbol;
GO
```

---

## Success Criteria

### Phase 2A Success
- [ ] Can inject temp table data into WEB CTE requests
- [ ] Supports full table, column array, and single value injection
- [ ] Works in both URL and BODY
- [ ] Clear error messages
- [ ] Example scripts demonstrating multi-system queries
- [ ] Documentation complete

### Phase 2B Success
- [ ] Can execute Python code within SQL scripts
- [ ] Can pass data between SQL and Python seamlessly
- [ ] Can use pandas for data transformation
- [ ] Proper error handling and user feedback
- [ ] Example scripts showing statistical analysis
- [ ] Documentation + Python API reference

### Overall Success
- [ ] Can chain queries across: FIX engine → Trade DB → Risk System
- [ ] Can reduce API payload sizes by 90%+ with targeted queries
- [ ] Can perform complex analysis in Python without leaving sql-cli
- [ ] Workflow is intuitive for SQL users
- [ ] Performance is acceptable (<1s overhead per Python block)

---

## Notes for Implementation

### Template Injection Gotchas
- **Large tables:** Warn if serializing >1000 rows
- **Nested structures:** Handle tables with complex types
- **Empty tables:** Inject `[]` not error
- **SQL injection:** User must trust temp table data sources

### Python Integration Gotchas
- **GIL:** Python Global Interpreter Lock limits parallelism
- **Memory:** Python can consume significant RAM
- **Startup time:** First Python execution has ~50ms overhead
- **Package conflicts:** Different pandas versions can cause issues
- **Type mismatches:** Python None vs SQL NULL handling

### Testing Strategy
- **Unit tests:** Template parser, JSON serialization
- **Integration tests:** End-to-end scripts with mock servers
- **Performance tests:** Large table injection (10K+ rows)
- **Error tests:** Invalid templates, missing tables, type errors

---

## Future Possibilities

### Phase 3+ Ideas
- **R Integration:** For statistical users
- **JavaScript/V8:** For web developers
- **WebAssembly:** For portable compiled functions
- **SQL Functions:** `CREATE FUNCTION calc_sharpe(...) AS PYTHON`
- **Async Execution:** Background Python jobs
- **Distributed:** Execute Python on remote workers
- **Streaming:** Process large datasets incrementally
- **Caching:** Cache Python function results

---

## Conclusion

Phase 2 will transform sql-cli from a powerful SQL tool into a complete data pipeline platform. The combination of:

1. **SQL** - Familiar query language
2. **Temporary Tables** - Intermediate results storage
3. **WEB CTEs** - API integration
4. **Template Injection** - Dynamic queries
5. **Python** - Complex analysis & ML

...creates an unprecedented level of flexibility for data analysis workflows.

**Start with template injection (Phase 2A)** - this is the highest-value, lowest-complexity enhancement that immediately unlocks the full potential of your multi-system data ecosystem.

**Next Steps:**
1. Review and approve this design
2. Start implementation of template injection
3. Test with real FIX logs + API calls
4. Gather feedback
5. Iterate toward Python integration

---

*Document Version: 1.0*
*Date: 2025-10-07*
*Author: Claude Code + User Collaboration*

# SQL CLI - Advanced SQL Editor for Trading Platform

A powerful terminal-based SQL editor with syntax-aware completion, advanced parsing, and intelligent caching for querying trading data.

## 🎯 Key Features

### Data Source Support
- **API Mode**: Query live trading data from REST API
- **CSV Files**: Load and query CSV files directly
- **JSON Files**: Load and query JSON files (array of flat objects)
- **Cached Mode**: Save API results for offline analysis

### Smart SQL Parsing & Completion
- **Recursive Descent Parser**: Handles complex queries with multiple AND/OR conditions
- **Context-Aware Tab Completion**: Suggests only relevant columns, keywords, and operators based on cursor position
- **Method Call Support**: `.Contains()`, `.StartsWith()`, `.EndsWith()` for string matching
- **DateTime Constructor**: Native support for `DateTime(year, month, day)` in queries
- **IN Clause Support**: Multiple value matching with `IN ("value1", "value2")`

### Advanced Data Analysis
- **Virtual Scrolling**: Handle datasets with 1000+ rows and 190+ columns efficiently
- **Column Sorting**: Click headers or use `s` key to sort ascending/descending
- **Regex Filtering**: Filter results with `/pattern` command
- **Vim-style Search**: Search within results using `?` with `n`/`N` navigation
- **CSV Export**: Export current view with `Ctrl+S`

### Enhanced Navigation
- **Word-based Navigation**: `Ctrl+Left/Right` or `Alt+B/F` jumps between SQL tokens
- **Line Navigation**: `Ctrl+A` (beginning), `Ctrl+E` (end of line)
- **Token Position Indicator**: Shows "Token: 5/12" in status bar
- **Horizontal Scrolling**: Long queries scroll horizontally instead of wrapping
- **Command History**: `Ctrl+R` for mcfly-style fuzzy search through past queries

## 🚀 Quick Start

### Load a JSON file
```bash
# JSON file should contain an array of flat objects
cargo run -- --enhanced sample_trades.json

# Example JSON structure:
# [
#   {"id": 1, "counterparty": "Bank of America", "commission": 75.25, ...},
#   {"id": 2, "counterparty": "JP Morgan", "commission": 100.00, ...}
# ]
```

### Load a CSV file
```bash
cargo run -- --enhanced trades.csv
```

### Connect to API
```bash
cargo run -- --enhanced
# Or specify custom API URL:
cargo run -- --enhanced --api-url http://localhost:5000
```

## 📋 Quick Query Reference

### Supported SQL Syntax
```sql
-- Basic structure
SELECT * FROM table WHERE conditions ORDER BY column

-- Comparison operators
WHERE column = "value"              -- Equal
WHERE column != "value"             -- Not equal
WHERE column > 100                  -- Greater than
WHERE column >= 100                 -- Greater than or equal
WHERE column < 100                  -- Less than
WHERE column <= 100                 -- Less than or equal

-- Logical operators
WHERE condition1 AND condition2     -- Both must be true
WHERE condition1 OR condition2      -- Either can be true
WHERE NOT condition                 -- Negates condition
WHERE (cond1 OR cond2) AND cond3   -- Parentheses for grouping

-- NULL handling
WHERE column IS NULL                -- Check for NULL
WHERE column IS NOT NULL            -- Check for non-NULL

-- List operations
WHERE column IN ("A", "B", "C")     -- Value in list
WHERE column NOT IN ("X", "Y")      -- Value not in list

-- Range operations
WHERE column BETWEEN 10 AND 20      -- Inclusive range

-- String operations
WHERE column LIKE "prefix%"         -- Pattern matching
WHERE column.Contains("text")       -- Substring search
WHERE column.StartsWith("text")     -- Prefix check
WHERE column.EndsWith("text")       -- Suffix check
WHERE column.Length() > 10          -- String length

-- Case conversion
WHERE column.ToLower() = "value"    -- Case-insensitive
WHERE column.ToUpper() = "VALUE"    -- Case-insensitive

-- Date operations
WHERE date > DateTime(2024, 1, 1)   -- After date
WHERE date = DateTime()             -- Today

-- Ordering
ORDER BY column                     -- Ascending (default)
ORDER BY column DESC                -- Descending
ORDER BY col1, col2                 -- Multiple columns
```

## 🚀 Example Queries

### Basic Examples

#### Simple Filtering
```sql
-- Numeric comparison
SELECT * FROM trades WHERE commission > 100

-- String equality
SELECT * FROM trades WHERE counterpartyCountry = "US"

-- Multiple conditions
SELECT * FROM trades WHERE commission > 50 AND quantity < 1000
```

#### LINQ-Style String Methods
```sql
-- Case-sensitive contains
SELECT * FROM trades WHERE counterparty.Contains("Bank")

-- Prefix matching
SELECT * FROM trades WHERE platformOrderId.StartsWith("ORD2024")

-- Suffix matching
SELECT * FROM trades WHERE instrumentName.EndsWith("Bond")
```

### Advanced Query Examples

#### NOT Operator Support
```sql
-- Negate any condition with NOT
SELECT * FROM trades WHERE NOT Country.Contains("US")

-- NOT with IN lists
SELECT * FROM trades WHERE Country NOT IN ("US", "CA", "MX")

-- Complex NOT conditions
SELECT * FROM trades 
WHERE NOT (status = "CANCELLED" OR status = "FAILED")
  AND NOT commission > 1000

-- NOT with method calls
SELECT * FROM trades 
WHERE NOT Country.StartsWith("U")
  AND NOT City.Contains("New")
```

#### Complex Boolean Logic with Parentheses
```sql
-- Parentheses override default precedence
SELECT * FROM trades 
WHERE (status = "Active" OR priority = "High") 
  AND (region IN ("US", "EU") OR commission > 1000)

-- Nested conditions with mixed operators
SELECT * FROM trades 
WHERE counterparty.Contains("Bank") 
  AND (
    (executionSide = "BUY" AND quantity > 500) 
    OR (executionSide = "SELL" AND commission < 50)
  )
```

#### Case-Insensitive Comparisons
```sql
-- ToLower() for case-insensitive matching
SELECT * FROM trades 
WHERE executionSide.ToLower() = "buy"
  AND status.ToLower() != "cancelled"

-- ToUpper() with multiple operators
SELECT * FROM trades 
WHERE status.ToUpper() IN ("COMPLETED", "PENDING")
  OR counterpartyCountry.ToUpper() = "US"

-- Mixed case conversions in complex queries
SELECT * FROM trades 
WHERE executionSide.ToLower() = "buy" 
  AND counterparty.ToUpper().Contains("BANK")
  AND status.ToLower() != "failed"
```

#### Advanced DateTime Filtering
```sql
-- DateTime constructor with full precision
SELECT * FROM trades 
WHERE createdDate >= DateTime(2024, 01, 15, 09, 30, 00)
  AND createdDate < DateTime(2024, 01, 15, 17, 00, 00)

-- Date ranges with business logic
SELECT * FROM trades 
WHERE tradeDate BETWEEN DateTime(2024,01,01) AND DateTime(2024,03,31)
  AND settlement_date > DateTime(2024,01,05)

-- Today's date (empty DateTime constructor)
SELECT * FROM trades 
WHERE createdDate >= DateTime()
```

#### Complex IN and NOT IN Operations
```sql
-- Multiple country filtering
SELECT * FROM trades 
WHERE counterpartyCountry IN ("JP", "FR", "DE", "UK")
  AND executionSide NOT IN ("CANCEL", "REJECT")

-- Combining IN with other conditions
SELECT * FROM trades 
WHERE counterpartyCountry IN ("US", "CA") 
  AND commission BETWEEN 50 AND 200
  AND counterparty.Contains("Bank")
```

#### BETWEEN Queries with Complex Logic
```sql
-- Numeric ranges with additional filters
SELECT * FROM trades 
WHERE commission BETWEEN 100 AND 500
  AND quantity BETWEEN 1000 AND 10000
  AND counterpartyCountry = "US"

-- Date and numeric ranges combined
SELECT * FROM trades 
WHERE createdDate BETWEEN DateTime(2024,01,01) AND DateTime(2024,12,31)
  AND price BETWEEN 50.00 AND 200.00
  AND status.ToLower() = "completed"
```

#### NULL Handling
```sql
-- Check for null values
SELECT * FROM trades 
WHERE settlement_date IS NULL
  AND status != "CANCELLED"

-- Exclude null values
SELECT * FROM trades 
WHERE commission IS NOT NULL
  AND counterparty IS NOT NULL
  AND quantity > 0

-- Complex NULL checks
SELECT * FROM trades 
WHERE (City IS NULL OR Phone IS NULL)
  AND Age IS NOT NULL

-- NULL values in CSV files
-- Empty fields in CSV (e.g., Name,,City) are treated as NULL
SELECT * FROM customers WHERE Age IS NULL

-- Combine NULL checks with other conditions
SELECT * FROM trades 
WHERE Phone IS NULL 
  AND Country.Contains("US")
  AND NOT Status = "INACTIVE"
```

#### String Length and Pattern Matching
```sql
-- String length comparisons
SELECT * FROM trades 
WHERE platformOrderId.Length() > 10
  AND counterparty.Length() BETWEEN 5 AND 50

-- LIKE pattern matching
SELECT * FROM trades 
WHERE platformOrderId LIKE "ORD%2024%"
  OR instrumentName LIKE "%Bond%"
```

#### Production-Ready Complex Queries
```sql
-- High-value US bank trades from Q1 2024
SELECT * FROM trades 
WHERE counterpartyCountry = "US"
  AND counterparty.ToUpper().Contains("BANK")
  AND commission > 1000
  AND tradeDate BETWEEN DateTime(2024,01,01) AND DateTime(2024,03,31)
  AND status.ToLower() IN ("completed", "settled")

-- Risk analysis: Large trades with specific criteria
SELECT * FROM trades 
WHERE (
    (executionSide.ToLower() = "buy" AND quantity > 10000)
    OR (executionSide.ToLower() = "sell" AND quantity > 5000)
  )
  AND counterpartyCountry NOT IN ("US", "CA")
  AND commission BETWEEN 500 AND 2000
  AND createdDate >= DateTime(2024,06,01)
  AND status.ToLower() != "cancelled"

-- Multi-criteria filtering with parentheses precedence
SELECT * FROM trades 
WHERE (
    counterparty.Contains("Morgan") OR counterparty.Contains("Goldman")
  ) 
  AND (
    (region = "APAC" AND commission > 200)
    OR (region = "EMEA" AND commission > 150)
    OR (region = "Americas" AND commission > 300)
  )
  AND executionSide.ToLower() IN ("buy", "sell")
  AND tradeDate > DateTime(2024,01,01)
```

### JSON vs CSV Query Compatibility

All examples work identically with both JSON and CSV files:

```bash
# Same query works with both formats
cargo run -- --enhanced trades.csv
cargo run -- --enhanced trades.json

# Query: SELECT * FROM trades WHERE commission > 100 AND status.ToLower() = "completed"
# Results are identical regardless of source file format
```

## ⌨️ Keyboard Shortcuts

### Query Editor Mode
| Key | Action |
|-----|--------|
| `Tab` | Context-aware SQL completion |
| `Enter` | Execute query |
| `Ctrl+A` / `Ctrl+E` | Jump to start/end of line |
| `Ctrl+Left` / `Ctrl+Right` | Navigate by word |
| `Alt+B` / `Alt+F` | Alternative word navigation |
| `Ctrl+P` / `Ctrl+N` | Previous/Next command |
| `Ctrl+R` | Search command history |
| `F5` | Show parser debug info |
| `Ctrl+D` | Exit |

### Results View Mode
| Key | Action |
|-----|--------|
| `j`/`k` or `↑`/`↓` | Navigate rows |
| `h`/`l` or `←`/`→` | Navigate columns |
| `q` | Return to query editor |
| `s` | Sort by current column |
| `/` | Filter rows (regex) |
| `?` | Search in results |
| `n`/`N` | Next/Previous match |
| `Ctrl+S` | Export to CSV |
| `Esc` | Clear filter/search |

## 💾 Caching for Large Datasets

### Cache Commands (Enhanced TUI Mode)
| Command | Description |
|---------|-------------|
| `:cache save` | Save current query results to cache |
| `:cache load <id>` | Load cached query by ID and enable cache mode |
| `:cache list` | List all cached queries (also accessible via F7) |
| `:cache clear` | Disable cache mode and return to live queries |

### Workflow Example
```bash
# 1. Fetch large dataset (expensive API call)
sql> SELECT * FROM trade_deal WHERE tradeDate > DateTime(2024,01,01)
# Returns: 10,000 rows in 1250ms

# 2. Return to SQL input (press Escape if in view mode)
# 3. Clear the input line (Ctrl+U) and save to cache
sql> :cache save
# Status shows: "Query cached with ID: 1 (10000 rows)"

# 4. Load cached data for offline work
sql> :cache load 1
# Status shows: "Loaded cache ID 1 with 10000 rows. Cache mode enabled."

# 5. Run unlimited local queries (instant!)
sql> SELECT * FROM trade_deal WHERE counterparty.Contains("Bank")
# Returns: 2,341 rows in 0ms [CACHED]

# 6. Press F7 to manage cached queries visually
```

### How It Works
- When in cache mode, queries run locally against cached data using the CSV query engine
- Much faster than hitting the server for repeated analysis
- Perfect for exploring large datasets without repeated API calls
- Cache persists between sessions

## 🏗️ Architecture

### Components
1. **Rust CLI** (`sql-cli`)
   - Terminal UI with ratatui
   - Recursive descent SQL parser
   - Local caching system
   - CSV export functionality

2. **C# API Server** (your existing server)
   - Proxy to expensive trading platform API
   - Handles bearer token authentication
   - No caching - just pass-through

3. **Local Cache** (planned)
   - JSON files for data storage
   - Metadata tracking (timestamps, row counts)
   - Query deduplication via SHA-256 hashing

## 🔧 Installation

```bash
# Clone the repository
git clone https://github.com/TimelordUK/sql-cli.git
cd sql-cli/sql-cli

# Build the project
cargo build --release

# Run the CLI
./target/release/sql-cli

# Or with specific API URL
TRADE_API_URL=http://your-server:5000 ./target/release/sql-cli
```

## ⚙️ Configuration

### Environment Variables
- `TRADE_API_URL`: Your C# API server URL (default: `http://localhost:5000`)

### Schema Configuration
The CLI loads column definitions from `schema.json`:
```json
{
  "tables": {
    "trade_deal": {
      "columns": [
        "dealId", "platformOrderId", "counterparty",
        "commission", "counterpartyCountry", "createdDate",
        // ... 90+ columns
      ]
    }
  }
}
```

## 📊 Status Bar Information

The status bar shows:
- **Mode**: Current mode (Command/Results/History)
- **Token Position**: e.g., "Token: 5/12" showing position in query
- **Query Status**: Execution time and row count
- **Filter/Search**: Active filters and matches

## 🎨 SQL Syntax Highlighting

The editor provides color-coded syntax:
- **Keywords**: Blue (SELECT, FROM, WHERE)
- **Strings**: Green ('value', "text")
- **Numbers**: Cyan (123, 45.67)
- **Operators**: Yellow (=, >, AND, OR)
- **Comments**: Gray (-- comment)

## 🌳 AST-Based WHERE Clause Processing

The SQL CLI uses a custom Abstract Syntax Tree (AST) parser for WHERE clauses, providing robust and reliable query filtering for cached data, CSV files, and JSON files. This approach was chosen over alternatives like tree-sitter for its simplicity and perfect fit for our SQL subset.

### Key Benefits

1. **Correct Operator Precedence**: The parser respects standard SQL precedence rules:
   - Comparisons (=, >, <, etc.) have highest precedence
   - NOT comes next
   - AND binds tighter than OR
   - Parentheses override default precedence

2. **No String Manipulation**: Unlike string-based parsing, the AST approach:
   - Handles operators in string values correctly
   - Eliminates case-sensitivity issues
   - Avoids regex complexity for DateTime() parsing
   - Prevents edge cases with nested conditions

3. **Clean Architecture**:
   - **Lexer** (`recursive_parser.rs`): Tokenizes the SQL query
   - **Parser** (`where_parser.rs`): Builds AST from tokens using recursive descent
   - **AST Walker** (`where_ast.rs`): Evaluates expressions against data rows

### Example: How Queries Are Processed

```sql
SELECT * FROM trades WHERE (status = "Active" OR priority = "High") AND region IN ("US", "EU")
```

This query becomes the following AST:
```
AND
  OR
    EQUAL(status, "Active")
    EQUAL(priority, "High")
  IN(region, ["US", "EU"])
```

The walker then:
1. Evaluates `status = "Active"` → false
2. Evaluates `priority = "High"` → true
3. Combines with OR → true
4. Evaluates `region IN ("US", "EU")` → true
5. Combines with AND → true (row matches)

### Debugging with F5

Press F5 in the TUI to see the AST visualization of your WHERE clause, helping you understand exactly how your query is parsed and why certain rows match or don't match.

### Supported Operations

All standard SQL operators plus LINQ-style methods:

#### Basic Operations
- **Comparisons**: `=`, `!=`, `<`, `>`, `<=`, `>=`
- **Logical**: `AND`, `OR`, `NOT`
- **Grouping**: Parentheses `()` for precedence control

#### Advanced Operations
- **Range**: `BETWEEN value1 AND value2`
- **List Membership**: `IN (value1, value2, ...)`, `NOT IN (value1, value2, ...)`
- **Pattern Matching**: `LIKE pattern` (% = any chars, _ = single char)
- **NULL Checks**: `IS NULL`, `IS NOT NULL`

#### LINQ-Style String Methods
- `.Contains("text")` - Case-sensitive substring search
- `.StartsWith("prefix")` - Check string prefix
- `.EndsWith("suffix")` - Check string suffix
- `.Length()` - Get string length for comparison
- `.ToLower()` - Convert to lowercase for comparison
- `.ToUpper()` - Convert to uppercase for comparison

#### DateTime Support
- `DateTime(year, month, day)` - Date at midnight
- `DateTime(year, month, day, hour, minute, second)` - Full precision
- `DateTime()` - Today at midnight
- Works with all comparison operators

#### Column Name Handling
- **Unquoted**: `columnName` (alphanumeric + underscore)
- **Quoted**: `"Column Name"` (for spaces or special chars)
- **Case Sensitivity**: Column names are case-insensitive by default

#### ORDER BY Support
- `ORDER BY column` - Sort ascending
- `ORDER BY column DESC` - Sort descending
- `ORDER BY column1, column2` - Multi-column sort
- Works with WHERE clauses: `WHERE condition ORDER BY column`

### JSON and CSV Compatibility

JSON and CSV files are loaded into identical internal structures, meaning:
- Same query syntax works for both file types
- Tab completion shows columns from either format
- WHERE clause filtering uses the same AST parser
- Performance is identical for equivalent data

## ⚠️ Limitations and Special Cases

### What's NOT Supported
- **JOIN operations**: Single table queries only
- **Aggregate functions**: No SUM, COUNT, AVG, etc.
- **GROUP BY / HAVING**: Not implemented
- **Subqueries**: Not supported
- **UNION / INTERSECT**: Not available
- **UPDATE / DELETE**: Read-only queries
- **DISTINCT**: Not implemented
- **Column aliases**: No AS support
- **SELECT specific columns**: Currently only `SELECT *` works

### Special Behaviors
1. **Case Sensitivity**:
   - Column names: Case-insensitive (`Country` = `country` = `COUNTRY`)
   - String values: Case-sensitive unless using `.ToLower()` or `.ToUpper()`
   - Keywords: Case-insensitive (`WHERE` = `where` = `Where`)

2. **String Quoting**:
   - Double quotes `"` for identifiers and string values
   - Single quotes `'` also work for string values
   - Column names with spaces must be quoted: `"Customer Id"`

3. **NULL Handling**:
   - Empty CSV fields become NULL
   - Cannot use `= NULL` or `!= NULL`, must use `IS NULL` / `IS NOT NULL`
   - NULL is not equal to empty string ""

4. **Method Calls**:
   - Methods like `.Contains()` are case-sensitive
   - Methods must have parentheses even if no arguments: `.Length()`
   - Methods can be chained: `.ToLower().Contains("text")`

5. **Operator Precedence** (highest to lowest):
   - Parentheses `()`
   - Method calls `.method()`
   - Comparisons `=, !=, <, >, <=, >=, IN, LIKE, IS NULL`
   - `NOT`
   - `AND`
   - `OR`

6. **Data Type Handling**:
   - Numbers in CSV are auto-detected and parsed as floats
   - Dates must use `DateTime()` constructor for comparisons
   - Boolean values not directly supported (use string comparison)

## 🚧 Roadmap

- [x] Recursive descent parser
- [x] Tab completion with context awareness
- [x] DateTime constructor support
- [x] Method call support (.Contains, etc.)
- [x] IN clause support
- [x] Virtual scrolling for large datasets
- [x] CSV export
- [x] Command history with fuzzy search
- [x] Complete caching implementation
- [x] AST visualization (F5 debug view)
- [ ] Server proxy endpoint
- [ ] Offline mode
- [ ] Query performance profiling

## 🤝 Contributing

This is an internal tool for trading platform data analysis. For issues or feature requests, please contact the development team.

## 📝 License

Proprietary - Internal Use Only
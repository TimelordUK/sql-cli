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

## 🚀 Example Queries

### Complex Multi-condition Query
```sql
SELECT * FROM trade_deal 
WHERE commission > 50 
  AND counterparty.Contains("Bank") 
  AND counterpartyCountry IN ("JP","FR") 
  AND createdDate > DateTime(2025,07,01)
```

This query demonstrates:
- Numeric comparisons (`commission > 50`)
- String method calls (`counterparty.Contains("Bank")`)
- IN clause with multiple values (`IN ("JP","FR")`)
- DateTime constructor (`DateTime(2025,07,01)`)

### String Matching Examples
```sql
-- Find all US banks
SELECT * FROM trade_deal 
WHERE counterparty.Contains("Bank") 
  AND counterpartyCountry = "US"

-- Case-insensitive matching
SELECT * FROM trade_deal
WHERE executionSide.ToLower() = "buy"
  OR status.ToUpper() = "COMPLETED"

-- Find orders with specific prefix
SELECT * FROM trade_deal 
WHERE platformOrderId.StartsWith("ORD2024")

-- Complex pattern matching
SELECT * FROM trade_deal 
WHERE instrumentName.Contains("Bond") 
  OR instrumentName.EndsWith("Note")
```

### Date Range Queries
```sql
-- Trades in Q1 2024
SELECT * FROM trade_deal 
WHERE tradeDate >= DateTime(2024,01,01) 
  AND tradeDate < DateTime(2024,04,01)

-- Recent trades with high commission
SELECT * FROM trade_deal 
WHERE createdDate > DateTime(2024,06,01) 
  AND commission > 1000
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
- **Comparisons**: =, !=, <, >, <=, >=
- **Logical**: AND, OR, NOT
- **Special**: BETWEEN, IN, NOT IN, LIKE, IS NULL, IS NOT NULL
- **LINQ Methods**: .Contains(), .StartsWith(), .EndsWith(), .Length(), .ToLower(), .ToUpper()
- **DateTime**: DateTime(year, month, day, hour, minute, second)

### JSON and CSV Compatibility

JSON and CSV files are loaded into identical internal structures, meaning:
- Same query syntax works for both file types
- Tab completion shows columns from either format
- WHERE clause filtering uses the same AST parser
- Performance is identical for equivalent data

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
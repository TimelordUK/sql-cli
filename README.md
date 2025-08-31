# SQL CLI - Powerful CSV/JSON TUI with Advanced SQL Engine

**A vim-inspired terminal UI for CSV and JSON files with sophisticated SQL query capabilities, intelligent completion, and lightning-fast navigation.**

![SQL-CLI Overview](sql-cli/demos/overview.gif)

## 🚀 Why SQL CLI?

**Think `less` for CSV files, but with SQL superpowers:**
- **📁 Point & Query**: Drop any CSV/JSON file and immediately start querying  
- **⚡ Lightning Fast**: In-memory engine handles 100K+ rows with sub-second response
- **🎯 Vim-Inspired**: Modal editing, `hjkl` navigation, powerful keyboard shortcuts
- **🧠 Smart Completion**: Context-aware SQL completion with fuzzy matching
- **🔍 Advanced Filtering**: Regex, fuzzy search, complex WHERE clauses
- **📊 Rich SQL Features**: JOINs, aggregations, date functions, string manipulation

![SQL-CLI CSV Demo](sql-cli/demos/overview-optimized.gif)

## ⚡ Quick Start

```bash
# Install from Cargo  
cargo install sql-cli

# Point at any CSV or JSON file
sql-cli data.csv

# Immediately start querying with full SQL support
SELECT * FROM data WHERE amount > 1000 ORDER BY date DESC LIMIT 10
```

## 💪 Powerful SQL Engine Features

### 🔥 **Core SQL + Modern Extensions**
Your SQL CLI combines traditional SQL with modern LINQ-style methods and advanced functions:

```sql
-- Traditional SQL with modern LINQ methods
SELECT 
    customer_name.Trim() as name,
    email.EndsWith('.com') as valid_email,
    ROUND(price * quantity, 2) as total,
    DATEDIFF('day', order_date, NOW()) as days_ago
FROM orders 
WHERE customer_name.Contains('corp')
  AND price BETWEEN 100 AND 1000
  AND order_date > DATEADD('month', -6, TODAY())
ORDER BY total DESC 
LIMIT 25
```

### 📊 **Advanced Functions Library**

#### **Date & Time Functions**
```sql
-- Comprehensive date handling with multiple format support
SELECT 
    NOW() as current_time,                    -- 2024-08-31 15:30:45
    TODAY() as current_date,                  -- 2024-08-31  
    DATEDIFF('day', '2024-01-01', order_date) as days_since_year,
    DATEADD('month', 3, ship_date) as warranty_expires
FROM orders
WHERE DATEDIFF('year', created_date, NOW()) <= 2
```

**Supported Date Formats:**
- ISO: `2024-01-15`, `2024-01-15 14:30:00`
- US: `01/15/2024`, `01/15/2024 2:30 PM` 
- EU: `15/01/2024`, `15/01/2024 14:30`
- Excel: `15-Jan-2024`, `Jan 15, 2024`
- Full: `January 15, 2024`, `15 January 2024`

#### **Mathematical Functions**  
```sql
-- Rich mathematical operations
SELECT 
    ROUND(price * 1.08, 2) as taxed_price,
    SQRT(POWER(width, 2) + POWER(height, 2)) as diagonal,
    MOD(id, 100) as batch_number,
    ABS(actual - target) as variance,
    POWER(growth_rate, years) as compound_growth
FROM products
WHERE SQRT(area) BETWEEN 10 AND 50
```

**Available Math Functions:**
`ROUND`, `ABS`, `FLOOR`, `CEILING`, `MOD`, `QUOTIENT`, `POWER`, `SQRT`, `EXP`, `LN`, `LOG`, `LOG10`, `PI()`

#### **String & Text Functions**
```sql
-- Advanced text manipulation
SELECT 
    TEXTJOIN(' | ', 1, first_name, last_name, department) as employee_info,
    name.Trim().Length() as clean_name_length,
    email.IndexOf('@') as at_position,
    description.StartsWith('Premium') as is_premium
FROM employees
WHERE name.Contains('manager') 
  AND email.EndsWith('.com')
  AND department.Trim() != ''
```

**LINQ-Style String Methods:**
- `column.Contains('text')` - Case-insensitive substring search
- `column.StartsWith('prefix')` - Case-insensitive prefix check  
- `column.EndsWith('suffix')` - Case-insensitive suffix check
- `column.Length()` - Character count
- `column.IndexOf('substring')` - Find position (0-based, -1 if not found)
- `column.Trim()` - Remove leading/trailing spaces
- `column.TrimStart()` - Remove leading spaces only
- `column.TrimEnd()` - Remove trailing spaces only

### 🎯 **Advanced Query Capabilities**

#### **Complex WHERE Clauses**
```sql
-- Sophisticated filtering with nested logic
SELECT * FROM financial_data
WHERE (category.StartsWith('equity') OR category.Contains('bond'))
  AND price BETWEEN 50 AND 500
  AND quantity NOT IN (0, 1)  
  AND trader_name.Length() > 3
  AND DATEDIFF('day', trade_date, settlement_date) <= 3
  AND commission NOT BETWEEN 0 AND 10
```

#### **Computed Columns & Expressions**
```sql
-- Complex calculations in SELECT
SELECT 
    -- Computed columns with aliases
    price * quantity * (1 - discount/100) as net_amount,
    ROUND((selling_price - cost_basis) / cost_basis * 100, 2) as profit_margin_pct,
    
    -- Nested function calls
    ROUND(SQRT(POWER(leg1, 2) + POWER(leg2, 2)), 3) as hypotenuse,
    
    -- Conditional logic with functions  
    CASE 
        WHEN price.Contains('.') THEN 'Decimal'
        WHEN MOD(ROUND(price, 0), 2) = 0 THEN 'Even'
        ELSE 'Odd'
    END as price_type
FROM trade_data
```

#### **Flexible ORDER BY**
```sql
-- Order by computed expressions and functions
SELECT *, price * quantity as total_value
FROM orders
ORDER BY 
    customer_name.Trim(),                    -- LINQ method in ORDER BY
    ROUND(price * quantity, 2) DESC,         -- Mathematical expression
    DATEDIFF('day', order_date, NOW()) ASC,  -- Date function
    total_value DESC                         -- Computed column alias
LIMIT 100
```

### 🧠 **Smart Type Handling**
- **Automatic Coercion**: String methods work on numbers (`quantity.Contains('5')`)
- **Flexible Parsing**: Multiple date formats automatically recognized
- **NULL Handling**: Graceful handling of missing/empty values
- **Error Recovery**: Helpful suggestions for column name typos

### ⚡ **Performance Features**
- **In-Memory Processing**: 100K+ rows with sub-second response times
- **Smart Caching**: Query results cached for instant re-filtering  
- **Optimized Evaluation**: Efficient column operations and expression parsing
- **Streaming Support**: Large dataset handling without memory bloat

## 🖥️ Vim-Inspired Terminal UI

### **Lightning-Fast Navigation**
- **Vim Keybindings**: `hjkl` movement, `g`/`G` for top/bottom, `/` and `?` for search
- **Column Operations**: Sort (`s`), Pin (`p`), Hide (`H`) columns with single keystrokes  
- **Smart Search**: Column search, data search, fuzzy matching with `n`/`N` navigation
- **Virtual Scrolling**: Handle datasets with 1000+ rows and 190+ columns efficiently
- **Mode Switching**: Insert (`i`), Append (`a`/`A`), Command mode (`Esc`)

### **Power User Features**
- **Key History**: See your last 10 keystrokes with 2s fade
- **Query Caching**: Results cached for instant re-filtering
- **Export**: `Ctrl+S` to save current view as CSV
- **Debug Mode**: `F5` for internal state inspection

## 🚀 **Why Choose SQL CLI?**

### **🔥 Unique Advantages**
| Feature | SQL CLI | csvlens | csvkit | Other Tools |
|---------|---------|---------|---------|-------------|
| **LINQ Methods** | ✅ `.Contains()`, `.StartsWith()` | ❌ | ❌ | ❌ |
| **Date Functions** | ✅ `DATEDIFF`, `DATEADD`, `NOW()` | ❌ | Limited | ❌ |
| **Math Functions** | ✅ `ROUND`, `SQRT`, `POWER`, etc. | ❌ | Basic | ❌ |
| **Vim Navigation** | ✅ Full vim-style | Basic | ❌ | ❌ |
| **Computed Columns** | ✅ `price * qty as total` | ❌ | ❌ | Limited |
| **Smart Completion** | ✅ Context-aware SQL | ❌ | ❌ | ❌ |
| **Multiple Date Formats** | ✅ Auto-detection | ❌ | ❌ | ❌ |

### **🎯 Perfect For**
- **Data Analysts**: Complex calculations with LINQ-style methods
- **Developers**: Vim navigation + SQL power for log analysis  
- **Financial Teams**: Advanced date arithmetic and mathematical functions
- **Anyone**: Who wants `less` for CSV files but with SQL superpowers

## 🔗 **Real-World Examples**

```sql
-- Financial Analysis
SELECT 
    trader.Trim() as trader_name,
    ROUND(SUM(price * quantity), 2) as total_volume,
    COUNT(*) as trade_count,
    ROUND(AVG(price), 4) as avg_price,
    DATEDIFF('day', MIN(trade_date), MAX(trade_date)) as trading_span
FROM trades
WHERE settlement_date > DATEADD('month', -3, TODAY())
  AND counterparty.Contains('BANK')
  AND commission BETWEEN 5 AND 100
  AND NOT status.StartsWith('CANCEL')
GROUP BY trader.Trim()
ORDER BY total_volume DESC
LIMIT 20;

-- Log Analysis  
SELECT 
    log_level,
    message.IndexOf('ERROR') as error_position,
    TEXTJOIN(' | ', 1, timestamp, service, user_id) as context,
    ROUND(response_time_ms / 1000.0, 3) as response_seconds
FROM application_logs
WHERE timestamp > DATEADD('hour', -24, NOW())
  AND (message.Contains('timeout') OR message.Contains('exception'))
  AND response_time_ms BETWEEN 1000 AND 30000
ORDER BY timestamp DESC;
```

## 📦 Installation

### Install with Cargo

```bash
# Install directly from git
cargo install --git https://github.com/YOUR_USERNAME/sql-cli.git

# Or install from crates.io (when published)
cargo install sql-cli
```

### Build from Source

```bash
git clone https://github.com/YOUR_USERNAME/sql-cli.git
cd sql-cli
cargo build --release
./target/release/sql-cli
```

## 🎮 Usage

### Basic Usage
```bash
# Load CSV file
sql-cli data.csv

# Load JSON file  
sql-cli sales.json

# With enhanced mode
sql-cli --enhanced large_dataset.csv
```

### Key Bindings
- **Navigation**: `hjkl` (vim-style), `g`/`G` (top/bottom)
- **Search**: `/` (column search), `?` (data search), `n`/`N` (next/prev)
- **Columns**: `s` (sort), `p` (pin), `H` (hide)  
- **Modes**: `i` (insert), `a`/`A` (append), `Esc` (normal)
- **Export**: `Ctrl+S` (save current view as CSV)

### Advanced SQL Examples

```sql
-- Date functions and complex filtering
SELECT * FROM data 
WHERE created_date > DATEADD(MONTH, -3, NOW()) 
  AND status.Contains('active')
ORDER BY updated_date DESC

-- Aggregations and grouping
SELECT category, COUNT(*) as count, AVG(amount) as avg_amount
FROM sales 
GROUP BY category 
HAVING count > 10

-- String manipulation
SELECT UPPER(name) as name_upper, 
       LEFT(description, 50) as desc_preview
FROM products
WHERE name.StartsWith('A')
```

## 🔧 Development

### Running Tests
```bash
# Run all tests
cargo test

# Run specific test suite
cargo test --test data_view_trades_test
```

### Build Commands
```bash
# Format code (required before commits)
cargo fmt

# Build release
cargo build --release

# Run with file
cargo run data.csv
```

## 🎯 Performance

- **10K-100K rows**: Interactive queries (50-200ms)
- **Complex queries on 100K rows**: ~100-200ms  
- **Memory usage**: ~50MB for 100K rows
- **Navigation**: Zero-latency keyboard response

## 📚 Documentation

Comprehensive documentation available in the `docs/` folder covering:
- Architecture and design decisions
- SQL parser implementation  
- TUI component system
- Performance optimization techniques

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Run `cargo fmt` before committing (required)
4. Submit a pull request

## 📄 License

[MIT License](LICENSE) - see the LICENSE file for details.

---

**Built with Rust 🦀 | Powered by ratatui + crossterm | Inspired by vim**
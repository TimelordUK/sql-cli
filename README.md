# SQL CLI - Powerful CSV/JSON Query Tool with Interactive TUI & CLI Modes

**A vim-inspired SQL query tool for CSV and JSON files. Features both an interactive terminal UI for data exploration and a non-interactive CLI mode for scripting and automation.**

![SQL-CLI Overview](sql-cli/demos/overview.gif)

## 🚀 Why SQL CLI?

**Think `less` for CSV files, but with SQL superpowers:**
- **🎯 Two Modes**: Interactive TUI for exploration, non-interactive for scripting & automation
- **📁 Point & Query**: Drop any CSV/JSON file and immediately start querying  
- **⚡ Lightning Fast**: In-memory engine handles 100K+ rows with sub-second response
- **🎮 Vim-Inspired**: Modal editing, `hjkl` navigation, powerful keyboard shortcuts
- **🧠 Smart Completion**: Context-aware SQL completion with fuzzy matching
- **🔍 Advanced Filtering**: Regex, fuzzy search, complex WHERE clauses
- **📊 Rich SQL Features**: Date functions, string manipulation, mathematical operations
- **📤 Multiple Outputs**: CSV, JSON, TSV, or pretty tables - perfect for pipelines

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

## 🎯 Two Powerful Modes

### 🖥️ **Interactive TUI Mode** (Default)
Launch the full vim-inspired terminal interface for data exploration:

```bash
# Interactive mode - explore your data with vim keybindings
sql-cli data.csv
sql-cli trades.json

# Navigate with hjkl, search with /, execute queries interactively
```

### 🚀 **Non-Interactive Query Mode** (New!)
Execute SQL queries directly from the command line - perfect for scripting and automation:

```bash
# Run a query and get CSV output
sql-cli data.csv -q "SELECT * FROM data WHERE price > 100"

# Output as JSON
sql-cli data.csv -q "SELECT id, name, value FROM data" -o json

# Pretty table format
sql-cli data.csv -q "SELECT * FROM data LIMIT 10" -o table

# Save results to file
sql-cli data.csv -q "SELECT * FROM data WHERE status='active'" -O results.csv

# Execute SQL from a file
sql-cli large_dataset.json -f complex_analysis.sql -o table

# Limit output rows
sql-cli data.csv -q "SELECT * FROM data" -o json -l 100
```

#### **Non-Interactive Options:**
- `-q, --query <SQL>` - Execute SQL query directly
- `-f, --query-file <file>` - Execute SQL from file
- `-o, --output <format>` - Output format: `csv`, `json`, `table`, `tsv` (default: csv)
- `-O, --output-file <file>` - Write results to file
- `-l, --limit <n>` - Limit output to n rows
- `--case-insensitive` - Case-insensitive string matching
- `--auto-hide-empty` - Auto-hide empty columns

#### **Use Cases:**
```bash
# Data pipeline integration
sql-cli raw_data.csv -q "SELECT * FROM raw_data WHERE valid=1" | process_further.sh

# Automated reporting
sql-cli sales.csv -f monthly_report.sql -o json > report_$(date +%Y%m).json

# Quick data analysis
sql-cli logs.csv -q "SELECT COUNT(*) as errors FROM logs WHERE level='ERROR'" -o table

# Data cleaning
sql-cli messy_data.csv -q "SELECT * FROM messy_data WHERE email.EndsWith('.com')" -O clean_data.csv

# Complex calculations
sql-cli finances.csv -q "SELECT date, amount * (1 + tax_rate) as total FROM finances" -o table
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
`ROUND`, `ABS`, `FLOOR`, `CEILING`, `MOD`, `QUOTIENT`, `POWER`, `SQRT`, `EXP`, `LN`, `LOG`, `LOG10`

#### **🧮 Scientific Calculator Mode with DUAL Table**
```sql
-- Use DUAL table for calculations (Oracle-compatible)
SELECT PI() * POWER(5, 2) as circle_area FROM DUAL;
SELECT DEGREES(PI()/2) as right_angle FROM DUAL;

-- Scientific notation support
SELECT 1e-10 * 3.14e5 as tiny_times_huge FROM DUAL;
SELECT 6.022e23 / 1000 as molecules_per_liter FROM DUAL;

-- Physics constants for scientific computing
SELECT 
    C() as speed_of_light,        -- 299792458 m/s
    ME() as electron_mass,        -- 9.109e-31 kg
    PLANCK() as planck_constant,  -- 6.626e-34 J⋅s
    NA() as avogadro_number       -- 6.022e23 mol⁻¹
FROM DUAL;

-- Complex physics calculations
SELECT PLANCK() * C() / 500e-9 as photon_energy_500nm FROM DUAL;
SELECT MP() / ME() as proton_electron_mass_ratio FROM DUAL;

-- No FROM clause needed for simple calculations
SELECT 2 + 2;
SELECT SQRT(2) * PI();
```

**Scientific Constants Available:**
- **Math**: `PI()`, `EULER()`, `TAU()`, `PHI()`, `SQRT2()`, `LN2()`, `LN10()`
- **Physics - Fundamental**: `C()`, `G()`, `PLANCK()`, `HBAR()`, `BOLTZMANN()`, `AVOGADRO()`, `R()`
- **Physics - Electromagnetic**: `E0()`, `MU0()`, `QE()`
- **Physics - Particles**: `ME()`, `MP()`, `MN()`, `AMU()`
- **Physics - Other**: `ALPHA()`, `RYDBERG()`, `SIGMA()`
- **Conversions**: `DEGREES(radians)`, `RADIANS(degrees)`

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
- **Help**: Press `F1` for comprehensive help and keybindings
- **Vim Keybindings**: `hjkl` movement, `g`/`G` for top/bottom, `/` and `?` for search
- **Column Operations**: Sort (`s`), Pin (`p`), Hide (`H`) columns with single keystrokes  
- **Smart Search**: Column search, data search, fuzzy matching with `n`/`N` navigation
- **Virtual Scrolling**: Handle datasets with 1000+ rows and 190+ columns efficiently
- **Mode Switching**: Insert (`i`), Append (`a`/`A`), Command mode (`Esc`)

### **Power User Features**
- **Key History**: See your last 10 keystrokes with 2s fade
- **Query Caching**: Results cached for instant re-filtering
- **Export**: `Ctrl+S` to save current view as CSV
- **Debug View**: Press `F5` to see internal state and diagnostics

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

## 📊 Terminal Charts (NEW!)

SQL CLI now includes a powerful **standalone charting tool** (`sql-cli-chart`) that creates terminal-based visualizations of your SQL query results. Perfect for time series analysis, trend visualization, and data exploration.

### Chart Tool Usage

```bash
# Basic time series chart
sql-cli-chart data.csv -q "SELECT time, value FROM data" -x time -y value -t "My Chart"

# Filter data with SQL WHERE clause
sql-cli-chart trades.csv \
  -q "SELECT timestamp, price FROM trades WHERE symbol = 'AAPL'" \
  -x timestamp -y price -t "AAPL Price Chart"
```

### Real-World Example: VWAP Trading Analysis

Visualize algorithmic trading data with SQL filtering to focus on specific patterns:

```bash
# Chart fill volume progression for CLIENT orders only
sql-cli-chart data/production_vwap_final.csv \
  -q "SELECT snapshot_time, filled_quantity FROM production_vwap_final WHERE order_type LIKE '%CLIENT%'" \
  -x snapshot_time -y filled_quantity \
  -t "CLIENT Order Fill Progression"

# Compare with ALL orders (shows chaotic "Christmas tree" pattern)
sql-cli-chart data/production_vwap_final.csv \
  -q "SELECT snapshot_time, filled_quantity FROM production_vwap_final" \
  -x snapshot_time -y filled_quantity \
  -t "All Orders - Mixed Pattern"
```

**The Power of SQL Filtering**: The first query filters to show only CLIENT orders (991 rows), displaying a clean upward progression. The second shows all 3320 rows including ALGO and SLICE orders, creating a noisy pattern. This demonstrates how SQL queries let you focus on exactly the data patterns you want to visualize.

### Interactive Chart Controls

Once the chart opens, use these vim-like controls:
- **hjkl** - Pan left/down/up/right
- **+/-** - Zoom in/out
- **r** - Reset view to auto-fit
- **q/Esc** - Quit

### Example Scripts

Ready-to-use chart examples are in the `scripts/` directory:

```bash
# VWAP average price over time
./scripts/chart-vwap-price.sh

# Fill volume progression
./scripts/chart-vwap-volume.sh

# Compare different order types
./scripts/chart-vwap-algo-comparison.sh
```

### Chart Features

- **SQL Query Integration**: Use full SQL power to filter and transform data before charting
- **Smart Auto-Scaling**: Automatically adapts Y-axis range for optimal visibility
- **Time Series Support**: Automatic timestamp parsing and time-based X-axis
- **Interactive Navigation**: Pan and zoom to explore your data
- **Terminal Native**: Pure terminal graphics, no GUI dependencies

## 🔄 Unit Conversions

SQL CLI includes a comprehensive unit conversion system accessible through the `CONVERT()` function. Convert between 150+ units across 8 categories, perfect for scientific calculations and data analysis.

### Basic Syntax
```sql
SELECT CONVERT(value, 'from_unit', 'to_unit') FROM DUAL
```

### Supported Categories & Examples

#### **Length Conversions**
```sql
-- Metric to Imperial
SELECT CONVERT(100, 'km', 'miles') as distance FROM DUAL;     -- 62.14 miles
SELECT CONVERT(5.5, 'meters', 'feet') as height FROM DUAL;     -- 18.04 feet
SELECT CONVERT(25, 'cm', 'inches') as width FROM DUAL;         -- 9.84 inches

-- Nautical
SELECT CONVERT(10, 'nautical_mile', 'km') as distance FROM DUAL;  -- 18.52 km
```

#### **Mass/Weight Conversions**
```sql
-- Common conversions
SELECT CONVERT(75, 'kg', 'lb') as weight FROM DUAL;            -- 165.35 pounds
SELECT CONVERT(16, 'oz', 'grams') as weight FROM DUAL;         -- 453.59 grams
SELECT CONVERT(1, 'metric_ton', 'pounds') as heavy FROM DUAL;  -- 2204.62 lbs
```

#### **Temperature Conversions**
```sql
-- Temperature scales
SELECT CONVERT(32, 'F', 'C') as freezing FROM DUAL;            -- 0°C
SELECT CONVERT(100, 'C', 'F') as boiling FROM DUAL;            -- 212°F
SELECT CONVERT(20, 'C', 'K') as room_temp FROM DUAL;           -- 293.15 K
```

#### **Volume Conversions**
```sql
-- Cooking and fuel
SELECT CONVERT(1, 'cup', 'ml') as volume FROM DUAL;            -- 236.59 ml
SELECT CONVERT(3.785, 'L', 'gal') as fuel FROM DUAL;           -- 1 gallon
SELECT CONVERT(750, 'ml', 'fl_oz') as wine FROM DUAL;          -- 25.36 fl oz
```

#### **Time Conversions**
```sql
SELECT CONVERT(1.5, 'hours', 'minutes') as duration FROM DUAL;  -- 90 minutes
SELECT CONVERT(365, 'days', 'years') as age FROM DUAL;         -- 1 year
SELECT CONVERT(5000, 'ms', 'seconds') as delay FROM DUAL;      -- 5 seconds
```

#### **Other Categories**
```sql
-- Area
SELECT CONVERT(100, 'sq_ft', 'm2') as area FROM DUAL;          -- 9.29 m²
SELECT CONVERT(5, 'acres', 'hectares') as land FROM DUAL;      -- 2.02 hectares

-- Speed
SELECT CONVERT(65, 'mph', 'kph') as speed FROM DUAL;           -- 104.61 km/h
SELECT CONVERT(100, 'knots', 'mph') as wind FROM DUAL;         -- 115.08 mph

-- Pressure
SELECT CONVERT(14.7, 'psi', 'bar') as pressure FROM DUAL;      -- 1.01 bar
SELECT CONVERT(1, 'atm', 'Pa') as standard FROM DUAL;          -- 101325 Pa
```

### Complex Calculations with Conversions

```sql
-- Calculate BMI converting from imperial to metric
SELECT 
    CONVERT(180, 'lb', 'kg') as weight_kg,
    CONVERT(72, 'inches', 'm') as height_m,
    CONVERT(180, 'lb', 'kg') / 
    (CONVERT(72, 'inches', 'm') * CONVERT(72, 'inches', 'm')) as BMI
FROM DUAL;

-- Fuel efficiency conversion (mpg to L/100km)
SELECT 
    (CONVERT(100, 'km', 'miles') / 30.0) * CONVERT(1, 'gal', 'L') 
    as liters_per_100km
FROM DUAL;  -- 30 mpg = 7.84 L/100km

-- Physics calculations with proper units
SELECT 
    0.5 * CONVERT(2000, 'lb', 'kg') * 
    POWER(CONVERT(60, 'mph', 'm/s'), 2) as kinetic_energy_joules
FROM DUAL;
```

### Features
- **Case-insensitive**: `'KM'`, `'km'`, `'Km'` all work
- **Unit aliases**: `'kilometer'`, `'kilometers'`, `'km'` are equivalent
- **High precision**: Maintains floating-point precision throughout conversions
- **Bidirectional**: All conversions work in both directions
- **Error handling**: Clear messages for incompatible unit types

### Complete Unit Reference

**Length**: m, meter, km, kilometer, cm, mm, nm, um, mile, yard, foot/feet, inch, nautical_mile

**Mass**: kg, kilogram, g, gram, mg, ug, tonne, metric_ton, lb, pound, oz, ounce, ton, stone

**Temperature**: C, celsius, F, fahrenheit, K, kelvin

**Volume**: L, liter, ml, m3, cm3, cc, gal, gallon, qt, quart, pt, pint, cup, fl_oz, tbsp, tsp

**Time**: s, second, ms, us, ns, minute, hour, day, week, month, year

**Area**: m2, km2, cm2, sq_ft, sq_in, sq_mi, acre, hectare

**Speed**: m/s, kph, mph, knot, fps

**Pressure**: Pa, kPa, MPa, GPa, bar, mbar, atm, psi, torr, mmHg

## 🌌 Astronomical Constants & Calculations

SQL CLI includes comprehensive astronomical constants for solar system calculations and astrophysics:

### **Solar System Constants**
```sql
-- Calculate Earth's surface gravity (should be ~9.82 m/s²)
SELECT G() * MASS_EARTH() / POWER(6.371e6, 2) as earth_gravity FROM DUAL;

-- Compare planetary masses
SELECT 
    MASS_JUPITER() / MASS_EARTH() as jupiter_earth_ratio,  -- ~318x
    MASS_EARTH() / MASS_MOON() as earth_moon_ratio        -- ~81x
FROM DUAL;

-- Orbital distances in AU (Astronomical Units)
SELECT 
    DIST_MARS() / AU() as mars_au,        -- ~1.52 AU
    DIST_JUPITER() / AU() as jupiter_au,  -- ~5.2 AU
    DIST_NEPTUNE() / AU() as neptune_au   -- ~30.1 AU
FROM DUAL;
```

### **Astrophysics Calculations**
```sql
-- Escape velocity from celestial bodies
SELECT 
    SQRT(2 * G() * MASS_EARTH() / 6.371e6) as earth_escape_ms,  -- ~11,200 m/s
    SQRT(2 * G() * MASS_MOON() / 1.737e6) as moon_escape_ms     -- ~2,380 m/s
FROM DUAL;

-- Schwarzschild radius (black hole event horizon)
SELECT 
    2 * G() * MASS_SUN() / (C() * C()) as sun_schwarzschild_m  -- ~2,954 m
FROM DUAL;

-- Kepler's Third Law: Calculate orbital period
SELECT 
    SQRT(4 * PI() * PI() * POWER(DIST_EARTH(), 3) / (G() * MASS_SUN())) 
    / (365.25 * 24 * 3600) as earth_period_years  -- Should be ~1.0
FROM DUAL;
```

### **Combined with Unit Conversions**
```sql
-- Convert astronomical distances to human-scale units
SELECT 
    CONVERT(DIST_EARTH(), 'm', 'miles') as earth_orbit_miles,  -- ~93 million
    CONVERT(LIGHTYEAR(), 'm', 'km') as lightyear_km,          -- ~9.46 trillion
    CONVERT(PARSEC(), 'm', 'lightyear') as parsec_in_ly       -- ~3.26
FROM DUAL;

-- Calculate with mixed units
SELECT 
    G() * MASS_EARTH() / POWER(CONVERT(6371, 'km', 'm'), 2) as g_from_km
FROM DUAL;
```

### **Available Astronomical Constants**

**Particle Radii**:
- `RE()` - Classical electron radius (2.82×10⁻¹⁵ m)
- `RP()` - Proton radius (8.41×10⁻¹⁶ m)
- `RN()` - Neutron radius (8.4×10⁻¹⁶ m)

**Solar System Masses** (kg):
- `MASS_SUN()` - 1.989×10³⁰
- `MASS_EARTH()` - 5.972×10²⁴
- `MASS_MOON()` - 7.342×10²²
- `MASS_MERCURY()`, `MASS_VENUS()`, `MASS_MARS()`, `MASS_JUPITER()`, `MASS_SATURN()`, `MASS_URANUS()`, `MASS_NEPTUNE()`

**Orbital Distances** (meters from Sun):
- `DIST_MERCURY()` through `DIST_NEPTUNE()`
- `AU()` - Astronomical Unit (1.496×10¹¹ m)

**Distance Units**:
- `PARSEC()` - 3.086×10¹⁶ m
- `LIGHTYEAR()` - 9.461×10¹⁵ m

## ⚠️ SQL Features Not Yet Supported

While SQL CLI provides extensive SQL functionality, some standard SQL features are not yet implemented:

### **Aggregate Functions**
- `COUNT(*)`, `COUNT(column)` - Row counting
- `SUM(column)` - Sum of values
- `AVG(column)` - Average calculation
- `MIN(column)`, `MAX(column)` - Min/max values
- `STDDEV()`, `VARIANCE()` - Statistical functions

### **Grouping & Aggregation**
- `GROUP BY` clause - Grouping rows
- `HAVING` clause - Filtering groups
- Aggregate expressions in SELECT

### **Joins & Subqueries**
- `JOIN`, `LEFT JOIN`, `RIGHT JOIN` - Table joins
- `UNION`, `INTERSECT`, `EXCEPT` - Set operations
- Subqueries and correlated queries
- Common Table Expressions (CTEs)

### **Data Modification**
- `INSERT`, `UPDATE`, `DELETE` - Data modification
- `CREATE TABLE`, `ALTER TABLE` - DDL operations

### **Other Features**
- `DISTINCT` keyword - Unique values only
- Window functions (`ROW_NUMBER()`, `RANK()`, etc.)
- `EXISTS`, `ALL`, `ANY` operators

**Note**: SQL CLI is designed for read-only data analysis and exploration. For full SQL database functionality, consider using a traditional database system.

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
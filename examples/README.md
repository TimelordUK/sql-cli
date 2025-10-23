# SQL-CLI Examples

This directory contains example SQL queries showcasing the powerful features of sql-cli.

## Running Examples

You can run any example using the `-f` flag:

```bash
sql-cli -f examples/prime_numbers.sql
sql-cli -f examples/string_functions.sql
```

Or test queries directly with sample data:

```bash
sql-cli data/test_simple_math.csv -f examples/math_functions.sql
```

## Running Specific Statements

You can execute individual statements from a script using `--execute-statement`:

```bash
# Run statement #2 from the script
sql-cli data.csv -f examples/script.sql --execute-statement 2 -o table
```

This automatically handles dependencies (temp tables, CTEs, etc.) and only executes the specified statement.

## Available Examples

### 🎨 Colorful Number Classification (`color_numbers.sql`) ⭐ NEW!

**ANSI Color Functions Demo** - Beautiful visualization of number theory with terminal colors!

**Quick start:**
```bash
# Run all 6 demos
./scripts/demo_colors.sh

# Run specific demo (recommended!)
./scripts/demo_colors.sh 2  # Twin Primes Showcase
./scripts/demo_colors.sh 5  # Number Rainbow 1-100

# Or directly with --execute-statement
./target/release/sql-cli data/numbers_1_100.csv \
  -f examples/color_numbers.sql --execute-statement 2 -o table
```

**Six visual demos:**
1. **Comprehensive Classification** - All special numbers (twin primes, primes, squares) color-coded
2. **Twin Primes Showcase** - Prime pairs (3&5, 11&13) with gap/sum/average stats
3. **Prime Density Visualization** - Bar chart showing primes per decade (1-10, 11-20, etc)
4. **Perfect Squares** - Elegant display with square roots and visual indicators
5. **Number Rainbow (1-100)** - All 100 numbers color-coded by their primary property
6. **Properties Matrix** - Compact multi-property checkmark table

**Color scheme:**
- 🔴 **RED (bold)** - Twin Primes (primes differing by 2)
- 🟡 **GOLD** - Regular Primes
- 🔵 **CYAN** - Perfect Squares (1, 4, 9, 16, 25...)
- ⚫ **GRAY** - Even Numbers
- ⚪ **WHITE** - Odd Composites

**Functions demonstrated:**
- `ANSI_RGB(r, g, b, text)` - 24-bit true color (16 million colors!)
- `ANSI_COLOR(name, text)` - Named colors (red, cyan, green, etc)
- `ANSI_BOLD(text)` - Bold text styling
- `IS_PRIME(n)` - Prime number detection
- Complex CTEs with multiple classification criteria

### 🔢 Prime Number Functions (`prime_numbers.sql`)
- Check if numbers are prime with `IS_PRIME()`
- Get the Nth prime number with `NTH_PRIME()`
- Count primes up to N with `PRIME_PI()`
- Find next/previous primes with `NEXT_PRIME()` and `PREV_PRIME()`

### 🌌 Physics Constants (`physics_constants.sql`)
- Access fundamental constants: speed of light, Planck constant, gravitational constant
- Astronomical masses: Sun, Earth, Moon, planets
- Particle masses: electron, proton, neutron
- Practical physics calculations using these constants

### 🧪 Chemical Formulas (`chemical_formulas.sql`)
- Parse chemical formulas with `ATOMIC_MASS()`
- Get formulas from common names with `MOLECULE_FORMULA()`
- Calculate molecular weights
- Analyze compound properties

### 📝 String Functions (`string_functions.sql`)
- Text manipulation: UPPER, LOWER, TRIM, REVERSE
- String extraction: SUBSTRING, LEFT, RIGHT, SPLIT_PART
- Pattern matching: RLIKE, REGEXP_REPLACE, REGEXP_EXTRACT
- Hashing: MD5, SHA1, SHA256
- Advanced formatting and padding

### 📅 Date/Time Functions (`date_time_functions.sql`)
- Current date/time: NOW(), TODAY()
- Date arithmetic: DATEADD(), DATEDIFF()
- Date extraction: EXTRACT() for year, month, day, etc.
- Date formatting and parsing
- Support for both US and European date formats

### 📊 GROUP BY and Aggregates (`group_by_aggregates.sql`)
- Aggregation functions: COUNT, SUM, AVG, MIN, MAX, STDDEV
- Multi-column GROUP BY
- HAVING clause for post-aggregation filtering
- Complex aggregations with CASE statements
- Time-based grouping and analysis

### 🧮 Math Functions (`math_functions.sql`)
- Arithmetic operations and functions
- Trigonometric functions: SIN, COS, TAN, etc.
- Logarithmic and exponential functions
- Statistical functions
- Bitwise operations
- Financial calculations

### 🏷️ Utility Examples
- `least_label.sql` - Find the smallest labeled value using LEAST_LABEL()
- `case_test_mass_fns.sql` - CASE expressions with physics constants

## Quick Feature Showcase

```sql
-- Combine multiple advanced features
SELECT 
    trader_name,
    COUNT(*) as trade_count,
    SUM(quantity) as total_volume,
    AVG(price) as avg_price,
    ATOMIC_MASS('C8H10N4O2') as caffeine_mass,  -- Chemical
    IS_PRIME(COUNT(*)) as is_prime_count,              -- Prime check
    DATEDIFF('day', MIN(trade_date), NOW()) as days_trading,  -- Date math
    MD5(trader_name) as trader_hash                    -- Hashing
FROM trades
WHERE trade_date >= DATEADD('month', -3, TODAY())
GROUP BY trader_name
HAVING COUNT(*) > 10
ORDER BY total_volume DESC;
```

## Advanced Capabilities

sql-cli is not just a SQL query tool - it's a powerful data analysis platform with:

- **Scientific Computing**: Built-in physics constants and prime number theory
- **Chemistry Support**: Parse and analyze chemical formulas
- **Advanced Math**: From basic arithmetic to complex statistical functions
- **Date Intelligence**: Comprehensive date/time manipulation
- **Text Processing**: Regex, hashing, and advanced string operations
- **GROUP BY with HAVING**: Full aggregation support with post-filtering

## Tips

1. **Non-interactive testing**: Test queries quickly without entering TUI:
   ```bash
   sql-cli data.csv -q "SELECT * FROM data WHERE IS_PRIME(id)" -o csv
   ```

2. **Query plan inspection**: Debug complex queries:
   ```bash
   sql-cli data.csv -q "YOUR_QUERY" --query-plan
   ```

3. **Combine features**: Mix different function types for powerful analysis:
   ```sql
   SELECT 
       UPPER(name) as name,
       ATOMIC_MASS(compound) as mass,
       IS_PRIME(EXTRACT(DAY FROM date)) as prime_day
   FROM experiments;
   ```

## Contributing

Feel free to add more examples showcasing sql-cli's capabilities!
kkk:q!:q!# SQL CLI - Stage 2 Roadmap

## Current State (Stage 1 Complete ✅)

We have successfully built the foundational architecture:
- **Core SQL Engine**: Recursive descent parser with AST
- **Function Registry**: Extensible function system with 100+ functions
- **Data Sources**: CSV, JSON, WEB CTEs
- **TUI**: Vim-like interface with advanced navigation
- **Neovim Integration**: Full plugin with function help, formatting
- **Professional Output**: Formatting functions for reports
- **Window Frame Support**: Full ROWS/RANGE PRECEDING/FOLLOWING with BETWEEN syntax
- **Window Aggregates**: MIN/MAX/AVG/SUM/COUNT with frame awareness

## Stage 2: Data Analysis Power Tools

### 🎯 Priority 1: Statistical & Financial Functions

#### Basic Statistics
- [x] `STDDEV(column)` - Standard deviation (as aggregate and window function) ✅
- [x] `VARIANCE(column)` - Variance (as aggregate and window function) ✅
- [x] `MEDIAN(column)` - Median value ✅
- [x] `PERCENTILE(column)` - 50th percentile (median equivalent) ✅
- [x] `MODE(column)` - Most frequent value (perfect for distribution analysis) ✅
- [ ] `CORR(col1, col2)` - Correlation coefficient
- [ ] `COVAR(col1, col2)` - Covariance

#### Financial/Trading Analytics
- [x] `RETURNS(current, previous)` - Calculate simple returns ✅
- [x] `LOG_RETURNS(current, previous)` - Logarithmic returns ✅
- [x] `VOLATILITY(...)` - Volatility calculation ✅
- [x] `SHARPE_RATIO(mean, rf, vol)` - Sharpe ratio ✅
- [ ] `BOLLINGER_UPPER(price, window, num_std)` - Upper Bollinger Band
- [ ] `BOLLINGER_LOWER(price, window, num_std)` - Lower Bollinger Band
- [ ] `EMA(column, window)` - Exponential Moving Average
- [ ] `RSI(price, period)` - Relative Strength Index
- [ ] `BETA(returns, market_returns)` - Beta coefficient

Example usage:
```sql
WITH price_data AS (
    SELECT date, close_price,
           NORM_RETURNS(close_price) OVER (ORDER BY date) as norm_return,
           VOLATILITY(close_price, 20) OVER (ORDER BY date) as vol_20d,
           BOLLINGER_UPPER(close_price, 20, 2) OVER (ORDER BY date) as bb_upper,
           BOLLINGER_LOWER(close_price, 20, 2) OVER (ORDER BY date) as bb_lower
    FROM stock_prices
)
SELECT * FROM price_data WHERE close_price > bb_upper;  -- Breakout detection
```

### 🎯 Priority 2: Window Function Registry & Syntactic Sugar

#### Core Window Functions (Completed ✅)
- [x] `LAG(column, n)` - Access previous row's value ✅
- [x] `LEAD(column, n)` - Access next row's value ✅
- [x] `FIRST_VALUE(column)` - First value in window ✅
- [x] `LAST_VALUE(column)` - Last value in window ✅
- [x] `ROW_NUMBER()` - Row number in partition ✅
- [x] Window frames: `ROWS n PRECEDING/FOLLOWING` ✅
- [x] `BETWEEN...AND` syntax ✅
- [x] `UNBOUNDED PRECEDING/FOLLOWING` ✅

#### Planned Window Functions
- [ ] `RANK()` - Ranking with gaps
- [ ] `DENSE_RANK()` - Ranking without gaps
- [ ] `NTH_VALUE(column, n)` - Nth value in window
- [ ] `PERCENT_RANK()` - Percentile ranking
- [ ] `CUME_DIST()` - Cumulative distribution
- [ ] `NTILE(n)` - Divide into n buckets

#### Window Function Registry Architecture (Completed ✅)
- [x] **Decouple from engine**: Move all window logic to specialized registry ✅
- [x] **Syntactic sugar functions**: Simplify complex patterns ✅
  - `MOVING_AVG(col, 20)` → `AVG(col) OVER (ORDER BY date ROWS 19 PRECEDING)` ✅
  - `CUMULATIVE_SUM(col)` → `SUM(col) OVER (ORDER BY date ROWS UNBOUNDED PRECEDING)` ✅
  - `ROLLING_STDDEV(col, 30)` → `STDDEV(col) OVER (ORDER BY date ROWS 29 PRECEDING)` ✅
  - `BOLLINGER_UPPER/LOWER(col, window, std)` - Bollinger Bands ✅
  - `Z_SCORE(col, window)` - Statistical Z-score ✅
  - `PERCENT_CHANGE(col, periods)` - Period-over-period changes ✅
- [ ] **GPU offloading**: Parallel computation for large windows
- [ ] **Multi-threading**: Process partitions in parallel
- [ ] **Lazy evaluation**: Compute only requested rows

#### Aggregate Function Registry Architecture (Completed ✅)
- [x] **Registry Pattern**: Created AggregateFunction and AggregateState traits ✅
- [x] **Basic Aggregates**: COUNT, SUM, AVG, MIN, MAX moved to registry ✅
- [x] **STRING_AGG**: Implemented with configurable separator ✅
- [x] **Full Integration**: Replace hardcoded aggregate logic in evaluator ✅
- [x] **DISTINCT Support**: Uniform DISTINCT handling across all aggregates ✅
- [x] **Additional Aggregates**: MEDIAN ✅, STDDEV ✅, VARIANCE ✅, PERCENTILE ✅, MODE ✅
- [ ] **Remaining Aggregates**: CORR, COVAR (multi-column functions)

### 🎯 Priority 3: Better Error Messages

- [ ] Show available columns when "Column not found"
- [ ] Suggest similar function names when function not found
- [ ] Show data types in type mismatch errors
- [ ] Add line/column numbers to parse errors
- [ ] "Did you mean?" suggestions for typos

### 📊 ASCII Visualizations (via Neovim/Lua)

Since Lua is more flexible for this, implement in nvim plugin:
- [ ] Scatter plots - `:SqlScatter x_col y_col`
- [ ] Histograms - `:SqlHist column bins`
- [ ] Line charts - `:SqlLine x_col y_col`
- [ ] Bar charts - `:SqlBar category value`
- [ ] Box plots - `:SqlBox column`
- [ ] Pie charts - `:SqlPie category value`
- [ ] Correlation matrix heatmap - `:SqlCorr`

### 🚀 Performance Optimizations

- [ ] Query result caching for CTEs
- [ ] Parallel execution for independent CTEs
- [ ] Columnar storage for better cache locality
- [ ] Index creation for large CSV files
- [ ] Query optimizer for JOIN ordering

### 🔧 SQL Feature Completeness

#### Easier Syntax
- [ ] Simple CASE: `CASE value WHEN 1 THEN... WHEN 2 THEN...`
- [ ] IN operator: `WHERE x IN (1,2,3)`
- [ ] BETWEEN: `WHERE x BETWEEN 1 AND 10`
- [ ] UNION ALL support with schema coercion

#### Advanced Features
- [ ] CREATE TEMP TABLE for multi-step analysis
- [ ] Variables: `SET @start_date = '2024-01-01'`
- [ ] Stored procedures/functions in Lua
- [ ] PIVOT/UNPIVOT operations

### 📁 Data Source Expansion

- [ ] **Parquet files** - Common in data engineering
- [ ] **Direct database connections** - PostgreSQL, SQLite, DuckDB
- [ ] **Excel files** - Business reality
- [ ] **S3/Cloud storage** - For larger datasets
- [ ] **Arrow flight** - For distributed queries

### 💡 Quality of Life Improvements

- [ ] Query history with search (`:SqlHistory`)
- [ ] Save/load named queries (`:SqlSave trader_analytics`)
- [ ] Auto-completion for table/column names
- [ ] Column statistics preview (`:SqlDescribe table`)
- [ ] Export with custom formatting templates
- [ ] Batch processing mode for multiple files

### 🎨 Output Formatting

- [ ] Custom number formats (e.g., basis points, percentages)
- [ ] Conditional formatting (highlight outliers)
- [ ] Report templates (headers, footers, pagination)
- [ ] Export to Markdown tables
- [ ] Export to LaTeX tables

## Implementation Strategy

1. **Phase 1**: Statistical functions (high impact, fits existing architecture)
2. **Phase 2**: Financial/trading functions (builds on statistics)
3. **Phase 3**: ASCII visualizations in nvim (separate from core)
4. **Phase 4**: Performance optimizations (after features stable)
5. **Phase 5**: Additional data sources (as needed)

## Design Principles

- **Maintain extensibility**: All new functions through registry
- **Window Function Registry**: Separate registry for window-specific logic
- **Syntactic Sugar**: Make complex patterns accessible via helper functions
- **Keep core simple**: Complex features via plugins/extensions
- **Prioritize composability**: Functions should work together
- **Performance at Scale**: GPU/multi-threading for heavy computations
- **Fast iteration**: Use Lua for prototyping when possible
- **User-driven**: Build what traders/analysts actually need

## Success Metrics

- Can perform complete statistical analysis without leaving the tool
- Can analyze trading strategies with built-in functions
- Can visualize results directly in nvim
- Performance competitive with specialized tools for <1GB datasets
- Community contributions to function library

## Notes

The key insight from Stage 1 is that the function registry architecture allows unlimited expansion without touching core code. This means we can add domain-specific functions (trading, statistics, ML) as needed while maintaining stability.

The TUI is "good enough" - future UI work should focus on the nvim plugin where Lua provides more flexibility.
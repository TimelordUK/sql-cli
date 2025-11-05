# SQL CLI v1.67.0

**Release Date:** November 05, 2025

## 🚀 Window Function Performance Revolution - 86% Faster!

This release brings dramatic performance improvements to window functions through our new batch evaluation engine. Complex analytical queries now run up to **86% faster** on large datasets!

## 📊 Release Overview
- **Major Performance Boost**: 86% improvement for window functions
- **New Functions**: RANK and DENSE_RANK window functions
- **Smart Optimization**: Automatic O(n) algorithms for cumulative patterns
- **Zero Configuration**: All improvements enabled by default

## ✨ Highlights

### 🎯 Blazing Fast Window Functions
- **50,000 rows with LAG**: 2.24s → **350ms** (86% improvement!)
- **Cumulative SUM**: Previously timing out → Now **338ms**
- All window functions now use optimized batch evaluation by default

### 📈 What's Optimized
- ✅ Window aggregates: `SUM`, `AVG`, `MIN`, `MAX`, `COUNT`, `FIRST_VALUE`, `LAST_VALUE`
- ✅ Positional functions: `LAG`, `LEAD`, `ROW_NUMBER`
- ✅ Ranking functions: `RANK`, `DENSE_RANK` (newly implemented!)
- ✅ Smart O(n) algorithms for cumulative patterns

## 📝 Example Usage

```sql
-- This query is now 86% faster!
SELECT 
    sale_date,
    amount,
    SUM(amount) OVER (ORDER BY sale_date ROWS UNBOUNDED PRECEDING) as running_total,
    AVG(amount) OVER (ORDER BY sale_date ROWS 30 PRECEDING) as moving_avg_30,
    LAG(amount, 1) OVER (ORDER BY sale_date) as prev_amount,
    RANK() OVER (ORDER BY amount DESC) as sales_rank
FROM sales
ORDER BY sale_date;
```

## 🔧 Technical Details

### Performance Improvements
- **Batch Evaluation**: Process all rows in a single pass instead of per-row evaluation
- **Hash-Based Caching**: Pre-create and cache WindowContext objects (50,000 lookups → 1)
- **Running Aggregates**: O(n) incremental calculation for UNBOUNDED PRECEDING frames
- **Smart Detection**: Automatically optimizes cumulative and running total patterns

### Configuration
- Batch evaluation is **enabled by default** - no configuration needed!
- To opt-out (not recommended): Set `SQL_CLI_BATCH_WINDOW=0`
- Complex expressions automatically use the optimal evaluation strategy

## 🐛 Bug Fixes
- Fixed window aggregate functions (AVG) returning incorrect results in certain cases
- Fixed Python test suite warnings about test functions returning values
- Fixed expression evaluation for window functions embedded in calculations

## 📋 Full Commit List

<details>
<summary>View all commits</summary>

- feat: Make batch window evaluation the default and fix expression handling
- fix: Update Python tests to use assertions instead of return values
- feat: Optimize window aggregates for UNBOUNDED PRECEDING frames
- feat: Implement batch evaluation for window functions - 86% performance improvement!
- feat: Window function optimization Phase 2 + Step 0 prep for batch evaluation
- feat: Add Phase 1 window function profiling infrastructure

</details>

## 🎯 Key Features

- **Window Functions**: Now with 86% better performance!
- **Instant Data Preview**: CSV/JSON files load immediately
- **Advanced SQL**: CTEs, subqueries, window functions, aggregates
- **Powerful Search**: Regular search (Ctrl+F), fuzzy filter (Ctrl+/)
- **Data Export**: Save as CSV or JSON
- **Vim Navigation**: Full vim-style key bindings

## 📦 Installation

```bash
cargo install sql-cli
```

Or download the binary for your platform from the assets below.

## 📈 Upgrade Recommendation

**Highly recommended upgrade** for anyone using window functions, especially on large datasets. The performance improvements are substantial and all existing queries will automatically benefit.

---

**Thank you for using SQL CLI!** 🎉

Report issues: [GitHub Issues](https://github.com/TimelordUK/sql-cli/issues)
Full changelog: [CHANGELOG.md](https://github.com/TimelordUK/sql-cli/blob/main/CHANGELOG.md)
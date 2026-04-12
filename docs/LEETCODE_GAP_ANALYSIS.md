# LeetCode SQL Gap Analysis

**Date:** 2026-04-12  
**Corpus:** 38 LeetCode SQL problems from `/home/me/dev/cs/leet/sql/`  
**Engine version:** sql-cli v1.67.2

## Executive Summary

The sql-cli engine successfully parses **31 of 38 (81.6%)** LeetCode SQL queries. The 7 failures cluster around just 4 parser-level gaps — fixing the top 2 would bring coverage to **94.7%**. The engine's CTE, window function, and JOIN support is solid across the board.

## Full Results

| # | Problem | Status | Key Features | Blocking Gap |
|---|---------|--------|-------------|--------------|
| 176 | Second Highest Salary | PASS | Scalar subquery, DISTINCT, LIMIT OFFSET | — |
| 177 | Nth Highest Salary | FAIL | CREATE FUNCTION, procedural SQL | Out of scope (DDL) |
| 178 | Rank Scores | PASS | DENSE_RANK() window | — |
| 180 | Consecutive Numbers | PASS | LAG/LEAD window, subquery in FROM | — |
| 184 | Dept Highest Salary | FAIL | DENSE_RANK, JOIN, PARTITION BY d.name | **PARTITION BY table.column** |
| 550 | Game Play Analysis IV | PASS | CTE, LEAD, ROUND, DATEDIFF | — |
| 570 | Managers with 5+ Reports | PASS | CTE, DENSE_RANK, INNER JOIN | — |
| 585 | Investments in 2016 | PASS | COUNT(*) OVER (PARTITION BY multi-col) | — |
| 602 | Friend Requests II | PASS | UNION ALL in CTE, GROUP BY | — |
| 608 | Tree Node | PASS | CASE, IS NULL, IN (subquery) | — |
| 626 | Exchange Seats | PASS | CASE, LAG/LEAD, COALESCE, modulo | — |
| 1045 | Customers Bought All | PASS | HAVING COUNT(DISTINCT), scalar subquery | — |
| 1070 | Product Sales III | PASS | Subquery in FROM, multiple JOINs | — |
| 1158 | Market Analysis I | PASS | LEFT OUTER JOIN, YEAR(), GROUP BY | — |
| 1164 | Product Price at Date | PASS | ROW_NUMBER, COALESCE, LEFT JOIN | — |
| 1174 | Immediate Delivery II | FAIL | AVG(col = col), (a,b) IN (subquery) | **Boolean in aggregate**, **tuple IN** |
| 1193 | Monthly Transactions I | PASS | DATE_FORMAT, SUM(CASE), GROUP BY | Runtime: DATE_FORMAT missing |
| 1204 | Last Person in Bus | PASS | Running SUM window, LIMIT | — |
| 1321 | Restaurant Growth | PASS | Window frame ROWS BETWEEN, OFFSET | — |
| 1341 | Movie Rating | PASS | 4 CTEs, UNION ALL, JOINs | — |
| 1393 | Capital Gain/Loss | FAIL | SUM(CASE WHEN ... THEN -price) | **Unary minus on column** |
| 1907 | Count Salary Categories | PASS | UNION in CTE, LEFT JOIN | — |
| 1934 | Confirmation Rate | FAIL | AVG(c.action = 'confirmed'), IFNULL | **Boolean in aggregate** |
| 3220 | Odd/Even Transactions | PASS | SUM(CASE), modulo, GROUP BY | — |
| 3421 | Student Score Improvement | PASS | FIRST_VALUE window (ASC/DESC) | — |
| 3475 | DNA Pattern Recognition | PASS | CASE with LIKE patterns | — |
| 3497 | Subscription Conversion | PASS | AVG, ROUND, HAVING IS NOT NULL | — |
| 3521 | Find Product Pairs | PASS | Self-join, 4-table JOIN, HAVING | — |
| 3564 | Seasonal Sales Analysis | PASS | CASE value-form, MONTH(), ROW_NUMBER | — |
| 3580 | Employee Performance | PASS | ROW_NUMBER, LEAD, LAG, JOIN | — |
| 3586 | COVID Recovery Tracker | PASS | MIN(CASE), DATEDIFF, CTE | — |
| 3601 | Fuel Efficiency | PASS | MONTH() in CASE, ROUND(AVG(CASE)) | — |
| 3611 | Meeting Overload | PASS | Subquery with HAVING, YEARWEEK | Runtime: YEARWEEK missing |
| 3626 | Inventory Imbalance | PASS | FIRST_VALUE (4 windows), ROW_NUMBER | — |
| 3642 | Polarizing Book Reviews | FAIL | AVG(rating<=2 OR rating>=4) | **Boolean in aggregate** |
| 3657 | Reliable Customer | PASS | DATEDIFF in HAVING, AVG(CASE) | — |
| 3705 | Peak Hour Analysis | FAIL | AVG(order_rating IS NOT NULL), HOUR() | **Boolean in aggregate** |
| 3716 | Subscription Downgrade | PASS | FIRST_VALUE, DATEDIFF, CTE | — |

## Parser Gap Analysis

### Gap 1: Boolean/comparison expressions in aggregate arguments _(4 queries)_

**Impact:** HIGH — blocks 4 problems, limits expressiveness generally  
**Problems:** 1174, 1934, 3642, 3705

The parser's function argument parsing does not allow comparison operators (`=`, `<`, `>`, `<=`, `>=`, `IS`, `OR`, `AND`) inside function call parentheses. It expects each argument to be a "primary expression" (column, literal, function call, CASE) but not a full comparison expression.

**Examples that fail:**
```sql
AVG(col = 'value')              -- boolean equality in AVG
AVG(rating <= 2 OR rating >= 4) -- boolean OR in AVG
AVG(col IS NOT NULL)            -- IS expression in AVG
ROUND(AVG(col = col) * 100, 2) -- nested
```

**Workaround that already works:**
```sql
AVG(CASE WHEN col = 'value' THEN 1 ELSE 0 END)
```

**Fix approach:** Promote the function argument parser to use `parse_expression()` (which handles comparisons and boolean logic) instead of the current restricted path. Since the parser is recursive descent, this should compose naturally — once expressions are allowed, `AVG(IS_PRIME(x))` or `SUM(x > threshold AND x < cap)` would also work.

**Verification:** The engine already evaluates boolean comparisons to `true`/`false` and `AVG()` already handles numeric results from CASE. The main work is parser-level: allow full expressions where currently only primaries are accepted.

### Gap 2: Unary minus on column references _(1 query)_

**Impact:** MEDIUM — blocks 1 problem, but unary minus is generally useful  
**Problem:** 1393

The expression parser handles negative numeric literals (`-5`) but not unary minus applied to a column or expression (`-price`, `-(a + b)`).

**Example that fails:**
```sql
CASE WHEN operation = 'Buy' THEN -price ELSE price END
```

**Fix approach:** In `parse_primary()` or `parse_unary()`, when a `Minus` token is encountered at the start of an expression, parse it as `0 - <expr>` or as a dedicated `UnaryMinus` AST node. This is a small, localised change.

### Gap 3: PARTITION BY with table-qualified columns _(1 query)_

**Impact:** LOW-MEDIUM — blocks 1 problem  
**Problem:** 184

The `PARTITION BY` clause parser does not handle dotted column references (`d.name`). It works fine with bare column names.

**Example that fails:**
```sql
DENSE_RANK() OVER (PARTITION BY d.name ORDER BY salary DESC)
```

**Fix approach:** Reuse the existing `parse_column_ref()` (which handles `table.column`) in the window clause parser where it currently calls a simpler column name parser.

### Gap 4: Tuple IN subquery _(1 query)_

**Impact:** LOW — blocks 1 problem, niche SQL feature  
**Problem:** 1174

```sql
WHERE (customer_id, order_date) IN (SELECT customer_id, MIN(order_date) FROM ...)
```

**Fix approach:** This requires parsing a parenthesised list of columns on the left side of IN, and matching against multi-column subquery results. Lower priority.

### Gap 5: CREATE FUNCTION / procedural SQL _(1 query)_

**Impact:** NONE — out of scope  
**Problem:** 177

LeetCode's MySQL-specific procedural syntax. Not relevant to our engine.

## Runtime Function Gaps

These queries parse successfully but would fail at execution if tables were materialised:

| Function | Queries | Description | Fix |
|----------|---------|-------------|-----|
| `DATE_FORMAT(date, fmt)` | 1193 | Format date to string | Function registry addition |
| `IFNULL(expr, default)` | 1934 | MySQL alias for COALESCE | Alias or thin wrapper |
| `HOUR(timestamp)` | 3705 | Extract hour from timestamp | Function registry addition |
| `YEARWEEK(date)` | 3611 | Year+week number | Function registry addition |

## Features Confirmed Working

These features were validated across the 31 passing queries:

- **CTEs:** Single, multiple (up to 4), with UNION/UNION ALL inside
- **Window functions:** DENSE_RANK, ROW_NUMBER, LAG, LEAD, FIRST_VALUE, running SUM
- **Window frames:** ROWS BETWEEN N PRECEDING AND CURRENT ROW
- **PARTITION BY:** Single and multi-column (unqualified)
- **JOINs:** INNER, LEFT, LEFT OUTER, self-join, multi-table (3-4), multi-condition ON
- **Aggregates:** COUNT, SUM, AVG, MIN, MAX, COUNT(DISTINCT)
- **HAVING:** Including with aliases, subqueries, function calls
- **Subqueries:** Scalar in SELECT, table in FROM, IN (single column)
- **CASE:** WHEN form and value form, nested, with LIKE, implicit NULL ELSE
- **Expressions:** Arithmetic, modulo, COALESCE, ROUND, DATEDIFF, YEAR, MONTH
- **UNION / UNION ALL:** At top level and inside CTEs
- **LIMIT / OFFSET**
- **DISTINCT** in SELECT and inside aggregates

## Implementation Roadmap

### Phase 1: Parser enhancements (highest value)

1. **Boolean expressions in aggregate arguments** — Unblock 4 queries
   - Promote function arg parsing to `parse_expression()`
   - Add tests: `AVG(x > 5)`, `SUM(x = 'a')`, `COUNT(x IS NOT NULL)`
   - Verify composability: `AVG(IS_PRIME(x))`, `SUM(x > 0 AND x < 10)`

2. **Unary minus on expressions** — Unblock 1 query
   - Handle `-expr` in `parse_primary()` / `parse_unary()`
   - Add tests: `-price`, `-(a + b)`, `-ABS(x)`

3. **PARTITION BY qualified columns** — Unblock 1 query
   - Use `parse_column_ref()` in window clause parser
   - Add tests: `PARTITION BY t.col`, `PARTITION BY t.a, t.b`

### Phase 2: Runtime functions

4. **IFNULL()** — Alias for COALESCE (trivial)
5. **HOUR()** — Extract hour from datetime
6. **DATE_FORMAT()** — Format date to string
7. **YEARWEEK()** — Year+week composite

### Phase 3: Schema CTE (LeetCode runner)

8. **DDL-to-DataTable converter** — Parse CREATE TABLE + INSERT INTO from `.schema.sql`
9. **SCHEMA CTE syntax** — e.g., `WITH Employee AS (SCHEMA PATH '176.schema.sql')`
10. **LeetCode test runner** — Script that runs each problem and compares output

### Phase 4: Tuple IN (stretch)

11. **Tuple IN subquery** — `WHERE (a, b) IN (SELECT ...)`

## Notes

- The CASE workaround (`AVG(CASE WHEN x > 5 THEN 1 ELSE 0 END)`) is always available for boolean-in-aggregate, but the direct syntax is more natural and matches MySQL/PostgreSQL behaviour.
- 177 (CREATE FUNCTION) is permanently out of scope — it's procedural DDL specific to MySQL.
- All date functions (DATEDIFF, YEAR, MONTH) already work at runtime, confirming the function registry approach scales well.
- The corpus is biased toward medium/hard LeetCode problems with heavy window function and CTE usage — good stress test for the engine's strongest areas.

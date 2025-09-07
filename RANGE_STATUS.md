# RANGE Function Status Report

## ✅ Working Features

### Basic RANGE Operations
- `SELECT value FROM RANGE(1, 10)` - Basic range generation ✓
- `SELECT value FROM RANGE(0, 20, 5)` - Range with step parameter ✓
- `SELECT value FROM RANGE(-5, -1)` - Negative ranges ✓
- `SELECT value FROM RANGE(5, 5)` - Single value range ✓

### RANGE with CTEs
- Basic CTE with RANGE:
  ```sql
  WITH numbers AS (SELECT value FROM RANGE(1, 5))
  SELECT value, value * 2 AS doubled FROM numbers
  ```
  ✓ Works perfectly

- Nested CTEs with RANGE:
  ```sql
  WITH first_range AS (SELECT value AS n FROM RANGE(1, 3)),
       squared AS (SELECT n, n * n AS sq FROM first_range)
  SELECT n, sq, sq * 2 AS doubled_square FROM squared
  ```
  ✓ Works perfectly

### RANGE with Prime Functions
- `SELECT value FROM RANGE(2, 20) WHERE IS_PRIME(value) = true` ✓
- `SELECT COUNT(*) FROM RANGE(2, 100) WHERE IS_PRIME(value) = true` ✓
- Prime density analysis with CTEs:
  ```sql
  WITH blocks AS (SELECT value AS block_num FROM RANGE(1, 5))
  SELECT block_num, 
         PRIME_PI(block_num * 100) - PRIME_PI((block_num - 1) * 100) AS primes_in_block
  FROM blocks
  ```
  ✓ Works perfectly

### RANGE with Calculations
- `SELECT value, value * value AS squared FROM RANGE(1, 5)` ✓
- `SELECT value, SQRT(value) AS square_root FROM RANGE(1, 5)` ✓
- `SELECT value, value * value * value AS cubed FROM RANGE(1, 5)` ✓

### RANGE with Aggregates in CTEs
```sql
WITH prime_analysis AS (
    SELECT value, IS_PRIME(value) AS is_prime FROM RANGE(1, 100)
)
SELECT COUNT(*) AS total_numbers,
       SUM(CASE WHEN is_prime = true THEN 1 ELSE 0 END) AS prime_count
FROM prime_analysis
```
✓ Works perfectly

### RANGE with ORDER BY
```sql
SELECT value, value * value AS squared 
FROM RANGE(1, 5) 
ORDER BY squared DESC
```
✓ Works perfectly

## ❌ Known Limitations / Issues

### 1. Column Reference Issues Outside Basic SELECT
- **Problem**: `value` column not accessible in WHERE clause directly on RANGE
  ```sql
  SELECT value FROM RANGE(1, 20) WHERE value % 3 = 0  -- Doesn't work
  ```
  
### 2. Cross Joins Between Multiple RANGE CTEs
- **Problem**: Column resolution issues with multiple CTEs
  ```sql
  WITH small_range AS (SELECT value AS small FROM RANGE(1, 3)),
       large_range AS (SELECT value AS large FROM RANGE(10, 12))
  SELECT small, large FROM small_range, large_range  -- Column 'large' not found
  ```

### 3. GROUP BY on RANGE Values in CTEs
- **Problem**: Column 'value' not found when using GROUP BY
  ```sql
  WITH numbers AS (SELECT value FROM RANGE(1, 20))
  SELECT value % 5 AS remainder, COUNT(*) FROM numbers GROUP BY value % 5
  ```

### 4. Window Functions with RANGE
- **Problem**: Requires data file even when using RANGE as virtual table
  ```sql
  WITH data AS (SELECT value FROM RANGE(1, 12))
  SELECT value, ROW_NUMBER() OVER (ORDER BY value) FROM data
  ```
  Error: "Data file (CSV or JSON) required for non-interactive query mode"

## 📊 Test Results Summary

**Passed Tests (12/17):**
- ✓ Basic RANGE operations
- ✓ RANGE with step parameter
- ✓ RANGE in simple CTEs
- ✓ Nested CTEs with RANGE
- ✓ RANGE with IS_PRIME filtering
- ✓ Prime counting in ranges
- ✓ PRIME_PI with RANGE
- ✓ Prime density block calculations
- ✓ Complex CTEs with aggregates
- ✓ RANGE with calculations (SQRT, power operations)
- ✓ RANGE edge cases (single value, negative ranges)
- ✓ RANGE with ORDER BY

**Failed Tests (5/17):**
- ✗ WHERE clause directly on RANGE
- ✗ Multiple RANGE cross joins
- ✗ GROUP BY with RANGE values
- ✗ Window functions with RANGE
- ✗ PARTITION BY with RANGE-based CTEs

## 🎯 Recommendations

1. **Use CTEs for Complex Operations**: Wrap RANGE in CTEs for better compatibility
2. **Prime Analysis Works Well**: The combination of RANGE + IS_PRIME + PRIME_PI is fully functional
3. **Stick to Simple Patterns**: Basic SELECT, calculations, and single CTEs work reliably
4. **Avoid**: Direct WHERE on RANGE, complex joins, window functions with RANGE

## Example: Working CTE with RANGE and PARTITION BY Alternative

Since direct PARTITION BY doesn't work, use this pattern:
```sql
-- Instead of PARTITION BY, use grouped CTEs:
WITH base AS (SELECT value FROM RANGE(1, 20)),
     categorized AS (
         SELECT value,
                CASE WHEN IS_PRIME(value) = true THEN 'prime'
                     WHEN value % 2 = 0 THEN 'even'
                     ELSE 'odd' END AS category
         FROM base
     ),
     counts AS (
         SELECT category, COUNT(*) as cnt 
         FROM categorized 
         GROUP BY category
     )
SELECT * FROM counts;
```

This achieves similar results without needing window functions.
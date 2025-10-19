-- HAVING Clause - Supported Patterns
--
-- IMPORTANT: HAVING works with column ALIASES, not direct aggregate function calls
-- ✅ HAVING alias_name > value        (WORKS)
-- ❌ HAVING COUNT(*) > value          (DOESN'T WORK)
-- ❌ HAVING SUM(column) > value       (DOESN'T WORK)
--
-- Workaround: Always give aggregates an alias and reference the alias in HAVING

-- Example 1: Filter traders by trade count (using alias)
WITH trades AS (
  SELECT 'Alice' as trader, 'AAPL' as symbol, 100 as qty
  UNION ALL SELECT 'Alice', 'MSFT', 200
  UNION ALL SELECT 'Alice', 'GOOGL', 50
  UNION ALL SELECT 'Bob', 'AAPL', 150
  UNION ALL SELECT 'Carol', 'MSFT', 300
)
SELECT
  trader,
  COUNT(*) as trade_count  -- Give it an alias
FROM trades
GROUP BY trader
HAVING trade_count > 1;  -- Reference the alias (NOT COUNT(*))
GO

-- Example 2: Filter by total exposure
WITH trades AS (
  SELECT 'Alice' as trader, 15000 as notional
  UNION ALL SELECT 'Alice', 60000
  UNION ALL SELECT 'Bob', 22000
  UNION ALL SELECT 'Carol', 90000
)
SELECT
  trader,
  SUM(notional) as total_exposure  -- Alias the aggregate
FROM trades
GROUP BY trader
HAVING total_exposure > 50000;  -- Use the alias
GO

-- Example 3: Multiple conditions in HAVING
WITH trades AS (
  SELECT 'Alice' as trader, 100 as qty, 150.0 as price
  UNION ALL SELECT 'Alice', 200, 300.0
  UNION ALL SELECT 'Alice', 50, 2800.0
  UNION ALL SELECT 'Bob', 150, 800.0
  UNION ALL SELECT 'Carol', 300, 250.0
)
SELECT
  trader,
  COUNT(*) as trade_count,
  SUM(qty * price) as total_value,
  AVG(price) as avg_price
FROM trades
GROUP BY trader
HAVING trade_count > 1           -- Multiple conditions
   AND total_value > 50000       -- All using aliases
   AND avg_price > 200;
GO

-- Example 4: Expressions on aggregate aliases
WITH trades AS (
  SELECT 'Alice' as trader, 100 as qty, 150.0 as price
  UNION ALL SELECT 'Alice', 200, 300.0
  UNION ALL SELECT 'Bob', 150, 800.0
)
SELECT
  trader,
  SUM(qty) as total_qty,
  COUNT(*) as trade_count,
  SUM(qty * price) as total_value
FROM trades
GROUP BY trader
HAVING total_value / trade_count > 30000;  -- Expression using aliases
GO

-- Example 5: Risk analysis - Filter counterparties by exposure
WITH exposures AS (
  SELECT 'Bank A' as counterparty, 'Alice' as trader, 500000 as exposure
  UNION ALL SELECT 'Bank A', 'Bob', 300000
  UNION ALL SELECT 'Bank A', 'Carol', 400000
  UNION ALL SELECT 'Bank B', 'Alice', 150000
  UNION ALL SELECT 'Bank B', 'Bob', 100000
)
SELECT
  counterparty,
  SUM(exposure) as total_exposure,
  COUNT(DISTINCT trader) as num_traders,
  MAX(exposure) as max_single_exposure
FROM exposures
GROUP BY counterparty
HAVING total_exposure > 1000000         -- High total exposure
   AND max_single_exposure > 400000;    -- Large single concentration
GO

-- Example 6: Volume analysis - Active symbols only
WITH trades AS (
  SELECT 'AAPL' as symbol, 'Alice' as trader, 1000 as volume
  UNION ALL SELECT 'AAPL', 'Bob', 1500
  UNION ALL SELECT 'AAPL', 'Carol', 2000
  UNION ALL SELECT 'MSFT', 'Alice', 500
  UNION ALL SELECT 'GOOGL', 'Bob', 3000
  UNION ALL SELECT 'GOOGL', 'Carol', 2500
)
SELECT
  symbol,
  SUM(volume) as total_volume,
  COUNT(DISTINCT trader) as unique_traders,
  AVG(volume) as avg_trade_size
FROM trades
GROUP BY symbol
HAVING total_volume > 3000              -- High volume symbols
   AND unique_traders > 2;              -- Multiple traders active
GO

-- Example 7: Comparison between aggregates
WITH trades AS (
  SELECT 'AAPL' as symbol, 140.0 as price
  UNION ALL SELECT 'AAPL', 145.0
  UNION ALL SELECT 'AAPL', 160.0
  UNION ALL SELECT 'MSFT', 300.0
  UNION ALL SELECT 'MSFT', 305.0
)
SELECT
  symbol,
  MIN(price) as min_price,
  MAX(price) as max_price,
  MAX(price) - MIN(price) as price_range
FROM trades
GROUP BY symbol
HAVING max_price - min_price > 10;      -- Volatile symbols
GO

-- COMMON MISTAKES TO AVOID
--
-- ❌ DON'T: HAVING COUNT(*) > 1
-- ✅ DO:    HAVING trade_count > 1  (where trade_count is COUNT(*) alias)
--
-- ❌ DON'T: HAVING SUM(notional) > 1000
-- ✅ DO:    HAVING total_notional > 1000  (where total_notional is SUM(notional) alias)
--
-- ❌ DON'T: HAVING AVG(price) > MAX(price) * 0.8
-- ✅ DO:    HAVING avg_price > max_price * 0.8  (using aliases for both)

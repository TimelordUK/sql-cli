-- CROSS JOIN Example: Trade Portfolio Stress Testing
-- Demonstrates how CROSS JOIN combines every trade with every risk scenario
-- Common use case: Risk analysis, scenario planning, parameter sweeps

-- Example 1: Basic CROSS JOIN - Every trader with every scenario
WITH traders AS (
  SELECT 'Alice' as trader, 'Equity' as desk
  UNION ALL
  SELECT 'Bob' as trader, 'FX' as desk
  UNION ALL
  SELECT 'Carol' as trader, 'Rates' as desk
),
risk_scenarios AS (
  SELECT 'Base' as scenario, 1.0 as factor
  UNION ALL
  SELECT 'Stressed' as scenario, 1.5 as factor
  UNION ALL
  SELECT 'Extreme' as scenario, 2.0 as factor
)
SELECT trader, desk, scenario, factor
FROM traders
CROSS JOIN risk_scenarios
ORDER BY trader, scenario;
GO

-- Example 2: Trade Stress Testing with PnL Impact
-- Calculate how each trade performs under different market scenarios
WITH trades AS (
  SELECT 'AAPL' as symbol, 100 as quantity, 150.0 as price, 'Alice' as trader, 'Long' as direction
  UNION ALL
  SELECT 'MSFT' as symbol, 200 as quantity, 300.0 as price, 'Bob' as trader, 'Long' as direction
  UNION ALL
  SELECT 'GOOGL' as symbol, 50 as quantity, 2800.0 as price, 'Alice' as trader, 'Long' as direction
  UNION ALL
  SELECT 'TSLA' as symbol, 75 as quantity, 800.0 as price, 'Carol' as trader, 'Short' as direction
),
market_scenarios AS (
  SELECT 'Base Case' as scenario, 1.0 as price_shock, 1.0 as vol_shock, 'Normal market' as description
  UNION ALL
  SELECT 'Market Stress' as scenario, 0.9 as price_shock, 1.5 as vol_shock, '10% drawdown' as description
  UNION ALL
  SELECT 'Black Swan' as scenario, 0.7 as price_shock, 3.0 as vol_shock, '30% crash' as description
  UNION ALL
  SELECT 'Rally' as scenario, 1.2 as price_shock, 0.8 as vol_shock, '20% rally' as description
)
SELECT
  trader,
  symbol,
  direction,
  quantity,
  price as original_price,
  scenario,
  description,
  price * price_shock as stressed_price,
  quantity * price as original_notional,
  quantity * price * price_shock as stressed_notional,
  -- PnL impact depends on direction
  CASE
    WHEN direction = 'Long' THEN (quantity * price * price_shock) - (quantity * price)
    WHEN direction = 'Short' THEN (quantity * price) - (quantity * price * price_shock)
    ELSE 0
  END as pnl_impact
FROM trades
CROSS JOIN market_scenarios
ORDER BY trader, symbol, scenario;
GO

-- Example 3: Multi-Parameter Grid Search
-- Test different combinations of parameters (common in quant research)
WITH strike_prices AS (
  SELECT 140 as strike UNION ALL
  SELECT 145 as strike UNION ALL
  SELECT 150 as strike UNION ALL
  SELECT 155 as strike UNION ALL
  SELECT 160 as strike
),
volatilities AS (
  SELECT 0.15 as vol UNION ALL
  SELECT 0.20 as vol UNION ALL
  SELECT 0.25 as vol UNION ALL
  SELECT 0.30 as vol
),
maturities AS (
  SELECT 30 as days UNION ALL
  SELECT 60 as days UNION ALL
  SELECT 90 as days
)
SELECT
  strike,
  vol,
  days,
  -- Simplified Black-Scholes approximation (for demonstration)
  150 - strike as intrinsic_value,
  vol * SQRT(days / 365.0) * strike as time_value_proxy
FROM strike_prices
CROSS JOIN volatilities
CROSS JOIN maturities
ORDER BY strike, vol, days;
GO

-- Example 4: Counterparty Risk Matrix
-- Every trader exposure against every counterparty limit
WITH trader_positions AS (
  SELECT 'Alice' as trader, 1500000 as exposure
  UNION ALL
  SELECT 'Bob' as trader, 2200000 as exposure
  UNION ALL
  SELECT 'Carol' as trader, 900000 as exposure
),
counterparty_limits AS (
  SELECT 'Bank A' as counterparty, 2000000 as risk_limit
  UNION ALL
  SELECT 'Bank B' as counterparty, 1000000 as risk_limit
  UNION ALL
  SELECT 'Hedge Fund C' as counterparty, 3000000 as risk_limit
)
SELECT
  trader,
  exposure,
  counterparty,
  risk_limit as counterparty_limit,
  CASE
    WHEN exposure <= risk_limit THEN 'PASS'
    ELSE 'LIMIT BREACH'
  END as status,
  risk_limit - exposure as available_capacity
FROM trader_positions
CROSS JOIN counterparty_limits
ORDER BY trader, counterparty;
GO

-- Example 5: Performance Note
-- CROSS JOIN has safety limit of 1,000,000 rows to prevent memory explosion
-- Example: 1000 rows × 1000 rows = 1,000,000 rows (maximum allowed)
-- Exceeding this will result in error: "CROSS JOIN would produce X rows, which exceeds the safety limit"

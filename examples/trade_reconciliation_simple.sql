-- ============================================================================
-- Trade Reconciliation: Finding Unmoved Positions (Simplified)
-- ============================================================================
-- Run: ./target/release/sql-cli sample_trades.csv -f examples/trade_reconciliation_simple.sql -o table
-- ============================================================================

-- First query: Find all trades in HOLDING or UNKNOWN accounts
SELECT 
    PlatformOrderId,
    CashAccount,
    Status,
    BuySell,
    Quantity,
    Price
FROM sample_trades
WHERE (CashAccount = 'HOLDING' OR CashAccount = 'UNKNOWN')
  AND Status = 'Active'
ORDER BY PlatformOrderId, BuySell;
GO

-- Summary by CashAccount
SELECT 
    CashAccount,
    COUNT(*) as trade_count,
    SUM(CASE WHEN BuySell = 'Buy' THEN Quantity ELSE 0 END) as buy_quantity,
    SUM(CASE WHEN BuySell = 'Sell' THEN Quantity ELSE 0 END) as sell_quantity,
    SUM(CASE WHEN BuySell = 'Buy' THEN Quantity ELSE -Quantity END) as net_position
FROM sample_trades
WHERE Status = 'Active'
GROUP BY CashAccount
ORDER BY trade_count DESC;
GO

-- Find net positions by removing instrument suffix manually
-- This shows all orders that start with the same root
SELECT 
    PlatformOrderId.Substring(0, 13) as RootOrder,
    COUNT(*) as trade_count,
    SUM(CASE WHEN BuySell = 'Buy' THEN Quantity ELSE 0 END) as total_buys,
    SUM(CASE WHEN BuySell = 'Sell' THEN Quantity ELSE 0 END) as total_sells,
    SUM(CASE WHEN BuySell = 'Buy' THEN Quantity ELSE -Quantity END) as net_position
FROM sample_trades
WHERE Status = 'Active'
GROUP BY PlatformOrderId.Substring(0, 13)
ORDER BY net_position DESC;
GO

-- Find orders that haven't moved from HOLDING
WITH holding_trades AS (
    SELECT 
        PlatformOrderId.Substring(0, 13) as RootOrder,
        CashAccount,
        SUM(Quantity) as total_qty
    FROM sample_trades
    WHERE Status = 'Active'
      AND CashAccount = 'HOLDING'
    GROUP BY PlatformOrderId.Substring(0, 13), CashAccount
)
SELECT 
    RootOrder,
    CashAccount,
    total_qty,
    'NEEDS MOVEMENT' as action
FROM holding_trades
WHERE total_qty > 0
ORDER BY total_qty DESC;
GO

-- Check for split positions (same root order in multiple accounts)
WITH order_accounts AS (
    SELECT 
        PlatformOrderId.Substring(0, 13) as RootOrder,
        CashAccount,
        COUNT(*) as trades,
        SUM(Quantity) as qty
    FROM sample_trades
    WHERE Status = 'Active'
    GROUP BY PlatformOrderId.Substring(0, 13), CashAccount
),
multi_account AS (
    SELECT 
        RootOrder,
        COUNT(DISTINCT CashAccount) as account_count
    FROM order_accounts
    GROUP BY RootOrder
    HAVING COUNT(DISTINCT CashAccount) > 1
)
SELECT 
    oa.RootOrder,
    oa.CashAccount,
    oa.trades,
    oa.qty,
    ma.account_count,
    'SPLIT POSITION' as issue
FROM order_accounts oa
INNER JOIN multi_account ma ON oa.RootOrder = ma.RootOrder
ORDER BY oa.RootOrder, oa.CashAccount;
GO
-- ============================================================================
-- Trade Reconciliation - Final Version with Dynamic Root Order Extraction
-- Uses IndexOf('_') to find underscore and extract root order ID
-- ============================================================================
-- Run: ./target/release/sql-cli yourfile.csv -f examples/trade_reconciliation_final.sql -o table
-- ============================================================================

-- Find positions stuck in HOLDING accounts
WITH trades_with_root AS (
    SELECT 
        PlatformOrderId,
        PlatformOrderId.Substring(0, PlatformOrderId.IndexOf('_')) as RootOrderId,
        CashAccount,
        Status,
        BuySell,
        Quantity,
        Price
    FROM sample_trades
    WHERE Status != 'Void'
)
SELECT 
    RootOrderId,
    CashAccount,
    COUNT(*) as trade_count,
    SUM(CASE WHEN BuySell = 'Buy' THEN Quantity ELSE 0 END) as buy_qty,
    SUM(CASE WHEN BuySell = 'Sell' THEN Quantity ELSE 0 END) as sell_qty,
    SUM(CASE WHEN BuySell = 'Buy' THEN Quantity ELSE 0 END) - 
    SUM(CASE WHEN BuySell = 'Sell' THEN Quantity ELSE 0 END) as net_position,
    ROUND(AVG(Price), 2) as avg_price,
    'STUCK IN HOLDING' as issue
FROM trades_with_root
WHERE CashAccount = 'HOLDING'
GROUP BY RootOrderId, CashAccount
ORDER BY net_position DESC;
GO

-- Find positions stuck in UNKNOWN accounts
WITH trades_with_root AS (
    SELECT 
        PlatformOrderId,
        PlatformOrderId.Substring(0, PlatformOrderId.IndexOf('_')) as RootOrderId,
        CashAccount,
        Status,
        BuySell,
        Quantity,
        Price
    FROM sample_trades
    WHERE Status != 'Void'
)
SELECT 
    RootOrderId,
    CashAccount,
    COUNT(*) as trade_count,
    SUM(CASE WHEN BuySell = 'Buy' THEN Quantity ELSE 0 END) as buy_qty,
    SUM(CASE WHEN BuySell = 'Sell' THEN Quantity ELSE 0 END) as sell_qty,
    SUM(CASE WHEN BuySell = 'Buy' THEN Quantity ELSE 0 END) - 
    SUM(CASE WHEN BuySell = 'Sell' THEN Quantity ELSE 0 END) as net_position,
    ROUND(AVG(Price), 2) as avg_price,
    'STUCK IN UNKNOWN' as issue
FROM trades_with_root
WHERE CashAccount = 'UNKNOWN'
GROUP BY RootOrderId, CashAccount
ORDER BY net_position DESC;
GO

-- Net positions by root order across all accounts
WITH trades_with_root AS (
    SELECT 
        PlatformOrderId.Substring(0, PlatformOrderId.IndexOf('_')) as RootOrderId,
        CashAccount,
        Status,
        BuySell,
        Quantity
    FROM sample_trades
    WHERE Status != 'Void'
)
SELECT 
    RootOrderId,
    COUNT(DISTINCT CashAccount) as accounts_used,
    SUM(CASE WHEN BuySell = 'Buy' THEN Quantity ELSE 0 END) as total_buys,
    SUM(CASE WHEN BuySell = 'Sell' THEN Quantity ELSE 0 END) as total_sells,
    SUM(CASE WHEN BuySell = 'Buy' THEN Quantity ELSE 0 END) - 
    SUM(CASE WHEN BuySell = 'Sell' THEN Quantity ELSE 0 END) as net_position,
    CASE 
        WHEN SUM(CASE WHEN BuySell = 'Buy' THEN Quantity ELSE 0 END) = 
             SUM(CASE WHEN BuySell = 'Sell' THEN Quantity ELSE 0 END) THEN 'FLAT'
        WHEN SUM(CASE WHEN BuySell = 'Buy' THEN Quantity ELSE 0 END) > 
             SUM(CASE WHEN BuySell = 'Sell' THEN Quantity ELSE 0 END) THEN 'LONG'
        ELSE 'SHORT'
    END as position_type
FROM trades_with_root
GROUP BY RootOrderId
ORDER BY ABS(SUM(CASE WHEN BuySell = 'Buy' THEN Quantity ELSE 0 END) - 
             SUM(CASE WHEN BuySell = 'Sell' THEN Quantity ELSE 0 END)) DESC;
GO

-- Summary by account type
WITH trades_with_root AS (
    SELECT 
        PlatformOrderId.Substring(0, PlatformOrderId.IndexOf('_')) as RootOrderId,
        CashAccount,
        Status,
        BuySell,
        Quantity,
        Price
    FROM sample_trades
    WHERE Status != 'Void'
)
SELECT 
    CashAccount,
    COUNT(DISTINCT RootOrderId) as unique_orders,
    COUNT(*) as total_trades,
    SUM(Quantity) as total_quantity,
    ROUND(SUM(Quantity * Price), 2) as total_value,
    ROUND(AVG(Price), 2) as avg_price,
    CASE 
        WHEN CashAccount = 'HOLDING' THEN 'NEEDS MOVEMENT'
        WHEN CashAccount = 'UNKNOWN' THEN 'NEEDS MOVEMENT'
        ELSE 'CLIENT ACCOUNT'
    END as account_status
FROM trades_with_root
GROUP BY CashAccount
ORDER BY unique_orders DESC;
GO
-- ============================================================================
-- Trade Reconciliation: Finding Unmoved Positions
-- Groups trades by root PlatformOrderId and identifies stuck positions
-- ============================================================================
-- Run: ./target/release/sql-cli trades.csv -f examples/trade_reconciliation.sql -o table
-- ============================================================================

-- First, extract root order ID and aggregate trades
WITH trade_summary AS (
    SELECT 
        -- Extract root PlatformOrderId (everything before underscore)
        CASE 
            WHEN PlatformOrderId.Contains('_') 
            THEN PlatformOrderId.Substring(0, PlatformOrderId.IndexOf('_'))
            ELSE PlatformOrderId 
        END as RootOrderId,
        CashAccount,
        Status,
        BuySell,
        COUNT(*) as trade_count,
        SUM(Quantity) as total_quantity,
        AVG(Price) as avg_price
    FROM sample_trades
    GROUP BY 
        CASE 
            WHEN PlatformOrderId.Contains('_') 
            THEN PlatformOrderId.Substring(0, PlatformOrderId.IndexOf('_'))
            ELSE PlatformOrderId 
        END,
        CashAccount,
        Status,
        BuySell
),
-- Find positions still in holding/unknown accounts (not voided)
unmoved_positions AS (
    SELECT * FROM trade_summary
    WHERE (CashAccount = 'HOLDING' OR CashAccount = 'UNKNOWN')
      AND Status != 'Void'
      AND Status != 'VOID'
      AND Status != 'Cancelled'
)
-- Show the unmoved positions
SELECT 
    RootOrderId,
    CashAccount,
    BuySell,
    Status,
    trade_count,
    total_quantity,
    ROUND(avg_price, 2) as avg_price,
    'NEEDS MOVEMENT' as action_required
FROM unmoved_positions
ORDER BY total_quantity DESC, RootOrderId;
GO

-- Summary of all positions by account type
WITH trade_summary AS (
    SELECT 
        CASE 
            WHEN PlatformOrderId.Contains('_') 
            THEN PlatformOrderId.Substring(0, PlatformOrderId.IndexOf('_'))
            ELSE PlatformOrderId 
        END as RootOrderId,
        CashAccount,
        Status,
        BuySell,
        COUNT(*) as trade_count,
        SUM(Quantity) as total_quantity
    FROM sample_trades
    GROUP BY 
        CASE 
            WHEN PlatformOrderId.Contains('_') 
            THEN PlatformOrderId.Substring(0, PlatformOrderId.IndexOf('_'))
            ELSE PlatformOrderId 
        END,
        CashAccount,
        Status,
        BuySell
)
SELECT 
    CashAccount,
    COUNT(DISTINCT RootOrderId) as unique_orders,
    SUM(CASE WHEN BuySell = 'Buy' OR BuySell = 'BUY' THEN trade_count ELSE 0 END) as buy_trades,
    SUM(CASE WHEN BuySell = 'Sell' OR BuySell = 'SELL' THEN trade_count ELSE 0 END) as sell_trades,
    SUM(CASE WHEN BuySell = 'Buy' OR BuySell = 'BUY' THEN total_quantity ELSE 0 END) as buy_quantity,
    SUM(CASE WHEN BuySell = 'Sell' OR BuySell = 'SELL' THEN total_quantity ELSE 0 END) as sell_quantity,
    SUM(CASE WHEN Status != 'Void' AND Status != 'VOID' THEN trade_count ELSE 0 END) as active_trades
FROM trade_summary
GROUP BY CashAccount
ORDER BY unique_orders DESC;
GO

-- Find specific problematic patterns
WITH trade_details AS (
    SELECT 
        CASE 
            WHEN PlatformOrderId.Contains('_') 
            THEN PlatformOrderId.Substring(0, PlatformOrderId.IndexOf('_'))
            ELSE PlatformOrderId 
        END as RootOrderId,
        PlatformOrderId,
        CashAccount,
        Status,
        BuySell,
        Quantity,
        Price
    FROM sample_trades
),
-- Find orders split across multiple accounts
split_orders AS (
    SELECT 
        RootOrderId,
        COUNT(DISTINCT CashAccount) as account_count,
        COUNT(*) as trade_count
    FROM trade_details
    WHERE Status != 'Void' AND Status != 'VOID'
    GROUP BY RootOrderId
    HAVING COUNT(DISTINCT CashAccount) > 1
)
SELECT 
    td.RootOrderId,
    td.CashAccount,
    td.BuySell,
    COUNT(*) as trades_in_account,
    SUM(td.Quantity) as total_quantity,
    so.account_count as accounts_used,
    'SPLIT ACROSS ACCOUNTS' as issue
FROM trade_details td
INNER JOIN split_orders so ON td.RootOrderId = so.RootOrderId
WHERE td.Status != 'Void' AND td.Status != 'VOID'
GROUP BY td.RootOrderId, td.CashAccount, td.BuySell, so.account_count
ORDER BY td.RootOrderId, td.CashAccount;
GO

-- Net position by root order (to check if fully netted)
WITH trade_positions AS (
    SELECT 
        CASE 
            WHEN PlatformOrderId.Contains('_') 
            THEN PlatformOrderId.Substring(0, PlatformOrderId.IndexOf('_'))
            ELSE PlatformOrderId 
        END as RootOrderId,
        SUM(CASE 
            WHEN (BuySell = 'Buy' OR BuySell = 'BUY') AND Status != 'Void' 
            THEN Quantity 
            ELSE 0 
        END) as buy_qty,
        SUM(CASE 
            WHEN (BuySell = 'Sell' OR BuySell = 'SELL') AND Status != 'Void' 
            THEN Quantity 
            ELSE 0 
        END) as sell_qty
    FROM sample_trades
    GROUP BY 
        CASE 
            WHEN PlatformOrderId.Contains('_') 
            THEN PlatformOrderId.Substring(0, PlatformOrderId.IndexOf('_'))
            ELSE PlatformOrderId 
        END
)
SELECT 
    RootOrderId,
    buy_qty,
    sell_qty,
    buy_qty - sell_qty as net_position,
    CASE 
        WHEN buy_qty - sell_qty = 0 THEN 'FLAT'
        WHEN buy_qty - sell_qty > 0 THEN 'LONG'
        ELSE 'SHORT'
    END as position_type,
    CASE 
        WHEN buy_qty - sell_qty = 0 THEN 'OK'
        ELSE 'CHECK'
    END as status
FROM trade_positions
WHERE buy_qty > 0 OR sell_qty > 0
ORDER BY ABS(buy_qty - sell_qty) DESC;
GO

-- Find orders only in HOLDING/UNKNOWN (never moved)
WITH trade_accounts AS (
    SELECT 
        CASE 
            WHEN PlatformOrderId.Contains('_') 
            THEN PlatformOrderId.Substring(0, PlatformOrderId.IndexOf('_'))
            ELSE PlatformOrderId 
        END as RootOrderId,
        CashAccount,
        Status,
        COUNT(*) as trade_count,
        SUM(Quantity) as total_qty
    FROM sample_trades
    WHERE Status != 'Void' AND Status != 'VOID'
    GROUP BY 
        CASE 
            WHEN PlatformOrderId.Contains('_') 
            THEN PlatformOrderId.Substring(0, PlatformOrderId.IndexOf('_'))
            ELSE PlatformOrderId 
        END,
        CashAccount,
        Status
),
only_holding AS (
    SELECT 
        RootOrderId,
        SUM(CASE WHEN CashAccount = 'HOLDING' OR CashAccount = 'UNKNOWN' THEN 1 ELSE 0 END) as holding_count,
        SUM(CASE WHEN CashAccount != 'HOLDING' AND CashAccount != 'UNKNOWN' THEN 1 ELSE 0 END) as real_count
    FROM trade_accounts
    GROUP BY RootOrderId
    HAVING SUM(CASE WHEN CashAccount != 'HOLDING' AND CashAccount != 'UNKNOWN' THEN 1 ELSE 0 END) = 0
)
SELECT 
    ta.RootOrderId,
    ta.CashAccount,
    ta.trade_count,
    ta.total_qty,
    'NEVER MOVED FROM HOLDING' as issue
FROM trade_accounts ta
INNER JOIN only_holding oh ON ta.RootOrderId = oh.RootOrderId
ORDER BY ta.total_qty DESC;
GO
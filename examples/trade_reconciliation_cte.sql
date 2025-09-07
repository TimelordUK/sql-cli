-- ============================================================================
-- Trade Reconciliation using CTEs
-- First extracts root order ID, then analyzes positions
-- ============================================================================
-- Run: ./target/release/sql-cli data/sample_trades.csv -f examples/trade_reconciliation_cte.sql -o table
-- ============================================================================

-- Extract root order IDs and find unmoved positions
WITH trades_with_root AS (
    SELECT 
        PlatformOrderId,
        PlatformOrderId.Substring(0, 13) as RootOrderId,
        CashAccount,
        Status,
        BuySell,
        Quantity,
        Price
    FROM sample_trades
),
unmoved_positions AS (
    SELECT 
        RootOrderId,
        CashAccount,
        COUNT(*) as trade_count,
        SUM(CASE WHEN BuySell = 'Buy' THEN Quantity ELSE 0 END) as buy_qty,
        SUM(CASE WHEN BuySell = 'Sell' THEN Quantity ELSE 0 END) as sell_qty,
        SUM(CASE WHEN BuySell = 'Buy' THEN Quantity * Price ELSE 0 END) as buy_value,
        SUM(CASE WHEN BuySell = 'Sell' THEN Quantity * Price ELSE 0 END) as sell_value
    FROM trades_with_root
    WHERE (CashAccount = 'HOLDING' OR CashAccount = 'UNKNOWN')
      AND Status = 'Active'
    GROUP BY RootOrderId, CashAccount
)
SELECT 
    RootOrderId,
    CashAccount,
    trade_count,
    buy_qty,
    sell_qty,
    buy_qty - sell_qty as net_position,
    ROUND(buy_value, 2) as buy_value,
    ROUND(sell_value, 2) as sell_value
FROM unmoved_positions
WHERE buy_qty > sell_qty
ORDER BY buy_qty - sell_qty DESC;
GO

-- Find net positions across all accounts by root order
WITH trades_with_root AS (
    SELECT 
        PlatformOrderId.Substring(0, 13) as RootOrderId,
        CashAccount,
        Status,
        BuySell,
        Quantity
    FROM sample_trades
),
net_positions AS (
    SELECT 
        RootOrderId,
        SUM(CASE WHEN BuySell = 'Buy' AND Status = 'Active' THEN Quantity ELSE 0 END) as total_buys,
        SUM(CASE WHEN BuySell = 'Sell' AND Status = 'Active' THEN Quantity ELSE 0 END) as total_sells,
        COUNT(DISTINCT CashAccount) as accounts_used
    FROM trades_with_root
    GROUP BY RootOrderId
)
SELECT 
    RootOrderId,
    total_buys,
    total_sells,
    total_buys - total_sells as net_position,
    accounts_used,
    CASE 
        WHEN total_buys = total_sells THEN 'FLAT'
        WHEN total_buys > total_sells THEN 'LONG'
        ELSE 'SHORT'
    END as position_status
FROM net_positions
ORDER BY ABS(total_buys - total_sells) DESC;
GO

-- Identify split positions (orders across multiple accounts)
WITH trades_with_root AS (
    SELECT 
        PlatformOrderId.Substring(0, 13) as RootOrderId,
        CashAccount,
        Status,
        Quantity
    FROM sample_trades
    WHERE Status = 'Active'
),
account_summary AS (
    SELECT 
        RootOrderId,
        CashAccount,
        SUM(Quantity) as qty_in_account
    FROM trades_with_root
    GROUP BY RootOrderId, CashAccount
),
split_orders AS (
    SELECT 
        RootOrderId,
        COUNT(DISTINCT CashAccount) as account_count
    FROM account_summary
    GROUP BY RootOrderId
    HAVING COUNT(DISTINCT CashAccount) > 1
)
SELECT 
    a.RootOrderId,
    a.CashAccount,
    a.qty_in_account,
    s.account_count,
    'SPLIT ACROSS ' || s.account_count || ' ACCOUNTS' as issue
FROM account_summary a
INNER JOIN split_orders s ON a.RootOrderId = s.RootOrderId
ORDER BY a.RootOrderId, a.CashAccount;
GO

-- Summary statistics by account type
WITH trades_with_root AS (
    SELECT 
        PlatformOrderId.Substring(0, 13) as RootOrderId,
        CashAccount,
        Status,
        BuySell,
        Quantity,
        Price
    FROM sample_trades
    WHERE Status = 'Active'
),
account_stats AS (
    SELECT 
        CashAccount,
        COUNT(DISTINCT RootOrderId) as unique_orders,
        COUNT(*) as total_trades,
        SUM(Quantity) as total_quantity,
        SUM(Quantity * Price) as total_value
    FROM trades_with_root
    GROUP BY CashAccount
)
SELECT 
    CashAccount,
    unique_orders,
    total_trades,
    total_quantity,
    ROUND(total_value, 2) as total_value,
    ROUND(total_value / total_quantity, 2) as avg_price,
    CASE 
        WHEN CashAccount = 'HOLDING' OR CashAccount = 'UNKNOWN' THEN 'NEEDS REVIEW'
        ELSE 'CLIENT ACCOUNT'
    END as account_type
FROM account_stats
ORDER BY unique_orders DESC;
GO

-- Find orders that never left HOLDING/UNKNOWN
WITH trades_with_root AS (
    SELECT 
        PlatformOrderId.Substring(0, 13) as RootOrderId,
        CashAccount,
        Status,
        Quantity
    FROM sample_trades
    WHERE Status = 'Active'
),
order_accounts AS (
    SELECT 
        RootOrderId,
        MAX(CASE WHEN CashAccount = 'HOLDING' OR CashAccount = 'UNKNOWN' THEN 1 ELSE 0 END) as has_holding,
        MAX(CASE WHEN CashAccount != 'HOLDING' AND CashAccount != 'UNKNOWN' THEN 1 ELSE 0 END) as has_client
    FROM trades_with_root
    GROUP BY RootOrderId
),
stuck_orders AS (
    SELECT RootOrderId
    FROM order_accounts
    WHERE has_holding = 1 AND has_client = 0
)
SELECT 
    t.RootOrderId,
    t.CashAccount,
    SUM(t.Quantity) as stuck_quantity,
    COUNT(*) as trade_count,
    'NEVER MOVED' as status
FROM trades_with_root t
INNER JOIN stuck_orders s ON t.RootOrderId = s.RootOrderId
GROUP BY t.RootOrderId, t.CashAccount
ORDER BY stuck_quantity DESC;
GO
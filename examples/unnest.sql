-- #! ../data/fix_allocations.csv

-- UNNEST: Row Expansion for Delimited Strings
-- ============================================
-- UNNEST splits delimited strings into separate rows, useful for:
-- - FIX protocol allocation messages
-- - Tag lists and CSV-embedded arrays
-- - Any pipe/comma-delimited data

-- Example 1: Single UNNEST - Expand accounts into separate rows
-- Input:  ORD001, "ACC_1|ACC_2|ACC_3"
-- Output: ORD001, ACC_1
--         ORD001, ACC_2
--         ORD001, ACC_3
SELECT
    order_id,
    UNNEST(accounts, '|') AS account
FROM fix_allocations
WHERE order_id = 'ORD001';
GO

-- Example 2: Multiple UNNEST - Expand both accounts AND amounts
-- This creates matched pairs: each account with its corresponding amount
SELECT
    order_id,
    symbol,
    UNNEST(accounts, '|') AS account,
    UNNEST(amounts, ',') AS amount
FROM fix_allocations
WHERE order_id = 'ORD001';
GO

-- Example 3: UNNEST all rows - Process entire allocation set
-- Shows how regular columns (order_id, symbol) are replicated
-- across all expanded rows
SELECT
    order_id,
    symbol,
    UNNEST(accounts, '|') AS account,
    UNNEST(amounts, ',') AS amount
FROM fix_allocations;
GO

-- Example 4: UNNEST with WHERE filter
-- Filter BEFORE expansion - only process Allocation (AS) messages
SELECT
    order_id,
    UNNEST(accounts, '|') AS account
FROM fix_allocations
WHERE msg_type = 'AS';
GO

-- Example 5: UNNEST with ORDER BY
-- Sort AFTER expansion - alphabetically by account name
SELECT
    UNNEST(accounts, '|') AS account,
    order_id
FROM fix_allocations
ORDER BY account;
GO

-- Example 6: Mismatched lengths - NULL padding
-- When arrays have different lengths, shorter ones are padded with NULL
-- ORD002 has 2 accounts but 2 amounts - they match
-- If data had 3 accounts and 2 amounts, the 3rd amount would be NULL
SELECT
    order_id,
    UNNEST(accounts, '|') AS account,
    UNNEST(amounts, ',') AS amount
FROM fix_allocations
WHERE order_id = 'ORD002';
GO

-- Example 7: Single item (no delimiter)
-- UNNEST works even when there's no delimiter - returns single row
SELECT
    order_id,
    UNNEST(accounts, '|') AS account,
    UNNEST(amounts, ',') AS amount
FROM fix_allocations
WHERE order_id = 'ORD003';
GO

-- Real-world use case: Analyze allocation distribution
-- Shows all account allocations sorted by amount for easy analysis
SELECT
    order_id,
    UNNEST(accounts, '|') AS account,
    UNNEST(amounts, ',') AS amount
FROM fix_allocations
WHERE msg_type = 'AS'
ORDER BY amount DESC;
GO

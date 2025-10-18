-- #! ../data/production_vwap_final.csv
--
-- File-Level Variables Demo
-- This demonstrates using @SET to define variables at the file level
-- that can be used throughout the script with ${VAR_NAME} syntax
--
-- To use:
-- 1. Change ORDER_ID below to the ID you want to query
-- 2. Run with \sq in Neovim or: sql-cli -f examples/file_variables_demo.sql
--

-- Define file-level variables
-- @SET ORDER_ID = CLIENT_001
-- @SET TICKER = ASML.AS
-- @SET STATUS = completed

select * from data;
go

-- Query 1: Find specific order
SELECT
    snapshot_time,
    event_type,
    order_id,
    ticker,
    side,
    client_name,
    event_type,
    state,
    filled_quantity
FROM sales_data
WHERE order_id = '${ORDER_ID}'
;
GO

-- Query 2: Filter by amount and status
SELECT
*
FROM data
WHERE ticker = '${TICKER}'
;
GO


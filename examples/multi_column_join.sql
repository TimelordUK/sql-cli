-- #! ../data/test_trades_multi.csv

-- Example: Multiple JOIN conditions with AND
-- This demonstrates joining tables on multiple columns using AND

-- Match buy and sell trades on both instrument and price
WITH
    buy_trades AS (
        SELECT * FROM test_trades_multi
        WHERE side = 'Buy'
    ),
    sell_trades AS (
        SELECT * FROM test_trades_multi
        WHERE side = 'Sell'
    )
SELECT
    buy.instrument,
    buy.quantity as buy_qty,
    buy.price as buy_price,
    sell.quantity as sell_qty,
    sell.price as sell_price
FROM buy_trades buy
INNER JOIN sell_trades sell
    ON buy.instrument = sell.instrument
    AND buy.price = sell.price;
GO

-- Find trades with different quantities for same instrument/price
WITH
    buy_trades AS (
        SELECT * FROM test_trades_multi
        WHERE side = 'Buy'
    ),
    sell_trades AS (
        SELECT * FROM test_trades_multi
        WHERE side = 'Sell'
    )
SELECT
    buy.instrument,
    buy.quantity as buy_qty,
    sell.quantity as sell_qty,
    buy.price
FROM buy_trades buy
INNER JOIN sell_trades sell
    ON buy.instrument = sell.instrument
    AND buy.price = sell.price
    AND buy.quantity <> sell.quantity;
GO

-- Self-join to find all trade pairs for the same instrument
-- where first trade has lower ID than second
WITH trades AS (
    SELECT * FROM test_trades_multi
)
SELECT
    a.trade_id as trade1_id,
    b.trade_id as trade2_id,
    a.instrument,
    a.side as side1,
    b.side as side2
FROM trades a
INNER JOIN trades b
    ON a.instrument = b.instrument
    AND a.trade_id < b.trade_id
ORDER BY trade1_id, trade2_id;
GO
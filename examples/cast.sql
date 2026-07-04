-- #! ../data/trades.csv
-- CAST / TRY_CAST Examples
-- Demonstrates explicit type casting within sql-cli's type system.
-- data: data/trades.csv  (columns: symbol, trade_time, price, volume)
--
-- sql-cli coerces types implicitly for most operations; CAST(expr AS type) makes
-- that conversion explicit. We support the handful of types the engine actually
-- stores: INTEGER, DOUBLE, VARCHAR, BOOLEAN, DATE/TIMESTAMP. The wider "zoo" of
-- SQL type names (CHAR, TEXT, DECIMAL, BIGINT, ...) is mapped onto these, and any
-- precision/scale such as DECIMAL(10,2) or VARCHAR(50) is accepted but ignored.

-- === Basic casts on literals ===

-- String -> INTEGER, float -> INTEGER (rounds), integer -> DOUBLE
SELECT
    CAST('42' AS INTEGER)   AS str_to_int,
    CAST(2.9 AS INTEGER)    AS float_to_int,
    CAST(5 AS DOUBLE)       AS int_to_double;
GO

-- Float -> INTEGER rounds to nearest (not truncates), with round-half-to-even
-- on exact .5 ties, matching DuckDB: 2.5 -> 2, 3.5 -> 4.
SELECT
    CAST(2.5 AS INTEGER) AS half_down,
    CAST(3.5 AS INTEGER) AS half_up;
GO

-- === The character-type "zoo" all collapses to VARCHAR ===

-- VARCHAR, CHAR, TEXT and STRING are equivalent; a size like VARCHAR(50) is
-- parsed and ignored.
SELECT
    CAST(7 AS VARCHAR)      AS as_varchar,
    CAST(7 AS TEXT)         AS as_text,
    CAST(7 AS VARCHAR(50))  AS as_varchar_sized;
GO

-- DECIMAL / NUMERIC map to DOUBLE; the precision/scale spec is ignored
-- (we cast within our own types rather than honouring width or scale).
SELECT CAST(price AS DECIMAL(10, 2)) AS price_as_decimal
FROM trades;
GO

-- === Casting to BOOLEAN ===

-- Numbers: zero is false, anything non-zero is true.
-- Strings: true/false, t/f, yes/no, 1/0, on/off (case-insensitive).
SELECT
    CAST(0 AS BOOLEAN)        AS zero_is_false,
    CAST(3 AS BOOLEAN)        AS nonzero_is_true,
    CAST('true' AS BOOLEAN)   AS str_true,
    CAST('no' AS BOOLEAN)     AS str_no;
GO

-- === Casting real columns ===

-- Round each trade price to a whole number.
SELECT symbol, price, CAST(price AS INTEGER) AS price_rounded
FROM trades;
GO

-- CAST participates in expressions like any other value.
SELECT symbol, CAST(volume AS DOUBLE) / 1000 AS volume_k
FROM trades;
GO

-- Use CAST in a WHERE clause to filter on the whole-number price.
SELECT symbol, price
FROM trades
WHERE CAST(price AS INTEGER) > 185;
GO

-- === TRY_CAST: NULL instead of an error on failure ===

-- CAST('AAPL' AS INTEGER) would raise an error; TRY_CAST yields NULL instead,
-- so a bad value doesn't abort the whole query.
SELECT
    symbol,
    TRY_CAST(symbol AS INTEGER) AS symbol_as_int,
    TRY_CAST('123' AS INTEGER)  AS good_value
FROM trades;
GO

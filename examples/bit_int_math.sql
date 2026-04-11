-- Integer Bitwise Operations Examples
-- Demonstrates the integer-side bitwise functions (complement to bit_string_math.sql).

-- Population count: number of set bits in an integer.
-- POPCOUNT is the canonical name; COUNT_BITS and the now-polymorphic BIT_COUNT
-- return the same result.
SELECT
    7 as n,
    POPCOUNT(7) as popcount,
    COUNT_BITS(7) as count_bits,
    BIT_COUNT(7) as bit_count_int,
    BIT_COUNT('111') as bit_count_str;
GO

-- POPCOUNT across a range, visualised.
SELECT
    value as n,
    LPAD(TO_BINARY(value), 8, '0') as binary,
    POPCOUNT(value) as bits_set,
    REPEAT('█', POPCOUNT(value)) as density
FROM RANGE(0, 16);
GO

-- LEADING_ZEROS: default width is 64, so LEADING_ZEROS(1) == 63.
-- Pass an explicit width (8/16/32/64) for the intuitive answer within
-- a smaller container.
SELECT
    8 as n,
    LEADING_ZEROS(8) as lz_default_64,
    LEADING_ZEROS(8, 32) as lz_32,
    LEADING_ZEROS(8, 16) as lz_16,
    LEADING_ZEROS(8, 8) as lz_8;
GO

-- TRAILING_ZEROS: width-invariant, -1 for n=0 (matches LOWEST_BIT).
SELECT
    value as n,
    LPAD(TO_BINARY(value), 6, '0') as binary,
    TRAILING_ZEROS(value) as tz,
    LOWEST_BIT(value) as lowest_bit
FROM RANGE(0, 17);
GO

-- Powers of two: one set bit, leading zeros decrease by 1 each step.
-- POW() returns a float, so we cast with TO_INT() since the bit functions
-- only accept integers (same convention as COUNT_BITS / HIGHEST_BIT).
WITH powers AS (
    SELECT value as exponent, TO_INT(POW(2, value)) as n
    FROM RANGE(0, 16)
)
SELECT
    exponent,
    n,
    LPAD(TO_BINARY(n), 16, '0') as binary,
    POPCOUNT(n) as popcount,
    LEADING_ZEROS(n, 16) as lz_16,
    TRAILING_ZEROS(n) as tz,
    IS_POWER_OF_TWO(n) as is_pow2
FROM powers;
GO

-- HIGHEST_BIT vs LEADING_ZEROS: inside an N-bit window they're complementary.
-- highest_bit(n) == (width - 1) - leading_zeros(n, width).
SELECT
    value as n,
    HIGHEST_BIT(value) as highest_bit,
    LEADING_ZEROS(value, 8) as lz_8,
    (8 - 1 - LEADING_ZEROS(value, 8)) as derived_highest_bit
FROM RANGE(1, 16);
GO

-- NEXT_POWER_OF_TWO + LEADING_ZEROS: how many bits do we need to store n?
-- bits_needed = width - leading_zeros(n, width).
SELECT
    value as n,
    NEXT_POWER_OF_TWO(value) as next_pow2,
    LEADING_ZEROS(value, 32) as lz_32,
    (32 - LEADING_ZEROS(value, 32)) as bits_needed
FROM RANGE(1, 20);
GO

-- BINARY_FORMAT for pretty-printing with separators.
SELECT
    BINARY_FORMAT(255) as plain,
    BINARY_FORMAT(255, '_') as grouped_4,
    BINARY_FORMAT(65535, '_', 8) as grouped_8;
GO

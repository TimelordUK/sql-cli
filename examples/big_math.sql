-- #! ../data/solar_system.csv

-- Big Math and Bit Manipulation Examples
-- ========================================
-- Demonstrating big integer operations, bit manipulation, and extreme calculations

-- Example 1: Big Integer Addition - Adding astronomical distances
-- Jupiter's orbit + Saturn's orbit in kilometers (values that overflow regular integers)
SELECT
    'Jupiter + Saturn Orbits' as calculation,
    BIGADD('778500000', '1434000000') as total_km,
    LENGTH(BIGADD('778500000', '1434000000')) as digits;
GO

-- Example 2: Extreme multiplication using BIGMUL
-- Calculate the number of seconds in multiple years (overflows regular int)
WITH time_calc AS (
    SELECT value as years FROM RANGE(100, 1001, 100)
)
SELECT
    years,
    BIGMUL(BIGMUL(BIGMUL(years, '365'), '24'), '3600') as seconds_in_years,
    LENGTH(BIGMUL(BIGMUL(BIGMUL(years, '365'), '24'), '3600')) as digit_count
FROM time_calc
LIMIT 5;
GO

-- Example 3: Integer limits exploration
SELECT
    'INT64_MAX' as constant,
    INT64_MAX() as value,
    TO_BINARY(INT64_MAX()) as binary_representation,
    LENGTH(TO_BINARY(INT64_MAX())) as bit_count;
GO

-- Example 4: Powers of 2 showing exponential growth
-- Demonstrate how quickly powers of 2 grow
WITH
    ranged AS (
        SELECT value AS n
        FROM RANGE(1, 101, 10)
    )
SELECT
    n,
    BIGPOW('2', n) AS power_of_2,
    LENGTH(BIGPOW('2', n)) AS decimal_digits,
    TO_BINARY(BIGPOW('2', n)) AS binary_representation
FROM ranged
LIMIT 5;
GO

-- Example 5: Bit manipulation - Powers of 2 and their properties
WITH powers AS (
    SELECT value as exponent FROM RANGE(0, 64)
)
SELECT
    exponent,
    BIGPOW('2', exponent) as power_of_2,
    TO_BINARY(BIGPOW('2', exponent)) as binary,
    LENGTH(TO_BINARY(BIGPOW('2', exponent))) as bit_length
FROM powers
WHERE exponent IN (0, 1, 2, 4, 8, 16, 32, 63)
ORDER BY exponent;
GO

-- Example 6: Factorials and their binary representations
-- Shows how quickly factorials grow
WITH factorials AS (
    SELECT value as n FROM RANGE(1, 25)
)
SELECT
    n,
    BIGFACT(n) as factorial,
    LENGTH(BIGFACT(n)) as digits,
    LENGTH(TO_BINARY(BIGFACT(n))) as bits_needed,
    -- Compare to INT64_MAX
    CASE
        WHEN LENGTH(TO_BINARY(BIGFACT(n))) > 64
        THEN 'Exceeds INT64_MAX'
        ELSE 'Within INT64'
    END as overflow_status
FROM factorials
WHERE n IN (5, 10, 15, 20, 24);
GO

-- Example 7: Large number calculations
-- Demonstrate calculations with very large numbers
SELECT
    '10^50' as number,
    BIGPOW('10', '50') as value,
    LENGTH(BIGPOW('10', '50')) as digits,
    SUBSTRING(BIGPOW('10', '50'), 1, 20) || '...' as preview;
GO

-- Example 8: Binary arithmetic patterns
-- Adding powers of 2 in binary shows interesting patterns
SELECT
    '2^10' as first_power,
    BIGPOW('2', '10') as first_value,
    TO_BINARY(BIGPOW('2', '10')) as first_binary,
    '2^20' as second_power,
    BIGPOW('2', '20') as second_value,
    TO_BINARY(BIGPOW('2', '20')) as second_binary,
    BIGADD(BIGPOW('2', '10'), BIGPOW('2', '20')) as sum,
    TO_BINARY(BIGADD(BIGPOW('2', '10'), BIGPOW('2', '20'))) as sum_binary;
GO

-- Example 9: Fibonacci numbers in binary
-- Shows the binary representation of Fibonacci sequence
WITH fib AS (
    SELECT value as n, FIBONACCI(value) as fib_num FROM RANGE(1, 50, 5)
)
SELECT
    n as position,
    fib_num as fibonacci_value,
    TO_BINARY(fib_num) as binary_representation,
    LENGTH(TO_BINARY(fib_num)) as bits_needed
FROM fib;
GO

-- Example 10: Extreme calculations - Googol and beyond
-- A googol is 10^100
SELECT
    'Googol (10^100)' as number,
    BIGPOW('10', '100') as value,
    LENGTH(BIGPOW('10', '100')) as digit_count,
    LENGTH(TO_BINARY(BIGPOW('10', '100'))) as bits_needed,
    -- Show first 50 chars
    SUBSTRING(BIGPOW('10', '100'), 1, 50) || '...' as preview;
GO

-- Example 11: Bit manipulation with OR operations
-- Combine different bit flags
SELECT
    '0b1010' as flag1,
    '0b1100' as flag2,
    FROM_BINARY('1010') as flag1_decimal,
    FROM_BINARY('1100') as flag2_decimal,
    -- Manual OR operation (would be: flag1 | flag2 = 0b1110 = 14)
    TO_BINARY(14) as or_result_binary,
    14 as or_result_decimal;
GO

-- Example 12: Prime factorization using generator
-- Factor some interesting numbers (shows individual prime factors)

SELECT
    1001 AS number,
    factor,
    power,
    TEXTJOIN('', 1, TEXTJOIN('', 1, factor, '^'), power) AS factor_term
FROM PRIME_FACTORS(1001);
go

SELECT
    12345 AS number,
    factor,
    power,
    TEXTJOIN('', 1, TEXTJOIN('', 1, factor, '^'), power) AS expr_4
FROM PRIME_FACTORS(12345);
GO

SELECT
    60,
    factor,
    power,
    factor || '^' || power as expression
FROM PRIME_FACTORS(60);
GO

-- Example 13: Powers of different bases in binary
-- Show how different bases grow in binary representation
WITH
    ranged AS (
        SELECT value AS base
        FROM RANGE(2, 11, 2)
    ),
    bases AS (
        SELECT
            base,
            BIGPOW(base, '10') AS power_10,
            TO_BINARY(BIGPOW(base, '10')) AS binary,
            LENGTH(TO_BINARY(BIGPOW(base, '10'))) AS bits
        FROM ranged
    )
SELECT *
FROM bases
GO
GO

-- Example 14: Maximum values and their properties
-- Show various integer type limits
SELECT
    'INT8_MAX' as type,
    INT8_MAX() as max_value,
    TO_BINARY(INT8_MAX()) as binary,
    LENGTH(TO_BINARY(INT8_MAX())) as bits;
GO

SELECT
    'INT16_MAX' as type,
    INT16_MAX() as max_value,
    TO_BINARY(INT16_MAX()) as binary,
    LENGTH(TO_BINARY(INT16_MAX())) as bits;
GO

SELECT
    'INT32_MAX' as type,
    INT32_MAX() as max_value,
    TO_BINARY(INT32_MAX()) as binary,
    LENGTH(TO_BINARY(INT32_MAX())) as bits;
GO

SELECT
    'INT64_MAX' as type,
    INT64_MAX() as max_value,
    TO_BINARY(INT64_MAX()) as binary,
    LENGTH(TO_BINARY(INT64_MAX())) as bits;
GO

-- Example 15: Collatz sequence exploration
-- Explore the Collatz sequence for number 27 (famous for its long sequence)
SELECT
    'Collatz(27)' as sequence,
    COUNT(*) as steps_to_one,
    MAX(value) as peak_value,
    MIN(value) as min_value
FROM COLLATZ(27);
GO

-- Show first 10 steps of Collatz sequence for 27
SELECT
    step,
    value,
    CASE
        WHEN value % 2 = 0 THEN 'even -> divide by 2'
        ELSE 'odd -> 3n + 1'
    END as operation
FROM COLLATZ(27)
LIMIT 10;
GO

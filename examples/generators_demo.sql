-- #!

-- Example 1: Generate prime numbers up to 100
SELECT * FROM GENERATE_PRIMES(100);
GO

-- Example 2: Get prime factorization of 2520 (LCM of 1-10)
SELECT
    factor,
    power,
    POWER(factor, power) AS value
FROM PRIME_FACTORS(2520);
GO

-- Example 3: First 20 Fibonacci numbers
SELECT * FROM FIBONACCI(20);
GO

-- Example 4: Find Fibonacci numbers that are also prime
WITH fib AS (
    SELECT value FROM FIBONACCI(20) WHERE value > 1
),
primes AS (
    SELECT prime FROM GENERATE_PRIMES(500)
)
SELECT
    value AS fibonacci_prime
FROM fib
WHERE value IN (SELECT prime FROM primes)
ORDER BY value;
GO

-- Example 5: Analyze prime gaps
WITH primes AS (
    SELECT
        prime,
        ROW_NUMBER() OVER (ORDER BY prime) AS position
    FROM GENERATE_PRIMES(100)
)
SELECT
    prime,
    LEAD(prime) OVER (ORDER BY prime) AS next_prime,
    LEAD(prime) OVER (ORDER BY prime) - prime AS gap
FROM primes
WHERE prime < 90;
GO

-- Example 6: Find numbers with exactly 3 prime factors
WITH numbers AS (
    SELECT * FROM RANGE(10, 50)
),
factor_counts AS (
    SELECT
        value,
        (SELECT COUNT(*) FROM PRIME_FACTORS(value)) AS num_factors
    FROM numbers
)
SELECT value
FROM factor_counts
WHERE num_factors = 3;
GO
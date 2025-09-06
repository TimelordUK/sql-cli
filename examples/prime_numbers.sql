-- Prime number functions showcase
-- sql-cli supports sophisticated prime number operations

-- Check if numbers are prime
SELECT 
    IS_PRIME(17) as seventeen_is_prime,
    IS_PRIME(100) as hundred_is_prime,
    IS_PRIME(997) as large_prime_check;
GO

-- Get the Nth prime number
SELECT 
    NTH_PRIME(1) as first_prime,
    NTH_PRIME(10) as tenth_prime,
    NTH_PRIME(100) as hundredth_prime,
    NTH_PRIME(1000) as thousandth_prime;
GO

-- Count primes up to a given number
SELECT 
    PRIME_PI(10) as primes_up_to_10,
    PRIME_PI(100) as primes_up_to_100,
    PRIME_PI(1000) as primes_up_to_1000;
GO

-- Get next and previous prime numbers
SELECT 
    NEXT_PRIME(100) as next_prime_after_100,
    PREV_PRIME(100) as prev_prime_before_100,
    NEXT_PRIME(1000) as next_prime_after_1000;
GO

-- Practical example: Finding prime-numbered IDs (requires data_table)
-- Uncomment below if you have a data_table with an id column:
-- SELECT 
--     id,
--     value,
--     IS_PRIME(id) as is_prime_id,
--     NTH_PRIME(id) as nth_prime_value
-- FROM data_table
-- WHERE IS_PRIME(id) = true
-- LIMIT 10;
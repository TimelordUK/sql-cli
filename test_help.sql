-- Test SQL file to demonstrate function and generator help
-- Place cursor on any function/generator name and press Shift+K to see help

-- Regular functions:
SELECT MEDIAN(salary) FROM employees;
SELECT PERCENTILE(score, 75) FROM tests;
SELECT SKEW(value, mean, stddev) FROM measurements;
SELECT KURTOSIS(value, mean, stddev) FROM measurements;

-- Generator functions:
SELECT * FROM FIBONACCI(20);
SELECT * FROM GENERATE_PRIMES(100);
SELECT * FROM COLLATZ(27);
SELECT * FROM RANDOM_INT(1, 100);

-- Combined example:
SELECT
    n,
    value,
    SQRT(value) as sqrt_value,
    IS_PRIME(value) as is_prime
FROM FIBONACCI(10);
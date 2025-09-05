#!/bin/bash
# Demonstration of Prime Number Functions in SQL CLI

echo "=== Prime Number Analysis with SQL CLI ==="
echo

echo "1. First 10 primes and their properties:"
./target/release/sql-cli data/numbers_1_to_100.csv -q "
    SELECT 
        n as position,
        PRIME(n) as prime_value,
        PRIME(n+1) - PRIME(n) as gap_to_next,
        IS_PRIME(PRIME(n)) as verification
    FROM numbers_1_to_100 
    WHERE n <= 10
" -o csv

echo
echo "2. Twin primes (primes with gap of 2) in first 50 positions:"
./target/release/sql-cli data/numbers_1_to_100.csv -q "
    SELECT 
        n as position,
        PRIME(n) as first_twin,
        PRIME(n+1) as second_twin
    FROM numbers_1_to_100 
    WHERE n <= 50 
        AND PRIME(n+1) - PRIME(n) = 2
" -o csv | head -10

echo
echo "3. Largest prime gaps in first 30 primes:"
./target/release/sql-cli data/numbers_1_to_100.csv -q "
    SELECT 
        n as position,
        PRIME(n) as prime,
        PRIME(n+1) as next_prime,
        PRIME(n+1) - PRIME(n) as gap
    FROM numbers_1_to_100 
    WHERE n <= 30
        AND PRIME(n+1) - PRIME(n) >= 6
" -o csv

echo
echo "4. Prime counting function demonstration:"
echo "up_to,prime_count,density_percent" > /tmp/prime_counts.csv
echo "10,$(./target/release/sql-cli -q "SELECT PRIME_COUNT(10) FROM DUAL" -o csv | tail -1),$(./target/release/sql-cli -q "SELECT ROUND(CAST(PRIME_COUNT(10) AS FLOAT) / 10 * 100, 1) FROM DUAL" -o csv | tail -1)" >> /tmp/prime_counts.csv
echo "100,$(./target/release/sql-cli -q "SELECT PRIME_COUNT(100) FROM DUAL" -o csv | tail -1),$(./target/release/sql-cli -q "SELECT ROUND(CAST(PRIME_COUNT(100) AS FLOAT) / 100 * 100, 1) FROM DUAL" -o csv | tail -1)" >> /tmp/prime_counts.csv
echo "1000,$(./target/release/sql-cli -q "SELECT PRIME_COUNT(1000) FROM DUAL" -o csv | tail -1),$(./target/release/sql-cli -q "SELECT ROUND(CAST(PRIME_COUNT(1000) AS FLOAT) / 1000 * 100, 1) FROM DUAL" -o csv | tail -1)" >> /tmp/prime_counts.csv
echo "10000,$(./target/release/sql-cli -q "SELECT PRIME_COUNT(10000) FROM DUAL" -o csv | tail -1),$(./target/release/sql-cli -q "SELECT ROUND(CAST(PRIME_COUNT(10000) AS FLOAT) / 10000 * 100, 1) FROM DUAL" -o csv | tail -1)" >> /tmp/prime_counts.csv
cat /tmp/prime_counts.csv

echo
echo "5. Performance test - accessing large pre-computed primes:"
time ./target/release/sql-cli /tmp/prime_counts.csv -q "
    SELECT 
        PRIME(1000) as thousandth_prime,
        PRIME(10000) as ten_thousandth_prime,
        PRIME(50000) as fifty_thousandth_prime,
        IS_PRIME(1299709) as is_100000th_prime
    FROM prime_counts
    LIMIT 1
" -o csv

echo
echo "6. Special prime properties:"
./target/release/sql-cli data/numbers_1_to_100.csv -q "
    SELECT 
        n,
        POWER(2, n) - 1 as mersenne_candidate,
        IS_PRIME(POWER(2, n) - 1) as is_mersenne_prime
    FROM numbers_1_to_100
    WHERE n IN (2, 3, 5, 7, 11, 13)
        AND IS_PRIME(n)
" -o csv

echo
echo "=== Prime Engine Statistics ==="
echo "Pre-computed primes: 100,000 (up to 1,299,709)"
echo "Instant access: O(1) for first 100K primes"
echo "Memory usage: ~400KB for prime tables"
echo "Algorithms: Sieve (build-time), Miller-Rabin (runtime for large numbers)"
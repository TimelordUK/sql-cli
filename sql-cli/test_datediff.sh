#!/bin/bash

# Test DATEDIFF function with our test CSV
echo "Testing DATEDIFF function..."
echo

# Test 1: Basic day difference
echo "Test 1: Basic day difference (should match expected_diff column)"
echo "SELECT id, DATEDIFF('day', order_date, ship_date) as calc_diff, expected_diff FROM test_dates LIMIT 5" | ./target/release/sql-cli test_dates.csv
echo

# Test 2: Check row 21 (should be 5 days)
echo "Test 2: Row 21 should show 5 days difference"
echo "SELECT id, order_date, ship_date, DATEDIFF('day', order_date, ship_date) as days_diff FROM test_dates WHERE id = 21" | ./target/release/sql-cli test_dates.csv
echo

# Test 3: Check negative differences (early deliveries)
echo "Test 3: Early deliveries (negative differences)"
echo "SELECT id, DATEDIFF('day', order_date, ship_date) as days_diff, status FROM test_dates WHERE id >= 81 AND id <= 85" | ./target/release/sql-cli test_dates.csv
echo

# Test 4: Calculate age in years (approximately)
echo "Test 4: Calculate age in years from birth_date"
echo "SELECT id, birth_date, order_date, DATEDIFF('year', birth_date, order_date) as age_years FROM test_dates LIMIT 5" | ./target/release/sql-cli test_dates.csv
echo

# Test 5: Test with NOW() function
echo "Test 5: Days since last login using NOW()"
echo "SELECT id, last_login, DATEDIFF('day', last_login, NOW()) as days_inactive FROM test_dates LIMIT 5" | ./target/release/sql-cli test_dates.csv
echo

# Test 6: Test TODAY() function
echo "Test 6: Days from order_date to TODAY()"
echo "SELECT id, order_date, TODAY() as today, DATEDIFF('day', order_date, TODAY()) as days_ago FROM test_dates LIMIT 3" | ./target/release/sql-cli test_dates.csv
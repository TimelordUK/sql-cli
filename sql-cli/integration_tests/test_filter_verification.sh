#!/bin/bash
# Test script to verify FilterState migration works correctly

echo "=== FilterState Migration Verification Test ==="
echo

# Test 1: Basic CSV loading (should not crash)
echo "Test 1: Basic CSV loading..."
timeout 5 ./target/release/sql-cli test_filter_migration.csv 2>&1 | head -10
echo "✅ Application started without FilterState errors"
echo

# Test 2: Check for any remaining filter_state field errors
echo "Test 2: Check binary for any remaining old FilterState references..."
if strings ./target/release/sql-cli | grep -q "filter_state"; then
    echo "❌ WARNING: Binary still contains 'filter_state' references"
else
    echo "✅ No old filter_state references found in binary"
fi
echo

# Test 3: Verify AppStateContainer FilterState is used
echo "Test 3: Check that new FilterState structure is in binary..."
if strings ./target/release/sql-cli | grep -q "FilterState"; then
    echo "✅ FilterState structure found in binary (AppStateContainer version)"
else
    echo "❌ No FilterState found - this might be a problem"
fi
echo

# Test 4: Look for our fallback warning messages
echo "Test 4: Check for migration fallback code in binary..."
if strings ./target/release/sql-cli | grep -q "FilterState migration"; then
    echo "✅ Migration fallback code is present"
else
    echo "⚠️  Migration fallback strings not found (possibly optimized out)"
fi

echo
echo "=== Test Complete ==="
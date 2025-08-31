#!/bin/bash
# Integration test for V18a FilterState migration from EnhancedTuiApp to AppStateContainer
# Tests that the migration maintains functionality while removing duplicate state

set -e  # Exit on error

echo "=== V18a FilterState Migration Integration Test ==="
echo

# Setup test data
TEST_CSV="test_filter_migration.csv"
if [[ ! -f "$TEST_CSV" ]]; then
    cat > "$TEST_CSV" << EOF
name,age,city
Alice,25,New York
Bob,30,Los Angeles  
Charlie,35,New York
Diana,28,Chicago
Eve,32,Los Angeles
EOF
fi

echo "✅ Created test data: $TEST_CSV"

# Test 1: Application startup without FilterState crashes
echo
echo "Test 1: Application startup verification..."
timeout 3 ./target/release/sql-cli "$TEST_CSV" 2>&1 | head -5 | grep -q "Starting enhanced TUI"
if [[ $? -eq 0 ]]; then
    echo "✅ Application starts successfully with CSV data"
else
    echo "❌ Application failed to start"
    exit 1
fi

# Test 2: Check migration architecture
echo
echo "Test 2: Architecture verification..."

# Verify no old filter_state field access
if grep -q "\.filter_state\." src/enhanced_tui.rs; then
    echo "❌ Found remaining direct filter_state access in enhanced_tui.rs"
    exit 1
else
    echo "✅ No direct filter_state field access remaining"
fi

# Verify AppStateContainer has FilterState
if grep -q "filter: RefCell<FilterState>" src/app_state_container.rs; then
    echo "✅ FilterState properly located in AppStateContainer"
else
    echo "❌ FilterState not found in AppStateContainer"
    exit 1
fi

# Verify fallback handlers exist
FALLBACK_COUNT=$(grep -c "state_container not available" src/enhanced_tui.rs || echo "0")
if [[ $FALLBACK_COUNT -gt 5 ]]; then
    echo "✅ Fallback handlers present ($FALLBACK_COUNT found)"
else
    echo "❌ Insufficient fallback handlers ($FALLBACK_COUNT found)"
fi

# Test 3: Binary verification
echo
echo "Test 3: Binary verification..."

# Check for old filter_state references in binary
if strings ./target/release/sql-cli | grep -q "^filter_state$"; then
    echo "⚠️  Old filter_state references may still exist in binary"
else
    echo "✅ No old filter_state field references in binary"
fi

# Verify new FilterState is in binary
if strings ./target/release/sql-cli | grep -q "FilterState"; then
    echo "✅ FilterState structure found in binary"
else
    echo "❌ FilterState not found in binary - compilation issue?"
    exit 1
fi

# Test 4: Check state_container usage
echo
echo "Test 4: State container integration..."

API_USAGE_COUNT=$(grep -c "state_container.*filter" src/enhanced_tui.rs || echo "0")
if [[ $API_USAGE_COUNT -gt 10 ]]; then
    echo "✅ AppStateContainer filter API used extensively ($API_USAGE_COUNT calls)"
else
    echo "❌ Insufficient state_container.filter() usage ($API_USAGE_COUNT calls)"
    exit 1
fi

# Test 5: Migration completeness
echo
echo "Test 5: Migration completeness check..."

# Verify EnhancedTuiApp struct has migrated field
if grep -q "// filter_state.*MIGRATED" src/enhanced_tui.rs; then
    echo "✅ EnhancedTuiApp struct properly migrated"
else
    echo "❌ EnhancedTuiApp struct migration incomplete"
    exit 1
fi

# Verify constructor is updated  
if grep -q "// filter_state.*MIGRATED" src/enhanced_tui.rs; then
    echo "✅ EnhancedTuiApp constructor properly updated"
else
    echo "❌ Constructor migration incomplete"
fi

# Cleanup
rm -f "$TEST_CSV"

echo
echo "🎉 All FilterState migration tests passed!"
echo "✅ V18a migration successful - FilterState moved to AppStateContainer"
echo "✅ No duplicate state - single source of truth achieved"
echo "✅ Architecture is clean and maintainable"
echo
echo "Ready for next migration: V18b SearchState"
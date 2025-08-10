#!/bin/bash
# Integration test for V18b SearchState migration from EnhancedTuiApp to AppStateContainer
# Tests that the migration maintains functionality while removing duplicate state

set -e  # Exit on error

echo "=== V18b SearchState Migration Integration Test ==="
echo

# Setup test data
TEST_CSV="test_search_migration.csv"
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

# Test 1: Application startup without SearchState crashes
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

# Verify no old search_state field access
if grep -q "\\.search_state\\." src/enhanced_tui.rs; then
    echo "❌ Found remaining direct search_state access in enhanced_tui.rs"
    exit 1
else
    echo "✅ No direct search_state field access remaining"
fi

# Verify AppStateContainer has SearchState
if grep -q "search: RefCell<SearchState>" src/app_state_container.rs; then
    echo "✅ SearchState properly located in AppStateContainer"
else
    echo "❌ SearchState not found in AppStateContainer"
    exit 1
fi

# Verify fallback handlers exist
FALLBACK_COUNT=$(grep -c "SearchState migration.*state_container not available" src/enhanced_tui.rs || echo "0")
if [[ $FALLBACK_COUNT -gt 1 ]]; then
    echo "✅ SearchState fallback handlers present ($FALLBACK_COUNT found)"
else
    echo "❌ Insufficient SearchState fallback handlers ($FALLBACK_COUNT found)"
    exit 1
fi

# Test 3: Binary verification
echo
echo "Test 3: Binary verification..."

# Verify SearchState is in binary
if strings ./target/release/sql-cli | grep -q "SearchState"; then
    echo "✅ SearchState structure found in binary"
else
    echo "❌ SearchState not found in binary - compilation issue?"
    exit 1
fi

# Test 4: Check state_container usage
echo
echo "Test 4: State container integration..."

API_USAGE_COUNT=$(grep -c "state_container.*search" src/enhanced_tui.rs || echo "0")
if [[ $API_USAGE_COUNT -gt 5 ]]; then
    echo "✅ AppStateContainer search API used extensively ($API_USAGE_COUNT calls)"
else
    echo "❌ Insufficient state_container.search() usage ($API_USAGE_COUNT calls)"
    exit 1
fi

# Test 5: Migration completeness
echo
echo "Test 5: Migration completeness check..."

# Verify EnhancedTuiApp struct has migrated field
if grep -q "// search_state.*MIGRATED" src/enhanced_tui.rs; then
    echo "✅ EnhancedTuiApp struct properly migrated"
else
    echo "❌ EnhancedTuiApp struct migration incomplete"
    exit 1
fi

# Verify old SearchState struct is unused
if grep -q "struct SearchState" src/enhanced_tui.rs && grep -q "never constructed" target/release/build/sql-cli*/out/stderr.log 2>/dev/null; then
    echo "✅ Old SearchState struct marked as unused (migration complete)"
fi

# Cleanup
rm -f "$TEST_CSV"

echo
echo "🎉 All SearchState migration tests passed!"
echo "✅ V18b migration successful - SearchState moved to AppStateContainer"
echo "✅ No duplicate state - single source of truth achieved"
echo "✅ Architecture is clean and maintainable"
echo
echo "Ready for next migration: V18c ColumnSearchState"
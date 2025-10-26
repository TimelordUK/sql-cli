#!/bin/bash
# Integration test runner for examples/ directory
#
# Usage:
#   ./tests/integration/test_examples.sh                    # Run all examples
#   ./tests/integration/test_examples.sh physics_astronomy  # Run specific example
#
# Test Modes:
#   1. FORMAL TEST: If examples/expectations/<name>.json exists
#      - Validates output exactly matches expected JSON
#      - Fails on any mismatch
#   2. SMOKE TEST: No expectation file exists
#      - Just ensures query runs without errors
#      - Acts as demo/documentation

set -e

CLI="./target/release/sql-cli"
EXAMPLES_DIR="examples"
EXPECTATIONS_DIR="examples/expectations"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Counters
TOTAL=0
FORMAL_TESTS=0
SMOKE_TESTS=0
PASSED=0
FAILED=0
SKIPPED=0

# Results arrays
declare -a FAILED_TESTS
declare -a PASSED_TESTS

echo "=== Examples Integration Test Suite ==="
echo

# Check if CLI binary exists
if [ ! -f "$CLI" ]; then
    echo -e "${RED}ERROR: $CLI not found. Run 'cargo build --release' first.${NC}"
    exit 1
fi

# Function to run a single test
run_test() {
    local sql_file="$1"
    local base_name=$(basename "$sql_file" .sql)
    local expectation_file="$EXPECTATIONS_DIR/${base_name}.json"

    TOTAL=$((TOTAL + 1))

    # Check if this is a formal test or smoke test
    if [ -f "$expectation_file" ]; then
        TEST_MODE="FORMAL"
        FORMAL_TESTS=$((FORMAL_TESTS + 1))
    else
        TEST_MODE="SMOKE"
        SMOKE_TESTS=$((SMOKE_TESTS + 1))
    fi

    echo -n "[$TEST_MODE] $base_name ... "

    # Run the SQL file and capture output
    if output=$($CLI -f "$sql_file" -o json 2>&1); then
        # Query executed successfully
        if [ "$TEST_MODE" = "FORMAL" ]; then
            # Compare with expected output
            if echo "$output" | jq -S . > /tmp/actual_output.json 2>/dev/null; then
                if diff -q <(jq -S . "$expectation_file") /tmp/actual_output.json > /dev/null 2>&1; then
                    echo -e "${GREEN}✓ PASS${NC}"
                    PASSED=$((PASSED + 1))
                    PASSED_TESTS+=("$base_name")
                else
                    echo -e "${RED}✗ FAIL - Output mismatch${NC}"
                    FAILED=$((FAILED + 1))
                    FAILED_TESTS+=("$base_name (output mismatch)")

                    # Show diff for debugging
                    echo -e "${YELLOW}Expected vs Actual:${NC}"
                    diff <(jq -S . "$expectation_file") /tmp/actual_output.json | head -20
                fi
            else
                echo -e "${RED}✗ FAIL - Invalid JSON output${NC}"
                FAILED=$((FAILED + 1))
                FAILED_TESTS+=("$base_name (invalid JSON)")
            fi
        else
            # Smoke test - just check it ran
            echo -e "${GREEN}✓ PASS (runs)${NC}"
            PASSED=$((PASSED + 1))
            PASSED_TESTS+=("$base_name")
        fi
    else
        # Query failed
        echo -e "${RED}✗ FAIL${NC}"
        FAILED=$((FAILED + 1))
        FAILED_TESTS+=("$base_name")

        # Show error for debugging
        echo -e "${YELLOW}Error:${NC}"
        echo "$output" | head -5
    fi
}

# Find all SQL files in examples directory
if [ $# -eq 0 ]; then
    # Run all examples
    for sql_file in "$EXAMPLES_DIR"/*.sql; do
        if [ -f "$sql_file" ]; then
            run_test "$sql_file"
        fi
    done
else
    # Run specific example(s)
    for pattern in "$@"; do
        found=false
        for sql_file in "$EXAMPLES_DIR"/${pattern}*.sql; do
            if [ -f "$sql_file" ]; then
                run_test "$sql_file"
                found=true
            fi
        done
        if ! $found; then
            echo -e "${YELLOW}WARNING: No example matching '$pattern' found${NC}"
        fi
    done
fi

# Print summary
echo
echo "=== Test Summary ==="
echo -e "Total examples:  $TOTAL"
echo -e "  Formal tests:  $FORMAL_TESTS (with expectations/*.json)"
echo -e "  Smoke tests:   $SMOKE_TESTS (no expectations)"
echo
echo -e "${GREEN}Passed: $PASSED${NC}"
echo -e "${RED}Failed: $FAILED${NC}"
echo

# Show failed tests
if [ $FAILED -gt 0 ]; then
    echo -e "${RED}Failed tests:${NC}"
    for test in "${FAILED_TESTS[@]}"; do
        echo "  - $test"
    done
    echo
    exit 1
fi

echo -e "${GREEN}All tests passed!${NC}"
exit 0

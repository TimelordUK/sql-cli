#!/bin/bash
# Test all SQL example files to ensure they work

EXAMPLES_DIR="examples"
SQL_CLI="./target/release/sql-cli"

# Color codes for output
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo "Testing all SQL example files..."
echo "================================"

# Array of example files to test
# Note: group_by_aggregates.sql requires actual data files, so it's excluded
examples=(
    "math_functions.sql"
    "string_functions.sql"
    "date_time_functions.sql"
    "physics_constants.sql"
    "chemical_formulas.sql"
    "showcase_all_features.sql"
)

failed=0
passed=0

for example in "${examples[@]}"; do
    if [ -f "$EXAMPLES_DIR/$example" ]; then
        echo -n "Testing $example... "
        
        # Run the example and capture exit code
        if $SQL_CLI -f "$EXAMPLES_DIR/$example" -o csv > /dev/null 2>&1; then
            echo -e "${GREEN}✓ PASSED${NC}"
            ((passed++))
        else
            echo -e "${RED}✗ FAILED${NC}"
            ((failed++))
            # Show the error
            echo "  Error details:"
            $SQL_CLI -f "$EXAMPLES_DIR/$example" -o csv 2>&1 | grep -E "Error|Failed" | head -3
        fi
    else
        echo -e "${RED}✗ File not found: $EXAMPLES_DIR/$example${NC}"
        ((failed++))
    fi
done

echo "================================"
echo "Results: $passed passed, $failed failed"

if [ $failed -eq 0 ]; then
    echo -e "${GREEN}All examples working correctly!${NC}"
    exit 0
else
    echo -e "${RED}Some examples failed. Please review.${NC}"
    exit 1
fi
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
examples=(
    "math_functions.sql"
    "string_functions.sql"
    "date_time_functions.sql"
    "physics_constants.sql"
    "chemical_formulas.sql"
    "showcase_all_features.sql"
)

# Test group_by_aggregates.sql with sample data if available
if [ -f "data/sales_sample.csv" ]; then
    examples+=("group_by_aggregates.sql")
    GROUP_BY_DATA="data/sales_sample.csv"
fi

# Test window functions with sales data
if [ -f "data/sales_data.csv" ]; then
    examples+=("window_functions.sql")
    examples+=("window_functions_filtering.sql")
    WINDOW_DATA="data/sales_data.csv"
fi

failed=0
passed=0

for example in "${examples[@]}"; do
    if [ -f "$EXAMPLES_DIR/$example" ]; then
        echo -n "Testing $example... "
        
        # Special handling for group_by_aggregates.sql
        if [ "$example" = "group_by_aggregates.sql" ] && [ -n "$GROUP_BY_DATA" ]; then
            if $SQL_CLI "$GROUP_BY_DATA" -f "$EXAMPLES_DIR/$example" -o csv > /dev/null 2>&1; then
                echo -e "${GREEN}✓ PASSED${NC}"
                ((passed++))
            else
                echo -e "${RED}✗ FAILED${NC}"
                ((failed++))
                echo "  Error details:"
                $SQL_CLI "$GROUP_BY_DATA" -f "$EXAMPLES_DIR/$example" -o csv 2>&1 | grep -E "Error|Failed" | head -3
            fi
        # Special handling for window function examples
        elif [[ "$example" = "window_functions.sql" || "$example" = "window_functions_filtering.sql" ]] && [ -n "$WINDOW_DATA" ]; then
            if $SQL_CLI "$WINDOW_DATA" -f "$EXAMPLES_DIR/$example" -o csv > /dev/null 2>&1; then
                echo -e "${GREEN}✓ PASSED${NC}"
                ((passed++))
            else
                echo -e "${RED}✗ FAILED${NC}"
                ((failed++))
                echo "  Error details:"
                $SQL_CLI "$WINDOW_DATA" -f "$EXAMPLES_DIR/$example" -o csv 2>&1 | grep -E "Error|Failed" | head -3
            fi
        else
            # Run the example normally
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
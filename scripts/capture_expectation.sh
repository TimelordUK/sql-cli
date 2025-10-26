#!/bin/bash
# Helper script to capture expected output for an example
#
# Usage:
#   ./scripts/capture_expectation.sh physics_astronomy_showcase
#   ./scripts/capture_expectation.sh qualified_names
#
# This runs the example and saves the JSON output to examples/expectations/

set -e

if [ $# -ne 1 ]; then
    echo "Usage: $0 <example_name>"
    echo "Example: $0 physics_astronomy_showcase"
    exit 1
fi

EXAMPLE_NAME="$1"
CLI="./target/release/sql-cli"
EXAMPLES_DIR="examples"
EXPECTATIONS_DIR="examples/expectations"
SQL_FILE="$EXAMPLES_DIR/${EXAMPLE_NAME}.sql"
EXPECTATION_FILE="$EXPECTATIONS_DIR/${EXAMPLE_NAME}.json"

# Check if SQL file exists
if [ ! -f "$SQL_FILE" ]; then
    echo "ERROR: Example file not found: $SQL_FILE"
    exit 1
fi

# Check if CLI binary exists
if [ ! -f "$CLI" ]; then
    echo "ERROR: $CLI not found. Run 'cargo build --release' first."
    exit 1
fi

# Create expectations directory if it doesn't exist
mkdir -p "$EXPECTATIONS_DIR"

echo "Capturing output for: $EXAMPLE_NAME"
echo "Running: $CLI -f $SQL_FILE -o json"
echo

# Run and capture output
if output=$($CLI -f "$SQL_FILE" -o json 2>&1); then
    # Validate it's valid JSON and pretty-print
    if echo "$output" | jq -S . > "$EXPECTATION_FILE"; then
        echo "✓ Expectation captured successfully!"
        echo "  Saved to: $EXPECTATION_FILE"
        echo
        echo "Preview (first 20 lines):"
        head -20 "$EXPECTATION_FILE"
        echo
        echo "To test: ./tests/integration/test_examples.sh $EXAMPLE_NAME"
    else
        echo "ERROR: Output is not valid JSON"
        echo "$output"
        exit 1
    fi
else
    echo "ERROR: Query failed"
    echo "$output"
    exit 1
fi

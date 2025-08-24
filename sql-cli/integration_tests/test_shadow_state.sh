#!/bin/bash

echo "Testing Shadow State Manager"
echo "============================"
echo ""
echo "Starting application with shadow state logging enabled"
echo ""
echo "Test procedure:"
echo "1. Load file - should see state transition logged"
echo "2. Press Enter on query - should see Command -> Results transition"
echo "3. Press '/' to search - should see search start logged"
echo "4. Press Escape - should see search end and return to Results"
echo "5. Watch status line for [Shadow: STATE] display"
echo ""

# Run with shadow state feature and debug logging
RUST_LOG=shadow_state=info,state=info ./target/release/sql-cli test_shadow.csv --classic
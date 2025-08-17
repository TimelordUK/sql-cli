#!/bin/bash

echo "Testing G command crosshair highlighting"
echo "========================================"
echo ""

# Create test CSV with manageable number of rows for debugging
cat > test_g_crosshair.csv << 'EOF'
id,name,status
1,Alice,active
2,Bob,pending
3,Charlie,active
4,Diana,inactive
5,Eve,active
6,Frank,pending
7,Grace,active
8,Henry,inactive
9,Iris,active
10,Jack,pending
11,Karen,active
12,Larry,inactive
13,Mary,active
14,Nancy,pending
15,Oscar,active
16,Paul,inactive
17,Quinn,active
18,Rita,pending
19,Sam,active
20,Tina,inactive
EOF

echo "Test data created: test_g_crosshair.csv (20 rows)"
echo ""
echo "Running with crosshair debug logging:"
echo ""

# Run with specific crosshair logging
RUST_LOG=crosshair=debug timeout 3 ./target/release/sql-cli test_g_crosshair.csv -e "select * from data" --classic

echo ""
echo "Test completed. Check debug output above for crosshair highlighting behavior."
echo ""
echo "Expected:"
echo "1. Initial load should highlight row 0"
echo "2. Pressing G should scroll to last page and highlight row 19"
echo "3. Debug should show: 'selected_row=19, is_current=true' for the last row"
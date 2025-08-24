#!/bin/bash

echo "Manual test instructions for fuzzy filter clear bug:"
echo "=================================================="
echo ""
echo "1. Run: ./target/release/sql-cli test_fuzzy_demo.csv -e 'select * from data'"
echo "2. Verify you see 'Rows: 10' in the status bar"
echo "3. Press Shift+F to enter fuzzy filter"
echo "4. Type 'rejected' and press Enter"
echo "5. Verify you see 'Rows: 3' (filtered results)"
echo "6. Press Shift+F again to re-enter fuzzy filter"
echo "7. Press Enter immediately (with empty input)"
echo "8. EXPECTED: Should see 'Rows: 10' again (all rows restored)"
echo ""
echo "If the bug is fixed, you'll see all 10 rows after step 7."
echo "If the bug still exists, you'll still see only 3 rows."
echo ""
echo "Press Enter to start the test..."
read

./target/release/sql-cli test_fuzzy_demo.csv -e "select * from data"
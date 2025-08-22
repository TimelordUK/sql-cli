#!/bin/bash
# Test script for yank chord operations using the new Action system

set -e

echo "==================================="
echo "Yank Chord Actions Test"
echo "==================================="
echo ""

# Create test data
cat > test_yank_chords.csv << 'EOF'
id,name,department,salary,status
1,Alice,Engineering,120000,Active
2,Bob,Marketing,95000,Active
3,Charlie,Engineering,110000,Inactive
4,David,Sales,85000,Active
5,Eve,Engineering,125000,Active
EOF

echo "✅ Test data created: test_yank_chords.csv"
echo ""
echo "🎯 YANK CHORD FEATURES TO TEST:"
echo "================================"
echo ""
echo "1. YANK ROW (yy or yr):"
echo "   - Navigate to a row"
echo "   - Press 'yy' or 'yr'"
echo "   - Should copy entire row to clipboard"
echo ""
echo "2. YANK COLUMN (yc):"
echo "   - Navigate to any cell in a column"
echo "   - Press 'yc'"
echo "   - Should copy entire column data"
echo ""
echo "3. YANK ALL (ya):"
echo "   - Press 'ya' from anywhere"
echo "   - Should copy all data"
echo ""
echo "4. YANK CELL (yv):"
echo "   - Navigate to a specific cell"
echo "   - Press 'yv'"
echo "   - Should copy just that cell value"
echo ""
echo "5. YANK QUERY (yq):"
echo "   - Press 'yq'"
echo "   - Should copy the current SQL query"
echo ""
echo "6. CHORD STATUS:"
echo "   - After pressing 'y', status should show 'Yank mode: y=row, c=column, a=all, ESC=cancel'"
echo "   - After completing a chord, should show what was yanked"
echo ""
echo "7. KEY INDICATOR:"
echo "   - Press F12 to enable key indicator if not visible"
echo "   - Should show 'y(a,c,q,r,v)' when in yank mode"
echo ""
echo "Running: ./target/release/sql-cli test_yank_chords.csv"
echo ""
echo "Expected behavior:"
echo "- All yank operations through chord system using Actions"
echo "- No more string-based action handling"
echo "- Proper status messages for each yank type"
echo ""
echo "Press Ctrl+C to exit the application"
#!/bin/bash

echo "Testing enhanced navigation features..."

# Create test CSV
cat > test_data.csv << EOF
id,name,age,department
1,Alice,30,Engineering
2,Bob,25,Sales
3,Charlie,35,Marketing
4,David,28,Engineering
EOF

echo "Test CSV created as test_data.csv"
echo ""
echo "To test the features:"
echo "1. Run: cargo run -- test_data.csv"
echo "2. Type: SELECT * FROM <TAB> (should complete to test_data)"
echo "3. Use Alt+[ and Alt+] to jump between SQL tokens"
echo "4. Use Ctrl+W to delete word backward, Alt+D to delete forward"
echo "5. Use Ctrl+K to kill line, Ctrl+U to kill backward, Ctrl+Y to yank"
echo "6. Use Ctrl+Z to undo"
echo "7. Check the status bar - it should show Token position and current token"
echo ""
echo "In results mode, the status bar will show Row X/Y and filter status"
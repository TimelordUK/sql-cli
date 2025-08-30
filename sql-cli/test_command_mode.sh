#!/bin/bash
# Test script for command mode

# Create a test CSV file
cat > test_data.csv << EOF
id,name,value
1,Alice,100
2,Bob,200
3,Charlie,300
EOF

echo "Testing command mode text editing..."
echo "Commands to test:"
echo "1. Press 'a' to enter command mode with append"
echo "2. Type 'SELECT * FROM data'"
echo "3. Use Ctrl+A to go to start"
echo "4. Use Ctrl+E to go to end"
echo "5. Use Ctrl+W to delete word backward"
echo "6. Use Alt+B/F to move by word"
echo "7. Use Tab for completion"
echo ""
echo "Run: ./target/release/sql-cli test_data.csv"
echo ""
echo "If everything works, you should be able to:"
echo "- Type text normally"
echo "- Navigate with Ctrl+A/E"
echo "- Delete words with Ctrl+W"
echo "- Move by words with Alt+B/F"
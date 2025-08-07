#!/bin/bash

# Test fuzzy filter functionality
echo "Testing fuzzy filter functionality..."

# Create test data
cat > test_fuzzy.json << 'EOF'
[
  {"id": 1, "name": "Alice Johnson", "status": "active"},
  {"id": 2, "name": "Bob Smith", "status": "pending"},
  {"id": 3, "name": "Charlie Brown", "status": "active"},
  {"id": 4, "name": "David Wilson", "status": "inactive"},
  {"id": 5, "name": "Eve Davis", "status": "pending"}
]
EOF

echo "Test data created in test_fuzzy.json"
echo ""
echo "To test fuzzy filter:"
echo "1. Run: ./target/release/sql-cli test_fuzzy.json"
echo "2. Wait for data to load"
echo "3. Press 'f' to enter fuzzy filter mode"
echo "4. Type a filter pattern (e.g., 'pen' to match 'pending')"
echo "5. Press Enter to apply the filter"
echo "6. Press 'f' again to clear the filter"
echo "7. Check that the status line updates correctly"
echo ""
echo "Expected behavior:"
echo "- When filter is active, status line should show 'Fuzzy: <pattern>'"
echo "- When filter is cleared (pressing 'f' then Enter), status line should not show 'Fuzzy:'"
echo "- Original SQL query should be preserved throughout"
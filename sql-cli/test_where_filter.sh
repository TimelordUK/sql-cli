#!/bin/bash
# Test script for WHERE clause filtering

cat > test_where_filter.csv << 'EOF'
id,name,age,city,price
1,Alice,30,NYC,99.99
2,Bob,25,London,150.50
3,Charlie,35,Paris,75.00
4,David,28,Berlin,120.75
5,Eve,32,Tokyo,200.00
6,Frank,45,NYC,95.50
7,Grace,29,London,180.25
8,Henry,31,Paris,110.00
EOF

echo "Testing WHERE Clause Filtering"
echo "==============================="
echo ""
echo "Test Cases:"
echo "1. SELECT * FROM data WHERE age > 30"
echo "   - Should return 4 rows (Charlie, Eve, Frank, Henry)"
echo ""
echo "2. SELECT * FROM data WHERE city = 'NYC'"
echo "   - Should return 2 rows (Alice, Frank)"
echo ""
echo "3. SELECT * FROM data WHERE price > 100 AND age < 35"
echo "   - Should return 3 rows (Bob, David, Eve)"
echo ""
echo "4. SELECT * FROM data WHERE name.contains('a')"
echo "   - Should return 4 rows (Charlie, David, Frank, Grace)"
echo ""
echo "5. SELECT * FROM data WHERE price.contains('.')"
echo "   - Should return all 8 rows (testing numeric to string coercion)"
echo ""
echo "Starting SQL-CLI with debug logging..."

RUST_LOG=sql_cli::data::query_engine=debug,sql_cli::data::recursive_where_evaluator=debug ./target/release/sql-cli test_where_filter.csv
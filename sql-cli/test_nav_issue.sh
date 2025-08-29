#!/bin/bash

echo "Testing column 20 -> 21 navigation issue..."

# Create test data
cat > test_nav_issue.csv << 'EOF'
c0,c1,c2,c3,c4,c5,c6,c7,c8,c9,c10,c11,c12,c13,c14,c15,c16,c17,c18,c19,c20,c21,c22,c23,c24,c25
a0,a1,a2,a3,a4,a5,a6,a7,a8,a9,a10,a11,a12,a13,a14,a15,a16,a17,a18,a19,a20,a21,a22,a23,a24,a25
EOF

# Navigate to column 20 then press l once more
nav_sequence=""
for i in {1..20}; do
    nav_sequence="${nav_sequence}l"
done
nav_sequence="${nav_sequence}lq"  # One more l, then quit

echo -e "$nav_sequence" | RUST_LOG=sql_cli::ui::viewport_manager=debug,sql_cli::ui::traits::column_ops=debug timeout 2 ./target/release/sql-cli test_nav_issue.csv -e "select * from data" 2>&1 | grep -E "(CRITICAL DEBUG|COLUMN_OPS|Input current_display|navigation result|applying result)" | tail -20

rm test_nav_issue.csv
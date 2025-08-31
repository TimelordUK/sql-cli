#!/bin/bash

# Create a test CSV with known "emerging" locations
cat > test_emerging.csv << 'EOF'
id,book,status,value
1,Derivatives,active,100
2,Options Trading,pending,200
3,Futures Trading,active,300
4,ETF Trading,pending,400
5,Options Trading,active,500
6,Emerging Markets,pending,600
7,Fixed Income,active,700
8,Commodities,emerging,800
9,Emerging Markets,active,900
10,Derivatives,emerging,1000
EOF

echo "Test file created with 'emerging' at:"
echo "  Row 6, Column 2: 'Emerging Markets'"
echo "  Row 8, Column 3: 'emerging' (in status)"
echo "  Row 9, Column 2: 'Emerging Markets'"
echo "  Row 10, Column 3: 'emerging' (in status)"
echo ""
echo "Running search test..."

RUST_LOG=search=info ./target/release/sql-cli test_emerging.csv -e "select * from data" 2>&1 | grep -E "SEARCH START|Pattern:|Data dimensions:|SearchManager found|FIRST MATCH|Match #|VALUE AT NAVIGATION|NAVIGATION START|Set (row|column)|Set crosshair" | head -30
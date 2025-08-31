#!/bin/bash

echo "Testing ViewportManager crosshair management"
echo "============================================="
echo ""
echo "This test verifies that crosshair is managed by ViewportManager"
echo "in visual coordinates and displayed correctly in status line."
echo ""
echo "Test steps:"
echo "1. Create test data with many columns"
echo "2. Hide some columns to test visual vs DataTable indices"
echo "3. Navigate with h/l/j/k to verify crosshair updates"
echo "4. Check status line shows [V:row,col] with visual coordinates"
echo ""

# Create test CSV
cat > test_crosshair.csv << 'EOF'
a,b,c,d,e,f,g,h,i,j,k,l,m,n,o,p,q,r,s,t,u,v,w,x,y,z
1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26
2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27
3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28
4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29
5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30
EOF

echo "Test data created: test_crosshair.csv"
echo ""
echo "Running with debug logging to see ViewportManager crosshair updates..."
echo ""
echo "Commands to test:"
echo "  1. Press 'H' and hide column 'c' (index 2)"
echo "  2. Press 'H' and hide column 'f' (index 5)"
echo "  3. Navigate with h/l keys - should skip hidden columns"
echo "  4. Navigate with j/k keys - should move rows"
echo "  5. Check status line shows [V:row,col] with visual coordinates"
echo "  6. Press F5 to see debug info with ViewportManager crosshair"
echo ""

RUST_LOG=sql_cli::ui::viewport_manager=debug ./target/release/sql-cli test_crosshair.csv -e "select * from data"
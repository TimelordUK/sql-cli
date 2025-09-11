#!/bin/bash

# Test the Neovim plugin schema display

# Create a test CSV with 15+ columns to test the color consistency
cat > /tmp/test_schema.csv << 'EOF'
col1,col2,col3,col4,col5,col6,col7,col8,col9,col10,col11,col12,col13,col14,col15
1,2,3,4,5,6,7,8,9,10,11,12,13,14,15
a,b,c,d,e,f,g,h,i,j,k,l,m,n,o
EOF

echo "Test CSV created at /tmp/test_schema.csv"
echo ""
echo "To test the fix in Neovim:"
echo "1. Open Neovim: nvim /tmp/test_schema.csv"
echo "2. Set the data file: :SqlCliSetData /tmp/test_schema.csv"
echo "3. Show schema: \\sh (or press <leader>sh)"
echo ""
echo "All column numbers (1-15) should now have consistent coloring."
echo "The numbers should all appear in the same color (typically cyan/blue for Number highlight)."
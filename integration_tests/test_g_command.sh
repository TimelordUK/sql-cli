#!/bin/bash

echo "Testing G command fix"
echo "===================="
echo ""

# Create test CSV with many rows
cat > test_g_fix.csv << 'EOF'
id,name,value
1,Row1,100
2,Row2,200
3,Row3,300
4,Row4,400
5,Row5,500
6,Row6,600
7,Row7,700
8,Row8,800
9,Row9,900
10,Row10,1000
11,Row11,1100
12,Row12,1200
13,Row13,1300
14,Row14,1400
15,Row15,1500
16,Row16,1600
17,Row17,1700
18,Row18,1800
19,Row19,1900
20,Row20,2000
21,Row21,2100
22,Row22,2200
23,Row23,2300
24,Row24,2400
25,Row25,2500
26,Row26,2600
27,Row27,2700
28,Row28,2800
29,Row29,2900
30,Row30,3000
31,Row31,3100
32,Row32,3200
33,Row33,3300
34,Row34,3400
35,Row35,3500
36,Row36,3600
37,Row37,3700
38,Row38,3800
39,Row39,3900
40,Row40,4000
41,Row41,4100
42,Row42,4200
43,Row43,4300
44,Row44,4400
45,Row45,4500
46,Row46,4600
47,Row47,4700
48,Row48,4800
49,Row49,4900
50,Row50,5000
51,Row51,5100
52,Row52,5200
53,Row53,5300
54,Row54,5400
55,Row55,5500
56,Row56,5600
57,Row57,5700
58,Row58,5800
59,Row59,5900
60,Row60,6000
61,Row61,6100
62,Row62,6200
63,Row63,6300
64,Row64,6400
65,Row65,6500
66,Row66,6600
67,Row67,6700
68,Row68,6800
69,Row69,6900
70,Row70,7000
71,Row71,7100
72,Row72,7200
73,Row73,7300
74,Row74,7400
75,Row75,7500
76,Row76,7600
77,Row77,7700
78,Row78,7800
79,Row79,7900
80,Row80,8000
81,Row81,8100
82,Row82,8200
83,Row83,8300
84,Row84,8400
85,Row85,8500
86,Row86,8600
87,Row87,8700
88,Row88,8800
89,Row89,8900
90,Row90,9000
91,Row91,9100
92,Row92,9200
93,Row93,9300
94,Row94,9400
95,Row95,9500
96,Row96,9600
97,Row97,9700
98,Row98,9800
99,Row99,9900
100,Row100,10000
EOF

echo "Test data created: test_g_fix.csv (100 rows)"
echo ""
echo "To test G command manually:"
echo "  ./target/release/sql-cli test_g_fix.csv -e \"select * from data\""
echo ""
echo "Expected behavior:"
echo "1. Start with first page (rows 1-79)"
echo "2. Press 'G' to jump to last row"
echo "3. Should show last page (rows ~22-100) with crosshair on row 100"
echo "4. Status should show [V:99,0 @ 79r] or similar"
echo ""
echo "✅ FIXED: NavigationState.scroll_offset now synchronized with ViewportManager"
echo "✅ The G command should now correctly scroll to the last page"
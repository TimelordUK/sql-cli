#!/bin/bash

echo "Testing viewport scrolling with size display"
echo "============================================"
echo ""
echo "This test verifies that:"
echo "1. Viewport size is displayed in status line"
echo "2. Scrolling only occurs when crosshair reaches viewport edge"
echo ""

# Create test CSV with many rows to test scrolling
cat > test_viewport.csv << 'EOF'
id,name,value,status,category
1,Row1,100,active,A
2,Row2,200,active,B
3,Row3,300,inactive,A
4,Row4,400,active,C
5,Row5,500,pending,B
6,Row6,600,active,A
7,Row7,700,inactive,C
8,Row8,800,active,B
9,Row9,900,pending,A
10,Row10,1000,active,C
11,Row11,1100,inactive,B
12,Row12,1200,active,A
13,Row13,1300,pending,C
14,Row14,1400,active,B
15,Row15,1500,inactive,A
16,Row16,1600,active,C
17,Row17,1700,pending,B
18,Row18,1800,active,A
19,Row19,1900,inactive,C
20,Row20,2000,active,B
21,Row21,2100,pending,A
22,Row22,2200,active,C
23,Row23,2300,inactive,B
24,Row24,2400,active,A
25,Row25,2500,pending,C
26,Row26,2600,active,B
27,Row27,2700,inactive,A
28,Row28,2800,active,C
29,Row29,2900,pending,B
30,Row30,3000,active,A
31,Row31,3100,inactive,C
32,Row32,3200,active,B
33,Row33,3300,pending,A
34,Row34,3400,active,C
35,Row35,3500,inactive,B
36,Row36,3600,active,A
37,Row37,3700,pending,C
38,Row38,3800,active,B
39,Row39,3900,inactive,A
40,Row40,4000,active,C
41,Row41,4100,pending,B
42,Row42,4200,active,A
43,Row43,4300,inactive,C
44,Row44,4400,active,B
45,Row45,4500,pending,A
46,Row46,4600,active,C
47,Row47,4700,inactive,B
48,Row48,4800,active,A
49,Row49,4900,pending,C
50,Row50,5000,active,B
51,Row51,5100,inactive,A
52,Row52,5200,active,C
53,Row53,5300,pending,B
54,Row54,5400,active,A
55,Row55,5500,inactive,C
56,Row56,5600,active,B
57,Row57,5700,pending,A
58,Row58,5800,active,C
59,Row59,5900,inactive,B
60,Row60,6000,active,A
61,Row61,6100,pending,C
62,Row62,6200,active,B
63,Row63,6300,inactive,A
64,Row64,6400,active,C
65,Row65,6500,pending,B
66,Row66,6600,active,A
67,Row67,6700,inactive,C
68,Row68,6800,active,B
69,Row69,6900,pending,A
70,Row70,7000,active,C
71,Row71,7100,inactive,B
72,Row72,7200,active,A
73,Row73,7300,pending,C
74,Row74,7400,active,B
75,Row75,7500,inactive,A
76,Row76,7600,active,C
77,Row77,7700,pending,B
78,Row78,7800,active,A
79,Row79,7900,inactive,C
80,Row80,8000,active,B
81,Row81,8100,pending,A
82,Row82,8200,active,C
83,Row83,8300,inactive,B
84,Row84,8400,active,A
85,Row85,8500,pending,C
86,Row86,8600,active,B
87,Row87,8700,inactive,A
88,Row88,8800,active,C
89,Row89,8900,pending,B
90,Row90,9000,active,A
91,Row91,9100,inactive,C
92,Row92,9200,active,B
93,Row93,9300,pending,A
94,Row94,9400,active,C
95,Row95,9500,inactive,B
96,Row96,9600,active,A
97,Row97,9700,pending,C
98,Row98,9800,active,B
99,Row99,9900,inactive,A
100,Row100,10000,active,C
EOF

echo "Test data created: test_viewport.csv (100 rows)"
echo ""
echo "To test vertical scrolling:"
echo "  ./target/release/sql-cli test_viewport.csv"
echo ""
echo "Instructions:"
echo "1. Note the viewport size shown in status line: [V:0,0 @ XXr]"
echo "2. Press 'j' repeatedly to move down"
echo "3. Verify crosshair moves within viewport first"
echo "4. Scrolling should start when row = viewport_size - 1"
echo "5. Use F5 to see debug info for verification"
echo ""
echo "Expected with recent fixes:"
echo "- Status shows: [V:row,col @ viewport_height]"
echo "- Scrolling only at viewport edge (not at row 71)"
echo "- Crosshair highlight visible from start"
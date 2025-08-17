#!/bin/bash

echo "Testing Viewport Lock (Ctrl+Space) vs Cursor Lock (x)"
echo "======================================================"
echo ""

# Create test CSV with enough rows to test scrolling
cat > test_viewport_lock.csv << 'EOF'
id,name,value,status
1,Row001,100,active
2,Row002,200,pending
3,Row003,300,active
4,Row004,400,inactive
5,Row005,500,active
6,Row006,600,pending
7,Row007,700,active
8,Row008,800,inactive
9,Row009,900,active
10,Row010,1000,pending
11,Row011,1100,active
12,Row012,1200,inactive
13,Row013,1300,active
14,Row014,1400,pending
15,Row015,1500,active
16,Row016,1600,inactive
17,Row017,1700,active
18,Row018,1800,pending
19,Row019,1900,active
20,Row020,2000,inactive
21,Row021,2100,active
22,Row022,2200,pending
23,Row023,2300,active
24,Row024,2400,inactive
25,Row025,2500,active
26,Row026,2600,pending
27,Row027,2700,active
28,Row028,2800,inactive
29,Row029,2900,active
30,Row030,3000,pending
31,Row031,3100,active
32,Row032,3200,inactive
33,Row033,3300,active
34,Row034,3400,pending
35,Row035,3500,active
36,Row036,3600,inactive
37,Row037,3700,active
38,Row038,3800,pending
39,Row039,3900,active
40,Row040,4000,inactive
41,Row041,4100,active
42,Row042,4200,pending
43,Row043,4300,active
44,Row044,4400,inactive
45,Row045,4500,active
46,Row046,4600,pending
47,Row047,4700,active
48,Row048,4800,inactive
49,Row049,4900,active
50,Row050,5000,pending
51,Row051,5100,active
52,Row052,5200,inactive
53,Row053,5300,active
54,Row054,5400,pending
55,Row055,5500,active
56,Row056,5600,inactive
57,Row057,5700,active
58,Row058,5800,pending
59,Row059,5900,active
60,Row060,6000,inactive
61,Row061,6100,active
62,Row062,6200,pending
63,Row063,6300,active
64,Row064,6400,inactive
65,Row065,6500,active
66,Row066,6600,pending
67,Row067,6700,active
68,Row068,6800,inactive
69,Row069,6900,active
70,Row070,7000,pending
71,Row071,7100,active
72,Row072,7200,inactive
73,Row073,7300,active
74,Row074,7400,pending
75,Row075,7500,active
76,Row076,7600,inactive
77,Row077,7700,active
78,Row078,7800,pending
79,Row079,7900,active
80,Row080,8000,inactive
81,Row081,8100,active
82,Row082,8200,pending
83,Row083,8300,active
84,Row084,8400,inactive
85,Row085,8500,active
86,Row086,8600,pending
87,Row087,8700,active
88,Row088,8800,inactive
89,Row089,8900,active
90,Row090,9000,pending
91,Row091,9100,active
92,Row092,9200,inactive
93,Row093,9300,active
94,Row094,9400,pending
95,Row095,9500,active
96,Row096,9600,inactive
97,Row097,9700,active
98,Row098,9800,pending
99,Row099,9900,active
100,Row100,10000,inactive
EOF

echo "Test data created: test_viewport_lock.csv (100 rows)"
echo ""
echo "Test 1: CURSOR LOCK (x key)"
echo "----------------------------"
echo "1. Press 'x' to enable cursor lock"
echo "2. Press 'j' multiple times - data should scroll under cursor"
echo "3. Cursor should stay at same viewport position"
echo "4. Press 'x' again to disable"
echo ""
echo "Test 2: VIEWPORT LOCK (Ctrl+Space)"
echo "-----------------------------------"
echo "1. Press Ctrl+Space to enable viewport lock"
echo "2. Press 'j' multiple times - cursor moves within viewport only"
echo "3. Scrolling should be prevented - viewport stays fixed"
echo "4. Cursor cannot go beyond current viewport bounds"
echo "5. Press Ctrl+Space again to disable"
echo ""
echo "Manual test:"
echo "  ./target/release/sql-cli test_viewport_lock.csv -e \"select * from data\""
echo ""
echo "Key differences:"
echo "  x (cursor lock): Cursor fixed, data scrolls"
echo "  Ctrl+Space (viewport lock): Viewport fixed, cursor moves within bounds"
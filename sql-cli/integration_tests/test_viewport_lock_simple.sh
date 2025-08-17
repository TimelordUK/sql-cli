#!/bin/bash

# Create simple test data
cat > test_viewport_simple.csv << 'EOF'
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
EOF

echo "Testing viewport lock with debug logging..."
RUST_LOG=sql_cli::ui::viewport_manager=debug timeout 10 ./target/release/sql-cli test_viewport_simple.csv -e "select * from data" 2>&1 | grep -E "viewport_lock|Viewport lock|viewport.*boundaries" | head -20
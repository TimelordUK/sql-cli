#!/bin/bash

# Test Window Function Registry Syntactic Sugar
echo "=== Window Function Registry Syntactic Sugar Test ==="
echo ""

# Test MOVING_AVG
echo "1. MOVING_AVG vs AVG with ROWS PRECEDING:"
echo "----------------------------------------"
echo "Syntactic Sugar:"
./target/release/sql-cli data/AAPL_data.csv -q "SELECT date, close, RENDER_NUMBER(MOVING_AVG(close, 5) OVER (ORDER BY date), 'standard', 2) as ma_5 FROM AAPL_data ORDER BY date LIMIT 5" -o csv

echo ""
echo "Verbose:"
./target/release/sql-cli data/AAPL_data.csv -q "SELECT date, close, RENDER_NUMBER(AVG(close) OVER (ORDER BY date ROWS 4 PRECEDING), 'standard', 2) as ma_5 FROM AAPL_data ORDER BY date LIMIT 5" -o csv

echo ""
echo "2. CUMULATIVE_SUM vs SUM with UNBOUNDED PRECEDING:"
echo "---------------------------------------------------"
echo "Syntactic Sugar:"
./target/release/sql-cli data/AAPL_data.csv -q "SELECT date, volume, CUMULATIVE_SUM(volume) OVER (ORDER BY date) as cumul_vol FROM AAPL_data ORDER BY date LIMIT 5" -o csv

echo ""
echo "Verbose:"
./target/release/sql-cli data/AAPL_data.csv -q "SELECT date, volume, SUM(volume) OVER (ORDER BY date ROWS UNBOUNDED PRECEDING) as cumul_vol FROM AAPL_data ORDER BY date LIMIT 5" -o csv

echo ""
echo "3. Z_SCORE calculation:"
echo "-----------------------"
echo "Syntactic Sugar:"
./target/release/sql-cli data/AAPL_data.csv -q "SELECT date, close, RENDER_NUMBER(Z_SCORE(close, 20) OVER (ORDER BY date), 'standard', 2) as z_score FROM AAPL_data WHERE date >= '2013-04-01' ORDER BY date LIMIT 5" -o csv

echo ""
echo "Manual calculation (verbose):"
./target/release/sql-cli data/AAPL_data.csv -q "WITH stats AS (SELECT date, close, AVG(close) OVER (ORDER BY date ROWS 19 PRECEDING) as ma_20, STDDEV(close) OVER (ORDER BY date ROWS 19 PRECEDING) as stddev_20 FROM AAPL_data) SELECT date, close, RENDER_NUMBER((close - ma_20) / stddev_20, 'standard', 2) as z_score FROM stats WHERE date >= '2013-04-01' ORDER BY date LIMIT 5" -o csv

echo ""
echo "=== All tests complete! ==="
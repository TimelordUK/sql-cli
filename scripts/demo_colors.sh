#!/bin/bash
# Colorful Number Classification Demos
# Showcases ANSI color functions with prime numbers and other interesting properties

echo "🎨 SQL-CLI Colorful Number Demos 🎨"
echo ""
echo "Running all color demos from examples/color_numbers.sql..."
echo ""

./target/release/sql-cli -f examples/color_numbers.sql -o table

echo ""
echo "✨ Demo complete! ✨"
echo ""
echo "Try these individual demos:"
echo "  1. Twin primes:       ./target/release/sql-cli data/numbers_1_100.csv -q \"SELECT ... (Query 2)\""
echo "  2. Number rainbow:    ./target/release/sql-cli data/numbers_1_100.csv -q \"SELECT ... (Query 5)\""
echo "  3. Prime density:     ./target/release/sql-cli data/numbers_1_100.csv -q \"SELECT ... (Query 3)\""

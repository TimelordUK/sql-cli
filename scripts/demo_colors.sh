#!/bin/bash
# Colorful Number Classification Demos
# Showcases ANSI color functions with prime numbers and other interesting properties

DATA_FILE="data/numbers_1_100.csv"
SCRIPT_FILE="examples/color_numbers.sql"

# Function to run a specific demo
run_demo() {
    local demo_num=$1
    echo ""
    echo "════════════════════════════════════════════════════════════════"
    case $demo_num in
        1) echo "Demo 1: Comprehensive Number Classification" ;;
        2) echo "Demo 2: Twin Primes Showcase" ;;
        3) echo "Demo 3: Prime Density Visualization" ;;
        4) echo "Demo 4: Perfect Squares with Colors" ;;
        5) echo "Demo 5: Number Rainbow (1-100)" ;;
        6) echo "Demo 6: Number Properties Matrix" ;;
    esac
    echo "════════════════════════════════════════════════════════════════"
    ./target/release/sql-cli "$DATA_FILE" -f "$SCRIPT_FILE" --execute-statement "$demo_num" -o table
}

# Check if a demo number was provided
if [ $# -eq 1 ]; then
    demo_num=$1
    if [[ "$demo_num" =~ ^[1-6]$ ]]; then
        run_demo "$demo_num"
        exit 0
    else
        echo "Error: Demo number must be between 1 and 6"
        exit 1
    fi
fi

# No arguments - show menu
echo "🎨 SQL-CLI Colorful Number Demos 🎨"
echo ""
echo "Available demos:"
echo "  1. Comprehensive Classification - Twin primes, primes, squares color-coded"
echo "  2. Twin Primes Showcase - Prime pairs (3&5, 11&13) with statistics"
echo "  3. Prime Density Visualization - Bar chart showing primes per decade"
echo "  4. Perfect Squares - Elegant display with visual indicators"
echo "  5. Number Rainbow (1-100) - All numbers color-coded by property"
echo "  6. Properties Matrix - Multi-property checkmark table"
echo ""
echo "Usage:"
echo "  $0           - Run all demos"
echo "  $0 <1-6>     - Run a specific demo"
echo ""
echo "Examples:"
echo "  $0 2         - Show twin primes only"
echo "  $0 5         - Show number rainbow"
echo ""

read -p "Press Enter to run all demos, or Ctrl+C to exit..."

echo ""
echo "Running all 6 color demos..."
echo ""

for i in {1..6}; do
    run_demo "$i"
    echo ""
done

echo "✨ All demos complete! ✨"
echo ""
echo "Color scheme:"
echo "  🔴 RED (bold)  - Twin Primes (primes differing by 2)"
echo "  🟡 GOLD        - Regular Primes"
echo "  🔵 CYAN        - Perfect Squares (1, 4, 9, 16...)"
echo "  ⚫ GRAY        - Even Numbers"
echo "  ⚪ WHITE       - Odd Composites"

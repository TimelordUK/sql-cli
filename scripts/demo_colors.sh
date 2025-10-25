#!/bin/bash
# Colorful Number Classification Demos
# Showcases ANSI color functions with prime numbers and other interesting properties

DATA_FILE="data/numbers_1_100.csv"
SCRIPT_FILE="examples/color_numbers.sql"
PI_SCRIPT_FILE="examples/pi_digits_colors.sql"

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
        7) echo "Demo 7: Pi Digits Rainbow (First 50 Digits)" ;;
        8) echo "Demo 8: Pi Digit Frequency Analysis (100 Digits)" ;;
        9) echo "Demo 9: Compact Pi Rainbow (100 Digits Flowing)" ;;
        10) echo "Demo 10: Pi Digit Patterns - Repeats & Sequences" ;;
    esac
    echo "════════════════════════════════════════════════════════════════"

    # Use different script file for Pi demos (7-10)
    if [ "$demo_num" -ge 7 ] && [ "$demo_num" -le 10 ]; then
        local pi_statement=$((demo_num - 6))
        ./target/release/sql-cli -f "$PI_SCRIPT_FILE" --execute-statement "$pi_statement" -o table
    else
        ./target/release/sql-cli "$DATA_FILE" -f "$SCRIPT_FILE" --execute-statement "$demo_num" -o table
    fi
}

# Check if a demo number was provided
if [ $# -eq 1 ]; then
    demo_num=$1
    if [[ "$demo_num" =~ ^[1-9]$ ]] || [[ "$demo_num" == "10" ]]; then
        run_demo "$demo_num"
        exit 0
    else
        echo "Error: Demo number must be between 1 and 10"
        exit 1
    fi
fi

# No arguments - show menu
echo "🎨 SQL-CLI Colorful Number Demos 🎨"
echo ""
echo "Available demos:"
echo "  Number Classification (1-6):"
echo "    1. Comprehensive Classification - Twin primes, primes, squares color-coded"
echo "    2. Twin Primes Showcase - Prime pairs (3&5, 11&13) with statistics"
echo "    3. Prime Density Visualization - Bar chart showing primes per decade"
echo "    4. Perfect Squares - Elegant display with visual indicators"
echo "    5. Number Rainbow (1-100) - All numbers color-coded by property"
echo "    6. Properties Matrix - Multi-property checkmark table"
echo ""
echo "  Pi Digits Rainbow (7-10):"
echo "    7. Pi Digits Rainbow - First 50 digits with unique colors per digit"
echo "    8. Pi Frequency Analysis - Digit distribution in first 100 digits"
echo "    9. Compact Pi Rainbow - 100 digits in flowing color format"
echo "   10. Pi Digit Patterns - Find repeats and consecutive sequences"
echo ""
echo "Usage:"
echo "  $0            - Run all demos"
echo "  $0 <1-10>     - Run a specific demo"
echo ""
echo "Examples:"
echo "  $0 2          - Show twin primes only"
echo "  $0 7          - Show colorful Pi digits"
echo "  $0 9          - Show compact Pi rainbow"
echo ""

read -p "Press Enter to run all demos, or Ctrl+C to exit..."

echo ""
echo "Running all 10 color demos..."
echo ""

for i in {1..10}; do
    run_demo "$i"
    echo ""
done

echo "✨ All demos complete! ✨"
echo ""
echo "Color schemes:"
echo ""
echo "Number Classification (Demos 1-6):"
echo "  🔴 RED (bold)  - Twin Primes (primes differing by 2)"
echo "  🟡 GOLD        - Regular Primes"
echo "  🔵 CYAN        - Perfect Squares (1, 4, 9, 16...)"
echo "  ⚫ GRAY        - Even Numbers"
echo "  ⚪ WHITE       - Odd Composites"
echo ""
echo "Pi Digits Rainbow (Demos 7-10):"
echo "  Each digit 0-9 has its own vibrant color:"
echo "  0=Purple, 1=Red, 2=Orange, 3=Gold, 4=Yellow-Green"
echo "  5=Green, 6=Cyan, 7=Blue, 8=Magenta, 9=Hot Pink"

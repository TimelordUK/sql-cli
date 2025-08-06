# VHS script for column statistics feature

Output demos/statistics.gif

Set FontSize 16
Set Width 1100
Set Height 700
Set Theme "Nord"

# Start sql-cli
Type "sql-cli"
Enter
Sleep 1s

# Load dataset
Type ":cache load 7"
Enter
Sleep 2s

# Enter TUI mode
Type ":tui"
Enter
Sleep 2s

# Navigate to a numeric column
Type "\\"
Sleep 500ms
Type "price"
Sleep 1s
Enter
Sleep 1s

# Show statistics
Type "s"
Sleep 3s

# Statistics panel shows:
# - Column name and type
# - Min, Max, Mean, Median
# - Unique values count
# - Null count
# - Distribution histogram

Sleep 2s

# Close statistics
Type "s"
Sleep 1s

# Navigate to another column
Type "\\"
Sleep 500ms
Type "quantity"
Sleep 1s
Enter
Sleep 1s

# Show its statistics
Type "s"
Sleep 3s

# Close
Type "s"
Sleep 1s

# Exit
Type "q"
Sleep 500ms
Type ":exit"
Enter
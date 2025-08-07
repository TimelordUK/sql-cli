# SELECT * Expansion Demo
# Shows: Ctrl+X to expand * to all column names, F9 kill line

Output select-star-expansion.gif

Set FontSize 20
Set Width 1200
Set Height 600
Set Theme "Dracula"

# Start the actual demo
Type "sql-cli data/trades.json"
Enter
Sleep 2s

# Show what we're demonstrating with a comment
Type "# Demo: Expand SELECT * to all columns with Ctrl+X"
Enter
Sleep 2s
Type "# First, let's clear and start fresh..."
Enter
Sleep 1s

# Type and clear to show Ctrl+U works
Type "SELECT * FROM trades WHERE price > 100"
Sleep 1s
Ctrl+U  # Kill backward - this should work in VHS
Sleep 1s

# Type fresh SELECT *
Type "SELECT * FROM trades"
Sleep 1s
Type "  # Now press Ctrl+X to expand..."
Sleep 2s
Backspace 30  # Remove the comment
Ctrl+X
Sleep 2s

# Add WHERE clause
Type " WHERE dealType = 'Swap'"
Enter
Sleep 3s

Type "q"
Sleep 500ms
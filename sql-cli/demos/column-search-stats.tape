# Column Search with Statistics Demo
# Shows: JSON loading, column search, and frequency histogram

Output column-search-stats.gif

Set FontSize 20
Set Width 1200
Set Height 700
Set Theme "Dracula"

# Start the actual demo
Type "sql-cli data/trades.json"
Enter
Sleep 2s

# Show what we're demonstrating
Type "# Demo: Column Search (/) and Statistics (Shift+S)"
Enter
Sleep 2s
Type "# Search for column, then view frequency histogram"
Enter
Sleep 2s


# Show initial query
Type "SELECT * FROM trades"
Enter
Sleep 2s

# Enter column search mode
Type "/"
Sleep 500ms
Type "counterpartyCount"
Enter
Sleep 2s

# Show statistics
Type "S"
Sleep 3s

# Exit stats
Escape
Sleep 1s

# Exit column search
Escape
Sleep 1s

# Show we're back in normal results view
Type "q"
Sleep 500ms
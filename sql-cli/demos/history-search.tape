# SQL History Search Demo  
# Shows: Ctrl+R fuzzy search, smart history with McFly

Output history-search.gif

Set FontSize 20
Set Width 1200
Set Height 700
Set Theme "Dracula"

# Show what we're about to demo
Type "# Demo: Smart SQL History with Ctrl+R"
Enter
Type "# - Fuzzy search through query history"
Enter
Type "# - Context-aware suggestions (McFly algorithm)"
Enter
Type "# - Quick query reuse and modification"
Enter
Sleep 3s
Type "clear"
Enter

# Start the actual demo
Type "sql-cli data/trades.json"
Enter
Sleep 2s

# Run a few queries to build history
Type "SELECT * FROM trades WHERE dealType = 'Spot'"
Enter
Sleep 1s
Type "q"

Type "SELECT dealId, price FROM trades WHERE price > 1000"
Enter
Sleep 1s
Type "q"

Type "SELECT COUNT(*) FROM trades WHERE counterpartyCount > 5"
Enter
Sleep 1s
Type "q"

# Now use Ctrl+R to search history
Ctrl+R
Sleep 1s

# Type to search
Type "price"
Sleep 1s

# Navigate results
Down
Sleep 500ms
Up
Sleep 500ms

# Select one
Enter
Sleep 1s

# Modify and run
Type " ORDER BY price DESC"
Enter
Sleep 2s

Type "q"
Type ":exit"
Enter
# Query Cache Demo
# Shows: Saving queries, listing cache, loading from cache

Output cache-feature.gif

Set FontSize 20
Set Width 1200
Set Height 700
Set Theme "Dracula"

# Start the actual demo
Type "sql-cli data/trades.json"
Enter
Sleep 2s

Type "# Demo: Query Cache - Save and reload results"
Enter
Sleep 2s

# Execute a query
Type "SELECT * FROM trades WHERE price > 5000"
Enter
Sleep 2s

# Save to cache
Type ":cache save expensive_trades"
Enter
Sleep 2s

Type "# Saved! Now let's run another query..."
Enter
Sleep 1s

# Run another query
Type "q"
Sleep 500ms
Type "SELECT COUNT(*) as total, dealType FROM trades GROUP BY dealType"
Enter
Sleep 2s

# Save this one too
Type ":cache save deal_summary"
Enter
Sleep 2s

Type "# List all cached queries:"
Enter
Sleep 1s
Type ":cache list"
Enter
Sleep 3s

Type "# Load cached query by ID:"
Enter
Sleep 1s
Type ":cache load 1"
Enter
Sleep 3s

# Exit
Type "q"
Sleep 500ms
Type ":exit"
Enter
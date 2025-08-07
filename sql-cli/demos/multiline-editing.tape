# Multi-line SQL Editing Demo
# Shows: F3 toggle, auto-formatting, vim navigation

Output multiline-editing.gif

Set FontSize 20
Set Width 1200
Set Height 700
Set Theme "Dracula"

# Show what we're about to demo
Type "# Demo: Multi-line SQL editing with F3"
Enter
Type "# - Toggle between single/multi-line modes"
Enter
Type "# - Auto-formatting of complex queries"
Enter
Type "# - Vim-style navigation (hjkl)"
Enter
Sleep 3s
Type "clear"
Enter

# Start the actual demo
Type "sql-cli data/trades.json"
Enter
Sleep 2s

# Type a complex query in single line
Type "SELECT dealId, dealType, price, quantity, counterpartyCount FROM trades WHERE dealType = 'Swap' AND price > 1000 ORDER BY price DESC"
Sleep 1s

# Toggle to multi-line with F3
F3
Sleep 2s

# Show vim navigation
Type "j"  # Down
Sleep 300ms
Type "k"  # Up
Sleep 300ms
Type "l"  # Right
Sleep 300ms
Type "h"  # Left
Sleep 300ms

# Go to WHERE clause and add condition
Type "j"
Type "j"
Type "A"  # Append at end of line
Type " AND quantity > 10"
Sleep 1s

# Execute with Ctrl+Enter
Ctrl+Enter
Sleep 3s

# Toggle back to single line
Type "q"
Sleep 500ms
F3
Sleep 1s

Type "q"
Sleep 500ms
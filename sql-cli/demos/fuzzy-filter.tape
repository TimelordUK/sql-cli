# VHS script for demonstrating fuzzy filtering and exact matching

Output demos/fuzzy-filter.gif

Set FontSize 16
Set Width 1000
Set Height 600
Set Theme "Monokai"

# Start sql-cli
Type "sql-cli"
Enter
Sleep 1s

# Load cached trade data
Type ":cache load 7"
Enter
Sleep 2s

# Enter TUI mode
Type ":tui"
Enter
Sleep 2s

# Show fuzzy filtering
Type "/"
Sleep 500ms
Type "ubs"
Sleep 2s

# Show how fuzzy matching highlights matches
Type "e"
Sleep 1s
Type "q"
Sleep 1s
Type "t"
Sleep 2s

# Clear and show exact match
Escape
Sleep 1s

Type "/"
Sleep 500ms
Type "'ubseqt"
Sleep 2s

# No matches with exact
Escape
Sleep 1s

Type "/"
Sleep 500ms
Type "'USD"
Sleep 2s

# Shows only exact USD matches
Sleep 2s

# Navigate through filtered results
Type "j" Sleep 200ms
Type "j" Sleep 200ms
Type "j" Sleep 200ms
Type "k" Sleep 200ms

Escape
Sleep 1s

# Exit
Type "q"
Sleep 500ms
Type ":exit"
Enter
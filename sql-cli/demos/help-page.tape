# VHS script demonstrating F1 help page

Output demos/help-page.gif

Set FontSize 16
Set Width 1100
Set Height 700
Set Theme "Dracula"

# Show keypresses
Show

# Start sql-cli
Type "sql-cli ../../data/small-customer.csv"
Enter
Sleep 2s

# Enter TUI mode
Type ":tui"
Enter
Sleep 2s

# Navigate a bit
Type "j" Sleep 200ms
Type "j" Sleep 200ms
Type "l" Sleep 200ms
Type "l" Sleep 200ms
Sleep 1s

# Show :help command directly in TUI
Type ":help"
Enter
Sleep 3s

# Press Escape to exit help
Escape
Sleep 1s

# Back in normal TUI mode
Type "j" Sleep 200ms
Type "k" Sleep 200ms
Sleep 1s

# Show column search
Type "\\"
Sleep 500ms
Type "city"
Sleep 1s
Escape
Sleep 1s

# Show fuzzy filter
Type "/"
Sleep 500ms
Type "york"
Sleep 2s
Escape
Sleep 1s

# Exit
Type "q"
Sleep 500ms
Type ":exit"
Enter
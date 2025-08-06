# VHS script for column search and navigation

Output demos/column-navigation.gif

Set FontSize 16
Set Width 1200
Set Height 700
Set Theme "GitHub Dark"

# Start sql-cli
Type "sql-cli"
Enter
Sleep 1s

# Load wide dataset
Type ":cache load 7"
Enter
Sleep 2s

# Enter TUI mode
Type ":tui"
Enter
Sleep 2s

# Show scrolling through many columns manually
Type "l" Sleep 100ms
Type "l" Sleep 100ms
Type "l" Sleep 100ms
Type "l" Sleep 100ms
Type "l" Sleep 100ms
Sleep 1s

# Go back to start
Type "0"
Sleep 1s

# Use column search to find specific column
Type "\\"
Sleep 500ms
Type "price"
Sleep 1s

# Navigate through matches with Tab
Tab
Sleep 1s
Tab
Sleep 1s

# Select column
Enter
Sleep 2s

# Try another search
Type "\\"
Sleep 500ms
Type "instrument"
Sleep 1s
Tab
Sleep 1s
Enter
Sleep 2s

# Search for trade columns
Type "\\"
Sleep 500ms
Type "trade"
Sleep 1s
Tab
Sleep 500ms
Tab
Sleep 500ms
Tab
Sleep 500ms
Enter
Sleep 2s

# Exit
Type "q"
Sleep 500ms
Type ":exit"
Enter
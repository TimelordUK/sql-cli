# VHS script demonstrating vim-style navigation

Output demos/vim-navigation.gif

Set FontSize 16
Set Width 1000
Set Height 650
Set Theme "Solarized Dark"

# Start sql-cli
Type "sql-cli"
Enter
Sleep 1s

# Create sample data
Type "select * from read_csv('https://raw.githubusercontent.com/mwaskom/seaborn-data/master/iris.csv')"
Enter
Sleep 2s

# Enter TUI mode
Type ":tui"
Enter
Sleep 2s

# Vim navigation basics
Type "j" Sleep 200ms
Type "j" Sleep 200ms
Type "j" Sleep 200ms
Type "k" Sleep 200ms
Type "k" Sleep 200ms
Sleep 1s

Type "l" Sleep 200ms
Type "l" Sleep 200ms
Type "h" Sleep 200ms
Type "h" Sleep 200ms
Sleep 1s

# Jump to start/end
Type "g"
Sleep 1s

Type "G"
Sleep 1s

# Jump to first/last column
Type "0"
Sleep 1s

Type "$"
Sleep 1s

# Page navigation
Type "g"
Sleep 500ms

Ctrl+D
Sleep 1s

Ctrl+U
Sleep 1s

# Search within data
Type "/"
Sleep 500ms
Type "setosa"
Sleep 2s

Escape
Sleep 1s

# Column search
Type "\\"
Sleep 500ms
Type "petal"
Sleep 1s
Tab
Sleep 500ms
Enter
Sleep 1s

# Exit
Type "q"
Sleep 500ms
Type ":exit"
Enter
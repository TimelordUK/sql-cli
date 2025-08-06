# Test :help command in sql-cli

Output demos/test-help.gif

Set FontSize 16
Set Width 1000
Set Height 600
Set Theme "Dracula"

# Show keypresses
Show

# Start sql-cli
Type "./target/release/sql-cli ../../data/small-customer.csv"
Enter
Sleep 2s

# Type :help command
Type ":help"
Enter
Sleep 3s

# Press Escape to exit help
Escape
Sleep 1s

# Try a query
Type "select * from data limit 3"
Enter
Sleep 2s

# Use :exit to quit
Type ":exit"
Enter
Sleep 1s
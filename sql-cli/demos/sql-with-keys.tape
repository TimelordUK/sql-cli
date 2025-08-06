# VHS script showing keypresses on screen

Output demos/sql-with-keys.gif

Set FontSize 16
Set Width 1100
Set Height 600
Set Theme "Dracula"

# Show keypresses in the corner
Show

# Start sql-cli with a CSV file
Type "sql-cli ../../data/small-customer.csv"
Enter
Sleep 2s

# Run initial query
Type "select * from data"
Enter
Sleep 2s

# Press Up arrow to recall last query (will show "↑")
Up
Sleep 1s

# Go to end and add WHERE clause (will show "Ctrl+E")
Ctrl+E
Sleep 500ms
Type " where Coun"
Sleep 500ms

# Tab to autocomplete (will show "Tab")
Tab
Sleep 1s

# Complete the Contains clause
Type "('Pan')"
Sleep 500ms

# Execute the query (will show "Enter")
Enter
Sleep 3s

# Another example with visible keypresses
Up
Sleep 500ms
Ctrl+A
Sleep 500ms
Ctrl+K
Sleep 500ms
Type "select name, city from data where city.Contains('York')"
Enter
Sleep 2s

# Hide keypresses for clean output
Hide
Sleep 1s
Type ":exit"
Enter
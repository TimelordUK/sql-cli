# VHS script demonstrating SQL autocomplete with Tab

Output demos/sql-autocomplete.gif

Set FontSize 16
Set Width 1100
Set Height 600
Set Theme "Dracula"

# Start sql-cli with a CSV file
Type "sql-cli ../../data/small-customer.csv"
Enter
Sleep 2s

# Run initial query
Type "select * from data"
Enter
Sleep 2s

# Press Up arrow to recall last query
Up
Sleep 1s

# Go to end and add WHERE clause with autocomplete
Ctrl+E
Type " where Coun"
Sleep 500ms

# Tab to autocomplete to Contains
Tab
Sleep 1s

# Complete the Contains clause
Type "('Pan'"
Sleep 500ms
Type ")"
Sleep 500ms

# Execute the query
Enter
Sleep 3s

# Another example - Up arrow and add different WHERE
Up
Sleep 1s

# Clear and try with city
Ctrl+A
Ctrl+K
Type "select * from data where city."
Sleep 500ms
Tab
Sleep 1s
Type "('New York')"
Enter
Sleep 2s

# Exit
Type ":exit"
Enter
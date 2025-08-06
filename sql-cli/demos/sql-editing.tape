# VHS script demonstrating SQL query editing with arrow keys

Output demos/sql-editing.gif

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

# Go to end of line and add WHERE clause
Ctrl+E
Type " where age > 30"
Enter
Sleep 2s

# Up arrow again to edit
Up
Sleep 1s

# Clear the line and write new query
Ctrl+A
Ctrl+K
Type "select name, age, city from data where city = 'New York'"
Enter
Sleep 2s

# Up arrow to recall and modify
Up
Sleep 1s

# Navigate with arrow keys to edit middle of query
Ctrl+A
Right Right Right Right Right Right Right Right Right Right Right Right Right Right Right
Type "distinct "
Enter
Sleep 2s

# Show another example with Up arrow
Type "select count(*) from data"
Enter
Sleep 2s

Up
Ctrl+E
Type " where age between 25 and 45"
Enter
Sleep 2s

# Exit
Type ":exit"
Enter
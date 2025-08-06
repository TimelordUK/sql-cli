# VHS script for sql-cli overview demo
# Shows key features with a real dataset

Output demos/overview.gif

Set FontSize 14
Set Width 1000  # Reduced from 1200
Set Height 600  # Reduced from 800
Set Theme "Dracula"

Show

Type sql-cli ../../data/small-customer.csv
Sleep 500ms
Enter

# fuzzy searcher

Type f
Sleep 1s
Type "'Bonnie"
Enter
Sleep 1s
Type "l" Sleep 200ms
Type "l" Sleep 200ms
Type f
Sleep 1s
Enter

# show help

Up
Sleep 1
Ctrl+E
Ctrl+U
Type ":help"
Enter
Sleep 1s
Escape
Up
Sleep 1
Ctrl+E
Ctrl+U
Type "select * from "
Tab
Sleep 1s
Enter

# compact mode

Type "C"

# find a col

Type "\"
Type "Email"
Enter

# Navigate around to show the data

Type "l" Sleep 100ms
Type "j" Sleep 100ms
Type "j" Sleep 100ms

Type "h" Sleep 200ms

Sleep 1s

# Press Up arrow to recall last query
Up
Sleep 1s

# Go to end and add WHERE clause with autocomplete
Ctrl+E
Type " where Coun"
Sleep 500ms

# Tab auto complete gets Contains
Tab
Type ".Cont"
# Tab to autocomplete to Contains
Tab
Sleep 1s

# Complete the Contains clause
Type "Pan"
Sleep 200ms

Enter
Sleep 3s

Type "$"
Sleep 200ms
Type "^"
Sleep 200ms

Type "G"
Sleep 200ms
Type "g"
Sleep 200ms

# Press Up arrow to recall last query
Up
Sleep 1s

# Go to end and add WHERE clause with autocomplete
Ctrl+E
Type " and Last"
Sleep 1s
Tab
Type ".Leng"
Sleep 200ms
Tab
Sleep 200ms
Type "() > 5"

# Execute the query
Enter
Sleep 1500ms

Up
Sleep 1s
Ctrl+E
Ctrl+U
Type "select city"
Tab
Sleep 200ms
Type ", company"
Tab
Sleep 200ms
Type ", email"
Tab
Sleep 200ms
Type ", Country"
Tab
Sleep 200ms
Type ", customer"
Tab
Sleep 200ms
Type ", First"
Tab
Sleep 200ms
Type ", Last"
Tab
Sleep 200ms

Type " from small_customer where City in ('Kimbury', 'New Fred') order by Country ASC, City DESC, Email DESC"
Enter
Sleep 1500ms

# Exit TUI
Type "q"
Sleep 1s

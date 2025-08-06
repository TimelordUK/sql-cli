# Optimized overview - shorter sleeps, lower resolution
# Target: under 3MB while showing all features

Output demos/overview-optimized.gif

Set FontSize 14
Set Width 1000  # Reduced from 1200
Set Height 600  # Reduced from 800
Set Theme "Dracula"

# Faster typing and shorter pauses
Set TypingSpeed 50ms  # Faster typing
Set PlaybackSpeed 1.5  # Speed up playback

Show

Type sql-cli ../../data/small-customer.csv
Enter
Sleep 1s  # Reduced from 2s

# Fuzzy search - quick demo
Type f
Sleep 500ms
Type "'Bonnie"
Enter
Sleep 1.5s  # Reduced from 3s
Type f
Sleep 1s
Enter

# Help - very brief
Type ":help"
Enter
Sleep 1s  # Just show it exists
Escape

# SQL completion
Type "select * from "
Tab
Sleep 500ms
Enter

# Column navigation - faster
Type "\\"
Type "Email"
Enter
Type "p"
Sleep 500ms

# Quick navigation
Type "llll"
Sleep 200ms
Type "hhhh"
Sleep 200ms

# Statistics - brief
Type "s"
Sleep 1s
Escape

# Fuzzy filter
Type "/"
Type "York"
Sleep 1s
Escape

# SQL with autocomplete
Up
Ctrl+E
Type " where City.Cont"
Tab
Type "('York')"
Enter
Sleep 1.5s

# Show jumping
Type "G"
Sleep 500ms
Type "g"
Sleep 500ms

# Exit
Type ":exit"
Enter
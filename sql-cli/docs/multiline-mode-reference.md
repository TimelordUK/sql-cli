# SQL CLI Multi-line Mode Reference

## Overview
Press F3 to toggle between single-line and multi-line editing modes. The multi-line mode provides auto-formatting and easier editing for complex queries.

## Key Features

### Auto-Formatting
- When entering multi-line mode (F3), your SQL query is automatically pretty-formatted
- Complex queries are broken into readable lines with proper indentation  
- When returning to single-line mode, the query is compacted back

### Keybindings (Built-in textarea shortcuts)
- **Ctrl+A** - Move to beginning of line
- **Ctrl+E** - Move to end of line
- **Ctrl+F** / **Ctrl+B** - Move forward/backward by character
- **Alt+F** / **Alt+B** - Move forward/backward by word
- **Ctrl+N** / **Ctrl+P** - Move to next/previous line
- **Ctrl+K** - Delete from cursor to end of line
- **Ctrl+W** - Delete word before cursor
- **Ctrl+U** - Delete from cursor to beginning of line
- **Ctrl+Z** - Undo
- **Ctrl+Y** - Redo

### Special Keys
- **Tab** - SQL completion (works the same as single-line mode)
- **Enter** - Add new line
- **Ctrl+Enter** - Execute query
- **F3** - Toggle back to single-line mode

### Syntax Preview
- Below the textarea, a single-line syntax preview shows your query with full color highlighting
- Preview height is minimal (3 lines) to maximize editing space

## Example Workflow
1. Type a SQL query in single-line mode
2. Press F3 to enter multi-line mode - query is auto-formatted
3. Edit using the built-in keybindings
4. Use Tab for completion as needed
5. Press Ctrl+Enter to execute the query
6. Press F3 to return to single-line mode

## Benefits
- Clean, predictable behavior
- No mode confusion - always in "insert" mode
- Tab completion works consistently
- Built-in keybindings are familiar to terminal users
- Auto-formatting makes complex queries readable
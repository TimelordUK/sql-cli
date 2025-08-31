# SQL CLI Single-Line Mode Enhanced Navigation

## Overview
The single-line mode now includes powerful navigation and editing features similar to those found in bash/zsh.

## Navigation Features

### Word Movement
- **Ctrl+Left** / **Alt+B** - Move backward one word
- **Ctrl+Right** / **Alt+F** - Move forward one word
- **Ctrl+A** - Jump to beginning of line
- **Ctrl+E** - Jump to end of line

### SQL Token Navigation
- **Alt+[** - Jump to previous SQL token
- **Alt+]** - Jump to next SQL token

## Editing Features

### Deletion
- **Ctrl+W** - Delete word backward
- **Alt+D** - Delete word forward
- **Ctrl+K** - Kill line (delete from cursor to end of line)
- **Ctrl+U** - Kill line backward (delete from cursor to beginning of line)

### Undo/Redo and Kill Ring
- **Ctrl+Z** - Undo last edit
- **Ctrl+Y** - Yank (paste from kill ring)
  - The kill ring stores text deleted with Ctrl+K or Ctrl+U

## Benefits
- Faster navigation through complex SQL queries
- Familiar keybindings for terminal users
- Token-aware navigation understands SQL syntax
- Undo functionality prevents accidental deletions
- Kill ring allows cut/paste operations

## Example Workflow
1. Type a SQL query: `SELECT id, name FROM users WHERE age > 25`
2. Use Alt+[ to jump back through tokens (25, >, age, WHERE, etc.)
3. Use Ctrl+W to delete the word "age"
4. Type "created_at"
5. Use Ctrl+Z to undo if you made a mistake
6. Use Ctrl+K to kill the rest of the line and Ctrl+Y to paste it elsewhere
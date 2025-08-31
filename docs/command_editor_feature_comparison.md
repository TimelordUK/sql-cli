# CommandEditor Feature Comparison

## ✅ What CommandEditor DOES Handle

### Character Input
- [x] Normal character typing (a-z, A-Z, 0-9, special chars)
- [x] Backspace
- [x] Delete

### Line Navigation  
- [x] **Ctrl+A** - Move to beginning of line
- [x] **Ctrl+E** - Move to end of line
- [x] **Home** - Jump to start
- [x] **End** - Jump to end
- [x] **Left/Right arrows** - Character movement

### Word Operations
- [x] **Ctrl+W** - Delete word backward
- [x] **Ctrl+D** - Delete word forward (when at word boundary)
- [x] **Alt+B** - Move word backward
- [x] **Alt+F** - Move word forward
- [x] **Alt+D** - Delete word forward
- [x] **Ctrl+Left/Right** - Word navigation

### Line Operations
- [x] **Ctrl+K** - Kill line (delete from cursor to end)
- [x] **Ctrl+U** - Kill line backward (delete from start to cursor)

## ❌ What CommandEditor DOESN'T Handle (Still in old methods)

### Clipboard/Kill Ring Operations
- [ ] **Ctrl+Y** - Yank from kill ring
- [ ] **Ctrl+V** - Paste from system clipboard
- [ ] Kill ring management (storing killed text)

### SQL-Specific Navigation
- [ ] **Alt+[** - Jump to previous SQL token
- [ ] **Alt+]** - Jump to next SQL token
- [ ] Token-aware navigation

### History Operations
- [ ] **Ctrl+P** - Previous history command
- [ ] **Ctrl+N** - Next history command
- [ ] **Ctrl+R** - History search mode
- [ ] **Alt+Up/Down** - Alternative history navigation

### Application Commands (Correctly excluded)
- [ ] **Ctrl+X** - Expand asterisk (handled by action system)
- [ ] **Alt+X** - Expand asterisk visible only
- [ ] **Tab** - SQL completion (needs full TUI context)
- [ ] **Enter** - Execute query
- [ ] **Escape** - Mode transitions

## Summary

CommandEditor has **ALL core text editing functions** including:
- ✅ Kill to end of line (Ctrl+K)
- ✅ Kill to beginning (Ctrl+U)  
- ✅ Delete word operations (Ctrl+W, Alt+D)
- ✅ Word movement (Alt+B/F, Ctrl+Left/Right)
- ✅ Line navigation (Ctrl+A/E, Home/End)

What's missing are **application-specific features**:
- Clipboard/kill ring integration
- SQL token navigation
- History navigation
- These should probably stay in the main TUI for now

## Recommendation

The old `try_handle_text_editing` method can have its text editing parts removed since CommandEditor handles them all. Keep only:
1. Clipboard operations (Ctrl+Y/V)
2. SQL token jumps (Alt+[/])
3. Special application commands

This would remove about 60-80 lines of duplicated code!
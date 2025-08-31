# CommandEditor Cleanup Plan

## Current Situation
After implementing CommandEditor, there's significant duplication between:
1. CommandEditor's handle_input method
2. try_handle_text_editing method
3. try_handle_history_navigation method

## What Can Be Removed

### From try_handle_text_editing:
These are now handled by CommandEditor and can be removed:
- **Lines 2621-2640**: delete_word_backward, delete_word_forward, kill_line, kill_line_backward
- **Lines 2641-2648**: move_word_backward, move_word_forward  
- **Lines 2667-2688**: Ctrl+K/U hardcoded handlers
- **Lines 2709-2728**: Ctrl+Left/Right, Alt+B/F movement handlers

### Duplicated in F-keys:
- **F9/F10** (lines 2333-2361): kill_line operations - duplicates CommandEditor

## What Must Stay

### In try_handle_text_editing:
These are NOT in CommandEditor and must remain:
- **expand_asterisk** operations (Ctrl+X, Alt+X) - application-specific
- **Ctrl+Y**: yank from kill ring - clipboard operations
- **Ctrl+V**: paste from system clipboard
- **Alt+[/]**: jump to SQL tokens - SQL-specific navigation
- **jump_to_prev_token/jump_to_next_token** - SQL-aware navigation

### In try_handle_history_navigation:
All of this should stay for now (future Phase 2):
- **Ctrl+R**: history search
- **Ctrl+P/N**: previous/next history
- **Alt+Up/Down**: alternative history navigation

## Recommended Approach

### Option 1: Minimal Cleanup (Safe)
1. Comment out the duplicated handlers in try_handle_text_editing
2. Test thoroughly
3. Remove commented code after verification

### Option 2: Refactor try_handle_text_editing (Better)
1. Rename to `try_handle_special_commands`
2. Keep only non-text-editing operations:
   - expand_asterisk
   - clipboard operations (Ctrl+Y/V)
   - SQL token navigation
3. Remove all basic text editing

### Option 3: Full Migration (Phase 2)
1. Move clipboard operations to CommandEditor
2. Move SQL token navigation to CommandEditor
3. Keep only application-level commands (expand_asterisk)
4. Eventually move history to CommandEditor

## Benefits of Cleanup
1. **Remove ~100 lines** of duplicated code
2. **Clear separation**: CommandEditor handles text editing, TUI handles app commands
3. **Easier maintenance**: Single source of truth for text operations
4. **Better performance**: Fewer checks in the command input pipeline

## Testing After Cleanup
Verify these still work:
- [x] Basic text editing (Ctrl+A/E/W/K/U)
- [x] Word navigation (Alt+B/F)
- [x] Ctrl+X for expand asterisk
- [ ] Ctrl+Y for yank from kill ring
- [ ] Ctrl+V for clipboard paste
- [ ] Alt+[/] for SQL token jumps
- [ ] Ctrl+P/N for history
- [ ] Ctrl+R for history search
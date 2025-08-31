# Key Migration Audit Report

## Summary
This document audits the current state of key handling migration from the legacy dispatcher/direct handling system to the Redux-like action system.

## Results Mode

### ✅ Already Migrated to Actions
- Navigation: `h`, `j`, `k`, `l`, `g`, `G`, `H`, `M`, `L` (viewport navigation)
- Column operations: `p` (pin), `s` (sort), `/` (column search), `:` (jump to row)
- View toggles: `N` (row numbers), `C` (compact mode)
- Export: `Ctrl+X` (CSV), `Ctrl+J` (JSON)
- F-keys: `F8` (case insensitive), `F12` (key indicator)
- Filter: `Shift+C` (clear filter)

### 🔄 Handled by Key Dispatcher (Legacy System)
**High Priority for Migration:**
- Basic navigation: Arrow keys, PageUp/PageDown
- Mode switching: `Esc`, `Up` (exit to Command)
- Search operations: `/` (search), `\` (column search), `F`, `f` (filters)
- Column movement: `Shift+Left/Right`
- Column operations: `^`, `0`, `$` (first/last column)
- Search navigation: `n`, `N` (next/previous match)
- Export: `Ctrl+E` (CSV alternative)
- Help: `F1`, `?`
- Debug: `F5`

**Medium Priority:**
- Selection mode: `v` (toggle selection mode)
- Pin operations: `Shift+P` (clear pins)  
- Column stats: `Shift+S`
- Quit: `q`

### 🔧 Direct Key Handling (Hardest to Migrate)
**Complex Operations:**
- ` ` (Space) - Toggle viewport lock (involves ViewportManager)
- `x`/`X` - Toggle cursor lock (involves ViewportManager) 
- `Ctrl+Space` - Toggle viewport lock (alternative)
- `y` - Yank operations (complex multi-mode behavior)

**F-key Operations in Results:**
- `F1`/`?` - Help toggle (duplicate with dispatcher)

## Command Mode

### ✅ Already Migrated to Actions
- Text editing: `Backspace`, `Delete`, `Ctrl+U` (clear line)
- Cursor movement: Arrow keys, `Home`, `End`, `Ctrl+A/E`
- Word operations: `Ctrl+W`, `Alt+D`, `Alt+B/F`
- Line operations: `Ctrl+K`, `F9`, `F10`
- Clipboard: `Ctrl+V` (paste)
- F-keys: `F8` (case insensitive), `F12` (key indicator)

### 🔄 Handled by Key Dispatcher (Legacy System)
**Buffer Management:**
- `Alt+N` (new buffer), `Alt+W` (close buffer)
- `Alt+Tab` (next buffer), `F11` (previous), `F12` (next)
- `Ctrl+1-9` (switch to buffer N)

**Query Operations:**
- `Tab` (completion), `Enter` (execute)
- `Ctrl+R` (history search)
- `Ctrl+*` (expand asterisk)
- `Ctrl+P/N` (history navigation)

**F-key Operations:**
- `F6` (pretty query), `F5` (debug), `F1` (help)

### 🔧 Direct Key Handling (Special Cases)
- `Ctrl+C/D` - Quit (safety critical)
- History search handling (complex state management)
- Buffer switching with specific numbers
- Multi-line editor features (deprecated)

## Other Modes

### ✅ Handled by Specialized Widgets
- **Search/Filter/ColumnSearch**: Handled by `SearchModesWidget`
- **Help**: Handled by `HelpWidget` 
- **History**: Custom handler for history search
- **Debug/PrettyQuery**: Simple escape-to-exit handlers
- **ColumnStats**: Handled by `StatsWidget`
- **JumpToRow**: Simple number input handler

### 🔧 Complex Multi-Key Operations
- **Chord Handler**: `y` sequences (`yy`, `yc`, `ya`, `yv`, `yq`) - Yank operations
- **Hide Column**: `H` operations - Column hiding/showing

## Migration Priority List

### Phase 1: High Value, Low Risk
1. **Basic navigation in Results** - Arrow keys, PageUp/Down (high usage)
2. **Mode switching** - Esc, Up arrow (fundamental operations)
3. **Search navigation** - n, N (search workflow completion)
4. **Column navigation** - ^, 0, $ (vim-style navigation)

### Phase 2: Medium Value, Medium Risk  
1. **Export operations** - Ctrl+E alternative (consolidate exports)
2. **Selection mode** - v toggle (user workflow)
3. **Help/Debug F-keys** - F1, F5 (consolidate with existing)
4. **Column stats** - Shift+S (complete column operations)

### Phase 3: Complex Operations (Careful Design Needed)
1. **Viewport operations** - Space, x, Ctrl+Space (ViewportManager integration)
2. **Buffer management** - Alt+N, Alt+W, etc. (complex state)
3. **Yank operations** - y sequences (multi-key, multi-mode)
4. **Query completion** - Tab (complex completion logic)

### Phase 4: Edge Cases & Cleanup
1. **Duplicate handlers** - Consolidate F1/?, Ctrl+E/Ctrl+X
2. **Legacy F-keys** - F3 (deprecated), F6 (available)
3. **Command mode F-keys** - F9/F10 alternatives  

## Current Action System Coverage

**Estimated Migration Progress: ~40%**

- Results mode: ~50% migrated (basic operations done, complex ones remain)
- Command mode: ~60% migrated (text editing complete, buffer management remains)
- Other modes: ~90% migrated (mostly handled by specialized widgets)

## Recommendations

1. **Focus on Phase 1** - High-impact, low-risk migrations for immediate UX improvement
2. **Redesign yank system** - Consider simplifying the multi-key yank operations
3. **Consolidate duplicates** - Remove duplicate key handlers (F1/?, Ctrl+E/Ctrl+X)
4. **ViewportManager integration** - Design proper action system integration for viewport operations
5. **Buffer management actions** - Create comprehensive buffer management action set

## Benefits of Continuing Migration

- **Consistency**: Single key handling pathway
- **Testability**: Actions can be unit tested independently
- **Debuggability**: Clear action logging and tracing
- **Maintainability**: Centralized key mapping configuration
- **Extensibility**: Easy to add new key combinations
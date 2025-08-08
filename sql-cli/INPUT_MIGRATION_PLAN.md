# Input Area Migration Plan - Detailed Implementation Steps

## Overview
Migrating the input area from direct field access to a buffer-based architecture while maintaining full functionality and ensuring the TUI remains working at each step.

## Phase 1: Create InputManager Abstraction ✅
**Status: COMPLETED**
- Created `input_manager.rs` with unified `InputManager` trait
- Implemented `SingleLineInput` wrapper for `tui_input::Input`
- Implemented `MultiLineInput` wrapper for `tui_textarea::TextArea`
- Provides unified interface for both input types

**Files Created:**
- `src/input_manager.rs`

**Next Steps:**
- Add module to lib.rs
- Test basic functionality

---

## Phase 2: Integrate InputManager into Buffer ✅
**Status: COMPLETED**
**Goal:** Add InputManager to Buffer struct without breaking existing code

**Steps Completed:**
1. ✅ Added `input_manager` module to lib.rs
2. ✅ Updated Buffer struct to include `input_manager: Box<dyn InputManager>`
3. ✅ Initialize with SingleLineInput by default
4. ✅ Added BufferAPI methods:
   - `get_input_text() -> String`
   - `set_input_text(text: String)`
   - `handle_input_key(event: KeyEvent) -> bool`
   - `switch_input_mode(multiline: bool)`
   - `get_input_cursor_position() -> usize`
   - `set_input_cursor_position(position: usize)`
   - `is_input_multiline() -> bool`
5. ✅ Kept existing `input` and `textarea` fields for compatibility
6. ✅ Implemented manual Clone for Buffer
7. ✅ Added sync helper methods for migration period

**Files Modified:**
- `src/lib.rs` - Added input_manager module
- `src/buffer.rs` - Added InputManager integration

---

## Phase 3: Synchronization Layer
**Goal:** Keep InputManager in sync with legacy fields during transition

**Steps:**
1. Add sync methods in Buffer:
   - `sync_from_input()` - Copy from legacy input to InputManager
   - `sync_to_input()` - Copy from InputManager to legacy input
2. Call sync methods in BufferAPI setters/getters
3. Ensure text content stays consistent
4. Test edit mode switching with synchronization

**Files to Modify:**
- `src/buffer.rs`

---

## Phase 4: Update Enhanced TUI - Read Path
**Goal:** Start using BufferAPI for reading input values

**Steps:**
1. Replace `self.input.value()` with `buffer.get_input_text()`
2. Update query execution to use buffer methods
3. Update status display to use buffer methods
4. Keep write operations using legacy fields for now
5. Test that queries still execute correctly

**Files to Modify:**
- `src/enhanced_tui.rs` (partial - read operations only)

---

## Phase 5: Update Enhanced TUI - Write Path
**Goal:** Use BufferAPI for modifying input

**Steps:**
1. Replace `self.input = Input::new(...)` with `buffer.set_input_text(...)`
2. Update history navigation to use buffer methods
3. Update completion insertion to use buffer methods
4. Route key events through `buffer.handle_input_key()`
5. Test all input modifications work

**Files to Modify:**
- `src/enhanced_tui.rs` (partial - write operations)

---

## Phase 6: Edit Mode Switching
**Goal:** Move F3 mode switching entirely to Buffer

**Steps:**
1. Implement `switch_input_mode()` in Buffer
2. Handle content transfer between single/multi-line
3. Preserve cursor position when switching
4. Update F3 handler to use buffer method
5. Remove direct textarea/input manipulation from TUI
6. Test mode switching preserves content and cursor

**Files to Modify:**
- `src/buffer.rs`
- `src/enhanced_tui.rs` (F3 handler)

---

## Phase 7: Advanced Input Features
**Goal:** Migrate complex input features to work through InputManager

**Steps:**
1. Kill ring operations:
   - Ctrl+K (kill to end of line)
   - Ctrl+U (kill to start of line)
   - Ctrl+Y (yank)
2. Undo/Redo:
   - Store InputManager state snapshots
   - Restore through InputManager methods
3. Selection and clipboard:
   - Track selection through InputManager
   - Copy/paste operations
4. Test all keyboard shortcuts work correctly

**Files to Modify:**
- `src/input_manager.rs` (add kill ring support)
- `src/buffer.rs` (update undo/redo to use InputManager)
- `src/enhanced_tui.rs` (update key handlers)

---

## Phase 8: Completion State Integration
**Goal:** Move CompletionState into Buffer and integrate with InputManager

**Steps:**
1. Move `CompletionState` struct to buffer.rs
2. Add completion fields to Buffer
3. Update completion to work with InputManager cursor position
4. Cache completions per buffer
5. Update Tab handler to use buffer completion
6. Test tab completion in both single and multi-line modes

**Files to Modify:**
- `src/buffer.rs`
- `src/enhanced_tui.rs` (Tab handler)

---

## Phase 9: Syntax Highlighting Integration
**Goal:** Apply highlighting through InputManager

**Steps:**
1. Add highlighting cache to Buffer
2. Create `get_highlighted_input()` method
3. Update rendering to use highlighted version
4. Implement incremental highlighting updates
5. Test highlighting performance with large queries

**Files to Modify:**
- `src/buffer.rs`
- `src/enhanced_tui.rs` (render methods)

---

## Phase 10: Search/Filter Input Handling
**Goal:** Properly handle input state for search/filter modes

**Steps:**
1. Create input state stack in Buffer
2. Push current input when entering search/filter
3. Restore input when exiting search/filter
4. Use InputManager for search/filter input
5. Test mode transitions preserve input correctly

**Files to Modify:**
- `src/buffer.rs`
- `src/enhanced_tui.rs` (mode handlers)

---

## Phase 11: Remove Legacy Fields
**Goal:** Clean up redundant fields from TUI and Buffer

**Steps:**
1. Remove `input` field from EnhancedTuiApp
2. Remove `textarea` field from EnhancedTuiApp
3. Remove legacy `input` and `textarea` from Buffer
4. Remove synchronization methods
5. Update all remaining direct field access
6. Test everything still works

**Files to Modify:**
- `src/enhanced_tui.rs`
- `src/buffer.rs`

---

## Phase 12: Testing and Documentation
**Goal:** Ensure robust testing and documentation

**Steps:**
1. Unit tests for InputManager trait
2. Integration tests for buffer input operations
3. Tests for mode switching
4. Tests for all keyboard shortcuts
5. Performance benchmarks
6. Update documentation

**Files to Create:**
- `tests/input_manager_test.rs`
- `tests/buffer_input_test.rs`

---

## Testing Strategy for Each Phase

### After Each Phase:
1. **Build:** `cargo build`
2. **Basic Test:** Run and type a simple query
3. **Mode Test:** Switch between single/multi-line (F3)
4. **Completion Test:** Tab completion works
5. **History Test:** Up/down arrow for history
6. **Execute Test:** Query execution works
7. **Special Keys:** Ctrl+K, Ctrl+U, Ctrl+Y work

### Critical Test Scenarios:
- [ ] Single-line query execution
- [ ] Multi-line query with proper formatting
- [ ] Tab completion in both modes
- [ ] History navigation
- [ ] Mode switching preserves content
- [ ] Kill ring operations
- [ ] Undo/redo functionality
- [ ] Search/filter mode transitions
- [ ] Syntax highlighting display
- [ ] Cursor position accuracy

## Risk Mitigation

1. **Backward Compatibility:** Keep legacy fields during transition
2. **Incremental Changes:** Small, testable commits
3. **Feature Flags:** Could add feature flag for new input system
4. **Rollback Plan:** Each phase can be reverted independently
5. **Testing:** Comprehensive tests before removing legacy code

## Success Criteria

- [ ] All existing input features work identically
- [ ] Multiple buffers can have independent input state
- [ ] No performance regression
- [ ] Clean separation between UI and buffer state
- [ ] Improved testability of input handling
- [ ] Foundation for future enhancements (smart completion, etc.)

## Notes

- Each phase should be a separate PR if possible
- Run full test suite after each phase
- Document any behavioral changes
- Keep performance metrics to ensure no regression
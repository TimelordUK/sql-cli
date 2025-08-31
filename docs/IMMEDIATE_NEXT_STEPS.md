# Immediate Next Steps for State Management

## Current Status
- Shadow state has comprehensive read methods
- Started replacing buffer().get_mode() calls (7 done, 43 remaining)
- Shadow state observes but doesn't control yet

## Priority Order

### 1. Complete Shadow State as Source of Truth (This Week)
**Why First**: Need centralized state before we can snapshot it

#### Step 1.1: Add Write-Through Methods
```rust
impl ShadowStateManager {
    /// Set mode - updates shadow state and buffer (temporarily)
    pub fn set_mode(&mut self, mode: AppMode, buffer: &mut Buffer) {
        let old_mode = self.get_mode();
        self.state = self.mode_to_state(mode.clone());
        buffer.set_mode(mode); // Keep buffer in sync for now
        
        self.record_transition(old_mode, mode, "set_mode");
    }
    
    /// Eventually buffer param will be removed
}
```

#### Step 1.2: Replace All State Reads
- Replace remaining 43 buffer().get_mode() calls
- Use shadow_state.borrow().is_in_*() methods
- This can be done incrementally, file by file

#### Step 1.3: Replace All State Writes  
- Find all buffer.set_mode() calls
- Replace with shadow_state.borrow_mut().set_mode()
- Shadow state becomes authoritative

#### Step 1.4: Remove Mode from Buffer
- Delete mode field from Buffer struct
- Remove get_mode/set_mode from Buffer
- All mode access now through shadow state

### 2. Prepare for State Snapshots (Next Week)
**Why Second**: Need clean state management first

#### Step 2.1: Define Snapshot Types
```rust
// Start simple - just the essentials
pub struct BufferStateSnapshot {
    mode: AppMode,
    cursor_position: (usize, usize),
    search_pattern: Option<String>,
    filter_active: bool,
}
```

#### Step 2.2: Add Snapshot Field to Buffer
- Add `state_snapshot: Option<BufferStateSnapshot>`
- Don't use it yet, just prepare structure

### 3. Implement Save/Restore (Week 3)
**Why Third**: Build on clean architecture

#### Step 3.1: Implement Save
- ShadowStateManager::save_to_buffer()
- Collect from all state sources

#### Step 3.2: Implement Restore  
- ShadowStateManager::restore_from_buffer()
- Distribute to all state targets

#### Step 3.3: Test with Manual Commands
- Add debug commands to save/restore
- Verify state preservation works

### 4. Add Buffer Switching UI (Week 4)
**Why Last**: Everything else must work first

- Add buffer list display
- Add switching keybindings (Ctrl+Tab, etc)
- Wire up save/restore on switch

## Today's Focus

Let's continue with **Step 1.2** - replacing buffer().get_mode() calls:

1. Start with the most critical paths:
   - Key event handlers
   - Navigation logic  
   - Mode switching logic

2. Use this pattern:
   ```rust
   // Old:
   if self.buffer().get_mode() == AppMode::Results {
   
   // New:
   if self.shadow_state.borrow().is_in_results_mode() {
   ```

3. Run tests after each batch of replacements

## File Priority for Migration

Based on impact and usage:

1. **enhanced_tui.rs** - Main event loop (highest impact)
2. **action_handlers.rs** - All actions go through here
3. **key_mapper.rs** - Key handling logic
4. **traits/*.rs** - Shared behavior
5. **viewport_manager.rs** - Navigation
6. Other files as needed

## Success Metrics

- [ ] All 50+ get_mode() calls use shadow state
- [ ] All set_mode() calls use shadow state  
- [ ] Buffer struct has no mode field
- [ ] Tests pass with centralized state
- [ ] Can save/restore a basic snapshot
- [ ] Buffer switching preserves cursor position

## Parallel Work Possible

While migrating state access, we can also:
- Document the state architecture
- Write tests for state transitions
- Design the buffer list UI
- Plan the undo/redo system (uses snapshots)
# StateCoordinator Migration Plan

## Goal
Decouple the TUI from complex state synchronization logic by centralizing it in StateCoordinator.

## Current State
The EnhancedTuiApp currently owns and manages:
- `state_container: AppStateContainer`
- `hybrid_parser: HybridParser` 
- `viewport_manager: RefCell<Option<ViewportManager>>`
- `shadow_state: RefCell<ShadowStateManager>`

And has these sync methods scattered throughout:
- `sync_mode()` - synchronizes mode across all components
- `set_mode_via_shadow_state()` - alternative mode setter
- `sync_after_buffer_switch()` - syncs viewport and parser after buffer switch
- `save_viewport_to_current_buffer()` - saves viewport state
- `restore_viewport_from_current_buffer()` - restores viewport state
- `update_parser_for_current_buffer()` - updates parser schema

## Migration Strategy

### Phase 1: Delegation (Current Phase)
Keep the existing structure but delegate sync logic to StateCoordinator:

1. **Create StateCoordinator instance methods that accept references**
   - This allows us to use StateCoordinator without moving ownership
   - Methods like `sync_mode_with_refs(&mut state_container, &shadow_state, mode, trigger)`

2. **Update TUI sync methods to delegate**
   ```rust
   fn sync_mode(&mut self, mode: AppMode, trigger: &str) {
       StateCoordinator::sync_mode_with_refs(
           &mut self.state_container,
           &self.shadow_state,
           mode,
           trigger
       );
   }
   ```

3. **Benefits**:
   - Minimal structural changes
   - Can test incrementally
   - Logic is centralized even if ownership isn't

### Phase 2: Partial Ownership
Move some components to StateCoordinator:

1. **Move simple components first**
   - `hybrid_parser` is a good candidate (less interconnected)
   
2. **Create accessor methods in TUI**
   ```rust
   fn parser(&self) -> &HybridParser {
       self.state_coordinator.parser()
   }
   ```

3. **Update all parser access to go through coordinator**

### Phase 3: Full Ownership
Eventually move all state components to StateCoordinator:

1. **StateCoordinator owns all state**
   ```rust
   pub struct EnhancedTuiApp {
       state_coordinator: StateCoordinator,
       // Only UI-specific fields remain
       input: tui_input::Input,
       editor_widget: EditorWidget,
       // etc...
   }
   ```

2. **TUI only handles**:
   - User input
   - Rendering
   - UI widgets
   - Delegating actions to StateCoordinator

## Implementation Order

### Immediate Tasks (Phase 1)
1. ✅ Create StateCoordinator with basic sync methods
2. Create static/instance methods that work with references
3. Update TUI's `sync_mode()` to delegate
4. Update TUI's `sync_after_buffer_switch()` to delegate
5. Test that existing functionality still works

### Next Tasks (Phase 2)
1. Move `hybrid_parser` to StateCoordinator
2. Update all parser access points
3. Move mode synchronization fully to coordinator
4. Add comprehensive tests

### Future Tasks (Phase 3)
1. Move `state_container` ownership
2. Move `viewport_manager` ownership  
3. Move `shadow_state` ownership
4. Refactor TUI to be purely UI-focused

## Success Criteria
- [ ] All state sync logic is in StateCoordinator
- [ ] TUI has no direct state synchronization code
- [ ] Tests pass without regression
- [ ] Code is cleaner and more maintainable
- [ ] Clear separation between UI and state management

## Risks and Mitigations
1. **Risk**: Breaking existing functionality
   - **Mitigation**: Incremental changes with testing at each step

2. **Risk**: Complex refactoring of viewport management
   - **Mitigation**: Leave viewport for last, it's the most complex

3. **Risk**: Circular dependencies
   - **Mitigation**: Use RefCell/Rc carefully, consider weak references

## Notes
- The viewport management is particularly complex due to its interaction with rendering
- Consider creating a separate ViewportCoordinator later
- Keep the migration incremental to avoid large breaking changes
# Critical Issues & Technical Debt Tracking

## 🚨 CRITICAL ISSUES

### 1. History File Corruption - Multiple CommandHistory Instances
**Priority**: P0 - Data Loss  
**Status**: Identified  
**Impact**: User history gets truncated to 1 entry on save

**Root Cause**: Multiple CommandHistory instances fighting over same file:
- `enhanced_tui.rs`: Creates own CommandHistory instance
- `app_state_container.rs`: Creates own CommandHistory instance  
- `global_state.rs`: May also create instance (needs verification)

**Fix Required**:
- Remove duplicate CommandHistory instances
- Use single shared instance from AppStateContainer
- Test history save/load after consolidation

**Evidence**:
- Backup files show 103-105 entries correctly preserved
- Main history.json gets overwritten with only 1 entry
- HistoryProtection backups working correctly

---

### 2. V16 NavigationState Merge Completion
**Priority**: P1 - Architecture  
**Status**: ✅ COMPLETED  
**Impact**: Main branch now has complete V16+V17 with unified logging

**Completed Actions**:
- Merged complete V16 NavigationState from refactor-v16-more-data
- Resolved merge conflicts between V16 and V17
- Updated all logging to unified debug_service.log() pattern
- V17 branch can be safely deleted

---

### 3. Ctrl+R History Search in Results Mode
**Priority**: P2 - UX  
**Status**: ✅ COMPLETED  
**Impact**: Users can now search history from Results view

**Completed Actions**:
- Added "start_history_search" action to Results mode handler
- Switches to Command mode, populates last query
- Activates History mode with search results
- Shows match count in status message

---

## 📋 TECHNICAL DEBT

### State Management Refactoring Progress
Based on `docs/STATE_MANAGEMENT_REFACTOR.md`:

- ✅ Phase 1: Complete State Consolidation (v10-v13)
- 🚧 Phase 2: Search/Filter State (v14) - NOT STARTED
- ✅ Phase 3: History Search State (v15) - COMPLETED
- ✅ Phase 4: Navigation State (v16) - COMPLETED  
- ✅ Phase 5: Buffer/Results State (v17) - COMPLETED
- 📋 Phase 6: Input State Completion (v18) - PENDING
- 📋 Phase 7: Subscription System (v19) - PENDING
- 📋 Phase 8: Widget State Binding (v20) - PENDING

### Architecture Issues to Address
1. **Multiple CommandHistory instances** (P0 - causes data loss)
2. **Search/Filter state still scattered** in EnhancedTuiApp (P2)
3. **Input state not fully centralized** (P3)
4. **No subscription system** for state changes (P3)
5. **Widgets not bound to state** (P4)

---

## 🧪 TESTING REQUIREMENTS

### Before Next Release
- [ ] Test complete V16+V17 functionality on Linux
- [ ] Test complete V16+V17 functionality on Windows
- [ ] Verify history save/load after architecture fix
- [ ] Verify Ctrl+R works in all modes
- [ ] Run full regression test suite

### Regression Testing Checklist
- [ ] Table navigation (arrow keys, page up/down)
- [ ] Search functionality (Ctrl+F)
- [ ] Filter functionality
- [ ] History search (Ctrl+R) 
- [ ] Query execution and results display
- [ ] Mode switching (Command/Results/History)
- [ ] File operations (save, load)
- [ ] Window resizing and viewport management

---

## 🔄 NEXT ACTIONS

### Immediate (Current Session)
1. ✅ Document critical issues in this file
2. Continue with comprehensive testing of V16+V17 on main branch
3. Address history corruption in future session

### Next Session Priority
1. **Fix CommandHistory architecture** (P0 - data loss prevention)
2. **Start Phase 2: Search/Filter State** (v14 refactoring)
3. **Complete comprehensive testing** on both platforms

### Future Sessions
4. Continue state management refactoring (v18-v20)
5. Implement subscription system
6. Add widget state binding

---

## 🔮 FEATURE REQUESTS

### JSON1 Format Support  
**Priority**: P3 - Enhancement  
**Status**: Requested by user  
**Description**: Support for JSON1 format (one JSON object per line)

**Technical Considerations**:
- Current system assumes consistent structure across rows
- JSON1 can have varying schemas per line
- Need to handle schema detection and column alignment
- May require separate parsing mode

**Impact**: Medium - affects data import/export workflows

### Viewport Navigation Issues in Fuzzy Mode
**Priority**: P2 - UX Bug  
**Status**: Identified  
**Description**: G/g navigation keys don't work when fuzzy search is active

**Technical Issues**:
- Disconnect between search state and viewport navigation
- Key handlers may be intercepted by fuzzy search mode
- Navigation state not properly synchronized with search results

**Note**: To be addressed after state management refactoring completion

---

## 📝 NOTES

### User Feedback Context
- History truncation has been "ongoing problem" 
- Backups are working correctly (10 backup files with 103-105 entries)
- User wants to "push on with all data migration" for now
- Address architecture issues in future sessions
- Considering JIRA for better issue tracking

### Success Metrics  
- No data loss in history files
- Consistent state across all application modes
- Clean separation between UI and state logic
- All state changes logged and traceable
- Performance improvements from reduced re-renders

---

*Last Updated: 2025-08-10*  
*Created during: V16+V17 merge completion and critical issue discovery*
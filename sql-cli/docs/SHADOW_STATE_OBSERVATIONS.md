# Shadow State Observation Points

## Current Observations (What Gets Logged)

### 1. **Query Execution** (Line 3271)
**Trigger**: When user presses Enter to execute a SQL query  
**Location**: `enhanced_tui.rs:3271` in `execute_query()`
**Logged Transition**: `COMMAND -> RESULTS`
**Log Message**: `[#1] COMMAND -> RESULTS (trigger: execute_query_success)`

### 2. **Vim Search Start** (Line 3799)
**Trigger**: When user presses `/` key to start vim search  
**Location**: `enhanced_tui.rs:3799` in `start_vim_search()`
**Logged Transition**: `RESULTS -> SEARCH(Vim)`
**Log Message**: `[#2] RESULTS -> SEARCH(Vim) (trigger: slash_key_pressed)`

### 3. **Search Cancellation** (Lines 2993, 3002)
**Trigger**: When user presses Escape while searching  
**Location**: `enhanced_tui.rs:2993-3002` in search widget cancel handler
**Logged Transitions**: 
- First: Search ends
- Then: `SEARCH -> RESULTS`
**Log Messages**:
- `[#3] Exiting search -> RESULTS (trigger: search_cancelled)`
- `[#4] RESULTS -> RESULTS (trigger: return_from_search)` (may be redundant)

## How to See the Logs

### Enable Shadow State Logging:
```bash
# Build with shadow-state feature
cargo build --release --features shadow-state

# Run with logging enabled
RUST_LOG=shadow_state=info ./target/release/sql-cli test.csv
```

### What You'll See in Terminal:

1. **In Status Line** (always visible):
   - `[Shadow: COMMAND]` when typing queries
   - `[Shadow: RESULTS]` when viewing results
   - `[Shadow: SEARCH(Vim)]` when searching

2. **In Log Output** (with RUST_LOG):
```
[INFO shadow_state] Shadow state manager initialized
[INFO shadow_state] [#1] COMMAND -> RESULTS (trigger: execute_query_success)
[INFO shadow_state] [#2] RESULTS -> SEARCH(Vim) (trigger: slash_key_pressed)
[INFO shadow_state] [#3] Exiting search -> RESULTS (trigger: search_cancelled)
```

## Test Sequence to See All Observations

1. **Start application**: `./target/release/sql-cli test.csv`
   - Shadow state starts in COMMAND mode
   - Status line shows: `[Shadow: COMMAND]`

2. **Execute a query**: Type `select * from data` and press Enter
   - Log: `[#1] COMMAND -> RESULTS (trigger: execute_query_success)`
   - Status line changes to: `[Shadow: RESULTS]`

3. **Start vim search**: Press `/`
   - Log: `[#2] RESULTS -> SEARCH(Vim) (trigger: slash_key_pressed)`
   - Status line changes to: `[Shadow: SEARCH(Vim)]`

4. **Cancel search**: Press Escape
   - Log: `[#3] Exiting search -> RESULTS (trigger: search_cancelled)`
   - Status line changes back to: `[Shadow: RESULTS]`

## Side Effects Logged

When transitions happen, shadow state also logs expected side effects:

- **COMMAND -> RESULTS**: 
  - `Expected side effects: Clear searches, reset viewport, enable nav keys`

- **RESULTS -> SEARCH**: 
  - `Expected side effects: Clear other searches, setup search UI`

- **SEARCH -> RESULTS**: 
  - `Expected side effects: Clear search UI, restore nav keys`

## Missing Observations (TODO)

Currently NOT observing:
- Column search start/end
- Filter mode transitions  
- Help/Debug mode transitions
- Return to Command mode (Escape in Results)
- History search mode
- Many other mode transitions (57 total set_mode calls!)

## Debug Information

To see full state history in debug view (F5):
```rust
// The shadow state manager tracks:
- Last 100 transitions with timestamps
- Total transition count
- Any discrepancies detected
```

## Next Steps

1. Add more observation points for other mode transitions
2. Watch for patterns in the logs
3. Identify missing side effects
4. Use insights to design proper state management
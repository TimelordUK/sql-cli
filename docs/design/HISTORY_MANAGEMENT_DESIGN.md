# History Management Design - Solving Query Refinement Pollution

## The Problem
When iteratively refining a query, history gets polluted with many intermediate versions:
```sql
SELECT * FROM trades WHERE symbol = 'AAPL'
SELECT * FROM trades WHERE symbol = 'AAPL' AND price > 100
SELECT * FROM trades WHERE symbol = 'AAPL' AND price > 100 ORDER BY date
SELECT * FROM trades WHERE symbol = 'AAPL' AND price > 150 ORDER BY date
SELECT * FROM trades WHERE symbol = 'AAPL' AND price > 150 ORDER BY date DESC LIMIT 10  -- The "gold" query
```

## Proposed Solution: Two-Tier History System

### 1. Session History (Volatile)
- **What**: All queries from current session
- **Lifecycle**: Cleared on restart
- **Purpose**: Quick iteration and refinement
- **Access**: Up/Down arrows (primary navigation)

### 2. Persistent History (Curated)
- **What**: "Starred" or explicitly saved queries
- **Lifecycle**: Persists across sessions
- **Purpose**: Keep the "gold" queries
- **Access**: Ctrl+R (fuzzy search), or dedicated history mode

## Implementation Approaches

### Approach A: Manual Curation
```
Commands:
- Ctrl+S (in Command mode): Star/save current query to persistent history
- Ctrl+D: Delete current history entry
- :history clean: Remove duplicates and near-duplicates
- :history star: Mark current as favorite
```

### Approach B: Smart Detection
```
Auto-promotion rules:
- Query executed >3 times → auto-save
- Query with successful results + high row count → suggest save
- Query unchanged for >5 minutes → likely "final version"
- Similarity detection: Keep only the longest/most complete version
```

### Approach C: Hybrid with Annotations
```
Features:
- Session history for everything
- Star/favorite system for keepers
- Comments/tags for queries: "-- @save Production query for daily report"
- Auto-dedupe on similarity (Levenshtein distance)
```

## Proposed UI/UX

### Quick Actions (While Navigating History)
```
[↑/↓ Navigate History]
Current: SELECT * FROM trades WHERE symbol = 'AAPL' AND price > 150 ORDER BY date DESC LIMIT 10

[Ctrl+S] Save to persistent  [Ctrl+D] Delete  [Ctrl+/] Add note
```

### History Modes
1. **Quick History** (default Up/Down):
   - Shows session + recent persistent
   - Auto-dedupes exact matches
   - Most recent first

2. **Full History** (Ctrl+R):
   - Fuzzy search across all
   - Shows star status
   - Groups by similarity

3. **Starred Only** (Ctrl+Shift+R):
   - Only saved queries
   - Organized by tags/categories

## Data Structure Enhancement

```rust
pub struct HistoryEntry {
    pub command: String,
    pub timestamp: DateTime<Utc>,
    pub execution_count: u32,
    pub success: bool,
    pub duration_ms: Option<u64>,
    
    // New fields
    pub session_id: String,        // To track session history
    pub is_starred: bool,          // Manually saved
    pub auto_saved: bool,          // Auto-promoted
    pub similarity_group: Option<String>,  // For deduplication
    pub tags: Vec<String>,         // User annotations
    pub note: Option<String>,      // User comment
}
```

## Deduplication Strategy

### Similarity Detection
1. **Exact match**: Remove older duplicates
2. **Prefix match**: If new query extends old, keep only new
3. **Levenshtein distance < 10%**: Group as variations
4. **Structural similarity**: Parse and compare AST

### Example Deduplication
```sql
-- These would be grouped:
SELECT * FROM trades WHERE symbol='AAPL'
SELECT * FROM trades WHERE symbol = 'AAPL'
SELECT * FROM trades WHERE   symbol = 'AAPL'

-- Keep only the final version:
SELECT * FROM trades WHERE symbol = 'AAPL' ORDER BY date
SELECT * FROM trades WHERE symbol = 'AAPL' ORDER BY date DESC  -- Keep this
```

## Implementation Phases

### Phase 1: Session History
- Add session_id to history entries
- Filter current session in Up/Down navigation
- Keep persistent history in Ctrl+R

### Phase 2: Star System
- Add Ctrl+S to star queries
- Add is_starred field
- Filter starred in special mode

### Phase 3: Smart Deduplication
- Implement similarity detection
- Auto-group related queries
- Show only "best" version by default

### Phase 4: Auto-Promotion
- Track execution patterns
- Auto-star frequently used queries
- Suggest saves for complex queries

## Benefits

1. **Clean Navigation**: Up/Down shows relevant queries, not noise
2. **Preserved Gold Queries**: Important queries never lost
3. **Natural Workflow**: Iterate freely without polluting history
4. **Smart Defaults**: System learns what's important
5. **Manual Control**: User can always override

## Configuration

```toml
[history]
# Session history size (cleared on restart)
session_size = 100

# Persistent history size
persistent_size = 1000

# Auto-save queries executed more than N times
auto_save_threshold = 3

# Deduplication similarity threshold (0-100)
similarity_threshold = 90

# Keep session history on exit
preserve_session = false

# Auto-deduplicate on navigation
auto_dedupe = true
```

## Quick Win Implementation

For immediate improvement:
1. Add session tracking (queries from this run)
2. Make Up/Down prefer session history
3. Add Ctrl+S to explicitly save good queries
4. On restart, only load starred/saved queries

This would solve 80% of the problem with minimal changes.
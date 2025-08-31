# Incremental State Manager Design

## Phase 1: Shell Struct with Centralized Flow

Start with a simple shell that **all state changes flow through**, with comprehensive logging and status line rendering.

## State Definition: Hierarchical Enums

```rust
/// Primary application mode
#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    /// User typing SQL queries
    Command(CommandSubState),
    
    /// Navigating query results
    Results(ResultsSubState),
    
    /// Special modes
    Help,
    Debug,
    PrettyQuery,
}

/// Command mode substates
#[derive(Debug, Clone, PartialEq)]
pub enum CommandSubState {
    Normal,                    // Regular typing
    TabCompletion,             // Tab completion active
    HistorySearch { pattern: String }, // Ctrl+R search
}

/// Results mode substates  
#[derive(Debug, Clone, PartialEq)]
pub enum ResultsSubState {
    Normal,                    // Regular navigation
    VimSearch(VimSearchState), // / search
    ColumnSearch { pattern: String }, // Column name search
    DataSearch { pattern: String },   // Data content search
    FuzzyFilter { pattern: String },  // Live filtering
    Selection(SelectionMode),  // Cell/row selection
    JumpToRow,                // Jump to row number
}

/// Vim search specific states
#[derive(Debug, Clone, PartialEq)]
pub enum VimSearchState {
    Typing { pattern: String },
    Navigating { 
        pattern: String, 
        current_match: usize,
        total_matches: usize,
    },
}

/// Selection modes
#[derive(Debug, Clone, PartialEq)]
pub enum SelectionMode {
    Cell,
    Row, 
    Column,
    Range,
}
```

## Shell State Manager

```rust
use tracing::{info, debug};

pub struct StateManager {
    current_state: AppState,
    previous_state: Option<AppState>,
    
    // State history for debugging
    state_history: VecDeque<(Instant, AppState, String)>, // (when, state, trigger)
    
    // Transition counter for debugging
    transition_count: usize,
}

impl StateManager {
    pub fn new() -> Self {
        let initial = AppState::Command(CommandSubState::Normal);
        info!(target: "state", "StateManager initialized with {:?}", initial);
        
        Self {
            current_state: initial.clone(),
            previous_state: None,
            state_history: VecDeque::with_capacity(100),
            transition_count: 0,
        }
    }
    
    /// Central state transition point - EVERYTHING flows through here
    pub fn set_state(&mut self, new_state: AppState, trigger: &str) {
        let old_state = self.current_state.clone();
        
        // Log every transition
        info!(target: "state", 
            "[#{}] State transition: {:?} -> {:?} (trigger: {})",
            self.transition_count, old_state, new_state, trigger
        );
        
        // Update state
        self.previous_state = Some(old_state.clone());
        self.current_state = new_state.clone();
        self.transition_count += 1;
        
        // Keep history (last 100 transitions)
        self.state_history.push_back((
            Instant::now(),
            new_state.clone(),
            trigger.to_string()
        ));
        if self.state_history.len() > 100 {
            self.state_history.pop_front();
        }
        
        // Log side effects needed
        self.log_required_side_effects(&old_state, &new_state);
    }
    
    /// Get current state
    pub fn current(&self) -> &AppState {
        &self.current_state
    }
    
    /// Check if we're in any search mode
    pub fn is_search_active(&self) -> bool {
        match &self.current_state {
            AppState::Results(sub) => match sub {
                ResultsSubState::VimSearch(_) |
                ResultsSubState::ColumnSearch { .. } |
                ResultsSubState::DataSearch { .. } |
                ResultsSubState::FuzzyFilter { .. } => true,
                _ => false,
            },
            _ => false,
        }
    }
    
    /// Get search type if active
    pub fn active_search_type(&self) -> Option<SearchType> {
        match &self.current_state {
            AppState::Results(ResultsSubState::VimSearch(_)) => Some(SearchType::Vim),
            AppState::Results(ResultsSubState::ColumnSearch { .. }) => Some(SearchType::Column),
            AppState::Results(ResultsSubState::DataSearch { .. }) => Some(SearchType::Data),
            AppState::Results(ResultsSubState::FuzzyFilter { .. }) => Some(SearchType::Fuzzy),
            _ => None,
        }
    }
    
    /// Get display string for status line
    pub fn status_display(&self) -> String {
        match &self.current_state {
            AppState::Command(sub) => match sub {
                CommandSubState::Normal => "COMMAND".to_string(),
                CommandSubState::TabCompletion => "COMMAND [Tab]".to_string(),
                CommandSubState::HistorySearch { pattern } => 
                    format!("COMMAND [History: {}]", pattern),
            },
            AppState::Results(sub) => match sub {
                ResultsSubState::Normal => "RESULTS".to_string(),
                ResultsSubState::VimSearch(vim) => match vim {
                    VimSearchState::Typing { pattern } => 
                        format!("RESULTS [/{}]", pattern),
                    VimSearchState::Navigating { current_match, total_matches, .. } =>
                        format!("RESULTS [Search {}/{}]", current_match + 1, total_matches),
                },
                ResultsSubState::ColumnSearch { pattern } => 
                    format!("RESULTS [Col: {}]", pattern),
                ResultsSubState::DataSearch { pattern } => 
                    format!("RESULTS [Find: {}]", pattern),
                ResultsSubState::FuzzyFilter { pattern } => 
                    format!("RESULTS [Filter: {}]", pattern),
                ResultsSubState::Selection(mode) => 
                    format!("RESULTS [Select: {:?}]", mode),
                ResultsSubState::JumpToRow => "RESULTS [Jump]".to_string(),
            },
            AppState::Help => "HELP".to_string(),
            AppState::Debug => "DEBUG".to_string(),
            AppState::PrettyQuery => "PRETTY SQL".to_string(),
        }
    }
    
    /// Get debug dump of state history
    pub fn debug_history(&self) -> String {
        let mut output = format!("State History (last 10 transitions):\n");
        for (time, state, trigger) in self.state_history.iter().rev().take(10) {
            output.push_str(&format!(
                "  {:?} ago: {:?} ({})\n", 
                time.elapsed(),
                state,
                trigger
            ));
        }
        output.push_str(&format!("\nTotal transitions: {}\n", self.transition_count));
        output
    }
    
    fn log_required_side_effects(&self, old: &AppState, new: &AppState) {
        debug!(target: "state", "Side effects needed:");
        
        // Entering Results mode
        if !matches!(old, AppState::Results(_)) && matches!(new, AppState::Results(_)) {
            debug!(target: "state", "  - Clear all search states");
            debug!(target: "state", "  - Reset viewport to (0,0)");
            debug!(target: "state", "  - Update key mapping to navigation");
        }
        
        // Exiting any search
        if self.was_search_active(old) && !self.is_state_search(new) {
            debug!(target: "state", "  - Clear search UI");
            debug!(target: "state", "  - Restore normal key mappings");
            debug!(target: "state", "  - Update status line");
        }
        
        // Entering search
        if !self.was_search_active(old) && self.is_state_search(new) {
            debug!(target: "state", "  - Clear other search states");
            debug!(target: "state", "  - Setup search UI");
            debug!(target: "state", "  - Capture input for search");
        }
    }
    
    fn was_search_active(&self, state: &AppState) -> bool {
        match state {
            AppState::Results(sub) => matches!(sub,
                ResultsSubState::VimSearch(_) |
                ResultsSubState::ColumnSearch { .. } |
                ResultsSubState::DataSearch { .. } |
                ResultsSubState::FuzzyFilter { .. }
            ),
            _ => false,
        }
    }
    
    fn is_state_search(&self, state: &AppState) -> bool {
        self.was_search_active(state)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SearchType {
    Vim,
    Column,
    Data,
    Fuzzy,
}
```

## Integration Points

### 1. Replace all `set_mode()` calls:

```rust
// BEFORE: Direct mode setting
self.buffer_mut().set_mode(AppMode::Results);

// AFTER: Through state manager
self.state_manager.set_state(
    AppState::Results(ResultsSubState::Normal),
    "execute_query"
);
```

### 2. Update action context:

```rust
// BEFORE: Complex multi-source check
has_search: !buffer.get_search_pattern().is_empty() 
    || self.vim_search_manager.borrow().is_active()
    || self.state_container.column_search().is_active

// AFTER: Single source
has_search: self.state_manager.is_search_active()
```

### 3. Status line rendering:

```rust
// Add state display to status line
let state_display = self.state_manager.status_display();
spans.push(Span::styled(
    format!(" [{}] ", state_display),
    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
));
```

### 4. Debug view (F5):

```rust
// Show state history in debug view
if self.show_debug {
    let history = self.state_manager.debug_history();
    // Render history in debug panel
}
```

## Migration Strategy

### Step 1: Add StateManager to EnhancedTuiApp
```rust
pub struct EnhancedTuiApp {
    // ... existing fields
    state_manager: StateManager,  // NEW
}
```

### Step 2: Replace one set_mode at a time
Start with the most common transitions:
1. Command → Results (execute query)
2. Results → Search modes
3. Search → Normal Results (escape)

### Step 3: Add logging and observe
With comprehensive logging, we can see:
- All state transitions
- Missing side effects
- Conflicting states
- Patterns to optimize

### Step 4: Fix issues incrementally
As we observe the logs:
- Add missing side effects
- Prevent invalid transitions
- Consolidate duplicate logic

## Benefits of This Approach

1. **Immediate Visibility**: See every state change in logs and status line
2. **Incremental Migration**: Replace set_mode() calls one at a time
3. **No Big Bang**: Existing code continues to work
4. **Debug-Friendly**: Complete state history for troubleshooting
5. **Single Source**: Even as a shell, provides single source of truth

## Next Step

Implement the basic StateManager struct and replace just ONE set_mode() call to validate the approach. Once we see it working with comprehensive logging, we can gradually migrate all 57 locations.
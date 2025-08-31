# State Combination: How It Actually Works

## The Complete State Definition

Our **total state** is a single enum value that includes everything. No tuples needed - the enum hierarchy handles it all.

## Concrete Examples

### Example 1: User typing a query
```rust
// The COMPLETE state is:
let state = AppState::Command(CommandSubState::Normal);

// This single value tells us:
// - Main mode: Command
// - Sub state: Normal (just typing)
// - NOT in results, NOT searching, NOT in help
```

### Example 2: User doing vim search in results
```rust
// The COMPLETE state is:
let state = AppState::Results(
    ResultsSubState::VimSearch(
        VimSearchState::Typing { 
            pattern: "active".to_string() 
        }
    )
);

// This single value tells us:
// - Main mode: Results
// - Sub state: VimSearch
// - Vim search state: Currently typing the pattern "active"
```

### Example 3: User navigating search results
```rust
// The COMPLETE state is:
let state = AppState::Results(
    ResultsSubState::VimSearch(
        VimSearchState::Navigating {
            pattern: "active".to_string(),
            current_match: 2,
            total_matches: 10,
        }
    )
);

// This single value tells us:
// - Main mode: Results  
// - Sub state: VimSearch
// - Vim search state: Navigating, on match 3 of 10
```

## How We Check State

### Pattern Matching - The Rust Way
```rust
impl StateManager {
    pub fn get_mode_info(&self) -> String {
        match &self.current_state {
            // Command mode cases
            AppState::Command(sub) => {
                match sub {
                    CommandSubState::Normal => 
                        "Typing SQL query".to_string(),
                    CommandSubState::TabCompletion => 
                        "Tab completion active".to_string(),
                    CommandSubState::HistorySearch { pattern } => 
                        format!("Searching history for: {}", pattern),
                }
            },
            
            // Results mode cases  
            AppState::Results(sub) => {
                match sub {
                    ResultsSubState::Normal => 
                        "Navigating results".to_string(),
                    
                    ResultsSubState::VimSearch(vim_state) => {
                        match vim_state {
                            VimSearchState::Typing { pattern } =>
                                format!("Typing search: /{}", pattern),
                            VimSearchState::Navigating { current_match, total_matches, .. } =>
                                format!("Search match {}/{}", current_match + 1, total_matches),
                        }
                    },
                    
                    ResultsSubState::ColumnSearch { pattern } =>
                        format!("Finding column: {}", pattern),
                    
                    ResultsSubState::FuzzyFilter { pattern } =>
                        format!("Filtering: {}", pattern),
                    
                    _ => "Results mode".to_string(),
                }
            },
            
            // Other modes
            AppState::Help => "Reading help".to_string(),
            AppState::Debug => "Debug view".to_string(),
            AppState::PrettyQuery => "Pretty SQL view".to_string(),
        }
    }
}
```

## Convenience Methods for Common Checks

```rust
impl StateManager {
    /// Are we in command mode at all?
    pub fn is_command_mode(&self) -> bool {
        matches!(self.current_state, AppState::Command(_))
    }
    
    /// Are we in results mode at all?
    pub fn is_results_mode(&self) -> bool {
        matches!(self.current_state, AppState::Results(_))
    }
    
    /// Are we in ANY search mode?
    pub fn is_search_active(&self) -> bool {
        match &self.current_state {
            AppState::Results(sub) => {
                matches!(sub,
                    ResultsSubState::VimSearch(_) |
                    ResultsSubState::ColumnSearch { .. } |
                    ResultsSubState::DataSearch { .. } |
                    ResultsSubState::FuzzyFilter { .. }
                )
            },
            _ => false,
        }
    }
    
    /// Get the search pattern if we're searching
    pub fn get_search_pattern(&self) -> Option<String> {
        match &self.current_state {
            AppState::Results(ResultsSubState::VimSearch(vim)) => {
                match vim {
                    VimSearchState::Typing { pattern } |
                    VimSearchState::Navigating { pattern, .. } => 
                        Some(pattern.clone()),
                }
            },
            AppState::Results(ResultsSubState::ColumnSearch { pattern }) |
            AppState::Results(ResultsSubState::DataSearch { pattern }) |
            AppState::Results(ResultsSubState::FuzzyFilter { pattern }) => 
                Some(pattern.clone()),
            _ => None,
        }
    }
    
    /// Check if 'N' key should navigate search or toggle line numbers
    pub fn should_n_key_navigate_search(&self) -> bool {
        // Only if we're in vim search navigation mode
        matches!(
            self.current_state,
            AppState::Results(ResultsSubState::VimSearch(VimSearchState::Navigating { .. }))
        )
    }
}
```

## Setting State - Real Examples

```rust
// Example: User presses '/' to start vim search
self.state_manager.set_state(
    AppState::Results(
        ResultsSubState::VimSearch(
            VimSearchState::Typing { pattern: String::new() }
        )
    ),
    "user_pressed_slash"
);

// Example: User presses Enter to execute query
self.state_manager.set_state(
    AppState::Results(ResultsSubState::Normal),
    "execute_query"
);

// Example: User presses Escape while searching
self.state_manager.set_state(
    AppState::Results(ResultsSubState::Normal),
    "escape_from_search"
);

// Example: User types in search
if let AppState::Results(ResultsSubState::VimSearch(VimSearchState::Typing { pattern })) = 
    &self.state_manager.current_state {
    
    let new_pattern = format!("{}a", pattern); // User typed 'a'
    self.state_manager.set_state(
        AppState::Results(
            ResultsSubState::VimSearch(
                VimSearchState::Typing { pattern: new_pattern }
            )
        ),
        "search_input_char"
    );
}
```

## The Power of This Approach

### 1. **Single Source of Truth**
```rust
// Our ENTIRE application state is ONE value:
let complete_state: AppState = self.state_manager.current_state;

// Not multiple booleans:
// ❌ is_command_mode && !is_searching && !has_completion && ...

// Just one enum:
// ✅ AppState::Command(CommandSubState::Normal)
```

### 2. **Impossible States Can't Exist**
```rust
// This is IMPOSSIBLE to represent:
// ❌ Command mode AND VimSearch active
// The type system prevents it!

// You can only have valid states:
// ✅ AppState::Command(CommandSubState::Normal)
// ✅ AppState::Results(ResultsSubState::VimSearch(...))
```

### 3. **Clear State Transitions**
```rust
// When user presses 'N' key:
match self.state_manager.current_state {
    AppState::Results(ResultsSubState::VimSearch(VimSearchState::Navigating { .. })) => {
        // We're navigating search results, so N = next match
        self.next_search_match();
    },
    _ => {
        // We're NOT in search navigation, so N = toggle line numbers
        self.toggle_line_numbers();
    }
}
// The 'N' key bug is FIXED by design!
```

## Visual Representation

```
AppState (Total State)
├── Command
│   ├── Normal ← Complete state: AppState::Command(CommandSubState::Normal)
│   ├── TabCompletion ← Complete state: AppState::Command(CommandSubState::TabCompletion)
│   └── HistorySearch { pattern } ← Complete state: AppState::Command(CommandSubState::HistorySearch { .. })
│
├── Results  
│   ├── Normal ← Complete state: AppState::Results(ResultsSubState::Normal)
│   ├── VimSearch
│   │   ├── Typing { pattern } ← Complete state: AppState::Results(ResultsSubState::VimSearch(VimSearchState::Typing { .. }))
│   │   └── Navigating { pattern, current, total } ← Complete state: AppState::Results(ResultsSubState::VimSearch(VimSearchState::Navigating { .. }))
│   ├── ColumnSearch { pattern } ← Complete state: AppState::Results(ResultsSubState::ColumnSearch { .. })
│   ├── DataSearch { pattern } ← Complete state: AppState::Results(ResultsSubState::DataSearch { .. })
│   └── FuzzyFilter { pattern } ← Complete state: AppState::Results(ResultsSubState::FuzzyFilter { .. })
│
├── Help ← Complete state: AppState::Help
├── Debug ← Complete state: AppState::Debug  
└── PrettyQuery ← Complete state: AppState::PrettyQuery
```

Each path from root to leaf is ONE complete state value!

## Summary

- **No tuples needed** - The enum hierarchy IS the complete state
- **One variable** - `self.current_state` contains EVERYTHING
- **Type safe** - Can't have invalid state combinations
- **Pattern matching** - Rust's match makes state checks elegant
- **Single source of truth** - Check one place for any state question
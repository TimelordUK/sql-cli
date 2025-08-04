# Cache Integration Example

## Adding Cache Commands to Enhanced TUI

### 1. Update EnhancedTuiApp struct

```rust
use crate::cache::{QueryCache, QueryMode};

pub struct EnhancedTuiApp {
    // ... existing fields ...
    query_cache: QueryCache,
    query_mode: QueryMode,
    cached_data_id: Option<u64>, // Currently loaded cache ID
}
```

### 2. Handle Cache Commands

Add to `handle_command_input()`:

```rust
fn handle_command_input(&mut self, input: &str) -> Result<()> {
    let parts: Vec<&str> = input.split_whitespace().collect();
    
    match parts.get(0) {
        Some(&"\\cache") => {
            match parts.get(1) {
                Some(&"list") => self.show_cache_list()?,
                Some(&"load") => {
                    if let Some(id_str) = parts.get(2) {
                        if let Ok(id) = id_str.parse::<u64>() {
                            self.load_cached_query(id)?;
                        }
                    }
                },
                Some(&"save") => {
                    let desc = parts[2..].join(" ");
                    self.save_current_to_cache(Some(desc))?;
                },
                Some(&"info") => self.show_cache_info()?,
                _ => self.status_message = "Usage: \\cache [list|load|save|info]".to_string(),
            }
        },
        Some(&"\\mode") => {
            match parts.get(1) {
                Some(&"live") => {
                    self.query_mode = QueryMode::Live;
                    self.status_message = "Mode: Live (querying server)".to_string();
                },
                Some(&"cached") => {
                    self.query_mode = QueryMode::Cached;
                    self.status_message = "Mode: Cached (offline)".to_string();
                },
                Some(&"hybrid") => {
                    self.query_mode = QueryMode::Hybrid;
                    self.status_message = "Mode: Hybrid (cache + server)".to_string();
                },
                _ => self.status_message = "Usage: \\mode [live|cached|hybrid]".to_string(),
            }
        },
        _ => {
            // Handle regular SQL queries
            self.execute_query(input)?;
        }
    }
    Ok(())
}
```

### 3. Execute Query with Cache Support

```rust
fn execute_query(&mut self, query: &str) -> Result<()> {
    match self.query_mode {
        QueryMode::Live => {
            // Current behavior - query server
            self.execute_server_query(query)?;
        },
        QueryMode::Cached => {
            // Only work with cached data
            if let Some(cache_id) = self.cached_data_id {
                self.apply_query_to_cached_data(query)?;
            } else {
                self.status_message = "No cached data loaded. Use \\cache load <id>".to_string();
            }
        },
        QueryMode::Hybrid => {
            // Check cache first
            if let Ok(cached_result) = self.query_cache.find_cached_query(query) {
                self.load_cached_result(cached_result)?;
                self.status_message = format!("Loaded from cache ({}ms)", 0);
            } else {
                // Fall back to server
                self.execute_server_query(query)?;
            }
        }
    }
    Ok(())
}
```

### 4. Cache List UI

```rust
fn show_cache_list(&mut self) -> Result<()> {
    let queries = self.query_cache.list_cached_queries();
    
    // Switch to cache browser mode
    self.mode = AppMode::CacheBrowser;
    self.cache_list_state = ListState::default();
    
    // Format cache entries for display
    self.cache_entries = queries.iter().map(|q| {
        format!("[{}] {} - {} rows - {}",
            q.id,
            q.description.as_ref().unwrap_or(&q.query_text[..50.min(q.query_text.len())]),
            q.row_count,
            q.timestamp.format("%Y-%m-%d %H:%M")
        )
    }).collect();
    
    Ok(())
}
```

### 5. Status Bar Update

Add cache status to the status bar:

```rust
fn render_status_bar(&self, f: &mut Frame, area: Rect) {
    let mode_indicator = match self.query_mode {
        QueryMode::Live => "LIVE",
        QueryMode::Cached => "CACHED",
        QueryMode::Hybrid => "HYBRID",
    };
    
    let cache_indicator = if let Some(id) = self.cached_data_id {
        format!(" [Cache: #{}]", id)
    } else {
        String::new()
    };
    
    let status = format!("{} | Mode: {}{} | {}",
        self.get_mode_string(),
        mode_indicator,
        cache_indicator,
        self.status_message
    );
    
    // ... render status ...
}
```

### 6. Example Usage Flow

```bash
# Start the CLI
$ sql-cli

# Switch to hybrid mode
sql> \mode hybrid

# Fetch large dataset
sql> SELECT * FROM trade_deal WHERE tradeDate > DateTime(2024, 01, 01)
# Returns: 8,543 rows (1250ms)

# Save to cache
sql> \cache save January 2024 trades

# List cached queries
sql> \cache list
[1] January 2024 trades - 8543 rows - 2024-08-04 17:45

# Work offline
sql> \mode cached
sql> \cache load 1

# Now filter the cached data locally (instant!)
sql> SELECT * FROM trade_deal WHERE counterparty.Contains("Bank")
# Returns: 2,341 rows (0ms) [CACHED]
```

## Benefits

1. **Performance**: Local queries are instant
2. **Cost Savings**: One expensive query, many local analyses
3. **Offline Work**: Continue analysis without connection
4. **History**: Keep important query results for comparison
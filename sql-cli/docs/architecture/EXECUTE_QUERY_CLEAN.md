# Clean execute_query Implementation

Replace the entire execute_query function with this simplified version:

```rust
fn execute_query(&mut self, query: &str) -> Result<()> {
    info!(target: "query", "Executing query: {}", query);
    
    // 1. Save query to buffer and state container
    self.buffer_mut().set_last_query(query.to_string());
    self.state_container.set_last_executed_query(query.to_string());
    
    // 2. Update status
    self.buffer_mut().set_status_message(format!("Executing query: '{}'...", query));
    let start_time = std::time::Instant::now();
    
    // 3. Execute query on DataView
    let result = if let Some(dataview) = self.buffer().get_dataview() {
        // Get the DataTable Arc (should add source_arc() method to DataView to avoid cloning)
        let table_arc = Arc::new(dataview.source().clone());
        let case_insensitive = self.buffer().is_case_insensitive();
        
        // Execute using QueryEngine
        let engine = crate::data::query_engine::QueryEngine::with_case_insensitive(case_insensitive);
        engine.execute(table_arc, query)
    } else {
        return Err(anyhow::anyhow!("No data loaded"));
    };
    
    // 4. Handle result
    match result {
        Ok(new_dataview) => {
            let duration = start_time.elapsed();
            let row_count = new_dataview.row_count();
            let col_count = new_dataview.column_count();
            
            // Store the new DataView in buffer
            self.buffer_mut().set_dataview(Some(new_dataview));
            
            // Update status
            self.buffer_mut().set_status_message(format!(
                "Query executed: {} rows, {} columns ({} ms)",
                row_count, col_count, duration.as_millis()
            ));
            
            // 5. Add to history
            let columns = self.buffer()
                .get_dataview()
                .map(|v| v.column_names())
                .unwrap_or_default();
            
            let table_name = self.buffer()
                .get_dataview()
                .map(|v| v.source().name.clone())
                .unwrap_or_else(|| "data".to_string());
            
            self.state_container
                .command_history_mut()
                .add_entry_with_schema(
                    query.to_string(),
                    true,  // success
                    Some(duration.as_millis() as u64),
                    columns,
                    Some(table_name),
                )?;
            
            // 6. Switch to results mode and reset navigation
            self.buffer_mut().set_mode(AppMode::Results);
            self.buffer_mut().set_selected_row(Some(0));
            self.buffer_mut().set_current_column(0);
            self.buffer_mut().set_scroll_offset((0, 0));
            
            Ok(())
        }
        Err(e) => {
            let error_msg = format!("Query error: {}", e);
            self.buffer_mut().set_status_message(error_msg.clone());
            
            // Add failed query to history
            self.state_container
                .command_history_mut()
                .add_entry(query.to_string(), false, None)?;
            
            Err(anyhow::anyhow!(error_msg))
        }
    }
}
```

## What this simplified version does:

1. **Save query** - Store in buffer and state container
2. **Execute query** - Use QueryEngine on the DataTable from DataView
3. **Store result** - Put the new DataView in the buffer
4. **Update history** - Add to command history with schema info
5. **Switch mode** - Go to Results mode to show the data

## What was removed:

- All `is_csv_mode()` checks - obsolete
- All `is_cache_mode()` checks - obsolete  
- All CSV client code - obsolete
- Cache checking logic - obsolete
- QueryResponse conversion - not needed, we use DataView directly
- Complex branching - now just one simple path
- `set_filtered_data` calls - DataView handles this
- Redundant memory tracking - can add back if needed

## Benefits:

- ~280 lines → ~80 lines
- Single clear path through the code
- No mode checking
- Direct DataView usage
- Much easier to understand and maintain
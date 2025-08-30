use crate::app_state_container::AppStateContainer;
use crate::data::data_view::DataView;
use crate::services::{QueryExecutionResult, QueryExecutionService};
use crate::ui::search::vim_search_adapter::VimSearchAdapter;
use anyhow::Result;
use std::cell::RefCell;
use tracing::info;

/// Orchestrates the entire query execution flow
/// This handles all the side effects and state management around query execution
pub struct QueryOrchestrator {
    query_execution_service: QueryExecutionService,
}

impl QueryOrchestrator {
    pub fn new(case_insensitive: bool, auto_hide_empty: bool) -> Self {
        Self {
            query_execution_service: QueryExecutionService::new(case_insensitive, auto_hide_empty),
        }
    }

    /// Execute a query with all necessary state management
    pub fn execute_query(
        &mut self,
        query: &str,
        state_container: &mut AppStateContainer,
        vim_search_adapter: &RefCell<VimSearchAdapter>,
    ) -> Result<QueryExecutionContext> {
        info!(target: "query", "Executing query: {}", query);

        // 1. Clear all search-related state before executing new query
        self.clear_all_search_state(state_container, vim_search_adapter);

        // 2. Record the query being executed
        self.record_query_execution(query, state_container);

        // 3. Set executing status
        state_container.set_status_message(format!("Executing query: '{}'...", query));

        // 4. Execute the query
        let current_dataview = state_container.get_buffer_dataview();
        let result = self
            .query_execution_service
            .execute(query, current_dataview)?;

        // 5. Clear any active filters (new query should start with clean state)
        self.clear_all_filters(state_container);

        // 6. Return the context for the TUI to apply
        Ok(QueryExecutionContext {
            result,
            query: query.to_string(),
        })
    }

    /// Clear all search-related state
    fn clear_all_search_state(
        &self,
        state_container: &mut AppStateContainer,
        vim_search_adapter: &RefCell<VimSearchAdapter>,
    ) {
        // Clear container search states
        state_container.clear_search();
        state_container.clear_column_search();

        // Clear vim search adapter state
        vim_search_adapter.borrow_mut().cancel_search();
    }

    /// Record that a query is being executed
    fn record_query_execution(&self, query: &str, state_container: &mut AppStateContainer) {
        state_container.set_last_query(query.to_string());
        state_container.set_last_executed_query(query.to_string());
    }

    /// Clear all filter-related state
    fn clear_all_filters(&self, state_container: &mut AppStateContainer) {
        state_container.set_filter_pattern(String::new());
        state_container.set_fuzzy_filter_pattern(String::new());
        state_container.set_filter_active(false);
        state_container.set_fuzzy_filter_active(false);
    }

    /// Update service configuration
    pub fn set_case_insensitive(&mut self, case_insensitive: bool) {
        self.query_execution_service
            .set_case_insensitive(case_insensitive);
    }

    pub fn set_auto_hide_empty(&mut self, auto_hide: bool) {
        self.query_execution_service.set_auto_hide_empty(auto_hide);
    }
}

/// Context returned from query execution
pub struct QueryExecutionContext {
    pub result: QueryExecutionResult,
    pub query: String,
}

impl QueryExecutionContext {
    /// Apply this context to the TUI state
    /// This is what the TUI will call to update its state after query execution
    pub fn apply_to_tui(
        self,
        state_container: &mut AppStateContainer,
        update_viewport: impl FnOnce(DataView),
        calculate_widths: impl FnOnce(),
        reset_table: impl FnOnce(),
    ) -> Result<()> {
        // Apply the new DataView
        state_container.set_dataview(Some(self.result.dataview.clone()));

        // Update viewport (delegate to TUI)
        update_viewport(self.result.dataview.clone());

        // Update navigation state
        state_container
            .update_data_size(self.result.stats.row_count, self.result.stats.column_count);

        // Calculate column widths (delegate to TUI)
        calculate_widths();

        // Update status message
        state_container.set_status_message(self.result.status_message());

        // Add to history
        state_container
            .command_history_mut()
            .add_entry_with_schema(
                self.query.clone(),
                true,
                Some(self.result.stats.execution_time.as_millis() as u64),
                self.result.column_names(),
                Some(self.result.table_name()),
            )?;

        // Switch to results mode
        state_container.set_mode(crate::buffer::AppMode::Results);

        // Reset table (delegate to TUI)
        reset_table();

        Ok(())
    }
}

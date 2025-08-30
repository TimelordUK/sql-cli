use crate::app_state_container::AppStateContainer;
use crate::buffer::BufferAPI;
use crate::ui::input::actions::{Action, YankTarget};
use crate::yank_manager::YankManager;
use anyhow::Result;
use tracing::debug;

pub struct YankHandler;

impl YankHandler {
    pub fn handle_yank_action(
        action: &Action,
        buffer: &dyn BufferAPI,
        state_container: &AppStateContainer,
    ) -> Result<Option<String>> {
        match action {
            Action::Yank(target) => {
                let result = Self::execute_yank(target, buffer, state_container)?;
                Ok(Some(result))
            }
            _ => Ok(None),
        }
    }

    fn execute_yank(
        target: &YankTarget,
        buffer: &dyn BufferAPI,
        state_container: &AppStateContainer,
    ) -> Result<String> {
        match target {
            YankTarget::Cell => Self::yank_cell(buffer, state_container),
            YankTarget::Row => Self::yank_row(buffer, state_container),
            YankTarget::Column => Self::yank_column(buffer, state_container),
            YankTarget::All => Self::yank_all(buffer, state_container),
            YankTarget::Query => Self::yank_query(buffer, state_container),
        }
    }

    fn yank_cell(buffer: &dyn BufferAPI, state_container: &AppStateContainer) -> Result<String> {
        debug!("yank_cell called");

        // Use buffer.get_selected_row() to get the actual table row selection
        if let Some(selected_row) = buffer.get_selected_row() {
            let column = buffer.get_current_column();
            debug!("Yanking cell at row={}, column={}", selected_row, column);

            let result = YankManager::yank_cell(buffer, state_container, selected_row, column)?;
            let message = format!("Yanked cell: {}", result.full_value);
            Ok(message)
        } else {
            debug!("No row selected for yank");
            Ok("No row selected".to_string())
        }
    }

    fn yank_row(buffer: &dyn BufferAPI, state_container: &AppStateContainer) -> Result<String> {
        // Use buffer.get_selected_row() to get the actual table row selection
        if let Some(selected_row) = buffer.get_selected_row() {
            let result = YankManager::yank_row(buffer, state_container, selected_row)?;
            let message = format!("Yanked {}", result.description);
            Ok(message)
        } else {
            Ok("No row selected".to_string())
        }
    }

    fn yank_column(buffer: &dyn BufferAPI, state_container: &AppStateContainer) -> Result<String> {
        let column = buffer.get_current_column();
        let result = YankManager::yank_column(buffer, state_container, column)?;
        let message = format!("Yanked {}", result.description);
        Ok(message)
    }

    fn yank_all(buffer: &dyn BufferAPI, state_container: &AppStateContainer) -> Result<String> {
        let result = YankManager::yank_all(buffer, state_container)?;
        let message = format!("Yanked {}: {}", result.description, result.preview);
        Ok(message)
    }

    fn yank_query(buffer: &dyn BufferAPI, state_container: &AppStateContainer) -> Result<String> {
        let query = buffer.get_input_text();

        if query.trim().is_empty() {
            return Ok("No query to yank".to_string());
        }

        // Just write to clipboard - AppStateContainer handles the rest
        state_container.write_to_clipboard(&query)?;

        let char_count = query.len();
        let status_msg = format!("Yanked SQL ({} chars)", char_count);
        debug!("Yanking query: {}", &status_msg);

        Ok(status_msg)
    }

    pub fn handle_yank_as_test_case(
        buffer: &dyn BufferAPI,
        state_container: &AppStateContainer,
    ) -> Result<String> {
        use crate::utils::debug_info::DebugInfo;

        let test_case = DebugInfo::generate_test_case(buffer);
        state_container.yank_test_case(test_case.clone())?;

        let message = format!(
            "Copied complete test case to clipboard ({} lines)",
            test_case.lines().count()
        );
        Ok(message)
    }

    pub fn handle_yank_debug_context(
        buffer: &dyn BufferAPI,
        state_container: &AppStateContainer,
    ) -> Result<String> {
        use crate::utils::debug_info::DebugInfo;

        let debug_context = DebugInfo::generate_debug_context(buffer);
        state_container.yank_debug_context(debug_context.clone())?;

        let message = format!(
            "Copied debug context to clipboard ({} lines)",
            debug_context.lines().count()
        );
        Ok(message)
    }
}

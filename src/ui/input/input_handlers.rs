// Input handlers extracted from EnhancedTuiApp
// Start simple and build up gradually

use crate::app_state_container::AppStateContainer;
use crate::buffer::{AppMode, BufferAPI, BufferManager};
use crate::ui::state::shadow_state::ShadowStateManager;
use crate::widgets::help_widget::HelpAction;
use crate::widgets::stats_widget::StatsAction;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::cell::RefCell;

/// Minimal context for debug/pretty query input handlers
/// We'll add more fields as we extract more handlers
pub struct DebugInputContext<'a> {
    pub buffer_manager: &'a mut BufferManager,
    pub debug_widget: &'a mut crate::widgets::debug_widget::DebugWidget,
    pub shadow_state: &'a RefCell<ShadowStateManager>,
}

/// Context for help input handler
pub struct HelpInputContext<'a> {
    pub buffer_manager: &'a mut BufferManager,
    pub help_widget: &'a mut crate::widgets::help_widget::HelpWidget,
    pub state_container: &'a AppStateContainer,
    pub shadow_state: &'a RefCell<ShadowStateManager>,
}

/// Context for column stats input handler
pub struct StatsInputContext<'a> {
    pub buffer_manager: &'a mut BufferManager,
    pub stats_widget: &'a mut crate::widgets::stats_widget::StatsWidget,
    pub shadow_state: &'a RefCell<ShadowStateManager>,
}

/// Handle debug mode input
pub fn handle_debug_input(ctx: &mut DebugInputContext, key: KeyEvent) -> Result<bool> {
    // Handle special keys for test case generation
    match key.code {
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            // Ctrl+C to quit
            return Ok(true);
        }
        KeyCode::Char('t') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            // Ctrl+T: "Yank as Test" - capture current session as test case
            // Note: For now we can't do this as it needs yank_as_test_case
            // We'll add this capability later
            if let Some(buffer) = ctx.buffer_manager.current_mut() {
                buffer.set_status_message(
                    "Test case yank not yet implemented in extracted handler".to_string(),
                );
            }
            return Ok(false);
        }
        KeyCode::Char('y') if key.modifiers.contains(KeyModifiers::SHIFT) => {
            // Shift+Y: Yank debug dump with context
            // Note: For now we can't do this as it needs yank_debug_with_context
            // We'll add this capability later
            if let Some(buffer) = ctx.buffer_manager.current_mut() {
                buffer.set_status_message(
                    "Debug yank not yet implemented in extracted handler".to_string(),
                );
            }
            return Ok(false);
        }
        _ => {}
    }

    // Let the widget handle navigation and exit
    if ctx.debug_widget.handle_key(key) {
        // Widget returned true - exit debug mode
        if let Some(buffer) = ctx.buffer_manager.current_mut() {
            ctx.shadow_state
                .borrow_mut()
                .set_mode(AppMode::Command, buffer, "debug_exit");
        }
    }

    Ok(false)
}

/// Handle pretty query mode input
pub fn handle_pretty_query_input(ctx: &mut DebugInputContext, key: KeyEvent) -> Result<bool> {
    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        return Ok(true);
    }

    // Let debug widget handle the key (includes scrolling and exit)
    if ctx.debug_widget.handle_key(key) {
        // Widget returned true - exit pretty query mode
        if let Some(buffer) = ctx.buffer_manager.current_mut() {
            ctx.shadow_state
                .borrow_mut()
                .set_mode(AppMode::Command, buffer, "pretty_query_exit");
        }
    }

    Ok(false)
}

/// Handle cache list mode input
pub fn handle_cache_list_input(ctx: &mut DebugInputContext, key: KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Ok(true),
        KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => {
            if let Some(buffer) = ctx.buffer_manager.current_mut() {
                ctx.shadow_state
                    .borrow_mut()
                    .set_mode(AppMode::Command, buffer, "cache_list_exit");
            }
        }
        _ => {}
    }
    Ok(false)
}

/// Handle help mode input  
pub fn handle_help_input(ctx: &mut HelpInputContext, key: KeyEvent) -> Result<bool> {
    // Use the new HelpWidget
    match ctx.help_widget.handle_key(key) {
        HelpAction::Exit => {
            exit_help(ctx);
        }
        HelpAction::ShowDebug => {
            // F5 was pressed in help - this is handled by the widget itself
        }
        _ => {
            // Other actions are handled internally by the widget
        }
    }
    Ok(false)
}

/// Helper function for help mode exit
fn exit_help(ctx: &mut HelpInputContext) {
    ctx.help_widget.on_exit();
    ctx.state_container.set_help_visible(false);
    // Scroll is automatically reset when help is hidden in state_container
    let mode = if let Some(buffer) = ctx.buffer_manager.current() {
        if buffer.has_dataview() {
            AppMode::Results
        } else {
            AppMode::Command
        }
    } else {
        AppMode::Command
    };
    if let Some(buffer) = ctx.buffer_manager.current_mut() {
        ctx.shadow_state
            .borrow_mut()
            .set_mode(mode, buffer, "help_exit");
    }
}

/// Handle column stats mode input
pub fn handle_column_stats_input(ctx: &mut StatsInputContext, key: KeyEvent) -> Result<bool> {
    match ctx.stats_widget.handle_key(key) {
        StatsAction::Quit => return Ok(true),
        StatsAction::Close => {
            if let Some(buffer) = ctx.buffer_manager.current_mut() {
                buffer.set_column_stats(None);
                ctx.shadow_state
                    .borrow_mut()
                    .set_mode(AppMode::Results, buffer, "stats_close");
            }
        }
        StatsAction::Continue | StatsAction::PassThrough => {}
    }
    Ok(false)
}

#!/usr/bin/env python3
"""
Script to integrate HistoryWidget into enhanced_tui.rs
"""

import re

# Read the original file
with open('src/enhanced_tui.rs.backup_v10', 'r') as f:
    content = f.read()

# 1. Add import
import_line = "use sql_cli::history::{CommandHistory, HistoryMatch};"
new_import = "use sql_cli::history::{CommandHistory, HistoryMatch};\nuse sql_cli::history_widget::{HistoryWidget, HistoryAction};"
content = content.replace(import_line, new_import)

# 2. Remove HistoryState struct
history_state_pattern = r'#\[derive\(Clone\)\]\nstruct HistoryState \{[^}]+\}'
content = re.sub(history_state_pattern, '// HistoryState moved to history_widget.rs', content)

# 3. Replace field in struct
content = content.replace(
    "    history_state: HistoryState,\n    command_history: CommandHistory,",
    "    history_widget: HistoryWidget,\n    command_history: CommandHistory,  // Keep for navigation"
)

# 4. Fix initialization
init_pattern = r'history_state: HistoryState \{[^}]+\},\s+command_history: CommandHistory::new\(\)\.unwrap_or_default\(\),'
init_replacement = '''command_history: CommandHistory::new().unwrap_or_default(),
            history_widget: HistoryWidget::new(CommandHistory::new().unwrap_or_default()),  // TODO: Share command_history'''
content = re.sub(init_pattern, init_replacement, content, flags=re.MULTILINE | re.DOTALL)

# 5. Fix History mode initialization  
history_init_pattern = r'// Special handling for History mode[^}]+self\.command_history\.get_all\(\)\.len\(\);[^}]+\}'
history_init_replacement = '''// Special handling for History mode - initialize history search
                if mode == AppMode::History {
                    self.history_widget.initialize();
                    self.buffer_mut().set_status_message(
                        "History mode: ↑/↓ navigate, Enter execute, Tab edit, / search".to_string()
                    );
                }'''
content = re.sub(history_init_pattern, history_init_replacement, content, flags=re.MULTILINE | re.DOTALL)

# 6. Replace handle_history_input method
handle_history_pattern = r'fn handle_history_input\(&mut self, key: crossterm::event::KeyEvent\) -> Result<bool> \{[^}]+\n    \}'
handle_history_replacement = '''fn handle_history_input(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        match self.history_widget.handle_key(key) {
            HistoryAction::Quit => return Ok(true),
            HistoryAction::Exit => {
                self.buffer_mut().set_mode(AppMode::Command);
            }
            HistoryAction::ExecuteCommand(cmd) => {
                self.set_input_text(cmd.clone());
                self.execute_query(&cmd)?;
                self.buffer_mut().set_mode(AppMode::Results);
            }
            HistoryAction::UseCommand(cmd) => {
                self.set_input_text(cmd);
                self.buffer_mut().set_mode(AppMode::Command);
                self.buffer_mut()
                    .set_status_message("Command loaded from history".to_string());
                // Reset scroll to show end of command
                self.input_scroll_offset = 0;
                self.update_horizontal_scroll(120);
            }
            HistoryAction::StartSearch => {
                // Already in history mode, this enables search within it
                self.buffer_mut()
                    .set_status_message("Type to search history".to_string());
            }
            HistoryAction::None => {}
        }
        Ok(false)
    }'''
content = re.sub(handle_history_pattern, handle_history_replacement, content, flags=re.MULTILINE | re.DOTALL)

# 7. Remove update_history_matches method
update_history_pattern = r'fn update_history_matches\(&mut self\) \{[^}]+\n    \}'
content = re.sub(update_history_pattern, '// update_history_matches moved to HistoryWidget', content, flags=re.MULTILINE | re.DOTALL)

# 8. Replace render_history method
render_history_pattern = r'fn render_history\(&self, f: &mut Frame, area: Rect\) \{[^}]+self\.render_selected_command_preview[^;]+;\s+\}'
render_history_replacement = '''fn render_history(&self, f: &mut Frame, area: Rect) {
        self.history_widget.render(f, area);
    }'''
content = re.sub(render_history_pattern, render_history_replacement, content, flags=re.MULTILINE | re.DOTALL)

# 9. Remove render_history_list and render_selected_command_preview
render_list_pattern = r'fn render_history_list\(&self, f: &mut Frame, area: Rect\) \{[^}]+\n    \}'
content = re.sub(render_list_pattern, '', content, flags=re.MULTILINE | re.DOTALL)

render_preview_pattern = r'fn render_selected_command_preview\(&self, f: &mut Frame, area: Rect\) \{[^}]+\n    \}'
content = re.sub(render_preview_pattern, '// render_history_list and render_selected_command_preview moved to HistoryWidget', content, flags=re.MULTILINE | re.DOTALL)

# Write the result
with open('src/enhanced_tui.rs', 'w') as f:
    f.write(content)

print("Integration complete! File written to src/enhanced_tui.rs")
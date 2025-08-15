use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use sql_cli::app_state_container::SelectionMode;
use sql_cli::buffer::AppMode;
use sql_cli::ui::actions::{Action, ActionContext};
use sql_cli::ui::key_mapper::KeyMapper;
use std::io::{self, Write};

fn format_key(key: &KeyEvent) -> String {
    let mut result = String::new();

    // Add modifiers
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        result.push_str("Ctrl+");
    }
    if key.modifiers.contains(KeyModifiers::ALT) {
        result.push_str("Alt+");
    }
    if key.modifiers.contains(KeyModifiers::SHIFT) {
        result.push_str("Shift+");
    }

    // Add key code
    match key.code {
        KeyCode::Char(c) => result.push(c),
        KeyCode::F(n) => result.push_str(&format!("F{}", n)),
        KeyCode::Up => result.push_str("↑"),
        KeyCode::Down => result.push_str("↓"),
        KeyCode::Left => result.push_str("←"),
        KeyCode::Right => result.push_str("→"),
        KeyCode::PageUp => result.push_str("PgUp"),
        KeyCode::PageDown => result.push_str("PgDn"),
        KeyCode::Home => result.push_str("Home"),
        KeyCode::End => result.push_str("End"),
        KeyCode::Enter => result.push_str("Enter"),
        KeyCode::Tab => result.push_str("Tab"),
        KeyCode::Backspace => result.push_str("Bksp"),
        KeyCode::Delete => result.push_str("Del"),
        KeyCode::Esc => result.push_str("Esc"),
        _ => result.push_str("?"),
    }

    result
}

fn format_action(action: &Action) -> String {
    match action {
        Action::Navigate(nav) => format!("Navigate({:?})", nav),
        Action::ToggleSelectionMode => "ToggleSelectionMode".to_string(),
        Action::Quit => "Quit".to_string(),
        Action::ForceQuit => "ForceQuit".to_string(),
        Action::ShowHelp => "ShowHelp".to_string(),
        Action::ShowDebugInfo => "ShowDebugInfo".to_string(),
        Action::ToggleColumnPin => "ToggleColumnPin".to_string(),
        Action::Sort(col) => format!("Sort({:?})", col),
        Action::ExitCurrentMode => "ExitCurrentMode".to_string(),
        _ => format!("{:?}", action),
    }
}

fn main() -> io::Result<()> {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║         SQL CLI Action System Logger (Simple Version)        ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║ Press keys to see how they map to actions.                  ║");
    println!("║ Try: j, k, h, l, 5j, v, p, s, F1, arrows, etc.             ║");
    println!("║ Press Ctrl+C or 'q' to quit.                                ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    enable_raw_mode()?;

    let mut key_mapper = KeyMapper::new();
    let mut count_display = String::new();

    loop {
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Filter out key release events
                if key.kind != crossterm::event::KeyEventKind::Press {
                    continue;
                }

                let key_str = format_key(&key);

                // Build context
                let context = ActionContext {
                    mode: AppMode::Results,
                    selection_mode: SelectionMode::Row,
                    has_results: true,
                    has_filter: false,
                    has_search: false,
                    row_count: 100,
                    column_count: 10,
                    current_row: 5,
                    current_column: 2,
                };

                // Check count buffer before
                let count_before = key_mapper.get_count_buffer().to_string();
                let was_collecting = !count_before.is_empty();

                // Map the key
                let action = key_mapper.map_key(key, &context);

                // Check count buffer after
                let count_after = key_mapper.get_count_buffer().to_string();
                let is_collecting = !count_after.is_empty();

                // Format output
                if was_collecting && !is_collecting && action.is_some() {
                    // Count was applied
                    println!(
                        "│ {:6} │ Count: {:3} │ => {}",
                        key_str,
                        count_before,
                        format_action(action.as_ref().unwrap())
                    );
                } else if is_collecting {
                    // Building count
                    count_display = count_after.clone();
                    println!("│ {:6} │ Building count: {} │", key_str, count_display);
                } else if let Some(ref act) = action {
                    // Normal action
                    println!("│ {:6} │ => {}", key_str, format_action(act));
                } else {
                    // No mapping
                    println!("│ {:6} │ (no mapping in Results mode)", key_str);
                }
                
                // Flush stdout to ensure each line appears immediately
                io::stdout().flush().unwrap();

                // Check for quit
                if matches!(action, Some(Action::Quit) | Some(Action::ForceQuit)) {
                    break;
                }
            }
        }
    }

    disable_raw_mode()?;
    println!();
    println!("Goodbye!");
    Ok(())
}

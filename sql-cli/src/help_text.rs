use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;

/// Manages help text content for the TUI
/// Extracted from the monolithic enhanced_tui.rs
pub struct HelpText;

impl HelpText {
    /// Get the left column content for help display
    pub fn left_column() -> Vec<Line<'static>> {
        vec![
            Line::from("SQL CLI Help - Enhanced Features 🚀").style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Line::from(""),
            Line::from("COMMAND MODE").style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Line::from("  Enter    - Execute query"),
            Line::from("  Tab      - Auto-complete"),
            Line::from("  Ctrl+R   - Search history"),
            Line::from("  Ctrl+P   - Previous command in history"),
            Line::from("  Ctrl+N   - Next command in history"),
            Line::from("  Alt+↑    - Previous command (alternative)"),
            Line::from("  Alt+↓    - Next command (alternative)"),
            Line::from("  Ctrl+X   - Expand SELECT * to columns"),
            Line::from("  F3       - (Multi-line mode removed)"),
            Line::from(""),
            Line::from("NAVIGATION").style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Line::from("  Ctrl+A   - Beginning of line"),
            Line::from("  Ctrl+E   - End of line"),
            Line::from("  Ctrl+←   - Move backward word"),
            Line::from("  Ctrl+→   - Move forward word"),
            Line::from("  Alt+B    - Move backward word (bash-style)"),
            Line::from("  Alt+F    - Move forward word (bash-style)"),
            Line::from(""),
            Line::from("EDITING").style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Line::from("  Ctrl+W   - Delete word backward"),
            Line::from("  Alt+D    - Delete word forward"),
            Line::from("  Ctrl+K   - Kill to end of line"),
            Line::from("  Ctrl+U   - Kill to beginning of line"),
            Line::from("  F9       - Kill to end (Ctrl+K alternative)"),
            Line::from("  F10      - Kill to beginning (Ctrl+U alternative)"),
            Line::from("  Ctrl+Y   - Yank (paste from kill ring)"),
            Line::from("  Ctrl+V   - Paste from system clipboard"),
            Line::from("  Ctrl+Z   - Undo"),
            Line::from("  Alt+[    - Jump to previous SQL token"),
            Line::from("  Alt+]    - Jump to next SQL token"),
            Line::from(""),
            Line::from("BUFFER MANAGEMENT").style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Line::from("  F11/Ctrl+PgUp - Previous buffer"),
            Line::from("  F12/Ctrl+PgDn - Next buffer"),
            Line::from("  Ctrl+6        - Quick switch"),
            Line::from("  Alt+N         - New buffer"),
            Line::from("  Alt+W         - Close buffer"),
            Line::from("  Alt+B         - List buffers"),
            Line::from(""),
            Line::from("VIEW MODES").style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Line::from("  F1/?     - Toggle this help"),
            Line::from("  F5       - Debug info"),
            Line::from("  F6       - Pretty query view"),
            Line::from("  F7       - Cache management"),
            Line::from("  F8       - Case-insensitive"),
            Line::from("  ↓        - Enter results mode"),
            Line::from("  Ctrl+C/q - Exit"),
            Line::from(""),
            Line::from("CACHE COMMANDS").style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Line::from("  :cache save [id] - Save with ID"),
            Line::from("  :cache load ID   - Load by ID"),
            Line::from("  :cache list      - Show cached"),
            Line::from("  :cache clear     - Disable cache"),
            Line::from(""),
            Line::from("🌟 FEATURES").style(
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Line::from("  • Column statistics (S key)"),
            Line::from("  • Column pinning (p/P keys)"),
            Line::from("  • Dynamic column sizing"),
            Line::from("  • Compact mode (C key)"),
            Line::from("  • Rainbow parentheses"),
            Line::from("  • Auto-execute CSV/JSON"),
            Line::from("  • Multi-source indicators"),
            Line::from("  • LINQ-style null checking"),
            Line::from("  • Named cache IDs"),
            Line::from("  • Row numbers (N key)"),
            Line::from("  • Jump to row (: key)"),
        ]
    }

    /// Get the right column content for help display
    pub fn right_column() -> Vec<Line<'static>> {
        vec![
            Line::from("Use ↓/↑ or j/k to scroll help").style(Style::default().fg(Color::DarkGray)),
            Line::from(""),
            Line::from("RESULTS NAVIGATION").style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Line::from("  j/↓      - Next row"),
            Line::from("  k/↑      - Previous row"),
            Line::from("  h/←      - Previous column"),
            Line::from("  l/→      - Next column"),
            Line::from("  g        - First row"),
            Line::from("  G        - Last row"),
            Line::from("  H        - Top of viewport"),
            Line::from("  M        - Middle of viewport"),
            Line::from("  L        - Bottom of viewport"),
            Line::from("  0/^      - First column"),
            Line::from("  $        - Last column"),
            Line::from("  PgDn     - Page down"),
            Line::from("  PgUp     - Page up"),
            Line::from(""),
            Line::from("RESULTS FEATURES").style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Line::from("  C        - 🎯 Toggle compact"),
            Line::from("  N        - 🔢 Toggle row nums"),
            Line::from("  :        - 📍 Jump to row"),
            Line::from("  Space    - 🔒 Toggle viewport lock"),
            Line::from("  x/X      - 🔒 Toggle cursor lock (alternative)"),
            Line::from("  Ctrl+Space - 🔒 Toggle cursor lock"),
            Line::from("  p        - 📌 Pin/unpin column"),
            Line::from("  P        - Clear all pins"),
            Line::from("  /        - Search in results"),
            Line::from("  \\        - Search column names"),
            Line::from("  n/N      - Next/prev match"),
            Line::from("  Shift+F  - Filter rows (regex)"),
            Line::from("  f        - Fuzzy filter rows"),
            Line::from("  'text    - Exact match filter"),
            Line::from("             (matches highlighted)"),
            Line::from("  v        - Toggle cell/row mode"),
            Line::from("  s        - Sort by column"),
            Line::from("  S        - 📊 Column statistics"),
            Line::from("  1-9      - Sort by column #"),
            Line::from("  y        - Yank (cell mode: yank cell)"),
            Line::from("    yy     - Yank current row (row mode)"),
            Line::from("    yc     - Yank current column"),
            Line::from("    ya     - Yank all data"),
            Line::from("    yq     - Yank current query"),
            Line::from("  Ctrl+E   - Export to CSV"),
            Line::from("  Ctrl+J   - Export to JSON"),
            Line::from("  ↑/Esc    - Back to command"),
            Line::from("  q        - Quit"),
            Line::from(""),
            Line::from("SEARCH/FILTER").style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Line::from("  Enter    - Apply"),
            Line::from("  Esc      - Cancel"),
            Line::from(""),
            Line::from("DEBUG MODE (F5)")
                .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Line::from("  g/G      - Go to top/bottom"),
            Line::from("  j/k      - Scroll up/down"),
            Line::from("  PgUp/Dn  - Page up/down"),
            Line::from("  Ctrl+T   - Yank as test case ✨"),
            Line::from("  Shift+Y  - Yank debug context ✨"),
            Line::from("  Esc/q    - Exit debug mode"),
            Line::from(""),
            Line::from("💡 TIPS").style(
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Line::from("  • Load CSV: sql-cli data.csv"),
            Line::from("  • Press C for compact view"),
            Line::from("  • Press N for row numbers"),
            Line::from("  • Press : then 200 → row 200"),
            Line::from("  • Space locks viewport"),
            Line::from("  • Columns auto-adjust width"),
            Line::from("  • Named: :cache save q1"),
            Line::from("  • f + 'ubs = exact 'ubs' match"),
            Line::from("  • \\ + name = find column by name"),
            Line::from("  • F5 + Ctrl+T = Auto-generate tests!"),
            Line::from(""),
            Line::from("📦 Cache 📁 File 🌐 API 🗄️ SQL 🧪 Test"),
        ]
    }
}

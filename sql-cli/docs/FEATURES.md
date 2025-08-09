# SQL CLI Features

A powerful terminal-based SQL client with advanced features inspired by csvlens, mcfly, and modern TUI applications.

## Core Features

### 1. **Advanced SQL Parser with Tab Completion**
- Context-aware tab completion for columns, tables, and SQL keywords
- Smart completion after AND/OR operators maintains WHERE context
- Method completion for Dynamic LINQ-style queries (e.g., `column.Contains("value")`)
- DateTime constructor support for date comparisons
- Recursive descent parser for robust SQL understanding

### 2. **csvlens-Style Table Navigation**
- Vim-like keybindings (hjkl) for navigation
- Virtual scrolling for large datasets (1000+ rows, 190+ columns)
- Column-based navigation with h/l keys
- Page up/down support
- Jump to first/last row with g/G

### 3. **Data Manipulation**
- **Search**: `/` to search, `n`/`N` for next/previous match
- **Filter**: `F` to filter rows using regex patterns
- **Sort**: `s` to sort by current column, `1-9` for direct column sorting
- **Export**: `Ctrl+S` to export results to CSV with proper escaping

### 4. **mcfly-Style Command History**
- `Ctrl+R` for fuzzy search through command history
- Preview panel showing full command with syntax highlighting
- Execution time and success/failure tracking
- Persistent history across sessions

### 5. **SQL Syntax Highlighting**
- Real-time syntax highlighting in command input
- Color-coded keywords, strings, operators, and identifiers
- Syntax highlighting in history preview

### 6. **Advanced Cursor Navigation**
- `Ctrl+A` - Jump to beginning of line
- `Ctrl+E` - Jump to end of line
- Horizontal scrolling for long queries
- Proper cursor positioning even with 300+ character queries

### 7. **Developer Tools**
- `F5` - Parser debug mode with AST visualization
- Clipboard integration for debug output
- Schema configuration via JSON file
- Environment variable support (TRADE_API_URL)

### 8. **TUI Features**
- Multiple modes: Command, Results, Search, Filter, History, Debug
- Status bar with helpful messages
- Help screen (`F1` or `?`)
- Responsive layout with proper terminal handling
- CPU-efficient rendering (20fps cap)

### 9. **Error Handling**
- Graceful fallback to classic mode on TTY issues
- Proper terminal restoration on exit
- Clear error messages in status bar

## Keyboard Shortcuts

### Command Mode
- `Enter` - Execute query
- `Tab` - Auto-complete
- `Ctrl+R` - Search command history
- `Ctrl+A` - Jump to beginning of line
- `Ctrl+E` - Jump to end of line
- `↓` - Enter results mode
- `F1`/`?` - Toggle help
- `F5` - Debug parser
- `Ctrl+C`/`Ctrl+D` - Exit

### Results Mode
- `j`/`↓` - Next row
- `k`/`↑` - Previous row
- `h`/`←` - Previous column
- `l`/`→` - Next column
- `g` - First row
- `G` - Last row
- `Ctrl+F` - Page down
- `Ctrl+B` - Page up
- `/` - Search
- `n` - Next match
- `N` - Previous match
- `F` - Filter rows
- `s` - Sort by current column
- `1-9` - Sort by column number
- `Ctrl+S` - Export to CSV
- `↑`/`Esc` - Back to command mode
- `q` - Quit

## Technical Architecture
- Built with Rust for performance and safety
- Uses ratatui for terminal UI
- Recursive descent parser for SQL analysis
- Async HTTP client for API communication
- Persistent command history with fuzzy matching
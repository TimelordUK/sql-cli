# Neovim Plugin Logging System

**Date**: 2025-10-04
**Status**: ✅ Implemented

## Overview

Comprehensive file-based logging system for the SQL CLI Neovim plugin. Logs are written to the same directory as the TUI logs (`~/.local/share/sql-cli/logs/`) with automatic rotation and cleanup.

## Features

### Core Functionality
- **File-based logging** - All logs written to timestamped files
- **Automatic rotation** - New log file per Neovim session
- **Configurable retention** - Automatically deletes old log files (default: keep 20 most recent)
- **Log levels** - TRACE, DEBUG, INFO, WARN, ERROR
- **Buffered writes** - Performance-optimized with configurable buffer size
- **Auto-flush** - Periodic flushing with configurable interval
- **Thread-safe** - Safe for concurrent access
- **Cross-platform** - Works on Unix and Windows

### Integration
- **Unified log directory** - Same location as TUI logs
- **Symlink to latest** - `latest.log` always points to current session
- **Easy access** - Commands to open, tail, and locate log files
- **Module-level logging** - Each module logs with its own target/context

## Configuration

### Default Settings

```lua
require('sql-cli').setup({
  logging = {
    enabled = true,                -- Enable file logging
    level = "INFO",                -- Log level: TRACE, DEBUG, INFO, WARN, ERROR
    max_files = 20,                -- Maximum number of log files to keep
    buffer_size = 100,             -- Messages to buffer before flush
    auto_flush_interval = 5000,    -- Auto-flush every 5 seconds (ms)
  },
})
```

### Changing Log Level

For debugging, set level to DEBUG or TRACE:

```lua
require('sql-cli').setup({
  logging = {
    level = "DEBUG",  -- More verbose logging
  },
})
```

### Disabling Logging

```lua
require('sql-cli').setup({
  logging = {
    enabled = false,  -- Completely disable logging
  },
})
```

## Usage

### Commands

| Command | Description |
|---------|-------------|
| `:SqlCliOpenLog` | Open current log file in a split |
| `:SqlCliTailLog` | Tail log file with live updates |
| `:SqlCliLogPath` | Show and copy log path to clipboard |

### File Locations

**Unix/Linux:**
```
~/.local/share/sql-cli/logs/nvim-plugin_YYYYMMDD_HHMMSS.log
~/.local/share/sql-cli/logs/latest.log (symlink)
```

**Windows:**
```
%LOCALAPPDATA%\sql-cli\logs\nvim-plugin_YYYYMMDD_HHMMSS.log
%LOCALAPPDATA%\sql-cli\logs\latest.log (text pointer file)
```

### Viewing Logs

#### In Terminal
```bash
# View current log
cat ~/.local/share/sql-cli/logs/latest.log

# Tail current log
tail -f ~/.local/share/sql-cli/logs/latest.log

# List all plugin logs
ls -lh ~/.local/share/sql-cli/logs/nvim-plugin*.log

# Search logs
grep ERROR ~/.local/share/sql-cli/logs/nvim-plugin*.log
```

#### In Neovim
```vim
:SqlCliOpenLog    " Open in split
:SqlCliTailLog    " Tail with auto-refresh
```

## For Developers

### Adding Logging to a Module

```lua
-- At top of module
local M = {}

-- Get logger (lazy load to avoid circular dependencies)
local logger = nil
local function get_logger()
  if not logger then
    logger = require('sql-cli').logger
  end
  return logger
end

-- Use in functions
function M.some_function(arg)
  local log = get_logger()
  if log then
    log.info('my_module', 'some_function called with arg: ' .. tostring(arg))
  end

  -- ... function logic ...

  if some_error then
    if log then
      log.error('my_module', 'Error occurred: ' .. error_msg)
    end
  end
end
```

### Log Level Functions

```lua
local log = require('sql-cli').logger

-- Different log levels
log.trace('module', 'Very detailed trace message')
log.debug('module', 'Debug information')
log.info('module', 'Informational message')
log.warn('module', 'Warning message')
log.error('module', 'Error message')

-- Auto-detect module context
log.auto('INFO', 'Message with auto-detected module')

-- Log a table
log.log_table('DEBUG', 'module', some_table, 'table_name')
```

### Direct Logger Access

```lua
-- Access logger from any module
local sql_cli = require('sql-cli')
local log = sql_cli.logger

-- Check if logging is enabled
if log and log.config.enabled then
  log.info('init', 'Logging is active')
end

-- Flush immediately
log.flush()

-- Get log file path
local path = log.get_log_path()
```

### Best Practices

1. **Use appropriate log levels**:
   - `TRACE`: Very detailed flow tracking
   - `DEBUG`: Diagnostic information
   - `INFO`: General informational messages
   - `WARN`: Potentially problematic situations
   - `ERROR`: Error events

2. **Use meaningful targets**:
   ```lua
   -- Good
   log.info('executor', 'Executing query: ' .. query)
   log.debug('fuzzy_filter', 'Parsed ' .. row_count .. ' rows')

   -- Bad
   log.info('general', 'Doing something')
   log.info('unknown', 'An event occurred')
   ```

3. **Lazy load logger**:
   - Prevents circular dependencies
   - Doesn't fail if logging is disabled
   - Always check if logger exists before using

4. **Log meaningful context**:
   ```lua
   -- Good
   log.error('parser', 'Failed to parse query at line ' .. line .. ': ' .. error)

   -- Bad
   log.error('parser', 'Parse error')
   ```

5. **Use structured logging**:
   ```lua
   log.info('query', string.format('Query executed in %.2fms, returned %d rows',
     elapsed_ms, row_count))
   ```

## Log Format

```
[HH:MM:SS] LEVEL [target] message
```

Example:
```
[14:32:15] INFO [executor] Executing query: SELECT * FROM data
[14:32:15] DEBUG [fuzzy_filter] Parsed table with 1000 rows, 5 columns
[14:32:16] WARN [parser] Query contains deprecated syntax
[14:32:17] ERROR [loader] Failed to load data file: file not found
```

## Automatic Cleanup

The logger automatically:
- Deletes old log files when count exceeds `max_files`
- Keeps the most recent logs based on modification time
- Runs cleanup on initialization
- Logs cleanup activity

Example:
```
[14:30:00] INFO [logger] SQL CLI Neovim Plugin logging initialized
[14:30:00] DEBUG [logger] Deleted old log file: nvim-plugin_20250901_120000.log
[14:30:00] DEBUG [logger] Deleted old log file: nvim-plugin_20250902_080000.log
```

## Performance

### Buffering
- Logs are buffered in memory (default: 100 messages)
- Automatically flushed when buffer is full
- Auto-flush timer ensures periodic writes (default: 5 seconds)
- Manual flush available via `log.flush()`

### Impact
- Minimal performance impact with buffering
- File I/O only on flush events
- No overhead when logging is disabled

## Troubleshooting

### Log File Not Created

1. **Check if logging is enabled**:
   ```lua
   :lua print(require('sql-cli').logger.config.enabled)
   ```

2. **Check directory permissions**:
   ```bash
   ls -ld ~/.local/share/sql-cli/logs/
   ```

3. **Check for initialization errors**:
   ```vim
   :messages
   ```

### Logs Not Appearing

1. **Check log level**:
   ```lua
   :lua print(require('sql-cli').logger.config.level)
   ```

2. **Manually flush**:
   ```lua
   :lua require('sql-cli').logger.flush()
   ```

3. **Check buffer hasn't filled**:
   - Increase buffer_size or decrease auto_flush_interval

### Too Many Log Files

1. **Reduce retention**:
   ```lua
   require('sql-cli').setup({
     logging = { max_files = 10 }
   })
   ```

2. **Manual cleanup**:
   ```bash
   cd ~/.local/share/sql-cli/logs
   ls -t nvim-plugin*.log | tail -n +11 | xargs rm
   ```

## Comparison with TUI Logs

| Aspect | TUI Logs | Plugin Logs |
|--------|----------|-------------|
| Location | `~/.local/share/sql-cli/logs/sql-cli_*.log` | `~/.local/share/sql-cli/logs/nvim-plugin_*.log` |
| Language | Rust (tracing crate) | Lua (custom logger) |
| Rotation | Per TUI session | Per Neovim session |
| Format | Same format | Same format |
| Viewing | F5 in TUI + file | Commands + file |

Both log to the same directory for centralized troubleshooting.

## Examples

### Example 1: Debugging Query Execution

```lua
-- In executor.lua
local log = get_logger()

function execute_query(query, config, state)
  if log then
    log.info('executor', 'Executing query: ' .. query)
    log.debug('executor', 'Config: output_format=' .. config.output_format)
  end

  local start_time = vim.loop.hrtime()

  -- Execute query...

  local elapsed = (vim.loop.hrtime() - start_time) / 1000000  -- Convert to ms

  if log then
    log.info('executor', string.format('Query completed in %.2fms', elapsed))
  end
end
```

**Log output:**
```
[14:45:00] INFO [executor] Executing query: SELECT * FROM sales WHERE amount > 100
[14:45:00] DEBUG [executor] Config: output_format=table
[14:45:00] INFO [executor] Query completed in 23.45ms
```

### Example 2: Debugging Fuzzy Filter

```lua
-- In fuzzy_filter.lua
function M.open_fuzzy_finder(bufnr)
  local log = get_logger()
  if log then
    log.debug('fuzzy_filter', 'Opening fuzzy finder for buffer ' .. bufnr)
  end

  local data = parse_table_from_buffer(bufnr)
  if not data then
    if log then
      log.warn('fuzzy_filter', 'No table found in buffer ' .. bufnr)
    end
    return
  end

  if log then
    log.info('fuzzy_filter', string.format('Parsed table: %d rows, %d columns',
      #data.rows, #data.headers))
    log.debug('fuzzy_filter', 'Match mode: ' .. M.config.match_mode)
  end

  -- Continue...
end
```

**Log output:**
```
[15:00:00] DEBUG [fuzzy_filter] Opening fuzzy finder for buffer 3
[15:00:00] INFO [fuzzy_filter] Parsed table: 1000 rows, 5 columns
[15:00:00] DEBUG [fuzzy_filter] Match mode: exact
```

## Related Files

- `nvim-plugin/lua/sql-cli/logger.lua` - Logger implementation
- `nvim-plugin/lua/sql-cli/init.lua` - Initialization and commands
- `nvim-plugin/lua/sql-cli/config.lua` - Configuration defaults
- `nvim-plugin/test_logger.lua` - Test suite

## See Also

- [FUZZY_FILTER_EXACT_MODE.md](FUZZY_FILTER_EXACT_MODE.md) - Uses logging for debug
- TUI logging in `src/utils/logging.rs` - Similar concepts in Rust

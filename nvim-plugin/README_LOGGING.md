# SQL CLI Plugin Logging - Quick Reference

## For Users

### View Logs
```vim
:SqlCliOpenLog     " Open in split
:SqlCliTailLog     " Live tail
:SqlCliLogPath     " Show path and copy to clipboard
```

### Configure Logging
```lua
require('sql-cli').setup({
  logging = {
    enabled = true,      -- Turn on/off
    level = "INFO",      -- TRACE, DEBUG, INFO, WARN, ERROR
    max_files = 20,      -- Number of logs to keep
  },
})
```

### Find Logs
```bash
# Unix/Linux
ls ~/.local/share/sql-cli/logs/nvim-plugin*.log

# Windows
dir %LOCALAPPDATA%\sql-cli\logs\nvim-plugin*.log
```

## For Developers

### Add Logging to Your Module
```lua
-- At top of file
local logger = nil
local function get_logger()
  if not logger then
    logger = require('sql-cli').logger
  end
  return logger
end

-- In functions
function M.my_function()
  local log = get_logger()
  if log then
    log.info('my_module', 'Function called')
  end
end
```

### Log Levels
```lua
local log = require('sql-cli').logger

log.trace('module', 'Detailed trace')     -- Very verbose
log.debug('module', 'Debug info')         -- Development
log.info('module', 'Information')         -- Normal operation
log.warn('module', 'Warning')             -- Potential issues
log.error('module', 'Error')              -- Errors
```

### Tips
- Use lazy loading pattern (avoid circular deps)
- Always check `if log then` before logging
- Use descriptive module names as first argument
- Include relevant context in messages
- Call `log.flush()` for immediate writes

## See Full Documentation
- [NVIM_PLUGIN_LOGGING.md](../docs/NVIM_PLUGIN_LOGGING.md)

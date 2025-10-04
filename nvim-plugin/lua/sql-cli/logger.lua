-- Logger module for SQL CLI Neovim plugin
-- Writes to the same log directory as the TUI: ~/.local/share/sql-cli/logs/
--
-- Features:
-- - Automatic log rotation (timestamped files)
-- - Configurable log levels
-- - Automatic cleanup of old logs
-- - Thread-safe file writing
-- - Performance-optimized (buffered writes)

local M = {}

-- Log levels (ordered by severity)
M.levels = {
  TRACE = 1,
  DEBUG = 2,
  INFO = 3,
  WARN = 4,
  ERROR = 5,
}

-- Default configuration
M.config = {
  level = M.levels.INFO,        -- Minimum level to log
  max_files = 20,               -- Maximum number of log files to keep
  buffer_size = 100,            -- Number of messages to buffer before flush
  auto_flush_interval = 5000,   -- Auto-flush every 5 seconds (milliseconds)
  enabled = true,               -- Master switch
}

-- Internal state
local log_file = nil
local log_path = nil
local buffer = {}
local buffer_count = 0
local flush_timer = nil

-- Get platform-specific log directory
local function get_log_dir()
  local home = vim.fn.expand('~')
  if vim.fn.has('win32') == 1 or vim.fn.has('win64') == 1 then
    -- Windows: %LOCALAPPDATA%\sql-cli\logs
    local localappdata = vim.fn.getenv('LOCALAPPDATA')
    if localappdata and localappdata ~= vim.NIL then
      return localappdata .. '\\sql-cli\\logs'
    else
      return 'C:\\temp\\sql-cli\\logs'
    end
  else
    -- Unix: ~/.local/share/sql-cli/logs
    return home .. '/.local/share/sql-cli/logs'
  end
end

-- Get current timestamp in format matching TUI logs
local function get_timestamp()
  return os.date('%Y%m%d_%H%M%S')
end

-- Get display timestamp for log entries
local function get_display_timestamp()
  return os.date('%H:%M:%S')
end

-- Format log level name
local function format_level(level)
  for name, value in pairs(M.levels) do
    if value == level then
      return name
    end
  end
  return 'UNKNOWN'
end

-- Initialize logging system
function M.init(config)
  -- Merge user config with defaults
  if config then
    M.config = vim.tbl_deep_extend('force', M.config, config)
  end

  if not M.config.enabled then
    return
  end

  -- Create log directory
  local log_dir = get_log_dir()
  vim.fn.mkdir(log_dir, 'p')

  -- Create timestamped log file
  local timestamp = get_timestamp()
  local filename = string.format('nvim-plugin_%s.log', timestamp)
  log_path = log_dir .. (vim.fn.has('win32') == 1 and '\\' or '/') .. filename

  -- Open log file
  log_file = io.open(log_path, 'a')
  if not log_file then
    vim.notify('Failed to open log file: ' .. log_path, vim.log.levels.WARN)
    M.config.enabled = false
    return
  end

  -- Set line buffering
  log_file:setvbuf('line')

  -- Create symlink/pointer to latest log (Unix only for symlink)
  local latest_path = log_dir .. (vim.fn.has('win32') == 1 and '\\latest.log' or '/latest.log')
  if vim.fn.has('unix') == 1 then
    -- Remove old symlink and create new one
    vim.fn.delete(latest_path)
    vim.fn.system({'ln', '-sf', log_path, latest_path})
  else
    -- On Windows, write a text file with the path
    local pointer = io.open(latest_path, 'w')
    if pointer then
      pointer:write('Current log file: ' .. log_path .. '\n')
      pointer:close()
    end
  end

  -- Write header
  M.raw_log('INFO', 'logger', 'SQL CLI Neovim Plugin logging initialized')
  M.raw_log('INFO', 'logger', 'Log file: ' .. log_path)
  M.raw_log('INFO', 'logger', 'Log level: ' .. format_level(M.config.level))
  M.raw_log('INFO', 'logger', 'Max files: ' .. M.config.max_files)

  -- Set up auto-flush timer
  if M.config.auto_flush_interval > 0 then
    flush_timer = vim.fn.timer_start(M.config.auto_flush_interval, function()
      M.flush()
    end, {['repeat'] = -1})  -- Repeat indefinitely
  end

  -- Clean up old logs
  M.cleanup_old_logs()
end

-- Cleanup old log files (keep only max_files most recent)
function M.cleanup_old_logs()
  if not M.config.enabled then
    return
  end

  local log_dir = get_log_dir()

  -- Get all nvim-plugin log files
  local pattern = log_dir .. (vim.fn.has('win32') == 1 and '\\nvim-plugin_*.log' or '/nvim-plugin_*.log')
  local files = vim.fn.glob(pattern, false, true)

  if #files <= M.config.max_files then
    return  -- Nothing to clean up
  end

  -- Sort by modification time (oldest first)
  table.sort(files, function(a, b)
    local stat_a = vim.loop.fs_stat(a)
    local stat_b = vim.loop.fs_stat(b)
    if stat_a and stat_b then
      return stat_a.mtime.sec < stat_b.mtime.sec
    end
    return false
  end)

  -- Delete oldest files
  local to_delete = #files - M.config.max_files
  for i = 1, to_delete do
    vim.fn.delete(files[i])
    M.raw_log('DEBUG', 'logger', 'Deleted old log file: ' .. files[i])
  end
end

-- Raw log function (bypasses buffering)
function M.raw_log(level_name, target, message)
  if not M.config.enabled or not log_file then
    return
  end

  local level = M.levels[level_name] or M.levels.INFO

  -- Check if this level should be logged
  if level < M.config.level then
    return
  end

  -- Format: [HH:MM:SS] LEVEL [target] message
  local timestamp = get_display_timestamp()
  local formatted = string.format('[%s] %s [%s] %s\n',
    timestamp, level_name, target, message)

  -- Write directly to file
  log_file:write(formatted)
  log_file:flush()
end

-- Buffered log function (for performance)
function M.log(level_name, target, message)
  if not M.config.enabled then
    return
  end

  local level = M.levels[level_name] or M.levels.INFO

  -- Check if this level should be logged
  if level < M.config.level then
    return
  end

  -- Format: [HH:MM:SS] LEVEL [target] message
  local timestamp = get_display_timestamp()
  local formatted = string.format('[%s] %s [%s] %s\n',
    timestamp, level_name, target, message)

  -- Add to buffer
  table.insert(buffer, formatted)
  buffer_count = buffer_count + 1

  -- Flush if buffer is full
  if buffer_count >= M.config.buffer_size then
    M.flush()
  end
end

-- Flush buffered log messages to file
function M.flush()
  if not M.config.enabled or not log_file or buffer_count == 0 then
    return
  end

  -- Write all buffered messages
  for _, msg in ipairs(buffer) do
    log_file:write(msg)
  end
  log_file:flush()

  -- Clear buffer
  buffer = {}
  buffer_count = 0
end

-- Convenience functions for each log level
function M.trace(target, message)
  M.log('TRACE', target, message)
end

function M.debug(target, message)
  M.log('DEBUG', target, message)
end

function M.info(target, message)
  M.log('INFO', target, message)
end

function M.warn(target, message)
  M.log('WARN', target, message)
end

function M.error(target, message)
  M.log('ERROR', target, message)
end

-- Shorthand: log with automatic target detection
function M.auto(level_name, message)
  -- Get calling function info
  local info = debug.getinfo(2, 'Sn')
  local target = 'unknown'

  if info then
    if info.name then
      target = info.name
    elseif info.source then
      -- Extract filename from source
      target = info.source:match('([^/\\]+)%.lua$') or 'unknown'
    end
  end

  M.log(level_name, target, message)
end

-- Log a table (pretty-print)
function M.log_table(level_name, target, tbl, name)
  if not M.config.enabled then
    return
  end

  name = name or 'table'
  M.log(level_name, target, name .. ' = ' .. vim.inspect(tbl))
end

-- Get log file path
function M.get_log_path()
  return log_path
end

-- Get log directory
function M.get_log_dir()
  return get_log_dir()
end

-- Open log file in a split
function M.open_log()
  if not log_path then
    vim.notify('Logging not initialized', vim.log.levels.WARN)
    return
  end

  M.flush()  -- Flush before opening
  vim.cmd('vsplit ' .. vim.fn.fnameescape(log_path))
  vim.cmd('normal! G')  -- Jump to end
end

-- Tail log file (follow mode)
function M.tail_log()
  if not log_path then
    vim.notify('Logging not initialized', vim.log.levels.WARN)
    return
  end

  M.flush()  -- Flush before tailing

  -- Open in split
  vim.cmd('vsplit ' .. vim.fn.fnameescape(log_path))
  vim.cmd('normal! G')  -- Jump to end

  -- Set up auto-refresh
  vim.cmd('setlocal autoread')
  vim.fn.timer_start(1000, function()
    vim.cmd('checktime')
    vim.cmd('normal! G')
  end, {['repeat'] = -1})
end

-- Shutdown logging (cleanup)
function M.shutdown()
  M.flush()

  if flush_timer then
    vim.fn.timer_stop(flush_timer)
    flush_timer = nil
  end

  if log_file then
    M.raw_log('INFO', 'logger', 'Shutting down logging')
    log_file:close()
    log_file = nil
  end
end

-- Set up autocmd to flush on VimLeavePre
vim.api.nvim_create_autocmd('VimLeavePre', {
  callback = function()
    M.shutdown()
  end,
  desc = 'Flush and close SQL CLI plugin logs'
})

return M

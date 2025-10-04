-- Test script for logger module
-- Run with: lua nvim-plugin/test_logger.lua

-- Set up minimal vim mock for testing
_G.vim = {
  fn = {
    expand = function(path)
      if path == '~' then
        return os.getenv('HOME') or '/home/user'
      end
      return path
    end,
    mkdir = function(path, mode)
      os.execute('mkdir -p ' .. path)
    end,
    has = function(feature)
      if feature == 'win32' or feature == 'win64' then
        return 0
      elseif feature == 'unix' then
        return 1
      end
      return 0
    end,
    glob = function(pattern, nosuf, list)
      -- Simple mock - just return empty list
      return {}
    end,
    delete = function(path) end,
    timer_start = function(delay, callback, opts)
      -- Mock timer - return dummy ID
      return 1
    end,
    timer_stop = function(id) end,
    system = function(cmd)
      -- Mock system command
      if type(cmd) == 'table' then
        os.execute(table.concat(cmd, ' '))
      else
        os.execute(cmd)
      end
    end,
    setreg = function(reg, value) end,
  },
  loop = {
    fs_stat = function(path)
      -- Mock stat
      return { mtime = { sec = os.time() } }
    end,
  },
  api = {
    nvim_create_autocmd = function() end,
  },
  version = function()
    return { major = 0, minor = 9, patch = 0 }
  end,
  log = {
    levels = {
      TRACE = 0,
      DEBUG = 1,
      INFO = 2,
      WARN = 3,
      ERROR = 4,
    }
  },
  tbl_deep_extend = function(mode, ...)
    -- Simple merge
    local result = {}
    for _, tbl in ipairs({...}) do
      for k, v in pairs(tbl) do
        result[k] = v
      end
    end
    return result
  end,
  inspect = function(tbl)
    return vim.print(tbl)
  end,
  print = function(val)
    return tostring(val)
  end,
}

-- Load logger module
package.path = package.path .. ';nvim-plugin/lua/?.lua;nvim-plugin/lua/?/init.lua'
local logger = require('sql-cli.logger')

print('=== Logger Module Test ===\n')

-- Test 1: Initialize logger
print('Test 1: Initialize logger')
logger.init({
  enabled = true,
  level = logger.levels.DEBUG,
  max_files = 5,
  buffer_size = 10,
  auto_flush_interval = 0,  -- Disable auto-flush for testing
})

local log_path = logger.get_log_path()
print('✓ Logger initialized')
print('  Log path: ' .. (log_path or 'nil'))
print('  Log directory: ' .. logger.get_log_dir())
print()

-- Test 2: Write logs at different levels
print('Test 2: Write logs at different levels')
logger.trace('test', 'This is a TRACE message')
logger.debug('test', 'This is a DEBUG message')
logger.info('test', 'This is an INFO message')
logger.warn('test', 'This is a WARN message')
logger.error('test', 'This is an ERROR message')
logger.flush()
print('✓ Wrote 5 log messages')
print()

-- Test 3: Buffering
print('Test 3: Test buffering')
for i = 1, 5 do
  logger.info('buffer_test', 'Buffered message ' .. i)
end
print('✓ Buffered 5 messages (not flushed yet)')
logger.flush()
print('✓ Flushed buffer')
print()

-- Test 4: Log a table
print('Test 4: Log table')
local test_table = {
  name = 'test',
  count = 42,
  items = {'a', 'b', 'c'}
}
logger.log_table('DEBUG', 'test', test_table, 'test_table')
logger.flush()
print('✓ Logged table structure')
print()

-- Test 5: Auto log (with function detection)
print('Test 5: Auto log with function detection')
logger.auto('INFO', 'Auto-detected function context')
logger.flush()
print('✓ Auto-logged with context')
print()

-- Test 6: Read log file
print('Test 6: Read log file contents')
if log_path then
  local file = io.open(log_path, 'r')
  if file then
    print('--- Log file contents ---')
    local content = file:read('*a')
    file:close()

    -- Show first 500 chars
    if #content > 500 then
      print(content:sub(1, 500) .. '\n... (truncated)')
    else
      print(content)
    end
    print('--- End of log ---')
    print('✓ Successfully read log file')
  else
    print('✗ Could not open log file')
  end
else
  print('✗ No log path available')
end
print()

-- Test 7: Shutdown
print('Test 7: Shutdown logger')
logger.shutdown()
print('✓ Logger shut down cleanly')
print()

print('=== All Tests Complete ===')
print('\nLog file location: ' .. (log_path or 'unknown'))
print('You can view it with: cat ' .. (log_path or ''))

#!/usr/bin/env lua

-- Test CLI Analyzer Module
-- Run this to verify cli_analyzer.lua works correctly

-- Mock vim functions for testing
_G.vim = {
  fn = {
    system = function(cmd)
      local cmd_str = type(cmd) == 'table' and table.concat(cmd, ' ') or cmd
      local handle = io.popen(cmd_str)
      local result = handle:read('*a')
      handle:close()
      return result
    end
  },
  v = {
    shell_error = 0  -- Mock: assume success
  },
  json = {
    decode = function(str)
      -- Simple JSON decode (works for our test cases)
      return loadstring('return ' .. str:gsub('null', 'nil'):gsub('true', 'true'):gsub('false', 'false'))()
    end
  },
  api = {
    nvim_buf_get_lines = function(bufnr, start, end_, strict)
      return {}
    end
  }
}

-- Mock require for sql-cli config
package.loaded['sql-cli'] = {
  config = {
    command_path = './target/release/sql-cli'
  },
  logger = {
    debug = function(module, msg) print('[DEBUG] ' .. module .. ': ' .. msg) end,
    info = function(module, msg) print('[INFO] ' .. module .. ': ' .. msg) end,
    error = function(module, msg) print('[ERROR] ' .. module .. ': ' .. msg) end,
  }
}

-- Add lua path for local modules
package.path = package.path .. ';./lua/?.lua;./lua/?/init.lua'

-- Load cli_analyzer module
local cli_analyzer = require('sql-cli.cli_analyzer')

print('=' .. string.rep('=', 70))
print('Testing CLI Analyzer Module')
print('=' .. string.rep('=', 70))

-- Test 1: Analyze simple query
print('\n[Test 1] Analyze simple query with SELECT *')
local analysis, err = cli_analyzer.analyze_query("SELECT * FROM trades WHERE price > 100")
if err then
  print('❌ FAILED: ' .. err)
else
  print('✓ Analysis successful')
  print('  - Has star: ' .. tostring(analysis.has_star))
  print('  - Tables: ' .. table.concat(analysis.tables, ', '))
  print('  - CTEs: ' .. #analysis.ctes)
end

-- Test 2: Analyze query with CTEs
print('\n[Test 2] Analyze query with multiple CTEs')
local query_with_ctes = [[
WITH WEB raw AS (URL 'http://example.com' METHOD GET),
filtered AS (SELECT * FROM raw WHERE id > 10)
SELECT * FROM filtered
]]
analysis, err = cli_analyzer.analyze_query(query_with_ctes)
if err then
  print('❌ FAILED: ' .. err)
else
  print('✓ Analysis successful')
  print('  - CTEs found: ' .. #analysis.ctes)
  for i, cte in ipairs(analysis.ctes) do
    print(string.format('    %d. %s (%s)', i, cte.name, cte.cte_type))
  end
end

-- Test 3: Extract CTE
print('\n[Test 3] Extract CTE from query')
local cte_query, err = cli_analyzer.extract_cte(query_with_ctes, 'filtered')
if err then
  print('❌ FAILED: ' .. err)
else
  print('✓ CTE extracted successfully')
  print('  Extracted query:')
  for line in cte_query:gmatch('[^\n]+') do
    print('    ' .. line)
  end
end

-- Test 4: Find query context
print('\n[Test 4] Find query context at position')
local context, err = cli_analyzer.find_query_context(query_with_ctes, 2, 10)
if err then
  print('❌ FAILED: ' .. err)
else
  print('✓ Context found')
  print('  - Type: ' .. context.context_type)
  if context.cte_name then
    print('  - CTE name: ' .. context.cte_name)
  end
  print('  - Can execute independently: ' .. tostring(context.can_execute_independently))
end

-- Test 5: Helper functions
print('\n[Test 5] Helper functions')
local has_star = cli_analyzer.has_star("SELECT * FROM test")
print('  - has_star("SELECT * FROM test"): ' .. tostring(has_star))

local cte_names = cli_analyzer.get_cte_names(analysis)
print('  - get_cte_names(): ' .. table.concat(cte_names, ', '))

local cte = cli_analyzer.find_cte_by_name(analysis, 'raw')
if cte then
  print('  - find_cte_by_name("raw"): found (' .. cte.cte_type .. ')')
  print('  - is_web_cte(raw): ' .. tostring(cli_analyzer.is_web_cte(cte)))
end

print('\n' .. string.rep('=', 72))
print('All tests completed!')
print(string.rep('=', 72))

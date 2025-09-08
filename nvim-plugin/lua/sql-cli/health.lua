-- Health check for SQL CLI plugin
local M = {}

function M.check()
  vim.health.start("SQL CLI Plugin")
  
  -- Check if plugin is loaded
  if vim.g.loaded_sql_cli then
    vim.health.ok("Plugin loaded")
  else
    vim.health.error("Plugin not loaded")
  end
  
  -- Check for sql-cli executable
  local sql_cli = require('sql-cli').config.command
  if vim.fn.executable(sql_cli) == 1 then
    vim.health.ok("sql-cli executable found: " .. sql_cli)
  else
    vim.health.error("sql-cli executable not found: " .. sql_cli)
  end
  
  -- Check Neovim version
  if vim.fn.has('nvim-0.7') == 1 then
    vim.health.ok("Neovim version compatible")
  else
    vim.health.error("Requires Neovim 0.7+")
  end
end

return M
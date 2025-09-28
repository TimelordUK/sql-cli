-- Debug script to verify multi-token manager setup
-- Run this in nvim with :source ~/dev/sql-cli/nvim-plugin/examples/debug_multi_token.lua

-- Check if multi_token_manager module exists
local ok, multi_token = pcall(require, 'sql-cli.multi_token_manager')
if not ok then
  vim.notify("ERROR: Cannot load multi_token_manager module", vim.log.levels.ERROR)
  vim.notify("Error: " .. tostring(multi_token), vim.log.levels.ERROR)

  -- Check where nvim is looking for modules
  vim.notify("Package path: " .. package.path, vim.log.levels.INFO)

  -- Try to find the file manually
  local paths = {
    vim.fn.expand("~/dev/sql-cli/nvim-plugin/lua/sql-cli/multi_token_manager.lua"),
    vim.fn.expand("~/AppData/Local/nvim-data/lazy/sql-cli/lua/sql-cli/multi_token_manager.lua"),
    vim.fn.expand("~/AppData/Local/nvim-data/lazy/sql-cli.nvim/lua/sql-cli/multi_token_manager.lua"),
  }

  for _, path in ipairs(paths) do
    if vim.fn.filereadable(path) == 1 then
      vim.notify("Found file at: " .. path, vim.log.levels.INFO)
    else
      vim.notify("Not found at: " .. path, vim.log.levels.WARN)
    end
  end

  return
end

vim.notify("✓ multi_token_manager module loaded successfully", vim.log.levels.INFO)

-- Check if it's been set up
local tokens = multi_token.tokens or {}
local num_tokens = vim.tbl_count(tokens)

if num_tokens == 0 then
  vim.notify("WARNING: No tokens configured", vim.log.levels.WARN)
  vim.notify("You need to call multi_token_manager.setup() with token configurations", vim.log.levels.INFO)

  -- Show example setup
  local example = [[
Example setup:

local multi_token = require('sql-cli.multi_token_manager')
multi_token.setup({
  JWT_TOKEN = {
    command = "powershell.exe -File ~/GetToken.ps1",
    refresh_interval = 600,
    auto_refresh = true,
  },
  JWT_TOKEN_PROD = {
    command = "powershell.exe -File ~/GetProdToken.ps1",
    refresh_interval = 600,
    auto_refresh = true,
  }
})
multi_token.create_commands()
]]

  print(example)
else
  vim.notify("✓ Found " .. num_tokens .. " configured tokens:", vim.log.levels.INFO)
  for name, config in pairs(tokens) do
    local status = config.current_token and "Has token" or "No token"
    local timer = config.timer and "Timer active" or "No timer"
    vim.notify("  • " .. name .. ": " .. status .. ", " .. timer, vim.log.levels.INFO)
  end
end

-- Check if commands exist
local commands = {
  "TokenRefreshAll",
  "TokenRefresh",
  "TokenStatus",
  "TokenStop",
  "TokenStart",
}

vim.notify("\nChecking commands:", vim.log.levels.INFO)
for _, cmd in ipairs(commands) do
  local exists = vim.fn.exists(":" .. cmd) == 2
  if exists then
    vim.notify("  ✓ :" .. cmd .. " exists", vim.log.levels.INFO)
  else
    vim.notify("  ✗ :" .. cmd .. " missing", vim.log.levels.WARN)
  end
end

-- If no multi-token commands, create them
if vim.fn.exists(":TokenRefreshAll") ~= 2 then
  vim.notify("\nCreating multi-token commands...", vim.log.levels.INFO)
  multi_token.create_commands()
  vim.notify("Commands created. Try :TokenStatus now", vim.log.levels.INFO)
end

-- Test function to manually set up tokens
vim.api.nvim_create_user_command('SetupTestTokens', function()
  vim.notify("Setting up test tokens...", vim.log.levels.INFO)

  multi_token.setup({
    JWT_TOKEN = {
      command = 'powershell.exe -Command "Write-Output test-token-uat"',
      refresh_interval = 60,
      auto_refresh = true,
      debug = true,
    },
    JWT_TOKEN_PROD = {
      command = 'powershell.exe -Command "Write-Output test-token-prod"',
      refresh_interval = 60,
      auto_refresh = true,
      debug = true,
    }
  })

  multi_token.create_commands()
  vim.notify("Test tokens configured. Try :TokenStatus", vim.log.levels.INFO)
end, {desc = "Setup test tokens for debugging"})
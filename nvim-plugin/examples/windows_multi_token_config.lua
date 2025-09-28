-- Example Windows configuration for multi-token manager
-- Add this to your Windows nvim config at:
-- C:\Users\YOUR_USERNAME\AppData\Local\nvim\lua\plugins\sql-cli.lua

-- Option 1: Using $HOME environment variable (works in PowerShell)
multi_token_manager.setup({
  JWT_TOKEN = {
    -- Use $HOME which PowerShell understands
    command = 'powershell.exe -NoProfile -File "$HOME\\dev\\sql-cli\\ExportJwt.ps1"',
    refresh_interval = 20,  -- 20 seconds for testing
    auto_refresh = true,
    debug = true,
  },
  JWT_TOKEN_PROD = {
    command = 'powershell.exe -NoProfile -File "$HOME\\dev\\sql-cli\\ExportJwtProd.ps1"',
    refresh_interval = 840,  -- 14 minutes
    auto_refresh = true,
    debug = false,
  },
})

-- Option 2: Using full paths
-- Replace YOUR_USERNAME with your actual Windows username
--[[
multi_token_manager.setup({
  JWT_TOKEN = {
    command = 'powershell.exe -NoProfile -File "C:\\Users\\YOUR_USERNAME\\dev\\sql-cli\\ExportJwt.ps1"',
    refresh_interval = 20,
    auto_refresh = true,
    debug = true,
  },
  JWT_TOKEN_PROD = {
    command = 'powershell.exe -NoProfile -File "C:\\Users\\YOUR_USERNAME\\dev\\sql-cli\\ExportJwtProd.ps1"',
    refresh_interval = 840,
    auto_refresh = true,
    debug = false,
  },
})
--]]

-- Option 3: Using vim.fn.expand with Windows paths
--[[
multi_token_manager.setup({
  JWT_TOKEN = {
    -- This will expand to the correct Windows path
    command = 'powershell.exe -NoProfile -File "' .. vim.fn.expand("~/dev/sql-cli/ExportJwt.ps1") .. '"',
    refresh_interval = 20,
    auto_refresh = true,
    debug = true,
  },
  JWT_TOKEN_PROD = {
    command = 'powershell.exe -NoProfile -File "' .. vim.fn.expand("~/dev/sql-cli/ExportJwtProd.ps1") .. '"',
    refresh_interval = 840,
    auto_refresh = true,
    debug = false,
  },
})
--]]

-- Option 4: Using environment variables
--[[
-- First set environment variables in Windows:
-- $env:TOKEN_SCRIPT_DIR = "C:\Users\YOUR_USERNAME\dev\sql-cli"
-- Then use them in the config:
multi_token_manager.setup({
  JWT_TOKEN = {
    command = 'powershell.exe -NoProfile -Command "& $env:TOKEN_SCRIPT_DIR\\ExportJwt.ps1"',
    refresh_interval = 20,
    auto_refresh = true,
    debug = true,
  },
  JWT_TOKEN_PROD = {
    command = 'powershell.exe -NoProfile -Command "& $env:TOKEN_SCRIPT_DIR\\ExportJwtProd.ps1"',
    refresh_interval = 840,
    auto_refresh = true,
    debug = false,
  },
})
--]]
-- Example Windows configuration for multi-token manager
-- Add this to your Windows nvim config at:
-- C:\Users\YOUR_USERNAME\AppData\Local\nvim\lua\plugins\sql-cli.lua

-- Option 1: Using -Command to bypass execution policy (RECOMMENDED)
multi_token_manager.setup({
  JWT_TOKEN = {
    -- Use -Command with & operator to bypass execution policy
    command = 'powershell.exe -NoProfile -Command "& $HOME\\dev\\sql-cli\\ExportJwt.ps1"',
    refresh_interval = 20,  -- 20 seconds for testing
    auto_refresh = true,
    debug = true,
  },
  JWT_TOKEN_PROD = {
    command = 'powershell.exe -NoProfile -Command "& $HOME\\dev\\sql-cli\\ExportJwtProd.ps1"',
    refresh_interval = 840,  -- 14 minutes
    auto_refresh = true,
    debug = false,
  },
})

-- Option 2: Using full paths with -Command
-- Replace YOUR_USERNAME with your actual Windows username
--[[
multi_token_manager.setup({
  JWT_TOKEN = {
    command = 'powershell.exe -NoProfile -Command "& C:\\Users\\YOUR_USERNAME\\dev\\sql-cli\\ExportJwt.ps1"',
    refresh_interval = 20,
    auto_refresh = true,
    debug = true,
  },
  JWT_TOKEN_PROD = {
    command = 'powershell.exe -NoProfile -Command "& C:\\Users\\YOUR_USERNAME\\dev\\sql-cli\\ExportJwtProd.ps1"',
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

-- Option 4: Using ExecutionPolicy Bypass flag
--[[
multi_token_manager.setup({
  JWT_TOKEN = {
    -- Use -ExecutionPolicy Bypass to allow script execution
    command = 'powershell.exe -NoProfile -ExecutionPolicy Bypass -File "$HOME\\dev\\sql-cli\\ExportJwt.ps1"',
    refresh_interval = 20,
    auto_refresh = true,
    debug = true,
  },
  JWT_TOKEN_PROD = {
    command = 'powershell.exe -NoProfile -ExecutionPolicy Bypass -File "$HOME\\dev\\sql-cli\\ExportJwtProd.ps1"',
    refresh_interval = 840,
    auto_refresh = true,
    debug = false,
  },
})
--]]

-- TROUBLESHOOTING EXECUTION POLICY ISSUES:
-- If you get "running scripts is disabled on this system" error, try:
-- 1. Use -Command with & operator (Option 1 above) - RECOMMENDED
-- 2. Use -ExecutionPolicy Bypass flag (Option 4 above)
-- 3. Change system execution policy (run as admin): Set-ExecutionPolicy RemoteSigned
-- 4. Use a batch file wrapper instead of PowerShell script
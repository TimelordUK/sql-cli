-- SQL-CLI Configuration for Windows
-- Place this file at: C:\Users\sjame\AppData\Local\nvim\lua\plugins\sql-cli.lua

return {
  setup = function()
    -- Load SQL-CLI plugin
    local sql_cli = require('sql-cli')

    -- Basic SQL-CLI setup
    sql_cli.setup({
      -- Your existing config...
    })

    -- OPTION 1: Use the existing single token manager (current setup)
    -- local token_manager = require('sql-cli.token_manager')
    -- token_manager.setup({
    --   token_command = "powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File " .. vim.fn.expand("~/dev/sql-cli/ExportJwt.ps1"),
    --   auto_refresh = true,
    --   refresh_interval = 600,
    --   token_var_name = "JWT_TOKEN",
    --   debug = true,
    -- })

    -- OPTION 2: Use the new multi-token manager for multiple environments
    local multi_token_manager = require('sql-cli.multi_token_manager')

    multi_token_manager.setup({
      -- UAT Token
      JWT_TOKEN = {
        -- Use your existing PowerShell script for UAT
        command = "powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File " .. vim.fn.expand("~/dev/sql-cli/ExportJwt.ps1"),

        refresh_interval = 600,  -- 10 minutes (adjust based on your token lifetime)
        auto_refresh = true,
        debug = true,  -- Set false for production
      },

      -- Production Token
      JWT_TOKEN_PROD = {
        -- Create a separate script for production token
        -- You'll need to create ExportJwtProd.ps1 with prod credentials
        command = "powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File " .. vim.fn.expand("~/dev/sql-cli/ExportJwtProd.ps1"),

        -- Or if you have a different auth endpoint/method for prod:
        -- command = "powershell.exe -NoProfile -Command \"& { (Invoke-RestMethod -Uri 'https://auth.prod.example.com/token' -Method POST -Body @{grant_type='client_credentials'} -Headers @{Authorization='Basic xxx'}).access_token }\"",

        refresh_interval = 600,
        auto_refresh = true,
        debug = true,
      },
    })

    -- Create commands for token management
    multi_token_manager.create_commands()

    -- Show status on startup
    vim.defer_fn(function()
      vim.notify("SQL-CLI Multi-Token Manager Ready", vim.log.levels.INFO)
      vim.notify("Tokens configured: JWT_TOKEN (UAT), JWT_TOKEN_PROD", vim.log.levels.INFO)
      vim.notify("Use :TokenStatus to see current state", vim.log.levels.INFO)
    end, 1000)

    -- Optional: Create convenient keymaps
    vim.keymap.set('n', '<leader>st', ':TokenStatus<CR>', { desc = 'Show token status' })
    vim.keymap.set('n', '<leader>sr', ':TokenRefreshAll<CR>', { desc = 'Refresh all tokens' })
  end
}

-- If using lazy.nvim or similar plugin manager, you can also structure it like:
-- return {
--   'TimelordUK/sql-cli',
--   config = function()
--     -- Configuration above goes here
--   end
-- }
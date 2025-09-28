-- Example setup for multiple JWT tokens in SQL-CLI
-- Add this to your nvim config (e.g., ~/.config/nvim/after/plugin/sql-cli.lua)

-- Load the multi-token manager
local multi_token_manager = require('sql-cli.multi_token_manager')

-- Configure multiple tokens with independent refresh cycles
multi_token_manager.setup({
  -- UAT Token
  JWT_TOKEN = {
    -- PowerShell command for UAT token
    command = "powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File " .. vim.fn.expand("~/dev/sql-cli/GetUatToken.ps1"),

    -- Or use a shell script on Linux/Mac:
    -- command = vim.fn.expand("~/.config/sql-cli/get_uat_token.sh"),

    -- Or fetch from an endpoint:
    -- endpoint = "http://auth.uat.example.com/token",
    -- headers = { ["Authorization"] = "Basic " .. vim.fn.base64encode("client:secret") },
    -- token_path = "access_token",  -- JSON path to extract

    refresh_interval = 840,  -- 14 minutes for 15-minute tokens
    auto_refresh = true,
    debug = false,  -- Set true to see refresh activity
  },

  -- Production Token
  JWT_TOKEN_PROD = {
    -- PowerShell command for production token
    command = "powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File " .. vim.fn.expand("~/dev/sql-cli/GetProdToken.ps1"),

    -- Or use different auth method for prod:
    -- endpoint = "https://auth.prod.example.com/oauth/token",
    -- headers = {
    --   ["Content-Type"] = "application/x-www-form-urlencoded",
    -- },
    -- token_path = "access_token",

    refresh_interval = 840,  -- Same refresh interval
    auto_refresh = true,
    debug = false,
  },

  -- Additional tokens as needed
  -- AZURE_TOKEN = {
  --   command = "az account get-access-token --resource https://api.example.com --query accessToken -o tsv",
  --   refresh_interval = 3600,  -- 1 hour
  --   auto_refresh = true,
  -- },
})

-- Create user commands for token management
multi_token_manager.create_commands()

-- Notify that multi-token manager is configured
vim.notify("Multi-Token Manager configured for JWT_TOKEN and JWT_TOKEN_PROD", vim.log.levels.INFO)
vim.notify("Commands: :TokenStatus, :TokenRefreshAll, :TokenRefresh [name]", vim.log.levels.INFO)

-- Optional: Update template system to use the multi-token manager
vim.api.nvim_create_autocmd("FileType", {
  pattern = "sql",
  callback = function()
    -- Hook into template expansion to ensure fresh tokens
    local old_expand = vim.b.sql_template_expand
    if old_expand then
      vim.b.sql_template_expand = function(text)
        -- Refresh tokens if needed before template expansion
        local expanded = text

        -- Replace token variables with fresh values
        expanded = expanded:gsub("${JWT_TOKEN}", function()
          return multi_token_manager.get_token("JWT_TOKEN") or ""
        end)

        expanded = expanded:gsub("${JWT_TOKEN_PROD}", function()
          return multi_token_manager.get_token("JWT_TOKEN_PROD") or ""
        end)

        -- Continue with normal template expansion
        if old_expand then
          return old_expand(expanded)
        end
        return expanded
      end
    end
  end
})
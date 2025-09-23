-- Example configuration for token management in SQL-CLI nvim plugin
-- Add this to your Neovim config (init.lua or wherever you setup plugins)

require('sql-cli').setup({
  -- Other config options...

  -- Token management configuration
  token = {
    -- URL of your token endpoint
    token_endpoint = "http://localhost:5000/token",  -- Your Flask endpoint

    -- Or for more complex endpoints with authentication
    -- token_endpoint = "https://auth.company.com/api/v1/token",

    -- Automatically refresh token when it's about to expire
    auto_refresh = true,

    -- Refresh interval in seconds (14 minutes for 15-minute tokens)
    refresh_interval = 14 * 60,

    -- Name of the environment variable to set
    token_var_name = "JWT_TOKEN",  -- This is the default

    -- JSON path to extract token from response
    -- For simple response: {"token": "xxx"}
    token_path = "token",

    -- For nested response: {"data": {"access_token": "xxx"}}
    -- token_path = "data.access_token",

    -- Custom headers for token request (if needed)
    custom_headers = {
      -- ["X-API-Key"] = "your-api-key",
      -- ["Accept"] = "application/json",
    },
  }
})

-- Available commands after setup:
-- :TokenRefresh    - Manually refresh the token
-- :TokenShow       - Show current token (first 30 chars)
-- :TokenConfig     - Show current config
-- :TokenConfig <url> - Set token endpoint URL

-- Usage in SQL macros:
-- The token will be automatically used when you have:
-- API_TOKEN = ${JWT_TOKEN}
-- in your CONFIG macro

-- Example workflow:
-- 1. Open nvim
-- 2. :TokenRefresh  (or it auto-refreshes if configured)
-- 3. Expand your macros - token is automatically injected
-- 4. Token refreshes automatically every 14 minutes (or your configured interval)
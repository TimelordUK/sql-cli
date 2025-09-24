# Token Manager Guide

The SQL-CLI token manager provides automatic JWT token refresh for authenticated API calls in your SQL queries.

## Features

- **Script-based token fetching** - Run any command/script to get tokens
- **HTTP endpoint support** - Fetch tokens from REST APIs (legacy)
- **Automatic refresh timer** - Refresh tokens before they expire
- **Environment variable integration** - Sets `JWT_TOKEN` (or custom var)
- **Debug mode** - See detailed refresh activity

## Configuration

### Option 1: Script/Command Based (Recommended)

Configure token manager to run a script or command:

```lua
-- In your Neovim config
local token_manager = require('sql-cli.token_manager')

token_manager.setup({
  -- Command to run (must output ONLY the token to stdout)
  token_command = "./export_jwt.sh",
  -- Or for PowerShell:
  -- token_command = "pwsh -NoProfile -Command ./ExportJwt.ps1",

  -- Enable automatic refresh
  auto_refresh = true,

  -- Refresh interval in seconds (840 = 14 minutes for 15-min tokens)
  refresh_interval = 840,

  -- Environment variable to set (default: JWT_TOKEN)
  token_var_name = "JWT_TOKEN",

  -- Enable debug output
  debug = false,
})
```

### Option 2: HTTP Endpoint (Legacy)

```lua
token_manager.setup({
  -- URL to fetch token from
  token_endpoint = "https://auth.example.com/token",

  -- JSON path to extract token from response
  token_path = "access_token",

  -- Optional custom headers
  custom_headers = {
    ["X-API-Key"] = "your-api-key"
  },

  auto_refresh = true,
  refresh_interval = 840,
})
```

## Script Requirements

Your token script must:
1. Output ONLY the token to stdout
2. Exit with code 0 on success
3. Output errors to stderr (not stdout)

### Example Shell Script

```bash
#!/bin/bash
# export_jwt.sh

# Fetch token from your auth service
TOKEN=$(curl -s -X POST https://auth.example.com/token \
  -H "Content-Type: application/json" \
  -d '{"client_id":"myapp","client_secret":"secret"}' \
  | jq -r '.access_token')

# Output ONLY the token
echo "$TOKEN"
```

### Example PowerShell Script

```powershell
# ExportJwt.ps1

# Fetch token from your auth service
$response = Invoke-RestMethod -Uri "https://auth.example.com/token" `
  -Method Post `
  -ContentType "application/json" `
  -Body '{"client_id":"myapp","client_secret":"secret"}'

# Output ONLY the token
Write-Output $response.access_token
```

## Available Commands

- `:TokenConfig` - Show current configuration
- `:TokenRefresh` - Manually refresh token
- `:TokenShow` - Display current token (truncated)
- `:TokenStartTimer` - Start auto-refresh timer
- `:TokenStopTimer` - Stop auto-refresh timer
- `:TokenSetCommand <cmd>` - Set token fetch command
- `:TokenDebug` - Toggle debug mode

## Using in SQL Queries

Once configured, the token is available as `${JWT_TOKEN}` (or your custom var name) in your SQL queries:

```sql
-- The token is automatically available
WITH WEB trades AS (
    URL '@{VAR:BASE_URL}/api/trades'
    METHOD POST
    HEADERS (
        'Authorization': 'Bearer @{VAR:API_TOKEN}',
        'Content-Type': 'application/json'
    )
    BODY '{
        "query": "SELECT * FROM trades"
    }'
    FORMAT JSON
)
SELECT * FROM trades;
```

With the CONFIG macro:

```sql
-- MACRO: CONFIG
-- BASE_URL = https://api.example.com
-- API_TOKEN = ${JWT_TOKEN}
-- END MACRO
```

## Testing

Use the provided test configuration with a 10-second interval:

```vim
:luafile test_token_timer.lua
```

This will:
- Configure token manager with `./export_jwt.sh`
- Set refresh interval to 10 seconds
- Enable debug mode
- Start the timer automatically

Watch the notifications to see the timer working.

## Troubleshooting

### Timer not working?
- Check `:TokenConfig` to see if timer is active
- Enable debug mode with `:TokenDebug`
- Verify your script works: `!./export_jwt.sh`
- Check Neovim messages: `:messages`

### Token not refreshing?
- Ensure `auto_refresh = true` in setup
- Check `refresh_interval` is in seconds (not ms)
- Verify script outputs ONLY the token
- Check script exit code is 0

### Windows/PowerShell issues?
- Use `-NoProfile` flag to skip profile loading
- Ensure execution policy allows scripts
- Use full path to PowerShell if needed
- Test script manually first

## Integration with Templates

The token manager integrates seamlessly with the template system:

1. Token is fetched before macro expansion
2. Available as `@{VAR:API_TOKEN}` in macros
3. Automatically refreshed based on timer
4. No manual token management needed
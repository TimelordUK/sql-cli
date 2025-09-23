# Token Management for SQL-CLI Neovim Plugin

## Overview
Automatically refresh JWT tokens within Neovim without exiting your editor. Perfect for APIs with short-lived tokens (e.g., 15-minute expiry).

## Quick Start

### 1. Configure Your Token Endpoint

Add to your Neovim config (`~/.config/nvim/init.lua`):

```lua
require('sql-cli').setup({
  -- Your existing config...

  token = {
    -- URL of your token endpoint
    token_endpoint = 'http://localhost:5000/token',

    -- Auto-refresh before expiry
    auto_refresh = true,

    -- Refresh interval in seconds (14 minutes for 15-min tokens)
    refresh_interval = 14 * 60,

    -- Environment variable name (matches ${JWT_TOKEN} in macros)
    token_var_name = 'JWT_TOKEN',

    -- JSON path to extract token from response
    token_path = 'token',  -- For {"token": "xxx"}
  }
})
```

### 2. Flask/API Response Format

Your token endpoint should return JSON:

**Simple format:**
```json
{
  "token": "eyJ0eXAiOiJKV1QiLCJhbGc..."
}
```

**Nested format:**
```json
{
  "data": {
    "access_token": "eyJ0eXAiOiJKV1QiLCJhbGc..."
  }
}
```

For nested format, use: `token_path = 'data.access_token'`

### 3. Available Commands

- `:TokenRefresh` - Manually refresh token
- `:TokenShow` - Display current token (truncated)
- `:TokenConfig` - Show current configuration
- `:TokenConfig <url>` - Set new token endpoint

### 4. Usage in SQL Macros

In your SQL files:

```sql
-- MACRO: CONFIG
-- BASE_URL = http://tradeapi
-- API_TOKEN = ${JWT_TOKEN}  -- Automatically uses fresh token
-- END MACRO
```

The `${JWT_TOKEN}` will automatically use the refreshed token.

## Configuration Options

| Option | Default | Description |
|--------|---------|-------------|
| `token_endpoint` | `nil` | URL to fetch token from |
| `auto_refresh` | `false` | Auto-refresh before expiry |
| `refresh_interval` | `840` (14 min) | Seconds between refreshes |
| `token_var_name` | `"JWT_TOKEN"` | Environment variable name |
| `token_path` | `"token"` | JSON path to token in response |
| `custom_headers` | `{}` | Additional headers for request |

## Example Flask Endpoint

```python
from flask import Flask, jsonify
import jwt
import datetime

app = Flask(__name__)

@app.route('/token')
def get_token():
    """Generate a JWT token valid for 15 minutes."""
    payload = {
        'user': 'api-user',
        'exp': datetime.datetime.utcnow() + datetime.timedelta(minutes=15)
    }

    token = jwt.encode(payload, 'secret-key', algorithm='HS256')

    return jsonify({
        'token': token,
        'expires_in': 900  # optional
    })

if __name__ == '__main__':
    app.run(port=5000)
```

## Testing Setup

1. Start the test server:
   ```bash
   cd sql-cli/tests
   uv run python test_trade_server.py
   ```

2. Test token endpoint:
   ```bash
   curl http://localhost:5001/token
   ```

3. In Neovim:
   ```vim
   :TokenRefresh
   :TokenShow
   ```

## Troubleshooting

### Token not refreshing
- Check `:TokenConfig` shows correct endpoint
- Verify endpoint is accessible: `curl <your-endpoint>`
- Check `:messages` for error details

### Wrong token path
- Check your API response structure
- Adjust `token_path` accordingly:
  - `"token"` for `{"token": "..."}`
  - `"access_token"` for `{"access_token": "..."}`
  - `"data.token"` for `{"data": {"token": "..."}}`

### Manual refresh needed
- Set `auto_refresh = true` for automatic refresh
- Adjust `refresh_interval` based on token lifetime

## Integration with Workflows

The token manager integrates seamlessly with macro expansion:

1. Token expires every 15 minutes
2. Auto-refresh runs every 14 minutes (configurable)
3. Macros always use fresh token
4. No need to exit Neovim or manually update environment variables

## Security Notes

- Tokens are stored in Neovim session memory only
- Not persisted to disk
- Cleared when Neovim exits
- Use HTTPS for production endpoints
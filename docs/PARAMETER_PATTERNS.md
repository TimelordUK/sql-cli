# Parameter Pattern Guidelines

## Two Types of Parameters

### 1. Interactive Parameters: `{{PARAM}}`
- **Purpose**: Prompted at execution time
- **Usage**: Values that change frequently (dates, sources, filters)
- **Example**: `{{SOURCE}}`, `{{DATE}}`, `{{STATUS}}`
- **Behavior**:
  - Prompts user for value when executing with `\sx`
  - Remembers history of previous values
  - Can cycle through history with `\spn` / `\spp`

### 2. Environment Variables: `${VAR}`
- **Purpose**: Resolved from environment at runtime
- **Usage**: Dynamic values that auto-refresh (JWT tokens, credentials)
- **Example**: `${JWT_TOKEN}`, `${API_KEY}`, `${USER}`
- **Behavior**:
  - NOT prompted - left as-is for sql-cli to resolve
  - Reads from environment variables
  - Automatically uses latest value (e.g., refreshed JWT token)

## Examples

### Query with Both Types
```sql
WITH WEB trades AS (
    URL 'https://api.trading.com/trades'
    HEADERS (
        'Authorization': 'Bearer ${JWT_TOKEN}',  -- Environment variable (auto-refreshed)
        'Content-Type': 'application/json'
    )
    BODY '{
        "Source": "{{SOURCE}}",      -- Interactive parameter (prompted)
        "Date": "{{DATE}}"            -- Interactive parameter (prompted)
    }'
)
SELECT * FROM trades;
```

### Execution Flow
1. Press `\sx` to execute
2. Prompted for `{{SOURCE}}` - enter "Bloomberg"
3. Prompted for `{{DATE}}` - enter "2024-03-27"
4. `${JWT_TOKEN}` is NOT prompted - resolved from environment
5. Query executes with resolved values

### Setting Environment Variables
```bash
# In your shell or .bashrc/.zshrc
export JWT_TOKEN=$(curl -s http://localhost:5001/token | jq -r .token)

# Or in nvim before executing
:!export JWT_TOKEN="your-token-here"
```

### Auto-refresh Pattern
For tokens that expire every 10 minutes:
```bash
# In a background script
while true; do
    export JWT_TOKEN=$(curl -s http://auth-server/token | jq -r .token)
    sleep 600  # Refresh every 10 minutes
done
```

## Summary
- Use `{{}}` for values you want to be prompted for
- Use `${}` for environment variables that should auto-refresh
- Never mix the patterns - they serve different purposes
- `${}` is perfect for JWT tokens that refresh frequently
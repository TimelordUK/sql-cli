# Token Management for Web CTEs

SQL-CLI now supports managing multiple authentication tokens with automatic refresh commands. This is perfect for working with multiple environments (UAT, Production) or different API providers.

## Overview

Instead of manually managing environment variables like `JWT_TOKEN` and `JWT_TOKEN_PROD`, you can now configure tokens in your config file with associated refresh commands. SQL-CLI will automatically run these commands to get fresh tokens when needed.

## Configuration

Add token definitions to `~/.config/sql-cli/config.toml`:

```toml
[tokens]
# Auto-refresh tokens before they expire (future feature)
auto_refresh = false

# Default token lifetime in seconds
default_lifetime = 3600

# UAT Environment Token
[tokens.tokens.JWT_TOKEN]
description = "UAT environment JWT token"
refresh_command = "~/.config/sql-cli/get_uat_token.sh"
lifetime = 3600  # 1 hour

# Production Environment Token
[tokens.tokens.JWT_TOKEN_PROD]
description = "Production environment JWT token"
refresh_command = "~/.config/sql-cli/get_prod_token.sh"
lifetime = 7200  # 2 hours

# Additional tokens as needed...
[tokens.tokens.AZURE_TOKEN]
description = "Azure access token"
refresh_command = "az account get-access-token --resource https://api.example.com --query accessToken -o tsv"
lifetime = 3600
```

## Token Refresh Commands

Each token needs a `refresh_command` that:
1. Outputs **ONLY** the token to stdout
2. Returns exit code 0 on success
3. Can be any executable command or script

### Example Token Scripts

#### OAuth2 Client Credentials
```bash
#!/bin/bash
# get_uat_token.sh
curl -s -X POST https://auth.uat.example.com/oauth/token \
  -d "client_id=${CLIENT_ID}" \
  -d "client_secret=${CLIENT_SECRET}" \
  -d "grant_type=client_credentials" | \
  jq -r .access_token
```

#### Azure CLI
```bash
#!/bin/bash
# get_azure_token.sh
az account get-access-token \
  --resource https://api.example.com \
  --query accessToken -o tsv
```

#### AWS Secrets Manager
```bash
#!/bin/bash
# get_prod_token.sh
aws secretsmanager get-secret-value \
  --secret-id prod/jwt-token \
  --query SecretString --output text
```

#### HashiCorp Vault
```bash
#!/bin/bash
# get_vault_token.sh
vault kv get -field=token secret/prod/jwt
```

## Token Management Script

Use the provided script to manage tokens:

```bash
# Refresh all configured tokens
./scripts/manage_tokens.sh refresh

# Refresh specific token
./scripts/manage_tokens.sh refresh JWT_TOKEN_PROD

# Show token status
./scripts/manage_tokens.sh status

# List configured tokens
./scripts/manage_tokens.sh list

# Export tokens for shell session
eval $(./scripts/manage_tokens.sh export)

# Clear cached tokens
./scripts/manage_tokens.sh clear
```

## Using Tokens in Queries

Once configured, use tokens in your Web CTEs as environment variables:

```sql
-- Uses ${JWT_TOKEN} from config
WITH WEB trades AS (
    URL 'https://api.uat.example.com/trades'
    HEADERS
        'Authorization': 'Bearer ${JWT_TOKEN}',
        'Content-Type': 'application/json'
    FORMAT JSON
    CACHE 3600
)
SELECT * FROM trades;

-- Uses ${JWT_TOKEN_PROD} from config
WITH WEB prod_trades AS (
    URL 'https://api.prod.example.com/trades'
    HEADERS
        'Authorization': 'Bearer ${JWT_TOKEN_PROD}',
        'Content-Type': 'application/json'
    FORMAT JSON
    CACHE 7200
)
SELECT * FROM prod_trades;
```

## Security Best Practices

1. **Token Scripts**: Store in `~/.config/sql-cli/` with permissions 700
   ```bash
   chmod 700 ~/.config/sql-cli/get_*_token.sh
   ```

2. **Secrets**: Never hardcode secrets in scripts. Use:
   - Environment variables from secure sources
   - Secret management tools (Vault, AWS Secrets Manager)
   - OS keychains (secret-tool, security)

3. **Token Cache**: Tokens are cached in `~/.cache/sql-cli/tokens/` with 600 permissions

4. **Config File**: Keep config file secure
   ```bash
   chmod 600 ~/.config/sql-cli/config.toml
   ```

## Workflow Example

1. **Setup tokens in config**:
   ```toml
   [tokens.tokens.JWT_TOKEN]
   description = "UAT API token"
   refresh_command = "~/.config/sql-cli/get_uat_token.sh"

   [tokens.tokens.JWT_TOKEN_PROD]
   description = "Production API token"
   refresh_command = "~/.config/sql-cli/get_prod_token.sh"
   ```

2. **Create token getter scripts**:
   ```bash
   vim ~/.config/sql-cli/get_uat_token.sh
   vim ~/.config/sql-cli/get_prod_token.sh
   chmod 700 ~/.config/sql-cli/get_*.sh
   ```

3. **Refresh tokens before work session**:
   ```bash
   ./scripts/manage_tokens.sh refresh
   ```

4. **Use in queries**:
   ```sql
   -- Automatically uses the right token
   WITH WEB uat_data AS (
       URL 'https://uat.api/data'
       HEADERS 'Authorization': 'Bearer ${JWT_TOKEN}'
   )
   SELECT * FROM uat_data;
   ```

## Integration with Shell

For shell scripts or automation:

```bash
# Source tokens into environment
eval $(./scripts/manage_tokens.sh export)

# Now tokens are available as env vars
echo $JWT_TOKEN
echo $JWT_TOKEN_PROD

# Use with sql-cli directly
sql-cli -q "WITH WEB trades AS (URL 'https://api/trades' HEADERS 'Auth': 'Bearer \${JWT_TOKEN}') SELECT * FROM trades"
```

## Troubleshooting

- **Token not found**: Check token name matches config exactly
- **Empty token**: Verify refresh command outputs only the token
- **Permission denied**: Check script has execute permissions
- **Token expired**: Run `manage_tokens.sh refresh TOKEN_NAME`

## Future Enhancements

- Auto-refresh tokens before expiry when `auto_refresh = true`
- Token rotation strategies
- Encrypted token cache
- Integration with system keychains
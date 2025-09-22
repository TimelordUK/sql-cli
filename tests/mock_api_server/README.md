# Mock API Server for SQL-CLI Testing

This mock server replicates common enterprise REST API patterns to test SQL-CLI's WEB CTE features.

## Quick Start

```bash
# Start the server
uv run python tests/mock_api_server/server.py

# Or specify a custom port
uv run python tests/mock_api_server/server.py 8080
```

## Test Examples

### Basic GET with JSON_PATH
```sql
WITH WEB trades AS (
    URL 'http://localhost:5000/trades'
    FORMAT JSON
    JSON_PATH 'Result'
)
SELECT * FROM trades LIMIT 5;
```

### POST with Filtering
```sql
WITH WEB filtered_trades AS (
    URL 'http://localhost:5000/trades'
    METHOD POST
    BODY '{"Where": "Symbol = \"AAPL\"", "Limit": 10}'
    FORMAT JSON
    JSON_PATH 'Result'
)
SELECT TradeId, Symbol, Price * Quantity as TotalValue
FROM filtered_trades;
```

### Nested JSON Path Extraction
```sql
WITH WEB nested AS (
    URL 'http://localhost:5000/nested/data'
    FORMAT JSON
    JSON_PATH 'data.trades.Result'
)
SELECT * FROM nested;
```

### Authentication Flow
```sql
-- First get token
WITH WEB auth AS (
    URL 'http://localhost:5000/auth/token'
    METHOD POST
    BODY 'client_id=${CLIENT_ID}&client_secret=${CLIENT_SECRET}&grant_type=client_credentials'
    FORMAT JSON
),
-- Then use token for protected endpoint
WEB protected AS (
    URL 'http://localhost:5000/protected/data'
    HEADERS (
        'Authorization': CONCAT('Bearer ', (SELECT access_token FROM auth))
    )
    FORMAT JSON
    JSON_PATH 'Result'
)
SELECT * FROM protected;
```

## Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/trades` | GET/POST | Trading data with optional filtering |
| `/nested/data` | GET | Deeply nested JSON for path testing |
| `/api/v2/market-data` | POST | Advanced market data with complex filtering |
| `/csv-data` | GET | CSV data returned as JSON |
| `/auth/token` | POST | OAuth2 token endpoint |
| `/protected/data` | GET | Requires Bearer token |
| `/graphql` | POST | Mock GraphQL endpoint |
| `/batch` | POST | Batch operations |
| `/streaming/prices` | GET | Server-sent events |
| `/health` | GET | Health check |

## Features

- **Result Wrapper**: All data endpoints return `{Result: [...]}` format
- **Authentication**: OAuth2 Bearer token support
- **Filtering**: POST body with SQL-like WHERE clauses
- **Nested JSON**: Complex nested structures for JSON_PATH testing
- **Realistic Data**: Generates realistic trading/market data
- **Environment Variables**: Supports `${VAR}` substitution in headers/body

## Development

The server is designed to be extended with new endpoints as needed for testing SQL-CLI features.
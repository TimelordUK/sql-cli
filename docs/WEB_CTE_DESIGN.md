# WEB CTE Design Document

## Overview
WEB CTEs enable SQL CLI to fetch data directly from HTTP/HTTPS endpoints and use it as Common Table Expressions in SQL queries. This transforms SQL CLI into a powerful data integration tool that can combine local data with live API data.

## Core Concept
```sql
-- Basic syntax: Fetch CSV data from a URL
WITH WEB sales_data AS (
    URL 'https://api.example.com/sales.csv'
    FORMAT CSV
),
local_targets AS (
    SELECT * FROM targets_table
)
SELECT
    s.region,
    s.revenue,
    t.target,
    (s.revenue / t.target) * 100 AS achievement_pct
FROM sales_data s
JOIN local_targets t ON s.region = t.region;
```

## Syntax Design

### Basic WEB CTE Syntax
```sql
WITH WEB cte_name AS (
    URL 'https://endpoint.com/data'
    [FORMAT CSV|JSON]                    -- Data format (default: auto-detect)
    [CACHE duration]                      -- Cache results for N seconds
    [HEADERS (key = 'value', ...)]       -- HTTP headers
    [METHOD GET|POST]                     -- HTTP method (default: GET)
    [BODY 'request body']                 -- Request body for POST
    [TIMEOUT seconds]                     -- Request timeout
)
```

### Phase 1: Basic Implementation
```sql
-- Simple CSV fetch
WITH WEB weather AS (
    URL 'https://api.weather.com/current.csv'
    FORMAT CSV
)
SELECT * FROM weather WHERE temp > 20;

-- Simple JSON fetch
WITH WEB users AS (
    URL 'https://api.github.com/users/torvalds/repos'
    FORMAT JSON
)
SELECT name, stargazers_count FROM users
ORDER BY stargazers_count DESC;
```

### Phase 2: Authentication Support
```sql
-- Bearer token authentication
WITH WEB protected_data AS (
    URL 'https://api.company.com/data'
    HEADERS (
        Authorization = 'Bearer ${TOKEN}',  -- From environment variable
        Accept = 'application/json'
    )
    FORMAT JSON
)
SELECT * FROM protected_data;

-- API key authentication
WITH WEB api_data AS (
    URL 'https://api.service.com/v1/data'
    HEADERS (
        'X-API-Key' = '${API_KEY}'
    )
)
SELECT * FROM api_data;
```

### Phase 3: Advanced Features
```sql
-- POST request with body
WITH WEB graphql_result AS (
    URL 'https://api.github.com/graphql'
    METHOD POST
    HEADERS (
        Authorization = 'Bearer ${GITHUB_TOKEN}',
        'Content-Type' = 'application/json'
    )
    BODY '{
        "query": "{ viewer { login repositories(first: 5) { nodes { name } } } }"
    }'
    FORMAT JSON
)
SELECT * FROM graphql_result;

-- Caching for expensive APIs
WITH WEB fx_rates AS (
    URL 'https://api.exchangerate.com/latest'
    CACHE 3600  -- Cache for 1 hour
    FORMAT JSON
)
SELECT * FROM fx_rates;
```

## Implementation Architecture

### 1. Parser Extensions
```rust
// Add to ast.rs
pub enum CTEType {
    Standard(SelectStatement),
    Web(WebCTESpec),
}

pub struct WebCTESpec {
    pub url: String,
    pub format: DataFormat,
    pub headers: HashMap<String, String>,
    pub method: HttpMethod,
    pub body: Option<String>,
    pub cache_duration: Option<Duration>,
    pub timeout: Option<Duration>,
}

pub enum DataFormat {
    CSV,
    JSON,
    Auto,  // Auto-detect from Content-Type or extension
}
```

### 2. HTTP Client Module
```rust
// New module: src/web/http_client.rs
pub struct WebDataFetcher {
    client: reqwest::Client,
    cache: Arc<Mutex<LruCache<String, CachedResponse>>>,
}

impl WebDataFetcher {
    pub async fn fetch(&self, spec: &WebCTESpec) -> Result<DataTable> {
        // Check cache first
        if let Some(cached) = self.get_cached(&spec.url) {
            return Ok(cached);
        }

        // Build request
        let mut request = self.client
            .request(spec.method, &spec.url)
            .timeout(spec.timeout.unwrap_or(Duration::from_secs(30)));

        // Add headers
        for (key, value) in &spec.headers {
            request = request.header(key, self.resolve_value(value)?);
        }

        // Add body if POST
        if let Some(body) = &spec.body {
            request = request.body(body.clone());
        }

        // Execute request
        let response = request.send().await?;

        // Parse response based on format
        let data_table = match spec.format {
            DataFormat::CSV => self.parse_csv(response).await?,
            DataFormat::JSON => self.parse_json(response).await?,
            DataFormat::Auto => self.auto_parse(response).await?,
        };

        // Cache if specified
        if let Some(duration) = spec.cache_duration {
            self.cache_response(&spec.url, &data_table, duration);
        }

        Ok(data_table)
    }

    fn resolve_value(&self, value: &str) -> Result<String> {
        // Handle ${VAR} environment variable substitution
        if value.starts_with("${") && value.ends_with("}") {
            let var_name = &value[2..value.len()-1];
            std::env::var(var_name)
                .map_err(|_| anyhow!("Environment variable {} not found", var_name))
        } else {
            Ok(value.to_string())
        }
    }
}
```

### 3. Data Parsing
```rust
// CSV parsing
async fn parse_csv(&self, response: Response) -> Result<DataTable> {
    let text = response.text().await?;
    let mut reader = csv::Reader::from_reader(text.as_bytes());

    // Build DataTable from CSV
    let headers = reader.headers()?.clone();
    let mut table = DataTable::new("web_data");

    // Add columns
    for header in headers.iter() {
        table.add_column(DataColumn {
            name: header.to_string(),
            data_type: DataType::String,  // Infer later
            // ...
        });
    }

    // Add rows
    for result in reader.records() {
        let record = result?;
        table.add_row(/* convert record to DataRow */);
    }

    Ok(table)
}

// JSON parsing
async fn parse_json(&self, response: Response) -> Result<DataTable> {
    let json: serde_json::Value = response.json().await?;

    // Handle both array of objects and single object
    let rows = match json {
        Value::Array(arr) => arr,
        Value::Object(_) => vec![json],
        _ => return Err(anyhow!("JSON must be object or array")),
    };

    // Infer schema from first few rows
    let schema = infer_json_schema(&rows[..rows.len().min(10)]);

    // Build DataTable
    let mut table = DataTable::new("web_data");
    for (col_name, col_type) in schema {
        table.add_column(/* ... */);
    }

    // Populate rows
    for row in rows {
        table.add_row(/* extract values based on schema */);
    }

    Ok(table)
}
```

## Authentication Strategy

### Phase 1: Environment Variables
```bash
# Set tokens in environment
export API_TOKEN="sk-abc123..."
export GITHUB_TOKEN="ghp_xyz789..."

# Use in SQL
sql-cli -q "
WITH WEB data AS (
    URL 'https://api.service.com/data'
    HEADERS (Authorization = 'Bearer \${API_TOKEN}')
)
SELECT * FROM data"
```

### Phase 2: Configuration File
```toml
# ~/.config/sql-cli/web_auth.toml
[tokens]
github = "ghp_xyz789..."
openai = "sk-abc123..."

[endpoints."api.company.com"]
auth_type = "bearer"
token_env = "COMPANY_API_TOKEN"

[endpoints."data.service.com"]
auth_type = "api_key"
header_name = "X-API-Key"
key_env = "SERVICE_API_KEY"
```

### Phase 3: Dynamic Token Fetching
```sql
-- Fetch token from another endpoint first
WITH WEB auth AS (
    URL 'https://auth.company.com/token'
    METHOD POST
    BODY '{"client_id": "${CLIENT_ID}", "client_secret": "${CLIENT_SECRET}"}'
),
WEB data AS (
    URL 'https://api.company.com/data'
    HEADERS (Authorization = 'Bearer ' || (SELECT token FROM auth))
)
SELECT * FROM data;
```

## Use Cases

### 1. FX Rate Integration
```sql
WITH WEB fx_rates AS (
    URL 'https://api.exchangerate-api.com/v4/latest/USD'
    CACHE 3600
    FORMAT JSON
),
local_transactions AS (
    SELECT * FROM transactions
)
SELECT
    t.id,
    t.amount_usd,
    t.target_currency,
    t.amount_usd * fx.rates[t.target_currency] AS converted_amount
FROM local_transactions t
CROSS JOIN fx_rates fx;
```

### 2. Multi-Source Data Join
```sql
WITH WEB github_repos AS (
    URL 'https://api.github.com/users/torvalds/repos'
    FORMAT JSON
),
WEB github_user AS (
    URL 'https://api.github.com/users/torvalds'
    FORMAT JSON
)
SELECT
    u.name AS author,
    u.public_repos AS total_repos,
    r.name AS repo_name,
    r.stargazers_count
FROM github_user u
CROSS JOIN github_repos r
ORDER BY r.stargazers_count DESC
LIMIT 10;
```

### 3. Real-time Weather Analysis
```sql
WITH WEB current_weather AS (
    URL 'https://api.openweathermap.org/data/2.5/weather?q=London&appid=${WEATHER_API_KEY}'
    FORMAT JSON
),
historical_weather AS (
    SELECT * FROM weather_history WHERE city = 'London'
)
SELECT
    cw.main.temp AS current_temp,
    AVG(hw.temperature) AS historical_avg,
    cw.main.temp - AVG(hw.temperature) AS deviation
FROM current_weather cw
CROSS JOIN historical_weather hw
WHERE hw.date >= DATE_SUB(NOW(), INTERVAL 30 DAY);
```

## Security Considerations

1. **URL Whitelisting**: Option to restrict URLs to specific domains
2. **Timeout Protection**: Default timeout to prevent hanging requests
3. **Size Limits**: Maximum response size to prevent memory exhaustion
4. **Rate Limiting**: Built-in rate limiting per domain
5. **SSL Verification**: Enforce HTTPS for sensitive data
6. **Token Security**: Never log or display authentication tokens

## Configuration Options
```toml
# ~/.config/sql-cli/web.toml
[security]
allowed_domains = ["api.company.com", "*.trusted-partner.com"]
require_https = true
max_response_size_mb = 100
default_timeout_seconds = 30

[cache]
default_duration_seconds = 300
max_cache_size_mb = 500

[rate_limiting]
requests_per_minute = 60
burst_size = 10
```

## Error Handling

- Network errors: Retry with exponential backoff
- Parse errors: Provide helpful error messages with sample data
- Authentication errors: Clear indication of auth failure
- Rate limit errors: Automatic retry after delay
- Timeout errors: Suggest increasing timeout

## Performance Optimizations

1. **Connection Pooling**: Reuse HTTP connections
2. **Parallel Fetching**: Fetch multiple WEB CTEs concurrently
3. **Smart Caching**: Cache based on URL + headers
4. **Streaming**: Stream large responses instead of loading all into memory
5. **Compression**: Support gzip/deflate response compression

## Testing Strategy

1. **Mock Server**: Test suite with mock HTTP server
2. **Real APIs**: Integration tests with public APIs (GitHub, etc.)
3. **Error Scenarios**: Test network failures, timeouts, bad data
4. **Performance**: Benchmark with various response sizes
5. **Security**: Test token handling, URL validation

## Implementation Phases

### Phase 1: Basic GET with CSV/JSON (Week 1)
- [ ] Parser support for WEB keyword
- [ ] Basic HTTP GET client
- [ ] CSV parsing
- [ ] JSON flattening
- [ ] Simple integration tests

### Phase 2: Headers and Environment Variables (Week 2)
- [ ] Header support in parser
- [ ] Environment variable substitution
- [ ] Bearer token authentication
- [ ] API key authentication
- [ ] Cache implementation

### Phase 3: Advanced Features (Week 3)
- [ ] POST/PUT/DELETE methods
- [ ] Request body support
- [ ] Dynamic token fetching
- [ ] Rate limiting
- [ ] Comprehensive error handling

### Phase 4: Production Hardening (Week 4)
- [ ] Configuration file support
- [ ] URL whitelisting
- [ ] Performance optimizations
- [ ] Security audit
- [ ] Documentation and examples
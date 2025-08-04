# Schema API Design

## Overview

Make the SQL CLI completely schema-agnostic by fetching table and column definitions from the server, allowing it to work with any database schema.

## API Endpoints

### 1. Get Available Tables
```
GET /api/schema/tables
Response:
{
  "tables": [
    {
      "name": "trade_deal",
      "description": "Trading transactions",
      "row_count": 1000000
    },
    {
      "name": "instrument",
      "description": "Financial instruments",
      "row_count": 50000
    }
  ]
}
```

### 2. Get Table Schema
```
GET /api/schema/table/{tableName}
Response:
{
  "table_name": "trade_deal",
  "columns": [
    {
      "name": "dealId",
      "type": "string",
      "nullable": false,
      "description": "Unique deal identifier"
    },
    {
      "name": "platformOrderId",
      "type": "string",
      "nullable": false,
      "description": "Platform order ID"
    },
    {
      "name": "counterparty",
      "type": "string",
      "nullable": true,
      "description": "Trading counterparty"
    },
    {
      "name": "commission",
      "type": "decimal",
      "nullable": true,
      "description": "Trade commission"
    },
    {
      "name": "tradeDate",
      "type": "datetime",
      "nullable": false,
      "description": "Date of trade execution"
    }
    // ... more columns
  ],
  "methods": {
    "string": ["Contains", "StartsWith", "EndsWith"],
    "datetime": ["DateTime"]
  }
}
```

### 3. Get All Schemas (Bulk)
```
GET /api/schema/all
Response:
{
  "schemas": {
    "trade_deal": { /* full schema */ },
    "instrument": { /* full schema */ }
  }
}
```

## CLI Implementation Changes

### 1. Schema Fetching on Startup
```rust
// In enhanced_tui.rs
impl EnhancedTuiApp {
    pub fn new() -> Result<Self> {
        let api_client = ApiClient::new(&base_url);
        
        // Fetch schema from server
        let schema = match api_client.fetch_schema() {
            Ok(schema) => schema,
            Err(_) => {
                // Fall back to local schema.json
                eprintln!("Warning: Could not fetch schema from server, using local cache");
                schema_config::load_schema_config()
            }
        };
        
        // Initialize with dynamic schema
        let columns = schema.get_columns("trade_deal");
        // ...
    }
}
```

### 2. Schema Caching
```rust
// Cache schema locally for offline use
pub struct SchemaCache {
    cache_file: PathBuf,
    schema: SchemaConfig,
    last_updated: DateTime<Utc>,
}

impl SchemaCache {
    pub fn update_from_server(&mut self, api_client: &ApiClient) -> Result<()> {
        let new_schema = api_client.fetch_schema()?;
        self.schema = new_schema;
        self.last_updated = Utc::now();
        self.save_to_disk()?;
        Ok(())
    }
    
    pub fn load_or_fetch(&mut self, api_client: &ApiClient) -> Result<SchemaConfig> {
        // Try server first
        if let Ok(schema) = api_client.fetch_schema() {
            self.schema = schema;
            self.save_to_disk()?;
            return Ok(self.schema.clone());
        }
        
        // Fall back to cache
        if self.cache_file.exists() {
            self.load_from_disk()?;
            return Ok(self.schema.clone());
        }
        
        // Last resort: built-in schema
        Ok(schema_config::get_default_schema())
    }
}
```

### 3. Dynamic Completion
```rust
// Parser now uses dynamic schema
impl CursorAwareParser {
    pub fn new(schema: SchemaConfig) -> Self {
        Self {
            schema,
            // ...
        }
    }
    
    pub fn update_schema(&mut self, schema: SchemaConfig) {
        self.schema = schema;
    }
}
```

## Benefits

1. **Flexibility**: Work with any database schema without code changes
2. **Discovery**: Users can explore available tables and columns
3. **Documentation**: Column descriptions help users understand data
4. **Type Safety**: Know column types for better query validation
5. **Offline Support**: Cached schema works without server connection

## Migration Path

1. **Phase 1**: Keep hardcoded schema as fallback
2. **Phase 2**: Add server endpoint to fetch schema
3. **Phase 3**: Implement schema caching
4. **Phase 4**: Remove hardcoded schema, rely on server/cache

## Usage Flow

```bash
# First run - fetches schema from server
$ sql-cli
Fetching schema from server... done!
Available tables: trade_deal, instrument, portfolio

# Subsequent runs - uses cached schema
$ sql-cli
Using cached schema (updated: 2024-08-04 10:30)

# Force schema refresh
$ sql-cli --refresh-schema
Refreshing schema from server... done!

# View schema information
sql> \describe trade_deal
Table: trade_deal
Columns:
  - dealId (string, not null): Unique deal identifier
  - platformOrderId (string, not null): Platform order ID
  - counterparty (string): Trading counterparty
  - commission (decimal): Trade commission
  ...
```

## Server Implementation (C#)

```csharp
[HttpGet("api/schema/table/{tableName}")]
public IActionResult GetTableSchema(string tableName)
{
    var schema = _schemaService.GetTableSchema(tableName);
    if (schema == null)
        return NotFound();
    
    return Ok(new
    {
        table_name = tableName,
        columns = schema.Columns.Select(c => new
        {
            name = c.Name,
            type = c.DataType,
            nullable = c.IsNullable,
            description = c.Description
        }),
        methods = new
        {
            @string = new[] { "Contains", "StartsWith", "EndsWith" },
            datetime = new[] { "DateTime" }
        }
    });
}
```
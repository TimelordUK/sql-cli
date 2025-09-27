# Web CTE File Upload Support

## Problem
Need to support file uploads in Web CTEs for querying large FIX message files (40MB+) through the JSON selector service.

## Current Limitation
Web CTE currently supports:
- JSON body via `BODY 'json_string'`
- Headers via `HEADERS`
- Methods (GET, POST, etc.)

But NOT file uploads via multipart/form-data.

## Proposed Solution

### 1. Extend WebCTESpec in ast.rs
```rust
pub struct WebCTESpec {
    pub url: String,
    pub format: Option<DataFormat>,
    pub headers: Vec<(String, String)>,
    pub cache_seconds: Option<u64>,
    pub method: Option<HttpMethod>,
    pub body: Option<String>,
    pub json_path: Option<String>,
    // NEW: Support for multipart/form-data file uploads
    pub form_files: Vec<(String, String)>,  // (field_name, file_path)
    pub form_fields: Vec<(String, String)>, // (field_name, value)
}
```

### 2. Parser Support
Add new clauses to parse_web_cte_spec():
```sql
WITH fix_messages AS WEB (
    URL 'http://localhost:5050/api/messagequery/example/fix',
    METHOD 'POST',
    FORM_FILE 'file' '/path/to/messages.json',  -- NEW
    FORM_FIELD 'query' '{"outputFormat": "csv"}' -- NEW (optional)
)
```

### 3. HTTP Client Implementation
In web/http_fetcher.rs, detect form_files and use multipart:
```rust
if !web_spec.form_files.is_empty() {
    // Build multipart/form-data request
    let form = reqwest::multipart::Form::new();

    // Add files
    for (field_name, file_path) in &web_spec.form_files {
        let file = std::fs::File::open(file_path)?;
        let part = reqwest::multipart::Part::file(file)?;
        form = form.part(field_name, part);
    }

    // Add regular form fields
    for (field_name, value) in &web_spec.form_fields {
        form = form.text(field_name, value);
    }

    request = request.multipart(form);
}
```

## Alternative Workarounds (Current)

### 1. Two-Step Process
```bash
# Step 1: Upload and save result
curl -X POST http://localhost:5050/api/messagequery/example/fix \
  -F "file=@messages.json" \
  -o /tmp/result.csv

# Step 2: Query with SQL CLI
sql-cli /tmp/result.csv -q "SELECT * FROM result WHERE quantity > 100"
```

### 2. Proxy Script
Create a wrapper script that handles the upload:
```bash
#!/bin/bash
# upload_and_query.sh
FILE=$1
QUERY=$2
curl -X POST http://localhost:5050/api/messagequery/example/fix \
  -F "file=@$FILE" | sql-cli - -q "$QUERY"
```

### 3. JSON Body Approach (Limited)
For smaller files, load into memory and send as JSON:
```sql
-- Only works for small files that fit in memory
WITH fix_messages AS WEB (
    URL 'http://localhost:5050/api/messagequery/json/fix',
    METHOD 'POST',
    BODY '[{"header": {...}, "body": {...}}]'
)
```

## Implementation Priority

Given that:
1. Files can be 40MB+ (too large for JSON body approach)
2. File upload is essential for the FIX workflow
3. The service already supports file uploads

**Recommendation**: Implement file upload support in Web CTE as outlined above.

## Files to Modify

1. `src/sql/parser/ast.rs` - Add form_files/form_fields to WebCTESpec
2. `src/sql/recursive_parser.rs` - Parse FORM_FILE and FORM_FIELD clauses
3. `src/web/http_fetcher.rs` - Handle multipart/form-data requests
4. `Cargo.toml` - Ensure reqwest has multipart feature enabled
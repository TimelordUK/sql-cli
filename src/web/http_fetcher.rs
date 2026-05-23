// HTTP fetcher for WEB CTEs (now supports file:// URLs too)
use anyhow::{Context, Result};
use regex::Regex;
use serde_json;
use std::fs::File;
use std::io::Cursor;
use std::path::Path;
use std::time::Duration;
use tracing::{debug, info};

use crate::data::datatable::DataTable;
use crate::data::stream_loader::{
    load_csv_from_reader_with_opts, load_json_from_reader, CsvReadOptions,
};
use crate::sql::parser::ast::{DataFormat, HttpMethod, WebCTESpec};

#[cfg(feature = "redis-cache")]
use crate::redis_cache_module::RedisCache;

/// Fetches data from a URL and converts it to a DataTable
pub struct WebDataFetcher {
    client: reqwest::blocking::Client,
}

impl WebDataFetcher {
    pub fn new() -> Result<Self> {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("sql-cli/1.0")
            .build()?;

        Ok(Self { client })
    }

    /// Fetch data from a WEB CTE specification (supports http://, https://, and file:// URLs)
    /// Each WEB CTE is cached independently based only on its own properties
    pub fn fetch(
        &self,
        spec: &WebCTESpec,
        table_name: &str,
        _query_context: Option<&str>, // Kept for API compatibility but not used for caching
    ) -> Result<DataTable> {
        // Check if this is a file:// URL (no caching for local files)
        if spec.url.starts_with("file://") {
            return self.fetch_file(spec, table_name);
        }

        // Generate cache key from ALL Web CTE spec fields
        // Each WEB CTE is independent - cache key depends only on the CTE itself
        #[cfg(feature = "redis-cache")]
        let cache_key = {
            let method = format!("{:?}", spec.method.as_ref().unwrap_or(&HttpMethod::GET));

            // Use the full cache key generation with all WebCTESpec fields
            RedisCache::generate_key_full(
                table_name, // CTE name
                &spec.url,
                Some(&method),
                &spec.headers,
                spec.body.as_deref(),
                "",                        // Empty context - not used for independent caching
                spec.json_path.as_deref(), // JSON extraction path
                &spec.form_files,          // Multipart form files
                &spec.form_fields,         // Multipart form fields
            )
        };

        // Try cache first
        #[cfg(feature = "redis-cache")]
        {
            let mut cache = RedisCache::new();
            if cache.is_enabled() {
                if let Some(cached_bytes) = cache.get(&cache_key) {
                    // Try to deserialize from cached parquet
                    match DataTable::from_parquet_bytes(&cached_bytes) {
                        Ok(table) => {
                            eprintln!(
                                "Cache HIT for {} (key: {}...)",
                                table_name,
                                &cache_key[0..48.min(cache_key.len())]
                            );
                            return Ok(table);
                        }
                        Err(e) => {
                            debug!("Failed to deserialize cached data: {}", e);
                            // Continue to fetch from network
                        }
                    }
                } else {
                    eprintln!(
                        "Cache MISS for {} (key: {}...)",
                        table_name,
                        &cache_key[0..48.min(cache_key.len())]
                    );
                }
            }
        }

        info!("Fetching data from URL: {}", spec.url);

        // Regular HTTP/HTTPS handling
        // Build request based on method
        let mut request = match spec.method.as_ref().unwrap_or(&HttpMethod::GET) {
            HttpMethod::GET => self.client.get(&spec.url),
            HttpMethod::POST => self.client.post(&spec.url),
            HttpMethod::PUT => self.client.put(&spec.url),
            HttpMethod::DELETE => self.client.delete(&spec.url),
            HttpMethod::PATCH => self.client.patch(&spec.url),
        };

        // Add headers if provided
        for (key, value) in &spec.headers {
            let resolved_value = self.resolve_env_var(value)?;
            request = request.header(key, resolved_value);
        }

        // Handle multipart form data if form_files are specified
        if !spec.form_files.is_empty() || !spec.form_fields.is_empty() {
            let mut form = reqwest::blocking::multipart::Form::new();

            // Add files
            for (field_name, file_path) in &spec.form_files {
                let resolved_path = self.resolve_env_var(file_path)?;
                let file = std::fs::File::open(&resolved_path)
                    .with_context(|| format!("Failed to open file: {}", resolved_path))?;
                let file_name = std::path::Path::new(&resolved_path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("file")
                    .to_string();
                let part = reqwest::blocking::multipart::Part::reader(file).file_name(file_name);
                form = form.part(field_name.clone(), part);
            }

            // Add regular form fields
            for (field_name, value) in &spec.form_fields {
                let resolved_value = self.resolve_env_var(value)?;
                form = form.text(field_name.clone(), resolved_value);
            }

            request = request.multipart(form);
        }
        // Add body if provided (typically for POST/PUT/PATCH) - only if not using multipart
        else if let Some(body) = &spec.body {
            let resolved_body = self.resolve_env_var(body)?;

            // Debug: Always print the expanded body to help verify template expansion
            eprintln!("\n=== WEB CTE Request Debug ===");
            eprintln!("URL: {}", spec.url);
            eprintln!(
                "Method: {:?}",
                spec.method.as_ref().unwrap_or(&HttpMethod::POST)
            );
            eprintln!("Body (after template expansion):");
            eprintln!("{}", resolved_body);
            eprintln!("=============================\n");

            request = request.body(resolved_body);
            // Set Content-Type to JSON if not already set and body looks like JSON
            if spec.body.as_ref().unwrap().trim().starts_with('{') {
                request = request.header("Content-Type", "application/json");
            }
        }

        // Execute request
        let response = request
            .send()
            .with_context(|| format!("Failed to fetch from URL: {}", spec.url))?;

        // Check status
        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "HTTP request failed with status {}: {}",
                response.status(),
                spec.url
            ));
        }

        // Get content type for format detection
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        debug!("Response content-type: {}", content_type);

        // Read response body
        let bytes = response.bytes()?;

        // Determine format
        let format = match &spec.format {
            Some(fmt) => fmt.clone(),
            None => self.detect_format(&spec.url, &content_type),
        };

        info!("Using format: {:?} for {}", format, spec.url);

        // Parse the data based on format
        let result = if let Some(json_path) = &spec.json_path {
            if matches!(format, DataFormat::JSON | DataFormat::Auto) {
                // Parse JSON and extract the path
                let json_value: serde_json::Value = serde_json::from_slice(&bytes)
                    .with_context(|| "Failed to parse JSON for path extraction")?;

                // Navigate to the specified path
                let extracted = self
                    .navigate_json_path(&json_value, json_path)
                    .with_context(|| format!("Failed to extract JSON path: {}", json_path))?;

                // Convert extracted value to bytes and parse as table
                let array_value = match extracted {
                    serde_json::Value::Array(_) => extracted.clone(),
                    _ => serde_json::Value::Array(vec![extracted.clone()]),
                };

                let extracted_bytes = serde_json::to_vec(&array_value)?;
                // JSON path: delimiter is irrelevant (output is JSON), but the
                // arg is still required by the function signature.
                self.parse_data(
                    extracted_bytes,
                    DataFormat::JSON,
                    table_name,
                    "web",
                    &spec.url,
                    spec.delimiter,
                )?
            } else {
                self.parse_data(
                    bytes.to_vec(),
                    format,
                    table_name,
                    "web",
                    &spec.url,
                    spec.delimiter,
                )?
            }
        } else {
            self.parse_data(
                bytes.to_vec(),
                format,
                table_name,
                "web",
                &spec.url,
                spec.delimiter,
            )?
        };

        // Cache the result if caching is enabled
        #[cfg(feature = "redis-cache")]
        {
            let mut cache = RedisCache::new();
            if cache.is_enabled() {
                // Determine TTL based on spec or smart defaults
                let ttl = spec.cache_seconds.unwrap_or_else(|| {
                    // Smart defaults based on URL and body
                    if spec.url.contains("prod") {
                        3600 // 1 hour for production
                    } else if spec.url.contains("staging") {
                        300 // 5 minutes for staging
                    } else {
                        600 // 10 minutes default
                    }
                });

                // Serialize to parquet and cache
                match result.to_parquet_bytes() {
                    Ok(parquet_bytes) => {
                        if let Err(e) = cache.set(&cache_key, &parquet_bytes, ttl) {
                            debug!("Failed to cache result: {}", e);
                        } else {
                            eprintln!("Cached {} for {} seconds", table_name, ttl);
                        }
                    }
                    Err(e) => {
                        debug!("Failed to serialize to parquet: {}", e);
                    }
                }
            }
        }

        Ok(result)
    }

    /// Fetch data from a file:// URL
    fn fetch_file(&self, spec: &WebCTESpec, table_name: &str) -> Result<DataTable> {
        // Extract path from file:// URL
        let file_path = if spec.url.starts_with("file://") {
            &spec.url[7..] // Remove "file://" prefix
        } else {
            &spec.url
        };

        info!("Reading local file: {}", file_path);

        // Check if file exists
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(anyhow::anyhow!("File not found: {}", file_path));
        }

        // Open file
        let file =
            File::open(path).with_context(|| format!("Failed to open file: {}", file_path))?;

        // Get file size for memory checks
        let metadata = file.metadata()?;
        let file_size = metadata.len();
        debug!("File size: {} bytes", file_size);

        // Determine format from extension or spec
        let format = match &spec.format {
            Some(fmt) => fmt.clone(),
            None => self.detect_format(file_path, ""),
        };

        info!("Using format: {:?} for {}", format, file_path);

        // Build CSV options once; DELIMITER clause wins, else comma. URL paths
        // are unreliable for extension auto-detect (query strings, redirects),
        // so we don't probe them here.
        let csv_opts = CsvReadOptions {
            delimiter: spec.delimiter.unwrap_or(b','),
            has_headers: true,
        };

        // Parse based on format
        match format {
            DataFormat::CSV => {
                load_csv_from_reader_with_opts(file, table_name, "file", file_path, &csv_opts)
                    .with_context(|| format!("Failed to parse CSV from {}", file_path))
            }
            DataFormat::JSON => load_json_from_reader(file, table_name, "file", file_path)
                .with_context(|| format!("Failed to parse JSON from {}", file_path)),
            DataFormat::Auto => {
                // For files, we can't retry with the same reader, so determine based on extension
                if file_path.ends_with(".json") {
                    let file = File::open(path)?;
                    load_json_from_reader(file, table_name, "file", file_path)
                        .with_context(|| format!("Failed to parse JSON from {}", file_path))
                } else {
                    // Default to CSV for auto-detect with files
                    let file = File::open(path)?;
                    load_csv_from_reader_with_opts(file, table_name, "file", file_path, &csv_opts)
                        .with_context(|| format!("Failed to parse CSV from {}", file_path))
                }
            }
        }
    }

    /// Parse data bytes based on format. `delimiter` is honoured for CSV (and
    /// Auto when it falls back to CSV); `None` means comma.
    fn parse_data(
        &self,
        bytes: Vec<u8>,
        format: DataFormat,
        table_name: &str,
        source_type: &str,
        source_path: &str,
        delimiter: Option<u8>,
    ) -> Result<DataTable> {
        let csv_opts = CsvReadOptions {
            delimiter: delimiter.unwrap_or(b','),
            has_headers: true,
        };
        match format {
            DataFormat::CSV => {
                let reader = Cursor::new(bytes);
                load_csv_from_reader_with_opts(
                    reader,
                    table_name,
                    source_type,
                    source_path,
                    &csv_opts,
                )
                .with_context(|| format!("Failed to parse CSV from {}", source_path))
            }
            DataFormat::JSON => {
                let reader = Cursor::new(bytes);
                load_json_from_reader(reader, table_name, source_type, source_path)
                    .with_context(|| format!("Failed to parse JSON from {}", source_path))
            }
            DataFormat::Auto => {
                // Try CSV first, then JSON
                let reader_csv = Cursor::new(bytes.clone());
                match load_csv_from_reader_with_opts(
                    reader_csv,
                    table_name,
                    source_type,
                    source_path,
                    &csv_opts,
                ) {
                    Ok(table) => Ok(table),
                    Err(_) => {
                        debug!("CSV parsing failed, trying JSON");
                        let reader_json = Cursor::new(bytes);
                        load_json_from_reader(reader_json, table_name, source_type, source_path)
                            .with_context(|| format!("Failed to parse data from {}", source_path))
                    }
                }
            }
        }
    }

    /// Detect format from URL extension or content type
    fn detect_format(&self, url: &str, content_type: &str) -> DataFormat {
        // Check content type first
        if content_type.contains("json") {
            return DataFormat::JSON;
        }
        if content_type.contains("csv") || content_type.contains("text/plain") {
            return DataFormat::CSV;
        }

        // Check URL extension
        if url.ends_with(".json") {
            DataFormat::JSON
        } else if url.ends_with(".csv") {
            DataFormat::CSV
        } else {
            // Default to auto-detect
            DataFormat::Auto
        }
    }

    /// Extract data from a specific JSON path
    fn extract_json_path(
        &self,
        _table: DataTable,
        json_path: &str,
        bytes: &[u8],
    ) -> Result<DataTable> {
        // Parse the JSON
        let json_value: serde_json::Value = serde_json::from_slice(bytes)
            .with_context(|| "Failed to parse JSON for path extraction")?;

        // Navigate to the specified path
        let extracted = self
            .navigate_json_path(&json_value, json_path)
            .with_context(|| format!("Failed to extract JSON path: {}", json_path))?;

        // If the extracted value is already an array, use it directly
        // Otherwise, wrap it in an array for consistent handling
        let array_value = match extracted {
            serde_json::Value::Array(_) => extracted.clone(),
            _ => serde_json::Value::Array(vec![extracted.clone()]),
        };

        // Convert to bytes and parse as a table
        let extracted_bytes = serde_json::to_vec(&array_value)?;

        // Re-parse the extracted JSON as a DataTable
        let reader = Cursor::new(extracted_bytes);
        load_json_from_reader(reader, "extracted", "web", json_path)
    }

    /// Navigate to a specific path in JSON structure
    fn navigate_json_path<'a>(
        &self,
        value: &'a serde_json::Value,
        path: &str,
    ) -> Result<&'a serde_json::Value> {
        let mut current = value;

        // Split path by dots (simple path navigation for now)
        // Future enhancement: support array indexing like "Result[0]"
        for part in path.split('.') {
            if part.is_empty() {
                continue;
            }

            current = current
                .get(part)
                .ok_or_else(|| anyhow::anyhow!("Path '{}' not found in JSON", part))?;
        }

        Ok(current)
    }

    /// Resolve environment variables in values (${VAR_NAME} or $VAR_NAME syntax)
    fn resolve_env_var(&self, value: &str) -> Result<String> {
        let mut result = value.to_string();

        // Handle ${VAR} syntax - can be embedded in strings
        // Use lazy_static for better performance, but for now just compile inline
        let re = Regex::new(r"\$\{([^}]+)\}").unwrap();
        for cap in re.captures_iter(value) {
            let var_name = &cap[1];
            match std::env::var(var_name) {
                Ok(var_value) => {
                    result = result.replace(&cap[0], &var_value);
                }
                Err(_) => {
                    // For security, don't expose which env vars exist
                    // Just log a debug message and keep the placeholder
                    debug!(
                        "Environment variable {} not found, keeping placeholder",
                        var_name
                    );
                }
            }
        }

        // Also handle simple $VAR syntax at the start of the string
        if result.starts_with('$') && !result.starts_with("${") {
            let var_name = &result[1..];
            if let Ok(var_value) = std::env::var(var_name) {
                return Ok(var_value);
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_format() {
        let fetcher = WebDataFetcher::new().unwrap();

        // Test URL-based detection
        assert!(matches!(
            fetcher.detect_format("http://example.com/data.csv", ""),
            DataFormat::CSV
        ));
        assert!(matches!(
            fetcher.detect_format("http://example.com/data.json", ""),
            DataFormat::JSON
        ));

        // Test content-type detection
        assert!(matches!(
            fetcher.detect_format("http://example.com/data", "application/json"),
            DataFormat::JSON
        ));
        assert!(matches!(
            fetcher.detect_format("http://example.com/data", "text/csv"),
            DataFormat::CSV
        ));

        // Test auto-detect fallback
        assert!(matches!(
            fetcher.detect_format("http://example.com/data", ""),
            DataFormat::Auto
        ));
    }
}

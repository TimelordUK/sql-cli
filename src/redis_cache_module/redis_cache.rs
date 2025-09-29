use redis::{Client, Commands, Connection};
use sha2::{Digest, Sha256};
use std::time::Duration;
use tracing::{debug, info};

pub struct RedisCache {
    connection: Option<Connection>,
    enabled: bool,
}

impl RedisCache {
    /// Try to create a new Redis cache connection
    pub fn new() -> Self {
        // Cache is OPT-IN - must explicitly enable via SQL_CLI_CACHE=true
        // This ensures sql-cli works exactly as before for users without Redis
        match std::env::var("SQL_CLI_CACHE") {
            Ok(val) => {
                // Only proceed if explicitly enabled
                if !val.eq_ignore_ascii_case("true")
                    && !val.eq_ignore_ascii_case("yes")
                    && val != "1"
                {
                    debug!("Cache not enabled (SQL_CLI_CACHE != true)");
                    return Self {
                        connection: None,
                        enabled: false,
                    };
                }
            }
            Err(_) => {
                // No SQL_CLI_CACHE variable = cache disabled (default)
                debug!("Cache disabled by default (set SQL_CLI_CACHE=true to enable)");
                return Self {
                    connection: None,
                    enabled: false,
                };
            }
        }

        debug!("Cache explicitly enabled via SQL_CLI_CACHE=true");

        // Get Redis URL from environment or use default
        let redis_url = std::env::var("SQL_CLI_REDIS_URL")
            .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

        // Try to connect with a short timeout
        match Client::open(redis_url.as_str()) {
            Ok(client) => {
                // Try to get connection with timeout
                match client.get_connection_with_timeout(Duration::from_secs(1)) {
                    Ok(mut conn) => {
                        // Test the connection
                        match redis::cmd("PING").query::<String>(&mut conn) {
                            Ok(_) => {
                                debug!("Redis cache connected successfully");
                                Self {
                                    connection: Some(conn),
                                    enabled: true,
                                }
                            }
                            Err(e) => {
                                debug!("Redis ping failed: {}", e);
                                Self {
                                    connection: None,
                                    enabled: false,
                                }
                            }
                        }
                    }
                    Err(e) => {
                        debug!("Redis connection failed: {}", e);
                        Self {
                            connection: None,
                            enabled: false,
                        }
                    }
                }
            }
            Err(e) => {
                debug!("Redis client creation failed: {}", e);
                Self {
                    connection: None,
                    enabled: false,
                }
            }
        }
    }

    /// Generate a cache key from Web CTE components (legacy - kept for compatibility)
    pub fn generate_key(
        table_name: &str, // CTE name to prevent collisions
        url: &str,
        method: Option<&str>,
        headers: &[(String, String)],
        body: Option<&str>,
    ) -> String {
        // Call the new method with empty context for backward compatibility
        Self::generate_key_with_context(table_name, url, method, headers, body, "")
    }

    /// Generate a cache key from Web CTE components with query context
    pub fn generate_key_with_context(
        table_name: &str, // CTE name
        url: &str,
        method: Option<&str>,
        headers: &[(String, String)],
        body: Option<&str>,
        query_context: &str, // Hash or unique identifier of the full query
    ) -> String {
        Self::generate_key_full(
            table_name,
            url,
            method,
            headers,
            body,
            query_context,
            None, // json_path
            &[],  // form_files
            &[],  // form_fields
        )
    }

    /// Generate a complete cache key from all Web CTE components
    pub fn generate_key_full(
        table_name: &str,
        url: &str,
        method: Option<&str>,
        headers: &[(String, String)],
        body: Option<&str>,
        query_context: &str,
        json_path: Option<&str>,
        form_files: &[(String, String)],
        form_fields: &[(String, String)],
    ) -> String {
        let mut hasher = Sha256::new();

        // Hash the query context first (most important for preventing collisions)
        hasher.update(query_context.as_bytes());
        hasher.update(b":::"); // Separator

        // Hash the CTE name
        hasher.update(table_name.as_bytes());
        hasher.update(b":::"); // Separator

        // Hash URL
        hasher.update(url.as_bytes());
        hasher.update(b":::");

        // Hash method
        if let Some(method) = method {
            hasher.update(method.as_bytes());
            hasher.update(b":::");
        }

        // Hash headers (sorted for consistency)
        let mut sorted_headers = headers.to_vec();
        sorted_headers.sort_by(|a, b| a.0.cmp(&b.0));
        for (key, value) in sorted_headers {
            hasher.update(key.as_bytes());
            hasher.update(b":");
            hasher.update(value.as_bytes());
            hasher.update(b";");
        }

        // Hash body
        if let Some(body) = body {
            hasher.update(b"body:");
            hasher.update(body.as_bytes());
            hasher.update(b":::");
        }

        // Hash json_path
        if let Some(path) = json_path {
            hasher.update(b"json_path:");
            hasher.update(path.as_bytes());
            hasher.update(b":::");
        }

        // Hash form_files (sorted for consistency)
        if !form_files.is_empty() {
            let mut sorted_files = form_files.to_vec();
            sorted_files.sort_by(|a, b| a.0.cmp(&b.0));
            for (field, path) in sorted_files {
                hasher.update(b"file:");
                hasher.update(field.as_bytes());
                hasher.update(b"=");
                hasher.update(path.as_bytes());
                hasher.update(b";");
            }
        }

        // Hash form_fields (sorted for consistency)
        if !form_fields.is_empty() {
            let mut sorted_fields = form_fields.to_vec();
            sorted_fields.sort_by(|a, b| a.0.cmp(&b.0));
            for (field, value) in sorted_fields {
                hasher.update(b"field:");
                hasher.update(field.as_bytes());
                hasher.update(b"=");
                hasher.update(value.as_bytes());
                hasher.update(b";");
            }
        }

        format!("sql-cli:web:{}:{:x}", table_name, hasher.finalize())
    }

    /// Check if cache is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get data from cache
    pub fn get(&mut self, key: &str) -> Option<Vec<u8>> {
        if !self.enabled {
            return None;
        }

        if let Some(ref mut conn) = self.connection {
            match conn.get::<_, Vec<u8>>(key) {
                Ok(data) => {
                    debug!("Cache HIT: {}", &key[0..32.min(key.len())]);
                    Some(data)
                }
                Err(_) => {
                    debug!("Cache MISS: {}", &key[0..32.min(key.len())]);
                    None
                }
            }
        } else {
            None
        }
    }

    /// Store data in cache with TTL
    pub fn set(&mut self, key: &str, data: &[u8], ttl_seconds: u64) -> Result<(), String> {
        if !self.enabled {
            return Ok(());
        }

        if let Some(ref mut conn) = self.connection {
            match conn.set_ex::<_, _, String>(key, data, ttl_seconds as usize) {
                Ok(_) => {
                    info!("Cached {} bytes with TTL {}s", data.len(), ttl_seconds);
                    Ok(())
                }
                Err(e) => {
                    debug!("Failed to cache: {}", e);
                    // Don't fail the query just because caching failed
                    Ok(())
                }
            }
        } else {
            Ok(())
        }
    }

    /// Get TTL of a key (for debugging)
    pub fn ttl(&mut self, key: &str) -> Option<i64> {
        if !self.enabled {
            return None;
        }

        if let Some(ref mut conn) = self.connection {
            conn.ttl(key).ok()
        } else {
            None
        }
    }

    /// Check cache statistics
    pub fn stats(&mut self) -> Option<String> {
        if !self.enabled {
            return None;
        }

        if let Some(ref mut conn) = self.connection {
            // Get all sql-cli keys
            if let Ok(keys) = redis::cmd("KEYS")
                .arg("sql-cli:*")
                .query::<Vec<String>>(conn)
            {
                let count = keys.len();
                let mut total_size = 0;
                let mut expiring_soon = 0;

                for key in &keys {
                    // Get memory usage
                    if let Ok(size) = redis::cmd("MEMORY")
                        .arg("USAGE")
                        .arg(key)
                        .query::<Option<usize>>(conn)
                    {
                        total_size += size.unwrap_or(0);
                    }

                    // Check TTL
                    if let Ok(ttl) = conn.ttl::<_, i64>(key) {
                        if ttl > 0 && ttl < 300 {
                            expiring_soon += 1;
                        }
                    }
                }

                return Some(format!(
                    "Cache stats: {} entries, {:.2} MB, {} expiring soon",
                    count,
                    total_size as f64 / 1_048_576.0,
                    expiring_soon
                ));
            }
        }

        None
    }
}

impl Default for RedisCache {
    fn default() -> Self {
        Self::new()
    }
}

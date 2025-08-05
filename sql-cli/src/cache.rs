use std::path::PathBuf;
use std::fs;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use chrono::{DateTime, Local};
use sha2::{Sha256, Digest};
use std::error::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedQuery {
    pub id: u64,
    pub query_hash: String,
    pub query_text: String,
    pub timestamp: DateTime<Local>,
    pub row_count: usize,
    pub file_path: String,
    pub description: Option<String>,
    pub expires_at: Option<DateTime<Local>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetadata {
    pub queries: Vec<CachedQuery>,
    pub next_id: u64,
}

pub struct QueryCache {
    cache_dir: PathBuf,
    metadata_path: PathBuf,
    metadata: CacheMetadata,
}

impl QueryCache {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let home_dir = dirs::home_dir().ok_or("Cannot find home directory")?;
        let cache_dir = home_dir.join(".sql-cli").join("cache");
        let data_dir = cache_dir.join("data");
        
        // Create directories if they don't exist
        fs::create_dir_all(&data_dir)?;
        
        let metadata_path = cache_dir.join("metadata.json");
        
        // Load or create metadata
        let metadata = if metadata_path.exists() {
            let content = fs::read_to_string(&metadata_path)?;
            serde_json::from_str(&content)?
        } else {
            CacheMetadata {
                queries: Vec::new(),
                next_id: 1,
            }
        };
        
        Ok(Self {
            cache_dir,
            metadata_path,
            metadata,
        })
    }
    
    pub fn save_query(
        &mut self, 
        query: &str, 
        data: &[Value], 
        description: Option<String>
    ) -> Result<u64, Box<dyn Error>> {
        // Generate hash for query
        let mut hasher = Sha256::new();
        hasher.update(query.as_bytes());
        let query_hash = format!("{:x}", hasher.finalize());
        
        // Check if query already cached
        if let Some(existing) = self.metadata.queries.iter().find(|q| q.query_hash == query_hash) {
            return Ok(existing.id);
        }
        
        // Generate filename
        let id = self.metadata.next_id;
        let filename = format!("query_{:06}.json", id);
        let file_path = self.cache_dir.join("data").join(&filename);
        
        // Save data to file
        let json_data = serde_json::to_string_pretty(&data)?;
        fs::write(&file_path, json_data)?;
        
        // Add to metadata
        let cached_query = CachedQuery {
            id,
            query_hash,
            query_text: query.to_string(),
            timestamp: Local::now(),
            row_count: data.len(),
            file_path: filename,
            description,
            expires_at: None, // Could add TTL logic here
        };
        
        self.metadata.queries.push(cached_query);
        self.metadata.next_id += 1;
        
        // Save metadata
        self.save_metadata()?;
        
        Ok(id)
    }
    
    pub fn load_query(&self, id: u64) -> Result<(String, Vec<Value>), Box<dyn Error>> {
        let cached_query = self.metadata.queries
            .iter()
            .find(|q| q.id == id)
            .ok_or(format!("Cache entry {} not found", id))?;
        
        let file_path = self.cache_dir.join("data").join(&cached_query.file_path);
        let json_data = fs::read_to_string(file_path)?;
        let data: Vec<Value> = serde_json::from_str(&json_data)?;
        
        Ok((cached_query.query_text.clone(), data))
    }
    
    pub fn list_cached_queries(&self) -> &[CachedQuery] {
        &self.metadata.queries
    }
    
    pub fn delete_query(&mut self, id: u64) -> Result<(), Box<dyn Error>> {
        if let Some(pos) = self.metadata.queries.iter().position(|q| q.id == id) {
            let cached_query = self.metadata.queries.remove(pos);
            let file_path = self.cache_dir.join("data").join(&cached_query.file_path);
            fs::remove_file(file_path)?;
            self.save_metadata()?;
        }
        Ok(())
    }
    
    pub fn clear_all(&mut self) -> Result<(), Box<dyn Error>> {
        // Remove all data files
        let data_dir = self.cache_dir.join("data");
        for entry in fs::read_dir(data_dir)? {
            let entry = entry?;
            if entry.path().extension().map_or(false, |ext| ext == "json") {
                fs::remove_file(entry.path())?;
            }
        }
        
        // Clear metadata
        self.metadata.queries.clear();
        self.metadata.next_id = 1;
        self.save_metadata()?;
        
        Ok(())
    }
    
    pub fn get_cache_stats(&self) -> CacheStats {
        let total_size: u64 = self.metadata.queries.iter()
            .filter_map(|q| {
                let path = self.cache_dir.join("data").join(&q.file_path);
                fs::metadata(path).ok().map(|m| m.len())
            })
            .sum();
        
        let total_rows: usize = self.metadata.queries.iter()
            .map(|q| q.row_count)
            .sum();
        
        CacheStats {
            total_queries: self.metadata.queries.len(),
            total_rows,
            total_size_bytes: total_size,
            oldest_entry: self.metadata.queries.iter()
                .min_by_key(|q| q.timestamp)
                .map(|q| q.timestamp),
            newest_entry: self.metadata.queries.iter()
                .max_by_key(|q| q.timestamp)
                .map(|q| q.timestamp),
        }
    }
    
    fn save_metadata(&self) -> Result<(), Box<dyn Error>> {
        let json = serde_json::to_string_pretty(&self.metadata)?;
        fs::write(&self.metadata_path, json)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct CacheStats {
    pub total_queries: usize,
    pub total_rows: usize,
    pub total_size_bytes: u64,
    pub oldest_entry: Option<DateTime<Local>>,
    pub newest_entry: Option<DateTime<Local>>,
}

impl CacheStats {
    pub fn format_size(&self) -> String {
        let size = self.total_size_bytes as f64;
        if size < 1024.0 {
            format!("{} B", size)
        } else if size < 1024.0 * 1024.0 {
            format!("{:.1} KB", size / 1024.0)
        } else if size < 1024.0 * 1024.0 * 1024.0 {
            format!("{:.1} MB", size / (1024.0 * 1024.0))
        } else {
            format!("{:.1} GB", size / (1024.0 * 1024.0 * 1024.0))
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum QueryMode {
    Live,      // Always query server
    Cached,    // Only use cached data
    Hybrid,    // Check cache first, then server
}

impl Default for QueryMode {
    fn default() -> Self {
        QueryMode::Live
    }
}
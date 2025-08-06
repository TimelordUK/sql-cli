use std::error::Error;
use std::fs;
use std::path::PathBuf;

pub struct AppPaths;

impl AppPaths {
    pub fn data_dir() -> Result<PathBuf, Box<dyn Error>> {
        let data_dir = dirs::data_dir()
            .ok_or("Cannot determine data directory")?
            .join("sql-cli");

        fs::create_dir_all(&data_dir)?;
        Ok(data_dir)
    }

    pub fn cache_dir() -> Result<PathBuf, Box<dyn Error>> {
        let cache_dir = dirs::cache_dir()
            .ok_or("Cannot determine cache directory")?
            .join("sql-cli");

        fs::create_dir_all(&cache_dir)?;
        Ok(cache_dir)
    }

    pub fn history_file() -> Result<PathBuf, Box<dyn Error>> {
        Ok(Self::data_dir()?.join("history.json"))
    }

    pub fn schemas_file() -> Result<PathBuf, Box<dyn Error>> {
        Ok(Self::data_dir()?.join("schemas.json"))
    }

    pub fn cache_metadata_file() -> Result<PathBuf, Box<dyn Error>> {
        Ok(Self::cache_dir()?.join("metadata.json"))
    }

    pub fn cache_data_dir() -> Result<PathBuf, Box<dyn Error>> {
        let data_dir = Self::cache_dir()?.join("data");
        fs::create_dir_all(&data_dir)?;
        Ok(data_dir)
    }
}

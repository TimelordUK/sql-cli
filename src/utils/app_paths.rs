use std::error::Error;
use std::fs;
use std::path::PathBuf;

pub struct AppPaths;

impl AppPaths {
    /// Env var that overrides the base directory for all app paths. When set,
    /// both data and cache resolve under it instead of the OS default. This is
    /// the only reliable way to relocate the paths on Windows: `dirs::data_dir`
    /// there resolves via the Win32 known-folder API and ignores `APPDATA` /
    /// `LOCALAPPDATA` env vars, so tests can't sandbox it by setting those.
    const BASE_DIR_ENV: &'static str = "SQL_CLI_DATA_DIR";

    fn base_override() -> Option<PathBuf> {
        std::env::var_os(Self::BASE_DIR_ENV)
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
    }

    pub fn data_dir() -> Result<PathBuf, Box<dyn Error>> {
        let base = match Self::base_override() {
            Some(base) => base,
            None => dirs::data_dir().ok_or("Cannot determine data directory")?,
        };
        let data_dir = base.join("sql-cli");

        fs::create_dir_all(&data_dir)?;
        Ok(data_dir)
    }

    pub fn cache_dir() -> Result<PathBuf, Box<dyn Error>> {
        let base = match Self::base_override() {
            Some(base) => base,
            None => dirs::cache_dir().ok_or("Cannot determine cache directory")?,
        };
        let cache_dir = base.join("sql-cli");

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

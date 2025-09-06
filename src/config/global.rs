use super::config::{BehaviorConfig, Config};
use once_cell::sync::OnceCell;
use std::sync::RwLock;

// Global config instance
static CONFIG: OnceCell<RwLock<Config>> = OnceCell::new();

/// Initialize the global config. Should be called early in application startup.
pub fn init_config(config: Config) {
    CONFIG.set(RwLock::new(config)).ok();
}

/// Get the date notation preference from the global config.
/// Returns "us" if config is not initialized or on error.
pub fn get_date_notation() -> String {
    CONFIG.get().and_then(|c| c.read().ok()).map_or_else(
        || "us".to_string(),
        |c| c.behavior.default_date_notation.clone(),
    )
}

/// Get a copy of the behavior config
pub fn get_behavior_config() -> BehaviorConfig {
    CONFIG
        .get()
        .and_then(|c| c.read().ok())
        .map(|c| c.behavior.clone())
        .unwrap_or_default()
}

/// Update the global config
pub fn update_config(config: Config) {
    if let Some(global) = CONFIG.get() {
        if let Ok(mut c) = global.write() {
            *c = config;
        }
    } else {
        init_config(config);
    }
}

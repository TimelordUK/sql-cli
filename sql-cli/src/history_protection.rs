use chrono::Utc;
use std::fs;
use std::path::PathBuf;
use tracing::{error, info, warn};

/// Protection layer for history to prevent data loss
pub struct HistoryProtection {
    backup_dir: PathBuf,
    min_entries_threshold: usize,
}

impl HistoryProtection {
    pub fn new(backup_dir: PathBuf) -> Self {
        // Create backup directory if it doesn't exist
        if !backup_dir.exists() {
            let _ = fs::create_dir_all(&backup_dir);
        }

        Self {
            backup_dir,
            min_entries_threshold: 5, // Never allow history to shrink below 5 entries
        }
    }

    /// Create a backup of current history before any write operation
    pub fn backup_before_write(&self, current_data: &str, entry_count: usize) {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let backup_file = self.backup_dir.join(format!(
            "history_backup_{}_entries_{}.json",
            timestamp, entry_count
        ));

        if let Err(e) = fs::write(&backup_file, current_data) {
            error!("Failed to create history backup: {}", e);
        } else {
            info!(
                "Created history backup: {:?} with {} entries",
                backup_file, entry_count
            );
        }

        // Keep only last 10 backups
        self.cleanup_old_backups();
    }

    /// Validate that new history data is safe to write
    pub fn validate_write(&self, old_entries: usize, new_entries: usize, new_data: &str) -> bool {
        // Rule 1: Never write empty history
        if new_entries == 0 && old_entries > 0 {
            error!(
                "BLOCKED: Attempted to write empty history (had {} entries)",
                old_entries
            );
            return false;
        }

        // Rule 2: Never shrink by more than 50% (unless deduplication)
        if new_entries < old_entries / 2 && old_entries > self.min_entries_threshold {
            error!(
                "BLOCKED: History would shrink too much ({} -> {})",
                old_entries, new_entries
            );
            return false;
        }

        // Rule 3: Never write if data is suspiciously small
        if new_data.len() < 50 && old_entries > 0 {
            error!("BLOCKED: History data too small ({} bytes)", new_data.len());
            return false;
        }

        // Rule 4: Warn if significant reduction
        if new_entries < old_entries && (old_entries - new_entries) > 5 {
            warn!(
                "History reduction detected: {} -> {} entries",
                old_entries, new_entries
            );
        }

        true
    }

    /// Clean up old backups, keeping only the most recent ones
    fn cleanup_old_backups(&self) {
        if let Ok(entries) = fs::read_dir(&self.backup_dir) {
            let mut backups: Vec<_> = entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.file_name()
                        .to_string_lossy()
                        .starts_with("history_backup_")
                })
                .collect();

            // Sort by modification time
            backups.sort_by_key(|e| e.metadata().ok().and_then(|m| m.modified().ok()));

            // Remove all but the last 10
            if backups.len() > 10 {
                for backup in backups.iter().take(backups.len() - 10) {
                    if let Err(e) = fs::remove_file(backup.path()) {
                        warn!("Failed to remove old backup: {}", e);
                    }
                }
            }
        }
    }

    /// Attempt to recover from the most recent backup
    pub fn recover_from_backup(&self) -> Option<String> {
        if let Ok(entries) = fs::read_dir(&self.backup_dir) {
            let mut backups: Vec<_> = entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.file_name()
                        .to_string_lossy()
                        .starts_with("history_backup_")
                })
                .collect();

            // Sort by modification time (newest last)
            backups.sort_by_key(|e| e.metadata().ok().and_then(|m| m.modified().ok()));

            // Try to read the most recent backup
            if let Some(latest) = backups.last() {
                match fs::read_to_string(latest.path()) {
                    Ok(content) => {
                        info!("Recovered history from backup: {:?}", latest.path());
                        return Some(content);
                    }
                    Err(e) => {
                        error!("Failed to read backup: {}", e);
                    }
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_never_write_empty() {
        let temp_dir = TempDir::new().unwrap();
        let protection = HistoryProtection::new(temp_dir.path().to_path_buf());

        // Should block empty write when we had entries
        assert!(!protection.validate_write(10, 0, ""));

        // Should allow if we never had entries
        assert!(protection.validate_write(0, 0, ""));
    }

    #[test]
    fn test_prevent_massive_shrink() {
        let temp_dir = TempDir::new().unwrap();
        let protection = HistoryProtection::new(temp_dir.path().to_path_buf());

        // Should block if shrinking by more than 50%
        assert!(!protection.validate_write(100, 40, "some data"));

        // Should allow reasonable shrink (deduplication)
        // Need at least 50 chars of data
        let valid_data = r#"[{"command": "SELECT * FROM users", "timestamp": "2025-01-01"}]"#;
        assert!(protection.validate_write(100, 80, valid_data));
    }
}

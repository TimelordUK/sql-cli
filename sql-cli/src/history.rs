use anyhow::Result;
use chrono::{DateTime, Utc};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub command: String,
    pub timestamp: DateTime<Utc>,
    pub execution_count: u32,
    pub success: bool,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct HistoryMatch {
    pub entry: HistoryEntry,
    pub score: i64,
    pub indices: Vec<usize>,
}

pub struct CommandHistory {
    entries: Vec<HistoryEntry>,
    history_file: PathBuf,
    matcher: SkimMatcherV2,
    command_counts: HashMap<String, u32>,
}

impl CommandHistory {
    pub fn new() -> Result<Self> {
        let history_file = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".sql_cli_history.json");

        let mut history = Self {
            entries: Vec::new(),
            history_file,
            matcher: SkimMatcherV2::default(),
            command_counts: HashMap::new(),
        };

        history.load_from_file()?;
        Ok(history)
    }

    pub fn add_entry(&mut self, command: String, success: bool, duration_ms: Option<u64>) -> Result<()> {
        // Don't add empty commands or duplicates of the last command
        if command.trim().is_empty() {
            return Ok(());
        }

        // Check if this is the same as the last command
        if let Some(last_entry) = self.entries.last() {
            if last_entry.command == command {
                return Ok(());
            }
        }

        let entry = HistoryEntry {
            command: command.clone(),
            timestamp: Utc::now(),
            execution_count: *self.command_counts.get(&command).unwrap_or(&0) + 1,
            success,
            duration_ms,
        };

        // Update command count
        *self.command_counts.entry(command).or_insert(0) += 1;

        self.entries.push(entry);

        // Keep only the last 1000 entries
        if self.entries.len() > 1000 {
            self.entries.drain(0..self.entries.len() - 1000);
        }

        self.save_to_file()?;
        Ok(())
    }

    pub fn search(&self, query: &str) -> Vec<HistoryMatch> {
        if query.is_empty() {
            // Return recent entries when no query
            return self.entries
                .iter()
                .rev()
                .take(50)
                .map(|entry| HistoryMatch {
                    entry: entry.clone(),
                    score: 100,
                    indices: Vec::new(),
                })
                .collect();
        }

        let mut matches: Vec<HistoryMatch> = self.entries
            .iter()
            .filter_map(|entry| {
                if let Some((score, indices)) = self.matcher.fuzzy_indices(&entry.command, query) {
                    Some(HistoryMatch {
                        entry: entry.clone(),
                        score,
                        indices,
                    })
                } else {
                    None
                }
            })
            .collect();

        // Sort by score (descending), then by recency and frequency
        matches.sort_by(|a, b| {
            // Primary sort: fuzzy match score
            let score_cmp = b.score.cmp(&a.score);
            if score_cmp != std::cmp::Ordering::Equal {
                return score_cmp;
            }

            // Secondary sort: execution count (more frequently used commands rank higher)
            let count_cmp = b.entry.execution_count.cmp(&a.entry.execution_count);
            if count_cmp != std::cmp::Ordering::Equal {
                return count_cmp;
            }

            // Tertiary sort: recency (more recent commands rank higher)
            b.entry.timestamp.cmp(&a.entry.timestamp)
        });

        matches.truncate(20); // Limit to top 20 matches
        matches
    }

    pub fn get_recent(&self, limit: usize) -> Vec<&HistoryEntry> {
        self.entries
            .iter()
            .rev()
            .take(limit)
            .collect()
    }

    pub fn get_all(&self) -> &[HistoryEntry] {
        &self.entries
    }

    pub fn clear(&mut self) -> Result<()> {
        self.entries.clear();
        self.command_counts.clear();
        self.save_to_file()?;
        Ok(())
    }

    fn load_from_file(&mut self) -> Result<()> {
        if !self.history_file.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&self.history_file)?;
        if content.trim().is_empty() {
            return Ok(());
        }

        let entries: Vec<HistoryEntry> = serde_json::from_str(&content)?;
        
        // Rebuild command counts
        self.command_counts.clear();
        for entry in &entries {
            *self.command_counts.entry(entry.command.clone()).or_insert(0) += 1;
        }

        self.entries = entries;
        Ok(())
    }

    fn save_to_file(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.entries)?;
        fs::write(&self.history_file, content)?;
        Ok(())
    }

    pub fn stats(&self) -> HistoryStats {
        let total_commands = self.entries.len();
        let unique_commands = self.command_counts.len();
        let successful_commands = self.entries.iter().filter(|e| e.success).count();
        let failed_commands = total_commands - successful_commands;

        let most_used = self.command_counts
            .iter()
            .max_by_key(|(_, &count)| count)
            .map(|(cmd, &count)| (cmd.clone(), count));

        HistoryStats {
            total_commands,
            unique_commands,
            successful_commands,
            failed_commands,
            most_used_command: most_used,
        }
    }
}

#[derive(Debug)]
pub struct HistoryStats {
    pub total_commands: usize,
    pub unique_commands: usize,
    pub successful_commands: usize,
    pub failed_commands: usize,
    pub most_used_command: Option<(String, u32)>,
}

impl Clone for CommandHistory {
    fn clone(&self) -> Self {
        Self {
            entries: self.entries.clone(),
            history_file: self.history_file.clone(),
            matcher: SkimMatcherV2::default(), // Create new matcher
            command_counts: self.command_counts.clone(),
        }
    }
}

impl Default for CommandHistory {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            entries: Vec::new(),
            history_file: PathBuf::from(".sql_cli_history.json"),
            matcher: SkimMatcherV2::default(),
            command_counts: HashMap::new(),
        })
    }
}
use anyhow::Result;
use chrono::{DateTime, Utc};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryMetadata {
    #[serde(default)]
    pub tables: Vec<String>,           // Tables referenced (FROM clause)
    #[serde(default)]
    pub select_columns: Vec<String>,   // Columns in SELECT clause
    #[serde(default)]
    pub where_columns: Vec<String>,    // Columns in WHERE clause
    #[serde(default)]
    pub order_by_columns: Vec<String>, // Columns in ORDER BY clause
    #[serde(default)]
    pub functions_used: Vec<String>,   // Functions/methods used (Contains, StartsWith, etc.)
    #[serde(default)]
    pub query_type: String,            // SELECT, INSERT, UPDATE, DELETE, etc.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub command: String,
    pub timestamp: DateTime<Utc>,
    pub execution_count: u32,
    pub success: bool,
    pub duration_ms: Option<u64>,
    #[serde(default)]
    pub schema_columns: Vec<String>,  // Column names from the data source
    #[serde(default)]
    pub data_source: Option<String>,  // e.g., "customers.csv", "trades_api", etc.
    #[serde(default)]
    pub metadata: Option<QueryMetadata>, // Parsed query metadata
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

    pub fn add_entry(
        &mut self,
        command: String,
        success: bool,
        duration_ms: Option<u64>,
    ) -> Result<()> {
        self.add_entry_with_schema(command, success, duration_ms, Vec::new(), None)
    }

    pub fn add_entry_with_schema(
        &mut self,
        command: String,
        success: bool,
        duration_ms: Option<u64>,
        schema_columns: Vec<String>,
        data_source: Option<String>,
    ) -> Result<()> {
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

        // Extract metadata from the query
        let metadata = self.extract_query_metadata(&command);

        let entry = HistoryEntry {
            command: command.clone(),
            timestamp: Utc::now(),
            execution_count: *self.command_counts.get(&command).unwrap_or(&0) + 1,
            success,
            duration_ms,
            schema_columns,
            data_source,
            metadata,
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
        self.search_with_schema(query, &[], None)
    }

    pub fn search_with_schema(
        &self,
        query: &str,
        current_columns: &[String],
        current_source: Option<&str>,
    ) -> Vec<HistoryMatch> {
        if query.is_empty() {
            // Return recent entries when no query, prioritizing schema matches
            let mut entries: Vec<_> = self
                .entries
                .iter()
                .rev()
                .take(100)
                .map(|entry| {
                    let schema_score = self.calculate_schema_match_score(
                        entry,
                        current_columns,
                        current_source,
                    );
                    HistoryMatch {
                        entry: entry.clone(),
                        score: 100 + schema_score,
                        indices: Vec::new(),
                    }
                })
                .collect();
            
            entries.sort_by(|a, b| b.score.cmp(&a.score));
            entries.truncate(50);
            return entries;
        }

        let mut matches: Vec<HistoryMatch> = self
            .entries
            .iter()
            .filter_map(|entry| {
                if let Some((score, indices)) = self.matcher.fuzzy_indices(&entry.command, query) {
                    let schema_score = self.calculate_schema_match_score(
                        entry,
                        current_columns,
                        current_source,
                    );
                    Some(HistoryMatch {
                        entry: entry.clone(),
                        score: score + schema_score,
                        indices,
                    })
                } else {
                    None
                }
            })
            .collect();

        // Sort by score (descending), then by recency and frequency
        matches.sort_by(|a, b| {
            // Primary sort: fuzzy match score (including schema bonus)
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

    fn calculate_schema_match_score(
        &self,
        entry: &HistoryEntry,
        current_columns: &[String],
        current_source: Option<&str>,
    ) -> i64 {
        let mut score = 0i64;

        // Bonus for matching data source
        if let (Some(entry_source), Some(current)) = (&entry.data_source, current_source) {
            if entry_source == current {
                score += 50; // High bonus for same data source
            }
        }

        // Bonus for matching columns in schema
        if !current_columns.is_empty() && !entry.schema_columns.is_empty() {
            let matching_columns = entry
                .schema_columns
                .iter()
                .filter(|col| current_columns.contains(col))
                .count();
            
            let total_columns = entry.schema_columns.len().max(current_columns.len());
            if total_columns > 0 {
                // Scale bonus based on percentage of matching columns
                let match_percentage = (matching_columns * 100) / total_columns;
                score += (match_percentage as i64) / 2; // Up to 50 points for perfect match
            }
        }

        // Additional bonus for matching columns in query metadata
        if let Some(metadata) = &entry.metadata {
            let metadata_columns: Vec<&String> = metadata.select_columns.iter()
                .chain(metadata.where_columns.iter())
                .chain(metadata.order_by_columns.iter())
                .collect();
            
            let matching_metadata = metadata_columns
                .iter()
                .filter(|col| current_columns.contains(col))
                .count();
            
            if matching_metadata > 0 {
                score += (matching_metadata as i64) * 5; // 5 points per matching column
            }
        }

        score
    }

    fn extract_query_metadata(&self, query: &str) -> Option<QueryMetadata> {
        let query_upper = query.to_uppercase();
        
        // Determine query type
        let query_type = if query_upper.starts_with("SELECT") {
            "SELECT"
        } else if query_upper.starts_with("INSERT") {
            "INSERT"
        } else if query_upper.starts_with("UPDATE") {
            "UPDATE"
        } else if query_upper.starts_with("DELETE") {
            "DELETE"
        } else {
            "OTHER"
        }.to_string();

        // Extract table names (simple regex-based approach)
        let mut tables = Vec::new();
        if let Some(from_idx) = query_upper.find(" FROM ") {
            let after_from = &query[from_idx + 6..];
            if let Some(end_idx) = after_from.find(|c: char| c == ' ' || c == '(' || c == ';') {
                let table_name = after_from[..end_idx].trim().to_string();
                if !table_name.is_empty() {
                    tables.push(table_name);
                }
            }
        }

        // Extract columns from SELECT clause
        let mut select_columns = Vec::new();
        if query_type == "SELECT" {
            if let Some(select_idx) = query_upper.find("SELECT ") {
                let after_select = &query[select_idx + 7..];
                if let Some(from_idx) = after_select.to_uppercase().find(" FROM") {
                    let select_clause = &after_select[..from_idx];
                    if !select_clause.trim().eq("*") {
                        // Parse column names (simplified)
                        for col in select_clause.split(',') {
                            let col_name = col.trim()
                                .split_whitespace()
                                .next()
                                .unwrap_or("")
                                .trim_matches('"')
                                .to_string();
                            if !col_name.is_empty() {
                                select_columns.push(col_name);
                            }
                        }
                    }
                }
            }
        }

        // Extract columns from WHERE clause and functions used
        let mut where_columns = Vec::new();
        let mut functions_used = Vec::new();
        if let Some(where_idx) = query_upper.find(" WHERE ") {
            let after_where = &query[where_idx + 7..];
            
            // Look for LINQ methods
            let linq_methods = ["Contains", "StartsWith", "EndsWith", "Length", 
                                "ToUpper", "ToLower", "IsNullOrEmpty"];
            for method in &linq_methods {
                if after_where.contains(method) {
                    functions_used.push(method.to_string());
                }
            }
            
            // Extract column names before operators or methods
            // This is simplified - a proper parser would be better
            let words: Vec<&str> = after_where.split(|c: char| !c.is_alphanumeric() && c != '_')
                .filter(|s| !s.is_empty())
                .collect();
            
            for (i, word) in words.iter().enumerate() {
                // If next word is an operator or method, this might be a column
                if i + 1 < words.len() {
                    let next = words[i + 1];
                    if linq_methods.contains(&next) || 
                       ["IS", "NOT", "LIKE", "BETWEEN"].contains(&next.to_uppercase().as_str()) {
                        where_columns.push(word.to_string());
                    }
                }
            }
        }

        // Extract ORDER BY columns
        let mut order_by_columns = Vec::new();
        if let Some(order_idx) = query_upper.find(" ORDER BY ") {
            let after_order = &query[order_idx + 10..];
            let end_idx = after_order.find(|c: char| c == ';' || c == ')')
                .unwrap_or(after_order.len());
            let order_clause = &after_order[..end_idx];
            
            for col in order_clause.split(',') {
                let col_name = col.trim()
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .trim_matches('"')
                    .to_string();
                if !col_name.is_empty() && col_name.to_uppercase() != "ASC" && col_name.to_uppercase() != "DESC" {
                    order_by_columns.push(col_name);
                }
            }
        }

        Some(QueryMetadata {
            tables,
            select_columns,
            where_columns,
            order_by_columns,
            functions_used,
            query_type,
        })
    }

    pub fn get_recent(&self, limit: usize) -> Vec<&HistoryEntry> {
        self.entries.iter().rev().take(limit).collect()
    }

    pub fn get_all(&self) -> &[HistoryEntry] {
        &self.entries
    }

    pub fn get_last_entry(&self) -> Option<&HistoryEntry> {
        self.entries.last()
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
            *self
                .command_counts
                .entry(entry.command.clone())
                .or_insert(0) += 1;
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

        let most_used = self
            .command_counts
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

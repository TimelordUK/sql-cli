use crate::debug::debug_trace::{DebugSection, DebugSectionBuilder, DebugTrace, Priority};
use std::sync::Arc;
use std::sync::RwLock;

/// Tracks memory usage over time
#[derive(Clone)]
pub struct MemoryTracker {
    history: Arc<RwLock<Vec<MemorySnapshot>>>,
    max_history: usize,
}

#[derive(Clone, Debug)]
struct MemorySnapshot {
    timestamp: std::time::Instant,
    memory_kb: usize,
}

impl MemoryTracker {
    pub fn new(max_history: usize) -> Self {
        Self {
            history: Arc::new(RwLock::new(Vec::new())),
            max_history,
        }
    }

    pub fn record_snapshot(&self) {
        if let Some(memory_kb) = crate::utils::memory_tracker::get_process_memory_kb() {
            let snapshot = MemorySnapshot {
                timestamp: std::time::Instant::now(),
                memory_kb,
            };

            if let Ok(mut history) = self.history.write() {
                history.push(snapshot);
                // Keep only the last max_history entries
                if history.len() > self.max_history {
                    let drain_count = history.len() - self.max_history;
                    history.drain(0..drain_count);
                }
            }
        }
    }

    pub fn get_current_memory_mb(&self) -> Option<f64> {
        crate::utils::memory_tracker::get_process_memory_kb().map(|kb| kb as f64 / 1024.0)
    }

    pub fn get_history(&self) -> Vec<(usize, f64)> {
        if let Ok(history) = self.history.read() {
            history
                .iter()
                .enumerate()
                .map(|(idx, snapshot)| (idx, snapshot.memory_kb as f64 / 1024.0))
                .collect()
        } else {
            Vec::new()
        }
    }
}

impl Default for MemoryTracker {
    fn default() -> Self {
        Self::new(100)
    }
}

/// Debug trace implementation for memory tracking
pub struct MemoryDebugProvider {
    tracker: MemoryTracker,
}

impl MemoryDebugProvider {
    pub fn new(tracker: MemoryTracker) -> Self {
        Self { tracker }
    }
}

impl DebugTrace for MemoryDebugProvider {
    fn name(&self) -> &str {
        "Memory"
    }

    fn debug_sections(&self) -> Vec<DebugSection> {
        let mut builder = DebugSectionBuilder::new();

        builder.add_section("MEMORY USAGE", "", Priority::MEMORY);

        // Current memory usage
        if let Some(memory_mb) = self.tracker.get_current_memory_mb() {
            builder.add_field("Current Memory", format!("{:.2} MB", memory_mb));
        } else {
            builder.add_field("Current Memory", "Unable to read");
        }

        // Memory history
        let history = self.tracker.get_history();
        if !history.is_empty() {
            builder.add_line("");
            builder.add_line("Memory History (last readings):");

            // Calculate statistics
            let values: Vec<f64> = history.iter().map(|(_, mb)| *mb).collect();
            let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            let avg = values.iter().sum::<f64>() / values.len() as f64;

            builder.add_field("  Min", format!("{:.2} MB", min));
            builder.add_field("  Max", format!("{:.2} MB", max));
            builder.add_field("  Avg", format!("{:.2} MB", avg));

            // Show last few readings
            builder.add_line("");
            builder.add_line("  Recent readings:");
            for (_, mb) in history.iter().rev().take(5) {
                builder.add_line(format!("    {:.2} MB", mb));
            }

            // Memory growth
            if history.len() >= 2 {
                let first = history.first().map(|(_, mb)| *mb).unwrap_or(0.0);
                let last = history.last().map(|(_, mb)| *mb).unwrap_or(0.0);
                let growth = last - first;
                let growth_pct = if first > 0.0 {
                    (growth / first) * 100.0
                } else {
                    0.0
                };

                builder.add_line("");
                builder.add_field(
                    "  Growth",
                    format!("{:+.2} MB ({:+.1}%)", growth, growth_pct),
                );
            }
        } else {
            builder.add_line("No memory history available");
        }

        // System memory info (if available)
        #[cfg(target_os = "linux")]
        {
            if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
                let mut total_mb = 0.0;
                let mut available_mb = 0.0;

                for line in meminfo.lines() {
                    if line.starts_with("MemTotal:") {
                        if let Some(kb) = line.split_whitespace().nth(1) {
                            if let Ok(kb_val) = kb.parse::<f64>() {
                                total_mb = kb_val / 1024.0;
                            }
                        }
                    } else if line.starts_with("MemAvailable:") {
                        if let Some(kb) = line.split_whitespace().nth(1) {
                            if let Ok(kb_val) = kb.parse::<f64>() {
                                available_mb = kb_val / 1024.0;
                            }
                        }
                    }
                }

                if total_mb > 0.0 {
                    builder.add_line("");
                    builder.add_line("System Memory:");
                    builder.add_field("  Total", format!("{:.2} MB", total_mb));
                    builder.add_field("  Available", format!("{:.2} MB", available_mb));
                    let used_pct = ((total_mb - available_mb) / total_mb) * 100.0;
                    builder.add_field("  Used", format!("{:.1}%", used_pct));
                }
            }
        }

        builder.build()
    }

    fn debug_summary(&self) -> Option<String> {
        self.tracker
            .get_current_memory_mb()
            .map(|mb| format!("{:.2} MB", mb))
    }
}

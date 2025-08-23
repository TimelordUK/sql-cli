/// Get current process memory usage in MB
pub fn get_memory_mb() -> usize {
    // Use /proc/self/status on Linux
    #[cfg(target_os = "linux")]
    {
        if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("VmRSS:") {
                    if let Some(kb_str) = line.split_whitespace().nth(1) {
                        if let Ok(kb) = kb_str.parse::<usize>() {
                            return kb / 1024;
                        }
                    }
                }
            }
        }
    }

    // Fallback or other platforms
    0
}

/// Get current process memory usage in KB (cross-platform)
pub fn get_process_memory_kb() -> Option<usize> {
    #[cfg(target_os = "linux")]
    {
        std::fs::read_to_string("/proc/self/status")
            .ok()?
            .lines()
            .find(|line| line.starts_with("VmRSS:"))
            .and_then(|line| {
                line.split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse::<usize>().ok())
            })
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        if let Ok(output) = Command::new("ps")
            .args(&["-o", "rss=", "-p", &std::process::id().to_string()])
            .output()
        {
            if let Ok(s) = String::from_utf8(output.stdout) {
                if let Ok(kb) = s.trim().parse::<usize>() {
                    return Some(kb);
                }
            }
        }
        None
    }

    #[cfg(target_os = "windows")]
    {
        // Windows implementation would go here
        // For now, return None
        None
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        None
    }
}

// Use thread-local storage instead of global static
thread_local! {
    static MEMORY_LOG: std::cell::RefCell<Vec<(String, usize)>> = std::cell::RefCell::new(Vec::new());
}

/// Track memory at a specific point
pub fn track_memory(label: &str) -> usize {
    let mb = get_memory_mb();

    MEMORY_LOG.with(|log| {
        let mut log = log.borrow_mut();

        // Calculate delta from last entry
        let delta = if let Some((_, last_mb)) = log.last() {
            let diff = (mb as i32) - (*last_mb as i32);
            if diff != 0 {
                format!(" ({:+} MB)", diff)
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        log.push((label.to_string(), mb));

        // Keep last 30 entries
        if log.len() > 30 {
            log.remove(0);
        }

        tracing::info!("MEMORY[{}]: {} MB{}", label, mb, delta);
    });

    mb
}

/// Get memory history for display
pub fn get_memory_history() -> Vec<(String, usize)> {
    MEMORY_LOG.with(|log| log.borrow().clone())
}

/// Format memory history as a string
pub fn format_memory_history() -> String {
    MEMORY_LOG.with(|log| {
        let log = log.borrow();
        let mut output = String::from("Memory History:\n");

        for (i, (label, mb)) in log.iter().enumerate() {
            let delta = if i > 0 {
                let prev_mb = log[i - 1].1;
                let diff = (*mb as i32) - (prev_mb as i32);
                if diff != 0 {
                    format!(" ({:+} MB)", diff)
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            output.push_str(&format!("  {}: {} MB{}\n", label, mb, delta));
        }

        output
    })
}

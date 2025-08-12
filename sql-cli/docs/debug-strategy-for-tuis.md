# Debug Strategy for TUIs

## The Challenge

Debugging Terminal User Interfaces (TUIs) presents unique challenges:
- Traditional debuggers can interfere with terminal control
- Breakpoints in render loops can cause chaos
- Raw terminal mode conflicts with debugger I/O
- WSL2/remote debugging adds path mapping complexity

## LLDB Configuration

### Setting up `.lldbinit`

LLDB supports initialization files to configure debugging sessions automatically.

**File Locations:**
- Global: `~/.lldbinit` (applies to all LLDB sessions)
- Project-local: `.lldbinit` in project root (requires permission)

**Example `.lldbinit` for TUI debugging:**

```bash
# ~/.lldbinit or project/.lldbinit

# Disable stopping on signals that TUIs commonly trigger
process handle SIGTTOU --stop false --pass true --notify false
process handle SIGTTIN --stop false --pass true --notify false
process handle SIGWINCH --stop false --pass true --notify false
process handle SIGPIPE --stop false --pass true --notify false

# Better formatting
settings set target.max-string-summary-length 1000
settings set target.max-memory-read-size 0x1000

# Auto-confirm potentially dangerous operations
settings set auto-confirm true

# Better source mapping for WSL2
settings set target.source-map /home/me/dev /mnt/c/Users/YourName/dev

# Load Rust pretty printers
command script import ~/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/etc/lldb_rust_formatters.py
type summary add --no-value --python-function lldb_rust_formatters.print_val -x ".*" --category Rust
type category enable Rust
```

To allow local `.lldbinit` files:
```bash
# Add to ~/.lldbinit
settings set target.load-cwd-lldbinit true
```

### Fixing Phantom Breakpoints

If LLDB pauses without visible breakpoints:

```bash
# In LLDB directly
(lldb) breakpoint delete --force
(lldb) watchpoint delete --force  
(lldb) settings clear target.process.stop-on-sharedlibrary-events
```

## Build Configuration

Optimize debug builds for better debugging experience:

```toml
# Cargo.toml
[profile.dev]
debug = 2
split-debuginfo = "off"  # Important for WSL2
```

## Dual Logging Strategy

Implement both ring buffer (for in-app display) and persistent file logging:

```rust
use std::sync::Mutex;
use std::fs::OpenOptions;
use std::io::Write;
use std::collections::VecDeque;

lazy_static! {
    // Ring buffer for F5 display (last 200 lines)
    static ref RING_BUFFER: Mutex<VecDeque<String>> = Mutex::new(VecDeque::with_capacity(200));
    
    // Physical log file for persistent history
    static ref LOG_FILE: Mutex<Option<File>> = Mutex::new(
        OpenOptions::new()
            .create(true)
            .append(true)
            .open("/tmp/sql-cli-debug.log")
            .ok()
    );
}

macro_rules! debug_log {
    ($($arg:tt)*) => {{
        let msg = format!("[{}] {}", chrono::Local::now().format("%H:%M:%S%.3f"), format!($($arg)*));
        
        // To ring buffer for in-app display
        if let Ok(mut buffer) = RING_BUFFER.lock() {
            if buffer.len() >= 200 {
                buffer.pop_front();
            }
            buffer.push_back(msg.clone());
        }
        
        // To file for persistent history
        if let Ok(mut file_opt) = LOG_FILE.lock() {
            if let Some(ref mut file) = *file_opt {
                let _ = writeln!(file, "{}", msg);
                let _ = file.flush(); // Important for debugging crashes!
            }
        }
        
        // Also to stderr if DEBUG env var set
        if std::env::var("DEBUG").is_ok() {
            eprintln!("{}", msg);
        }
    }};
}
```

### Benefits:
- **F5 key**: Shows last 200 lines in-app (ring buffer)
- **Log file**: Complete history preserved
- **Real-time monitoring**: `tail -f /tmp/sql-cli-debug.log`
- **Searchable**: `grep` patterns in the log file
- **No data loss**: Critical events preserved even if ring buffer overwrites

## Advanced: Using `tracing` Crate

For production-grade logging with multiple outputs:

```rust
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

tracing_subscriber::registry()
    .with(console_subscriber::spawn())        // For tokio-console
    .with(tracing_subscriber::fmt::layer())   // Console output
    .with(tracing_appender::rolling::daily("/tmp", "sql-cli.log")) // Rolling file output
    .init();
```

## Debug Mode for TUIs

Add a debug mode that disables raw terminal mode:

```rust
// Check for debug environment variable
if std::env::var("DEBUG_MODE").is_ok() {
    // Skip raw mode for easier debugging
    println!("Debug mode: Terminal raw mode disabled");
} else {
    terminal::enable_raw_mode()?;
}
```

## Alternative Debugging Tools

### 1. `dbg!()` Macro
Often more reliable than breakpoints for TUIs:
```rust
dbg!(&self.state_container);  // Prints value and returns it
```

### 2. CodeLLDB (VS Code)
Better WSL2 integration than some IDEs:
```json
{
    "type": "lldb",
    "request": "launch",
    "sourceLanguages": ["rust"],
    "preLaunchTask": "cargo build"
}
```

### 3. rr Debugger
Record and replay debugging sessions:
```bash
rr record ./target/debug/sql-cli
rr replay  # Debug the recording deterministically
```

### 4. GDB as Backup
Sometimes more stable than LLDB for certain scenarios:
```bash
rust-gdb ./target/debug/sql-cli
```

## Tips for TUI Debugging

1. **Use structured logging**: Better than println! debugging
2. **Create debug views**: Like the F5 state dump
3. **Unit test complex logic**: Test outside the TUI context
4. **Use debug assertions**: `debug_assert!()` for invariants
5. **Separate concerns**: Keep UI and logic separate for easier testing

## Common Issues and Solutions

| Issue | Solution |
|-------|----------|
| Phantom breakpoints | Clear LLDB state, rebuild with clean |
| Terminal corruption | Use debug mode without raw terminal |
| Lost debug output | Use dual logging (ring buffer + file) |
| WSL2 path issues | Configure source mapping in `.lldbinit` |
| Signal interference | Disable signal handling in `.lldbinit` |

## Monitoring Commands

```bash
# Watch log file in real-time
tail -f /tmp/sql-cli-debug.log

# Search for specific patterns
grep "ERROR" /tmp/sql-cli-debug.log

# Monitor with highlighting
tail -f /tmp/sql-cli-debug.log | grep --color=always -E "ERROR|WARNING|"

# Count occurrences
grep -c "pattern" /tmp/sql-cli-debug.log
```
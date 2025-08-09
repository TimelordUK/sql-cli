# Debugging TUI App in RustRover

Since ratatui apps need a real terminal (not RustRover's console), here are several approaches:

## Method 1: External Terminal + Attach Debugger (Recommended)

1. **Build debug binary:**
   ```bash
   cargo build --bin sql-cli
   ```

2. **In RustRover:**
   - Set your breakpoints in the code
   - Go to Run → Edit Configurations
   - Add new "Cargo Command" configuration
   - Set command: `build --bin sql-cli`
   - Add environment variable: `RUST_LOG=debug`

3. **Run with debugger in external terminal:**
   ```bash
   # Option A: Using lldb
   rust-lldb target/debug/sql-cli
   
   # Option B: Using gdb  
   rust-gdb target/debug/sql-cli
   ```

4. **In debugger console:**
   ```
   # Set breakpoints (example)
   b enhanced_tui.rs:100
   
   # Run the program
   run
   
   # Continue after breakpoint
   c
   ```

## Method 2: Remote Debugging with gdbserver

1. **Terminal 1 - Run gdbserver:**
   ```bash
   cargo build --bin sql-cli
   gdbserver :9999 target/debug/sql-cli
   ```

2. **In RustRover:**
   - Create "Remote Debug" configuration
   - Set host: `localhost`
   - Set port: `9999`
   - Click Debug button

## Method 3: Two-Terminal Approach

1. **Terminal 1 - Run the TUI:**
   ```bash
   cargo run --bin sql-cli
   ```

2. **Terminal 2 - Attach debugger:**
   ```bash
   # Find the process ID
   ps aux | grep sql-cli
   
   # Attach with lldb
   sudo lldb -p <PID>
   
   # Or with gdb
   sudo gdb -p <PID>
   ```

## Method 4: Logging-Based Debugging

1. **Add debug helpers to your code:**
   ```rust
   // At the top of enhanced_tui.rs
   mod debug_helpers;
   use debug_helpers::{init_debug_log, debug_breakpoint};
   
   // In your function
   debug_log!("Current state: {:?}", state);
   debug_breakpoint("before_render");
   ```

2. **Run with logging:**
   ```bash
   RUST_LOG=trace cargo run --bin sql-cli 2>&1 | tee debug.log
   ```

3. **Watch logs in real-time (separate terminal):**
   ```bash
   tail -f tui_debug.log
   ```

## Method 5: Conditional TUI Mode

Add a debug flag to run without full TUI:

```rust
// In main.rs or enhanced_tui.rs
if std::env::var("DEBUG_MODE").is_ok() {
    // Run simplified version without TUI
    debug_run()?;
} else {
    // Normal TUI mode
    run_enhanced_tui()?;
}
```

Then debug normally in RustRover with:
```
DEBUG_MODE=1 cargo run --bin sql-cli
```

## Quick Debug Script

Use the provided `debug_tui.sh` script:
```bash
./debug_tui.sh
```

This interactive script guides you through different debugging options.

## RustRover Configuration Tips

1. **For breakpoints to work:**
   - Ensure "Debug" build configuration
   - Set optimization level to 0 in Cargo.toml (already done)
   - Use `#[inline(never)]` on functions you want to debug

2. **Environment variables in RustRover:**
   - `RUST_LOG=debug` or `RUST_LOG=trace`
   - `RUST_BACKTRACE=1` or `RUST_BACKTRACE=full`

3. **Working directory:**
   - Set to project root: `$PROJECT_DIR$`

## Troubleshooting

- **TUI garbled in debugger:** Use external terminal, not IDE console
- **Breakpoints not hit:** Check optimization settings, use `#[inline(never)]`
- **Can't attach to process:** May need `sudo` for attach operations
- **Screen corruption:** The TUI and debugger output can conflict; use logging method for complex UI debugging

## Best Practice

For TUI debugging, combine methods:
1. Use logging for understanding flow
2. Use external terminal + lldb/gdb for breakpoint debugging
3. Keep a simplified non-TUI debug mode for logic testing
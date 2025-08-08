#!/bin/bash

# Debug helper script for TUI app with RustRover
# This script helps you debug ratatui apps that need a real terminal

echo "TUI Debug Helper for RustRover"
echo "==============================="
echo ""
echo "Choose debugging method:"
echo "1) External Terminal with lldb"
echo "2) External Terminal with gdb" 
echo "3) Log-based debugging (no debugger)"
echo "4) Two-terminal approach (app in one, debugger in another)"
echo ""
read -p "Enter choice (1-4): " choice

case $choice in
    1)
        echo "Starting with lldb..."
        echo ""
        echo "Building debug binary..."
        cargo build --bin sql-cli
        echo ""
        echo "To debug:"
        echo "1. Set breakpoints in RustRover"
        echo "2. Run in terminal: rust-lldb target/debug/sql-cli"
        echo "3. In lldb: 'run' to start, 'c' to continue"
        echo ""
        echo "Common lldb commands:"
        echo "  b <file>:<line>  - Set breakpoint"
        echo "  run              - Start program"
        echo "  c                - Continue"
        echo "  n                - Next line"
        echo "  s                - Step into"
        echo "  p <var>          - Print variable"
        echo ""
        rust-lldb target/debug/sql-cli
        ;;
        
    2)
        echo "Starting with gdb..."
        echo ""
        echo "Building debug binary..."
        cargo build --bin sql-cli
        echo ""
        echo "To debug:"
        echo "1. Set breakpoints in RustRover"
        echo "2. Run in terminal: rust-gdb target/debug/sql-cli"
        echo "3. In gdb: 'run' to start, 'c' to continue"
        echo ""
        echo "Common gdb commands:"
        echo "  break <file>:<line>  - Set breakpoint"
        echo "  run                  - Start program"
        echo "  continue             - Continue"
        echo "  next                 - Next line"
        echo "  step                 - Step into"
        echo "  print <var>          - Print variable"
        echo ""
        rust-gdb target/debug/sql-cli
        ;;
        
    3)
        echo "Starting with logging..."
        echo ""
        echo "Building debug binary with max logging..."
        RUST_LOG=trace RUST_BACKTRACE=full cargo run --bin sql-cli 2>&1 | tee debug.log
        echo ""
        echo "Debug output saved to debug.log"
        ;;
        
    4)
        echo "Two-terminal debugging setup"
        echo ""
        echo "Terminal 1 (this terminal):"
        echo "  Starting debugger server..."
        echo ""
        echo "Terminal 2 (open a new terminal):"
        echo "  Run: cargo run --bin sql-cli"
        echo ""
        echo "Building debug binary..."
        cargo build --bin sql-cli
        echo ""
        echo "Starting gdbserver on port 9999..."
        echo "In another terminal, run:"
        echo "  gdbserver :9999 target/debug/sql-cli"
        echo ""
        echo "Then connect from RustRover's Remote Debug configuration"
        ;;
        
    *)
        echo "Invalid choice"
        exit 1
        ;;
esac
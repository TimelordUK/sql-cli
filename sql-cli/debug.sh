#!/bin/bash
set -e

echo "🔧 Building debug version..."
cargo build --bin sql-cli

echo "🐛 Debug build ready!"
echo ""
echo "Debug options:"
echo "1. VS Code: Open VS Code and use Run & Debug (F5)"
echo "2. LLDB: lldb ./target/debug/sql-cli"
echo "3. GDB: gdb ./target/debug/sql-cli"
echo "4. Rust-LLDB: rust-lldb ./target/debug/sql-cli"
echo ""
echo "For VS Code debugging:"
echo "  - Set breakpoints by clicking in the gutter"
echo "  - Press F5 to start debugging"
echo "  - Choose a debug configuration (API mode, CSV, etc.)"
echo "  - Use F10 for step over, F11 for step into"
echo ""
echo "Example breakpoint locations:"
echo "  - src/main.rs: main() function"
echo "  - src/enhanced_tui.rs: run() method"
echo "  - src/enhanced_tui.rs: handle_command_input() method"
echo "  - src/enhanced_tui.rs: handle_results_input() method"
echo ""
echo "Binary location: ./target/debug/sql-cli"
echo "Debug symbols: included ✓"
echo "Optimizations: disabled ✓"
#!/bin/bash
set -e

echo "🔧 Building debug version..."
cargo build --bin sql-cli

echo "🐛 Starting GDB debugging session..."
echo ""
echo "Useful GDB commands:"
echo "  (gdb) break main                    # Break at main function"
echo "  (gdb) break enhanced_tui.rs:1620    # Break at handle_command_input"
echo "  (gdb) run data/small-customer.csv  # Run with CSV file"
echo "  (gdb) run                          # Run in API mode"
echo "  (gdb) continue                     # Continue execution"
echo "  (gdb) step                         # Step into"
echo "  (gdb) next                         # Step over"
echo "  (gdb) print variable_name          # Print variable"
echo "  (gdb) backtrace                    # Show call stack"
echo "  (gdb) info locals                  # Show local variables"
echo "  (gdb) quit                         # Exit GDB"
echo ""

# Start GDB with the debug binary
exec gdb ./target/debug/sql-cli
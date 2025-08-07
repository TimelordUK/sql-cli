#!/bin/bash
# Auto-format and run script

# Change to sql-cli directory
cd "$(dirname "$0")/sql-cli" || exit 1

echo "📝 Running cargo fmt..."
cargo fmt

echo "🚀 Running project..."
cargo run "$@"
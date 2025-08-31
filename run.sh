#!/bin/bash
# Auto-format and run script

# Stay in root directory (no longer need to change to sql-cli subdirectory)
cd "$(dirname "$0")" || exit 1

echo "📝 Running cargo fmt..."
cargo fmt

echo "🚀 Running project..."
cargo run "$@"
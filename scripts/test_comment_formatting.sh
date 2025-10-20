#!/bin/bash
# Test script for comment preservation in SQL formatting
# Demonstrates the new --preserve-comments flag

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
SQL_CLI="$PROJECT_DIR/target/release/sql-cli"

echo "=== SQL Comment Preservation Demo ==="
echo ""

# Check if binary exists
if [ ! -f "$SQL_CLI" ]; then
    echo "Error: sql-cli binary not found at $SQL_CLI"
    echo "Run: cargo build --release"
    exit 1
fi

echo "Testing with examples/reformat_comments.sql"
echo ""

# Example 1: Simple query
echo "--- Example 1: Simple Query ---"
cat <<'EOF' | "$SQL_CLI" --format --preserve-comments
-- Simple Query with Leading Comments
-- This demonstrates basic comment preservation
SELECT id, name, email FROM users WHERE active = true
EOF
echo ""

# Example 2: Block comment
echo "--- Example 2: Block Comment ---"
cat <<'EOF' | "$SQL_CLI" --format --preserve-comments
/* Multi-line block comment
   This query retrieves user statistics
   Used for dashboard reporting */
SELECT user_id, name, email FROM users WHERE created_at >= '2025-01-01'
EOF
echo ""

# Example 3: Complex formatting
echo "--- Example 3: Complex Query (before/after) ---"
QUERY="-- High-value customers
SELECT u.id,u.name,COUNT(*) as order_count,SUM(o.amount) as total FROM users u WHERE u.active=true"

echo "BEFORE:"
echo "$QUERY"
echo ""
echo "AFTER:"
echo "$QUERY" | "$SQL_CLI" --format --preserve-comments
echo ""

# Example 4: Without preserve-comments (default)
echo "--- Example 4: Without --preserve-comments (comments stripped) ---"
cat <<'EOF' | "$SQL_CLI" --format
-- This comment will be removed
-- Because we're not using --preserve-comments
SELECT id, name FROM users
EOF
echo ""

# Example 5: Mixed comment styles
echo "--- Example 5: Mixed Comment Styles ---"
cat <<'EOF' | "$SQL_CLI" --format --preserve-comments
-- Line comment
/* Block comment */
SELECT id, status FROM transactions
EOF
echo ""

echo "=== All tests complete! ==="
echo ""
echo "Usage in Neovim:"
echo "  1. Open a SQL file"
echo "  2. Position cursor on query"
echo "  3. Press \\sf to format (comments preserved automatically!)"
echo ""
echo "Usage from CLI:"
echo "  cat query.sql | $SQL_CLI --format --preserve-comments"

#!/bin/bash
# Demo script for WHERE clause alias expansion feature
# Showcases the preprocessing pipeline's ability to expand SELECT aliases in WHERE

CLI="./target/release/sql-cli"
DATA_FILE="data/test_simple_math.csv"

echo "╔═══════════════════════════════════════════════════════════════════════╗"
echo "║         WHERE Clause Alias Expansion - Quick Win #2                  ║"
echo "╚═══════════════════════════════════════════════════════════════════════╝"
echo
echo "Problem:"
echo "  Users want to reference SELECT aliases in WHERE clauses, but SQL"
echo "  evaluates WHERE before SELECT, so aliases don't exist yet."
echo
echo "Solution:"
echo "  The WhereAliasExpander preprocessing transformer automatically expands"
echo "  alias references to their full expressions."
echo
echo "───────────────────────────────────────────────────────────────────────"
echo

# Example 1: Basic alias expansion
echo "Example 1: Use calculated column alias in WHERE"
echo
echo "  Instead of repeating the expression:"
echo "  $ SELECT a, a * 2 as double_a FROM t WHERE a * 2 > 10"
echo
echo "  You can now use the alias directly:"
echo "  $ SELECT a, a * 2 as double_a FROM t WHERE double_a > 10"
echo
echo "  Query:"
$CLI "$DATA_FILE" -q "SELECT a, b, a * 2 as double_a FROM test_simple_math WHERE double_a > 10 LIMIT 5" -o table
echo

# Example 2: Multiple aliases
echo "───────────────────────────────────────────────────────────────────────"
echo
echo "Example 2: Multiple aliases in complex WHERE conditions"
echo
echo "  Query:"
echo "  SELECT a, a * 2 as double_a, a * 3 as triple_a"
echo "  FROM test_simple_math"
echo "  WHERE double_a > 10 AND triple_a < 25"
echo
$CLI "$DATA_FILE" -q "SELECT a, a * 2 as double_a, a * 3 as triple_a FROM test_simple_math WHERE double_a > 10 AND triple_a < 25" -o table
echo

# Example 3: Complex expression
echo "───────────────────────────────────────────────────────────────────────"
echo
echo "Example 3: Division and comparison"
echo
echo "  Query:"
echo "  SELECT a, b, b / 10 as tens FROM test_simple_math WHERE tens > 15"
echo
$CLI "$DATA_FILE" -q "SELECT a, b, b / 10 as tens FROM test_simple_math WHERE tens > 15 LIMIT 5" -o table
echo

# Example 4: String functions
echo "───────────────────────────────────────────────────────────────────────"
echo
echo "Example 4: String function alias in WHERE"
echo
echo "  Query:"
echo "  SELECT name, UPPER(name) as upper_name"
echo "  FROM test_simple_strings"
echo "  WHERE upper_name = 'ALICE' OR upper_name = 'BOB'"
echo
$CLI "data/test_simple_strings.csv" -q "SELECT name, UPPER(name) as upper_name FROM test_simple_strings WHERE upper_name = 'ALICE' OR upper_name = 'BOB'" -o table
echo

# Show preprocessing pipeline
echo "───────────────────────────────────────────────────────────────────────"
echo
echo "How it works: Preprocessing Pipeline"
echo
echo "  The WhereAliasExpander transformer runs automatically as part of the"
echo "  preprocessing pipeline:"
echo
$CLI "$DATA_FILE" -q "SELECT a, a * 2 as double_a FROM test_simple_math WHERE double_a > 10 LIMIT 3" -o csv --show-preprocessing 2>&1 | head -10
echo

# Performance
echo "───────────────────────────────────────────────────────────────────────"
echo
echo "Performance Impact:"
echo "  • Transformation time: ~0.02ms"
echo "  • Zero runtime overhead - alias expanded at query planning time"
echo "  • Same execution performance as writing full expression"
echo

echo "╔═══════════════════════════════════════════════════════════════════════╗"
echo "║  Benefits:                                                            ║"
echo "║  • Write more readable queries with less repetition                   ║"
echo "║  • Easier to maintain - change expression once in SELECT              ║"
echo "║  • No performance penalty                                             ║"
echo "║  • Works automatically - no flags required                            ║"
echo "╚═══════════════════════════════════════════════════════════════════════╝"

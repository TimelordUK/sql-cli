#!/bin/bash
# Demo script for GROUP BY clause alias expansion feature
# Showcases the preprocessing pipeline's ability to expand SELECT aliases in GROUP BY

CLI="./target/release/sql-cli"
DATA_FILE="data/test_simple_math.csv"

echo "╔═══════════════════════════════════════════════════════════════════════╗"
echo "║         GROUP BY Alias Expansion - Quick Win #3                      ║"
echo "╚═══════════════════════════════════════════════════════════════════════╝"
echo
echo "Problem:"
echo "  Users want to reference SELECT aliases in GROUP BY, but SQL evaluates"
echo "  GROUP BY before SELECT, so aliases don't exist yet."
echo
echo "Solution:"
echo "  The GroupByAliasExpander preprocessing transformer automatically expands"
echo "  alias references to their full expressions."
echo
echo "───────────────────────────────────────────────────────────────────────"
echo

# Example 1: Basic modulo grouping
echo "Example 1: Group by calculated expression using alias"
echo
echo "  Instead of repeating the expression:"
echo "  $ SELECT value % 3, COUNT(*) FROM range(10) GROUP BY value % 3"
echo
echo "  You can now use the alias directly:"
echo "  $ SELECT value % 3 as grp, COUNT(*) FROM range(10) GROUP BY grp"
echo
echo "  Query:"
$CLI -q "SELECT value % 3 as grp, COUNT(*) as count FROM range(10) GROUP BY grp ORDER BY grp" -o table
echo

# Example 2: Complex expression with arithmetic
echo "───────────────────────────────────────────────────────────────────────"
echo
echo "Example 2: Arithmetic expression grouping"
echo
echo "  Query:"
echo "  SELECT b / 10 as tens, COUNT(*), AVG(a) as avg_a"
echo "  FROM test_simple_math"
echo "  GROUP BY tens"
echo "  ORDER BY tens"
echo
$CLI "$DATA_FILE" -q "SELECT b / 10 as tens, COUNT(*) as count, AVG(a) as avg_a FROM test_simple_math GROUP BY tens ORDER BY tens LIMIT 5" -o table
echo

# Example 3: GROUP BY with HAVING (multiple transformers working together)
echo "───────────────────────────────────────────────────────────────────────"
echo
echo "Example 3: GROUP BY alias with HAVING clause (transformers combine!)"
echo
echo "  Query:"
echo "  SELECT id % 3 as grp, COUNT(*), SUM(b) as total_b"
echo "  FROM test_simple_math"
echo "  GROUP BY grp"
echo "  HAVING COUNT(*) > 2 AND SUM(b) > 100"
echo
$CLI "$DATA_FILE" -q "SELECT id % 3 as grp, COUNT(*) as count, SUM(b) as total_b FROM test_simple_math GROUP BY grp HAVING count > 2 AND total_b > 100 ORDER BY grp" -o table
echo

# Example 4: Multiple aliases in GROUP BY
echo "───────────────────────────────────────────────────────────────────────"
echo
echo "Example 4: Multiple aliases in GROUP BY"
echo
echo "  Query:"
echo "  SELECT value % 2 as even_odd, value % 3 as mod3, COUNT(*)"
echo "  FROM range(12)"
echo "  GROUP BY even_odd, mod3"
echo "  ORDER BY even_odd, mod3"
echo
$CLI -q "SELECT value % 2 as even_odd, value % 3 as mod3, COUNT(*) as count FROM range(12) GROUP BY even_odd, mod3 ORDER BY even_odd, mod3" -o table
echo

# Example 5: All transformers working together
echo "───────────────────────────────────────────────────────────────────────"
echo
echo "Example 5: WHERE + GROUP BY + HAVING + ORDER BY (all transformers!)"
echo
echo "  Query:"
echo "  SELECT value % 3 as grp, COUNT(*) as cnt"
echo "  FROM range(20)"
echo "  WHERE value > 5        -- WhereAliasExpander could expand aliases here"
echo "  GROUP BY grp           -- GroupByAliasExpander expands 'grp'"
echo "  HAVING cnt > 3         -- HavingAliasTransformer handles aggregate alias"
echo "  ORDER BY cnt DESC      -- ORDER BY naturally works with aliases"
echo
$CLI -q "SELECT value % 3 as grp, COUNT(*) as cnt FROM range(20) WHERE value > 5 GROUP BY grp HAVING cnt > 3 ORDER BY cnt DESC" -o table
echo

# Show preprocessing pipeline
echo "───────────────────────────────────────────────────────────────────────"
echo
echo "How it works: Preprocessing Pipeline"
echo
echo "  The GroupByAliasExpander transformer runs automatically as part of"
echo "  the preprocessing pipeline:"
echo
$CLI -q "SELECT value % 3 as grp, COUNT(*) FROM range(10) GROUP BY grp" -o csv --show-preprocessing 2>&1 | head -12
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
echo "║  • Works seamlessly with HAVING and ORDER BY                          ║"
echo "║  • No performance penalty                                             ║"
echo "║  • Works automatically - no flags required                            ║"
echo "╚═══════════════════════════════════════════════════════════════════════╝"

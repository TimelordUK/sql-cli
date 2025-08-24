#!/bin/bash

# Automated test for fuzzy filter clear bug
echo "Testing fuzzy filter clear functionality..."

# Create input sequence:
# F - enter fuzzy filter
# rejected - type filter pattern
# Enter - apply filter
# F - re-enter fuzzy filter
# Enter - apply empty filter (should clear)
# q - quit

(
  sleep 1
  echo -n "F"        # Enter fuzzy filter
  sleep 0.5
  echo -n "rejected" # Type pattern
  sleep 0.5
  printf "\r"        # Press Enter to apply
  sleep 1
  echo -n "F"        # Re-enter fuzzy filter
  sleep 0.5
  printf "\r"        # Press Enter with empty pattern
  sleep 1
  echo -n "q"        # Quit
) | RUST_LOG=search=debug ./target/release/sql-cli test_fuzzy_demo.csv -e "select * from data" 2>&1 | grep -E "(Rows:|FuzzyFilter|Filtered Rows:|apply_fuzzy_filter)" | tail -20

echo ""
echo "Check the output above:"
echo "- Should see 'Rows: 10' initially"
echo "- After filtering 'rejected', should see 'Rows: 3'"
echo "- After empty filter, should see 'Rows: 10' again"
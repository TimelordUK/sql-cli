#!/bin/bash
# Simple test of json-structured output

echo "=== Testing json-structured output ==="
echo

./target/release/sql-cli data/solar_system.csv -q "SELECT name, type FROM solar_system LIMIT 3" -o json-structured

echo
echo "=== Test complete ==="

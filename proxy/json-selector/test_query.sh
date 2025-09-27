#!/bin/bash

# Test the message-type-aware query endpoint with our FIX messages
curl -X POST http://localhost:5050/api/messagequery/example/fix \
  -F "file=@/home/me/dev/sql-cli/proxy/json-selector/test-data/fix-messages.json" \
  -o result.csv

echo "Query result saved to result.csv"
echo "Contents:"
cat result.csv
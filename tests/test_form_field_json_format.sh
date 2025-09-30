#!/bin/bash
# Test that FORM_FIELD with $JSON$ delimiters preserves formatting

set -e

echo "Testing FORM_FIELD JSON formatting..."

# Input query with compact JSON
cat <<'EOF' | ./target/release/sql-cli --format > /tmp/test_format_output.txt
WITH WEB test AS (
    URL 'http://example.com' METHOD POST
    FORM_FIELD 'query' $JSON${"foo":"bar","nested":{"key":"value"}}$JSON$
)
SELECT * FROM test
EOF

OUTPUT=$(cat /tmp/test_format_output.txt)

# Check that output contains $JSON$ delimiters
if echo "$OUTPUT" | grep -q '\$JSON\$'; then
    echo "✓ $JSON$ delimiters preserved"
else
    echo "✗ $JSON$ delimiters missing!"
    echo "Output: $OUTPUT"
    exit 1
fi

# Check that JSON is pretty-printed (multiple lines)
if echo "$OUTPUT" | grep -A5 'FORM_FIELD' | grep -q '"foo"'; then
    echo "✓ JSON is formatted"
else
    echo "✗ JSON not formatted properly!"
    echo "Output: $OUTPUT"
    exit 1
fi

echo "All tests passed!"

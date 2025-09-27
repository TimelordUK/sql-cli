-- Test SQL query using web CTE to fetch FIX messages from JSON selector service
WITH fix_messages AS WEB (
    URL 'http://localhost:5050/api/messagequery/example/fix',
    METHOD 'POST',
    FORM_FILE 'file' '/home/me/dev/sql-cli/proxy/json-selector/test-data/fix-messages.json'
)
SELECT
    msg_type,
    symbol,
    side,
    quantity,
    price,
    trader,
    fund_allocations,
    total_allocated
FROM fix_messages
WHERE quantity > 500
ORDER BY price DESC;
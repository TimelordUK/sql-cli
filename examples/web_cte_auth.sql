-- Example: Using environment variables for authentication in WEB CTEs
-- This demonstrates how to use ${ENV_VAR} syntax for secure authentication
--
-- Set these environment variables before running:
--   export API_TOKEN="your-api-token-here"
--   export API_KEY="your-api-key-here"

-- Example 1: Using Bearer token authentication
-- The ${API_TOKEN} will be replaced with the value from environment
-- WITH WEB api_data AS (
    -- URL 'https://api.github.com/user/repos'
    -- FORMAT JSON
    -- HEADERS (
        -- 'Authorization': 'Bearer ${GITHUB_TOKEN}',
        -- 'Accept': 'application/vnd.github.v3+json'
    -- )
-- )
-- SELECT
    -- name as repo_name,
    -- private,
    -- created_at,
    -- updated_at,
    -- stargazers_count as stars
-- FROM api_data
-- ORDER BY stargazers_count DESC
-- LIMIT 10;

GO

-- Example 2: Multiple headers with environment variables
-- You can mix literal values with environment variables
WITH WEB secure_api AS (
    URL 'https://jsonplaceholder.typicode.com/posts'
    FORMAT JSON
    HEADERS (
        'X-API-Key': '${API_KEY}',
        'User-Agent': 'SQL-CLI/1.0',
        'Accept': 'application/json'
    )
)
SELECT
    id,
    title,
    LENGTH(body) as body_length
FROM secure_api
WHERE userId = 1
LIMIT 5;

GO

-- Example 3: Using environment variables in URL (future feature)
-- Note: Currently env vars only work in HEADERS, not in URL
-- This is a planned future enhancement
--
-- WITH WEB dynamic_api AS (
--     URL 'https://${API_HOST}/api/v1/data'
--     FORMAT JSON
--     HEADERS (
--         'Authorization': 'ApiKey ${API_KEY}'
--     )
-- )

-- Example 4: Practical example with JSONPlaceholder (no auth needed)
-- This works without environment variables for testing
WITH WEB posts AS (
    URL 'https://jsonplaceholder.typicode.com/posts'
    FORMAT JSON
    HEADERS (
        'User-Agent': 'SQL-CLI Testing'
    )
)
SELECT
    userId,
    COUNT(*) as post_count,
    AVG(LENGTH(title)) as avg_title_length
FROM posts
GROUP BY userId
ORDER BY post_count DESC
LIMIT 5;
GO

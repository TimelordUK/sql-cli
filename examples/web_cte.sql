-- Example: Fetching data from web sources using WEB CTEs
-- This feature allows SQL CLI to query data directly from HTTP/HTTPS endpoints

-- Example 1: Fetching JSON data from a REST API
WITH WEB posts AS (
    URL 'https://jsonplaceholder.typicode.com/posts'
    FORMAT JSON
)
SELECT
    userId,
    COUNT(*) as post_count,
    MIN(id) as first_post_id,
    MAX(id) as last_post_id
FROM posts
GROUP BY userId
ORDER BY post_count DESC, userId
LIMIT 10;

GO

-- Example 2: Fetching and filtering JSON data
WITH WEB users AS (
    URL 'https://jsonplaceholder.typicode.com/users'
    FORMAT JSON
)
SELECT
    id,
    name,
    email,
    SUBSTRING(email, INSTR(email, '@') + 1) as domain
FROM users
WHERE email LIKE '%@%.biz'
ORDER BY name;

GO

-- Example 3: Joining data from multiple web sources
WITH WEB posts AS (
    URL 'https://jsonplaceholder.typicode.com/posts'
    FORMAT JSON
),
WEB users AS (
    URL 'https://jsonplaceholder.typicode.com/users'
    FORMAT JSON
)
SELECT
    u.name as author,
    COUNT(p.id) as total_posts,
    AVG(LENGTH(p.body)) as avg_post_length
FROM posts p
INNER JOIN users u ON p.userId = u.id
GROUP BY u.name
ORDER BY total_posts DESC
LIMIT 5;

GO

-- Example 4: Auto-format detection (no FORMAT specified)
WITH WEB todos AS (
    URL 'https://jsonplaceholder.typicode.com/todos'
)
SELECT
    userId,
    COUNT(*) as total_todos,
    SUM(CASE WHEN completed = 'true' THEN 1 ELSE 0 END) as completed_todos,
    SUM(CASE WHEN completed = 'false' THEN 1 ELSE 0 END) as pending_todos
FROM todos
GROUP BY userId
ORDER BY userId
LIMIT 10;

GO
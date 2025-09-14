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

WITH
    WEB posts AS (
        URL 'https://jsonplaceholder.typicode.com/posts' FORMAT JSON
    ),
    WEB users AS (
        URL 'https://jsonplaceholder.typicode.com/users' FORMAT JSON
    )
SELECT
    name AS author_name,
    username,
    COUNT('*') AS post_count,
    MIN(LENGTH(title)) AS shortest_title_len,
    MAX(LENGTH(title)) AS longest_title_len,
    AVG(LENGTH(body)) AS avg_body_length
FROM posts
INNER JOIN users ON users.userId = posts.id
GROUP BY name, username
ORDER BY post_count DESC, author_name ASC
LIMIT 10;
GO

WITH
    WEB posts AS (
        URL 'https://jsonplaceholder.typicode.com/posts' FORMAT JSON
    ),
    WEB users AS (
        URL 'https://jsonplaceholder.typicode.com/users' FORMAT JSON
    )
SELECT
    SUBSTRING(title, 1, 40) AS post_title_preview,
    name AS author,
    email,
    website
FROM posts
INNER JOIN users ON userId = id
WHERE email LIKE '%@april.biz'
LIMIT 5;
GO

-- FILE CTE Showcase
-- Demonstrates filesystem querying using the FILE CTE feature.
-- This example queries the sql-cli source tree itself.

-- List the top 10 largest Rust source files
WITH f AS (FILE PATH 'src' RECURSIVE GLOB '*.rs')
SELECT name, size, depth, parent
FROM f
WHERE is_dir = false
ORDER BY size DESC
LIMIT 10;
GO

-- Count files and total bytes per top-level source directory
WITH f AS (FILE PATH 'src' RECURSIVE)
SELECT parent, COUNT(*) as files, FORMAT_BYTES(SUM(size)) as total_bytes
FROM f
WHERE is_dir = false
GROUP BY parent
ORDER BY total_bytes DESC
LIMIT 15;
GO

-- File extension breakdown across the entire project (depth-limited)
WITH f AS (FILE PATH '.' RECURSIVE MAX_DEPTH 4 GLOB '*.*')
SELECT ext, COUNT(*) as count, SUM(size) as total_bytes
FROM f
WHERE is_dir = false AND ext IS NOT NULL
GROUP BY ext
ORDER BY count DESC
LIMIT 15;
GO

-- Find all SQL example files and their sizes
WITH f AS (FILE PATH 'examples' GLOB '*.sql')
SELECT name, size
FROM f
WHERE is_dir = false
ORDER BY name;
GO

-- Non-recursive listing of project root — directories vs files
WITH f AS (FILE PATH '.')
SELECT name, is_dir, size
FROM f
WHERE depth = 1
ORDER BY is_dir DESC, name;
GO

-- Find recently modified Rust files (using string comparison on ISO timestamps)
WITH f AS (FILE PATH 'src' RECURSIVE GLOB '*.rs')
SELECT name, size, modified
FROM f
WHERE is_dir = false
ORDER BY modified DESC
LIMIT 10;
GO

-- Hidden files at project root (include dotfiles)
WITH f AS (FILE PATH '.' INCLUDE_HIDDEN)
SELECT name, is_dir, size
FROM f
WHERE name LIKE '.%'
ORDER BY name;
GO

-- Directory tree summary — count entries at each depth level
WITH f AS (FILE PATH 'src' RECURSIVE MAX_DEPTH 3)
SELECT depth, COUNT(*) as entries,
       SUM(CASE WHEN is_dir = true THEN 1 ELSE 0 END) as dirs,
       SUM(CASE WHEN is_dir = false THEN 1 ELSE 0 END) as files
FROM f
GROUP BY depth
ORDER BY depth;
GO

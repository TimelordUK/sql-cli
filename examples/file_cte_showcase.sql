-- FILE CTE Showcase
-- Demonstrates filesystem querying using the FILE CTE feature.
-- This example queries the sql-cli source tree itself.

-- List the top 10 largest Rust source files
WITH f AS (FILE PATH 'src' RECURSIVE GLOB '*.rs')
SELECT name, size,  FORMAT_BYTES(size) as size_h, depth, parent
FROM f
WHERE is_dir = false
ORDER BY size DESC
LIMIT 10;
GO

-- Count files and total bytes per top-level source directory
WITH f AS (FILE PATH 'src' RECURSIVE)
SELECT parent, COUNT(*) as files, sum(size) as total_bytes, FORMAT_BYTES(SUM(size)) as total_bytes_h
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
SELECT * 
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

-- ============================================================
-- Path-manipulation functions (POSIX-style) applied to the
-- `path` column of the FILE CTE. These also work on any
-- string column containing a path.
--   BASENAME(p)   -> last component (file name with extension)
--   DIRNAME(p)    -> parent directory
--   EXTENSION(p)  -> extension without leading dot, or NULL
--   STEM(p)       -> file name without extension
--   PATH_DEPTH(p) -> number of components
--   PATH_PART(p, n) -> nth component (1-based; -1 = last)
-- ============================================================

-- Show each function side-by-side on a handful of source files
WITH f AS (FILE PATH 'src' RECURSIVE GLOB '*.rs')
SELECT BASENAME(path)      as fname,
       STEM(path)          as stem,
       EXTENSION(path)     as ext,
       PATH_PART(path, -2) as folder,
       PATH_DEPTH(path)    as depth
FROM f
WHERE is_dir = false
ORDER BY fname
LIMIT 10;
GO

-- Group Rust sources by their immediate parent folder using PATH_PART
WITH f AS (FILE PATH 'src' RECURSIVE GLOB '*.rs')
SELECT PATH_PART(path, -2) as folder,
       COUNT(*)            as files,
       SUM(size)           as total_bytes
FROM f
WHERE is_dir = false
GROUP BY folder
ORDER BY files DESC
LIMIT 15;
GO

-- STEM vs BASENAME — handy for multi-extension files like .tar.gz.
-- Also demonstrates EXTENSION returning NULL for files without one.
WITH samples AS (
    SELECT '/var/log/syslog'           AS p UNION ALL
    SELECT '/home/me/archive.tar.gz'         UNION ALL
    SELECT 'README'                          UNION ALL
    SELECT '.gitignore'                      UNION ALL
    SELECT 'src/main.rs'
)
SELECT p,
       BASENAME(p)  as base,
       STEM(p)      as stem,
       EXTENSION(p) as ext,
       DIRNAME(p)   as dir
FROM samples;
GO

-- Find every Rust file NOT at the top of src/ (depth > 2) and show
-- which sub-module it lives in using PATH_PART negative indexing
WITH f AS (FILE PATH 'src' RECURSIVE GLOB '*.rs')
SELECT PATH_PART(path, -2) as module,
       BASENAME(path)      as fname,
       PATH_DEPTH(path)    as depth
FROM f
WHERE is_dir = false AND PATH_DEPTH(path) > 2
ORDER BY module, fname
LIMIT 20;
GO

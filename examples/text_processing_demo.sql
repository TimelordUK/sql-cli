-- Text-processing demo: querying a plain-text book as a SQL table.
--
-- Source: Project Gutenberg eBook #84, Frankenstein; or, The Modern Prometheus
-- by Mary Shelley (1818 — public domain). We ship an excerpt of Letter 1
-- (Walton's opening letter) and Chapter 5 (the creation scene).
--
-- Run from the repo root so the relative path resolves:
--   ./target/release/sql-cli examples/text_processing_demo.sql


-- ================================================================
-- SECTION 1 — basic ingestion
-- ================================================================

-- How big is the excerpt? Lines, characters, words.
SELECT
    COUNT(*)             AS total_lines,
    SUM(LENGTH(line))    AS total_chars,
    SUM(WORD_COUNT(line)) AS total_words
FROM READ_TEXT('data/frankenstein_excerpt.txt');
GO


-- ================================================================
-- SECTION 2 — regex pre-filter at read time
-- ================================================================
-- READ_TEXT's optional second argument is a regex. Matching lines are the
-- only ones materialised — the rest of the file is skipped. line_num still
-- reflects the original file position, so row identity is preserved.

-- Structural markers (Letter N / Chapter N on a line by themselves).
SELECT line_num, line
FROM READ_TEXT('data/frankenstein_excerpt.txt', '^(Letter|Chapter) [0-9]+$');
GO

-- Every line that mentions death in the excerpt (case-insensitive).
SELECT line_num, line
FROM READ_TEXT('data/frankenstein_excerpt.txt', '(?i)\bdeath\b');
GO


-- ================================================================
-- SECTION 3 — GREP reader (composable)
-- ================================================================
-- GREP is a thin wrapper around the same line-reader. Useful when you want
-- "find lines" as a first-class CTE you can re-use.

-- Count all "I" sentences (Walton's first-person voice).
WITH first_person AS (
    SELECT * FROM GREP('data/frankenstein_excerpt.txt', '(?i)\bI\b')
)
SELECT COUNT(*) AS first_person_lines FROM first_person;
GO

-- grep -v style: lines that are NOT blank and NOT pure whitespace.
-- The third argument inverts the match.
SELECT COUNT(*) AS non_empty_lines
FROM GREP('data/frankenstein_excerpt.txt', '^\s*$', true);
GO


-- ================================================================
-- SECTION 4 — line-level analytics
-- ================================================================

-- Longest lines in the excerpt (excluding our provenance header).
SELECT line_num, LENGTH(line) AS chars, WORD_COUNT(line) AS words, line
FROM READ_TEXT('data/frankenstein_excerpt.txt')
WHERE line_num > 8
ORDER BY chars DESC
LIMIT 5;
GO

-- Distribution of line lengths bucketed by tens of characters.
-- Note: `/` is float division here, so we bucket via `x - (x % 10)` for
-- a clean integer bucket boundary.
SELECT
    LENGTH(line) - (LENGTH(line) % 10) AS length_bucket,
    COUNT(*)                           AS line_count
FROM READ_TEXT('data/frankenstein_excerpt.txt')
WHERE LENGTH(line) > 0
GROUP BY LENGTH(line) - (LENGTH(line) % 10)
ORDER BY length_bucket;
GO


-- ================================================================
-- SECTION 5 — section-scoped queries via line ranges
-- ================================================================
-- Once you know where a section begins and ends (see SECTION 2), you can
-- filter by line_num for scoped analysis.

-- Chapter 5 starts at line 133 in the excerpt. Word count for the chapter.
WITH chapter_five AS (
    SELECT *
    FROM READ_TEXT('data/frankenstein_excerpt.txt')
    WHERE line_num >= 133
)
SELECT
    COUNT(*)              AS chapter_lines,
    SUM(WORD_COUNT(line)) AS chapter_words,
    SUM(CASE WHEN line LIKE '%Clerval%' THEN 1 ELSE 0 END) AS clerval_mentions,
    SUM(CASE WHEN line LIKE '%Elizabeth%' THEN 1 ELSE 0 END) AS elizabeth_mentions
FROM chapter_five;
GO


-- ================================================================
-- SECTION 6 — sentiment-ish: word-class frequency
-- ================================================================
-- Naive emotional colour via scalar subqueries — each counts lines matching
-- a keyword family. READ_TEXT's own regex filter does the work; we just
-- wrap the COUNT. Neatly sidesteps stacked CASE WHEN with regex.

SELECT
    (SELECT COUNT(*) FROM READ_TEXT('data/frankenstein_excerpt.txt',
        '(?i)\b(horror|dread|terror|fear|despair)\b')) AS fear_lines,
    (SELECT COUNT(*) FROM READ_TEXT('data/frankenstein_excerpt.txt',
        '(?i)\b(beauty|love|joy|hope|delight)\b'))     AS light_lines,
    (SELECT COUNT(*) FROM READ_TEXT('data/frankenstein_excerpt.txt',
        '(?i)\b(death|dead|grave|corpse|lifeless)\b')) AS mortal_lines;
GO

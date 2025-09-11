-- Test file for the expand * feature
-- #!data: /tmp/test_schema.csv

-- Test 1: Simple SELECT *
SELECT * FROM test_schema

-- Test 2: SELECT * with WHERE clause
SELECT * FROM test_schema WHERE col1 > 5

-- Test 3: SELECT * with multiple clauses
SELECT * FROM test_schema WHERE col1 > 5 ORDER BY col2

-- Instructions:
-- 1. Open this file in Neovim: nvim test_expand_star.sql
-- 2. The data file should auto-detect from the #!data hint
-- 3. Place cursor on any line with SELECT *
-- 4. Press \se (or <leader>se) to expand the * to all column names
-- 
-- For tables with <= 5 columns, it will expand inline:
-- SELECT col1, col2, col3, col4, col5 FROM test_schema
--
-- For tables with > 5 columns, it will use multi-line format:
-- SELECT
--     col1
--   , col2
--   , col3
--   , col4
--   , col5
--   , col6
-- FROM test_schema
-- #!data: data/test_simple_strings.csv
-- Test file for SQL CLI autocomplete in Neovim
-- 
-- Instructions to test autocomplete:
-- 1. Open this file in Neovim
-- 2. The plugin should auto-detect the data file from the comment above
-- 3. Try these autocomplete triggers in insert mode:
--    - Type "SELECT " then press M-; (Alt+semicolon) to see column completions
--    - Type "SELECT na" then press M-; to see filtered completions for columns starting with "na"
--    - Press <C-Space> anywhere to trigger general SQL completion
--    - When completion menu is open:
--      * Press Tab to navigate down through options
--      * Press Shift-Tab to navigate up
--      * Press Enter to accept selected option
--      * Press 1-9 to quickly select and accept that numbered item
--
-- Test queries - try autocomplete at the marked positions:

-- Test 1: Basic column completion
SELECT  -- <- Press M-; here to see all columns

-- Test 2: Filtered column completion  
SELECT na -- <- Press M-; here to see columns starting with "na"

-- Test 3: Multiple column selection
SELECT id,  -- <- Press M-; here after the comma

-- Test 4: WHERE clause completion
SELECT * FROM test_simple_strings WHERE  -- <- Press M-; here

-- Test 5: Column in WHERE condition
SELECT * FROM test_simple_strings WHERE sta -- <- Press M-; to complete "status"

-- Test 6: Function completion
SELECT CO -- <- Press <C-Space> here to see CONVERT, COUNT, etc.

-- Test 7: Mix of columns and functions
SELECT id, UPPER( -- <- Press M-; here to select column for UPPER function

-- Available columns in test_simple_strings.csv:
-- - id (INTEGER)
-- - name (STRING)  
-- - email (STRING)
-- - status (STRING)
-- - code (STRING)
-- String manipulation functions
-- sql-cli provides essential string operations

-- Basic string operations
SELECT 
    UPPER('hello world') as uppercase,
    LOWER('HELLO WORLD') as lowercase,
    LENGTH('sql-cli') as string_length,
    TRIM('  spaces  ') as trimmed;
GO

-- String extraction and manipulation
SELECT 
    SUBSTRING('Hello World', 1, 5) as substr,
    MID('Database', 3, 4) as mid_extract,
    REPLACE('Hello World', 'World', 'SQL') as replaced;
GO

-- String joining and checking
SELECT 
    TEXTJOIN(' ', 1, 'Hello', 'World', 'SQL') as joined_text,
    TEXTJOIN(', ', 1, 'apple', 'banana', 'orange') as joined_with_comma,
    STARTSWITH('sql-cli', 'sql') as starts_check,
    ENDSWITH('database.db', '.db') as ends_check,
    CONTAINS('hello world', 'world') as contains_check;
GO

-- Edit distance for fuzzy matching
SELECT 
    EDIT_DISTANCE('kitten', 'sitting') as edit_dist,
    EDIT_DISTANCE('saturday', 'sunday') as weekend_dist,
    EDIT_DISTANCE('sql', 'SQL') as case_diff;
GO

-- TODO: These functions would be useful but are not yet implemented:
-- REVERSE() - Reverse a string
-- LEFT()/RIGHT() - Extract from left/right
-- CONCAT() - Concatenate strings
-- SPLIT_PART() - Split and extract part
-- POSITION() - Find position of substring
-- LPAD()/RPAD() - Padding functions
-- INITCAP() - Title case
-- MD5(), SHA1(), SHA256() - Hash functions (would be very useful!)
-- REGEXP_REPLACE() - Regex replacement
-- REGEXP_EXTRACT() - Regex extraction

-- Practical example: Data cleaning with available functions
-- SELECT 
--     email,
--     LOWER(TRIM(email)) as cleaned_email,
--     CASE 
--         WHEN ENDSWITH(email, '.com') THEN 'Commercial'
--         WHEN ENDSWITH(email, '.edu') THEN 'Educational'
--         WHEN ENDSWITH(email, '.org') THEN 'Organization'
--         ELSE 'Other'
--     END as domain_type
-- FROM users_table
-- WHERE email IS NOT NULL;
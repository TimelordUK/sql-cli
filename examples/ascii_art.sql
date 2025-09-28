-- ASCII Art Examples
-- Demonstrates the ASCII_ART, BIG_TEXT, and BANNER generators
WITH
    words AS (
        SELECT value AS word
        FROM SPLIT('hello world')
    )
SELECT word
FROM words;
go

-- Basic ASCII art generation
SELECT * FROM ASCII_ART('SQL-CLI');
GO

-- Using the BIG_TEXT alias
SELECT * FROM BIG_TEXT('DATA');
GO

-- Creating a banner with custom border
SELECT * FROM BANNER('HELLO', '*');
GO

-- Different border style
SELECT * FROM BANNER('SQL', '=');
GO

-- Combine with string operations - uppercase the ASCII art
SELECT UPPER(line) as big_line FROM ASCII_ART('wow');
GO

-- Create ASCII art for each word from a split string
SELECT * FROM ASCII_ART('SQL');
GO

SELECT * FROM ASCII_ART('CLI');
GO

SELECT * FROM ASCII_ART('ROCKS');
GO

-- Show the words that can be split
SELECT * FROM SPLIT('HELLO,WORLD,SQL', ',');
GO

-- Create banners for different sections
SELECT * FROM BANNER('INTRO', '=');
GO

SELECT * FROM BANNER('DATA', '#');
GO

SELECT * FROM BANNER('END', '*');
GO

-- Fun example: Create ASCII art for numbers
SELECT * FROM ASCII_ART('123');
GO

SELECT * FROM ASCII_ART('2024');
GO

-- Combine with string functions
SELECT
    LENGTH(line) as line_length,
    line
FROM ASCII_ART('HI');
GO

-- Create a report header
SELECT * FROM BANNER('REPORT', '=');
GO

SELECT * FROM ASCII_ART('Q4-2024');
GO

-- ASCII art with special characters
SELECT * FROM ASCII_ART('$$$');
GO

SELECT * FROM ASCII_ART('A-B-C');
GO

-- Counting lines in ASCII art
SELECT COUNT(*) as total_lines FROM ASCII_ART('TEST');
GO

-- Get the widest line from ASCII art
SELECT
    MAX(LENGTH(line)) as max_width,
    MIN(LENGTH(line)) as min_width
FROM ASCII_ART('WIDTH');
GO

-- Create a decorative separator
SELECT * FROM BANNER('---', '=');
GO

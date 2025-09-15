-- Fun conversion functions showcase

-- Roman numerals sequence
SELECT
    value as number,
    TO_ROMAN(value) as roman,
    TO_WORDS(value) as words,
    TO_ORDINAL(value) as ordinal
FROM RANGE(1, 12);
GO

-- Important years in Roman numerals
SELECT
    1066 as year,
    TO_ROMAN(1066) as roman_year,
    'Battle of Hastings' as event;
GO

-- More historical dates
SELECT
    CASE value
        WHEN 1 THEN 1492
        WHEN 2 THEN 1776
        WHEN 3 THEN 1969
        WHEN 4 THEN 2000
        WHEN 5 THEN 2024
    END as year,
    CASE value
        WHEN 1 THEN TO_ROMAN(1492)
        WHEN 2 THEN TO_ROMAN(1776)
        WHEN 3 THEN TO_ROMAN(1969)
        WHEN 4 THEN TO_ROMAN(2000)
        WHEN 5 THEN TO_ROMAN(2024)
    END as roman_year,
    CASE value
        WHEN 1 THEN 'Columbus discovers America'
        WHEN 2 THEN 'American Independence'
        WHEN 3 THEN 'Moon Landing'
        WHEN 4 THEN 'Y2K'
        WHEN 5 THEN 'Current Year'
    END as event
FROM RANGE(1, 5)
ORDER BY year;
GO

-- Fun with ordinals
SELECT
    value,
    TO_ORDINAL(value) as ordinal,
    TO_ORDINAL_WORDS(value) as ordinal_words
FROM RANGE(1, 31)
WHERE value IN (1, 2, 3, 11, 12, 13, 21, 22, 23, 31);
GO

-- Some interesting numbers in words
SELECT
    42 as number,
    TO_WORDS(42) as in_words,
    'Answer to everything' as description;
GO

-- More numbers in words
SELECT
    value as number,
    TO_WORDS(value) as in_words
FROM RANGE(100, 110);
GO

-- Century conversions
SELECT
    value as century,
    TO_ORDINAL_WORDS(value) || ' century' as century_name,
    TO_ROMAN((value - 1) * 100 + 1) || '-' || TO_ROMAN(value * 100) as year_range
FROM RANGE(18, 21);
GO
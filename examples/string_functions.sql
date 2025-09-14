-- #! ../data/international_sales.csv
-- String Functions Examples
-- Comprehensive guide to all string manipulation functions in sql-cli
-- data: data/international_sales.csv

-- === CONCATENATION ===

-- Concat operator (||) - simple and elegant string joining
SELECT
    'Hello' || ' ' || 'World' AS greeting,
    country || ' - ' || region AS location,
    product || ' (' || currency || ')' AS product_info
FROM international_sales
LIMIT 5;
GO

-- TEXTJOIN function - join multiple values with a separator
SELECT
    TEXTJOIN(', ', 1, 'apple', 'banana', 'orange') AS fruits,
    TEXTJOIN(' | ', 1, country, region, product) AS combined_info,
    TEXTJOIN('-', 0, 'A', NULL, 'B', '', 'C') AS with_nulls_and_empty
FROM international_sales
LIMIT 5;
GO

-- === CASE CONVERSION ===

-- UPPER, LOWER, TOUPPER, TOLOWER
SELECT
    'Hello World' AS original,
    UPPER('Hello World') AS upper_case,
    LOWER('Hello World') AS lower_case,
    TOUPPER('mixed CASE text') AS to_upper,
    TOLOWER('MIXED case TEXT') AS to_lower;
GO

-- === TRIMMING & PADDING ===

-- TRIM functions - remove whitespace
SELECT
    '  spaces  ' AS original,
    TRIM('  spaces  ') AS trimmed,
    TRIMSTART('  left spaces') AS trim_start,
    TRIMEND('right spaces  ') AS trim_end,
    LENGTH('  text  ') AS original_length,
    LENGTH(TRIM('  text  ')) AS trimmed_length;
GO

-- LPAD and RPAD - padding with characters
SELECT
    'SQL' AS original,
    LPAD('SQL', 10, '*') AS left_padded,
    RPAD('SQL', 10, '-') AS right_padded,
    LPAD('42', 5, '0') AS zero_padded,
    RPAD('Item', 20, '.') AS dotted_line;
GO

-- CENTER - center text within a field width
SELECT
    CENTER('Title', 20) AS centered_spaces,
    CENTER('SQL', 20, '=') AS centered_equals,
    CENTER('*', 10, '-') AS centered_star;
GO

-- === SUBSTRING OPERATIONS ===

-- LEFT and RIGHT - extract from ends
SELECT
    product,
    LEFT(product, 5) AS first_5_chars,
    RIGHT(product, 3) AS last_3_chars,
    LEFT(currency, 1) AS currency_symbol
FROM international_sales
LIMIT 5;
GO

-- MID and SUBSTRING - extract from middle
SELECT
    'ABCDEFGHIJ' AS text,
    MID('ABCDEFGHIJ', 3, 4) AS mid_3_4,
    SUBSTRING('ABCDEFGHIJ', 5, 3) AS substring_5_3,
    SUBSTRING('Hello World', 7, 5) AS extract_world;
GO

-- SUBSTRING_BEFORE and SUBSTRING_AFTER
SELECT
    'user@example.com' AS email,
    SUBSTRING_BEFORE('user@example.com', '@') AS username,
    SUBSTRING_AFTER('user@example.com', '@') AS domain,
    SUBSTRING_BEFORE('a-b-c-d', '-', 2) AS before_2nd_dash,
    SUBSTRING_AFTER('a-b-c-d', '-', 2) AS after_2nd_dash;
GO

-- === SEARCH & REPLACE ===

-- CONTAINS, STARTSWITH, ENDSWITH
-- Expressions in WHERE need to be computed in CTE first
WITH product_checks AS (
    SELECT
        product,
        CONTAINS(product, 'Phone') AS has_phone,
        STARTSWITH(product, 'Tab') AS starts_tab,
        ENDSWITH(product, 'et') AS ends_et
    FROM international_sales
)
SELECT
    product,
    has_phone,
    starts_tab,
    ends_et
FROM product_checks
WHERE has_phone = true
LIMIT 5;
GO

-- INDEXOF and INSTR - find position of substring
SELECT
    'Hello World' AS text,
    INDEXOF('Hello World', 'World') AS index_0_based,
    INSTR('Hello World', 'World') AS position_1_based,
    INDEXOF('Mississippi', 'ss') AS first_ss,
    INSTR('Mississippi', 'pp') AS position_pp;
GO

-- REPLACE - substitute text
SELECT
    product,
    REPLACE(product, 'Phone', 'Device') AS replaced_phone,
    REPLACE(product, ' ', '_') AS underscored,
    REPLACE(LOWER(product), 'smart', 'intelligent') AS smart_to_intelligent
FROM international_sales
LIMIT 5;
GO

-- FREQUENCY - count occurrences
SELECT
    'Mississippi' AS word,
    FREQUENCY('Mississippi', 's') AS count_s,
    FREQUENCY('Mississippi', 'ss') AS count_ss,
    FREQUENCY('Mississippi', 'i') AS count_i,
    FREQUENCY('aaa bbb aaa ccc aaa', 'aaa') AS count_aaa;
GO

WITH
    lengths AS (
        SELECT
            product,
            LENGTH(product) AS product_length,
            country,
            LENGTH(country) AS country_length
        FROM international_sales
    )
SELECT *
FROM lengths
ORDER BY product_length DESC
LIMIT 5;
GO

-- EDIT_DISTANCE - Levenshtein distance for similarity
SELECT
    EDIT_DISTANCE('kitten', 'sitting') AS kitten_sitting,
    EDIT_DISTANCE('saturday', 'sunday') AS saturday_sunday,
    EDIT_DISTANCE('SQL', 'sql') AS case_difference,
    EDIT_DISTANCE('database', 'databasE') AS one_char_diff;
GO

-- === SPLITTING ===

-- SPLIT_PART - extract parts from delimited strings
SELECT
    'one,two,three,four' AS csv_string,
    SPLIT_PART('one,two,three,four', ',', 1) AS first,
    SPLIT_PART('one,two,three,four', ',', 2) AS second,
    SPLIT_PART('one,two,three,four', ',', 3) AS third,
    SPLIT_PART('one,two,three,four', ',', 4) AS fourth;
GO

-- Practical: Parse file paths
SELECT
    '/home/user/documents/report.pdf' AS filepath,
    SPLIT_PART('/home/user/documents/report.pdf', '/', 3) AS username,
    SPLIT_PART('/home/user/documents/report.pdf', '/', 4) AS folder,
    SUBSTRING_AFTER('/home/user/documents/report.pdf', '/documents/') AS filename;
GO


SELECT * from SPLIT('123','45.67','abc','2024-01-15','true','NULL');
GO

-- === TYPE CHECKING ===

-- IS_* functions for data validation
WITH test_values AS (
    SELECT value as val from SPLIT('123 45.67 abc 2024-01-15 true NULL') AS val 
)
SELECT
    val,
    IS_INTEGER(val) AS is_int,
    IS_FLOAT(val) AS is_float,
    IS_NUMERIC(val) AS is_num,
    IS_DATE(val) AS is_date,
    IS_BOOL(val) AS is_bool,
    IS_NULL(val) AS is_null,
    IS_NOT_NULL(val) AS not_null
FROM test_values;
GO

-- === HASH FUNCTIONS ===

-- MD5, SHA1, SHA256, SHA512 - cryptographic hashes
SELECT
    'password123' AS text,
    MD5('password123') AS md5_hash,
    SHA1('password123') AS sha1_hash,
    LEFT(SHA256('password123'), 32) AS sha256_first32,
    LEFT(SHA512('password123'), 32) AS sha512_first32;
GO

-- === CHARACTER FUNCTIONS ===

-- CHR - ASCII code to character
SELECT
    CHR(65) AS char_A,
    CHR(97) AS char_a,
    CHR(48) AS char_0,
    CHR(33) AS exclamation,
    CHR(65) || CHR(66) || CHR(67) AS ABC,
    CHR(72) || CHR(105) || CHR(33) AS greeting;
GO

-- === LOREM IPSUM GENERATOR ===

-- LOREM_IPSUM - Generate placeholder text
SELECT
    LOREM_IPSUM(10) AS ten_words,
    LENGTH(LOREM_IPSUM(10)) AS text_length;
GO

-- Generate traditional Lorem Ipsum starting text
SELECT
    LOREM_IPSUM(20, 1) AS traditional_start,
    LOREM_IPSUM(20, 0) AS random_start;
GO

-- Generate various lengths of Lorem Ipsum
SELECT
    LOREM_IPSUM(5) AS short_text,
    LOREM_IPSUM(50) AS medium_text,
    LENGTH(LOREM_IPSUM(100)) AS hundred_words_length;
GO

-- Practical use: Generate test data with Lorem Ipsum
-- Using id as seed ensures each row gets different text
WITH test_records AS (
    SELECT value AS id FROM RANGE(1, 6)
)
SELECT
    id,
    'Product ' || id AS product_name,
    LOREM_IPSUM(10 + (id * 2), 0, id) AS description,
    ROUND(RANDOM() * 100, 2) AS price
FROM test_records;
GO

SELECT
    TEXTJOIN('', 1, 'Review ', value) AS review_id,
    LOREM_IPSUM(20, 0, value * 7) AS review_text,
    LOREM_IPSUM(5, 0, value * 13) AS summary
FROM RANGE(1, 3);
GO

SELECT *
FROM SPLIT('apple,banana,orange', ',');
GO

SELECT *
FROM SPLIT('hello world sql cli');
GO

SELECT
    value AS word,
    LENGTH(value) AS word_length
FROM SPLIT(LOREM_IPSUM(10))
ORDER BY word_length DESC;
GO

WITH
    emails AS (
        SELECT *
        FROM SPLIT('john@example.com,jane@test.org,bob@company.net', ',')
    )
SELECT
    value AS email,
    SUBSTRING_BEFORE(value, '@') AS username,
    SUBSTRING_AFTER(value, '@') AS domain
FROM emails;
GO

-- Character-by-character split (empty delimiter)
SELECT value AS char, index AS position
FROM SPLIT('SQL', '');
GO

SELECT
    value,
    SPLIT_PART(value, ':', 1) AS key,
    SPLIT_PART(value, ':', 2) AS val
FROM SPLIT('name:John,age:30,city:NYC', ',');
GO

WITH
    words AS (
        SELECT
            LOWER(REPLACE(REPLACE(value, '.', ''), ',', '')) AS word
        FROM SPLIT('The quick brown fox jumps over the lazy dog. The fox is quick.')
    )
SELECT
    word,
    COUNT('*') AS frequency
FROM words
WHERE LENGTH(word) > 0
GROUP BY word
ORDER BY frequency DESC;
GO

-- === FORMATTING FUNCTIONS ===

-- FORMAT_NUMBER - thousand separators and decimals
WITH sales AS (
    SELECT amount * quantity AS total
    FROM international_sales
    WHERE amount * quantity > 1000
)
SELECT
    total,
    FORMAT_NUMBER(total) AS formatted,
    FORMAT_NUMBER(total, 0) AS no_decimals,
    FORMAT_NUMBER(total, 1) AS one_decimal,
    FORMAT_NUMBER(total, 3) AS three_decimals
FROM sales
LIMIT 5;
GO

-- RENDER_NUMBER - various number formats including compact notation
SELECT
    1234567.89 AS number,
    RENDER_NUMBER(1234567.89) AS standard,
    RENDER_NUMBER(1234567.89, 'compact') AS compact,
    RENDER_NUMBER(1234567.89, 'compact', 1) AS compact_1d,
    RENDER_NUMBER(-1234.56, 'accounting') AS accounting,
    RENDER_NUMBER(1234.56, 'eu') AS european,
    RENDER_NUMBER(1234567, 'in') AS indian;
GO

-- FORMAT_CURRENCY - currency formatting with column values
SELECT
    product,
    amount,
    currency,
    FORMAT_CURRENCY(amount, currency) AS with_symbol,
    FORMAT_CURRENCY(amount, currency, 'code') AS with_code,
    FORMAT_CURRENCY(amount * quantity, currency, 'compact') AS total_compact
FROM international_sales
LIMIT 5;
GO

-- === PRACTICAL EXAMPLES ===

-- Build formatted messages using concatenation
-- Window functions need to be in CTE or separate column
WITH numbered_orders AS (
    SELECT
        ROW_NUMBER() OVER (ORDER BY date) AS order_num,
        country,
        quantity,
        product,
        amount,
        currency
    FROM international_sales
)
SELECT
    'Order #' || LPAD(order_num, 5, '0') ||
    ' from ' || country || ': ' ||
    quantity || ' x ' || product ||
    ' = ' || FORMAT_CURRENCY(amount * quantity, currency) AS order_summary
FROM numbered_orders
LIMIT 5;
GO

-- Clean and standardize data
SELECT
    product,
    TRIM(UPPER(product)) AS standardized,
    REPLACE(LOWER(product), ' ', '_') AS snake_case,
    REPLACE(REPLACE(LOWER(product), ' ', '-'), '&', 'and') AS url_slug
FROM international_sales
LIMIT 10;
GO

-- Extract and validate email parts
WITH emails AS (
  select value as email from SPLIT('john.doe@example.com jane@company.co.uk admin@localhost invalid-email')
)
SELECT
    email,
    CONTAINS(email, '@') AS is_valid,
    SUBSTRING_BEFORE(LOWER(email), '@') AS username,
    SUBSTRING_AFTER(LOWER(email), '@') AS domain,
    SPLIT_PART(SUBSTRING_AFTER(email, '@'), '.', 1) AS domain_name,
    CASE
        WHEN ENDSWITH(email, '.com') THEN 'Commercial'
        WHEN ENDSWITH(email, '.org') THEN 'Organization'
        WHEN ENDSWITH(email, '.edu') THEN 'Educational'
        WHEN ENDSWITH(email, '.gov') THEN 'Government'
        ELSE 'Other'
    END AS domain_type
FROM emails;
GO

-- Find similar products using edit distance
-- SELECT
    -- p1.product AS product1,
    -- p2.product AS product2,
    -- EDIT_DISTANCE(UPPER(p1.product), UPPER(p2.product)) AS distance
-- FROM (SELECT DISTINCT product FROM international_sales LIMIT 5) p1
-- CROSS JOIN (SELECT DISTINCT product FROM international_sales LIMIT 5) p2
-- WHERE p1.product != p2.product
  -- AND EDIT_DISTANCE(UPPER(p1.product), UPPER(p2.product)) <= 5
-- ORDER BY distance;
GO

-- Generate product codes
-- Window function in CTE for proper evaluation
WITH numbered_products AS (
    SELECT
        product,
        currency,
        ROW_NUMBER() OVER (ORDER BY product) AS row_num
    FROM international_sales
)
SELECT
    product,
    UPPER(LEFT(product, 3)) || '-' ||
    LPAD(row_num, 4, '0') || '-' ||
    LEFT(MD5(product), 6) AS product_code,
    LEFT(SHA256(product || currency), 8) AS sku_hash
FROM numbered_products
LIMIT 10;
GO

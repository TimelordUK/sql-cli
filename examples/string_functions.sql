-- Comprehensive string manipulation functions
-- sql-cli provides a rich set of string operations

-- Basic string operations
SELECT 
    UPPER('hello world') as uppercase,
    LOWER('HELLO WORLD') as lowercase,
    LENGTH('sql-cli') as string_length,
    REVERSE('stressed') as reversed,
    TRIM('  spaces  ') as trimmed;

-- String extraction and manipulation
SELECT 
    SUBSTRING('Hello World', 1, 5) as substr,
    LEFT('Database', 4) as left_part,
    RIGHT('Database', 4) as right_part,
    REPLACE('Hello World', 'World', 'SQL') as replaced,
    REPEAT('SQL', 3) as repeated;

-- Advanced string functions
SELECT 
    CONCAT('Hello', ' ', 'World') as concatenated,
    CONCAT_WS(',', 'apple', 'banana', 'orange') as joined_with_separator,
    SPLIT_PART('user@example.com', '@', 2) as domain,
    POSITION('cli' IN 'sql-cli') as position_found,
    STARTS_WITH('sql-cli', 'sql') as starts_check,
    ENDS_WITH('database.db', '.db') as ends_check;

-- Pattern matching and regex
SELECT 
    'hello123world' RLIKE '[0-9]+' as contains_numbers,
    REGEXP_REPLACE('hello123world', '[0-9]+', 'XXX') as numbers_replaced,
    REGEXP_EXTRACT('email: user@example.com', '[a-z]+@[a-z]+\\.[a-z]+') as extracted_email;

-- Padding and formatting
SELECT 
    LPAD('123', 6, '0') as left_padded,
    RPAD('test', 10, '-') as right_padded,
    FORMAT(1234567.89, 2) as formatted_number,
    INITCAP('hello world from sql') as title_case;

-- Hash functions
SELECT 
    MD5('password123') as md5_hash,
    SHA1('password123') as sha1_hash,
    SHA256('password123') as sha256_hash;

-- Practical example: Data cleaning and transformation
SELECT 
    email,
    LOWER(TRIM(email)) as cleaned_email,
    SPLIT_PART(email, '@', 1) as username,
    SPLIT_PART(email, '@', 2) as domain,
    CASE 
        WHEN ENDS_WITH(email, '.com') THEN 'Commercial'
        WHEN ENDS_WITH(email, '.edu') THEN 'Educational'
        WHEN ENDS_WITH(email, '.org') THEN 'Organization'
        ELSE 'Other'
    END as domain_type,
    MD5(LOWER(TRIM(email))) as email_hash
FROM users_table
WHERE email IS NOT NULL;
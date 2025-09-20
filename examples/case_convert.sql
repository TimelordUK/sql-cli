-- #! ../data/sample_words.csv

-- Case Conversion Examples
-- This demonstrates the various case conversion functions in SQL CLI
-- Useful for converting between different naming conventions

-- Basic case conversions for all sample words
SELECT
    id,
    original_text,
    TO_SNAKE_CASE(original_text) as snake_case,
    TO_CAMEL_CASE(original_text) as camel_case,
    TO_PASCAL_CASE(original_text) as pascal_case,
    TO_KEBAB_CASE(original_text) as kebab_case,
    TO_CONSTANT_CASE(original_text) as constant_case
FROM sample_words
ORDER BY id
LIMIT 10;
GO

-- Converting typical database column names to different conventions
-- Useful when integrating with different systems or APIs
SELECT
    original_text,
    description,
    TO_CAMEL_CASE(original_text) as js_style,
    TO_PASCAL_CASE(original_text) as csharp_style,
    TO_SNAKE_CASE(original_text) as python_style
FROM sample_words
WHERE id IN (1, 2, 3, 13, 14);
GO

-- Handling acronyms and special cases
-- Shows how the functions handle uppercase sequences
SELECT
    original_text,
    description,
    TO_SNAKE_CASE(original_text) as to_snake,
    TO_CAMEL_CASE(original_text) as to_camel
FROM sample_words
WHERE original_text LIKE '%HTTP%'
   OR original_text LIKE '%XML%'
   OR original_text LIKE '%API%'
   OR original_text LIKE '%HTML%';
GO

-- Converting from snake_case (common in databases) to other formats
-- Typical use case for API responses
SELECT
    original_text as database_column,
    TO_CAMEL_CASE(original_text) as json_field,
    TO_PASCAL_CASE(original_text) as class_property,
    TO_KEBAB_CASE(original_text) as url_slug
FROM sample_words
WHERE original_text LIKE '%_%';
GO

-- Converting from camelCase (common in JavaScript) to other formats
-- Useful for database migrations from NoSQL to SQL
SELECT
    original_text as js_field,
    TO_SNAKE_CASE(original_text) as sql_column,
    TO_CONSTANT_CASE(original_text) as env_variable,
    TO_KEBAB_CASE(original_text) as css_class
FROM sample_words
WHERE original_text IN ('userId', 'isActive');
GO

-- Practical example: Generating consistent API field names
-- Convert all different styles to a consistent camelCase format
SELECT
    original_text as mixed_input,
    TO_CAMEL_CASE(original_text) as api_output,
    CASE
        WHEN original_text != TO_CAMEL_CASE(original_text)
        THEN 'converted'
        ELSE 'unchanged'
    END as status
FROM sample_words
WHERE id <= 10;
GO

-- Practical example: Creating environment variable names
-- Convert various formats to CONSTANT_CASE for config files
SELECT
    original_text,
    TO_CONSTANT_CASE(original_text) as env_var_name,
    TEXTJOIN('', 1, 'export ', TO_CONSTANT_CASE(original_text), '="${',
           TO_CONSTANT_CASE(original_text), '}"') as bash_export
FROM sample_words
WHERE id <= 5;
GO

-- Edge cases and special handling
-- Demonstrating how numbers and special characters are handled
SELECT
    'v2API' as input,
    TO_SNAKE_CASE('v2API') as snake,
    TO_CAMEL_CASE('api_v2') as camel,
    TO_KEBAB_CASE('APIVersion2') as kebab;
GO

-- Chaining conversions - converting between any two formats
-- Example: kebab-case -> snake_case -> PascalCase
SELECT
    original_text as step1_original,
    TO_SNAKE_CASE(original_text) as step2_to_snake,
    TO_PASCAL_CASE(TO_SNAKE_CASE(original_text)) as step3_to_pascal
FROM sample_words
WHERE original_text LIKE '%-%'
LIMIT 5;
GO
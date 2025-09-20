-- ============================================================================
-- Base Conversion Functions Demonstration
-- ============================================================================
-- This example demonstrates SQL CLI's base conversion functions for converting
-- numbers between different bases (binary, octal, hexadecimal, and custom bases)
-- ============================================================================
-- Run: ./target/release/sql-cli -f examples/base_conversions.sql -o table
-- ============================================================================

-- Example 1: Convert decimal numbers to binary, hex, and octal
SELECT
    42 as decimal,
    TO_BINARY(42) as binary,
    TO_HEX(42) as hexadecimal,
    TO_OCTAL(42) as octal;
GO

-- Example 2: RGB color conversion - Convert RGB values to hex color codes
SELECT
    255 as red,
    165 as green,
    0 as blue,
    '#' || TO_HEX(255) || TO_HEX(165) || TO_HEX(0) as hex_color;
GO

-- Example 3: Powers of 2 in different bases
WITH powers AS (
    SELECT POW(2, value) as decimal
    FROM RANGE(0, 16)
)
SELECT
    decimal,
    TO_BINARY(decimal) as binary,
    TO_HEX(decimal) as hex,
    LENGTH(TO_BINARY(decimal)) as bit_count
FROM powers
ORDER BY decimal;
GO

-- Example 4: Convert from various bases back to decimal
SELECT
    '11111111' as binary_str,
    FROM_BINARY('11111111') as from_binary,
    'FF' as hex_str,
    FROM_HEX('FF') as from_hex,
    '377' as octal_str,
    FROM_OCTAL('377') as from_octal;
GO

-- Example 5: Base 36 - using all digits and letters
-- Base 36 uses 0-9 and a-z
SELECT
    TO_BASE(35, 36) as base36_35,  -- Should be 'z'
    TO_BASE(36, 36) as base36_36,  -- Should be '10'
    TO_BASE(1295, 36) as base36_1295,  -- Should be 'zz'
    FROM_BASE('hello', 36) as hello_as_number,
    FROM_BASE('sql', 36) as sql_as_number;
GO

-- Example 6: IP Address as integer conversions (simplified)
-- Convert IP octets to a single integer
SELECT
    '192.168.1.1' as ip_address,
    192 * 256 * 256 * 256 + 168 * 256 * 256 + 1 * 256 + 1 as ip_as_int,
    TO_HEX(192 * 256 * 256 * 256 + 168 * 256 * 256 + 1 * 256 + 1) as ip_as_hex;
GO

-- Example 7: Binary flags representation
-- Common permission flags: Read=4, Write=2, Execute=1
SELECT
    7 as permission_decimal,
    TO_BINARY(7) as permission_binary,
    'rwx' as meaning,
    '---' as separator,
    6 as permission_decimal2,
    TO_BINARY(6) as permission_binary2,
    'rw-' as meaning2;
GO

-- Example 8: Fun with large numbers in different bases
SELECT
    1000000 as million,
    TO_BINARY(1000000) as million_binary,
    LENGTH(TO_BINARY(1000000)) as bits_needed,
    TO_HEX(1000000) as million_hex,
    TO_BASE(1000000, 36) as million_base36;
GO

-- Example 9: ASCII values and hexadecimal
SELECT
    'A' as character,
    65 as ascii_value,
    TO_HEX(65) as hex_value,
    TO_BINARY(65) as binary_value,
    '---' as separator,
    'Z' as character2,
    90 as ascii_value2,
    TO_HEX(90) as hex_value2,
    TO_BINARY(90) as binary_value2;
GO

-- Example 10: Create a binary counter
WITH counter AS (
    SELECT value as num
    FROM RANGE(0, 16)
)
SELECT
    num,
    LPAD(TO_BINARY(num), 4, '0') as binary_4bit,
    TO_HEX(num) as hex,
    CASE
        WHEN num % 2 = 0 THEN 'even'
        ELSE 'odd'
    END as parity
FROM counter
ORDER BY num;
GO
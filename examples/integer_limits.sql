-- ============================================================================
-- Integer Type Limits Demonstration
-- ============================================================================
-- This example demonstrates SQL CLI's integer limit functions that provide
-- minimum and maximum values for various integer types (similar to C#/C++)
-- ============================================================================
-- Run: ./target/release/sql-cli -f examples/integer_limits.sql -o table
-- ============================================================================

-- Example 1: 8-bit integer limits (byte/char)
SELECT
    'INT8' as type_name,
    INT8_MIN() as min_value,
    INT8_MAX() as max_value,
    UINT8_MAX() as unsigned_max,
    TO_BINARY(INT8_MAX()) as max_binary,
    TO_HEX(UINT8_MAX()) as unsigned_hex;
GO

-- Example 2: 16-bit integer limits (short)
SELECT
    'INT16' as type_name,
    INT16_MIN() as min_value,
    INT16_MAX() as max_value,
    UINT16_MAX() as unsigned_max,
    TO_HEX(UINT16_MAX()) as unsigned_hex;
GO

-- Example 3: 32-bit integer limits (int)
SELECT
    'INT32' as type_name,
    INT32_MIN() as min_value,
    INT32_MAX() as max_value,
    UINT32_MAX() as unsigned_max,
    TO_HEX(UINT32_MAX()) as unsigned_hex;
GO

-- Example 4: 64-bit integer limits (long)
SELECT
    'INT64' as type_name,
    INT64_MIN() as min_value,
    INT64_MAX() as max_value,
    TO_HEX(INT64_MAX()) as max_hex;
GO

-- Example 5: Common alias names
SELECT
    'Byte' as type_name,
    BYTE_MIN() as min_value,
    BYTE_MAX() as max_value,
    '---' as separator,
    'Char' as type_name2,
    CHAR_MIN() as min_value2,
    CHAR_MAX() as max_value2;
GO

-- Example 6: Type comparisons with bit counts
SELECT
    'INT8' as type_name,
    INT8_MIN() as min_val,
    INT8_MAX() as max_val,
    8 as bits,
    POW(2, 8) as total_values;
GO

-- Example 7: Demonstrating overflow boundaries
SELECT
    'INT32 Overflow Check' as test,
    INT32_MAX() as max_value,
    INT32_MAX() + 1 as overflow_result,
    INT32_MIN() as min_value,
    INT32_MIN() - 1 as underflow_result;
GO

-- Example 8: Using limits for data validation
SELECT
    127 as value,
    CASE
        WHEN 127 >= INT8_MIN() AND 127 <= INT8_MAX() THEN 'Valid INT8'
        WHEN 127 >= 0 AND 127 <= UINT8_MAX() THEN 'Valid UINT8 only'
        ELSE 'Out of 8-bit range'
    END as int8_validation,
    CASE
        WHEN 127 >= INT16_MIN() AND 127 <= INT16_MAX() THEN 'Yes'
        ELSE 'No'
    END as fits_in_int16;
GO

-- Example 9: Bit patterns at boundaries
SELECT
    'Boundary Patterns' as description,
    TO_BINARY(INT8_MAX()) as int8_max_binary,
    TO_BINARY(UINT8_MAX()) as uint8_max_binary,
    TO_BINARY(INT16_MAX()) as int16_max_binary,
    TO_BINARY(UINT16_MAX()) as uint16_max_binary;
GO

-- Example 10: Powers of 2 and type limits
WITH powers AS (
    SELECT POW(2, value) as power_of_2, value as exponent
    FROM RANGE(0, 32)
)
SELECT
    exponent,
    power_of_2,
    CASE
        WHEN power_of_2 <= UINT8_MAX() THEN 'UINT8'
        WHEN power_of_2 <= UINT16_MAX() THEN 'UINT16'
        WHEN power_of_2 <= UINT32_MAX() THEN 'UINT32'
        ELSE 'UINT64'
    END as min_type_needed,
    TO_HEX(power_of_2) as hex_value
FROM powers
WHERE power_of_2 IN (1, 128, 256, 32768, 65536, 2147483648)
ORDER BY exponent;
GO
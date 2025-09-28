-- Bitwise String Operations Examples
-- Demonstrates string-based bitwise operations for binary visualization

-- Basic bitwise AND operation on binary strings
SELECT
    '1101' as a,
    '1011' as b,
    BIT_AND_STR('1101', '1011') as and_result;
GO

-- Basic bitwise OR operation on binary strings
SELECT
    '1100' as a,
    '1010' as b,
    BIT_OR_STR('1100', '1010') as or_result;
GO

-- Basic bitwise XOR operation on binary strings
SELECT
    '1100' as a,
    '1010' as b,
    BIT_XOR_STR('1100', '1010') as xor_result;
GO

-- Bit flipping and NOT operations
SELECT
    '11011010' as original,
    BIT_NOT_STR('11011010') as not_result,
    BIT_FLIP('11011010') as flipped,
    BIT_COUNT('11011010') as ones_count;
GO

-- Hamming distance - count differing bits
SELECT
    '11011010' as string1,
    '10011110' as string2,
    HAMMING_DISTANCE('11011010', '10011110') as diff_bits;
GO

-- Bit rotation operations
SELECT
    '11011010' as original,
    BIT_ROTATE_LEFT('11011010', 1) as rotate_left_1,
    BIT_ROTATE_LEFT('11011010', 2) as rotate_left_2,
    BIT_ROTATE_RIGHT('11011010', 1) as rotate_right_1,
    BIT_ROTATE_RIGHT('11011010', 2) as rotate_right_2;
GO

-- Bit shifting operations (with zero fill)
WITH bits AS (
    SELECT '10000000' as original
)
SELECT
    original,
    BIT_SHIFT_RIGHT(original, 1) as shift_r1,
    BIT_SHIFT_RIGHT(original, 2) as shift_r2,
    BIT_SHIFT_RIGHT(original, 3) as shift_r3,
    BIT_SHIFT_LEFT(original, 1) as shift_l1,
    BIT_SHIFT_LEFT(original, 2) as shift_l2
FROM bits;
GO

-- Visualize powers of 2 with bit shifting
SELECT
    value as n,
    POW(2, value) AS pow2,
    LPAD(TO_BINARY(POW(2, value)), 16, '0') AS binary,
    BIT_COUNT(TO_BINARY(POW(2, value))) as ones
FROM RANGE(0, 15);
GO

-- XOR pattern visualization
SELECT
    value as a,
    value + 7 as b,
    LPAD(TO_BINARY(value), 5, '0') as bin_a,
    LPAD(TO_BINARY(value + 7), 5, '0') as bin_b,
    LPAD(BIT_XOR_STR(TO_BINARY(value), TO_BINARY(value + 7)), 5, '0') as xor_result,
    HAMMING_DISTANCE(TO_BINARY(value), TO_BINARY(value + 7)) as diff_bits
FROM RANGE(1, 10);
GO

-- Complex bit manipulation - Gray code generation
-- Gray code: each successive value differs in only one bit
WITH nums AS (
    SELECT value as n FROM RANGE(0, 15)
)
SELECT
    n,
    LPAD(TO_BINARY(n), 4, '0') as binary,
    LPAD(BIT_XOR_STR(TO_BINARY(n), BIT_SHIFT_RIGHT(TO_BINARY(n), 1)), 4, '0') as gray_code
FROM nums;
GO

-- Bit masking example - extract specific bits
WITH data AS (
    SELECT '11011010' as value,
           '11110000' as mask1,  -- High nibble mask
           '00001111' as mask2   -- Low nibble mask
)
SELECT
    value,
    mask1,
    BIT_AND_STR(value, mask1) as high_nibble,
    mask2,
    BIT_AND_STR(value, mask2) as low_nibble
FROM data;
GO

-- Parity bit calculation for various byte patterns
WITH bytes AS (
    SELECT
        CASE value
            WHEN 0 THEN '10110011'
            WHEN 1 THEN '11001100'
            WHEN 2 THEN '10101010'
            WHEN 3 THEN '11111111'
            WHEN 4 THEN '00000000'
        END as byte_value
    FROM RANGE(0, 5)
)
SELECT
    byte_value,
    BIT_COUNT(byte_value) as ones,
    CASE
        WHEN BIT_COUNT(byte_value) % 2 = 0 THEN 'even'
        ELSE 'odd'
    END as parity
FROM bytes;
GO

-- Demonstrate all operations on same input
WITH input AS (
    SELECT '10110' as a, '11001' as b
)
SELECT
    a, b,
    BIT_AND_STR(a, b) as and_op,
    BIT_OR_STR(a, b) as or_op,
    BIT_XOR_STR(a, b) as xor_op,
    BIT_NOT_STR(a) as not_a,
    HAMMING_DISTANCE(a, b) as dist
FROM input;
GO
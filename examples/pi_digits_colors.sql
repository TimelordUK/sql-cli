-- Pi Digits Rainbow Visualization
-- ============================================================================
-- Showcases the digits of Pi with each digit (0-9) having its own vibrant color
-- Demonstrates ANSI_RGB color functions with the PI_DIGIT() function
--
-- Color mapping for digits:
--   0 - Deep Purple   (138, 43, 226)
--   1 - Red          (255, 50, 50)
--   2 - Orange       (255, 140, 0)
--   3 - Gold         (255, 215, 0)
--   4 - Yellow-Green (154, 205, 50)
--   5 - Green        (50, 205, 50)
--   6 - Cyan         (0, 206, 209)
--   7 - Blue         (65, 105, 225)
--   8 - Magenta      (255, 0, 255)
--   9 - Hot Pink     (255, 105, 180)
-- ============================================================================

-- Demo 1: First 50 Digits of Pi with Rainbow Colors
-- Each digit is colored according to its value, creating a beautiful rainbow effect

WITH pi_digits AS (
  SELECT
    value as position,
    PI_DIGIT(value) as digit
  FROM RANGE(1, 50)
)
SELECT
  LPAD(TO_STRING(position), 3, ' ') as Pos,
  CASE digit
    WHEN 0 THEN ANSI_BOLD(ANSI_RGB(138, 43, 226, TO_STRING(digit)))
    WHEN 1 THEN ANSI_BOLD(ANSI_RGB(255, 50, 50, TO_STRING(digit)))
    WHEN 2 THEN ANSI_BOLD(ANSI_RGB(255, 140, 0, TO_STRING(digit)))
    WHEN 3 THEN ANSI_BOLD(ANSI_RGB(255, 215, 0, TO_STRING(digit)))
    WHEN 4 THEN ANSI_BOLD(ANSI_RGB(154, 205, 50, TO_STRING(digit)))
    WHEN 5 THEN ANSI_BOLD(ANSI_RGB(50, 205, 50, TO_STRING(digit)))
    WHEN 6 THEN ANSI_BOLD(ANSI_RGB(0, 206, 209, TO_STRING(digit)))
    WHEN 7 THEN ANSI_BOLD(ANSI_RGB(65, 105, 225, TO_STRING(digit)))
    WHEN 8 THEN ANSI_BOLD(ANSI_RGB(255, 0, 255, TO_STRING(digit)))
    WHEN 9 THEN ANSI_BOLD(ANSI_RGB(255, 105, 180, TO_STRING(digit)))
  END as Digit,
  CASE digit
    WHEN 0 THEN ANSI_RGB(138, 43, 226, REPEAT('●', 1))
    WHEN 1 THEN ANSI_RGB(255, 50, 50, REPEAT('●', 1))
    WHEN 2 THEN ANSI_RGB(255, 140, 0, REPEAT('●', 2))
    WHEN 3 THEN ANSI_RGB(255, 215, 0, REPEAT('●', 3))
    WHEN 4 THEN ANSI_RGB(154, 205, 50, REPEAT('●', 4))
    WHEN 5 THEN ANSI_RGB(50, 205, 50, REPEAT('●', 5))
    WHEN 6 THEN ANSI_RGB(0, 206, 209, REPEAT('●', 6))
    WHEN 7 THEN ANSI_RGB(65, 105, 225, REPEAT('●', 7))
    WHEN 8 THEN ANSI_RGB(255, 0, 255, REPEAT('●', 8))
    WHEN 9 THEN ANSI_RGB(255, 105, 180, REPEAT('●', 9))
  END as Visual
FROM pi_digits;
GO

-- Demo 2: Pi Digits Frequency Analysis (First 100 Digits)
-- Shows how often each digit appears in the first 100 digits of Pi

WITH pi_100 AS (
  SELECT PI_DIGIT(value) as digit
  FROM RANGE(1, 100)
),
digit_counts AS (
  SELECT
    digit as digit_value,
    COUNT(*) as digit_count
  FROM pi_100
  GROUP BY digit
)
SELECT
  digit_value as digit_raw,
  CASE digit_value
    WHEN 0 THEN ANSI_BOLD(ANSI_RGB(138, 43, 226, TO_STRING(digit_value)))
    WHEN 1 THEN ANSI_BOLD(ANSI_RGB(255, 50, 50, TO_STRING(digit_value)))
    WHEN 2 THEN ANSI_BOLD(ANSI_RGB(255, 140, 0, TO_STRING(digit_value)))
    WHEN 3 THEN ANSI_BOLD(ANSI_RGB(255, 215, 0, TO_STRING(digit_value)))
    WHEN 4 THEN ANSI_BOLD(ANSI_RGB(154, 205, 50, TO_STRING(digit_value)))
    WHEN 5 THEN ANSI_BOLD(ANSI_RGB(50, 205, 50, TO_STRING(digit_value)))
    WHEN 6 THEN ANSI_BOLD(ANSI_RGB(0, 206, 209, TO_STRING(digit_value)))
    WHEN 7 THEN ANSI_BOLD(ANSI_RGB(65, 105, 225, TO_STRING(digit_value)))
    WHEN 8 THEN ANSI_BOLD(ANSI_RGB(255, 0, 255, TO_STRING(digit_value)))
    WHEN 9 THEN ANSI_BOLD(ANSI_RGB(255, 105, 180, TO_STRING(digit_value)))
  END as Digit,
  ANSI_RGB(100, 200, 100, TO_STRING(digit_count)) as Count,
  CASE digit_value
    WHEN 0 THEN ANSI_RGB(138, 43, 226, REPEAT('#', digit_count))
    WHEN 1 THEN ANSI_RGB(255, 50, 50, REPEAT('#', digit_count))
    WHEN 2 THEN ANSI_RGB(255, 140, 0, REPEAT('#', digit_count))
    WHEN 3 THEN ANSI_RGB(255, 215, 0, REPEAT('#', digit_count))
    WHEN 4 THEN ANSI_RGB(154, 205, 50, REPEAT('#', digit_count))
    WHEN 5 THEN ANSI_RGB(50, 205, 50, REPEAT('#', digit_count))
    WHEN 6 THEN ANSI_RGB(0, 206, 209, REPEAT('#', digit_count))
    WHEN 7 THEN ANSI_RGB(65, 105, 225, REPEAT('#', digit_count))
    WHEN 8 THEN ANSI_RGB(255, 0, 255, REPEAT('#', digit_count))
    WHEN 9 THEN ANSI_RGB(255, 105, 180, REPEAT('#', digit_count))
  END as Frequency_Bar
FROM digit_counts
ORDER BY digit_raw;
GO

-- Demo 3: Compact Pi Rainbow - First 100 digits in flowing format
-- Shows Pi digits in a compact, flowing rainbow of colors

WITH pi_100 AS (
  SELECT
    value as position,
    PI_DIGIT(value) as digit
  FROM RANGE(1, 100)
)
SELECT
  CASE digit
    WHEN 0 THEN ANSI_BOLD(ANSI_RGB(138, 43, 226, TO_STRING(digit)))
    WHEN 1 THEN ANSI_BOLD(ANSI_RGB(255, 50, 50, TO_STRING(digit)))
    WHEN 2 THEN ANSI_BOLD(ANSI_RGB(255, 140, 0, TO_STRING(digit)))
    WHEN 3 THEN ANSI_BOLD(ANSI_RGB(255, 215, 0, TO_STRING(digit)))
    WHEN 4 THEN ANSI_BOLD(ANSI_RGB(154, 205, 50, TO_STRING(digit)))
    WHEN 5 THEN ANSI_BOLD(ANSI_RGB(50, 205, 50, TO_STRING(digit)))
    WHEN 6 THEN ANSI_BOLD(ANSI_RGB(0, 206, 209, TO_STRING(digit)))
    WHEN 7 THEN ANSI_BOLD(ANSI_RGB(65, 105, 225, TO_STRING(digit)))
    WHEN 8 THEN ANSI_BOLD(ANSI_RGB(255, 0, 255, TO_STRING(digit)))
    WHEN 9 THEN ANSI_BOLD(ANSI_RGB(255, 105, 180, TO_STRING(digit)))
  END as Pi_Digit_Rainbow
FROM pi_100;
GO

-- Demo 4: Pi Digit Patterns - Looking for Consecutive Sequences
-- Finds and highlights interesting patterns like repeated or consecutive digits

WITH pi_200 AS (
  SELECT
    value as pos,
    PI_DIGIT(value) as digit,
    PI_DIGIT(value + 1) as next_digit
  FROM RANGE(1, 200)
),
patterns AS (
  SELECT
    pos,
    digit,
    next_digit,
    CASE
      WHEN digit = next_digit THEN 'REPEAT'
      WHEN (digit + 1) % 10 = next_digit THEN 'CONSECUTIVE'
      ELSE 'NORMAL'
    END as pattern_type
  FROM pi_200
)
SELECT
  LPAD(TO_STRING(pos), 3, ' ') as Position,
  CASE digit
    WHEN 0 THEN ANSI_BOLD(ANSI_RGB(138, 43, 226, TO_STRING(digit)))
    WHEN 1 THEN ANSI_BOLD(ANSI_RGB(255, 50, 50, TO_STRING(digit)))
    WHEN 2 THEN ANSI_BOLD(ANSI_RGB(255, 140, 0, TO_STRING(digit)))
    WHEN 3 THEN ANSI_BOLD(ANSI_RGB(255, 215, 0, TO_STRING(digit)))
    WHEN 4 THEN ANSI_BOLD(ANSI_RGB(154, 205, 50, TO_STRING(digit)))
    WHEN 5 THEN ANSI_BOLD(ANSI_RGB(50, 205, 50, TO_STRING(digit)))
    WHEN 6 THEN ANSI_BOLD(ANSI_RGB(0, 206, 209, TO_STRING(digit)))
    WHEN 7 THEN ANSI_BOLD(ANSI_RGB(65, 105, 225, TO_STRING(digit)))
    WHEN 8 THEN ANSI_BOLD(ANSI_RGB(255, 0, 255, TO_STRING(digit)))
    WHEN 9 THEN ANSI_BOLD(ANSI_RGB(255, 105, 180, TO_STRING(digit)))
  END as Digit_1,
  CASE next_digit
    WHEN 0 THEN ANSI_BOLD(ANSI_RGB(138, 43, 226, TO_STRING(next_digit)))
    WHEN 1 THEN ANSI_BOLD(ANSI_RGB(255, 50, 50, TO_STRING(next_digit)))
    WHEN 2 THEN ANSI_BOLD(ANSI_RGB(255, 140, 0, TO_STRING(next_digit)))
    WHEN 3 THEN ANSI_BOLD(ANSI_RGB(255, 215, 0, TO_STRING(next_digit)))
    WHEN 4 THEN ANSI_BOLD(ANSI_RGB(154, 205, 50, TO_STRING(next_digit)))
    WHEN 5 THEN ANSI_BOLD(ANSI_RGB(50, 205, 50, TO_STRING(next_digit)))
    WHEN 6 THEN ANSI_BOLD(ANSI_RGB(0, 206, 209, TO_STRING(next_digit)))
    WHEN 7 THEN ANSI_BOLD(ANSI_RGB(65, 105, 225, TO_STRING(next_digit)))
    WHEN 8 THEN ANSI_BOLD(ANSI_RGB(255, 0, 255, TO_STRING(next_digit)))
    WHEN 9 THEN ANSI_BOLD(ANSI_RGB(255, 105, 180, TO_STRING(next_digit)))
  END as Digit_2,
  CASE pattern_type
    WHEN 'REPEAT' THEN ANSI_RGB(255, 100, 100, '⚡ REPEAT')
    WHEN 'CONSECUTIVE' THEN ANSI_RGB(100, 255, 100, '→ CONSEC')
    ELSE ''
  END as Pattern
FROM patterns
WHERE pattern_type != 'NORMAL';
GO

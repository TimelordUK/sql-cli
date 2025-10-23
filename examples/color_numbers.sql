-- #! ../data/numbers_1_100.csv

-- ============================================================================
-- COLORFUL NUMBER CLASSIFICATION DEMOS
-- ============================================================================
-- These queries showcase ANSI color functions to create visually attractive
-- displays of number properties: primes, twin primes, perfect squares, etc.
--
-- Color scheme:
--   RED (bold)    - Twin Primes (primes that differ by 2)
--   GOLD          - Regular Primes
--   CYAN          - Perfect Squares (1, 4, 9, 16, 25, ...)
--   GRAY          - Even Numbers
--   WHITE         - Odd Composites
-- ============================================================================

-- Demo 1: Comprehensive Number Classification Table
-- Shows all numbers with special properties highlighted in vibrant colors

WITH properties AS (
  SELECT
    n,
    IS_PRIME(n) as is_prime,
    IS_PRIME(n - 2) as lower_twin,
    IS_PRIME(n + 2) as upper_twin,
    FLOOR(SQRT(n)) * FLOOR(SQRT(n)) = n as is_square,
    n % 2 = 0 as is_even
  FROM numbers_1_100
),
classified AS (
  SELECT
    n,
    CASE
      WHEN is_prime = 1 AND (lower_twin = 1 OR upper_twin = 1) THEN 'twin_prime'
      WHEN is_prime = 1 THEN 'prime'
      WHEN is_square = 1 THEN 'square'
      WHEN is_even = 1 THEN 'even'
      ELSE 'odd_composite'
    END as classification
  FROM properties
)
SELECT
  CASE classification
    WHEN 'twin_prime' THEN ANSI_BOLD(ANSI_RGB(255, 50, 50, LPAD(TO_STRING(n), 4, ' ')))
    WHEN 'prime' THEN ANSI_RGB(255, 215, 0, LPAD(TO_STRING(n), 4, ' '))
    WHEN 'square' THEN ANSI_RGB(100, 200, 255, LPAD(TO_STRING(n), 4, ' '))
    WHEN 'even' THEN ANSI_RGB(150, 150, 150, LPAD(TO_STRING(n), 4, ' '))
    ELSE LPAD(TO_STRING(n), 4, ' ')
  END as Number,
  CASE classification
    WHEN 'twin_prime' THEN ANSI_RGB(255, 50, 50, '***')
    WHEN 'prime' THEN ANSI_RGB(255, 215, 0, '**')
    WHEN 'square' THEN ANSI_RGB(100, 200, 255, '#')
    WHEN 'even' THEN ANSI_RGB(150, 150, 150, '.')
    ELSE ' '
  END as Visual,
  CASE classification
    WHEN 'twin_prime' THEN ANSI_RGB(255, 50, 50, 'Twin Prime')
    WHEN 'prime' THEN ANSI_RGB(255, 215, 0, 'Prime')
    WHEN 'square' THEN ANSI_RGB(100, 200, 255, 'Perfect Square')
    WHEN 'even' THEN ANSI_RGB(150, 150, 150, 'Even')
    ELSE 'Odd Composite'
  END as Category
FROM classified
WHERE classification IN ('twin_prime', 'prime', 'square');
GO

-- Demo 2: Twin Primes Showcase
-- Highlights pairs of primes that differ by 2 (e.g., 3&5, 5&7, 11&13)

WITH twin_primes AS (
  SELECT
    n as prime1,
    n + 2 as prime2
  FROM numbers_1_100
  WHERE IS_PRIME(n) = 1 AND IS_PRIME(n + 2) = 1
)
SELECT
  ANSI_BOLD(ANSI_RGB(255, 50, 50, '(' || TO_STRING(prime1) || ', ' || TO_STRING(prime2) || ')')) as TwinPrimePair,
  ANSI_RGB(255, 165, 0, TO_STRING(prime2 - prime1)) as Gap,
  ANSI_RGB(100, 255, 100, TO_STRING(prime1 + prime2)) as Sum,
  ANSI_RGB(150, 150, 255, TO_STRING((prime1 + prime2) / 2.0)) as Average
FROM twin_primes;
GO

-- Demo 3: Prime Density Visualization
-- Shows how many primes exist in each decade (1-10, 11-20, etc.)

WITH decades AS (
  SELECT
    FLOOR((n - 1) / 10) * 10 + 1 as decade_start,
    FLOOR((n - 1) / 10) * 10 + 10 as decade_end,
    n,
    IS_PRIME(n) as is_prime
  FROM numbers_1_100
),
prime_counts AS (
  SELECT
    decade_start,
    decade_end,
    SUM(CASE WHEN is_prime = 1 THEN 1 ELSE 0 END) as prime_count
  FROM decades
  GROUP BY decade_start, decade_end
)
SELECT
  ANSI_COLOR('cyan', TO_STRING(decade_start) || '-' || TO_STRING(decade_end)) as Range,
  ANSI_BOLD(ANSI_RGB(255, 215, 0, TO_STRING(prime_count))) as Primes,
  ANSI_RGB(
    255 - (prime_count * 50),
    50 + (prime_count * 40),
    100,
    REPEAT('#', prime_count)
  ) as VisualBar
FROM prime_counts;
GO

-- Demo 4: Perfect Squares with Colors
-- Beautiful display of perfect squares up to 100

SELECT
  ANSI_RGB(100, 200, 255, LPAD(TO_STRING(n), 4, ' ')) as Number,
  ANSI_RGB(255, 150, 255, LPAD(TO_STRING(SQRT(n)), 4, ' ')) as Root,
  ANSI_BOLD(ANSI_RGB(100, 200, 255, '#')) as Square,
  ANSI_RGB(200, 200, 200, REPEAT('.', SQRT(n))) as Visual
FROM numbers_1_100
WHERE FLOOR(SQRT(n)) * FLOOR(SQRT(n)) = n;
GO

-- Demo 5: Number Rainbow - All Numbers 1-100 Color-Coded
-- Every number colored based on its primary property

WITH properties AS (
  SELECT
    n,
    IS_PRIME(n) as is_prime,
    IS_PRIME(n - 2) as lower_twin,
    IS_PRIME(n + 2) as upper_twin,
    FLOOR(SQRT(n)) * FLOOR(SQRT(n)) = n as is_square,
    n % 2 = 0 as is_even
  FROM numbers_1_100
)
SELECT
  CASE
    WHEN is_prime = 1 AND (lower_twin = 1 OR upper_twin = 1) THEN ANSI_BOLD(ANSI_RGB(255, 50, 50, RPAD(TO_STRING(n), 5, ' ')))
    WHEN is_prime = 1 THEN ANSI_RGB(255, 215, 0, RPAD(TO_STRING(n), 5, ' '))
    WHEN is_square = 1 THEN ANSI_RGB(100, 200, 255, RPAD(TO_STRING(n), 5, ' '))
    WHEN is_even = 1 THEN ANSI_RGB(150, 150, 150, RPAD(TO_STRING(n), 5, ' '))
    ELSE RPAD(TO_STRING(n), 5, ' ')
  END as ColoredNumbers
FROM properties;
GO

-- Demo 6: Number Properties Matrix
-- Compact view showing multiple properties per number

WITH properties AS (
  SELECT
    n,
    IS_PRIME(n) as is_prime,
    IS_PRIME(n - 2) as lower_twin,
    IS_PRIME(n + 2) as upper_twin,
    FLOOR(SQRT(n)) * FLOOR(SQRT(n)) = n as is_square,
    n % 3 = 0 as div_by_3,
    n % 5 = 0 as div_by_5
  FROM numbers_1_100
  WHERE n <= 30
)
SELECT
  LPAD(TO_STRING(n), 3, ' ') as n,
  CASE WHEN is_prime = 1 AND (lower_twin = 1 OR upper_twin = 1) THEN ANSI_RGB(255, 50, 50, 'X') ELSE ' ' END as Twin,
  CASE WHEN is_prime = 1 THEN ANSI_RGB(255, 215, 0, 'X') ELSE ' ' END as Prime,
  CASE WHEN is_square = 1 THEN ANSI_RGB(100, 200, 255, 'X') ELSE ' ' END as Square,
  CASE WHEN div_by_3 = 1 THEN ANSI_RGB(150, 255, 150, 'X') ELSE ' ' END as Div3,
  CASE WHEN div_by_5 = 1 THEN ANSI_RGB(255, 200, 150, 'X') ELSE ' ' END as Div5
FROM properties;
GO

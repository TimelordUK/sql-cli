-- #! ../data/angles.csv
-- ============================================================================
-- Trigonometric Functions Demonstration
-- ============================================================================
-- This example demonstrates SQL CLI's trigonometric functions including:
-- SIN, COS, TAN, ASIN, ACOS, ATAN, ATAN2, SINH, COSH, TANH
-- DEGREES, RADIANS, and mathematical identities
-- ============================================================================
-- Run: ./target/release/sql-cli data/angles.csv -f examples/trig.sql -o table
-- ============================================================================

-- Example 1: Basic trigonometric functions for common angles
SELECT
    degrees,
    ROUND(RADIANS(degrees), 4) as radians,
    ROUND(SIN(RADIANS(degrees)), 4) as sine,
    ROUND(COS(RADIANS(degrees)), 4) as cosine,
    ROUND(TAN(RADIANS(degrees)), 4) as tangent
FROM angles
WHERE degrees IN (0, 30, 45, 60, 180, 360)
ORDER BY degrees;
GO

-- Example 2: Prove the Pythagorean identity: sin²θ + cos²θ = 1
SELECT
    degrees,
    ROUND(SIN(RADIANS(degrees)), 4) as sine,
    ROUND(COS(RADIANS(degrees)), 4) as cosine,
    ROUND(
        SIN(RADIANS(degrees)) * SIN(RADIANS(degrees)) +
        COS(RADIANS(degrees)) * COS(RADIANS(degrees)),
        10
    ) as sin_sq_plus_cos_sq
FROM angles
ORDER BY degrees;
GO

-- Example 3: Inverse trigonometric functions
-- Using specific values to demonstrate inverse functions
SELECT
    0.0 as x,
    ROUND(DEGREES(ASIN(0.0)), 2) as asin_degrees,
    ROUND(DEGREES(ACOS(0.0)), 2) as acos_degrees,
    ROUND(DEGREES(ATAN(0.0)), 2) as atan_degrees;
GO

SELECT
    0.5 as x,
    ROUND(DEGREES(ASIN(0.5)), 2) as asin_degrees,
    ROUND(DEGREES(ACOS(0.5)), 2) as acos_degrees,
    ROUND(DEGREES(ATAN(0.5)), 2) as atan_degrees;
GO

SELECT
    1.0 as x,
    ROUND(DEGREES(ASIN(1.0)), 2) as asin_degrees,
    ROUND(DEGREES(ACOS(1.0)), 2) as acos_degrees,
    ROUND(DEGREES(ATAN(1.0)), 2) as atan_degrees;
GO

-- Example 4: Hyperbolic functions
WITH x_values AS (
    SELECT value as x
    FROM RANGE(-2, 3)
)
SELECT
    x,
    ROUND(SINH(x), 4) as sinh_x,
    ROUND(COSH(x), 4) as cosh_x,
    ROUND(TANH(x), 4) as tanh_x,
    -- Verify identity: cosh²(x) - sinh²(x) = 1
    ROUND(COSH(x) * COSH(x) - SINH(x) * SINH(x), 10) as cosh_sq_minus_sinh_sq
FROM x_values
ORDER BY x;
GO

-- Example 5: ATAN2 for quadrant-aware angle calculation
-- Calculate angles of points on unit circle
WITH circle_points AS (
    SELECT
        description,
        COS(RADIANS(degrees)) as x,
        SIN(RADIANS(degrees)) as y,
        degrees as expected_degrees
    FROM angles
    WHERE degrees IN (0, 45, 90, 135, 180, 225, 270, 315)
)
SELECT
    description,
    ROUND(x, 3) as x_coord,
    ROUND(y, 3) as y_coord,
    expected_degrees,
    ROUND(DEGREES(ATAN2(y, x)), 1) as calculated_degrees
FROM circle_points
ORDER BY expected_degrees;
GO

-- Example 6: Demonstrate angle sum formulas
-- sin(A + B) = sin(A)cos(B) + cos(A)sin(B)
-- Testing with 30+60=90 degrees
SELECT
    30 as angle_a,
    60 as angle_b,
    30 + 60 as sum_degrees,
    -- Direct calculation
    ROUND(SIN(RADIANS(30 + 60)), 4) as sin_sum_direct,
    -- Using sum formula
    ROUND(
        SIN(RADIANS(30)) * COS(RADIANS(60)) +
        COS(RADIANS(30)) * SIN(RADIANS(60)),
        4
    ) as sin_sum_formula,
    -- Verify they match
    CASE
        WHEN ABS(
            SIN(RADIANS(30 + 60)) -
            (SIN(RADIANS(30)) * COS(RADIANS(60)) +
             COS(RADIANS(30)) * SIN(RADIANS(60)))
        ) < 0.0001 THEN 'VERIFIED'
        ELSE 'MISMATCH'
    END as formula_check;
GO

-- Example 7: Create a sine wave at key angles
SELECT
    degrees,
    ROUND(SIN(RADIANS(degrees)), 3) as sine_value,
    ROUND(COS(RADIANS(degrees)), 3) as cosine_value
FROM angles
WHERE degrees % 30 = 0
ORDER BY degrees;
GO

-- Example 8: Demonstrate double angle formulas
-- cos(2θ) = cos²(θ) - sin²(θ) = 2cos²(θ) - 1 = 1 - 2sin²(θ)
SELECT
    degrees,
    -- Direct calculation of cos(2θ)
    ROUND(COS(RADIANS(2 * degrees)), 4) as cos_2theta_direct,
    -- Using cos²(θ) - sin²(θ)
    ROUND(
        COS(RADIANS(degrees)) * COS(RADIANS(degrees)) -
        SIN(RADIANS(degrees)) * SIN(RADIANS(degrees)),
        4
    ) as cos_2theta_formula1,
    -- Using 2cos²(θ) - 1
    ROUND(
        2 * COS(RADIANS(degrees)) * COS(RADIANS(degrees)) - 1,
        4
    ) as cos_2theta_formula2,
    -- Using 1 - 2sin²(θ)
    ROUND(
        1 - 2 * SIN(RADIANS(degrees)) * SIN(RADIANS(degrees)),
        4
    ) as cos_2theta_formula3
FROM angles
WHERE degrees IN (0, 30, 45, 60)
ORDER BY degrees;
GO
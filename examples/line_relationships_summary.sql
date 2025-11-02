-- LINE RELATIONSHIPS: What We Can Determine
-- Using vector math to analyze relationships between lines

-- =============================================================================
-- 1. PARALLEL LINES CHECK
-- =============================================================================
-- Two lines are parallel if their direction vectors are parallel
-- Test: Cross product magnitude = 0

SELECT '=== PARALLEL LINES ===' as test;
GO

-- Parallel lines (same direction)
SELECT
    'Parallel' as relationship,
    VEC(0,0) as line1_start,
    VEC(2,2) as line1_end,
    VEC(1,1) as line2_start,
    VEC(3,3) as line2_end,

    VEC_MAG(VEC_CROSS(
        VEC(2,2,0),  -- line1 direction
        VEC(2,2,0)   -- line2 direction (same!)
    )) as cross_mag,

    CASE
        WHEN VEC_MAG(VEC_CROSS(VEC(2,2,0), VEC(2,2,0))) < 0.0001
        THEN 'PARALLEL'
        ELSE 'NOT PARALLEL'
    END as result;
GO

-- =============================================================================
-- 2. INTERSECTING LINES CHECK
-- =============================================================================
-- Two lines intersect if they're NOT parallel (cross product ≠ 0)

SELECT '=== INTERSECTING LINES ===' as test;
GO

-- Classic X intersection
SELECT
    'Intersecting' as relationship,
    VEC(0,0) as line1_start,
    VEC(4,4) as line1_end,
    VEC(0,4) as line2_start,
    VEC(4,0) as line2_end,

    VEC_MAG(VEC_CROSS(
        VEC(4,4,0),   -- line1 direction
        VEC(4,-4,0)   -- line2 direction
    )) as cross_mag,

    CASE
        WHEN VEC_MAG(VEC_CROSS(VEC(4,4,0), VEC(4,-4,0))) > 0.0001
        THEN 'INTERSECT'
        ELSE 'PARALLEL'
    END as result,

    -- Expected intersection point is (2,2) for this case
    'Expected: (2,2)' as note;
GO

-- =============================================================================
-- 3. PERPENDICULAR LINES CHECK
-- =============================================================================
-- Two lines are perpendicular if dot product of directions = 0

SELECT '=== PERPENDICULAR LINES ===' as test;
GO

SELECT
    'Perpendicular' as relationship,
    VEC(1,0) as line1_dir,
    VEC(0,1) as line2_dir,

    VEC_DOT(VEC(1,0), VEC(0,1)) as dot_product,

    CASE
        WHEN ABS(VEC_DOT(VEC(1,0), VEC(0,1))) < 0.0001
        THEN 'PERPENDICULAR'
        ELSE 'NOT PERPENDICULAR'
    END as result,

    VEC_ANGLE(VEC(1,0), VEC(0,1)) * 180 / PI() as angle_degrees;
GO

-- =============================================================================
-- 4. ANGLE BETWEEN LINES
-- =============================================================================
-- Can determine the angle of intersection

SELECT '=== ANGLE BETWEEN LINES ===' as test;
GO

SELECT
    'Acute angle' as case_type,
    VEC(1,0) as line1_dir,
    VEC(1,1) as line2_dir,
    VEC_ANGLE(VEC(1,0), VEC(1,1)) * 180 / PI() as angle_deg;
GO

SELECT
    'Right angle' as case_type,
    VEC(1,0) as line1_dir,
    VEC(0,1) as line2_dir,
    VEC_ANGLE(VEC(1,0), VEC(0,1)) * 180 / PI() as angle_deg;
GO

SELECT
    'Obtuse angle' as case_type,
    VEC(1,0) as line1_dir,
    VEC(-1,1) as line2_dir,
    VEC_ANGLE(VEC(1,0), VEC(-1,1)) * 180 / PI() as angle_deg;
GO

-- =============================================================================
-- 5. DISTANCE FROM POINT TO LINE
-- =============================================================================
-- Can compute perpendicular distance from any point to a line

SELECT '=== POINT-TO-LINE DISTANCE ===' as test;
GO

SELECT
    VEC(2,2) as point,
    'y=0 (x-axis)' as line,
    VEC(0,0) as line_point,
    VEC(1,0) as line_dir,

    VEC_MAG(VEC_CROSS(
        VEC(2,2,0),
        VEC(1,0,0)
    )) / VEC_MAG(VEC(1,0,0)) as distance,

    'Expected: 2' as verification;
GO

-- =============================================================================
-- 6. COLLINEAR CHECK
-- =============================================================================
-- Three points are collinear if cross product of vectors between them = 0

SELECT '=== COLLINEAR POINTS ===' as test;
GO

SELECT
    'Collinear' as case_type,
    VEC(0,0) as p1,
    VEC(1,1) as p2,
    VEC(2,2) as p3,

    VEC_MAG(VEC_CROSS(
        VEC_SUB(VEC(1,1,0), VEC(0,0,0)),
        VEC_SUB(VEC(2,2,0), VEC(0,0,0))
    )) as cross_mag,

    CASE
        WHEN VEC_MAG(VEC_CROSS(
            VEC_SUB(VEC(1,1,0), VEC(0,0,0)),
            VEC_SUB(VEC(2,2,0), VEC(0,0,0))
        )) < 0.0001
        THEN 'COLLINEAR'
        ELSE 'NOT COLLINEAR'
    END as result;
GO

-- =============================================================================
-- SUMMARY: What we CAN determine
-- =============================================================================
-- ✓ Are lines parallel?
-- ✓ Are lines perpendicular?
-- ✓ Do lines intersect (in 2D/3D)?
-- ✓ What angle do lines make?
-- ✓ Distance from point to line
-- ✓ Are three points collinear?
-- ✓ Direction of lines
-- ✓ Relative orientation (cross product sign)

-- What would make this even more powerful:
-- • Computing exact intersection point (needs parametric solving)
-- • Closest point on line to another point
-- • Line-segment intersection (requires bounds checking)
-- • 3D skew lines distance

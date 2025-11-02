-- Line-Line Intersection Demonstration
-- Shows how to determine if two lines intersect using vector math

-- Example 1: Two intersecting lines in 2D
-- Line 1: from (0,0) to (4,4)
-- Line 2: from (0,4) to (4,0)
SELECT
    VEC(0,0) as line1_p1,
    VEC(4,4) as line1_p2,
    VEC(0,4) as line2_p1,
    VEC(4,0) as line2_p2,

    -- Direction vectors
    VEC_SUB(VEC(4,4), VEC(0,0)) as line1_dir,
    VEC_SUB(VEC(4,0), VEC(0,4)) as line2_dir,

    -- For 2D lines, use cross product with z=0 to check if parallel
    -- If cross product is 0, lines are parallel
    VEC_CROSS(
        VEC_SUB(VEC(4,4,0), VEC(0,0,0)),
        VEC_SUB(VEC(4,0,0), VEC(0,4,0))
    ) as cross_product,

    -- Magnitude of cross product (0 = parallel, non-zero = intersecting/skew)
    VEC_MAG(VEC_CROSS(
        VEC_SUB(VEC(4,4,0), VEC(0,0,0)),
        VEC_SUB(VEC(4,0,0), VEC(0,4,0))
    )) as parallel_check;
GO

-- Example 2: Parallel lines (should have cross product magnitude of 0)
-- Line 1: from (0,0) to (4,2)
-- Line 2: from (0,2) to (4,4) - same direction!
SELECT
    VEC(0,0) as line1_p1,
    VEC(4,2) as line1_p2,
    VEC(0,2) as line2_p1,
    VEC(4,4) as line2_p2,

    VEC_SUB(VEC(4,2), VEC(0,0)) as line1_dir,
    VEC_SUB(VEC(4,4), VEC(0,2)) as line2_dir,

    -- Check if parallel using 2D cross product (extend to 3D with z=0)
    VEC_CROSS(
        VEC_SUB(VEC(4,2,0), VEC(0,0,0)),
        VEC_SUB(VEC(4,4,0), VEC(0,2,0))
    ) as cross_product,

    VEC_MAG(VEC_CROSS(
        VEC_SUB(VEC(4,2,0), VEC(0,0,0)),
        VEC_SUB(VEC(4,4,0), VEC(0,2,0))
    )) as is_parallel;
GO

-- Example 3: Perpendicular lines (dot product of directions should be 0)
-- Line 1: horizontal (1,0)
-- Line 2: vertical (0,1)
SELECT
    VEC(1,0) as line1_dir,
    VEC(0,1) as line2_dir,

    VEC_DOT(VEC(1,0), VEC(0,1)) as dot_product,

    -- Angle should be 90 degrees (π/2 radians)
    VEC_ANGLE(VEC(1,0), VEC(0,1)) as angle_rad,
    VEC_ANGLE(VEC(1,0), VEC(0,1)) * 180 / PI() as angle_deg;
GO

-- Example 4: 3D line intersection check
-- Two lines in 3D space
SELECT
    VEC(0,0,0) as line1_p1,
    VEC(1,1,1) as line1_p2,
    VEC(0,1,0) as line2_p1,
    VEC(1,0,0) as line2_p2,

    -- Direction vectors
    VEC_SUB(VEC(1,1,1), VEC(0,0,0)) as line1_dir,
    VEC_SUB(VEC(1,0,0), VEC(0,1,0)) as line2_dir,

    -- Cross product (perpendicular to both)
    VEC_CROSS(
        VEC_SUB(VEC(1,1,1), VEC(0,0,0)),
        VEC_SUB(VEC(1,0,0), VEC(0,1,0))
    ) as perpendicular_vec,

    -- If magnitude is 0, lines are parallel
    VEC_MAG(VEC_CROSS(
        VEC_SUB(VEC(1,1,1), VEC(0,0,0)),
        VEC_SUB(VEC(1,0,0), VEC(0,1,0))
    )) as parallel_check;
GO

-- Example 5: Angle between two line segments
-- Useful for detecting sharp vs smooth intersections
SELECT
    VEC(1,0) as dir1,
    VEC(1,1) as dir2,

    -- Angle between lines
    VEC_ANGLE(VEC(1,0), VEC(1,1)) as angle_rad,
    VEC_ANGLE(VEC(1,0), VEC(1,1)) * 180 / PI() as angle_deg,

    -- Normalize first to ensure we're comparing directions
    VEC_ANGLE(
        VEC_NORMALIZE(VEC(1,0)),
        VEC_NORMALIZE(VEC(1,1))
    ) * 180 / PI() as normalized_angle;
GO

-- Example 6: Distance from point to line
-- Using the formula: distance = ||(p - a) × dir|| / ||dir||
-- where p is the point, a is a point on the line, dir is line direction
SELECT
    VEC(2,2) as point,
    VEC(0,0) as line_point,
    VEC(1,0) as line_dir,

    -- Vector from line point to target point
    VEC_SUB(VEC(2,2), VEC(0,0)) as to_point,

    -- Cross product (needs to be 3D)
    VEC_CROSS(
        VEC_SUB(VEC(2,2,0), VEC(0,0,0)),
        VEC(1,0,0)
    ) as cross_vec,

    -- Distance = magnitude of cross / magnitude of direction
    VEC_MAG(VEC_CROSS(
        VEC_SUB(VEC(2,2,0), VEC(0,0,0)),
        VEC(1,0,0)
    )) / VEC_MAG(VEC(1,0,0)) as distance_to_line;
GO

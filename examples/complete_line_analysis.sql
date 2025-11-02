-- COMPLETE LINE ANALYSIS TOOLKIT
-- Comprehensive demonstration of all line geometry functions

-- =============================================================================
-- 1. EXACT LINE INTERSECTION
-- =============================================================================
SELECT '=== LINE INTERSECTION (Infinite Lines) ===' as demo;
GO

-- Classic X intersection at (2,2)
SELECT
    'Intersecting lines' as case_name,
    VEC(0,0) as line1_p1,
    VEC(4,4) as line1_p2,
    VEC(0,4) as line2_p1,
    VEC(4,0) as line2_p2,
    LINE_INTERSECT(VEC(0,0), VEC(4,4), VEC(0,4), VEC(4,0)) as intersection_point;
GO

-- Parallel lines (no intersection)
SELECT
    'Parallel lines' as case_name,
    VEC(0,0) as line1_p1,
    VEC(2,2) as line1_p2,
    VEC(1,1) as line2_p1,
    VEC(3,3) as line2_p2,
    LINE_INTERSECT(VEC(0,0), VEC(2,2), VEC(1,1), VEC(3,3)) as intersection_point;
GO

-- Perpendicular lines intersecting at origin
SELECT
    'Perpendicular at origin' as case_name,
    VEC(-1,0) as line1_p1,
    VEC(1,0) as line1_p2,
    VEC(0,-1) as line2_p1,
    VEC(0,1) as line2_p2,
    LINE_INTERSECT(VEC(-1,0), VEC(1,0), VEC(0,-1), VEC(0,1)) as intersection_point;
GO

-- =============================================================================
-- 2. SEGMENT INTERSECTION (Bounded)
-- =============================================================================
SELECT '=== SEGMENT INTERSECTION (Bounded) ===' as demo;
GO

-- Segments that intersect
SELECT
    'Intersecting segments' as case_name,
    VEC(0,0) as seg1_start,
    VEC(2,2) as seg1_end,
    VEC(0,2) as seg2_start,
    VEC(2,0) as seg2_end,
    SEGMENT_INTERSECT(VEC(0,0), VEC(2,2), VEC(0,2), VEC(2,0)) as intersection;
GO

-- Segments that DON'T intersect (but lines would)
SELECT
    'Non-intersecting segments' as case_name,
    VEC(0,0) as seg1_start,
    VEC(1,1) as seg1_end,
    VEC(5,5) as seg2_start,
    VEC(6,6) as seg2_end,
    SEGMENT_INTERSECT(VEC(0,0), VEC(1,1), VEC(5,5), VEC(6,6)) as intersection,
    'Would intersect if extended' as note;
GO

-- T-junction: one segment touches middle of another
SELECT
    'T-junction' as case_name,
    VEC(0,1) as seg1_start,
    VEC(4,1) as seg1_end,
    VEC(2,0) as seg2_start,
    VEC(2,2) as seg2_end,
    SEGMENT_INTERSECT(VEC(0,1), VEC(4,1), VEC(2,0), VEC(2,2)) as intersection;
GO

-- =============================================================================
-- 3. CLOSEST POINT ON LINE (Projection)
-- =============================================================================
SELECT '=== CLOSEST POINT ON LINE ===' as demo;
GO

-- Point above x-axis: closest point should be directly below
SELECT
    'Point to x-axis' as case_name,
    VEC(2,2) as point,
    VEC(0,0) as line_point,
    VEC(1,0) as line_direction,
    CLOSEST_POINT_ON_LINE(VEC(2,2), VEC(0,0), VEC(1,0)) as closest_point,
    'Expected: (2,0)' as verification;
GO

-- Point to diagonal line
SELECT
    'Point to diagonal' as case_name,
    VEC(3,1) as point,
    VEC(0,0) as line_point,
    VEC(1,1) as line_direction,
    CLOSEST_POINT_ON_LINE(VEC(3,1), VEC(0,0), VEC(1,1)) as closest_point;
GO

-- 3D projection
SELECT
    '3D projection' as case_name,
    VEC(1,1,1) as point,
    VEC(0,0,0) as line_point,
    VEC(1,0,0) as line_direction,
    CLOSEST_POINT_ON_LINE(VEC(1,1,1), VEC(0,0,0), VEC(1,0,0)) as closest_point,
    'Projects onto x-axis' as note;
GO

-- =============================================================================
-- 4. POINT-TO-LINE DISTANCE
-- =============================================================================
SELECT '=== POINT-TO-LINE DISTANCE ===' as demo;
GO

-- Distance from point to x-axis
SELECT
    'Distance to x-axis' as case_name,
    VEC(2,2) as point,
    VEC(0,0) as line_point,
    VEC(1,0) as line_direction,
    POINT_LINE_DISTANCE(VEC(2,2), VEC(0,0), VEC(1,0)) as distance,
    'Expected: 2' as verification;
GO

-- Distance from point to diagonal line
SELECT
    'Distance to diagonal' as case_name,
    VEC(0,2) as point,
    VEC(0,0) as line_point,
    VEC(1,1) as line_direction,
    POINT_LINE_DISTANCE(VEC(0,2), VEC(0,0), VEC(1,1)) as distance;
GO

-- Distance should match VEC_DISTANCE from point to closest point
SELECT
    'Verify with VEC_DISTANCE' as case_name,
    VEC(2,2) as point,
    POINT_LINE_DISTANCE(VEC(2,2), VEC(0,0), VEC(1,0)) as direct_distance,
    VEC_DISTANCE(
        VEC(2,2),
        CLOSEST_POINT_ON_LINE(VEC(2,2), VEC(0,0), VEC(1,0))
    ) as distance_via_closest,
    'Should be equal' as note;
GO

-- =============================================================================
-- 5. POINT REFLECTION ACROSS LINE
-- =============================================================================
SELECT '=== POINT REFLECTION ===' as demo;
GO

-- Reflect across x-axis
SELECT
    'Reflect across x-axis' as case_name,
    VEC(2,2) as original_point,
    VEC(0,0) as line_point,
    VEC(1,0) as line_direction,
    LINE_REFLECT_POINT(VEC(2,2), VEC(0,0), VEC(1,0)) as reflected_point,
    'Expected: (2,-2)' as verification;
GO

-- Reflect across y-axis
SELECT
    'Reflect across y-axis' as case_name,
    VEC(3,2) as original_point,
    VEC(0,0) as line_point,
    VEC(0,1) as line_direction,
    LINE_REFLECT_POINT(VEC(3,2), VEC(0,0), VEC(0,1)) as reflected_point,
    'Expected: (-3,2)' as verification;
GO

-- Reflect across diagonal (y=x)
SELECT
    'Reflect across y=x' as case_name,
    VEC(3,1) as original_point,
    VEC(0,0) as line_point,
    VEC(1,1) as line_direction,
    LINE_REFLECT_POINT(VEC(3,1), VEC(0,0), VEC(1,1)) as reflected_point,
    'Expected: (1,3) - swaps x and y' as verification;
GO

-- =============================================================================
-- 6. PRACTICAL APPLICATIONS
-- =============================================================================
SELECT '=== PRACTICAL APPLICATIONS ===' as demo;
GO

-- Collision detection: Do two moving objects intersect?
SELECT
    'Collision Detection' as application,
    VEC(0,0) as object1_start,
    VEC(5,5) as object1_end,
    VEC(0,5) as object2_start,
    VEC(5,0) as object2_end,
    CASE
        WHEN SEGMENT_INTERSECT(VEC(0,0), VEC(5,5), VEC(0,5), VEC(5,0)) IS NOT NULL
        THEN 'COLLISION'
        ELSE 'NO COLLISION'
    END as result,
    SEGMENT_INTERSECT(VEC(0,0), VEC(5,5), VEC(0,5), VEC(5,0)) as collision_point;
GO

-- Snap point to grid line (closest point)
SELECT
    'Snap to Grid' as application,
    VEC(2.3, 4.7) as cursor_position,
    VEC(0,5) as grid_line_point,
    VEC(1,0) as grid_line_direction,
    CLOSEST_POINT_ON_LINE(VEC(2.3, 4.7), VEC(0,5), VEC(1,0)) as snapped_position;
GO

-- Mirror image across axis
SELECT
    'Mirror Image' as application,
    VEC(10,5) as sprite_position,
    VEC(0,0) as mirror_axis_point,
    VEC(0,1) as mirror_axis_direction,
    LINE_REFLECT_POINT(VEC(10,5), VEC(0,0), VEC(0,1)) as mirrored_position;
GO

-- Distance from road (represented as line) to building (point)
SELECT
    'Building to Road Distance' as application,
    VEC(50,30) as building_location,
    VEC(0,0) as road_point,
    VEC(1,0) as road_direction,
    POINT_LINE_DISTANCE(VEC(50,30), VEC(0,0), VEC(1,0)) as distance_meters,
    'How far is building from road' as description;
GO

-- =============================================================================
-- SUMMARY OF CAPABILITIES
-- =============================================================================
-- ✓ Find exact intersection point of two infinite lines
-- ✓ Check if bounded segments intersect
-- ✓ Project point onto line (closest point)
-- ✓ Calculate perpendicular distance from point to line
-- ✓ Reflect point across line (mirror)
-- ✓ All functions work in 2D and 3D
-- ✓ Handles parallel lines gracefully (returns NULL)
-- ✓ Suitable for collision detection, CAD, graphics, mapping

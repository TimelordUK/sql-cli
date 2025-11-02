-- Vector Math Demonstration
-- Shows basic vector operations including addition, dot products, magnitude, etc.

-- Basic vector construction
SELECT
    VEC(1, 2, 3) as vector_3d,
    VEC(10, 20) as vector_2d;
GO

-- Vector addition
SELECT
    VEC(1, 2, 3) as v1,
    VEC(4, 5, 6) as v2,
    VEC_ADD(VEC(1,2,3), VEC(4,5,6)) as sum;
GO

-- Vector subtraction
SELECT
    VEC(10, 20, 30) as v1,
    VEC(1, 2, 3) as v2,
    VEC_SUB(VEC(10,20,30), VEC(1,2,3)) as difference;
GO

-- Scalar multiplication
SELECT
    VEC(1, 2, 3) as original,
    VEC_SCALE(VEC(1,2,3), 2.5) as scaled;
GO

-- Dot product
SELECT
    VEC(1, 2, 3) as v1,
    VEC(4, 5, 6) as v2,
    VEC_DOT(VEC(1,2,3), VEC(4,5,6)) as dot_product;
GO

-- Magnitude (length) of vector
SELECT
    VEC(3, 4) as vector_2d,
    VEC_MAG(VEC(3,4)) as magnitude;
GO

-- Normalize vector to unit length
SELECT
    VEC(3, 4) as original,
    VEC_NORMALIZE(VEC(3,4)) as unit_vector,
    VEC_MAG(VEC_NORMALIZE(VEC(3,4))) as should_be_one;
GO

-- Distance between two points (vectors)
SELECT
    VEC(0, 0) as point1,
    VEC(3, 4) as point2,
    VEC_DISTANCE(VEC(0,0), VEC(3,4)) as distance;
GO

-- Cross product (3D only)
SELECT
    VEC(1, 0, 0) as i_hat,
    VEC(0, 1, 0) as j_hat,
    VEC_CROSS(VEC(1,0,0), VEC(0,1,0)) as k_hat;
GO

-- Angle between vectors (in radians)
SELECT
    VEC(1, 0) as v1,
    VEC(0, 1) as v2,
    VEC_ANGLE(VEC(1,0), VEC(0,1)) as angle_rad,
    VEC_ANGLE(VEC(1,0), VEC(0,1)) * 180 / PI() as angle_deg;
GO

-- Physics example: velocity and displacement
SELECT
    VEC(10, 5, 0) as velocity_mps,
    3.0 as time_s,
    VEC_SCALE(VEC(10,5,0), 3.0) as displacement_m,
    VEC_MAG(VEC_SCALE(VEC(10,5,0), 3.0)) as distance_traveled;
GO

-- Physics example: Force resultant
SELECT
    VEC(100, 0) as force1_N,
    VEC(0, 100) as force2_N,
    VEC_ADD(VEC(100,0), VEC(0,100)) as resultant_N,
    VEC_MAG(VEC_ADD(VEC(100,0), VEC(0,100))) as magnitude_N;
GO

-- Mathematical functions and operations
-- sql-cli provides extensive mathematical capabilities

-- Basic arithmetic operations
SELECT 
    10 + 5 as addition,
    10 - 5 as subtraction,
    10 * 5 as multiplication,
    10 / 5 as division,
    10 % 3 as modulo,
    POWER(2, 8) as two_to_eighth,
    SQRT(144) as square_root;
GO

-- Rounding and precision functions
SELECT 
    ROUND(3.14159, 2) as rounded,
    CEIL(4.3) as ceiling,
    FLOOR(4.7) as floor,
    TRUNC(123.456, 1) as truncated,
    ABS(-42) as absolute_value,
    SIGN(-10) as sign_negative,
    SIGN(10) as sign_positive;
GO
-- Trigonometric functions
SELECT 
    SIN(PI() / 2) as sine_90_deg,
    COS(0) as cosine_0_deg,
    TAN(PI() / 4) as tangent_45_deg,
    ASIN(1) as arcsine,
    ACOS(0) as arccosine,
    ATAN(1) as arctangent,
    DEGREES(PI()) as pi_in_degrees,
    RADIANS(180) as half_circle_radians;

-- Logarithmic and exponential functions
SELECT 
    EXP(1) as e_to_first,
    LN(2.718281828) as natural_log,
    LOG(100) as log_base_10,
    LOG(2, 8) as log_base_2_of_8,
    LOG10(1000) as common_log;
GO

-- Statistical functions
SELECT 
    GREATEST(1, 5, 3, 9, 2) as maximum,
    LEAST(1, 5, 3, 9, 2) as minimum,
    RANDOM() as random_number,
    RANDOM() * 100 as random_0_to_100;
GO

-- Advanced mathematical operations
SELECT 
    FACTORIAL(5) as five_factorial,
    GCD(48, 18) as greatest_common_divisor,
    LCM(12, 18) as least_common_multiple,
    IS_PRIME(17) as check_prime,
    NTH_PRIME(10) as tenth_prime;
GO

-- Bitwise operations
SELECT 
    5 & 3 as bitwise_and,
    5 | 3 as bitwise_or,
    5 ^ 3 as bitwise_xor,
    ~5 as bitwise_not,
    5 << 2 as left_shift,
    20 >> 2 as right_shift;
GO

-- Financial calculations
SELECT 
    -- Compound interest: A = P(1 + r/n)^(nt)
    1000 * POWER(1 + 0.05/12, 12*5) as compound_interest_5_years,
    
    -- Monthly payment: M = P[r(1+r)^n]/[(1+r)^n-1]
    100000 * (0.04/12 * POWER(1 + 0.04/12, 360)) / (POWER(1 + 0.04/12, 360) - 1) as mortgage_payment;
GO

-- Practical example: Sales analysis with mathematical functions
SELECT 
    product_id,
    COUNT(*) as sales_count,
    SUM(quantity) as total_units,
    AVG(price) as avg_price,
    STDDEV(price) as price_volatility,
    MIN(price) as min_price,
    MAX(price) as max_price,
    ROUND(AVG(price * quantity), 2) as avg_sale_value,
    ROUND(SQRT(VARIANCE(price)), 2) as price_std_dev,
    ROUND(100.0 * (MAX(price) - MIN(price)) / MIN(price), 2) as price_range_pct
FROM sales
GROUP BY product_id
HAVING COUNT(*) > 10
ORDER BY avg_sale_value DESC;
GO

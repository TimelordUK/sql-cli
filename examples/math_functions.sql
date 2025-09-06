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
    ABS(-42) as absolute_value;
GO

-- Logarithmic and exponential functions
SELECT 
    EXP(1) as e_to_first,
    LN(2.718281828) as natural_log,
    LOG(100) as log_base_10,
    LOG(8, 2) as log_base_2_of_8,
    LOG10(1000) as common_log;
GO

-- Comparison functions
SELECT 
    GREATEST(1, 5, 3, 9, 2) as maximum,
    LEAST(1, 5, 3, 9, 2) as minimum;
GO

-- Advanced mathematical operations  
SELECT 
    FACTORIAL(5) as five_factorial,
    FACTORIAL(10) as ten_factorial,
    FACTORIAL(0) as zero_factorial,
    IS_PRIME(17) as check_prime,
    NTH_PRIME(10) as tenth_prime,
    PRIME_PI(100) as primes_up_to_100;
GO

-- Financial calculations using available functions
SELECT 
    -- Compound interest: A = P(1 + r/n)^(nt)
    1000 * POWER(1 + 0.05/12, 12*5) as compound_interest_5_years,
    
    -- Simple interest calculation
    1000 * 0.05 * 5 as simple_interest_5_years,
    
    -- Future value with monthly deposits
    500 * ((POWER(1 + 0.04/12, 60) - 1) / (0.04/12)) as future_value_5_years;
GO

-- Degree/Radian conversion
SELECT 
    DEGREES(3.14159) as pi_in_degrees,
    RADIANS(180) as half_circle_radians,
    DEGREES(RADIANS(90)) as round_trip_90;
GO

-- SQL-CLI Deterministic Feature Showcase
-- This file demonstrates all implemented features with DETERMINISTIC output
-- Safe for formal testing with expectations (no random/time-based functions)
-- Each section is separated by GO to execute as independent statements

-- ===== MATHEMATICAL FUNCTIONS =====
-- Basic math operations and advanced functions
SELECT 
    -- Arithmetic
    10 + 5 as addition,
    20 - 8 as subtraction,
    7 * 6 as multiplication,
    100 / 4 as division,
    10 % 3 as modulo,
    
    -- Power and roots
    POWER(2, 10) as two_to_tenth,
    SQRT(144) as square_root,
    
    -- Rounding
    ROUND(3.14159, 2) as rounded,
    FLOOR(9.8) as floored,
    CEIL(9.2) as ceiling,
    
    -- Advanced
    ABS(-42) as absolute,
    FACTORIAL(5) as five_factorial,
    LN(2.71828) as natural_log,
    LOG(100) as log_base10,
    EXP(1) as e_to_one;
GO

-- ===== PRIME NUMBER FUNCTIONS =====
-- Working with prime numbers
SELECT 
    PRIME(10) as tenth_prime,
    NTH_PRIME(20) as twentieth_prime,
    PRIME_COUNT(100) as primes_up_to_100,
    PRIME_PI(50) as primes_up_to_50,
    IS_PRIME(17) as seventeen_is_prime,
    IS_PRIME(18) as eighteen_is_prime;
GO

-- ===== PHYSICS CONSTANTS =====
-- Fundamental physical constants
SELECT 
    C() as speed_of_light_ms,
    H() as planck_constant,
    G() as gravitational_constant,
    K() as boltzmann_constant,
    AVOGADRO() as avogadro_number,
    ME() as electron_mass,
    MP() as proton_mass,
    MN() as neutron_mass;
GO

-- ===== ASTRONOMICAL MASSES =====
-- Celestial body masses in kg
SELECT 
    MASS_SUN() as sun_mass,
    MASS_EARTH() as earth_mass,
    MASS_MOON() as moon_mass,
    MASS_JUPITER() as jupiter_mass,
    MASS_MARS() as mars_mass,
    MASS_SUN() / MASS_EARTH() as sun_earth_ratio;
GO

-- ===== CHEMISTRY FUNCTIONS =====
-- Chemical formulas and atomic masses
SELECT 
    MOLECULE_FORMULA('water') as water_formula,
    ATOMIC_MASS('water') as water_mass,
    MOLECULE_FORMULA('methane') as methane_formula,
    ATOMIC_MASS('methane') as methane_mass,
    MOLECULE_FORMULA('glucose') as glucose_formula,
    ATOMIC_MASS('glucose') as glucose_mass;
GO

-- ===== STRING FUNCTIONS =====
-- Text manipulation capabilities
SELECT 
    -- Case conversion
    UPPER('hello world') as uppercase,
    LOWER('HELLO WORLD') as lowercase,
    
    -- String properties
    LENGTH('sql-cli') as string_length,
    
    -- String modification
    TRIM('  spaces  ') as trimmed,
    SUBSTRING('Hello World', 1, 5) as substr,
    MID('Database', 3, 4) as mid_extract,
    REPLACE('Hello World', 'World', 'SQL-CLI') as replaced;
GO

-- ===== ADVANCED STRING FUNCTIONS =====
-- More string operations
SELECT 
    -- String joining
    TEXTJOIN(' ', 1, 'SQL', 'CLI', 'Rocks') as joined_text,
    TEXTJOIN(', ', 1, 'apple', 'banana', 'orange') as fruit_list,
    
    -- String checking
    STARTSWITH('sql-cli', 'sql') as starts_with_sql,
    ENDSWITH('database.db', '.db') as ends_with_db,
    CONTAINS('hello world', 'world') as contains_world,
    
    -- Fuzzy matching
    EDIT_DISTANCE('kitten', 'sitting') as edit_dist;
GO

-- ===== DATE AND TIME FUNCTIONS =====
-- Working with dates and times (using fixed dates for deterministic output)
SELECT
    -- Fixed date/time for testing
    '2024-01-15 14:30:00' as example_timestamp,
    '2024-01-15' as example_date;
GO

-- ===== DATE ARITHMETIC =====
-- Date calculations
SELECT 
    -- Adding intervals
    DATEADD('day', 7, '2024-01-01') as week_later,
    DATEADD('month', 3, '2024-01-01') as three_months_later,
    DATEADD('year', 1, '2024-01-01') as next_year,
    
    -- Date differences
    DATEDIFF('day', '2024-01-01', '2024-12-31') as days_in_2024,
    DATEDIFF('month', '2024-01-01', '2024-12-31') as months_diff,
    DATEDIFF('year', '2000-01-01', '2024-01-01') as years_diff;
GO

-- Note: EXTRACT, DATE_FORMAT, and DATE_PARSE are not yet implemented
-- These would be useful additions for future versions

-- ===== PRACTICAL PHYSICS CALCULATIONS =====
-- Real-world physics applications
SELECT 
    -- Escape velocity from Earth: v = sqrt(2GM/r)
    SQRT(2 * G() * MASS_EARTH() / 6371000) as earth_escape_velocity_ms,
    
    -- Energy from mass (E=mc²)
    1 * POWER(C(), 2) as energy_joules_from_1kg,
    
    -- Schwarzschild radius of Sun
    (2 * G() * MASS_SUN()) / POWER(C(), 2) as sun_schwarzschild_radius_m;
GO

-- ===== FACTORIAL SHOWCASE =====
-- Demonstrate factorial function with lookup table
SELECT
    FACTORIAL(0) as fact_0,
    FACTORIAL(5) as fact_5,
    FACTORIAL(10) as fact_10,
    FACTORIAL(15) as fact_15,
    FACTORIAL(20) as fact_20;
GO

-- ===== TRIGONOMETRIC FUNCTIONS =====
-- Comprehensive trig functions (angles in radians)
SELECT
    -- Basic trig
    SIN(PI() / 6) as sin_30_degrees,
    COS(PI() / 3) as cos_60_degrees,
    TAN(PI() / 4) as tan_45_degrees,
    COT(PI() / 4) as cot_45_degrees,

    -- Inverse trig
    ASIN(0.5) as arcsin_half,
    ACOS(0.5) as arccos_half,
    ATAN(1) as arctan_one,
    ATAN2(1, 1) as arctan2_45deg,

    -- Hyperbolic
    SINH(1) as hyperbolic_sine,
    COSH(1) as hyperbolic_cosine,
    TANH(1) as hyperbolic_tangent,

    -- Angle conversion
    DEGREES(PI()) as pi_in_degrees,
    RADIANS(180) as degrees_180_in_radians;
GO

-- ===== GEOMETRY FUNCTIONS =====
-- Area, volume, and distance calculations
SELECT
    -- 2D geometry
    CIRCLE_AREA(10) as circle_area_r10,
    CIRCLE_CIRCUMFERENCE(10) as circle_circum_r10,
    TRIANGLE_AREA(3, 4, 5) as right_triangle_area,
    DISTANCE_2D(0, 0, 3, 4) as distance_origin_to_3_4,
    PYTHAGORAS(3, 4) as hypotenuse_3_4,

    -- 3D geometry
    SPHERE_VOLUME(10) as sphere_volume_r10,
    SPHERE_SURFACE_AREA(10) as sphere_surface_r10;
GO

-- ===== STATISTICAL FUNCTIONS =====
-- Note: These are aggregate functions, shown here as examples
-- In real use, they work with GROUP BY or window functions
SELECT
    -- Standard deviation and variance
    STDDEV(5) as example_stddev,
    VOLATILITY(0.01) as example_volatility,

    -- Financial calculations
    RETURNS(110, 100) as simple_return,
    LOG_RETURNS(110, 100) as log_return,
    SHARPE_RATIO(0.12, 0.02, 0.15) as sharpe_ratio;
GO

-- ===== MATHEMATICAL SEQUENCES =====
-- Fibonacci, harmonic, geometric series
SELECT
    -- Fibonacci sequence
    FIBONACCI(0) as fib_0,
    FIBONACCI(1) as fib_1,
    FIBONACCI(10) as fib_10,
    FIBONACCI(20) as fib_20,

    -- Series sums
    SUM_N(100) as sum_1_to_100,
    SUM_N_SQR(10) as sum_squares_1_to_10,
    SUM_N_CUBE(5) as sum_cubes_1_to_5,
    HARMONIC(10) as harmonic_10,
    GEOMETRIC(2, 0.5, 10) as geometric_series;
GO

-- ===== EXTENDED ASTRONOMICAL FUNCTIONS =====
-- Complete solar system data
SELECT
    'Mercury' as planet,
    MASS_MERCURY() as mass_kg,
    RADIUS_MERCURY() as radius_m,
    DIST_MERCURY() as distance_from_sun_m,
    ORBITAL_PERIOD_SOLAR_BODY('mercury') as orbital_period_days,
    ROTATION_PERIOD_SOLAR_BODY('mercury') as rotation_period_hours,
    GRAVITY_SOLAR_BODY('mercury') as surface_gravity_ms2,
    ESCAPE_VELOCITY_SOLAR_BODY('mercury') as escape_velocity_ms,
    DENSITY_SOLAR_BODY('mercury') as density_kg_m3,
    MOONS_SOLAR_BODY('mercury') as moon_count;
GO

-- ===== ASTRONOMICAL CALCULATIONS =====
-- Practical astronomical applications
SELECT
    -- Orbital velocities
    SQRT(G() * MASS_SUN() / DISTANCE_SOLAR_BODY('earth')) as earth_orbital_velocity_ms,

    -- Escape velocities
    ESCAPE_VELOCITY_SOLAR_BODY('earth') as earth_escape_velocity,
    ESCAPE_VELOCITY_SOLAR_BODY('moon') as moon_escape_velocity,

    -- Distance comparisons
    DIST_MARS() / AU() as mars_distance_in_AU,
    LIGHT_YEAR() / PARSEC() as ly_per_parsec,

    -- Density calculations
    DENSITY_SOLAR_BODY('earth') as earth_density,
    DENSITY_SOLAR_BODY('jupiter') as jupiter_density;
GO

-- ===== EXTENDED PHYSICS CONSTANTS =====
-- Additional fundamental constants
SELECT
    -- Quantum constants
    HBAR() as reduced_planck_constant,
    ALPHA() as fine_structure_constant,
    BOHR() as bohr_radius,
    RY() as rydberg_constant,

    -- Electromagnetic constants
    E0() as electric_permittivity,
    MU0() as magnetic_permeability,
    COULOMB() as coulomb_constant,

    -- Mathematical constants
    PHI() as golden_ratio,
    TAU() as tau_2pi,
    E() as euler_number,

    -- Other constants
    AMU() as atomic_mass_unit,
    R() as gas_constant,
    SIGMA() as stefan_boltzmann;
GO

-- ===== PARTICLE PHYSICS =====
-- Elementary particle properties
SELECT
    -- Electron properties
    ME() as electron_mass,
    CHARGE_ELECTRON() as electron_charge,
    RE() as electron_radius,

    -- Proton/Neutron properties
    MP() as proton_mass,
    CHARGE_PROTON() as proton_charge,
    RP() as proton_radius,
    MN() as neutron_mass,
    CHARGE_NEUTRON() as neutron_charge,
    RN() as neutron_radius,

    -- Other particles
    CHARGE_POSITRON() as positron_charge,
    CHARGE_MUON() as muon_charge,
    CHARGE_TAU() as tau_charge,
    CHARGE_UP_QUARK() as up_quark_charge,
    CHARGE_DOWN_QUARK() as down_quark_charge;
GO

-- ===== EXTENDED CHEMISTRY =====
-- Atomic structure and properties
SELECT
    -- Element properties
    ATOMIC_NUMBER('carbon') as carbon_atomic_num,
    ATOMIC_MASS('carbon') as carbon_atomic_mass,
    NEUTRONS('carbon') as carbon_neutrons,

    -- More molecules
    MOLECULE_FORMULA('caffeine') as caffeine_formula,
    ATOMIC_MASS('caffeine') as caffeine_mass,
    MOLECULE_FORMULA('aspirin') as aspirin_formula,
    ATOMIC_MASS('aspirin') as aspirin_mass,
    AVOGADRO() as avogadro_number;
GO

-- ===== BASE CONVERSIONS =====
-- Binary, octal, hex, and custom bases
SELECT
    -- Binary conversions
    TO_BINARY(42) as binary_42,
    FROM_BINARY('101010') as from_binary,

    -- Hexadecimal
    TO_HEX(255) as hex_255,
    FROM_HEX('FF') as from_hex_FF,

    -- Octal
    TO_OCTAL(64) as octal_64,
    FROM_OCTAL('100') as from_octal,

    -- Custom bases
    TO_BASE(100, 16) as base_16,
    FROM_BASE('64', 16) as from_base_16,
    TO_BASE(1234, 36) as base_36;
GO

-- ===== BIG NUMBER ARITHMETIC =====
-- Arbitrary precision calculations
SELECT
    -- Big integer operations
    BIGINT(999999999999999999) as big_int,
    BIGADD(9999999999999999, 1) as big_add,
    BIGMUL(999999999, 999999999) as big_multiply,
    BIGPOW(2, 100) as two_to_100th,
    BIGFACT(50) as factorial_50;
GO

-- ===== BITWISE OPERATIONS =====
-- Working with binary data
SELECT
    -- Basic bitwise ops
    BITAND(12, 10) as bitwise_and,
    BITOR(12, 10) as bitwise_or,
    BITXOR(12, 10) as bitwise_xor,
    BITNOT(5) as bitwise_not,
    BITSHIFT(8, 2) as shift_left_2,
    BITSHIFT(8, -2) as shift_right_2,

    -- Bit analysis
    COUNT_BITS(255) as count_ones,
    IS_POWER_OF_TWO(256) as is_pow_2,
    NEXT_POWER_OF_TWO(100) as next_pow_2,
    HIGHEST_BIT(255) as highest_bit,
    LOWEST_BIT(12) as lowest_bit;
GO

-- ===== BINARY STRING OPERATIONS =====
-- Advanced bit manipulation
SELECT
    -- Binary formatting
    BINARY_FORMAT(255, '_') as formatted_binary,

    -- Bit string operations
    BIT_AND_STR('1100', '1010') as bit_str_and,
    BIT_OR_STR('1100', '1010') as bit_str_or,
    BIT_XOR_STR('1100', '1010') as bit_str_xor,
    BIT_NOT_STR('1100') as bit_str_not,

    -- Bit shifting and rotation
    BIT_SHIFT_LEFT('1100', 2) as shift_left,
    BIT_ROTATE_LEFT('1100', 1) as rotate_left,

    -- Bit counting
    BIT_COUNT('11001100') as count_bits,
    HAMMING_DISTANCE('11001', '10101') as hamming_dist;
GO

-- ===== EXTENDED DATE FUNCTIONS =====
-- Complete date/time manipulation (using fixed date)
SELECT
    -- Date component extraction (using 2024-03-15)
    YEAR('2024-03-15') as year_value,
    MONTH('2024-03-15') as month_value,
    DAY('2024-03-15') as day_value,
    QUARTER('2024-03-15') as quarter_value,
    WEEKOFYEAR('2024-03-15') as week_of_year,
    DAYOFWEEK('2024-03-15') as day_of_week,

    -- Date names
    DAYNAME('2024-03-15', 'full') as day_name_full,
    DAYNAME('2024-03-15', 'short') as day_name_short,
    MONTHNAME('2024-03-15', 'full') as month_name_full,
    MONTHNAME('2024-03-15', 'short') as month_name_short,

    -- Date checks
    ISLEAPYEAR(2024) as is_2024_leap_year;
GO

-- ===== DATE FORMATTING AND PARSING =====
-- Custom date formatting (using fixed dates)
SELECT
    -- Format dates (using 2024-03-15)
    FORMAT_DATE('2024-03-15 10:30:45', '%Y-%m-%d') as iso_date,
    FORMAT_DATE('2024-03-15 10:30:45', '%B %d, %Y') as full_date,
    FORMAT_DATE('2024-03-15 10:30:45', '%m/%d/%Y') as us_date,

    -- Unix timestamps (using fixed timestamp)
    UNIX_TIMESTAMP('2024-01-15 12:00:00') as unix_time,
    FROM_UNIXTIME(1609459200) as from_epoch,

    -- Parse custom formats
    PARSE_DATETIME('2024-12-25 15:30:00', '%Y-%m-%d %H:%M:%S') as parsed_datetime;
GO

-- ===== ADVANCED STRING FUNCTIONS =====
-- Text encoding, hashing, and transformations
SELECT
    -- Hashing
    MD5('hello world') as md5_hash,
    SHA1('hello world') as sha1_hash,
    SHA256('hello world') as sha256_hash,
    SHA512('hello world') as sha512_hash,

    -- Encoding
    ENCODE('Hello', 'base64') as base64_encoded,
    DECODE('SGVsbG8=', 'base64') as base64_decoded,

    -- Phonetic and matching
    SOUNDEX('Smith') as soundex_smith,
    SOUNDEX('Smyth') as soundex_smyth;
GO

-- ===== FUN TEXT TRANSFORMATIONS =====
-- Playful text manipulation (deterministic only - removed SCRAMBLE and LOREM_IPSUM)
SELECT
    -- Encoding games
    MORSE_CODE('SOS') as morse_sos,
    ROT13('Hello World') as rot13_encoded,
    PIG_LATIN('Hello World') as pig_latin,

    -- Note: SCRAMBLE() and LOREM_IPSUM() are non-deterministic
    -- See showcase_all_features.sql for full demo
    'Removed for deterministic testing' as note_about_random_funcs;
GO

-- ===== CASE CONVERSIONS =====
-- Programming naming conventions
SELECT
    'hello world example' as original,
    TO_CAMEL_CASE('hello world example') as camelCase,
    TO_PASCAL_CASE('hello world example') as PascalCase,
    TO_SNAKE_CASE('Hello World Example') as snake_case,
    TO_KEBAB_CASE('Hello World Example') as kebab_case,
    TO_CONSTANT_CASE('hello world example') as CONSTANT_CASE;
GO

-- ===== NUMBER TO TEXT CONVERSIONS =====
-- Spelling out numbers
SELECT
    TO_WORDS(42) as forty_two,
    TO_WORDS(1234567) as spelled_out,
    TO_ORDINAL(1) as first_ordinal,
    TO_ORDINAL(42) as forty_second_ordinal,
    TO_ORDINAL_WORDS(1) as first_words,
    TO_ORDINAL_WORDS(21) as twenty_first_words;
GO

-- ===== ROMAN NUMERALS =====
-- Ancient number system
SELECT
    TO_ROMAN(1994) as mcmxciv,
    TO_ROMAN(2024) as year_2024_roman,
    FROM_ROMAN('MCMXCIV') as from_roman_1994,
    FROM_ROMAN('MMXXIV') as from_roman_2024;
GO

-- ===== NUMBER FORMATTING =====
-- Professional number display
SELECT
    -- Standard formatting
    FORMAT_NUMBER(1234567.89, 2) as formatted_with_commas,
    RENDER_NUMBER(1234567, 'thousands') as with_k_suffix,
    RENDER_NUMBER(1234567890, 'millions') as with_m_suffix,

    -- Currency formatting
    FORMAT_CURRENCY(1234.56, 'USD') as us_dollars,
    FORMAT_CURRENCY(1234.56, 'EUR') as euros,
    FORMAT_CURRENCY(1234.56, 'GBP') as pounds;
GO

-- ===== STRING PADDING AND ALIGNMENT =====
-- Text formatting
SELECT
    LPAD('42', 5, '0') as zero_padded,
    RPAD('Name', 20, '.') as dot_filled,
    CENTER('TITLE', 20) as centered_text,
    REPEAT('-', 10) as divider;
GO

-- ===== TEXT ANALYSIS =====
-- Word counting and extraction
SELECT
    WORD_COUNT('The quick brown fox jumps') as word_count,
    EXTRACT_WORDS('Hello, World! How are you?', 3) as words_min_3,
    TOKENIZE('test-data_here') as tokens,
    CLEAN_TEXT('  Extra   spaces   here  ') as cleaned,
    STRIP_PUNCTUATION('Hello, World!') as no_punctuation,
    FREQUENCY('banana', 'a') as count_letter_a;
GO

-- ===== STRING UTILITY FUNCTIONS =====
-- Additional string helpers
SELECT
    -- Character codes
    ASCII('A') as ascii_A,
    CHR(65) as chr_65,
    UNICODE('ABC') as unicode_codes,
    ORD('Z') as ord_Z,

    -- String inspection
    INDEXOF('Hello World', 'World') as zero_based_index,
    INSTR('Hello World', 'World') as one_based_index,
    SPLIT_PART('a,b,c,d', ',', 2) as second_part,

    -- Substring extraction
    SUBSTRING_BEFORE('user@example.com', '@') as username,
    SUBSTRING_AFTER('user@example.com', '@') as domain,
    LEFT('Hello', 3) as left_3,
    RIGHT('Hello', 2) as right_2,
    REVERSE('Hello') as reversed;
GO

-- ===== CONDITIONAL AND UTILITY FUNCTIONS =====
-- Logic and null handling
SELECT
    -- Conditionals (use 1/0 for true/false in IIF to avoid parser issues)
    IIF(1, 'Yes', 'No') as simple_if_true,
    IIF(0, 'Yes', 'No') as simple_if_false,
    COALESCE(NULL, NULL, 'default') as first_non_null,
    NULLIF(5, 5) as returns_null,
    NULLIF(5, 3) as returns_5,

    -- Comparisons
    GREATEST(1, 5, 3, 9, 2) as max_value,
    LEAST(1, 5, 3, 9, 2) as min_value,
    GREATEST_LABEL('A', 100, 'B', 250, 'C', 175) as best_label,
    LEAST_LABEL('A', 100, 'B', 250, 'C', 175) as worst_label;
GO

-- ===== TYPE CHECKING FUNCTIONS =====
-- Validate data types
SELECT
    IS_NULL(NULL) as is_null_check,
    IS_NOT_NULL('value') as is_not_null_check,
    IS_INTEGER('123') as is_int_true,
    IS_INTEGER('12.3') as is_int_false,
    IS_FLOAT('12.3') as is_float_true,
    IS_NUMERIC('123') as is_num_true,
    IS_BOOL('true') as is_bool_true,
    IS_DATE('2024-01-01') as is_date_true;
GO

-- ===== INTEGER LIMIT CONSTANTS =====
-- Data type boundaries
SELECT
    -- 8-bit limits
    INT8_MIN() as int8_min,
    INT8_MAX() as int8_max,
    UINT8_MAX() as uint8_max,
    BYTE_MIN() as byte_min,
    BYTE_MAX() as byte_max,

    -- 16-bit limits
    INT16_MIN() as int16_min,
    INT16_MAX() as int16_max,
    UINT16_MAX() as uint16_max,
    SHORT_MIN() as short_min,
    SHORT_MAX() as short_max,

    -- 32-bit limits
    INT32_MIN() as int32_min,
    INT32_MAX() as int32_max,
    UINT32_MAX() as uint32_max,

    -- 64-bit limits
    INT64_MIN() as int64_min,
    INT64_MAX() as int64_max,
    LONG_MIN() as long_min,
    LONG_MAX() as long_max;
GO

-- ===== RANDOM NUMBER GENERATION =====
-- Note: Random functions are non-deterministic and removed from this test
-- See showcase_all_features.sql for RANDOM(), RAND_INT(), RAND_RANGE() demos
-- Using fixed mathematical sequences instead
SELECT
    PI() as mathematical_constant,
    FIBONACCI(10) as tenth_fibonacci,
    PRIME(25) as twenty_fifth_prime;
GO

-- ===== PRIME NUMBER EXTENDED =====
-- Advanced prime operations
SELECT
    NEXT_PRIME(100) as next_prime_after_100,
    PREV_PRIME(100) as prev_prime_before_100,
    PRIME(100) as hundredth_prime,
    NTH_PRIME(1000) as thousandth_prime,
    PRIME_COUNT(1000) as primes_up_to_1000,
    IS_PRIME(1009) as is_1009_prime;
GO


-- ===== PI CONSTANTS =====
-- Pi to arbitrary precision
SELECT
    PI() as pi_default,
    PI_DIGITS(50) as pi_50_decimals,
    PI_DIGIT(1) as first_digit,
    PI_DIGIT(10) as tenth_digit,
    PI_DIGIT(100) as hundredth_digit;
GO


-- ===== ANSI TERMINAL COLORS =====
-- Colorful terminal output
SELECT
    ANSI_COLOR('red', 'ERROR') as red_error,
    ANSI_COLOR('green', 'SUCCESS') as green_success,
    ANSI_COLOR('yellow', 'WARNING') as yellow_warning,
    ANSI_COLOR('blue', 'INFO') as blue_info,

    -- Background colors
    ANSI_BG('red', 'CRITICAL') as red_background,

    -- RGB colors (true color)
    ANSI_RGB(255, 165, 0, 'Orange') as orange_text,
    ANSI_RGB_BG(0, 100, 200, 'Blue BG') as rgb_bg,

    -- Text formatting
    ANSI_BOLD('Bold Text') as bold,
    ANSI_ITALIC('Italic Text') as italic,
    ANSI_UNDERLINE('Underlined') as underline,
    ANSI_STRIKETHROUGH('Strikethrough') as strike,
    ANSI_BLINK('Blinking') as blink,
    ANSI_REVERSE('Reversed') as reverse;
GO

-- ===== END OF DETERMINISTIC SHOWCASE =====
-- This file demonstrates the extensive capabilities of sql-cli
-- With DETERMINISTIC output safe for formal testing with expectations
-- Covering: Math, Trig, Geometry, Stats, Astronomy, Physics, Chemistry,
--           String manipulation, Date/Time, Number systems, Bitwise ops,
--           Text analysis, Formatting, Colors, and much more!
--
-- For non-deterministic features (RANDOM, NOW, LOREM_IPSUM, SCRAMBLE):
-- See: examples/showcase_all_features.sql
--
-- Run with: ./target/release/sql-cli -f examples/showcase_deterministic.sql
-- Test with: uv run python tests/integration/test_examples.py showcase_deterministic

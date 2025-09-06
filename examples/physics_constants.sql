-- Physics constants and astronomical calculations
-- sql-cli includes comprehensive physics constants

-- Fundamental physics constants
SELECT 
    C() as speed_of_light_ms,
    H() as planck_constant_js,
    G() as gravitational_constant,
    AVOGADRO() as avogadro_number,
    K() as boltzmann_constant_jk;
GO

-- Astronomical masses (in kg)
SELECT 
    MASS_SUN() as sun_mass_kg,
    MASS_EARTH() as earth_mass_kg,
    MASS_MOON() as moon_mass_kg,
    MASS_JUPITER() as jupiter_mass_kg,
    MASS_MARS() as mars_mass_kg;
GO

-- Particle masses
SELECT 
    ME() as electron_mass_kg,
    MP() as proton_mass_kg,
    MN() as neutron_mass_kg;
GO

-- Practical calculations using constants
SELECT 
    -- Calculate escape velocity from Earth: v = sqrt(2GM/r)
    SQRT(2 * G() * MASS_EARTH() / 6371000) as earth_escape_velocity_ms,
    
    -- Energy equivalent of 1kg mass: E = mc²
    1 * POWER(C(), 2) as energy_joules_from_1kg,
    
    -- Schwarzschild radius of the Sun: r = 2GM/c²
    (2 * G() * MASS_SUN()) / POWER(C(), 2) as sun_schwarzschild_radius_m;
GO

-- Compare celestial bodies
SELECT 
    MASS_JUPITER() / MASS_EARTH() as jupiter_earth_mass_ratio,
    MASS_SUN() / MASS_EARTH() as sun_earth_mass_ratio,
    MASS_EARTH() / MASS_MOON() as earth_moon_mass_ratio;
GO

-- Physics constants and astronomical calculations
-- sql-cli includes comprehensive physics constants

-- Fundamental physics constants
SELECT 
    SPEED_OF_LIGHT() as c_meters_per_sec,
    PLANCK_CONSTANT() as h_joule_seconds,
    GRAVITATIONAL_CONSTANT() as G_newton_constant,
    AVOGADRO_NUMBER() as avogadro_const,
    BOLTZMANN_CONSTANT() as k_boltzmann;

-- Astronomical masses (in kg)
SELECT 
    MASS_SUN() as sun_mass_kg,
    MASS_EARTH() as earth_mass_kg,
    MASS_MOON() as moon_mass_kg,
    MASS_JUPITER() as jupiter_mass_kg,
    MASS_MARS() as mars_mass_kg;

-- Particle masses
SELECT 
    ELECTRON_MASS() as electron_kg,
    PROTON_MASS() as proton_kg,
    NEUTRON_MASS() as neutron_kg;

-- Practical calculations using constants
SELECT 
    -- Calculate escape velocity from Earth: v = sqrt(2GM/r)
    SQRT(2 * GRAVITATIONAL_CONSTANT() * MASS_EARTH() / 6371000) as earth_escape_velocity_ms,
    
    -- Energy equivalent of 1kg mass: E = mc²
    1 * POWER(SPEED_OF_LIGHT(), 2) as energy_joules_from_1kg,
    
    -- Schwarzschild radius of the Sun: r = 2GM/c²
    (2 * GRAVITATIONAL_CONSTANT() * MASS_SUN()) / POWER(SPEED_OF_LIGHT(), 2) as sun_schwarzschild_radius_m;

-- Compare celestial bodies
SELECT 
    MASS_JUPITER() / MASS_EARTH() as jupiter_earth_mass_ratio,
    MASS_SUN() / MASS_EARTH() as sun_earth_mass_ratio,
    MASS_EARTH() / MASS_MOON() as earth_moon_mass_ratio;
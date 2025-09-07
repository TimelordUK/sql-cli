-- ============================================================================
-- Physics and Astronomy Constants Showcase
-- Demonstrates all physics and astronomy functions available in SQL CLI
-- ============================================================================
-- Run: ./target/release/sql-cli -f examples/physics_astronomy_showcase.sql -o csv
-- ============================================================================

-- Fundamental Physics Constants
SELECT 
    '=== PHYSICS CONSTANTS ===' as category,
    G() as gravitational_constant,
    C() as speed_of_light,
    H() as planck_constant,
    K() as boltzmann_constant,
    AVOGADRO() as avogadro_number,
    E() as elementary_charge,
    ME() as electron_mass,
    MP() as proton_mass;

-- Astronomical Distances
SELECT 
    '=== ASTRONOMICAL DISTANCES ===' as category,
    AU() as astronomical_unit,
    LIGHTYEAR() as light_year,
    PARSEC() as parsec,
    AU() * 8.3 as distance_to_neptune,
    LIGHTYEAR() * 4.24 as distance_to_proxima_centauri;

-- Solar System Masses (in kg)
SELECT 
    '=== SOLAR SYSTEM MASSES ===' as category,
    MASS_SUN() as sun,
    MASS_EARTH() as earth,
    MASS_MOON() as moon,
    MASS_JUPITER() as jupiter,
    MASS_SATURN() as saturn,
    MASS_MARS() as mars;

-- Solar System Radii (in meters)
SELECT 
    '=== SOLAR SYSTEM RADII ===' as category,
    RADIUS_SUN() as sun,
    RADIUS_EARTH() as earth,
    RADIUS_MOON() as moon,
    RADIUS_JUPITER() as jupiter,
    RADIUS_SATURN() as saturn,
    RADIUS_MARS() as mars;

-- Planetary Orbital Distances (in meters)
SELECT 
    '=== ORBITAL DISTANCES ===' as category,
    DIST_MERCURY() as mercury,
    DIST_VENUS() as venus,
    DIST_EARTH() as earth,
    DIST_MARS() as mars,
    DIST_JUPITER() as jupiter,
    DIST_SATURN() as saturn;

-- Physics Calculations
SELECT 
    '=== PHYSICS CALCULATIONS ===' as category,
    -- Schwarzschild radius of the Sun
    2 * G() * MASS_SUN() / (C() * C()) as sun_schwarzschild_radius,
    -- Escape velocity from Earth
    SQRT(2 * G() * MASS_EARTH() / RADIUS_EARTH()) as earth_escape_velocity,
    -- Orbital velocity at Earth's distance
    SQRT(G() * MASS_SUN() / DIST_EARTH()) as earth_orbital_velocity;

-- Particle Physics Constants
SELECT 
    '=== PARTICLE PHYSICS ===' as category,
    COULOMB() as coulomb_constant,
    BOHR() as bohr_radius,
    RY() as rydberg_constant,
    COMPTON() as compton_wavelength,
    STEFAN() as stefan_boltzmann;

-- Signed Particle Charges
SELECT 
    '=== PARTICLE CHARGES ===' as category,
    CHARGE_ELECTRON() as electron,
    CHARGE_PROTON() as proton,
    CHARGE_NEUTRON() as neutron,
    CHARGE_POSITRON() as positron,
    CHARGE_MUON() as muon,
    CHARGE_ANTIPROTON() as antiproton;

-- Mass-Energy Equivalence
SELECT 
    '=== MASS-ENERGY ===' as category,
    ME() * C() * C() as electron_energy,
    MP() * C() * C() as proton_energy,
    (MP() - ME()) * C() * C() as mass_difference_energy;

-- Astronomical Scales
SELECT 
    '=== SCALE COMPARISONS ===' as category,
    PARSEC() / LIGHTYEAR() as parsecs_per_lightyear,
    LIGHTYEAR() / AU() as au_per_lightyear,
    DIST_EARTH() / AU() as earth_orbit_in_au,
    RADIUS_SUN() / RADIUS_EARTH() as sun_earth_size_ratio,
    MASS_SUN() / MASS_EARTH() as sun_earth_mass_ratio;

-- Gravitational Parameters
SELECT 
    '=== GRAVITATIONAL PARAMETERS ===' as category,
    G() * MASS_SUN() as solar_gm,
    G() * MASS_EARTH() as earth_gm,
    G() * MASS_JUPITER() as jupiter_gm,
    G() * MASS_MOON() as moon_gm;

-- Solar System Lookup Functions (by name)
SELECT 
    '=== SOLAR BODY LOOKUPS ===' as category,
    MASS_SOLAR_BODY('Earth') as earth_mass,
    RADIUS_SOLAR_BODY('Jupiter') as jupiter_radius,
    DIST_SOLAR_BODY('Mars') as mars_distance,
    MASS_SOLAR_BODY('Sun') as sun_mass,
    RADIUS_SOLAR_BODY('Moon') as moon_radius;

-- Physical Ratios and Dimensionless Numbers
SELECT 
    '=== DIMENSIONLESS NUMBERS ===' as category,
    H() / (2 * PI()) as h_bar,
    E() * E() / (4 * PI() * 8.854e-12 * H() * C()) as fine_structure_approx,
    MP() / ME() as proton_electron_mass_ratio,
    C() / 299792458 as c_verification;

-- Energy Scales
SELECT 
    '=== ENERGY SCALES ===' as category,
    K() * 300 as room_temp_energy,
    H() * C() / 500e-9 as green_photon_energy,
    13.6 * E() as hydrogen_ionization_energy,
    RY() * H() * C() as rydberg_energy;

-- Summary
SELECT 
    'Summary' as info,
    'Complete physics and astronomy constants library' as description,
    'Includes fundamental constants, solar system data, and particle physics' as features;
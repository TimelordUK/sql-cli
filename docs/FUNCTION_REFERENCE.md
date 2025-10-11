# SQL CLI Function Reference

This document is auto-generated from the function registry.

## Aggregate Functions

### GROUP_NUM()

**Description:** Assigns unique sequential numbers (starting from 0) to distinct values

**Arguments:** 1 argument

**Returns:** Integer - unique number for each distinct value

**Examples:**
```sql
SELECT order_id, GROUP_NUM(order_id) as grp_num FROM orders
SELECT customer, GROUP_NUM(customer) as cust_num FROM sales
```

## Astronomical Functions

### AU()

**Description:** Returns one Astronomical Unit in meters (1.496 × 10^11)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT AU()
SELECT distance_m / AU() AS distance_au
```

### DENSITY_SOLAR_BODY()

**Description:** Returns the density in kg/m³

**Arguments:** 1 argument

**Returns:** FLOAT

**Examples:**
```sql
SELECT DENSITY_SOLAR_BODY('saturn')
SELECT body, DENSITY_SOLAR_BODY(body) FROM planets ORDER BY DENSITY_SOLAR_BODY(body)
```

### DISTANCE_SOLAR_BODY()

**Description:** Returns the mean distance from the Sun in meters

**Arguments:** 1 argument

**Returns:** FLOAT

**Examples:**
```sql
SELECT DISTANCE_SOLAR_BODY('mars')
SELECT DISTANCE_SOLAR_BODY('neptune') / AU() AS neptune_au
```

### DIST_JUPITER()

**Description:** Returns Jupiter's mean distance from the Sun in meters (7.786 × 10^11)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT DIST_JUPITER()
SELECT DIST_JUPITER() / AU() AS jupiter_au
```

### DIST_MARS()

**Description:** Returns Mars's mean distance from the Sun in meters (2.279 × 10^11)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT DIST_MARS()
SELECT DIST_MARS() / AU() AS mars_au
```

### DIST_MERCURY()

**Description:** Returns Mercury's mean distance from the Sun in meters (5.791 × 10^10)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT DIST_MERCURY()
SELECT DIST_MERCURY() / AU() AS mercury_au
```

### DIST_NEPTUNE()

**Description:** Returns Neptune's mean distance from the Sun in meters (4.4951 × 10^12)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT DIST_NEPTUNE()
SELECT DIST_NEPTUNE() / AU() AS neptune_au
```

### DIST_SATURN()

**Description:** Returns Saturn's mean distance from the Sun in meters (1.4335 × 10^12)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT DIST_SATURN()
SELECT DIST_SATURN() / AU() AS saturn_au
```

### DIST_URANUS()

**Description:** Returns Uranus's mean distance from the Sun in meters (2.8725 × 10^12)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT DIST_URANUS()
SELECT DIST_URANUS() / AU() AS uranus_au
```

### DIST_VENUS()

**Description:** Returns Venus's mean distance from the Sun in meters (1.082 × 10^11)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT DIST_VENUS()
SELECT DIST_VENUS() / AU() AS venus_au
```

### ESCAPE_VELOCITY_SOLAR_BODY()

**Description:** Returns the escape velocity in m/s

**Arguments:** 1 argument

**Returns:** FLOAT

**Examples:**
```sql
SELECT ESCAPE_VELOCITY_SOLAR_BODY('earth')
SELECT body, ESCAPE_VELOCITY_SOLAR_BODY(body) / 1000 AS km_per_s FROM planets
```

### GRAVITY_SOLAR_BODY()

**Description:** Returns the surface gravity in m/s²

**Arguments:** 1 argument

**Returns:** FLOAT

**Examples:**
```sql
SELECT GRAVITY_SOLAR_BODY('earth')
SELECT GRAVITY_SOLAR_BODY('moon')
SELECT body, GRAVITY_SOLAR_BODY(body) / 9.807 AS earth_g FROM planets
```

### LIGHT_YEAR()

**Description:** Returns one light year in meters (9.461 × 10^15)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT LIGHT_YEAR()
SELECT star_distance / LIGHT_YEAR() AS distance_ly
```

### MASS_EARTH()

**Description:** Returns Earth's mass in kg (5.972 × 10^24)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT MASS_EARTH()
SELECT asteroid_mass / MASS_EARTH() AS earth_masses
```

### MASS_JUPITER()

**Description:** Returns Jupiter's mass in kg (1.898 × 10^27)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT MASS_JUPITER()
SELECT exoplanet_mass / MASS_JUPITER() AS jupiter_masses
```

### MASS_MARS()

**Description:** Returns Mars's mass in kg (6.417 × 10^23)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT MASS_MARS()
```

### MASS_MERCURY()

**Description:** Returns Mercury's mass in kg (3.301 × 10^23)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT MASS_MERCURY()
```

### MASS_MOON()

**Description:** Returns the Moon's mass in kg (7.342 × 10^22)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT MASS_MOON()
SELECT satellite_mass / MASS_MOON() AS lunar_masses
```

### MASS_NEPTUNE()

**Description:** Returns Neptune's mass in kg (1.024 × 10^26)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT MASS_NEPTUNE()
```

### MASS_SATURN()

**Description:** Returns Saturn's mass in kg (5.683 × 10^26)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT MASS_SATURN()
```

### MASS_SOLAR_BODY()

**Description:** Returns the mass of a solar system body in kg

**Arguments:** 1 argument

**Returns:** FLOAT

**Examples:**
```sql
SELECT MASS_SOLAR_BODY('earth')
SELECT MASS_SOLAR_BODY('pluto')
SELECT body_name, MASS_SOLAR_BODY(body_name) FROM solar_system
```

### MASS_SUN()

**Description:** Returns the Sun's mass in kg (1.989 × 10^30)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT MASS_SUN()
SELECT star_mass / MASS_SUN() AS solar_masses
```

### MASS_URANUS()

**Description:** Returns Uranus's mass in kg (8.681 × 10^25)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT MASS_URANUS()
```

### MASS_VENUS()

**Description:** Returns Venus's mass in kg (4.867 × 10^24)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT MASS_VENUS()
```

### MOONS_SOLAR_BODY()

**Description:** Returns the number of known moons

**Arguments:** 1 argument

**Returns:** INTEGER

**Examples:**
```sql
SELECT MOONS_SOLAR_BODY('jupiter')
SELECT body, MOONS_SOLAR_BODY(body) FROM planets ORDER BY MOONS_SOLAR_BODY(body) DESC
```

### ORBITAL_PERIOD_SOLAR_BODY()

**Description:** Returns the orbital period in days

**Arguments:** 1 argument

**Returns:** FLOAT

**Examples:**
```sql
SELECT ORBITAL_PERIOD_SOLAR_BODY('earth')
SELECT ORBITAL_PERIOD_SOLAR_BODY('pluto') / 365.256 AS pluto_years
```

### PARSEC()

**Description:** Returns one parsec in meters (3.086 × 10^16)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT PARSEC()
SELECT galaxy_distance / PARSEC() AS distance_pc
```

### RADIUS_EARTH()

**Description:** Returns Earth's radius in meters (6.371 × 10^6)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT RADIUS_EARTH()
SELECT planet_radius / RADIUS_EARTH() AS earth_radii
```

### RADIUS_JUPITER()

**Description:** Returns Jupiter's radius in meters (6.991 × 10^7)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT RADIUS_JUPITER()
SELECT exoplanet_radius / RADIUS_JUPITER() AS jupiter_radii
```

### RADIUS_MARS()

**Description:** Returns Mars's radius in meters (3.390 × 10^6)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT RADIUS_MARS()
```

### RADIUS_MERCURY()

**Description:** Returns Mercury's radius in meters (2.440 × 10^6)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT RADIUS_MERCURY()
```

### RADIUS_MOON()

**Description:** Returns the Moon's radius in meters (1.737 × 10^6)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT RADIUS_MOON()
SELECT satellite_radius / RADIUS_MOON() AS lunar_radii
```

### RADIUS_NEPTUNE()

**Description:** Returns Neptune's radius in meters (2.462 × 10^7)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT RADIUS_NEPTUNE()
```

### RADIUS_SATURN()

**Description:** Returns Saturn's radius in meters (5.823 × 10^7)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT RADIUS_SATURN()
```

### RADIUS_SOLAR_BODY()

**Description:** Returns the radius of a solar system body in meters

**Arguments:** 1 argument

**Returns:** FLOAT

**Examples:**
```sql
SELECT RADIUS_SOLAR_BODY('earth')
SELECT RADIUS_SOLAR_BODY('jupiter')
```

### RADIUS_SUN()

**Description:** Returns the Sun's radius in meters (6.96 × 10^8)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT RADIUS_SUN()
SELECT star_radius / RADIUS_SUN() AS solar_radii
```

### RADIUS_URANUS()

**Description:** Returns Uranus's radius in meters (2.536 × 10^7)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT RADIUS_URANUS()
```

### RADIUS_VENUS()

**Description:** Returns Venus's radius in meters (6.052 × 10^6)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT RADIUS_VENUS()
```

### ROTATION_PERIOD_SOLAR_BODY()

**Description:** Returns the rotation period in hours

**Arguments:** 1 argument

**Returns:** FLOAT

**Examples:**
```sql
SELECT ROTATION_PERIOD_SOLAR_BODY('earth')
SELECT body, ROTATION_PERIOD_SOLAR_BODY(body) / 24 AS days FROM planets
```

## BigNumber Functions

### BIGADD()

**Description:** Add two arbitrary precision integers

**Arguments:** 2 arguments

**Returns:** String representation of sum

**Examples:**
```sql
SELECT BIGADD('999999999999999999999', '1')
SELECT BIGADD('123456789012345678901234567890', '987654321098765432109876543210')
```

### BIGFACT()

**Description:** Calculate factorial of a number (n!)

**Arguments:** 1 argument

**Returns:** String representation of factorial

**Examples:**
```sql
SELECT BIGFACT(50)     -- 50!
SELECT BIGFACT(100)    -- 100!
SELECT LENGTH(BIGFACT(1000))  -- How many digits in 1000!
```

### BIGINT()

**Description:** Convert value to arbitrary precision integer

**Arguments:** 1 argument

**Returns:** String representation of big integer

**Examples:**
```sql
SELECT BIGINT('123456789012345678901234567890')
SELECT BIGINT(2) ^ BIGINT(100)  -- 2^100
```

### BIGMUL()

**Description:** Multiply two arbitrary precision integers

**Arguments:** 2 arguments

**Returns:** String representation of product

**Examples:**
```sql
SELECT BIGMUL('999999999999999999999', '999999999999999999999')
SELECT BIGMUL('123456789', '987654321')
```

### BIGPOW()

**Description:** Raise big integer to a power (base^exponent)

**Arguments:** 2 arguments

**Returns:** String representation of result

**Examples:**
```sql
SELECT BIGPOW('2', 100)    -- 2^100
SELECT BIGPOW('10', 50)    -- 10^50
SELECT BIGPOW('99', 99)    -- 99^99
```

### BITAND()

**Description:** Bitwise AND of two integers

**Arguments:** 2 arguments

**Returns:** Result of bitwise AND

**Examples:**
```sql
SELECT BITAND(12, 10)                    -- 8 (1100 & 1010 = 1000)
SELECT TO_BINARY(BITAND(255, 15))       -- '1111'
```

### BITOR()

**Description:** Bitwise OR of two integers

**Arguments:** 2 arguments

**Returns:** Result of bitwise OR

**Examples:**
```sql
SELECT BITOR(12, 10)                     -- 14 (1100 | 1010 = 1110)
SELECT TO_BINARY(BITOR(8, 4))           -- '1100'
```

### BITSHIFT()

**Description:** Bit shift: left for positive shift, right for negative

**Arguments:** 2 arguments

**Returns:** Result of bit shift

**Examples:**
```sql
SELECT BITSHIFT(1, 10)                   -- 1024 (1 << 10)
SELECT BITSHIFT(1024, -10)               -- 1 (1024 >> 10)
SELECT TO_BINARY(BITSHIFT(1, 100))       -- 1 followed by 100 zeros
```

### BITXOR()

**Description:** Bitwise XOR of two integers

**Arguments:** 2 arguments

**Returns:** Result of bitwise XOR

**Examples:**
```sql
SELECT BITXOR(12, 10)                    -- 6 (1100 ^ 1010 = 0110)
SELECT TO_BINARY(BITXOR(255, 170))      -- '01010101'
```

### FROM_BINARY()

**Description:** Convert binary string to decimal number

**Arguments:** 1 argument

**Returns:** Decimal string representation

**Examples:**
```sql
SELECT FROM_BINARY('101010')             -- '42'
SELECT FROM_BINARY('11111111')           -- '255'
SELECT FROM_BINARY('1' || REPEAT('0', 100)) -- 2^100
```

### FROM_HEX()

**Description:** Convert hexadecimal string to decimal

**Arguments:** 1 argument

**Returns:** Decimal string representation

**Examples:**
```sql
SELECT FROM_HEX('FF')                    -- '255'
SELECT FROM_HEX('DEADBEEF')              -- '3735928559'
```

### TO_BINARY()

**Description:** Convert number to binary string representation

**Arguments:** 1 argument

**Returns:** Binary string (e.g., '1010' for 10)

**Examples:**
```sql
SELECT TO_BINARY(42)                     -- '101010'
SELECT TO_BINARY('255')                  -- '11111111'
SELECT TO_BINARY(BIGPOW('2', 100))      -- Binary of 2^100
```

### TO_HEX()

**Description:** Convert number to hexadecimal string

**Arguments:** 1 argument

**Returns:** Hexadecimal string (e.g., '2A' for 42)

**Examples:**
```sql
SELECT TO_HEX(255)                       -- 'ff'
SELECT TO_HEX(BIGPOW('16', 10))         -- '10000000000'
```

## Bitwise Functions

### BINARY_FORMAT()

**Description:** Formats binary string with separators for readability

**Arguments:** 1 to 3 arguments

**Returns:** Formatted binary string

**Examples:**
```sql
SELECT BINARY_FORMAT(255)           -- Returns '11111111'
SELECT BINARY_FORMAT(255, '_')      -- Returns '1111_1111' (groups of 4)
SELECT BINARY_FORMAT(65535, '_', 8) -- Returns '11111111_11111111' (groups of 8)
```

### BITNOT()

**Description:** Performs bitwise NOT operation (ones' complement)

**Arguments:** 1 argument

**Returns:** Integer result of ~a

**Examples:**
```sql
SELECT BITNOT(0)      -- Returns -1 (all bits set)
SELECT BITNOT(255)    -- Returns -256
SELECT BITNOT(-1)     -- Returns 0
```

### BIT_AND_STR()

**Description:** Performs bitwise AND on two binary strings

**Arguments:** 2 arguments

**Returns:** Binary string result

**Examples:**
```sql
SELECT BIT_AND_STR('1101', '1011')
SELECT BIT_AND_STR(TO_BINARY(13), TO_BINARY(11))
```

### BIT_COUNT()

**Description:** Counts the number of 1 bits in a binary string

**Arguments:** 1 argument

**Returns:** Integer count of 1 bits

**Examples:**
```sql
SELECT BIT_COUNT('11011010')
SELECT BIT_COUNT(TO_BINARY(218))
```

### BIT_FLIP()

**Description:** Flips all bits in a binary string (alias for BIT_NOT_STR)

**Arguments:** 1 argument

**Returns:** Binary string with bits flipped

**Examples:**
```sql
SELECT BIT_FLIP('11011010')
```

### BIT_NOT_STR()

**Description:** Performs bitwise NOT on a binary string

**Arguments:** 1 argument

**Returns:** Binary string with bits flipped

**Examples:**
```sql
SELECT BIT_NOT_STR('1100')
SELECT BIT_NOT_STR(TO_BINARY(12))
```

### BIT_OR_STR()

**Description:** Performs bitwise OR on two binary strings

**Arguments:** 2 arguments

**Returns:** Binary string result

**Examples:**
```sql
SELECT BIT_OR_STR('1100', '1010')
SELECT BIT_OR_STR(TO_BINARY(12), TO_BINARY(10))
```

### BIT_ROTATE_LEFT()

**Description:** Rotates a binary string left by N positions

**Arguments:** 2 arguments

**Returns:** Rotated binary string

**Examples:**
```sql
SELECT BIT_ROTATE_LEFT('11011010', 2)
SELECT BIT_ROTATE_LEFT(TO_BINARY(218), 3)
```

### BIT_ROTATE_RIGHT()

**Description:** Rotates a binary string right by N positions

**Arguments:** 2 arguments

**Returns:** Rotated binary string

**Examples:**
```sql
SELECT BIT_ROTATE_RIGHT('11011010', 2)
SELECT BIT_ROTATE_RIGHT(TO_BINARY(218), 3)
```

### BIT_SHIFT_LEFT()

**Description:** Shifts a binary string left by N positions, filling with zeros

**Arguments:** 2 arguments

**Returns:** Shifted binary string

**Examples:**
```sql
SELECT BIT_SHIFT_LEFT('11011010', 2)
SELECT BIT_SHIFT_LEFT(TO_BINARY(218), 3)
```

### BIT_SHIFT_RIGHT()

**Description:** Shifts a binary string right by N positions, filling with zeros

**Arguments:** 2 arguments

**Returns:** Shifted binary string

**Examples:**
```sql
SELECT BIT_SHIFT_RIGHT('11011010', 2)
SELECT BIT_SHIFT_RIGHT(TO_BINARY(218), 3)
```

### BIT_XOR_STR()

**Description:** Performs bitwise XOR on two binary strings

**Arguments:** 2 arguments

**Returns:** Binary string result

**Examples:**
```sql
SELECT BIT_XOR_STR('1100', '1010')
SELECT BIT_XOR_STR(TO_BINARY(12), TO_BINARY(10))
```

### COUNT_BITS()

**Description:** Counts the number of set bits (1s) in the binary representation

**Arguments:** 1 argument

**Returns:** Integer count of set bits

**Examples:**
```sql
SELECT COUNT_BITS(7)    -- Returns 3 (111 has three 1s)
SELECT COUNT_BITS(255)  -- Returns 8 (11111111 has eight 1s)
SELECT COUNT_BITS(16)   -- Returns 1 (10000 has one 1)
```

### HAMMING_DISTANCE()

**Description:** Counts the number of differing bits between two binary strings

**Arguments:** 2 arguments

**Returns:** Integer count of different bits

**Examples:**
```sql
SELECT HAMMING_DISTANCE('1101', '1011')
SELECT HAMMING_DISTANCE(TO_BINARY(13), TO_BINARY(11))
```

### HIGHEST_BIT()

**Description:** Returns the position of the highest set bit (0-indexed)

**Arguments:** 1 argument

**Returns:** Integer position of highest bit, or -1 if no bits set

**Examples:**
```sql
SELECT HIGHEST_BIT(8)    -- Returns 3 (bit 3 is set in 1000)
SELECT HIGHEST_BIT(255)  -- Returns 7 (bit 7 is highest in 11111111)
SELECT HIGHEST_BIT(0)    -- Returns -1 (no bits set)
```

### IS_POWER_OF_TWO()

**Description:** Checks if a number is an exact power of two using n & (n-1) == 0

**Arguments:** 1 argument

**Returns:** Boolean true if power of two, false otherwise

**Examples:**
```sql
SELECT IS_POWER_OF_TWO(16)  -- Returns true (2^4)
SELECT IS_POWER_OF_TWO(15)  -- Returns false
SELECT IS_POWER_OF_TWO(1)   -- Returns true (2^0)
SELECT IS_POWER_OF_TWO(0)   -- Returns false
```

### LOWEST_BIT()

**Description:** Returns the position of the lowest set bit (0-indexed)

**Arguments:** 1 argument

**Returns:** Integer position of lowest bit, or -1 if no bits set

**Examples:**
```sql
SELECT LOWEST_BIT(8)    -- Returns 3 (bit 3 is the only bit in 1000)
SELECT LOWEST_BIT(12)   -- Returns 2 (bit 2 is lowest in 1100)
SELECT LOWEST_BIT(0)    -- Returns -1 (no bits set)
```

### NEXT_POWER_OF_TWO()

**Description:** Returns the next power of two greater than or equal to n

**Arguments:** 1 argument

**Returns:** Integer that is the next power of two

**Examples:**
```sql
SELECT NEXT_POWER_OF_TWO(5)   -- Returns 8
SELECT NEXT_POWER_OF_TWO(16)  -- Returns 16 (already power of 2)
SELECT NEXT_POWER_OF_TWO(17)  -- Returns 32
```

## Chemical Functions

### ATOMIC_MASS()

**Description:** Returns the atomic mass of an element or molecular formula in amu

**Arguments:** 1 argument

**Returns:** FLOAT

**Examples:**
```sql
SELECT ATOMIC_MASS('H')
SELECT ATOMIC_MASS('Carbon')
SELECT ATOMIC_MASS('H2O') AS water_mass
SELECT ATOMIC_MASS('Ca(OH)2') AS calcium_hydroxide
SELECT ATOMIC_MASS('water') AS water_mass
```

### ATOMIC_NUMBER()

**Description:** Returns the atomic number of an element

**Arguments:** 1 argument

**Returns:** INTEGER

**Examples:**
```sql
SELECT ATOMIC_NUMBER('H')
SELECT ATOMIC_NUMBER('Carbon')
SELECT ATOMIC_NUMBER('Au') AS gold_number
```

### AVOGADRO()

**Description:** Returns Avogadro's number (6.022 × 10^23)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT AVOGADRO()
SELECT molecules / AVOGADRO() AS moles
```

### MOLECULE_FORMULA()

**Description:** Returns the molecular formula for a compound name

**Arguments:** 1 argument

**Returns:** STRING

**Examples:**
```sql
SELECT MOLECULE_FORMULA('water')
SELECT MOLECULE_FORMULA('glucose')
SELECT MOLECULE_FORMULA('caffeine')
```

### NEUTRONS()

**Description:** Returns the number of neutrons in the most common isotope

**Arguments:** 1 argument

**Returns:** INTEGER

**Examples:**
```sql
SELECT NEUTRONS('C')
SELECT NEUTRONS('U')
SELECT NEUTRONS('Gold')
```

## Constant Functions

### ALPHA()

**Description:** Returns the fine structure constant (dimensionless)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT ALPHA()
SELECT 1 / ALPHA() as inverse_fine_structure
```

### AMU()

**Description:** Returns the atomic mass unit (kg)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT AMU()
SELECT molecular_weight * AMU() as mass_kg
```

### BOHR()

**Description:** Returns the Bohr radius a₀ = 5.29177210903 × 10^-11 m

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT BOHR()
SELECT BOHR() * n * n as orbital_radius
```

### C()

**Description:** Returns the speed of light in vacuum (m/s)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT C()
SELECT distance / C() as light_time
```

### CHARGE_DOWN_QUARK()

**Description:** Returns the down quark charge in coulombs (-1/3 × 1.602176634 × 10^-19 C)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT CHARGE_DOWN_QUARK()
SELECT CHARGE_UP_QUARK() + 2 * CHARGE_DOWN_QUARK() AS neutron_charge
```

### CHARGE_ELECTRON()

**Description:** Returns the electron charge in coulombs (-1.602176634 × 10^-19 C)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT CHARGE_ELECTRON()
SELECT n_electrons * CHARGE_ELECTRON() AS total_charge
```

### CHARGE_MUON()

**Description:** Returns the muon charge in coulombs (-1.602176634 × 10^-19 C)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT CHARGE_MUON()
SELECT CHARGE_MUON() / CHARGE_ELECTRON() AS charge_ratio
```

### CHARGE_NEUTRON()

**Description:** Returns the neutron charge in coulombs (0 C)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT CHARGE_NEUTRON()
SELECT CHARGE_NEUTRON() + CHARGE_PROTON() AS net_nucleon_charge
```

### CHARGE_POSITRON()

**Description:** Returns the positron charge in coulombs (+1.602176634 × 10^-19 C)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT CHARGE_POSITRON()
SELECT CHARGE_ELECTRON() + CHARGE_POSITRON() AS annihilation_charge
```

### CHARGE_PROTON()

**Description:** Returns the proton charge in coulombs (+1.602176634 × 10^-19 C)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT CHARGE_PROTON()
SELECT n_protons * CHARGE_PROTON() AS nuclear_charge
```

### CHARGE_TAU()

**Description:** Returns the tau lepton charge in coulombs (-1.602176634 × 10^-19 C)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT CHARGE_TAU()
SELECT CHARGE_TAU() = CHARGE_ELECTRON() AS same_charge
```

### CHARGE_UP_QUARK()

**Description:** Returns the up quark charge in coulombs (+2/3 × 1.602176634 × 10^-19 C)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT CHARGE_UP_QUARK()
SELECT 2 * CHARGE_UP_QUARK() + CHARGE_DOWN_QUARK() AS proton_charge
```

### COULOMB()

**Description:** Returns Coulomb's constant k_e = 8.9875517873681764 × 10^9 N⋅m²⋅C⁻²

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT COULOMB()
SELECT COULOMB() * q1 * q2 / (r * r) as electric_force
```

### E()

**Description:** Returns Euler's number (e ≈ 2.71828)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT E()
SELECT POW(E(), x) AS exp_x
```

### E0()

**Description:** Returns the electric permittivity of vacuum (F/m)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT E0()
SELECT 1 / (4 * PI() * E0()) as coulomb_constant
```

### G()

**Description:** Returns the gravitational constant (m³ kg⁻¹ s⁻²)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT G()
SELECT G() * mass1 * mass2 / (r * r) as force
```

### H()

**Description:** Returns the Planck constant (J⋅s)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT H()
SELECT H() * frequency as energy
```

### HBAR()

**Description:** Returns the reduced Planck constant ħ in J⋅s (1.055 × 10⁻³⁴)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT HBAR()
SELECT HBAR() / (2 * PI()) as h_bar
```

### K()

**Description:** Returns the Boltzmann constant (J/K)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT K()
SELECT K() * temperature as thermal_energy
```

### MASS_ELECTRON()

**Description:** Alias for ME() - Returns the mass of an electron in kg

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT MASS_ELECTRON()
```

### ME()

**Description:** Returns the mass of an electron in kg (9.10938356 × 10^-31)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT ME()
SELECT mass / ME() AS electron_masses
```

### MN()

**Description:** Returns the neutron mass (kg)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT MN()
SELECT mass / MN() as neutron_masses
```

### MP()

**Description:** Returns the proton mass (kg)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT MP()
SELECT mass / MP() as proton_masses
```

### MU0()

**Description:** Returns the magnetic permeability of vacuum (N/A²)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT MU0()
SELECT SQRT(MU0() * E0()) as impedance
```

### PHI()

**Description:** Returns the golden ratio φ (1.61803...)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT PHI()
SELECT width * PHI() as golden_height
```

### PI()

**Description:** Returns the value of π (pi)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT PI()
SELECT radius * 2 * PI() AS circumference
```

### QE()

**Description:** Returns the elementary charge (C)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT QE()
SELECT n_electrons * QE() as total_charge
```

### R()

**Description:** Returns the universal gas constant (J mol⁻¹ K⁻¹)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT R()
SELECT pressure * volume / (R() * temperature) as moles
```

### RE()

**Description:** Returns the classical electron radius (m)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT RE()
SELECT radius / RE() as electron_radii
```

### RN()

**Description:** Returns the neutron radius (m)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT RN()
SELECT radius / RN() as neutron_radii
```

### RP()

**Description:** Returns the proton radius (m)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT RP()
SELECT radius / RP() as proton_radii
```

### RY()

**Description:** Returns the Rydberg constant (m⁻¹)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT RY()
SELECT 1 / RY() as rydberg_wavelength
```

### SIGMA()

**Description:** Returns the Stefan-Boltzmann constant (W m⁻² K⁻⁴)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT SIGMA()
SELECT SIGMA() * area * POWER(temp, 4) as radiated_power
```

### TAU()

**Description:** Returns tau (τ = 2π = 6.28318...)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT TAU()
SELECT radius * TAU() as circumference
```

## Conversion Functions

### CONVERT()

**Description:** Convert a value from one unit to another

**Arguments:** 3 arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT CONVERT(100, 'km', 'miles')
SELECT CONVERT(0, 'celsius', 'fahrenheit')
SELECT CONVERT(1, 'gallon', 'liters')
SELECT CONVERT(70, 'kg', 'pounds')
SELECT CONVERT(1, 'bar', 'psi')
```

### FROM_ROMAN()

**Description:** Convert Roman numerals to integer

**Arguments:** 1 argument

**Returns:** Integer value of Roman numeral

**Examples:**
```sql
SELECT FROM_ROMAN('MMXXIV')     -- Returns 2024
SELECT FROM_ROMAN('MCMLXXXIV')  -- Returns 1984
SELECT FROM_ROMAN('XLIX')       -- Returns 49
```

### TO_DECIMAL()

**Description:** Convert string to decimal/float

**Arguments:** 1 argument

**Returns:** Float value or NULL if conversion fails

**Examples:**
```sql
SELECT TO_DECIMAL('123.45')  -- Returns 123.45
SELECT TO_DECIMAL('123')  -- Returns 123.0
SELECT TO_DECIMAL('abc')  -- Returns NULL
```

### TO_INT()

**Description:** Convert string to integer

**Arguments:** 1 argument

**Returns:** Integer value or NULL if conversion fails

**Examples:**
```sql
SELECT TO_INT('123')  -- Returns 123
SELECT TO_INT('45.67')  -- Returns 45 (truncates)
SELECT TO_INT('abc')  -- Returns NULL
```

### TO_ROMAN()

**Description:** Convert integer to Roman numerals (1-3999)

**Arguments:** 1 argument

**Returns:** String with Roman numeral representation

**Examples:**
```sql
SELECT TO_ROMAN(2024)     -- Returns 'MMXXIV'
SELECT TO_ROMAN(1984)     -- Returns 'MCMLXXXIV'
SELECT TO_ROMAN(49)       -- Returns 'XLIX'
```

### TO_STRING()

**Description:** Convert any value to string representation

**Arguments:** 1 argument

**Returns:** String representation of the value

**Examples:**
```sql
SELECT TO_STRING(123)  -- Returns '123'
SELECT TO_STRING(45.67)  -- Returns '45.67'
SELECT TO_STRING(NULL)  -- Returns NULL
```

## Date Functions

### DATEADD()

**Description:** Add a specified interval to a date

**Arguments:** 3 arguments

**Returns:** DATETIME

**Examples:**
```sql
SELECT DATEADD('day', 7, '2024-01-01')
SELECT DATEADD('month', -1, NOW())
SELECT DATEADD('year', 1, hire_date) FROM employees
```

### DATEDIFF()

**Description:** Calculate the difference between two dates in the specified unit

**Arguments:** 3 arguments

**Returns:** INTEGER

**Examples:**
```sql
SELECT DATEDIFF('day', '2024-01-01', '2024-01-15')
SELECT DATEDIFF('month', start_date, end_date) FROM projects
SELECT DATEDIFF('year', birth_date, TODAY()) as age
```

### DATETIME()

**Description:** Create datetime from components: (year, month, day, [hour], [minute], [second], [is_utc])

**Arguments:** 3 to 7 arguments

**Returns:** DATETIME

**Examples:**
```sql
SELECT DATETIME(2024, 1, 15)
SELECT DATETIME(2024, 1, 15, 14, 30, 0)
SELECT DATETIME(2024, 12, 31, 23, 59, 59)
-- Note: All times are interpreted as UTC
```

### DAY()

**Description:** Returns the day of month from a date (1-31)

**Arguments:** 1 argument

**Returns:** INTEGER

**Examples:**
```sql
SELECT DAY('2024-03-15')
SELECT DAY(NOW())
```

### DAYNAME()

**Description:** Returns day name. Optional second arg: 'full' (default) or 'short'

**Arguments:** 1 to 2 arguments

**Returns:** STRING

**Examples:**
```sql
SELECT DAYNAME('2024-01-01')
SELECT DAYNAME('2024-01-01', 'short')
SELECT DAYNAME(NOW(), 'full')
```

### DAYOFWEEK()

**Description:** Returns day of week as number (0=Sunday, 6=Saturday)

**Arguments:** 1 argument

**Returns:** INTEGER

**Examples:**
```sql
SELECT DAYOFWEEK('2024-01-01')
SELECT DAYOFWEEK(NOW())
SELECT DAYOFWEEK(date_column) FROM table
```

### FORMAT_DATE()

**Description:** Format a date using a format string

**Arguments:** 2 arguments

**Returns:** STRING

**Examples:**
```sql
SELECT FORMAT_DATE(NOW(), '%Y-%m-%d')
SELECT FORMAT_DATE(NOW(), '%B %d, %Y')
SELECT FORMAT_DATE(NOW(), '%Y%m%d_%H%M%S')
```

### FROM_UNIXTIME()

**Description:** Convert Unix epoch timestamp to datetime string

**Arguments:** 1 argument

**Returns:** DATETIME string in ISO format

**Examples:**
```sql
SELECT FROM_UNIXTIME(1704067200)
SELECT FROM_UNIXTIME(timestamp_col) FROM data
```

### ISLEAPYEAR()

**Description:** Returns true if the year is a leap year

**Arguments:** 1 argument

**Returns:** BOOLEAN

**Examples:**
```sql
SELECT ISLEAPYEAR('2024-01-01')
SELECT ISLEAPYEAR(2024)
SELECT ISLEAPYEAR(2023)
```

### MONTH()

**Description:** Returns the month from a date (1-12)

**Arguments:** 1 argument

**Returns:** INTEGER

**Examples:**
```sql
SELECT MONTH('2024-03-15')
SELECT MONTH(NOW())
```

### MONTHNAME()

**Description:** Returns month name. Optional second arg: 'full' (default) or 'short'

**Arguments:** 1 to 2 arguments

**Returns:** STRING

**Examples:**
```sql
SELECT MONTHNAME('2024-01-15')
SELECT MONTHNAME('2024-01-15', 'short')
```

### NOW()

**Description:** Returns the current date and time

**Arguments:** no arguments

**Returns:** DATETIME

**Examples:**
```sql
SELECT NOW()
SELECT * FROM orders WHERE created_at > NOW() - 7
```

### PARSE_DATETIME()

**Description:** Parse datetime string with custom format (uses chrono strftime format)

**Arguments:** 2 arguments

**Returns:** DATETIME

**Examples:**
```sql
SELECT PARSE_DATETIME('15/01/2024', '%d/%m/%Y')
SELECT PARSE_DATETIME('Jan 15, 2024 14:30', '%b %d, %Y %H:%M')
SELECT PARSE_DATETIME('2024-01-15T14:30:00', '%Y-%m-%dT%H:%M:%S')
SELECT PARSE_DATETIME(date_string, '%Y%m%d-%H:%M:%S%.3f') FROM data -- FIX format
```

### PARSE_DATETIME_UTC()

**Description:** Parse datetime as UTC. With 1 arg: auto-detect format. With 2 args: use custom format

**Arguments:** 1 to 2 arguments

**Returns:** DATETIME (UTC)

**Examples:**
```sql
SELECT PARSE_DATETIME_UTC('2024-01-15 14:30:00')
SELECT PARSE_DATETIME_UTC('20250925-14:52:15.567') -- FIX format auto-detected
SELECT PARSE_DATETIME_UTC('15/01/2024 14:30', '%d/%m/%Y %H:%M')
```

### QUARTER()

**Description:** Returns the quarter of the year (1-4)

**Arguments:** 1 argument

**Returns:** INTEGER

**Examples:**
```sql
SELECT QUARTER('2024-01-15')
SELECT QUARTER('2024-07-01')
```

### TIME_BUCKET()

**Description:** Round timestamp down to bucket boundary (for time-based grouping)

**Arguments:** 2 arguments

**Returns:** INTEGER (bucket timestamp)

**Examples:**
```sql
SELECT TIME_BUCKET(300, UNIX_TIMESTAMP(trade_time)) as bucket FROM trades -- 5 minute buckets
SELECT TIME_BUCKET(3600, UNIX_TIMESTAMP(trade_time)) as hour FROM trades -- 1 hour buckets
SELECT TIME_BUCKET(60, timestamp_col) as minute FROM data -- 1 minute buckets
```

### TODAY()

**Description:** Returns today's date

**Arguments:** no arguments

**Returns:** DATE

**Examples:**
```sql
SELECT TODAY()
SELECT * FROM events WHERE event_date = TODAY()
```

### UNIX_TIMESTAMP()

**Description:** Convert datetime to Unix epoch timestamp (seconds since 1970-01-01 00:00:00 UTC)

**Arguments:** 1 argument

**Returns:** INTEGER (seconds since epoch)

**Examples:**
```sql
SELECT UNIX_TIMESTAMP('2024-01-01 00:00:00')
SELECT UNIX_TIMESTAMP('2024-01-01T12:30:45')
SELECT UNIX_TIMESTAMP(trade_time) FROM trades
```

### WEEKOFYEAR()

**Description:** Returns the ISO week number of the year (1-53)

**Arguments:** 1 argument

**Returns:** INTEGER

**Examples:**
```sql
SELECT WEEKOFYEAR('2024-01-01')
SELECT WEEKOFYEAR(NOW())
```

### YEAR()

**Description:** Returns the year from a date

**Arguments:** 1 argument

**Returns:** INTEGER

**Examples:**
```sql
SELECT YEAR('2024-03-15')
SELECT YEAR(NOW())
```

## Mathematical Functions

### ABS()

**Description:** Returns the absolute value of a number

**Arguments:** 1 argument

**Returns:** NUMBER

**Examples:**
```sql
SELECT ABS(-5)
SELECT ABS(3.14)
SELECT ABS(price - cost) FROM products
```

### ACOS()

**Description:** Returns the arccosine (inverse cosine) in radians. Input must be between -1 and 1

**Arguments:** 1 argument

**Returns:** FLOAT

**Examples:**
```sql
SELECT ACOS(1)
SELECT ACOS(0)
SELECT DEGREES(ACOS(0.5))
```

### ASIN()

**Description:** Returns the arcsine (inverse sine) in radians. Input must be between -1 and 1

**Arguments:** 1 argument

**Returns:** FLOAT

**Examples:**
```sql
SELECT ASIN(0)
SELECT ASIN(1)
SELECT DEGREES(ASIN(0.5))
```

### ATAN()

**Description:** Returns the arctangent (inverse tangent) in radians

**Arguments:** 1 argument

**Returns:** FLOAT

**Examples:**
```sql
SELECT ATAN(0)
SELECT ATAN(1)
SELECT DEGREES(ATAN(1))
```

### ATAN2()

**Description:** Returns the arctangent of y/x in radians, using signs to determine quadrant

**Arguments:** 2 arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT ATAN2(1, 1)
SELECT ATAN2(1, 0)
SELECT DEGREES(ATAN2(-1, -1))
```

### BYTE_MAX()

**Description:** Maximum value for unsigned byte (255)

**Arguments:** no arguments

**Returns:** Integer value

**Examples:**
```sql
SELECT BYTE_MAX()  -- Returns 255
```

### BYTE_MIN()

**Description:** Minimum value for unsigned byte (0)

**Arguments:** no arguments

**Returns:** Integer value

**Examples:**
```sql
SELECT BYTE_MIN()  -- Returns 0
```

### CEIL()

**Description:** Alias for CEILING - Returns the smallest integer greater than or equal to the value

**Arguments:** 1 argument

**Returns:** INTEGER

**Examples:**
```sql
SELECT CEIL(3.2)
SELECT CEIL(-2.7)
SELECT CEIL(5)
```

### CEILING()

**Description:** Returns the smallest integer greater than or equal to the value

**Arguments:** 1 argument

**Returns:** INTEGER

**Examples:**
```sql
SELECT CEILING(3.2)
SELECT CEILING(-2.7)
SELECT CEILING(5)
```

### CHAR_MAX()

**Description:** Maximum value for signed char (127)

**Arguments:** no arguments

**Returns:** Integer value

**Examples:**
```sql
SELECT CHAR_MAX()  -- Returns 127
```

### CHAR_MIN()

**Description:** Minimum value for signed char (-128)

**Arguments:** no arguments

**Returns:** Integer value

**Examples:**
```sql
SELECT CHAR_MIN()  -- Returns -128
```

### CIRCLE_AREA()

**Description:** Calculate area of a circle given radius

**Arguments:** 1 argument

**Returns:** Float (area)

**Examples:**
```sql
CIRCLE_AREA(1) = 3.14159...
CIRCLE_AREA(5) = 78.5398...
```

### CIRCLE_CIRCUMFERENCE()

**Description:** Calculate circumference of a circle given radius

**Arguments:** 1 argument

**Returns:** Float (circumference)

**Examples:**
```sql
CIRCLE_CIRCUMFERENCE(1) = 6.28318...
CIRCLE_CIRCUMFERENCE(5) = 31.4159...
```

### COALESCE()

**Description:** Returns the first non-null value from a list

**Arguments:** any number of arguments

**Returns:** ANY

**Examples:**
```sql
SELECT COALESCE(NULL, 'default', 'backup')
SELECT COALESCE(phone, mobile, email) as contact FROM users
SELECT COALESCE(discount, 0) as discount_amount
```

### COS()

**Description:** Returns the cosine of an angle in radians

**Arguments:** 1 argument

**Returns:** FLOAT

**Examples:**
```sql
SELECT COS(0)
SELECT COS(PI())
SELECT COS(RADIANS(60))
```

### COSH()

**Description:** Returns the hyperbolic cosine of a number

**Arguments:** 1 argument

**Returns:** FLOAT

**Examples:**
```sql
SELECT COSH(0)
SELECT COSH(1)
SELECT COSH(-1)
```

### COT()

**Description:** Returns the cotangent of an angle in radians (1/tan)

**Arguments:** 1 argument

**Returns:** FLOAT

**Examples:**
```sql
SELECT COT(PI()/4)
SELECT COT(PI()/2)
SELECT COT(RADIANS(45))
```

### DEGREES()

**Description:** Convert radians to degrees

**Arguments:** 1 argument

**Returns:** FLOAT

**Examples:**
```sql
SELECT DEGREES(PI())
SELECT DEGREES(PI()/2)
SELECT DEGREES(1)
```

### DISTANCE_2D()

**Description:** Calculate distance between two points (x1, y1) and (x2, y2)

**Arguments:** 4 arguments

**Returns:** Float (distance)

**Examples:**
```sql
DISTANCE_2D(0, 0, 3, 4) = 5.0
DISTANCE_2D(1, 1, 4, 5) = 5.0
```

### EXP()

**Description:** Returns e raised to the power of the given number

**Arguments:** 1 argument

**Returns:** FLOAT

**Examples:**
```sql
SELECT EXP(1)
SELECT EXP(0)
SELECT EXP(LN(10))
```

### FACTORIAL()

**Description:** Returns the factorial of a non-negative integer (n!)

**Arguments:** 1 argument

**Returns:** INTEGER

**Examples:**
```sql
SELECT FACTORIAL(5)
SELECT FACTORIAL(10)
SELECT FACTORIAL(0)
```

### FIBONACCI()

**Description:** Returns the nth Fibonacci number (0, 1, 1, 2, 3, 5, 8, ...)

**Arguments:** 1 argument

**Returns:** Integer - nth Fibonacci number

**Examples:**
```sql
SELECT FIBONACCI(7) -- Returns 13
SELECT FIBONACCI(10) -- Returns 55
```

### FLOOR()

**Description:** Returns the largest integer less than or equal to the value

**Arguments:** 1 argument

**Returns:** INTEGER

**Examples:**
```sql
SELECT FLOOR(3.7)
SELECT FLOOR(-2.3)
SELECT FLOOR(5)
```

### FROM_BASE()

**Description:** Convert a string in the specified base (2-36) to a number

**Arguments:** 2 arguments

**Returns:** Numeric value

**Examples:**
```sql
SELECT FROM_BASE('ff', 16)     -- Returns 255
SELECT FROM_BASE('101010', 2)  -- Returns 42
SELECT FROM_BASE('144', 8)     -- Returns 100
```

### FROM_BINARY()

**Description:** Convert binary string to decimal number

**Arguments:** 1 argument

**Returns:** Decimal string representation

**Examples:**
```sql
SELECT FROM_BINARY('101010')             -- '42'
SELECT FROM_BINARY('11111111')           -- '255'
SELECT FROM_BINARY('1' || REPEAT('0', 100)) -- 2^100
```

### FROM_HEX()

**Description:** Convert hexadecimal string to decimal

**Arguments:** 1 argument

**Returns:** Decimal string representation

**Examples:**
```sql
SELECT FROM_HEX('FF')                    -- '255'
SELECT FROM_HEX('DEADBEEF')              -- '3735928559'
```

### FROM_OCTAL()

**Description:** Convert an octal string to a number

**Arguments:** 1 argument

**Returns:** Numeric value

**Examples:**
```sql
SELECT FROM_OCTAL('10')   -- Returns 8
SELECT FROM_OCTAL('100')  -- Returns 64
SELECT FROM_OCTAL('777')  -- Returns 511
```

### GEOMETRIC()

**Description:** Geometric series sum: a + ar + ar² + ... + ar^(n-1)

**Arguments:** 3 arguments

**Returns:** Float - sum of geometric series

**Examples:**
```sql
SELECT GEOMETRIC(1, 2, 5) -- Returns 31 (1 + 2 + 4 + 8 + 16)
SELECT GEOMETRIC(1, 0.5, 10) -- Returns 1.998... (converging series)
```

### GREATEST()

**Description:** Returns the greatest value from a list of values

**Arguments:** any number of arguments

**Returns:** ANY

**Examples:**
```sql
SELECT GREATEST(10, 20, 5)
SELECT GREATEST(salary, bonus, commission) as max_pay FROM employees
SELECT GREATEST('apple', 'banana', 'cherry')
SELECT GREATEST(date1, date2, date3) as latest_date
```

### GREATEST_LABEL()

**Description:** Returns the label associated with the greatest value from label/value pairs

**Arguments:** any number of arguments

**Returns:** STRING

**Examples:**
```sql
SELECT GREATEST_LABEL('earth', MASS_EARTH(), 'sun', MASS_SUN()) as bigger_body
SELECT GREATEST_LABEL('jan', 100, 'feb', 150, 'mar', 120) as best_month
SELECT GREATEST_LABEL('product_a', sales_a, 'product_b', sales_b) as top_product
```

### HARMONIC()

**Description:** Harmonic series: 1 + 1/2 + 1/3 + ... + 1/n

**Arguments:** 1 argument

**Returns:** Float - sum of harmonic series

**Examples:**
```sql
SELECT HARMONIC(4) -- Returns 2.0833... (1 + 0.5 + 0.333... + 0.25)
SELECT HARMONIC(10) -- Returns 2.9290...
```

### IIF()

**Description:** Returns second argument if first is true, third if false

**Arguments:** 3 arguments

**Returns:** ANY

**Examples:**
```sql
SELECT IIF(1 > 0, 'positive', 'negative')
SELECT IIF(MASS_SUN() > MASS_EARTH(), 'sun', 'earth') as bigger
SELECT IIF(price > 100, 'expensive', 'affordable') as price_category
```

### INT16_MAX()

**Description:** Maximum value for signed 16-bit integer (32767)

**Arguments:** no arguments

**Returns:** Integer value

**Examples:**
```sql
SELECT INT16_MAX()  -- Returns 32767
```

### INT16_MIN()

**Description:** Minimum value for signed 16-bit integer (-32768)

**Arguments:** no arguments

**Returns:** Integer value

**Examples:**
```sql
SELECT INT16_MIN()  -- Returns -32768
```

### INT32_MAX()

**Description:** Maximum value for signed 32-bit integer (2147483647)

**Arguments:** no arguments

**Returns:** Integer value

**Examples:**
```sql
SELECT INT32_MAX()  -- Returns 2147483647
```

### INT32_MIN()

**Description:** Minimum value for signed 32-bit integer (-2147483648)

**Arguments:** no arguments

**Returns:** Integer value

**Examples:**
```sql
SELECT INT32_MIN()  -- Returns -2147483648
```

### INT64_MAX()

**Description:** Maximum value for signed 64-bit integer (9223372036854775807)

**Arguments:** no arguments

**Returns:** Integer value

**Examples:**
```sql
SELECT INT64_MAX()  -- Returns 9223372036854775807
```

### INT64_MIN()

**Description:** Minimum value for signed 64-bit integer (-9223372036854775808)

**Arguments:** no arguments

**Returns:** Integer value

**Examples:**
```sql
SELECT INT64_MIN()  -- Returns -9223372036854775808
```

### INT8_MAX()

**Description:** Maximum value for signed 8-bit integer (127)

**Arguments:** no arguments

**Returns:** Integer value

**Examples:**
```sql
SELECT INT8_MAX()  -- Returns 127
```

### INT8_MIN()

**Description:** Minimum value for signed 8-bit integer (-128)

**Arguments:** no arguments

**Returns:** Integer value

**Examples:**
```sql
SELECT INT8_MIN()  -- Returns -128
```

### INT_MAX()

**Description:** Maximum value for signed 32-bit int (2147483647)

**Arguments:** no arguments

**Returns:** Integer value

**Examples:**
```sql
SELECT INT_MAX()  -- Returns 2147483647
```

### INT_MIN()

**Description:** Minimum value for signed 32-bit int (-2147483648)

**Arguments:** no arguments

**Returns:** Integer value

**Examples:**
```sql
SELECT INT_MIN()  -- Returns -2147483648
```

### IS_PRIME()

**Description:** Returns true if the number is prime, false otherwise

**Arguments:** 1 argument

**Returns:** BOOLEAN

**Examples:**
```sql
SELECT IS_PRIME(17)
SELECT IS_PRIME(100)
SELECT IS_PRIME(104729)
```

### LEAST()

**Description:** Returns the smallest value from a list of values

**Arguments:** any number of arguments

**Returns:** ANY

**Examples:**
```sql
SELECT LEAST(10, 20, 5)
SELECT LEAST(salary, min_wage) as lower_bound FROM employees
SELECT LEAST('apple', 'banana', 'cherry')
SELECT LEAST(date1, date2, date3) as earliest_date
```

### LEAST_LABEL()

**Description:** Returns the label associated with the smallest value from label/value pairs

**Arguments:** any number of arguments

**Returns:** STRING

**Examples:**
```sql
SELECT LEAST_LABEL('mercury', MASS_MERCURY(), 'earth', MASS_EARTH()) as smaller_planet
SELECT LEAST_LABEL('jan', 100, 'feb', 150, 'mar', 120) as worst_month
SELECT LEAST_LABEL('cost_a', 50, 'cost_b', 30) as cheapest_option
```

### LN()

**Description:** Returns the natural logarithm (base e) of a number

**Arguments:** 1 argument

**Returns:** FLOAT

**Examples:**
```sql
SELECT LN(2.718282)
SELECT LN(10)
SELECT LN(1)
```

### LOG()

**Description:** Returns the logarithm of a number (base 10 by default, or specified base)

**Arguments:** 1 to 2 arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT LOG(100)
SELECT LOG(8, 2)
SELECT LOG(1000, 10)
```

### LOG10()

**Description:** Returns the base-10 logarithm of a number

**Arguments:** 1 argument

**Returns:** FLOAT

**Examples:**
```sql
SELECT LOG10(100)
SELECT LOG10(1000)
SELECT LOG10(0.1)
```

### LOG_RETURNS()

**Description:** Calculate logarithmic returns from current and previous price

**Arguments:** 2 arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT LOG_RETURNS(close, LAG(close) OVER (ORDER BY date)) FROM stocks
SELECT LOG_RETURNS(100, 95)
```

### LONG_MAX()

**Description:** Maximum value for signed 64-bit long (9223372036854775807)

**Arguments:** no arguments

**Returns:** Integer value

**Examples:**
```sql
SELECT LONG_MAX()  -- Returns 9223372036854775807
```

### LONG_MIN()

**Description:** Minimum value for signed 64-bit long (-9223372036854775808)

**Arguments:** no arguments

**Returns:** Integer value

**Examples:**
```sql
SELECT LONG_MIN()  -- Returns -9223372036854775808
```

### MOD()

**Description:** Returns the remainder of division

**Arguments:** 2 arguments

**Returns:** NUMBER

**Examples:**
```sql
SELECT MOD(10, 3)
SELECT MOD(15, 4)
SELECT MOD(id, 100) FROM table
```

### NEXT_PRIME()

**Description:** Returns the smallest prime number >= n

**Arguments:** 1 argument

**Returns:** INTEGER

**Examples:**
```sql
SELECT NEXT_PRIME(100)
SELECT NEXT_PRIME(97)
SELECT NEXT_PRIME(1000)
```

### NTH_PRIME()

**Description:** Returns the Nth prime number (1-indexed) - alias for PRIME

**Arguments:** 1 argument

**Returns:** INTEGER

**Examples:**
```sql
SELECT NTH_PRIME(1)
SELECT NTH_PRIME(100)
SELECT NTH_PRIME(10000)
```

### NULLIF()

**Description:** Returns NULL if two values are equal, otherwise returns the first value

**Arguments:** 2 arguments

**Returns:** ANY

**Examples:**
```sql
SELECT NULLIF(0, 0)
SELECT NULLIF(price, 0) as non_zero_price
SELECT NULLIF(status, 'DELETED') as active_status
```

### POW()

**Description:** Returns a number raised to a power (alias for POWER)

**Arguments:** 2 arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT POW(2, 3)
SELECT POW(10, -2)
SELECT POW(9, 0.5)
```

### POWER()

**Description:** Returns a number raised to a power

**Arguments:** 2 arguments

**Returns:** NUMBER

**Examples:**
```sql
SELECT POWER(2, 3)
SELECT POWER(10, -2)
SELECT POWER(9, 0.5)
```

### PREV_PRIME()

**Description:** Returns the largest prime number <= n

**Arguments:** 1 argument

**Returns:** INTEGER

**Examples:**
```sql
SELECT PREV_PRIME(100)
SELECT PREV_PRIME(97)
SELECT PREV_PRIME(1000)
```

### PRIME()

**Description:** Returns the Nth prime number (1-indexed)

**Arguments:** 1 argument

**Returns:** INTEGER

**Examples:**
```sql
SELECT PRIME(1)
SELECT PRIME(100)
SELECT PRIME(10000)
```

### PRIME_COUNT()

**Description:** Returns the count of prime numbers up to n (π(n))

**Arguments:** 1 argument

**Returns:** INTEGER

**Examples:**
```sql
SELECT PRIME_COUNT(10)
SELECT PRIME_COUNT(100)
SELECT PRIME_COUNT(1000)
```

### PRIME_PI()

**Description:** Returns the count of prime numbers up to n (π(n)) - alias for PRIME_COUNT

**Arguments:** 1 argument

**Returns:** INTEGER

**Examples:**
```sql
SELECT PRIME_PI(10)
SELECT PRIME_PI(100)
SELECT PRIME_PI(1000)
```

### PYTHAGORAS()

**Description:** Calculate hypotenuse using Pythagorean theorem

**Arguments:** 2 arguments

**Returns:** Float (hypotenuse length)

**Examples:**
```sql
PYTHAGORAS(3, 4) = 5.0
PYTHAGORAS(5, 12) = 13.0
```

### QUOTIENT()

**Description:** Returns the integer portion of division

**Arguments:** 2 arguments

**Returns:** INTEGER

**Examples:**
```sql
SELECT QUOTIENT(10, 3)
SELECT QUOTIENT(15, 4)
SELECT QUOTIENT(100, 7)
```

### RADIANS()

**Description:** Convert degrees to radians

**Arguments:** 1 argument

**Returns:** FLOAT

**Examples:**
```sql
SELECT RADIANS(180)
SELECT RADIANS(90)
SELECT RADIANS(45)
```

### RANDOM()

**Description:** Generate a random float between 0 and 1

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT RANDOM()
SELECT ROUND(RANDOM() * 100, 0)
```

### RAND_INT()

**Description:** Generate a random integer between lower and upper bounds (inclusive)

**Arguments:** 2 arguments

**Returns:** INTEGER

**Examples:**
```sql
SELECT RAND_INT(1, 6)
SELECT RAND_INT(1, 100)
SELECT RAND_INT(0, 255)
```

### RAND_RANGE()

**Description:** Generate N random numbers between lower and upper bounds

**Arguments:** 3 arguments

**Returns:** TABLE

**Examples:**
```sql
SELECT * FROM RAND_RANGE(10, 1, 100)
SELECT * FROM RAND_RANGE(5, 0.0, 1.0)
SELECT AVG(value) FROM RAND_RANGE(1000, 1, 6)
```

### RETURNS()

**Description:** Calculate returns from current and previous price

**Arguments:** 2 arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT RETURNS(close, LAG(close) OVER (ORDER BY date)) FROM stocks
SELECT RETURNS(100, 95)
SELECT RETURNS(95, 100)
```

### ROUND()

**Description:** Round a number to specified decimal places

**Arguments:** 1 to 2 arguments

**Returns:** NUMBER

**Examples:**
```sql
SELECT ROUND(3.14159, 2)
SELECT ROUND(123.456)
SELECT ROUND(1234.5, -2)
```

### SHARPE_RATIO()

**Description:** Calculate Sharpe ratio: (mean_return - risk_free_rate) / volatility

**Arguments:** 3 arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT SHARPE_RATIO(0.08, 0.02, 0.15)
SELECT SHARPE_RATIO(mean_return, 0.02, volatility) FROM portfolio_stats
```

### SHORT_MAX()

**Description:** Maximum value for signed short (32767)

**Arguments:** no arguments

**Returns:** Integer value

**Examples:**
```sql
SELECT SHORT_MAX()  -- Returns 32767
```

### SHORT_MIN()

**Description:** Minimum value for signed short (-32768)

**Arguments:** no arguments

**Returns:** Integer value

**Examples:**
```sql
SELECT SHORT_MIN()  -- Returns -32768
```

### SIN()

**Description:** Returns the sine of an angle in radians

**Arguments:** 1 argument

**Returns:** FLOAT

**Examples:**
```sql
SELECT SIN(0)
SELECT SIN(PI()/2)
SELECT SIN(RADIANS(30))
```

### SINH()

**Description:** Returns the hyperbolic sine of a number

**Arguments:** 1 argument

**Returns:** FLOAT

**Examples:**
```sql
SELECT SINH(0)
SELECT SINH(1)
SELECT SINH(-1)
```

### SPHERE_SURFACE_AREA()

**Description:** Calculate surface area of a sphere given radius

**Arguments:** 1 argument

**Returns:** Float (surface area)

**Examples:**
```sql
SPHERE_SURFACE_AREA(1) = 12.5663...
SPHERE_SURFACE_AREA(3) = 113.097...
```

### SPHERE_VOLUME()

**Description:** Calculate volume of a sphere given radius

**Arguments:** 1 argument

**Returns:** Float (volume)

**Examples:**
```sql
SPHERE_VOLUME(1) = 4.18879...
SPHERE_VOLUME(3) = 113.097...
```

### SQRT()

**Description:** Returns the square root of a number

**Arguments:** 1 argument

**Returns:** FLOAT

**Examples:**
```sql
SELECT SQRT(16)
SELECT SQRT(2)
SELECT SQRT(area) FROM squares
```

### STDDEV()

**Description:** Calculate sample standard deviation

**Arguments:** any number of arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT STDDEV(1, 2, 3, 4, 5)
SELECT STDDEV(returns) OVER (ORDER BY date ROWS 19 PRECEDING) FROM stocks
```

### SUM_N()

**Description:** Calculate sum of first n natural numbers (triangular number)

**Arguments:** 1 argument

**Returns:** INTEGER

**Examples:**
```sql
SELECT SUM_N(10)
SELECT SUM_N(100)
SELECT SUM_N(5)
```

### SUM_N_CUBE()

**Description:** Sum of cubes from 1 to n: 1³ + 2³ + ... + n³

**Arguments:** 1 argument

**Returns:** Integer - sum of cubes ([n(n+1)/2]²)

**Examples:**
```sql
SELECT SUM_N_CUBE(4) -- Returns 100 (1+8+27+64)
SELECT SUM_N_CUBE(5) -- Returns 225
```

### SUM_N_SQR()

**Description:** Sum of squares from 1 to n: 1² + 2² + ... + n²

**Arguments:** 1 argument

**Returns:** Integer - sum of squares (n(n+1)(2n+1)/6)

**Examples:**
```sql
SELECT SUM_N_SQR(5) -- Returns 55 (1+4+9+16+25)
SELECT SUM_N_SQR(10) -- Returns 385
```

### TAN()

**Description:** Returns the tangent of an angle in radians

**Arguments:** 1 argument

**Returns:** FLOAT

**Examples:**
```sql
SELECT TAN(0)
SELECT TAN(PI()/4)
SELECT TAN(RADIANS(45))
```

### TANH()

**Description:** Returns the hyperbolic tangent of a number

**Arguments:** 1 argument

**Returns:** FLOAT

**Examples:**
```sql
SELECT TANH(0)
SELECT TANH(1)
SELECT TANH(-1)
```

### TO_BASE()

**Description:** Convert a number to a string in the specified base (2-36)

**Arguments:** 2 arguments

**Returns:** String representation in the specified base

**Examples:**
```sql
SELECT TO_BASE(255, 16)  -- Returns 'ff'
SELECT TO_BASE(42, 2)    -- Returns '101010'
SELECT TO_BASE(100, 8)   -- Returns '144'
```

### TO_BINARY()

**Description:** Convert number to binary string representation

**Arguments:** 1 argument

**Returns:** Binary string (e.g., '1010' for 10)

**Examples:**
```sql
SELECT TO_BINARY(42)                     -- '101010'
SELECT TO_BINARY('255')                  -- '11111111'
SELECT TO_BINARY(BIGPOW('2', 100))      -- Binary of 2^100
```

### TO_HEX()

**Description:** Convert number to hexadecimal string

**Arguments:** 1 argument

**Returns:** Hexadecimal string (e.g., '2A' for 42)

**Examples:**
```sql
SELECT TO_HEX(255)                       -- 'ff'
SELECT TO_HEX(BIGPOW('16', 10))         -- '10000000000'
```

### TO_OCTAL()

**Description:** Convert a number to octal string representation

**Arguments:** 1 argument

**Returns:** Octal string

**Examples:**
```sql
SELECT TO_OCTAL(8)    -- Returns '10'
SELECT TO_OCTAL(64)   -- Returns '100'
SELECT TO_OCTAL(511)  -- Returns '777'
```

### TRIANGLE_AREA()

**Description:** Calculate area of a triangle given three side lengths (Heron's formula)

**Arguments:** 3 arguments

**Returns:** Float (area)

**Examples:**
```sql
TRIANGLE_AREA(3, 4, 5) = 6.0
TRIANGLE_AREA(5, 5, 6) = 12.0
```

### UINT16_MAX()

**Description:** Maximum value for unsigned 16-bit integer (65535)

**Arguments:** no arguments

**Returns:** Integer value

**Examples:**
```sql
SELECT UINT16_MAX()  -- Returns 65535
```

### UINT32_MAX()

**Description:** Maximum value for unsigned 32-bit integer (4294967295)

**Arguments:** no arguments

**Returns:** Integer value

**Examples:**
```sql
SELECT UINT32_MAX()  -- Returns 4294967295
```

### UINT8_MAX()

**Description:** Maximum value for unsigned 8-bit integer (255)

**Arguments:** no arguments

**Returns:** Integer value

**Examples:**
```sql
SELECT UINT8_MAX()  -- Returns 255
```

### VOLATILITY()

**Description:** Calculate volatility (standard deviation) of returns

**Arguments:** any number of arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT VOLATILITY(0.01, -0.02, 0.015, -0.005, 0.008)
WITH returns AS (SELECT RETURNS(close, LAG(close) OVER (ORDER BY date)) as r FROM stocks) SELECT VOLATILITY(r) FROM returns
```

## Statistical Functions

### CORR()

**Description:** Calculates the Pearson correlation coefficient between two columns (aggregate function)

**Arguments:** 2 arguments

**Returns:** Numeric value between -1 and 1

**Examples:**
```sql
SELECT CORR(height, weight) FROM people
SELECT CORR(temperature, ice_cream_sales) FROM daily_data
SELECT month, CORR(advertising_spend, revenue) FROM sales GROUP BY month
```

### KURTOSIS()

**Description:** Calculate kurtosis contribution: ((x - mean) / stddev)^4

**Arguments:** 3 arguments

**Returns:** Kurtosis contribution of a single value

**Examples:**
```sql
SELECT KURTOSIS(value, mean, stddev) FROM measurements
SELECT KURTOSIS(100, 85, 15) -- returns 1.0 for value one std dev above mean
```

### MEDIAN()

**Description:** Returns the median (middle value) of a numeric column (aggregate function)

**Arguments:** 1 argument

**Returns:** Numeric value representing the median

**Examples:**
```sql
SELECT MEDIAN(salary) FROM employees
SELECT department, MEDIAN(age) FROM users GROUP BY department
```

### MODE()

**Description:** Returns the most frequently occurring value (aggregate function)

**Arguments:** 1 argument

**Returns:** The mode value (most frequent)

**Examples:**
```sql
SELECT MODE(category) FROM products
SELECT MODE(rating) FROM reviews
SELECT department, MODE(job_level) FROM employees GROUP BY department
```

### PERCENTILE()

**Description:** Returns the nth percentile of values (0-100) (aggregate function)

**Arguments:** 2 arguments

**Returns:** Numeric value at the specified percentile

**Examples:**
```sql
SELECT PERCENTILE(score, 75) FROM tests
SELECT PERCENTILE(income, 50) FROM users
SELECT PERCENTILE(response_time, 95) FROM requests
```

### SKEW()

**Description:** Calculate skewness contribution: ((x - mean) / stddev)^3

**Arguments:** 3 arguments

**Returns:** Skewness contribution of a single value

**Examples:**
```sql
SELECT SKEW(value, mean, stddev) FROM measurements
SELECT SKEW(100, 85, 15) -- returns ~1.0 for value one std dev above mean
```

### VARIANCE()

**Description:** Calculates the population variance of numeric values (aggregate function)

**Arguments:** 1 argument

**Returns:** Numeric value representing the variance

**Examples:**
```sql
SELECT VARIANCE(price) FROM products
SELECT category, VARIANCE(quantity) FROM inventory GROUP BY category
```

### VAR_POP()

**Description:** Calculates the population variance (same as VARIANCE) (aggregate function)

**Arguments:** 1 argument

**Returns:** Numeric value representing the population variance

**Examples:**
```sql
SELECT VAR_POP(amount) FROM transactions
```

### VAR_SAMP()

**Description:** Calculates the sample variance of numeric values (aggregate function)

**Arguments:** 1 argument

**Returns:** Numeric value representing the sample variance

**Examples:**
```sql
SELECT VAR_SAMP(score) FROM test_results
SELECT class, VAR_SAMP(height) FROM students GROUP BY class
```

## String Functions

### ASCII()

**Description:** Get ASCII/Unicode code point of first character

**Arguments:** 1 argument

**Returns:** Integer code point of the first character

**Examples:**
```sql
SELECT ASCII('A')  -- Returns 65
SELECT ASCII('ABC')  -- Returns 65 (first char only)
SELECT ASCII('€')  -- Returns 8364 (Euro symbol)
```

### CENTER()

**Description:** Center a string within a specified width

**Arguments:** 2 to 3 arguments

**Returns:** STRING

**Examples:**
```sql
SELECT CENTER('hello', 11)
SELECT CENTER('test', 10, '.')
SELECT CENTER('SQL', 7, '-')
```

### CHAR()

**Description:** Convert ASCII/Unicode code to character (alias for CHR)

**Arguments:** 1 argument

**Returns:** Character corresponding to the code

**Examples:**
```sql
SELECT CHAR(65)  -- Returns 'A'
SELECT CHAR(8364)  -- Returns '€' (Euro symbol)
```

### CHR()

**Description:** Convert ASCII code to character

**Arguments:** 1 argument

**Returns:** STRING

**Examples:**
```sql
SELECT CHR(65)
SELECT CHR(97)
SELECT CHR(48)
```

### CLEAN_TEXT()

**Description:** Clean text by normalizing whitespace and removing control characters

**Arguments:** 1 argument

**Returns:** Cleaned string

**Examples:**
```sql
SELECT CLEAN_TEXT('  hello   world  ')              -- Returns 'hello world'
SELECT CLEAN_TEXT('line1\nline2\tline3')            -- Returns 'line1 line2 line3'
```

### CONTAINS()

**Description:** Checks if string contains substring

**Arguments:** 2 arguments

**Returns:** BOOLEAN

**Examples:**
```sql
SELECT * FROM users WHERE name.Contains('john')
SELECT CONTAINS(name, 'john') FROM users
```

### DECODE()

**Description:** Decode base64 or hex encoded string

**Arguments:** 2 arguments

**Returns:** Decoded string

**Examples:**
```sql
SELECT DECODE('SGVsbG8=', 'base64')  -- Returns 'Hello'
SELECT DECODE('48656c6c6f', 'hex')  -- Returns 'Hello'
```

### EDIT_DISTANCE()

**Description:** Calculate the Levenshtein edit distance between two strings

**Arguments:** 2 arguments

**Returns:** INTEGER

**Examples:**
```sql
SELECT EDIT_DISTANCE('kitten', 'sitting')
SELECT EDIT_DISTANCE(name, 'John') FROM users
SELECT * FROM users WHERE EDIT_DISTANCE(name, 'Smith') <= 2
```

### ENCODE()

**Description:** Encode string to base64 or hex

**Arguments:** 2 arguments

**Returns:** Encoded string

**Examples:**
```sql
SELECT ENCODE('Hello', 'base64')  -- Returns 'SGVsbG8='
SELECT ENCODE('Hello', 'hex')  -- Returns '48656c6c6f'
```

### ENDSWITH()

**Description:** Checks if string ends with suffix

**Arguments:** 2 arguments

**Returns:** BOOLEAN

**Examples:**
```sql
SELECT * FROM users WHERE email.EndsWith('.com')
SELECT ENDSWITH(email, '.com') FROM users
```

### EXTRACT_WORDS()

**Description:** Extract words from text, with optional min length and case conversion

**Arguments:** 1 to 3 arguments

**Returns:** Comma-separated list of words

**Examples:**
```sql
SELECT EXTRACT_WORDS('Hello, world!')               -- Returns 'Hello,world'
SELECT EXTRACT_WORDS('The quick fox', 4)            -- Returns 'quick'
SELECT EXTRACT_WORDS('The Fox', 2, 'lower')         -- Returns 'the,fox'
```

### FORMAT_CURRENCY()

**Description:** Format numbers as currency with symbols, codes, or names. Currency can be from a column.

**Arguments:** 2 to 4 arguments

**Returns:** STRING

**Examples:**
```sql
SELECT FORMAT_CURRENCY(1234.56, 'USD')
SELECT FORMAT_CURRENCY(1234.56, 'GBP', 'symbol')
SELECT FORMAT_CURRENCY(amount, currency_code)
SELECT FORMAT_CURRENCY(3000, 'GBP', 'compact_code')
SELECT FORMAT_CURRENCY(1234.56, 'EUR', 'eu')
```

### FORMAT_NUMBER()

**Description:** Format a number with decimal places and thousand separators

**Arguments:** 1 to 3 arguments

**Returns:** STRING

**Examples:**
```sql
SELECT FORMAT_NUMBER(1234567.89, 2)
SELECT FORMAT_NUMBER(1234.5, 2, false)
SELECT FORMAT_NUMBER(1234567)
```

### FREQUENCY()

**Description:** Count occurrences of a substring within a string

**Arguments:** 2 arguments

**Returns:** INTEGER

**Examples:**
```sql
SELECT FREQUENCY('hello world', 'o')
SELECT FREQUENCY('mississippi', 'ss')
SELECT FREQUENCY(text_column, 'error') FROM logs
SELECT name, FREQUENCY(name, 'a') as a_count FROM users
```

### INDEXOF()

**Description:** Returns the position of the first occurrence of a substring (0-based)

**Arguments:** 2 arguments

**Returns:** INTEGER

**Examples:**
```sql
SELECT email.IndexOf('@') FROM users
SELECT INDEXOF(email, '@') FROM users
```

### INITCAP()

**Description:** Capitalizes the first letter of each word

**Arguments:** 1 argument

**Returns:** String with each word capitalized

**Examples:**
```sql
SELECT INITCAP('hello world') -- returns 'Hello World'
SELECT INITCAP('sql-cli is great') -- returns 'Sql-Cli Is Great'
```

### INSTR()

**Description:** Returns the position of the first occurrence of a substring (1-based, SQL standard)

**Arguments:** 2 arguments

**Returns:** INTEGER

**Examples:**
```sql
SELECT INSTR(email, '@') FROM users
SELECT SUBSTRING(email, INSTR(email, '@') + 1) FROM users
```

### IS_BOOL()

**Description:** Check if a value can be parsed as a boolean

**Arguments:** 1 argument

**Returns:** Boolean

**Examples:**
```sql
SELECT * FROM data WHERE IS_BOOL(column)
SELECT IS_BOOL('true')
SELECT IS_BOOL('123')
```

### IS_DATE()

**Description:** Check if a value can be parsed as a date/datetime

**Arguments:** 1 argument

**Returns:** Boolean

**Examples:**
```sql
SELECT * FROM data WHERE IS_DATE(column)
SELECT IS_DATE('2024-01-01')
SELECT IS_DATE('not a date')
```

### IS_FLOAT()

**Description:** Check if a value can be parsed as a float

**Arguments:** 1 argument

**Returns:** Boolean

**Examples:**
```sql
SELECT * FROM data WHERE IS_FLOAT(column)
SELECT IS_FLOAT('123.45')
SELECT IS_FLOAT('123')
```

### IS_INTEGER()

**Description:** Check if a value can be parsed as an integer

**Arguments:** 1 argument

**Returns:** Boolean

**Examples:**
```sql
SELECT * FROM data WHERE IS_INTEGER(column)
SELECT IS_INTEGER('123')
SELECT IS_INTEGER('123.45')
```

### IS_NOT_NULL()

**Description:** Check if a value is not NULL

**Arguments:** 1 argument

**Returns:** Boolean

**Examples:**
```sql
SELECT * FROM data WHERE IS_NOT_NULL(column)
SELECT IS_NOT_NULL('value')
SELECT IS_NOT_NULL(NULL)
```

### IS_NULL()

**Description:** Check if a value is NULL

**Arguments:** 1 argument

**Returns:** Boolean

**Examples:**
```sql
SELECT * FROM data WHERE IS_NULL(column)
SELECT IS_NULL(NULL)
SELECT IS_NULL('value')
```

### IS_NUMERIC()

**Description:** Check if a value can be parsed as a number (integer or float)

**Arguments:** 1 argument

**Returns:** Boolean

**Examples:**
```sql
SELECT * FROM data WHERE IS_NUMERIC(column)
SELECT IS_NUMERIC('123.45')
SELECT IS_NUMERIC('not a number')
```

### LEFT()

**Description:** Returns leftmost n characters from string

**Arguments:** 2 arguments

**Returns:** STRING

**Examples:**
```sql
SELECT LEFT(email, 5) FROM users
SELECT LEFT('hello@world', INSTR('hello@world', '@') - 1)
```

### LENGTH()

**Description:** Returns the length of a string

**Arguments:** 1 argument

**Returns:** INTEGER

**Examples:**
```sql
SELECT name.Length() FROM users
SELECT LENGTH(name) FROM users
```

### LOREM_IPSUM()

**Description:** Generate Lorem Ipsum placeholder text with specified number of words

**Arguments:** 1 to 3 arguments

**Returns:** STRING

**Examples:**
```sql
SELECT LOREM_IPSUM(10)
SELECT LOREM_IPSUM(50)
SELECT LOREM_IPSUM(20, 1)
SELECT LOREM_IPSUM(15, 0, id)
```

### LOWER()

**Description:** Convert string to lowercase

**Arguments:** 1 argument

**Returns:** STRING

**Examples:**
```sql
SELECT LOWER('HELLO')
SELECT LOWER(name) FROM table
```

### LPAD()

**Description:** Left pad a string to a specified length with a fill character

**Arguments:** 2 to 3 arguments

**Returns:** STRING

**Examples:**
```sql
SELECT LPAD('123', 5)
SELECT LPAD('123', 5, '0')
SELECT LPAD('hello', 10, '.')
```

### LPAD()

**Description:** Left pad a string to a specified length with a fill character

**Arguments:** 2 to 3 arguments

**Returns:** STRING

**Examples:**
```sql
SELECT LPAD('123', 5)
SELECT LPAD('123', 5, '0')
SELECT LPAD('hello', 10, '.')
```

### MD5()

**Description:** Calculate MD5 hash of a string

**Arguments:** 1 argument

**Returns:** String (32 character hex digest)

**Examples:**
```sql
MD5('hello') = '5d41402abc4b2a76b9719d911017c592'
MD5('test') = '098f6bcd4621d373cade4e832627b4f6'
```

### MID()

**Description:** Extract substring from text (1-based indexing)

**Arguments:** 3 arguments

**Returns:** STRING

**Examples:**
```sql
SELECT MID('Hello', 1, 3)
SELECT MID('World', 2, 3)
SELECT MID(name, 1, 5) FROM table
```

### MORSE_CODE()

**Description:** Converts text to Morse code

**Arguments:** 1 argument

**Returns:** Morse code representation

**Examples:**
```sql
SELECT MORSE_CODE('SOS') -- returns '... --- ...'
SELECT MORSE_CODE('HELLO') -- returns '.... . .-.. .-.. ---'
SELECT MORSE_CODE('SQL 123') -- returns '... --.- .-..   .---- ..--- ...--'
```

### ORD()

**Description:** Get Unicode code point of first character (alias for ASCII)

**Arguments:** 1 argument

**Returns:** Integer code point of the first character

**Examples:**
```sql
SELECT ORD('A')  -- Returns 65
SELECT ORD('€')  -- Returns 8364
```

### PIG_LATIN()

**Description:** Converts text to Pig Latin

**Arguments:** 1 argument

**Returns:** Pig Latin version of the text

**Examples:**
```sql
SELECT PIG_LATIN('hello') -- returns 'ellohay'
SELECT PIG_LATIN('apple') -- returns 'appleway'
SELECT PIG_LATIN('SQL') -- returns 'SQLay'
SELECT PIG_LATIN('hello world') -- returns 'ellohay orldway'
```

### PROPER()

**Description:** Alias for INITCAP - capitalizes first letter of each word

**Arguments:** 1 argument

**Returns:** String with each word capitalized

**Examples:**
```sql
SELECT PROPER('hello world') -- returns 'Hello World'
```

### RENDER_NUMBER()

**Description:** Format numbers with separators, abbreviations, or regional formats

**Arguments:** 1 to 3 arguments

**Returns:** STRING

**Examples:**
```sql
SELECT RENDER_NUMBER(1234567.89)
SELECT RENDER_NUMBER(1234567.89, 'compact')
SELECT RENDER_NUMBER(1234.56, 'eu')
SELECT RENDER_NUMBER(-1234.56, 'accounting')
SELECT RENDER_NUMBER(1500000, 'compact', 1)
```

### REPEAT()

**Description:** Repeat a string n times

**Arguments:** 2 arguments

**Returns:** String containing the input repeated n times

**Examples:**
```sql
SELECT REPEAT('*', 5)  -- Returns '*****'
SELECT REPEAT('ab', 3)  -- Returns 'ababab'
SELECT REPEAT('=', COUNT(*) / 10) FROM table  -- Create histogram
```

### REPLACE()

**Description:** Replaces all occurrences of a substring

**Arguments:** 3 arguments

**Returns:** STRING

**Examples:**
```sql
SELECT name.Replace('John', 'Jane') FROM users
SELECT REPLACE(name, 'John', 'Jane') FROM users
```

### REVERSE()

**Description:** Reverses the characters in a string

**Arguments:** 1 argument

**Returns:** Reversed string

**Examples:**
```sql
SELECT REVERSE('hello') -- returns 'olleh'
SELECT REVERSE('SQL CLI') -- returns 'ILC LQS'
```

### RIGHT()

**Description:** Returns rightmost n characters from string

**Arguments:** 2 arguments

**Returns:** STRING

**Examples:**
```sql
SELECT RIGHT(filename, 4) FROM files
SELECT RIGHT(email, LENGTH(email) - INSTR(email, '@'))
```

### ROT13()

**Description:** Applies ROT13 encoding (shifts letters by 13 positions)

**Arguments:** 1 argument

**Returns:** ROT13 encoded string

**Examples:**
```sql
SELECT ROT13('hello') -- returns 'uryyb'
SELECT ROT13('uryyb') -- returns 'hello' (ROT13 is reversible)
SELECT ROT13('SQL123') -- returns 'FDY123' (numbers unchanged)
```

### RPAD()

**Description:** Right pad a string to a specified length with a fill character

**Arguments:** 2 to 3 arguments

**Returns:** STRING

**Examples:**
```sql
SELECT RPAD('123', 5)
SELECT RPAD('123', 5, '0')
SELECT RPAD('hello', 10, '.')
```

### RPAD()

**Description:** Right pad a string to a specified length with a fill character

**Arguments:** 2 to 3 arguments

**Returns:** STRING

**Examples:**
```sql
SELECT RPAD('123', 5)
SELECT RPAD('123', 5, '0')
SELECT RPAD('hello', 10, '.')
```

### SCRAMBLE()

**Description:** Scrambles letters in words (keeps first and last letter)

**Arguments:** 1 argument

**Returns:** Scrambled text that's still somewhat readable

**Examples:**
```sql
SELECT SCRAMBLE('hello') -- might return 'hlelo'
SELECT SCRAMBLE('according') -- might return 'acdorcnig'
SELECT SCRAMBLE('The quick brown fox') -- scrambles each word
```

### SHA1()

**Description:** Calculate SHA1 hash of a string

**Arguments:** 1 argument

**Returns:** String (40 character hex digest)

**Examples:**
```sql
SHA1('hello') = '2aae6c35c94fcfb415dbe95f408b9ce91ee846ed'
SHA1('test') = 'a94a8fe5ccb19ba61c4c0873d391e987982fbbd3'
```

### SHA256()

**Description:** Calculate SHA256 hash of a string

**Arguments:** 1 argument

**Returns:** String (64 character hex digest)

**Examples:**
```sql
SHA256('hello') = '2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824'
SHA256('test') = '9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08'
```

### SHA512()

**Description:** Calculate SHA512 hash of a string

**Arguments:** 1 argument

**Returns:** String (128 character hex digest)

**Examples:**
```sql
SHA512('hello') = '9b71d224bd62f3785d96d46ad3ea3d73319bfbc2890caadae2dff72519673ca72323c3d99ba5c11d7c7acc6e14b8c5da0c4663475c2e5c3adef46f73bcdec043'
SHA512('test') = 'ee26b0dd4af7e749aa1a8ee3c10ae9923f618980772e473f8819a5d4940e0db27ac185f8a0e1d5f84f88bc887fd67b143732c304cc5fa9ad8e6f57f50028a8ff'
```

### SOUNDEX()

**Description:** Returns the Soundex code for phonetic matching

**Arguments:** 1 argument

**Returns:** 4-character Soundex code

**Examples:**
```sql
SELECT SOUNDEX('Smith') -- returns 'S530'
SELECT SOUNDEX('Smythe') -- returns 'S530' (sounds similar)
SELECT SOUNDEX('Johnson') -- returns 'J525'
SELECT SOUNDEX('Jonson') -- returns 'J525' (sounds similar)
```

### SPLIT_PART()

**Description:** Returns the nth part of a string split by delimiter (1-based index)

**Arguments:** 3 arguments

**Returns:** STRING

**Examples:**
```sql
SELECT SPLIT_PART('a.b.c.d', '.', 2)
SELECT SPLIT_PART(email, '@', 1) FROM users
```

### STARTSWITH()

**Description:** Checks if string starts with prefix

**Arguments:** 2 arguments

**Returns:** BOOLEAN

**Examples:**
```sql
SELECT * FROM users WHERE name.StartsWith('John')
SELECT STARTSWITH(name, 'John') FROM users
```

### STRIP_PUNCTUATION()

**Description:** Remove punctuation from text, optionally replacing with a character

**Arguments:** 1 to 2 arguments

**Returns:** String with punctuation removed

**Examples:**
```sql
SELECT STRIP_PUNCTUATION('Hello, world!')           -- Returns 'Hello world'
SELECT STRIP_PUNCTUATION('foo.bar@baz', ' ')        -- Returns 'foo bar baz'
SELECT STRIP_PUNCTUATION('data-file_v2.0', '')      -- Returns 'datafilev20'
```

### SUBSTRING()

**Description:** Extracts substring from string

**Arguments:** 2 to 3 arguments

**Returns:** STRING

**Examples:**
```sql
SELECT name.Substring(0, 5) FROM users
SELECT SUBSTRING(name, 0, 5) FROM users
```

### SUBSTRING_AFTER()

**Description:** Returns substring after the first (or nth) occurrence of delimiter

**Arguments:** 2 to 3 arguments

**Returns:** STRING

**Examples:**
```sql
SELECT SUBSTRING_AFTER(email, '@') FROM users
SELECT SUBSTRING_AFTER('a.b.c.d', '.', 2)
```

### SUBSTRING_BEFORE()

**Description:** Returns substring before the first (or nth) occurrence of delimiter

**Arguments:** 2 to 3 arguments

**Returns:** STRING

**Examples:**
```sql
SELECT SUBSTRING_BEFORE(email, '@') FROM users
SELECT SUBSTRING_BEFORE('a.b.c.d', '.', 2)
```

### TEXTJOIN()

**Description:** Join multiple text values with a delimiter

**Arguments:** any number of arguments

**Returns:** STRING

**Examples:**
```sql
SELECT TEXTJOIN(',', 1, 'a', 'b', 'c')
SELECT TEXTJOIN(' - ', 1, name, city) FROM table
SELECT TEXTJOIN('|', 0, col1, col2, col3) FROM table
```

### TOKENIZE()

**Description:** Split text into words, removing punctuation and optionally converting case

**Arguments:** 1 to 2 arguments

**Returns:** Space-separated words

**Examples:**
```sql
SELECT TOKENIZE('Hello, world!')                    -- Returns 'Hello world'
SELECT TOKENIZE('The quick brown fox.', 'lower')    -- Returns 'the quick brown fox'
SELECT TOKENIZE('foo-bar_baz', 'upper')             -- Returns 'FOO BAR BAZ'
```

### TOLOWER()

**Description:** Converts string to lowercase

**Arguments:** 1 argument

**Returns:** STRING

**Examples:**
```sql
SELECT name.ToLower() FROM users
SELECT TOLOWER(name) FROM users
```

### TOUPPER()

**Description:** Converts string to uppercase

**Arguments:** 1 argument

**Returns:** STRING

**Examples:**
```sql
SELECT name.ToUpper() FROM users
SELECT TOUPPER(name) FROM users
```

### TO_CAMEL_CASE()

**Description:** Converts text to camelCase

**Arguments:** 1 argument

**Returns:** String in camelCase format

**Examples:**
```sql
SELECT TO_CAMEL_CASE('snake_case') -- returns 'snakeCase'
SELECT TO_CAMEL_CASE('kebab-case') -- returns 'kebabCase'
SELECT TO_CAMEL_CASE('PascalCase') -- returns 'pascalCase'
SELECT TO_CAMEL_CASE('hello world') -- returns 'helloWorld'
```

### TO_CONSTANT_CASE()

**Description:** Converts text to CONSTANT_CASE (SCREAMING_SNAKE_CASE)

**Arguments:** 1 argument

**Returns:** String in CONSTANT_CASE format

**Examples:**
```sql
SELECT TO_CONSTANT_CASE('camelCase') -- returns 'CAMEL_CASE'
SELECT TO_CONSTANT_CASE('kebab-case') -- returns 'KEBAB_CASE'
SELECT TO_CONSTANT_CASE('hello world') -- returns 'HELLO_WORLD'
```

### TO_KEBAB_CASE()

**Description:** Converts text to kebab-case

**Arguments:** 1 argument

**Returns:** String in kebab-case format

**Examples:**
```sql
SELECT TO_KEBAB_CASE('snake_case') -- returns 'snake-case'
SELECT TO_KEBAB_CASE('CamelCase') -- returns 'camel-case'
SELECT TO_KEBAB_CASE('PascalCase') -- returns 'pascal-case'
SELECT TO_KEBAB_CASE('hello world') -- returns 'hello-world'
```

### TO_ORDINAL()

**Description:** Convert number to ordinal form (1st, 2nd, 3rd, etc.)

**Arguments:** 1 argument

**Returns:** String with ordinal representation

**Examples:**
```sql
SELECT TO_ORDINAL(1)       -- Returns '1st'
SELECT TO_ORDINAL(2)       -- Returns '2nd'
SELECT TO_ORDINAL(21)      -- Returns '21st'
SELECT TO_ORDINAL(123)     -- Returns '123rd'
```

### TO_ORDINAL_WORDS()

**Description:** Convert number to ordinal words (first, second, third, etc.)

**Arguments:** 1 argument

**Returns:** String with ordinal words

**Examples:**
```sql
SELECT TO_ORDINAL_WORDS(1)   -- Returns 'first'
SELECT TO_ORDINAL_WORDS(21)  -- Returns 'twenty-first'
SELECT TO_ORDINAL_WORDS(100) -- Returns 'one hundredth'
```

### TO_PASCAL_CASE()

**Description:** Converts text to PascalCase

**Arguments:** 1 argument

**Returns:** String in PascalCase format

**Examples:**
```sql
SELECT TO_PASCAL_CASE('snake_case') -- returns 'SnakeCase'
SELECT TO_PASCAL_CASE('kebab-case') -- returns 'KebabCase'
SELECT TO_PASCAL_CASE('camelCase') -- returns 'CamelCase'
SELECT TO_PASCAL_CASE('hello world') -- returns 'HelloWorld'
```

### TO_SNAKE_CASE()

**Description:** Converts text to snake_case

**Arguments:** 1 argument

**Returns:** String in snake_case format

**Examples:**
```sql
SELECT TO_SNAKE_CASE('CamelCase') -- returns 'camel_case'
SELECT TO_SNAKE_CASE('PascalCase') -- returns 'pascal_case'
SELECT TO_SNAKE_CASE('kebab-case') -- returns 'kebab_case'
SELECT TO_SNAKE_CASE('HTTPResponse') -- returns 'http_response'
SELECT TO_SNAKE_CASE('XMLHttpRequest') -- returns 'xml_http_request'
```

### TO_WORDS()

**Description:** Convert number to English words

**Arguments:** 1 argument

**Returns:** String with number spelled out in words

**Examples:**
```sql
SELECT TO_WORDS(42)        -- Returns 'forty-two'
SELECT TO_WORDS(999)       -- Returns 'nine hundred ninety-nine'
SELECT TO_WORDS(1234)      -- Returns 'one thousand two hundred thirty-four'
SELECT TO_WORDS(1000000)   -- Returns 'one million'
```

### TRIM()

**Description:** Removes leading and trailing whitespace

**Arguments:** 1 argument

**Returns:** STRING

**Examples:**
```sql
SELECT name.Trim() FROM users
SELECT TRIM(name) FROM users
```

### TRIM()

**Description:** Removes leading and trailing whitespace

**Arguments:** 1 argument

**Returns:** STRING

**Examples:**
```sql
SELECT name.Trim() FROM users
SELECT TRIM(name) FROM users
```

### TRIMEND()

**Description:** Removes trailing whitespace

**Arguments:** 1 argument

**Returns:** STRING

**Examples:**
```sql
SELECT name.TrimEnd() FROM users
SELECT TRIMEND(name) FROM users
```

### TRIMSTART()

**Description:** Removes leading whitespace

**Arguments:** 1 argument

**Returns:** STRING

**Examples:**
```sql
SELECT name.TrimStart() FROM users
SELECT TRIMSTART(name) FROM users
```

### UNICODE()

**Description:** Get Unicode code points of all characters as comma-separated list

**Arguments:** 1 argument

**Returns:** Comma-separated list of Unicode code points

**Examples:**
```sql
SELECT UNICODE('ABC')  -- Returns '65,66,67'
SELECT UNICODE('€$')  -- Returns '8364,36'
SELECT UNICODE('Hello')  -- Returns '72,101,108,108,111'
```

### UPPER()

**Description:** Convert string to uppercase

**Arguments:** 1 argument

**Returns:** STRING

**Examples:**
```sql
SELECT UPPER('hello')
SELECT UPPER(name) FROM table
```

### WORD_COUNT()

**Description:** Count the number of words in text

**Arguments:** 1 argument

**Returns:** Integer count of words

**Examples:**
```sql
SELECT WORD_COUNT('Hello world')                    -- Returns 2
SELECT WORD_COUNT('The quick brown fox')            -- Returns 4
SELECT WORD_COUNT('one,two;three')                  -- Returns 3
```


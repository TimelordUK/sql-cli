# Solar System Functions Usage Guide

## Available Functions

All functions take a solar body name (string) as input:

- `MASS_SOLAR_BODY(name)` - Returns mass in kg
- `RADIUS_SOLAR_BODY(name)` - Returns radius in meters  
- `GRAVITY_SOLAR_BODY(name)` - Returns surface gravity in m/s²
- `DISTANCE_SOLAR_BODY(name)` - Returns distance from Sun in meters
- `ORBITAL_PERIOD_SOLAR_BODY(name)` - Returns orbital period in days
- `DENSITY_SOLAR_BODY(name)` - Returns density in kg/m³
- `ESCAPE_VELOCITY_SOLAR_BODY(name)` - Returns escape velocity in m/s
- `ROTATION_PERIOD_SOLAR_BODY(name)` - Returns rotation period in hours
- `MOONS_SOLAR_BODY(name)` - Returns number of moons

## Supported Solar Bodies

- **Star**: Sun
- **Planets**: Mercury, Venus, Earth, Mars, Jupiter, Saturn, Uranus, Neptune
- **Dwarf Planets**: Pluto, Ceres, Eris
- **Moon**: Moon

## What Works ✅

### Functions in SELECT clause
```sql
SELECT name, GRAVITY_SOLAR_BODY(name) AS gravity
FROM test;
```

### Functions in WHERE clause
```sql
SELECT name 
FROM test
WHERE GRAVITY_SOLAR_BODY(name) > 9.807;
```

### Functions in CASE expressions
```sql
SELECT name,
  CASE 
    WHEN GRAVITY_SOLAR_BODY(name) < 2 THEN 'Low gravity'
    WHEN GRAVITY_SOLAR_BODY(name) < 10 THEN 'Earth-like'
    ELSE 'High gravity'
  END AS gravity_class
FROM test;
```

### Functions in calculations
```sql
SELECT name,
  ROUND(MASS_SOLAR_BODY(name) / MASS_SOLAR_BODY('Earth'), 2) AS earth_masses,
  ROUND(100 * GRAVITY_SOLAR_BODY(name) / 9.807, 1) AS weight_kg
FROM test;
```

### ORDER BY column aliases
```sql
SELECT name, GRAVITY_SOLAR_BODY(name) AS g
FROM test
ORDER BY g DESC;  -- Works: ordering by alias
```

## What Doesn't Work ❌

### ORDER BY function calls directly
```sql
-- This will fail:
SELECT name FROM test
ORDER BY GRAVITY_SOLAR_BODY(name);  -- Error: Column 'GRAVITY_SOLAR_BODY' not found
```

### Functions as LAG/LEAD arguments
```sql
-- This will fail:
SELECT LAG(GRAVITY_SOLAR_BODY(name), 1) OVER (ORDER BY position)
FROM test;  -- Error: LAG first argument must be a column
```

## Example Queries

### Find high-gravity worlds
```sql
SELECT name, 
       ROUND(GRAVITY_SOLAR_BODY(name), 2) AS gravity_ms2,
       ROUND(GRAVITY_SOLAR_BODY(name) / 9.807, 2) AS earth_g
FROM test
WHERE GRAVITY_SOLAR_BODY(name) > 9.807
ORDER BY gravity_ms2 DESC;
```

### Compare planet sizes
```sql
SELECT name,
       ROUND(RADIUS_SOLAR_BODY(name) / RADIUS_SOLAR_BODY('Earth'), 2) AS earth_radii
FROM test
WHERE type = 'Terrestrial'
ORDER BY earth_radii DESC;
```

### Calculate your weight on different worlds
```sql
SELECT name,
       ROUND(100 * GRAVITY_SOLAR_BODY(name) / 9.807, 1) AS your_weight_kg
FROM test
WHERE type IN ('Terrestrial', 'Dwarf Planet')
ORDER BY your_weight_kg DESC;
```

### Verify Kepler's Third Law
```sql
SELECT name,
       ROUND(DISTANCE_SOLAR_BODY(name) / AU(), 2) AS distance_au,
       ROUND(ORBITAL_PERIOD_SOLAR_BODY(name) / 365.256, 2) AS period_years,
       ROUND(POW(ORBITAL_PERIOD_SOLAR_BODY(name) / 365.256, 2) / 
             POW(DISTANCE_SOLAR_BODY(name) / AU(), 3), 3) AS keplers_constant
FROM test
WHERE type IN ('Terrestrial', 'Gas Giant')
ORDER BY distance_au;
```

## Tips

1. Always use column aliases when you need to ORDER BY a function result
2. Function names are case-insensitive: `gravity_solar_body('earth')` works too
3. Body names are also case-insensitive: 'Earth', 'EARTH', 'earth' all work
4. Use with other functions like `PI()`, `POW()`, `AU()`, `LIGHT_YEAR()` for complex calculations
5. Combine with window functions using the regular columns from your CSV

## Running the Examples

```bash
# Load data and run queries interactively
./target/release/sql-cli examples/solar_system.csv

# Run a single query
./target/release/sql-cli examples/solar_system.csv -q "SELECT name, GRAVITY_SOLAR_BODY(name) FROM test"

# Run the full example script
./target/release/sql-cli examples/solar_system.csv < examples/solar_system_calculations.sql
```
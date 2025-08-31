# Physics and Conversions SQL Extension Design

## Overview
Extend SQL CLI with elegant physics, conversions, and mathematical constants while maintaining SQL's clean syntax.

## Design Philosophy
- **Intuitive**: Natural SQL-like syntax that feels native
- **Composable**: Functions that work well together
- **Extensible**: Easy to add new units, constants, and formulas
- **Performance**: All calculations in-memory, instant results
- **Discoverable**: Clear naming conventions

## Proposed Syntax Approaches

### 1. Unit Conversion Functions (Recommended)

```sql
-- Option A: CONVERT_* family (clean, discoverable)
SELECT weight_lbs, CONVERT_LBS_KG(weight_lbs) as weight_kg FROM data;
SELECT distance_m, CONVERT_M_FT(distance_m) as distance_ft FROM data;

-- Option B: Universal CONVERT with units (flexible)
SELECT CONVERT(weight, 'lbs', 'kg') FROM data;
SELECT CONVERT(100, 'celsius', 'fahrenheit') FROM data;

-- Option C: Unit-aware operators (most elegant)
SELECT weight::lbs->kg FROM data;  -- Future enhancement
```

### 2. Physical Constants

```sql
-- Named constant functions (no parameters)
SELECT radius * 2 * PI() FROM circles;
SELECT mass * C() * C() as energy FROM physics;  -- E=mc²
SELECT G() * mass1 * mass2 / (distance * distance) as force;

-- Constants namespace (alternative)
SELECT CONST.PI, CONST.E, CONST.G FROM dual;
```

### 3. Physics Formulas

```sql
-- Geometry
SELECT SPHERE_VOL(radius) FROM spheres;
SELECT CUBE_VOL(side) FROM cubes;
SELECT CIRCLE_AREA(radius) FROM circles;

-- Physics calculations
SELECT KINETIC_ENERGY(mass, velocity) FROM motion;
SELECT MOMENTUM(mass, velocity) FROM collisions;
SELECT FORCE(mass, acceleration) FROM dynamics;
```

## Implementation Plan

### Phase 1: Core Infrastructure
```rust
// src/sql/functions/physics.rs
pub struct UnitConverter {
    conversions: HashMap<(Unit, Unit), ConversionFn>,
}

pub enum Unit {
    // Mass
    Kilogram, Gram, Pound, Ounce, Stone, Ton,
    // Length  
    Meter, Kilometer, Mile, Foot, Inch, Yard,
    // Time
    Second, Minute, Hour, Day, Week, Year,
    // Temperature
    Celsius, Fahrenheit, Kelvin,
    // Area
    SquareMeter, SquareFoot, Acre, Hectare,
    // Volume
    Liter, Gallon, CubicMeter, CubicFoot,
}
```

### Phase 2: Conversion Functions
```sql
-- Mass conversions
CONVERT_LBS_KG(value)
CONVERT_KG_LBS(value)
CONVERT_OZ_G(value)

-- Length conversions  
CONVERT_M_FT(value)
CONVERT_MI_KM(value)
CONVERT_IN_CM(value)

-- Temperature (non-linear)
CONVERT_C_F(value)  -- (C * 9/5) + 32
CONVERT_F_K(value)  -- (F + 459.67) * 5/9
```

### Phase 3: Constants Library
```sql
-- Mathematical
PI()      -- 3.14159265359
E()       -- 2.71828182846
PHI()     -- 1.61803398875 (golden ratio)

-- Physics
C()       -- 299792458 m/s (speed of light)
G()       -- 6.67430e-11 (gravitational constant)
H()       -- 6.62607015e-34 (Planck constant)
NA()      -- 6.02214076e23 (Avogadro's number)
KB()      -- 1.380649e-23 (Boltzmann constant)

-- Chemistry
R()       -- 8.314462618 (gas constant)
F()       -- 96485.33212 (Faraday constant)
```

### Phase 4: Formula Functions
```sql
-- Geometry (2D)
CIRCLE_AREA(radius)           -- π * r²
CIRCLE_CIRCUMFERENCE(radius)  -- 2 * π * r
TRIANGLE_AREA(base, height)   -- 0.5 * base * height

-- Geometry (3D)
SPHERE_VOL(radius)            -- (4/3) * π * r³
SPHERE_AREA(radius)           -- 4 * π * r²
CUBE_VOL(side)                -- side³
CYLINDER_VOL(radius, height)  -- π * r² * h

-- Physics
KINETIC_ENERGY(mass, velocity)     -- 0.5 * m * v²
POTENTIAL_ENERGY(mass, height)     -- m * g * h
MOMENTUM(mass, velocity)            -- m * v
FORCE(mass, acceleration)           -- m * a
WORK(force, distance)               -- f * d
POWER(work, time)                   -- w / t
```

## Usage Examples

### Example 1: Unit Conversion Pipeline
```sql
-- Convert recipe measurements
SELECT 
    ingredient,
    amount_oz,
    CONVERT_OZ_G(amount_oz) as amount_g,
    CONVERT_OZ_G(amount_oz) / 1000 as amount_kg
FROM recipes;
```

### Example 2: Physics Calculations
```sql
-- Calculate kinetic energy for moving objects
SELECT 
    object_name,
    mass_kg,
    velocity_ms,
    KINETIC_ENERGY(mass_kg, velocity_ms) as energy_joules,
    KINETIC_ENERGY(CONVERT_LBS_KG(mass_lbs), velocity_ms) as energy_from_lbs
FROM motion_data;
```

### Example 3: Geometry Computations
```sql
-- Calculate sphere properties
SELECT 
    sphere_id,
    radius_m,
    SPHERE_VOL(radius_m) as volume_m3,
    SPHERE_AREA(radius_m) as surface_area_m2,
    SPHERE_VOL(CONVERT_FT_M(radius_ft)) as volume_from_ft
FROM spheres;
```

### Example 4: Scientific Constants
```sql
-- Einstein's mass-energy equivalence
SELECT 
    particle,
    mass_kg,
    mass_kg * C() * C() as rest_energy_joules
FROM particles;

-- Ideal gas law: PV = nRT
SELECT 
    pressure * volume / (moles * R() * temperature) as gas_law_check
FROM gas_samples;
```

## Implementation Strategy

### 1. Parser Extension (recursive_parser.rs)
```rust
// Add to function parsing
Function::Convert { value, from_unit, to_unit }
Function::PhysicsConstant { name }
Function::PhysicsFormula { formula_type, args }
```

### 2. Function Registry
```rust
// src/sql/functions/mod.rs
pub fn register_physics_functions() {
    // Conversions
    register_fn("CONVERT_LBS_KG", convert_lbs_kg);
    register_fn("CONVERT_M_FT", convert_m_ft);
    
    // Constants
    register_const("PI", std::f64::consts::PI);
    register_const("E", std::f64::consts::E);
    register_const("C", 299792458.0);
    
    // Formulas
    register_fn("SPHERE_VOL", sphere_volume);
    register_fn("KINETIC_ENERGY", kinetic_energy);
}
```

### 3. Metadata & Discovery
```sql
-- Show available conversions
SELECT * FROM SYSTEM.CONVERSIONS;

-- Show physics constants
SELECT * FROM SYSTEM.CONSTANTS;

-- Show formula functions
SELECT * FROM SYSTEM.FORMULAS;
```

## Benefits of This Approach

1. **Clean Syntax**: Functions feel natural in SQL
2. **Composable**: Can chain conversions and calculations
3. **Discoverable**: CONVERT_* prefix makes finding conversions easy
4. **Extensible**: Easy to add new units, constants, formulas
5. **Type-Safe**: Rust implementation ensures correctness
6. **Fast**: All calculations in-memory, no external calls

## Future Enhancements

1. **Unit-aware columns**: 
   ```sql
   ALTER TABLE ADD COLUMN weight DECIMAL UNIT 'kg';
   ```

2. **Automatic unit conversion**:
   ```sql
   SELECT weight FROM table USING UNIT 'lbs';
   ```

3. **Complex number support**:
   ```sql
   SELECT COMPLEX_MAG(real, imag) FROM signals;
   ```

4. **Vector operations**:
   ```sql
   SELECT DOT_PRODUCT(v1, v2), CROSS_PRODUCT(v1, v2) FROM vectors;
   ```

5. **Statistical physics**:
   ```sql
   SELECT BOLTZMANN_DIST(energy, temperature) FROM states;
   ```

## Testing Strategy

```bash
# Test conversion accuracy
./target/release/sql-cli data/test_physics.csv -q "SELECT CONVERT_LBS_KG(100)" -o csv
# Expected: 45.35924

# Test constants
./target/release/sql-cli data/test_physics.csv -q "SELECT PI()" -o csv  
# Expected: 3.14159265359

# Test formulas
./target/release/sql-cli data/test_physics.csv -q "SELECT SPHERE_VOL(1)" -o csv
# Expected: 4.18879020479 (4π/3)
```

## Recommended Starting Point

Begin with Phase 1 & 2:
1. Implement basic CONVERT_X_Y functions for common units
2. Add PI(), E(), C() constants
3. Test with real data
4. Gather feedback on syntax preferences
5. Expand based on usage patterns

This approach maintains SQL elegance while providing powerful physics capabilities.
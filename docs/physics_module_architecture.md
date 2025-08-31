# Physics Module Architecture & Roadmap

## Tomorrow's Priority Tasks

### 1. Scientific Notation Support
```sql
-- Enable natural physics queries
SELECT 1e-10 * 1e10 as one FROM test;
SELECT 6.022e23 as avogadro FROM test;
SELECT 1.602e-19 as elementary_charge FROM test;
```

**Implementation**:
- Update lexer to recognize pattern: `[0-9]+\.?[0-9]*[eE][+-]?[0-9]+`
- Parse as `f64` using Rust's built-in parsing
- Add tests for edge cases (1e10, 1.5e-3, 2E+5)

### 2. DUAL Table Support
```sql
-- Oracle-style single-row virtual table for expressions
SELECT PI() FROM DUAL;
SELECT 2 + 2 FROM DUAL;
SELECT C() * 1e-9 as speed_of_light_km_per_ms FROM DUAL;
```

**Implementation**:
- Virtual table with single row, no columns
- Always available, no file needed
- Perfect for testing constants and calculations

## Module Architecture

### Core Design Principles
1. **Pure Functions**: All physics functions are pure (same input → same output)
2. **Modular Organization**: Separate modules by domain
3. **Memoization Ready**: Design for caching expensive calculations
4. **Type Safety**: Strong typing for units and dimensions

### Proposed Module Structure

```rust
src/
├── sql/
│   └── functions/
│       ├── mod.rs                 // Function registry
│       ├── constants/
│       │   ├── mod.rs             // Export all constants
│       │   ├── mathematical.rs    // PI, E, TAU, PHI
│       │   ├── physics.rs        // C, G, H, HBAR
│       │   ├── chemistry.rs      // NA, KB, R
│       │   └── particles.rs      // ME, MP, MN, MU
│       ├── conversions/
│       │   ├── mod.rs
│       │   ├── length.rs         // meters ↔ feet, miles ↔ km
│       │   ├── mass.rs           // kg ↔ lbs, grams ↔ oz
│       │   ├── temperature.rs    // C ↔ F ↔ K
│       │   ├── energy.rs         // J ↔ eV, cal ↔ J
│       │   └── pressure.rs       // Pa ↔ PSI, atm ↔ Pa
│       └── formulas/
│           ├── mod.rs
│           ├── geometry.rs       // SPHERE_VOL, CIRCLE_AREA
│           ├── mechanics.rs      // KINETIC_E, MOMENTUM
│           ├── relativity.rs     // LORENTZ_FACTOR, TIME_DILATION
│           ├── quantum.rs        // DE_BROGLIE, COMPTON
│           └── thermodynamics.rs // IDEAL_GAS, ENTROPY
```

### Implementation Example

```rust
// src/sql/functions/constants/physics.rs
use cached::proc_macro::cached;

/// Speed of light in vacuum (m/s)
#[inline(always)]
pub const fn speed_of_light() -> f64 {
    299_792_458.0
}

/// Gravitational constant (m³ kg⁻¹ s⁻²)
#[inline(always)]
pub const fn gravitational_constant() -> f64 {
    6.674_30e-11
}

// src/sql/functions/formulas/mechanics.rs
use cached::proc_macro::cached;

/// Calculate kinetic energy (J)
#[cached]
pub fn kinetic_energy(mass_kg: f64, velocity_ms: f64) -> f64 {
    0.5 * mass_kg * velocity_ms * velocity_ms
}

/// Calculate relativistic kinetic energy (J)
#[cached]
pub fn relativistic_kinetic_energy(mass_kg: f64, velocity_ms: f64) -> f64 {
    let c = speed_of_light();
    let gamma = lorentz_factor(velocity_ms);
    mass_kg * c * c * (gamma - 1.0)
}
```

## Memoization Strategy

### When to Memoize
1. **Complex calculations**: Lorentz factor, integration results
2. **Frequently called**: Common conversions (C→F)
3. **Recursive formulas**: Factorial, Fibonacci sequences

### Rust Memoization Options

#### Option 1: `cached` crate (Recommended)
```rust
use cached::proc_macro::cached;

#[cached]
fn expensive_calculation(x: f64, y: f64) -> f64 {
    // Complex physics calculation
    x.powf(y) * (x / y).sin()
}
```

#### Option 2: `memoize` crate
```rust
use memoize::memoize;

#[memoize]
fn schwarzschild_radius(mass_kg: f64) -> f64 {
    2.0 * G() * mass_kg / (C() * C())
}
```

#### Option 3: Manual with HashMap
```rust
use std::collections::HashMap;
use std::sync::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    static ref CACHE: Mutex<HashMap<String, f64>> = Mutex::new(HashMap::new());
}
```

## Function Registry Pattern

```rust
// src/sql/functions/mod.rs
pub struct FunctionRegistry {
    constants: HashMap<String, fn() -> f64>,
    unary_functions: HashMap<String, fn(f64) -> f64>,
    binary_functions: HashMap<String, fn(f64, f64) -> f64>,
}

impl FunctionRegistry {
    pub fn new() -> Self {
        let mut registry = Self::default();
        
        // Register constants
        registry.register_constant("PI", std::f64::consts::PI);
        registry.register_constant("C", constants::physics::speed_of_light);
        
        // Register functions
        registry.register_unary("SQRT", f64::sqrt);
        registry.register_binary("KINETIC_E", formulas::mechanics::kinetic_energy);
        
        registry
    }
}
```

## Phase 1: Core Physics Functions (Week 1)

### Geometry
- `SPHERE_VOL(r)` - Volume of sphere
- `SPHERE_AREA(r)` - Surface area of sphere
- `CIRCLE_AREA(r)` - Area of circle
- `CYLINDER_VOL(r, h)` - Volume of cylinder

### Basic Mechanics
- `KINETIC_E(m, v)` - Kinetic energy
- `MOMENTUM(m, v)` - Linear momentum
- `FORCE(m, a)` - Newton's second law
- `CENTRIPETAL_F(m, v, r)` - Centripetal force

### Conversions
- `CONVERT_M_FT(m)` - Meters to feet
- `CONVERT_KG_LBS(kg)` - Kilograms to pounds
- `CONVERT_C_F(c)` - Celsius to Fahrenheit
- `CONVERT_J_EV(j)` - Joules to electron volts

## Phase 2: Advanced Physics (Week 2)

### Relativity
- `LORENTZ_FACTOR(v)` - Lorentz gamma factor
- `TIME_DILATION(t, v)` - Time dilation
- `LENGTH_CONTRACTION(l, v)` - Length contraction
- `RELATIVISTIC_E(m, v)` - Relativistic energy

### Quantum Mechanics
- `DE_BROGLIE(m, v)` - De Broglie wavelength
- `PHOTON_E(f)` - Photon energy from frequency
- `COMPTON_WAVELENGTH(m)` - Compton wavelength
- `UNCERTAINTY(dx, dp)` - Heisenberg uncertainty

### Thermodynamics
- `IDEAL_GAS(n, T, V)` - Ideal gas pressure
- `ENTROPY(Q, T)` - Entropy change
- `CARNOT_EFFICIENCY(T_hot, T_cold)` - Carnot efficiency

## Phase 3: Universal CONVERT Function

```sql
-- Universal conversion syntax
SELECT CONVERT(100, 'kg', 'lbs') as weight_lbs;
SELECT CONVERT(0, 'celsius', 'kelvin') as absolute_zero;
SELECT CONVERT(1, 'atm', 'pascal') as pressure_pa;
```

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_kinetic_energy() {
        let ke = kinetic_energy(10.0, 5.0);
        assert!((ke - 125.0).abs() < 1e-10);
    }
    
    #[test]
    fn test_memoization() {
        // First call - calculates
        let t1 = std::time::Instant::now();
        let _ = expensive_calculation(3.14, 2.71);
        let d1 = t1.elapsed();
        
        // Second call - cached
        let t2 = std::time::Instant::now();
        let _ = expensive_calculation(3.14, 2.71);
        let d2 = t2.elapsed();
        
        assert!(d2 < d1 / 10); // Cached call should be 10x faster
    }
}
```

### SQL Integration Tests
```sql
-- Test constants
SELECT PI(), E(), C() FROM DUAL;

-- Test conversions
SELECT CONVERT_M_FT(1.0) FROM DUAL; -- Should return 3.28084

-- Test formulas
SELECT KINETIC_E(10, 20) FROM DUAL; -- Should return 2000

-- Test with real data
SELECT 
    mass_kg,
    velocity_ms,
    KINETIC_E(mass_kg, velocity_ms) as ke_classical,
    RELATIVISTIC_E(mass_kg, velocity_ms) as ke_relativistic
FROM particles
WHERE velocity_ms > 0.1 * C();
```

## Performance Considerations

1. **Inline Constants**: Use `#[inline(always)]` for constants
2. **Cache Expensive Ops**: Memoize transcendental functions
3. **SIMD Potential**: Future optimization for vector operations
4. **Lazy Evaluation**: Consider lazy evaluation for complex chains

## Future Enhancements

### Complex Numbers
```sql
SELECT COMPLEX_MAG(3, 4) as magnitude; -- Returns 5
SELECT COMPLEX_PHASE(1, 1) as phase;   -- Returns π/4
```

### Vector Operations
```sql
SELECT DOT_PRODUCT([1,2,3], [4,5,6]) as dot;
SELECT CROSS_PRODUCT([1,0,0], [0,1,0]) as cross; -- Returns [0,0,1]
```

### Matrix Operations
```sql
SELECT MATRIX_DET([[1,2],[3,4]]) as determinant;
SELECT MATRIX_MULT(A, B) as product FROM matrices;
```

### Unit-Aware Calculations
```sql
-- Automatic unit tracking
SELECT 100::kg * 5::m/s^2 as force::N;
SELECT 50::J / 10::s as power::W;
```

## Benefits of This Architecture

1. **Modularity**: Easy to add new functions without touching core
2. **Performance**: Memoization for expensive calculations
3. **Testability**: Pure functions are trivial to test
4. **Type Safety**: Rust's type system prevents unit errors
5. **Extensibility**: Plugin architecture for custom domains
6. **Documentation**: Self-documenting through module structure

## Implementation Priority

1. **Tomorrow**: Scientific notation + DUAL table
2. **Day 2**: Extract constants to modules
3. **Day 3**: Basic conversions (length, mass, temp)
4. **Day 4**: Core geometry & mechanics formulas
5. **Day 5**: Memoization framework
6. **Week 2**: Advanced physics functions
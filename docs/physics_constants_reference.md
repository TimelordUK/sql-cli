# Physics and Mathematical Constants Reference

## Overview
SQL CLI provides built-in physics and mathematical constants as zero-argument functions. These constants use precise values from CODATA 2018 and are suitable for scientific calculations.

## Mathematical Constants

### PI()
- **Value**: 3.141592653589793 
- **Description**: Circle constant (π), ratio of circumference to diameter
- **Example**: `SELECT radius * 2 * PI() as circumference FROM circles`

### E()
- **Value**: 2.718281828459045
- **Description**: Euler's number, base of natural logarithm
- **Example**: `SELECT POWER(E(), x) as exponential FROM data` (equivalent to EXP(x))

### TAU()
- **Value**: 6.283185307179586 (2π)
- **Description**: Full circle constant, τ = 2π
- **Example**: `SELECT angle / TAU() as rotations FROM angles`

### PHI()
- **Value**: 1.618033988749895
- **Description**: Golden ratio, φ = (1 + √5) / 2
- **Example**: `SELECT width * PHI() as golden_height FROM rectangles`

## Fundamental Physics Constants

### C()
- **Value**: 299,792,458 m/s
- **Description**: Speed of light in vacuum (exact value by definition)
- **Unit**: meters per second
- **Example**: `SELECT mass * C() * C() as rest_energy FROM particles` (E=mc²)

### G()
- **Value**: 6.67430 × 10⁻¹¹ m³ kg⁻¹ s⁻²
- **Description**: Newtonian gravitational constant
- **Unit**: cubic meters per kilogram per second squared
- **Example**: `SELECT G() * mass1 * mass2 / (distance * distance) as force FROM gravity`

### H()
- **Value**: 6.62607015 × 10⁻³⁴ J⋅s
- **Description**: Planck constant (exact value by definition)
- **Unit**: joule-seconds
- **Example**: `SELECT H() * frequency as photon_energy FROM spectrum`

## Chemistry Constants

### NA()
- **Value**: 6.02214076 × 10²³ mol⁻¹
- **Description**: Avogadro constant (exact value by definition)
- **Unit**: per mole
- **Example**: `SELECT moles * NA() as number_of_molecules FROM chemistry`

### KB()
- **Value**: 1.380649 × 10⁻²³ J⋅K⁻¹
- **Description**: Boltzmann constant (exact value by definition)
- **Unit**: joules per kelvin
- **Example**: `SELECT 3/2 * KB() * temperature as kinetic_energy FROM gases`

## Particle Physics Constants

### ME()
- **Value**: 9.1093837015 × 10⁻³¹ kg
- **Description**: Electron rest mass
- **Unit**: kilograms
- **Example**: `SELECT ME() * C() * C() as electron_rest_energy FROM physics`

### MP()
- **Value**: 1.67262192369 × 10⁻²⁷ kg
- **Description**: Proton rest mass
- **Unit**: kilograms
- **Example**: `SELECT MP() / ME() as proton_electron_mass_ratio FROM physics`

### MN()
- **Value**: 1.67492749804 × 10⁻²⁷ kg
- **Description**: Neutron rest mass
- **Unit**: kilograms
- **Example**: `SELECT (MN() - MP()) * C() * C() as neutron_proton_mass_difference FROM physics`

## Usage Examples

### Example 1: Calculate Schwarzschild Radius
```sql
-- Schwarzschild radius of a black hole
SELECT 
    mass_kg,
    2 * G() * mass_kg / (C() * C()) as schwarzschild_radius_m
FROM black_holes;
```

### Example 2: De Broglie Wavelength
```sql
-- De Broglie wavelength of an electron
SELECT 
    velocity_ms,
    H() / (ME() * velocity_ms) as de_broglie_wavelength_m
FROM electron_beams;
```

### Example 3: Ideal Gas Calculations
```sql
-- Ideal gas law: PV = nRT where R = NA * KB
SELECT 
    pressure_pa,
    volume_m3,
    moles,
    temperature_k,
    pressure_pa * volume_m3 / (moles * NA() * KB() * temperature_k) as pv_nrt_ratio
FROM gas_samples;
```

### Example 4: Photon Energy
```sql
-- Energy of a photon: E = hf = hc/λ
SELECT 
    wavelength_m,
    H() * C() / wavelength_m as photon_energy_j,
    H() * C() / wavelength_m / 1.602176634e-19 as photon_energy_ev
FROM spectrum;
```

### Example 5: Gravitational Force
```sql
-- Newton's law of universal gravitation
SELECT 
    body1_mass,
    body2_mass,
    distance_m,
    G() * body1_mass * body2_mass / (distance_m * distance_m) as force_n
FROM orbital_mechanics;
```

### Example 6: Circle Geometry
```sql
-- Various circle calculations
SELECT 
    radius,
    PI() * radius * radius as area,
    2 * PI() * radius as circumference,
    TAU() * radius as circumference_tau,
    4/3 * PI() * POWER(radius, 3) as sphere_volume
FROM circles;
```

## Precision Notes

- Mathematical constants use maximum precision available in 64-bit floating point
- Physics constants use CODATA 2018 recommended values
- Some constants (C, H, NA, KB) are exact by definition as of 2019 SI redefinition
- G has relatively large uncertainty (about 22 ppm) compared to other constants
- Particle masses have uncertainties in the last few digits

## Related Functions

These constants work seamlessly with other SQL CLI functions:
- Math functions: `POWER()`, `SQRT()`, `EXP()`, `LN()`, `LOG()`
- String functions for formatting: `ROUND()`, `FLOOR()`, `CEILING()`
- Can be used in any expression: `WHERE`, `ORDER BY`, `GROUP BY`, etc.

## Quick Reference Table

| Constant | Type | Value | SI Unit |
|----------|------|-------|---------|
| PI() | Math | 3.14159... | - |
| E() | Math | 2.71828... | - |
| TAU() | Math | 6.28318... | - |
| PHI() | Math | 1.61803... | - |
| C() | Physics | 299,792,458 | m/s |
| G() | Physics | 6.67430e-11 | m³/(kg⋅s²) |
| H() | Physics | 6.62607e-34 | J⋅s |
| NA() | Chemistry | 6.02214e23 | /mol |
| KB() | Chemistry | 1.38065e-23 | J/K |
| ME() | Particle | 9.10938e-31 | kg |
| MP() | Particle | 1.67262e-27 | kg |
| MN() | Particle | 1.67493e-27 | kg |

## Future Enhancements

Planned additions include:
- Unit conversion functions: `CONVERT_M_FT()`, `CONVERT_KG_LBS()`
- More constants: R (gas constant), F (Faraday), μ₀ (vacuum permeability)
- Complex calculations: `LORENTZ_FACTOR()`, `SCHWARZSCHILD_RADIUS()`
- Vector operations for physics simulations
# Changelog

All notable changes to SQL CLI will be documented in this file.

## [1.35.0] - 2025-01-09

### Added
- **Scientific Calculator Mode**: Transform SQL CLI into a powerful scientific calculator
  - DUAL table support (Oracle-compatible) for expression evaluation
  - Scientific notation support (1e-10, 3.14e5, etc.)
  - 27 new mathematical and physics constants:
    - Math constants: PI(), EULER(), TAU(), PHI(), SQRT2(), LN2(), LN10()
    - Physics fundamental: C() (speed of light), G() (gravitational), PLANCK(), HBAR(), BOLTZMANN(), AVOGADRO(), R() (gas constant)
    - Electromagnetic: E0() (permittivity), MU0() (permeability), QE() (elementary charge)
    - Particle masses: ME() (electron), MP() (proton), MN() (neutron), AMU()
    - Other: ALPHA() (fine structure), RYDBERG(), SIGMA() (Stefan-Boltzmann)
  - Angle conversion functions: DEGREES(), RADIANS()
  - Queries without FROM clause now implicitly use DUAL

- **String Functions**: MID(), UPPER(), LOWER(), TRIM() for text manipulation

### Changed
- **Parser Refactoring**: Removed hardcoded function list; parser now recognizes any identifier() as potential function
- Function validation moved to evaluator (better separation of concerns)
- Adding new functions no longer requires parser changes

### Fixed
- Critical bug: Division by zero false positives with extremely small numbers (< 2.22e-16)
- Now correctly handles physics constants like electron mass (9.1e-31 kg)

### Documentation
- Added comprehensive DUAL table examples to README
- Added F1 help instruction for TUI mode
- Documented all unsupported SQL features clearly
- Added scientific computing examples with physics calculations

## [1.34.0] - Previous Release

### Added
- SQL-driven charting with real-world VWAP examples
- Chart engine improvements
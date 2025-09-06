-- Chemical formula parsing and molecular calculations
-- sql-cli can parse chemical formulas and return molecular information

-- Get atomic/molecular mass from chemical formulas
SELECT 
    ATOMIC_MASS('H2O') as water_mass,
    ATOMIC_MASS('CO2') as carbon_dioxide_mass,
    ATOMIC_MASS('C6H12O6') as glucose_mass,
    ATOMIC_MASS('C8H10N4O2') as caffeine_mass,
    ATOMIC_MASS('NaCl') as salt_mass;
GO

-- Get chemical formula from common names
SELECT 
    MOLECULE_FORMULA('water') as water_formula,
    MOLECULE_FORMULA('glucose') as glucose_formula,
    MOLECULE_FORMULA('ethanol') as ethanol_formula,
    MOLECULE_FORMULA('caffeine') as caffeine_formula,
    MOLECULE_FORMULA('aspirin') as aspirin_formula;
GO

-- Combined usage - get formula then calculate mass
SELECT 
    MOLECULE_FORMULA('methane') as formula,
    ATOMIC_MASS(MOLECULE_FORMULA('methane')) as molecular_mass;
GO

-- Practical example with physics constants
SELECT 
    'Water' as compound,
    MOLECULE_FORMULA('water') as formula,
    ATOMIC_MASS('H2O') as molecular_mass_amu,
    AVOGADRO() as avogadro_constant,
    ATOMIC_MASS('H2O') / AVOGADRO() * 1e23 as grams_per_mole;
GO

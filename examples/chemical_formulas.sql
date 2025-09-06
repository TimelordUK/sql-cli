-- Chemical formula parsing and molecular calculations
-- sql-cli can parse chemical formulas and return molecular information

-- Get molecular mass from chemical formulas
SELECT 
    GET_MOLECULAR_MASS('H2O') as water_mass,
    GET_MOLECULAR_MASS('CO2') as carbon_dioxide_mass,
    GET_MOLECULAR_MASS('C6H12O6') as glucose_mass,
    GET_MOLECULAR_MASS('C8H10N4O2') as caffeine_mass,
    GET_MOLECULAR_MASS('NaCl') as salt_mass;
GO

-- Get chemical formula from common names
SELECT 
    GET_CHEMICAL_FORMULA('water') as water_formula,
    GET_CHEMICAL_FORMULA('glucose') as glucose_formula,
    GET_CHEMICAL_FORMULA('ethanol') as ethanol_formula,
    GET_CHEMICAL_FORMULA('caffeine') as caffeine_formula,
    GET_CHEMICAL_FORMULA('aspirin') as aspirin_formula;
GO
-- Combined usage - get formula then calculate mass
SELECT 
    GET_CHEMICAL_FORMULA('methane') as formula,
    GET_MOLECULAR_MASS(GET_CHEMICAL_FORMULA('methane')) as molecular_mass;

-- Practical example: Analyzing chemical compounds in a dataset
SELECT 
    compound_name,
    GET_CHEMICAL_FORMULA(compound_name) as formula,
    GET_MOLECULAR_MASS(GET_CHEMICAL_FORMULA(compound_name)) as molecular_weight,
    CASE 
        WHEN GET_MOLECULAR_MASS(GET_CHEMICAL_FORMULA(compound_name)) < 100 THEN 'Light'
        WHEN GET_MOLECULAR_MASS(GET_CHEMICAL_FORMULA(compound_name)) < 500 THEN 'Medium'
        ELSE 'Heavy'
    END as weight_category
FROM compounds_table
WHERE compound_name IN ('water', 'ethanol', 'glucose', 'benzene', 'acetone');
GO

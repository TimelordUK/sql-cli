-- Particle Electric Charges Example
-- Demonstrates the signed electric charge functions for fundamental particles
-- Run: ./target/release/sql-cli < examples/particle_charges.sql

-- Basic particle charges
SELECT 
    'Electron' AS particle,
    CHARGE_ELECTRON() AS charge_coulombs,
    CHARGE_ELECTRON() / QE() AS charge_in_e
UNION ALL
SELECT 
    'Proton',
    CHARGE_PROTON(),
    CHARGE_PROTON() / QE()
UNION ALL
SELECT 
    'Neutron',
    CHARGE_NEUTRON(),
    CHARGE_NEUTRON() / QE()
UNION ALL
SELECT 
    'Positron',
    CHARGE_POSITRON(),
    CHARGE_POSITRON() / QE()
UNION ALL
SELECT 
    'Muon',
    CHARGE_MUON(),
    CHARGE_MUON() / QE()
UNION ALL
SELECT 
    'Tau',
    CHARGE_TAU(),
    CHARGE_TAU() / QE();
GO

-- Quark charges
SELECT 
    'Up Quark' AS quark,
    CHARGE_UP_QUARK() AS charge_coulombs,
    ROUND(CHARGE_UP_QUARK() / QE(), 3) AS charge_in_e,
    '2/3' AS fractional_charge
UNION ALL
SELECT 
    'Down Quark',
    CHARGE_DOWN_QUARK(),
    ROUND(CHARGE_DOWN_QUARK() / QE(), 3),
    '-1/3';
GO

-- Verify charge conservation in atoms
SELECT
    'Hydrogen (H)' AS atom,
    1 AS protons,
    1 AS electrons,
    0 AS neutrons,
    1 * CHARGE_PROTON() + 1 * CHARGE_ELECTRON() AS net_charge
UNION ALL
SELECT
    'Helium (He)',
    2,
    2,
    2,
    2 * CHARGE_PROTON() + 2 * CHARGE_ELECTRON()
UNION ALL
SELECT
    'Carbon (C)',
    6,
    6,
    6,
    6 * CHARGE_PROTON() + 6 * CHARGE_ELECTRON()
UNION ALL
SELECT
    'Oxygen (O)',
    8,
    8,
    8,
    8 * CHARGE_PROTON() + 8 * CHARGE_ELECTRON();
GO

-- Verify quark composition of nucleons
SELECT
    'Proton (uud)' AS nucleon,
    '2 up + 1 down' AS quark_composition,
    2 * CHARGE_UP_QUARK() + CHARGE_DOWN_QUARK() AS calculated_charge,
    CHARGE_PROTON() AS expected_charge,
    CASE 
        WHEN ABS(2 * CHARGE_UP_QUARK() + CHARGE_DOWN_QUARK() - CHARGE_PROTON()) < 1e-30 
        THEN 'VERIFIED' 
        ELSE 'ERROR' 
    END AS verification
UNION ALL
SELECT
    'Neutron (udd)',
    '1 up + 2 down',
    CHARGE_UP_QUARK() + 2 * CHARGE_DOWN_QUARK(),
    CHARGE_NEUTRON(),
    CASE 
        WHEN ABS(CHARGE_UP_QUARK() + 2 * CHARGE_DOWN_QUARK() - CHARGE_NEUTRON()) < 1e-30 
        THEN 'VERIFIED' 
        ELSE 'ERROR' 
    END;
GO

-- Ion charges
SELECT
    'Na+' AS ion,
    11 AS protons,
    10 AS electrons,
    11 * CHARGE_PROTON() + 10 * CHARGE_ELECTRON() AS net_charge_coulombs,
    ROUND((11 * CHARGE_PROTON() + 10 * CHARGE_ELECTRON()) / QE(), 0) AS charge_in_e
UNION ALL
SELECT
    'Cl-',
    17,
    18,
    17 * CHARGE_PROTON() + 18 * CHARGE_ELECTRON(),
    ROUND((17 * CHARGE_PROTON() + 18 * CHARGE_ELECTRON()) / QE(), 0)
UNION ALL
SELECT
    'Ca2+',
    20,
    18,
    20 * CHARGE_PROTON() + 18 * CHARGE_ELECTRON(),
    ROUND((20 * CHARGE_PROTON() + 18 * CHARGE_ELECTRON()) / QE(), 0)
UNION ALL
SELECT
    'O2-',
    8,
    10,
    8 * CHARGE_PROTON() + 10 * CHARGE_ELECTRON(),
    ROUND((8 * CHARGE_PROTON() + 10 * CHARGE_ELECTRON()) / QE(), 0);
GO

-- Antiparticle annihilation
SELECT
    'Electron + Positron' AS reaction,
    CHARGE_ELECTRON() + CHARGE_POSITRON() AS total_charge,
    CASE 
        WHEN ABS(CHARGE_ELECTRON() + CHARGE_POSITRON()) < 1e-30 
        THEN 'Charge conserved in annihilation' 
        ELSE 'ERROR' 
    END AS conservation_check;
GO

-- Compare lepton charges
SELECT
    'Electron' AS lepton,
    ME() AS mass_kg,
    CHARGE_ELECTRON() AS charge_coulombs,
    ME() / QE() AS mass_charge_ratio
UNION ALL
SELECT
    'Muon',
    1.883531627e-28,  -- Muon mass
    CHARGE_MUON(),
    1.883531627e-28 / ABS(CHARGE_MUON())
UNION ALL
SELECT
    'Tau',
    3.16754e-27,  -- Tau mass
    CHARGE_TAU(),
    3.16754e-27 / ABS(CHARGE_TAU());
GO

-- Electric force between particles (Coulomb's law)
-- F = k * q1 * q2 / r^2
SELECT
    'Electron-Proton at 1 Bohr radius' AS interaction,
    COULOMB() * CHARGE_ELECTRON() * CHARGE_PROTON() / POW(BOHR(), 2) AS force_newtons,
    'Attractive' AS force_type
UNION ALL
SELECT
    'Electron-Electron at 1 nm',
    COULOMB() * CHARGE_ELECTRON() * CHARGE_ELECTRON() / POW(1e-9, 2),
    'Repulsive'
UNION ALL
SELECT
    'Proton-Proton at 1 fm',
    COULOMB() * CHARGE_PROTON() * CHARGE_PROTON() / POW(1e-15, 2),
    'Repulsive';
GO
-- ============================================================================
-- Electric Vehicle vs Gas Vehicle Range Comparison
-- Demonstrates fuel efficiency conversions and range calculations
-- ============================================================================
-- Run: ./target/release/sql-cli -f examples/ev_vs_gas_range_comparison.sql -o table
-- ============================================================================

-- Electric Vehicle Example: Tesla Model 3
SELECT 
    '=== TESLA MODEL 3 ===' as vehicle,
    75 as battery_kwh,
    6.5 as efficiency_km_per_kwh,
    75 * 6.5 as range_km,
    CONVERT(75 * 6.5, 'km', 'miles') as range_miles,
    CONVERT(6.5, 'kmkwh', 'mpge') as mpge_equivalent;
GO

-- Gas Vehicle Example: Toyota Camry
SELECT 
    '=== TOYOTA CAMRY ===' as vehicle,
    60 as fuel_tank_liters,
    12.5 as efficiency_km_per_liter,
    60 * 12.5 as range_km,
    CONVERT(60 * 12.5, 'km', 'miles') as range_miles,
    CONVERT(12.5, 'kml', 'mpg') as mpg_rating;
GO

-- Compare Different EV Efficiencies
SELECT 
    '=== EV EFFICIENCY COMPARISON ===' as category,
    CONVERT(6.5, 'kmkwh', 'miles/kwh') as tesla_mi_per_kwh,
    CONVERT(5.8, 'kmkwh', 'miles/kwh') as hyundai_mi_per_kwh,
    CONVERT(7.2, 'kmkwh', 'miles/kwh') as lucid_mi_per_kwh,
    CONVERT(4.5, 'kmkwh', 'mpge') as hummer_ev_mpge;
GO

-- Calculate Range for Different Battery Sizes
SELECT 
    '=== RANGE BY BATTERY SIZE ===' as category,
    50 as battery_kwh_small,
    50 * 6.0 as range_km_small,
    75 as battery_kwh_medium,
    75 * 6.0 as range_km_medium,
    100 as battery_kwh_large,
    100 * 6.0 as range_km_large,
    150 as battery_kwh_xlarge,
    150 * 6.0 as range_km_xlarge;
GO

-- Cost Comparison: Electricity vs Gasoline
SELECT 
    '=== COST PER 100KM ===' as comparison,
    -- EV: 16 kWh per 100km @ $0.12/kWh
    16 * 0.12 as ev_cost_usd,
    -- Gas: 8L per 100km @ $1.50/L
    8 * 1.50 as gas_cost_usd,
    -- Savings
    (8 * 1.50) - (16 * 0.12) as savings_per_100km,
    ((8 * 1.50) - (16 * 0.12)) * 200 as annual_savings_20k_km;
GO

-- Ship Fuel Efficiency Example  
SELECT 
    '=== CONTAINER SHIP ===' as vessel,
    3000 as bunker_fuel_tons,
    0.25 as nmi_per_ton,
    3000 * 0.25 as range_nmi,
    CONVERT(3000 * 0.25, 'nmi', 'km') as range_km;
GO

-- Compare Different Fuel Types
SELECT 
    '=== FUEL TYPE COMPARISON ===' as category,
    CONVERT(30, 'mpg', 'kml') as gasoline_30mpg,
    CONVERT(40, 'mpg', 'kml') as hybrid_40mpg,
    CONVERT(50, 'mpg', 'kml') as diesel_50mpg,
    CONVERT(120, 'mpge', 'kmkwh') as electric_120mpge;
GO

-- Real-world Range Calculation with Reserve
SELECT 
    '=== PRACTICAL RANGE WITH 20% RESERVE ===' as scenario,
    -- Tesla with 75 kWh battery
    75 * 0.8 as usable_kwh_tesla,
    75 * 0.8 * 6.5 as practical_range_km_tesla,
    -- Gas car with 60L tank
    60 * 0.8 as usable_liters_gas,
    60 * 0.8 * 12.5 as practical_range_km_gas;
GO

-- Energy Density Comparison
SELECT 
    '=== ENERGY DENSITY ===' as comparison,
    1 as gasoline_liter,
    9.5 as gasoline_kwh_per_liter,
    1 as battery_kg,
    0.25 as battery_kwh_per_kg,
    9.5 / 0.25 as energy_density_ratio;
GO

-- Calculate Charging Time vs Refueling
SELECT 
    '=== CHARGING/REFUELING TIME ===' as comparison,
    75 as battery_kwh,
    75 / 150 * 60 as supercharge_minutes_150kw,
    75 / 7 as home_charge_hours_7kw,
    60 as fuel_tank_liters,
    3 as gas_refuel_minutes;
GO
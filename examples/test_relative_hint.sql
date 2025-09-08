-- Test script demonstrating relative data file hint
-- #!data: ../data/solar_system.csv
-- This uses a relative path to find solar_system.csv in the data directory

-- Query 1: Show all celestial bodies
SELECT position, name, type, mean_distance_au, atmosphere 
FROM solar_system 
ORDER BY position;
GO

-- Query 2: Planets with atmosphere
SELECT 
    name,
    type,
    mean_distance_au,
    atmosphere
FROM solar_system
WHERE atmosphere != 'None' AND type != 'Star'
ORDER BY mean_distance_au;
GO
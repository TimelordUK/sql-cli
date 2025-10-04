-- Test Data Model Rendering
-- #! data/solar_system.csv

-- Test with data model enabled (set use_data_model = true in config)
SELECT name, type, mean_distance_au
FROM solar_system
LIMIT 20;

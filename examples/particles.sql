-- #! ../data/physics_particles.csv
SELECT *
FROM physics_particles;
GO


SELECT *
FROM physics_particles
WHERE name LIKE '%anti%';
GO

SELECT *
FROM physics_particles
WHERE charge > 0 AND charge < 1;
GO




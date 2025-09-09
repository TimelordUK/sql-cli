-- #! ../data/periodic_table.csv

SELECT * 
FROM 
  periodic_table;
GO

SELECT 
  Element, 
  Symbol, 
  Group, 
  Type  
FROM 
  periodic_table
WHERE 
  Type.Contains('Noble');
GO

SELECT 
  Type, 
  Count(*) as count
FROM 
  periodic_table
WHERE Type.Length() > 0
GROUP BY Type
ORDER BY 
  count desc;
GO


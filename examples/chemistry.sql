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
  Type.Contains('Noble')
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

with groups 
as
(
SELECT 
  Type, 
  Count(*) as member_count
FROM 
  periodic_table
GROUP BY
  Type
) 
SELECT 
Type, 
member_count
FROM 
  groups
ORDER BY 
  member_count desc
GO



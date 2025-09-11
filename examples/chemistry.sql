-- #! ../data/periodic_table.csv

SELECT
    AtomicNumber
  , Element
  , Symbol
  , AtomicMass
  , NumberofNeutrons
  , NumberofProtons
  , NumberofElectrons
  , Period
  , Group
  , Phase
  , Radioactive
  , Natural
  , Metal
  , Nonmetal
  , Metalloid
  , Type
  , AtomicRadius
  , Electronegativity
  , FirstIonization
  , Density
  , MeltingPoint
  , BoilingPoint
  , NumberOfIsotopes
  , Discoverer
  , Year
  , SpecificHeat
  , NumberofShells
  , NumberofValence
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

select 
  max(Year) as latest_year_discovery,
  min(Year) as earliest_year_discovery
from periodic_table;
GO

SELECT 
  count (distinct(Type)) as number_types,
  max(Year) as latest_year_discovery,
  min(Year) as earliest_year_discovery
  FROM 
    periodic_table;
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


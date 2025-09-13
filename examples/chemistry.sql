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

SELECT Element, Symbol, Group, Type
FROM periodic_table
WHERE Type.Contains("Noble")
GO

SELECT MAX(Year) AS latest_year_discovery, MIN(Year) AS earliest_year_discovery
FROM periodic_table;
GO

SELECT COUNT(DISTINCT Type) AS number_types, MAX(Year) AS latest_year_discovery, MIN(Year) AS earliest_year_discovery
FROM periodic_table
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

-- ============================================
-- JOIN Examples with CTEs
-- ============================================

-- Find elements discovered in the latest year using JOIN
WITH year_stats AS (
    SELECT 
        MAX(Year) as max_year,
        MIN(Year) as min_year
    FROM periodic_table
    WHERE Year IS NOT NULL
)
SELECT 
    Element,
    Symbol,
    Year,
    Discoverer,
    Type
FROM periodic_table
INNER JOIN year_stats ON periodic_table.Year = year_stats.max_year
ORDER BY AtomicNumber;
GO

-- Find elements discovered in the earliest year
WITH year_stats AS (
    SELECT 
        MAX(Year) as max_year,
        MIN(Year) as min_year
    FROM periodic_table
    WHERE Year IS NOT NULL
)
SELECT 
    Element,
    Symbol,
    Year,
    Discoverer,
    Type
FROM periodic_table
INNER JOIN year_stats ON periodic_table.Year = year_stats.min_year
ORDER BY AtomicNumber;
GO

-- Find all 21st century discoveries
WITH recent_discoveries AS (
    SELECT 
        Element,
        Symbol,
        Year,
        Type,
        Discoverer
    FROM periodic_table
    WHERE Year >= 2000
)
SELECT 
    Element,
    Symbol, 
    Year,
    Type
FROM recent_discoveries
ORDER BY Year DESC, Element;
GO

-- Join periodic table with year analysis statistics
WITH year_analysis AS (
    SELECT 
        MAX(Year) as max_discovery_year,
        MIN(Year) as min_discovery_year,
        COUNT(DISTINCT Year) as unique_years,
        COUNT(*) as total_elements
    FROM periodic_table
    WHERE Year IS NOT NULL
)
SELECT 
    Element,
    Symbol,
    Year as discovery_year,
    Type,
    Discoverer,
    max_discovery_year,
    min_discovery_year
FROM periodic_table
INNER JOIN year_analysis ON periodic_table.Year = year_analysis.max_discovery_year;
GO

-- Show discovery statistics by joining CTEs
WITH discovery_stats AS (
    SELECT 
        Year,
        COUNT(*) as elements_discovered
    FROM periodic_table
    WHERE Year IS NOT NULL
    GROUP BY Year
),
year_bounds AS (
    SELECT 
        MAX(Year) as latest_year,
        MIN(Year) as earliest_year
    FROM periodic_table
    WHERE Year IS NOT NULL
)
SELECT 
    Year,
    elements_discovered,
    latest_year,
    earliest_year
FROM discovery_stats
INNER JOIN year_bounds ON Year = latest_year OR Year = earliest_year
ORDER BY Year;
GO

-- Self-join example: Find pairs of elements discovered in same year
-- Note: This demonstrates a workaround since table aliases don't work
WITH first_element AS (
    SELECT 
        Element as element1,
        Symbol as symbol1,
        Year as year1,
        AtomicNumber as atomic1
    FROM periodic_table
    WHERE Year = 2010 AND AtomicNumber = 115
),
second_element AS (
    SELECT 
        Element as element2,
        Symbol as symbol2,
       Year as year2,
        AtomicNumber as atomic2
    FROM periodic_table
    WHERE Year = 2010 AND AtomicNumber = 117
)
SELECT 
    element1,
    symbol1,
    element2,
    symbol2,
    year1
FROM first_element
INNER JOIN second_element ON first_element.year1 = second_element.year2;
GO

-- rather than cte directly use a sub query 

SELECT
    Element,
    Year
FROM periodic_table
WHERE Year = (SELECT
    MAX(Year)
FROM periodic_table);
GO


-- another example of how we can use nested sub query

SELECT
    Element,
    AtomicNumber,
    AtomicMass,
    NumberofProtons,
    Type
FROM periodic_table
WHERE NumberofProtons in (SELECT
    NumberofProtons
FROM periodic_table
WHERE Element in ('Hydrogen', 'Helium'));
GO

-- years in which the most discoveries were made

SELECT
  Year, count(*) as number_discoveries
from
  periodic_table
where Year > 0
  GROUP BY Year
  ORDER by number_discoveries desc
  limit 4;
GO

WITH
    discoveries AS (
        SELECT Year, COUNT('*') AS number_discoveries
        FROM periodic_table
        WHERE Year > 0
        GROUP BY Year
    ),
    elements AS (
        SELECT *
        FROM periodic_table
    )
SELECT MAX(number_discoveries) AS most_discoveries
FROM discoveries

GO

WITH
    discoveries AS (
        SELECT Year, COUNT('*') AS number_discoveries
        FROM periodic_table
        WHERE Year > 0
        GROUP BY Year
    ),
    elements AS (
        SELECT *
        FROM periodic_table
    )
SELECT *
FROM elements
WHERE Year = (SELECT MAX(number_discoveries from discoveries));

GO
 
-- LEFT JOIN example: Show all noble gases with discovery year stats
-- WITH all_noble_gases AS (
    -- SELECT 
        -- Element,
        -- Symbol,
        -- Year,
        -- AtomicMass
    -- FROM periodic_table
    -- WHERE Type = 'Noble Gas'
-- ),
-- year_stats AS (
    -- SELECT 
        -- MAX(Year) as max_year,
        -- MIN(Year) as min_year
    -- FROM periodic_table  
    -- WHERE Type = 'Noble Gas' AND Year IS NOT NULL
-- )
-- SELECT 
    -- Element,
    -- Symbol,
    -- Year,
    -- AtomicMass,
    -- max_year,
    -- min_year
-- FROM all_noble_gases
-- LEFT JOIN year_stats ON 1 = 1
-- ORDER BY AtomicMass;
GO

-- ============================================
-- Known Limitations in Current Implementation:
-- ============================================
-- 1. Table aliases (p.column) are not supported - parser interprets as method calls
--    DOESN'T WORK: SELECT p.Element FROM periodic_table p
--    WORKS: SELECT Element FROM periodic_table
--
-- 2. Qualified column names in SELECT cause issues
--    DOESN'T WORK: SELECT year_stats.max_year FROM ...
--    WORKS: SELECT max_year FROM ...
--
-- 3. JOIN conditions with string literals not supported
--    DOESN'T WORK: INNER JOIN type_stats ON type_stats.Type = 'Noble Gas'
--    WORKS: INNER JOIN type_stats ON periodic_table.Type = type_stats.Type
--
-- 4. CROSS JOIN with empty join condition has issues
--    DOESN'T WORK: CROSS JOIN (may error on missing column)
--
-- 5. Table-qualified columns in ORDER BY not recognized
--    DOESN'T WORK: ORDER BY all_people.salesperson
--    WORKS: ORDER BY salesperson
--
-- 6. Complex JOIN conditions limited to equality
--    Only supports: ON table1.col = table2.col
--    Not yet: ON table1.col > table2.col
-- ============================================


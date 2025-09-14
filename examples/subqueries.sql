-- #! ../data/periodic_table.csv
-- SQL CLI Subquery Examples

-- ============================================
-- SCALAR SUBQUERIES (Return single value)
-- ============================================

-- Find elements discovered in the most recent year
SELECT Element, Symbol, Year
FROM periodic_table
WHERE Year = (SELECT MAX(Year) FROM periodic_table);

GO
-- Find elements discovered in the earliest year
SELECT Element, Symbol, Year
FROM periodic_table
WHERE Year = (SELECT MIN(Year) FROM periodic_table);
GO

-- Find elements with atomic mass above average
SELECT Element, Symbol, AtomicMass
FROM periodic_table
WHERE AtomicMass > (SELECT AVG(AtomicMass) FROM periodic_table)
ORDER BY AtomicMass DESC
LIMIT 10;
GO

-- Count elements discovered after median year
SELECT COUNT(*) as modern_elements
FROM periodic_table
WHERE Year > (SELECT AVG(Year) FROM periodic_table WHERE Year IS NOT NULL);
GO

-- ============================================
-- IN SUBQUERIES (Check membership in set)
-- ============================================

-- Find all noble gases (Group 18 elements)
SELECT Element, Symbol, AtomicNumber
FROM periodic_table
WHERE Element IN (SELECT Element FROM periodic_table WHERE Group = 18)
ORDER BY AtomicNumber;
GO

-- Find radioactive elements discovered after 2000
SELECT Element, Symbol, Year
FROM periodic_table
WHERE Element IN (
    SELECT Element 
    FROM periodic_table 
    WHERE Radioactive = 'yes' AND Year > 2000
)
ORDER BY Year DESC;
GO

-- Find elements in the same period as Gold
SELECT Element, Symbol, Period
FROM periodic_table
WHERE Period IN (SELECT Period FROM periodic_table WHERE Element = 'Gold')
ORDER BY AtomicNumber;
GO

-- ============================================
-- NOT IN SUBQUERIES (Check non-membership)
-- ============================================

-- Find non-radioactive elements
SELECT COUNT(*) as stable_elements
FROM periodic_table
WHERE Element NOT IN (
    SELECT Element 
    FROM periodic_table 
    WHERE Radioactive = 'yes'
);
GO

-- Find elements not discovered in the 20th century
SELECT Element, Symbol, Year
FROM periodic_table
WHERE Element NOT IN (
    SELECT Element 
    FROM periodic_table 
    WHERE Year >= 1900 AND Year < 2000
)
AND Year IS NOT NULL
ORDER BY Year;
GO

-- Find metals that are not in Group 1 (alkali metals)
SELECT Element, Symbol, Group
FROM periodic_table
WHERE Metal = 'yes' 
AND Element NOT IN (
    SELECT Element FROM periodic_table WHERE Group = 1
)
ORDER BY AtomicNumber
LIMIT 20;
GO

-- ============================================
-- COMPLEX SUBQUERY COMBINATIONS
-- ============================================

-- Find elements discovered in the same year as any noble gas
SELECT Element, Symbol, Year, Group
FROM periodic_table
WHERE Year IN (
    SELECT Year 
    FROM periodic_table 
    WHERE Group = 18 AND Year IS NOT NULL
)
ORDER BY Year, AtomicNumber;
GO

-- Find the heaviest element from each period
-- SELECT p1.Period, p1.Element, p1.AtomicMass
-- FROM periodic_table p1
-- WHERE p1.AtomicMass = (
    -- SELECT MAX(p2.AtomicMass)
    -- FROM periodic_table p2
    -- WHERE p2.Period = p1.Period
--   )
-- ORDER BY p1.Period;
--GO

-- Count elements by phase, excluding artificial elements
SELECT Phase, COUNT(*) as count
FROM periodic_table
WHERE Element NOT IN (
    SELECT Element 
    FROM periodic_table 
    WHERE Natural = 'no' OR Phase = 'artificial'
)
GROUP BY Phase
ORDER BY count DESC;
GO

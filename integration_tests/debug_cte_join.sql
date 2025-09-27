-- Simplified test case to debug CTE JOIN propagation
-- #! ../data/periodic_table.csv

-- Step 1: Test basic CTE works
WITH simple_years AS (
    SELECT Year, Element FROM periodic_table WHERE Year > 1800 AND Year < 1900
)
SELECT COUNT(*) as simple_count FROM simple_years;
GO

-- Step 2: Test CTE with JOIN works directly
WITH
    discoveries AS (
        SELECT Year, COUNT(*) AS number_discoveries
        FROM periodic_table
        WHERE Year > 0
        GROUP BY Year
    ),
    elements AS (
        SELECT Year, Element FROM periodic_table
    ),
    combined AS (
        SELECT
            Year,
            Element,
            number_discoveries
        FROM elements
        INNER JOIN discoveries ON elements.Year = discoveries.Year
        WHERE number_discoveries >= 3
    )
SELECT COUNT(*) as join_count FROM combined;
GO

-- Step 3: Test CTE chaining (this should fail)
WITH
    discoveries AS (
        SELECT Year, COUNT(*) AS number_discoveries
        FROM periodic_table
        WHERE Year > 0
        GROUP BY Year
    ),
    elements AS (
        SELECT Year, Element FROM periodic_table
    ),
    combined AS (
        SELECT
            Year,
            Element,
            number_discoveries
        FROM elements
        INNER JOIN discoveries ON elements.Year = discoveries.Year
        WHERE number_discoveries >= 3
    ),
    final_step AS (
        SELECT Year, COUNT(*) as element_count FROM combined GROUP BY Year
    )
SELECT COUNT(*) as final_count FROM final_step;
GO
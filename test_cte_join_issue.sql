-- Test case for CTE JOIN propagation issue
-- #! ../data/periodic_table.csv

-- First test: This should work (direct query of combined CTE)
WITH
    discoveries AS (
        SELECT Year, COUNT(*) AS number_discoveries
        FROM periodic_table
        WHERE Year > 0
        GROUP BY Year
    ),
    elements AS (
        SELECT *, ROW_NUMBER() OVER (PARTITION BY Year ORDER BY Year ASC) AS grp_member
        FROM periodic_table
    ),
    combined AS (
        SELECT
            Element,
            Symbol,
            Year,
            number_discoveries,
            grp_member,
            GROUP_NUM(Year) AS year_grp
        FROM elements
        INNER JOIN discoveries ON elements.Year = discoveries.Year
        WHERE Year IN (
            SELECT Year
            FROM discoveries
            WHERE number_discoveries > 3
        )
    )
SELECT * FROM combined
ORDER BY Year DESC
LIMIT 5;
GO

-- Second test: This should fail (subsequent CTE using combined)
WITH
    discoveries AS (
        SELECT Year, COUNT(*) AS number_discoveries
        FROM periodic_table
        WHERE Year > 0
        GROUP BY Year
    ),
    elements AS (
        SELECT *, ROW_NUMBER() OVER (PARTITION BY Year ORDER BY Year ASC) AS grp_member
        FROM periodic_table
    ),
    combined AS (
        SELECT
            Element,
            Symbol,
            Year,
            number_discoveries,
            grp_member,
            GROUP_NUM(Year) AS year_grp
        FROM elements
        INNER JOIN discoveries ON elements.Year = discoveries.Year
        WHERE Year IN (
            SELECT Year
            FROM discoveries
            WHERE number_discoveries > 3
        )
    ),
    final_analysis AS (
        SELECT
            Year,
            COUNT(*) as element_count,
            AVG(number_discoveries) as avg_discoveries
        FROM combined  -- This should reference the JOIN result from combined CTE
        GROUP BY Year
    )
SELECT * FROM final_analysis
ORDER BY Year DESC;
GO
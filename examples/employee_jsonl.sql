-- Employee JSONL starter: querying AdventureWorks-style HR records.
--
-- data/employee.jsonl is one JSON object per line (10 records). Same
-- READ_JSONL machinery as examples/jsonl_logs.sql, applied to a more
-- structured business dataset.
--
-- Run from the repo root:
--   ./target/release/sql-cli -f examples/employee_jsonl.sql


-- Headcount, gender split, salaried split.
SELECT
    COUNT(*)                                         AS headcount,
    SUM(CASE WHEN Gender = 'M' THEN 1 ELSE 0 END)    AS male,
    SUM(CASE WHEN Gender = 'F' THEN 1 ELSE 0 END)    AS female,
    SUM(CASE WHEN SalariedFlag THEN 1 ELSE 0 END)    AS salaried
FROM READ_JSONL('data/employee.jsonl');
GO

-- Distribution by organisation level.
SELECT OrganizationLevel, COUNT(*) AS people
FROM READ_JSONL('data/employee.jsonl')
GROUP BY OrganizationLevel
ORDER BY OrganizationLevel;
GO

-- Top vacation balances — useful for end-of-year planning.
SELECT JobTitle, Gender, VacationHours, SickLeaveHours
FROM READ_JSONL('data/employee.jsonl')
ORDER BY VacationHours DESC
LIMIT 5;
GO

-- Tenure: years between HireDate and a fixed reference date. Demonstrates
-- that JSONL date strings flow straight into the engine's date functions.
SELECT
    JobTitle,
    HireDate,
    DATEDIFF('year', HireDate, '2026-01-01') AS years_with_company
FROM READ_JSONL('data/employee.jsonl')
ORDER BY years_with_company DESC;
GO

-- Quick sanity check: filter on a domain field.
SELECT BusinessEntityID, JobTitle, MaritalStatus
FROM READ_JSONL('data/employee.jsonl')
WHERE MaritalStatus = 'M'
ORDER BY BusinessEntityID;
GO

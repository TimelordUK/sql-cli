-- JOIN with Expression Support - Two File Example
-- Demonstrating TRIM() to handle padded database exports
--
-- This is the most common real-world use case: joining two CSV files
-- where one has trailing spaces from a database export.

-- Example 1: Basic JOIN with TRIM() to handle padded spaces
WITH
    WEB portfolios AS (URL 'file://data/portfolios.csv' FORMAT CSV),
    WEB fund_names AS (URL 'file://data/fund_names_padded.csv' FORMAT CSV)
SELECT
    portfolios.*,                           -- All portfolio columns
    fund_names.fund_id,
    fund_names.manager,
    fund_names.Name as padded_name,
    TRIM(fund_names.Name) as trimmed_name,
    LENGTH(fund_names.Name) as name_length
FROM portfolios
JOIN fund_names ON portfolios.portfolio = TRIM(fund_names.Name);
GO

-- Example 2: Nested functions - UPPER(TRIM())
-- Combine TRIM with UPPER for case-insensitive + padding-tolerant matching
WITH
    WEB portfolios AS (URL 'file://data/portfolios.csv' FORMAT CSV),
    WEB fund_names AS (URL 'file://data/fund_names_padded.csv' FORMAT CSV)
SELECT
    portfolios.portfolio as internal_name,
    fund_names.Name as external_name_padded,
    UPPER(TRIM(fund_names.Name)) as external_name_normalized,
    fund_names.manager,
    portfolios.value
FROM portfolios
JOIN fund_names ON portfolios.portfolio = UPPER(TRIM(fund_names.Name));
GO

-- Example 3: Using scoped star expansion with JOIN
-- Select all columns from portfolios, specific columns from fund_names
WITH
    WEB portfolios AS (URL 'file://data/portfolios.csv' FORMAT CSV),
    WEB fund_names AS (URL 'file://data/fund_names_padded.csv' FORMAT CSV)
SELECT
    portfolios.*,                  -- Expands to: id, portfolio, value
    fund_names.fund_id,
    fund_names.manager
FROM portfolios
JOIN fund_names ON portfolios.portfolio = TRIM(fund_names.Name);
GO

EXIT;
go

-- JOIN with LEFT-side Expression Support - Phase 2 Examples (TEMPLATE)
-- ⚠️  NOTE: This file references CSV files that don't exist in the repo.
-- ⚠️  This is a TEMPLATE/DOCUMENTATION file showing example use cases.
-- ⚠️  For runnable examples, see: join_left_expression_demo.sql
--
-- Demonstrating expressions on the LEFT side of JOIN conditions
--
-- This mirrors Phase 1 functionality (which supported RIGHT-side expressions)
-- Now we can use functions and expressions on the LEFT side while RIGHT remains simple

-- Example 1: LEFT-side TRIM() - Clean up padded data on left side
-- Use case: Legacy system exports data with trailing spaces
WITH
    WEB users AS (URL 'file://data/users_padded.csv' FORMAT CSV),
    WEB contacts AS (URL 'file://data/contacts.csv' FORMAT CSV)
SELECT
    users.user_id,
    users.name as padded_name,
    TRIM(users.name) as trimmed_name,
    LENGTH(users.name) as padded_length,
    contacts.email,
    contacts.phone
FROM users
JOIN contacts ON TRIM(users.name) = contacts.name;
GO

-- Example 2: LEFT-side UPPER() for case-insensitive matching
-- Use case: User data has mixed case, master data is uppercase
WITH
    WEB transactions AS (URL 'file://data/transactions.csv' FORMAT CSV),
    WEB products AS (URL 'file://data/products_upper.csv' FORMAT CSV)
SELECT
    transactions.tx_id,
    transactions.product_code as original_code,
    UPPER(transactions.product_code) as normalized_code,
    products.product_name,
    products.category
FROM transactions
JOIN products ON UPPER(transactions.product_code) = products.PRODUCT_CODE;
GO

-- Example 3: LEFT-side SUBSTRING() - Extract key prefix
-- Use case: Transaction IDs contain embedded codes that need extraction
WITH
    WEB orders AS (URL 'file://data/orders.csv' FORMAT CSV),
    WEB regions AS (URL 'file://data/regions.csv' FORMAT CSV)
SELECT
    orders.order_id,
    SUBSTRING(orders.order_id, 1, 3) as region_code,
    regions.region_name,
    regions.country,
    orders.amount
FROM orders
JOIN regions ON SUBSTRING(orders.order_id, 1, 3) = regions.code;
GO

-- Example 4: LEFT-side nested functions - UPPER(TRIM())
-- Use case: Clean and normalize left side data for matching
WITH
    WEB legacy AS (URL 'file://data/legacy_codes.csv' FORMAT CSV),
    WEB master AS (URL 'file://data/master_codes.csv' FORMAT CSV)
SELECT
    legacy.code as legacy_padded,
    TRIM(legacy.code) as legacy_trimmed,
    UPPER(TRIM(legacy.code)) as legacy_normalized,
    master.code as master_code,
    master.description
FROM legacy
JOIN master ON UPPER(TRIM(legacy.code)) = master.code_clean;
GO

-- Example 5: Both sides with expressions (Phase 1 + Phase 2 combined!)
-- Use case: Both datasets need normalization
WITH
    WEB customers AS (URL 'file://data/customers.csv' FORMAT CSV),
    WEB accounts AS (URL 'file://data/accounts.csv' FORMAT CSV)
SELECT
    customers.customer_id,
    customers.email as customer_email,
    LOWER(TRIM(customers.email)) as normalized_customer,
    accounts.account_id,
    accounts.email as account_email,
    LOWER(TRIM(accounts.email)) as normalized_account
FROM customers
JOIN accounts ON LOWER(TRIM(customers.email)) = LOWER(TRIM(accounts.email));
GO

-- Example 6: Multiple conditions with LEFT-side expressions
-- Use case: Composite key join with data cleaning on left
WITH
    WEB sales AS (URL 'file://data/sales.csv' FORMAT CSV),
    WEB pricing AS (URL 'file://data/pricing.csv' FORMAT CSV)
SELECT
    sales.sale_id,
    sales.product as padded_product,
    TRIM(sales.product) as clean_product,
    sales.region,
    pricing.base_price,
    pricing.discount
FROM sales
JOIN pricing
    ON TRIM(sales.product) = pricing.product
    AND sales.region = pricing.region;
GO

-- Example 7: Arithmetic expressions on LEFT side
-- Use case: Calculate derived key on left for matching
WITH
    WEB measurements AS (URL 'file://data/measurements.csv' FORMAT CSV),
    WEB bins AS (URL 'file://data/bins.csv' FORMAT CSV)
SELECT
    measurements.id,
    measurements.value,
    measurements.value / 10 as bin_key,
    bins.bin_id,
    bins.category
FROM measurements
JOIN bins ON measurements.value / 10 = bins.bin_id;
GO

-- Example 8: Complex LEFT expression with RIGHT simple column
-- Use case: Concatenate parts on left to form key
WITH
    WEB parts AS (URL 'file://data/parts.csv' FORMAT CSV),
    WEB assemblies AS (URL 'file://data/assemblies.csv' FORMAT CSV)
SELECT
    parts.prefix,
    parts.suffix,
    CONCAT(parts.prefix, '-', parts.suffix) as full_code,
    assemblies.assembly_code,
    assemblies.description
FROM parts
JOIN assemblies ON CONCAT(parts.prefix, '-', parts.suffix) = assemblies.assembly_code;
GO

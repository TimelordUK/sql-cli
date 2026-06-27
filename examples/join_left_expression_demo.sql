-- JOIN Left-Side Expression Demo
-- Phase 2: Expressions on LEFT side (mirrors Phase 1 which supported RIGHT side)

-- Example 1: LEFT-side TRIM() - Remove padding from left table
WITH
    users_padded AS (
        SELECT 1 as user_id, 'Alice     ' as name
        UNION ALL SELECT 2, 'Bob       '
        UNION ALL SELECT 3, 'Carol     '
    ),
    contacts AS (
        SELECT 101 as contact_id, 'Alice' as name, 'alice@example.com' as email
        UNION ALL SELECT 102, 'Bob', 'bob@example.com'
        UNION ALL SELECT 103, 'Carol', 'carol@example.com'
    )
SELECT
    users_padded.user_id,
    users_padded.name as padded_name,
    TRIM(users_padded.name) as trimmed_name,
    LENGTH(users_padded.name) as padded_len,
    contacts.contact_id,
    contacts.email
FROM users_padded
JOIN contacts ON TRIM(users_padded.name) = contacts.name;
GO

-- Example 2: LEFT-side UPPER() - Case-insensitive matching
WITH
    products AS (
        SELECT 1 as id, 'widget' as code
        UNION ALL SELECT 2, 'gadget'
        UNION ALL SELECT 3, 'tool'
    ),
    inventory AS (
        SELECT 101 as inv_id, 'WIDGET' as CODE, 50 as quantity
        UNION ALL SELECT 102, 'GADGET', 30
        UNION ALL SELECT 103, 'TOOL', 75
    )
SELECT
    products.id,
    products.code as lowercase_code,
    UPPER(products.code) as uppercase_code,
    inventory.inv_id,
    inventory.quantity
FROM products
JOIN inventory ON UPPER(products.code) = inventory.CODE;
GO

-- Example 3: LEFT-side SUBSTRING() - Extract key prefix
-- Note: SQL SUBSTRING() is 1-indexed (SQL standard); the C# `.Substring()`
-- method form is 0-indexed.
WITH
    orders AS (
        SELECT 'NYC-12345' as order_id, 100.00 as amount
        UNION ALL SELECT 'LAX-67890', 200.00
        UNION ALL SELECT 'CHI-11111', 150.00
    ),
    regions AS (
        SELECT 'NYC' as code, 'New York' as name
        UNION ALL SELECT 'LAX', 'Los Angeles'
        UNION ALL SELECT 'CHI', 'Chicago'
    )
SELECT
    orders.order_id,
    SUBSTRING(orders.order_id, 1, 3) as region_code,
    regions.name as region_name,
    orders.amount
FROM orders
JOIN regions ON SUBSTRING(orders.order_id, 1, 3) = regions.code;
GO

-- Example 4: LEFT-side nested functions - UPPER(TRIM())
WITH
    legacy AS (
        SELECT 1 as id, 'tech   ' as code
        UNION ALL SELECT 2, 'bonds  '
        UNION ALL SELECT 3, 'equity '
    ),
    master AS (
        SELECT 101 as master_id, 'TECH' as code, 'Technology Fund' as description
        UNION ALL SELECT 102, 'BONDS', 'Bond Portfolio'
        UNION ALL SELECT 103, 'EQUITY', 'Equity Fund'
    )
SELECT
    legacy.id,
    legacy.code as legacy_padded,
    TRIM(legacy.code) as legacy_trimmed,
    UPPER(TRIM(legacy.code)) as legacy_normalized,
    master.master_id,
    master.description
FROM legacy
JOIN master ON UPPER(TRIM(legacy.code)) = master.code;
GO

-- Example 5: Both sides with expressions (Phase 1 + Phase 2 combined!)
WITH
    customers AS (
        SELECT 1 as id, '  Alice@EXAMPLE.COM  ' as email
        UNION ALL SELECT 2, '  bob@example.com  '
        UNION ALL SELECT 3, '  CAROL@Example.Com  '
    ),
    accounts AS (
        SELECT 101 as acct_id, '  alice@example.com  ' as email, 5000.00 as balance
        UNION ALL SELECT 102, '  BOB@EXAMPLE.COM  ', 7500.00
        UNION ALL SELECT 103, '  carol@example.com  ', 3200.00
    )
SELECT
    customers.id,
    customers.email as customer_email,
    LOWER(TRIM(customers.email)) as customer_normalized,
    accounts.acct_id,
    accounts.email as account_email,
    LOWER(TRIM(accounts.email)) as account_normalized,
    accounts.balance
FROM customers
JOIN accounts ON LOWER(TRIM(customers.email)) = LOWER(TRIM(accounts.email));
GO

-- Example 6: Multiple conditions with LEFT-side expressions
WITH
    sales AS (
        SELECT 1 as id, 'widget  ' as product, 'East' as region, 100.00 as amount
        UNION ALL SELECT 2, 'gadget  ', 'West', 150.00
        UNION ALL SELECT 3, 'widget  ', 'West', 120.00
    ),
    pricing AS (
        SELECT 'widget' as product, 'East' as region, 10.00 as discount
        UNION ALL SELECT 'gadget', 'West', 15.00
        UNION ALL SELECT 'widget', 'West', 12.00
    )
SELECT
    sales.id,
    sales.product as padded_product,
    TRIM(sales.product) as clean_product,
    sales.region,
    sales.amount,
    pricing.discount,
    sales.amount - pricing.discount as net_amount
FROM sales
JOIN pricing
    ON TRIM(sales.product) = pricing.product
    AND sales.region = pricing.region;
GO

-- Example 7: Arithmetic expression on LEFT side
WITH
    measurements AS (
        SELECT 1 as id, 20 as value
        UNION ALL SELECT 2, 50
        UNION ALL SELECT 3, 80
    ),
    bins AS (
        SELECT 2 as bin_id, 'Low' as category
        UNION ALL SELECT 5, 'Medium'
        UNION ALL SELECT 8, 'High'
    )
SELECT
    measurements.id,
    measurements.value,
    measurements.value / 10 as calculated_bin,
    bins.bin_id,
    bins.category
FROM measurements
JOIN bins ON measurements.value / 10 = bins.bin_id;
GO

-- Example 8: String concatenation on LEFT side (using || operator)
WITH
    parts AS (
        SELECT 1 as id, 'A' as prefix, '100' as suffix
        UNION ALL SELECT 2, 'B', '200'
        UNION ALL SELECT 3, 'C', '300'
    ),
    assemblies AS (
        SELECT 'A-100' as code, 'Assembly Alpha' as description
        UNION ALL SELECT 'B-200', 'Assembly Beta'
        UNION ALL SELECT 'C-300', 'Assembly Gamma'
    )
SELECT
    parts.id,
    parts.prefix,
    parts.suffix,
    parts.prefix || '-' || parts.suffix as full_code,
    assemblies.code as assembly_code,
    assemblies.description
FROM parts
JOIN assemblies ON parts.prefix || '-' || parts.suffix = assemblies.code;
GO

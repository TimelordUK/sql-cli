-- FIX Protocol Analysis - Simple Examples
-- Working examples that demonstrate FIX protocol analysis with file CTEs

-- ===============================================
-- Example 1: List All FIX Message Types
-- ===============================================
WITH WEB messages AS (URL 'file://data/FIX44_messages.csv')
SELECT DISTINCT
    message_name,
    msgtype,
    msgcat
FROM messages
ORDER BY msgcat, message_name
LIMIT 20;
GO

-- ===============================================
-- Example 2: Message Field Count Summary
-- ===============================================
WITH WEB messages AS (URL 'file://data/FIX44_messages.csv')
SELECT
    message_name,
    msgtype,
    msgcat,
    COUNT(*) as total_fields,
    SUM(CASE WHEN required = 'Y' THEN 1 ELSE 0 END) as required_fields,
    SUM(CASE WHEN required = 'N' THEN 1 ELSE 0 END) as optional_fields
FROM messages
GROUP BY message_name, msgtype, msgcat
ORDER BY total_fields DESC
LIMIT 15;
GO

-- ===============================================
-- Example 3: Most Complex Messages
-- ===============================================
WITH WEB messages AS (URL 'file://data/FIX44_messages.csv')
SELECT
    message_name,
    msgtype,
    COUNT(*) as field_count,
    SUM(CASE WHEN field_type = 'GROUP' THEN 1 ELSE 0 END) as group_count,
    COUNT(DISTINCT field_type) as type_variety
FROM messages
GROUP BY message_name, msgtype
ORDER BY field_count DESC;
GO

-- ===============================================
-- Example 4: Field Type Distribution
-- ===============================================
WITH WEB messages AS (URL 'file://data/FIX44_messages.csv')
SELECT
    field_type,
    COUNT(*) as occurrences,
    COUNT(DISTINCT field_name) as unique_fields,
    ROUND(COUNT(*) * 100.0 / (SELECT COUNT(*) FROM messages), 2) as percentage
FROM messages
GROUP BY field_type
ORDER BY occurrences DESC;
GO

-- ===============================================
-- Example 5: Required Fields Analysis
-- ===============================================
WITH WEB messages AS (URL 'file://data/FIX44_messages.csv')
SELECT
    field_name,
    field_number,
    COUNT(DISTINCT message_name) as used_in_messages,
    field_type
FROM messages
WHERE required = 'Y'
GROUP BY field_name, field_number, field_type
ORDER BY used_in_messages DESC
LIMIT 20;
GO

-- ===============================================
-- Example 6: Trading Messages Overview
-- ===============================================
WITH WEB messages AS (URL 'file://data/FIX44_messages.csv')
SELECT
    message_name,
    msgtype,
    COUNT(*) as field_count
FROM messages
WHERE message_name IN (
    'NewOrderSingle',
    'ExecutionReport',
    'OrderCancelRequest',
    'MarketDataRequest',
    'QuoteRequest',
    'Quote'
)
GROUP BY message_name, msgtype
ORDER BY field_count DESC;
GO

-- ===============================================
-- Example 7: Fields in NewOrderSingle Message
-- ===============================================
WITH WEB messages AS (URL 'file://data/FIX44_messages.csv')
SELECT
    field_name,
    field_number,
    field_type,
    required,
    position
FROM messages
WHERE message_name = 'NewOrderSingle'
ORDER BY position
LIMIT 30;
GO

-- ===============================================
-- Example 8: Enum Values for Order Status
-- ===============================================
WITH WEB enums AS (URL 'file://data/FIX44_enums.csv')
SELECT
    field_name,
    enum as value,
    description
FROM enums
WHERE field_name = 'OrdStatus'
ORDER BY value;
GO

-- ===============================================
-- Example 9: Field Descriptions
-- ===============================================
WITH WEB fields AS (URL 'file://data/FIX44_fields.csv')
SELECT
    number,
    name,
    type,
    SUBSTRING(description, 1, 80) as description_preview
FROM fields
WHERE name IN (
    'OrderID', 'ClOrdID', 'Symbol', 'Side',
    'OrderQty', 'Price', 'OrdType', 'TimeInForce'
)
ORDER BY name;
GO

-- ===============================================
-- Example 10: Components Analysis
-- ===============================================
WITH WEB components AS (URL 'file://data/FIX44_components.csv')
SELECT
    component_name,
    COUNT(*) as field_count,
    SUM(CASE WHEN required = 'Y' THEN 1 ELSE 0 END) as required_fields
FROM components
GROUP BY component_name
ORDER BY field_count DESC
LIMIT 15;
GO

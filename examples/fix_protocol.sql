-- FIX Protocol Message Analysis using File CTEs
-- This example demonstrates how to join FIX protocol tables to create enriched message definitions

WITH
    WEB fields AS (URL 'file://data/FIX44_fields.csv')
select * from fields 
limit 20;
GO

WITH
    WEB messages AS (URL 'file://data/FIX44_messages.csv')
select * from messages
limit 20;
GO

-- ===============================================
-- Example 1: Simple Message Field Enrichment
-- Get all fields for a specific message with descriptions
-- ===============================================
WITH
    WEB messages AS (URL 'file://data/FIX44_messages.csv'),
    WEB field AS (URL 'file://data/FIX44_fields.csv')
SELECT
    messages.message_name,
    position,
    messages.msgtype,
    messages.field_name,
    messages.field_number,
    messages.required,
    messages.field_type,
    field.number as field_number
FROM messages
LEFT JOIN field ON messages.field_number = field.number
WHERE messages.message_name = 'NewOrderSingle'
ORDER BY position;
GO

-- ===============================================
-- Example 2: Message Summary with Field Counts
-- Show message categories with required/optional field counts
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
ORDER BY msgcat, total_fields DESC
LIMIT 15;
GO

-- ===============================================
-- Example 3: Find Enum Values for Specific Fields
-- Show all possible values for OrdStatus field
-- ===============================================
WITH
    WEB enums AS (URL 'file://data/FIX44_enums.csv'),
    WEB fields AS (URL 'file://data/FIX44_fields.csv')
SELECT
    name as field_name,
    enum as value,
    description as meaning
FROM enums
INNER JOIN fields ON enums.field_number = fields.number
WHERE fields.name = 'OrdStatus'
ORDER BY value;
GO

-- ===============================================
-- Example 4: Most Common Required Fields
-- Find which required fields appear in the most messages
-- ===============================================
WITH
    WEB messages AS (
        URL 'file://data/FIX44_messages.csv'
    )
SELECT
    field_name,
    field_number,
    field_type,
    COUNT(DISTINCT message_name) AS used_in_messages
FROM messages
WHERE required = 'Y'
GROUP BY field_name, field_number, field_type
ORDER BY used_in_messages DESC
LIMIT 20;
GO


-- ===============================================
-- Example 5: Message Complexity Score
-- Calculate a complexity score based on field counts and types
-- ===============================================
WITH WEB messages AS (URL 'file://data/FIX44_messages.csv')
SELECT
    message_name,
    msgtype,
    msgcat,
    COUNT(*) as total_fields,
    SUM(CASE WHEN required = 'Y' THEN 2 ELSE 1 END) as complexity_score,
    COUNT(DISTINCT field_type) as unique_types,
    SUM(CASE WHEN field_type = 'GROUP' THEN 1 ELSE 0 END) as group_fields
FROM messages
GROUP BY message_name, msgtype, msgcat
ORDER BY complexity_score DESC
LIMIT 15;
GO

-- ===============================================
-- Example 6: Field Type Distribution
-- Analyze the distribution of field types across all messages
-- ===============================================
WITH WEB messages AS (URL 'file://data/FIX44_messages.csv')
SELECT
    field_type,
    COUNT(*) as total_occurrences,
    COUNT(DISTINCT field_name) as unique_fields,
    COUNT(DISTINCT message_name) as used_in_messages,
    ROUND(COUNT(*) * 100.0 / (SELECT COUNT(*) FROM messages), 2) as percentage
FROM messages
GROUP BY field_type
ORDER BY total_occurrences DESC;
GO

-- ===============================================
-- Example 7: Trading Message Analysis
-- Focus on core trading messages
-- ===============================================
WITH WEB messages AS (URL 'file://data/FIX44_messages.csv')
SELECT
    message_name,
    msgtype,
    COUNT(*) as field_count,
    SUM(CASE WHEN required = 'Y' THEN 1 ELSE 0 END) as required_count
FROM messages
WHERE message_name IN (
    'NewOrderSingle', 'ExecutionReport', 'OrderCancelRequest',
    'OrderCancelReplaceRequest', 'OrderStatusRequest',
    'MarketDataRequest', 'MarketDataSnapshotFullRefresh'
)
GROUP BY message_name, msgtype
ORDER BY field_count DESC;
GO

-- ===============================================
-- Example 8: Component Field Analysis
-- Analyze fields in the Instrument component
-- ===============================================
WITH WEB components AS (URL 'file://data/FIX44_components.csv')
SELECT
    component_name,
    field_name,
    field_type,
    required,
    COUNT(*) as occurrences
FROM components
WHERE component_name = 'Instrument'
GROUP BY component_name, field_name, field_type, required
ORDER BY field_name;
GO

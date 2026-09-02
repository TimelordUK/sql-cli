-- [TEST:SKIP] needs a web service to accept the FORM_FILE upload
-- Example: Using Web CTE with file uploads
-- This demonstrates the FORM_FILE syntax for uploading files to web services

-- Example 1: Basic file upload with POST request
-- Note: Replace with your actual endpoint that accepts file uploads
WITH WEB uploaded_data AS (
    URL 'https://httpbin.org/post'  -- Echo service for testing
    METHOD POST
    FORM_FILE 'file' '../data/sales_data.csv'
    FORM_FIELD 'description' 'Sales data upload'
    FORMAT JSON
)
SELECT * FROM uploaded_data LIMIT 1;
GO

-- Example 2: Multiple form fields with file upload
WITH WEB api_response AS (
    URL 'https://httpbin.org/post'
    METHOD POST
    FORM_FILE 'data_file' '../data/test_simple_math.csv'
    FORM_FIELD 'user' 'admin'
    FORM_FIELD 'action' 'process'
    FORM_FIELD 'format' 'csv'
    FORMAT JSON
)
SELECT * FROM api_response LIMIT 1;
GO

-- Example 3: Upload JSON file for processing
-- This would typically be used with a service that processes JSON files
WITH WEB json_upload AS (
    URL 'http://localhost:5050/api/messagequery/example/fix'
    METHOD POST
    FORM_FILE 'file' '../data/fix_messages.json'
    FORMAT CSV
)
SELECT
    msg_type,
    symbol,
    quantity,
    price
FROM json_upload
WHERE quantity > 500
ORDER BY price DESC
LIMIT 10;
GO

-- Syntax reference:
-- WITH WEB table_name AS (
--     URL 'endpoint_url'
--     METHOD POST                                  -- HTTP method
--     FORM_FILE 'field_name' 'path/to/file'       -- File upload field
--     FORM_FIELD 'field_name' 'value'             -- Additional form fields
--     FORMAT CSV|JSON|AUTO                        -- Expected response format
--     CACHE seconds                               -- Optional caching
--     HEADERS 'Header: value'                     -- Optional headers
-- )

-- Note: FORM_FILE and FORM_FIELD create a multipart/form-data request
-- This is equivalent to HTML form submission with enctype="multipart/form-data"
-- or curl -F "field_name=@file_path" in command line
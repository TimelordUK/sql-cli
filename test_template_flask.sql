-- Test using the WEB_QUERY template
-- Place cursor on @WEB_QUERY and press \ste to expand the template
-- Then press \sx to execute with parameter prompts

-- Option 1: Use default localhost URL
-- @WEB_QUERY

-- Option 2: Override the URL for production
-- Macro: TRADE_API_URL
-- Default: https://api.yourcompany.com/trades
-- @WEB_QUERY

-- After expansion, you can add additional processing:
-- GROUP BY Source
-- ORDER BY PlatformOrderId
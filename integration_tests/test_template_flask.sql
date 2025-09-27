-- Test using the WEB_QUERY template
-- Place cursor on @WEB_QUERY and press \ste to expand the template
-- Then press \sx to execute with parameter prompts

-- Option 1: Use default localhost URL and test token
-- @WEB_QUERY

-- Option 2: Override for production (define these BEFORE @WEB_QUERY)
-- Macro: TRADE_API_URL
-- https://api.yourcompany.com/trades
-- END MACRO
--
-- Macro: JWT_TOKEN_DEFAULT
-- your-production-token-here
-- END MACRO
--
-- @WEB_QUERY

-- After expansion, you can add additional processing:
-- GROUP BY Source
-- ORDER BY PlatformOrderId
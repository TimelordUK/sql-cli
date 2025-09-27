#!/bin/bash
# Setup environment for testing Web CTE templates

# Set a test JWT token (can be any dummy value for local testing)
export JWT_TOKEN="test-token-123"

echo "Test environment configured:"
echo "  JWT_TOKEN=$JWT_TOKEN"
echo ""
echo "You can now:"
echo "1. Start the test server: cd tests && uv run python test_trade_server.py"
echo "2. Open test_template_flask.sql in nvim"
echo "3. Place cursor on @WEB_QUERY and press \\ste to expand template"
echo "4. Press \\sx to execute with runtime parameters"
echo ""
echo "The template uses a TRADE_API_URL macro which defaults to:"
echo "  http://localhost:5001/trades"
echo ""
echo "To override for production, add this macro before @WEB_QUERY:"
echo "  -- Macro: TRADE_API_URL"
echo "  -- Default: https://api.yourcompany.com/trades"
echo ""
echo "At runtime you'll be prompted for:"
echo "  - SOURCE (e.g., Bloomberg, Reuters, TradeWeb)"
echo "  - DATE (e.g., 2024-03-27)"
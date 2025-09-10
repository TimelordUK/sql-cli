#!/bin/bash

echo "Testing SQL CLI Neovim Plugin Autocomplete"
echo "==========================================="
echo ""

# Test 1: Check if --schema-json works
echo "Test 1: Checking --schema-json flag..."
./target/release/sql-cli data/test_simple_strings.csv --schema-json > /tmp/schema_test.json 2>&1
if [ $? -eq 0 ]; then
    echo "✓ --schema-json flag works"
    echo "Schema output:"
    cat /tmp/schema_test.json | head -5
    echo "..."
else
    echo "✗ --schema-json flag failed"
    exit 1
fi

echo ""
echo "Test 2: Testing Neovim plugin loading..."
echo ""
echo "To test the Neovim plugin interactively:"
echo "1. Open Neovim: nvim test_autocomplete.sql"
echo "2. The plugin should auto-detect the data file from the comment"
echo "3. In insert mode, try:"
echo "   - Type 'SELECT ' then press Alt+; to see column completions"
echo "   - Type 'SELECT na' then press Alt+; to see filtered completions"
echo "   - Press Ctrl+Space for general SQL completions"
echo "   - Use Tab/Shift-Tab to navigate completions"
echo "   - Press Enter or numbers 1-9 to accept completions"
echo ""
echo "Keybindings configured:"
echo "  <M-;> or <M-.>  - Trigger column completion"
echo "  <C-Space>       - Trigger general SQL completion"
echo "  <Tab>           - Navigate down in completion menu"
echo "  <S-Tab>         - Navigate up in completion menu"
echo "  <CR>            - Accept selected completion"
echo "  1-9             - Quick select numbered completion"
echo ""
echo "Plugin file location: nvim-plugin/lua/sql-cli/init.lua"
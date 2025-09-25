# SQL-CLI Template System Documentation

## Overview

The SQL-CLI template system provides powerful macro expansion and template management for SQL queries. It uses a clean `@{TYPE:args}` syntax and supports external template files for better organization.

## File Format

Templates can be stored in `.sql` files with special markers, allowing them to benefit from SQL syntax highlighting and tree-sitter parsing:

```sql
-- Template: FETCH_TRADES
-- Description: Fetch trades from API with filters
WITH WEB trades AS (
    URL '@{VAR:BASE_URL}/query/trades'
    METHOD POST
    BODY '{
        "select": "*",
        "where": "@{JPICKER:SOURCES:Select Source}"
    }'
    FORMAT JSON
    HEADERS (
        'Authorization': 'Bearer @{VAR:JWT_TOKEN}'
    )
)
SELECT * FROM trades;

-- Macro: SOURCES
-- Bloomberg
-- Reuters
-- Internal

-- Macro: CONFIG
-- BASE_URL = http://localhost:8080
-- JWT_TOKEN = @{ENV:JWT_TOKEN}
```

## Template File Locations

The system searches for templates in these locations (in order):
1. `~/.config/nvim/sql-templates/` - User templates
2. `~/.local/share/sql-cli/templates/` - Shared templates
3. `./templates/` - Project-specific templates
4. Current file - Inline macros using `-- MACRO:` comments

## Syntax Reference

### Variables and Environment

| Pattern | Description | Example Output |
|---------|-------------|----------------|
| `@{VAR:name}` | Config variable or environment | `http://api.example.com` |
| `@{ENV:name}` | Environment variable only | `test-token-123` |
| `@{JVAR:name}` | JSON-escaped variable | `\"value\"` |

### User Input

| Pattern | Description | Example Output |
|---------|-------------|----------------|
| `@{INPUT:prompt}` | Text input | `user_value` |
| `@{INPUT:prompt:default}` | Input with default | `default_value` |
| `@{JINPUT:prompt}` | JSON-escaped input | `\"user_value\"` |

### Selection Lists

| Pattern | Description | Example Output |
|---------|-------------|----------------|
| `@{PICKER:macro:prompt}` | Select from list | `Bloomberg` |
| `@{JPICKER:macro:prompt}` | JSON-escaped selection | `\"Bloomberg\"` |

### Special Functions

| Pattern | Description | Example Output |
|---------|-------------|----------------|
| `@{DATE:prompt}` | Date picker | `DateTime(2025, 9, 25)` |
| `@{MACRO:name}` | Expand another macro | `(expanded content)` |

## Usage Examples

### 1. Inline Expansion in SQL File

```sql
-- In your .sql file
-- MACRO: WHERE_ACTIVE
-- Status = 'Active' AND Amount > 0
-- END MACRO

SELECT * FROM trades
WHERE @{MACRO:WHERE_ACTIVE};
```

Press `<leader>ste` with cursor on `@{MACRO:WHERE_ACTIVE}` to expand.

### 2. Template from External File

Create `~/.config/nvim/sql-templates/trades.sql`:

```sql
-- Template: FETCH_BY_DATE
-- Description: Fetch trades for date range
WITH WEB trades AS (
    URL '@{VAR:BASE_URL}/query'
    BODY '{
        "where": "Date >= @{DATE:Start} AND Date <= @{DATE:End}"
    }'
    FORMAT JSON
)
SELECT * FROM trades;
```

Use `:SqlTemplateSelect` or `<leader>sts` to insert.

### 3. JSON Context with Proper Escaping

```sql
-- Template: API_QUERY
WITH WEB data AS (
    URL '@{VAR:BASE_URL}/api'
    BODY '{
        "source": "@{JPICKER:SOURCES:Source}",
        "filter": "@{JINPUT:Filter Expression}"
    }'
)
SELECT * FROM data;
```

The `J` prefix automatically adds `\"` escaping for JSON.

## Loading Templates

### Method 1: Auto-discovery
Templates are automatically discovered from configured directories:

```vim
" In init.vim or init.lua
lua require('sql-cli.template').setup({
    template_dirs = {
        "~/.config/nvim/sql-templates/",
        "./project-templates/"
    }
})
```

### Method 2: Direct file loading
Load a specific template file:

```lua
local templates = require('sql-cli.template')
templates.load_file("~/my-templates/custom.sql")
```

### Method 3: Inline in current file
Define macros directly in your SQL file:

```sql
-- MACRO: CONFIG
-- BASE_URL = http://prod.api.com
-- API_KEY = secret123
-- END MACRO

-- Your SQL using @{VAR:BASE_URL}
```

## Commands

| Command | Description |
|---------|-------------|
| `:SqlTemplateExpand` | Expand macro at cursor position |
| `:SqlTemplateSelect` | Browse and insert template |
| `:SqlTemplateReload` | Clear template cache |
| `:SqlTemplateList` | List available templates |

## Keybindings

Default keybindings under `\st` prefix (customizable):

| Key | Action | Description |
|-----|--------|-------------|
| `\ste` | **E**xpand | Expand template/macro at cursor position |
| `\sts` | **S**elect | Select template from file to insert |
| `\stl` | **L**ist | List all available templates |
| `\str` | **R**eload | Reload template cache |
| `\stm` | **M**acro | Quick macro expand with feedback |

All template operations are grouped under the `\st` prefix for consistency, similar to:
- `\sr` - SQL fragment/refactoring operations
- `\sc` - SQL CTE operations
- `\sw` - SQL WHERE clause operations

## Configuration

```lua
require('sql-cli.template').setup({
    -- Template file extensions (can be .sql for syntax highlighting)
    extensions = {".sql", ".sqlt"},

    -- Search directories
    template_dirs = {
        vim.fn.stdpath("config") .. "/sql-templates/",
        vim.fn.stdpath("data") .. "/sql-cli/templates/",
        "./templates/",
    },

    -- Maximum recursion depth for macro expansion
    max_depth = 10,

    -- Enable debug logging
    debug = false,

    -- Custom keybindings prefix
    keymap_prefix = "<leader>st"
})
```

## File Extension Notes

Templates can use `.sql` extension for better editor support:
- SQL syntax highlighting works normally
- Tree-sitter parsing works for SQL portions
- LSP features available for SQL
- Special comments (`-- Template:`, `-- Macro:`) are recognized by the plugin

The template system intelligently parses these markers while preserving standard SQL compatibility.

## Migration from Old Syntax

| Old Syntax | New Syntax | Notes |
|------------|------------|-------|
| `${VAR}` | `@{VAR:VAR}` | Use VAR type explicitly |
| `${VAR:default}` | `@{VAR:VAR}` | Set defaults in CONFIG macro |
| `"${VAR}"` | `"@{VAR:VAR}"` | Quotes preserved |
| `\"${VAR}\"` | `@{JVAR:VAR}` | Use JVAR for JSON escaping |
| `Input("prompt")` | `@{INPUT:prompt}` | Cleaner syntax |
| `Picker(MACRO)` | `@{PICKER:MACRO:prompt}` | Explicit prompt |
| `DateTimePicker()` | `@{DATE:prompt}` | Simplified |

## Best Practices

1. **Use `.sql` extension** for template files to get syntax highlighting
2. **Organize templates** by domain (trades.sql, reports.sql, etc.)
3. **Define CONFIG macro** at the top of template files for settings
4. **Use JVAR/JINPUT** for JSON contexts to ensure proper escaping
5. **Keep macros simple** - complex logic should be in the SQL itself
6. **Document templates** with Description comments
7. **Test templates** with `:SqlTemplateTest` command during development

## Troubleshooting

### Templates not found
- Check `:SqlTemplateList` to see discovered templates
- Verify template directories exist and contain `.sql` files
- Run `:SqlTemplateReload` to refresh cache

### Expansion not working
- Ensure cursor is on the macro pattern
- Check macro is defined (in file or template)
- Look for typos in macro names (case-sensitive)

### Infinite loop warning
- Check for circular macro references (A calls B, B calls A)
- Reduce complexity of nested macros
- Increase `max_depth` if needed (default: 10)

### JSON escaping issues
- Use `@{JVAR:}` and `@{JINPUT:}` for JSON contexts
- Regular `@{VAR:}` for non-JSON contexts
- Check if you're inside a BODY block
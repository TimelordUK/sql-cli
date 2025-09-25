# SQL-CLI Template System Setup Guide

## ✅ Installation Complete

The new template system is now fully integrated with the following keymaps registered under the `\st` prefix:

| Keymap | Command | Description |
|--------|---------|-------------|
| `\ste` | `:SqlTemplateExpand` | **E**xpand macro/template at cursor |
| `\sts` | `:SqlTemplateSelect` | **S**elect template from file |
| `\stl` | `:SqlTemplateList` | **L**ist all available templates |
| `\str` | `:SqlTemplateReload` | **R**eload template cache |
| `\stm` | Quick macro expand | **M**acro expand with feedback |

## Quick Start

### 1. Expand a Macro/Template at Cursor

```sql
-- In your SQL file, type:
@FETCH_TRADES_BASIC

-- Place cursor on it and press: \ste
-- It will expand to the full template
```

### 2. Select from Template Library

```vim
" Press \sts or run:
:SqlTemplateSelect

" This will:
" 1. Show available template files
" 2. List templates in selected file
" 3. Insert chosen template at cursor
```

### 3. List All Templates

```vim
" Press \stl or run:
:SqlTemplateList

" Shows floating window with all templates
```

## File Structure

```
sql-cli/
├── nvim-plugin/
│   └── lua/
│       └── sql-cli/
│           └── template/        # New modular template system
│               ├── init.lua     # Main module & keymaps
│               ├── engine.lua   # Core expansion engine
│               ├── parser.lua   # Pattern parsing
│               ├── loader.lua   # File loading
│               └── resolver.lua # Value resolution
└── templates/                   # Your template files
    ├── trades.sql              # Trade query templates
    ├── column_builder.sql      # Column macro templates
    └── *.sql                   # Any .sql with templates/macros
```

## Template Files

Templates use regular `.sql` files with special comments:

```sql
-- Template: QUERY_NAME
-- Description: What this query does
SELECT * FROM table
WHERE @{INPUT:Condition};

-- Macro: COLUMN_LIST
-- col1, col2, col3
-- END MACRO
```

## Configuration

In your Neovim config:

```lua
require('sql-cli').setup({
  -- Template system config
  template = {
    template_dirs = {
      vim.fn.stdpath("config") .. "/sql-templates/",
      "./templates/",
    },
    extensions = {".sql", ".sqlt"},
    keymap_prefix = "<leader>st",  -- Change prefix if needed
  }
})
```

## Testing the Setup

1. **Check keymaps are registered:**
   ```vim
   :map \st
   ```
   Should show all 5 template keymaps

2. **Check commands exist:**
   ```vim
   :command SqlTemplate
   ```
   Should show SqlTemplateExpand, SqlTemplateSelect, etc.

3. **Test expansion:**
   ```sql
   -- Create a test macro in your SQL file
   -- MACRO: TEST
   -- Hello World
   -- END MACRO

   -- Type: @TEST
   -- Press: \ste
   -- Should expand to: Hello World
   ```

## Troubleshooting

### Keymaps not working?

1. Check if registered:
   ```vim
   :verbose map \ste
   ```

2. Reload plugin:
   ```vim
   :lua require('sql-cli.template').create_keymaps()
   ```

3. Check for conflicts:
   ```vim
   :map \st
   ```

### Templates not found?

1. Check template directories:
   ```vim
   :lua print(vim.inspect(require('sql-cli.template').config.template_dirs))
   ```

2. Reload cache:
   ```vim
   :SqlTemplateReload
   " or press \str
   ```

3. Verify file has template markers:
   ```sql
   -- Must have: -- Template: NAME
   -- or:        -- MACRO: NAME
   ```

### Old system removal

The old `templates.lua` has been removed. The new system is at:
- `nvim-plugin/lua/sql-cli/template/` (directory with modular components)

All functionality has been preserved and improved.

## Next Steps

1. **Create your templates** in `~/.config/nvim/sql-templates/`
2. **Use column macros** for reusable column lists
3. **Build a library** of common query patterns
4. **Share templates** with your team

## Quick Reference Card

```
Templates & Macros:
  \ste - Expand at cursor      (@NAME → expanded content)
  \sts - Select from file      (Browse template library)
  \stl - List all              (Show all available)
  \str - Reload cache          (Refresh after edits)
  \stm - Quick macro           (Expand with feedback)

Syntax:
  @NAME                        (Expand template or macro)
  @{TEMPLATE:name}             (Explicit template)
  @{MACRO:name}                (Explicit macro)
  @{VAR:name}                  (Variable substitution)
  @{INPUT:prompt}              (User input)
  @{PICKER:list:prompt}        (Select from list)
  @{DATE:prompt}               (Date picker)

Escape Patterns:
  $$@{VAR:name}                (Preserve literal - won't expand)
  @{LITERAL:content}           (Keep content as-is)
```
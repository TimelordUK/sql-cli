# SQL-CLI Nvim Plugin Troubleshooting

## Formatter Issues

### Problem: SELECT keyword disappears when formatting

If the SELECT keyword or other parts of your query disappear when using `<leader>sf`, try these steps:

#### 1. Test the formatter directly

In Neovim, run:
```vim
:lua require('sql-cli').test_formatter()
```

This will test if the formatter is working at all.

#### 2. Enable debug mode

Add to your config:
```lua
require('sql-cli').setup({
  debug_format = true,  -- Enable debug messages
  command = "/full/path/to/sql-cli",  -- Use full path
})
```

#### 3. Check if sql-cli is in PATH

In Neovim, run:
```vim
:echo exepath('sql-cli')
```

If this returns empty, sql-cli is not in your PATH.

#### 4. Use full path to sql-cli

Find where sql-cli is installed:
```bash
which sql-cli
# or if built from source:
# /home/username/sql-cli/target/release/sql-cli
```

Then configure the full path:
```lua
require('sql-cli').setup({
  command = "/home/username/sql-cli/target/release/sql-cli",
})
```

#### 5. Test the CLI formatter directly

From terminal:
```bash
echo "SELECT * FROM table" | sql-cli --format
```

If this works but Neovim doesn't, it's a PATH issue.

#### 6. Check for error messages

The plugin now shows warnings when:
- sql-cli is not found (falls back to regex formatter)
- Formatter returns empty result
- Formatter returns an error

Look for these messages after pressing `<leader>sf`.

### Problem: Formatter uses wrong style

Configure formatting preferences:
```lua
require('sql-cli').setup({
  format = {
    lowercase = true,   -- Use lowercase keywords
    compact = false,    -- Compact or expanded format
    tabs = false,      -- Use tabs instead of spaces
  },
})
```

### Problem: Fallback formatter is used instead of AST formatter

If you see the message "sql-cli not found, using fallback formatter":

1. Install sql-cli in your PATH:
   ```bash
   cargo install --path /path/to/sql-cli
   ```

2. Or use the full path in config (see step 4 above)

## Other Common Issues

### Data file not detected

1. Ensure your CSV/JSON file exists
2. Use `:SqlSetData` command to manually set the file
3. Check that file permissions allow reading

### Query execution fails

1. Check that data file is set: `:SqlShowData`
2. Verify SQL syntax is correct
3. Look for error messages in the output window

### Keybindings don't work

1. Check for conflicts with `:verbose map <leader>sf`
2. Ensure the plugin is loaded: `:lua print(require('sql-cli').version)`
3. Try using commands directly: `:SqlFormat`

## Getting Help

1. Enable debug mode for more information
2. Check `:messages` for any error output
3. Report issues at: https://github.com/TimelordUK/sql-cli/issues

Include:
- Your Neovim version (`:version`)
- Your sql-cli version (`sql-cli --version`)
- Your config
- The exact error message or behavior
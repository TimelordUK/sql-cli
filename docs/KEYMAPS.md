# SQL-CLI Keymap Reference

All SQL-CLI keymaps are organized under logical prefixes to avoid conflicts and make them easy to remember.

## Template Operations - `\st`

Templates and macro expansion operations:

| Keymap | Action | Description |
|--------|--------|-------------|
| `\ste` | **E**xpand | Expand template/macro at cursor position |
| `\sts` | **S**elect | Select template from file to insert |
| `\stl` | **L**ist | List all available templates |
| `\str` | **R**eload | Reload template cache |
| `\stm` | **M**acro | Quick macro expand with visual feedback |

## SQL Refactoring - `\sr`

SQL fragment and refactoring operations:

| Keymap | Action | Description |
|--------|--------|-------------|
| `\sra` | **A**dd fragment | Add selected SQL as reusable fragment |
| `\sri` | **I**nsert fragment | Insert saved fragment at cursor |
| `\srl` | **L**ist fragments | List all saved fragments |
| `\srd` | **D**elete fragment | Delete a saved fragment |
| `\sre` | **E**dit fragment | Edit an existing fragment |

## CTE Operations - `\sc`

Common Table Expression (WITH clause) operations:

| Keymap | Action | Description |
|--------|--------|-------------|
| `\sce` | **E**xtract to CTE | Extract selection to CTE |
| `\scr` | **R**ename CTE | Rename CTE under cursor |
| `\scc` | **C**ollapse CTEs | Collapse/inline CTEs |
| `\scn` | **N**avigate | Navigate between CTEs |
| `\sct` | **T**est CTE | Test CTE in isolation |

## WHERE Clause Building - `\sw`

WHERE clause construction helpers:

| Keymap | Action | Description |
|--------|--------|-------------|
| `\swb` | **B**uild | Build WHERE clause interactively |
| `\swa` | **A**dd condition | Add AND condition |
| `\swo` | Add **O**R condition | Add OR condition |
| `\swg` | **G**roup | Group selected conditions |
| `\swn` | **N**egate | Negate condition (NOT) |

## Query Execution - `\sq`

Query execution and testing:

| Keymap | Action | Description |
|--------|--------|-------------|
| `\sqr` | **R**un query | Execute current query |
| `\sqe` | **E**xplain | Show query execution plan |
| `\sqp` | **P**review | Preview query results |
| `\sqt` | **T**ime | Time query execution |
| `\sql` | **L**imit | Add/modify LIMIT clause |

## Data Export

Export and output operations (various prefixes):

| Keymap | Action | Description |
|--------|--------|-------------|
| `\se` | **E**xport menu | Show export format menu |
| `\sm` | Export **M**arkdown | Export as markdown table |
| `\sT` | Export **T**SV | Export as Tab-separated (Excel) |

Note: `\sT` uses capital T to avoid conflict with `\st` (templates)

## Navigation - `\sn`

SQL navigation helpers:

| Keymap | Action | Description |
|--------|--------|-------------|
| `\snc` | Next **C**lause | Jump to next SQL clause |
| `\snp` | **P**revious clause | Jump to previous clause |
| `\snt` | Next **T**able | Jump to next table reference |
| `\snf` | Next **F**unction | Jump to next function call |
| `\sns` | Next **S**ubquery | Jump to next subquery |

## Formatting - `\sf`

SQL formatting operations:

| Keymap | Action | Description |
|--------|--------|-------------|
| `\sff` | **F**ormat | Format entire query |
| `\sfa` | **A**lign | Align columns/keywords |
| `\sfi` | **I**ndent | Fix indentation |
| `\sfc` | **C**ompress | Compress to single line |
| `\sfe` | **E**xpand | Expand to multiple lines |

## Visual Mode Mappings

Additional mappings available in visual mode:

| Keymap | Mode | Action |
|--------|------|--------|
| `\ste` | Visual | Expand selected macro |
| `\sra` | Visual | Add selection as fragment |
| `\sce` | Visual | Extract selection to CTE |
| `\swg` | Visual | Group selected conditions |

## Customization

To customize the prefix in your vim config:

```vim
" Change template prefix
let g:sql_cli_template_prefix = '<leader>t'

" Change refactoring prefix
let g:sql_cli_refactor_prefix = '<leader>r'
```

Or in Lua:

```lua
require('sql-cli.template').setup({
    keymap_prefix = '<leader>t'
})

require('sql-cli.refactor').setup({
    keymap_prefix = '<leader>r'
})
```

## Quick Reference Card

Most common operations:

| Operation | Keymap | What it does |
|-----------|--------|--------------|
| Expand macro | `\ste` | Expands @{MACRO:name} at cursor |
| Insert template | `\sts` | Browse and insert from template file |
| Add fragment | `\sra` | Save selected SQL as reusable fragment |
| Extract CTE | `\sce` | Extract selected SQL to WITH clause |
| Run query | `\sqr` | Execute current SQL query |
| Format SQL | `\sff` | Format and beautify SQL |

## Conflict Resolution

If you have conflicts with existing mappings:

1. Check existing mappings: `:verbose map \st`
2. Unmap conflicting keys: `:unmap \st`
3. Or use different prefix in setup

## Mnemonic Guide

The prefixes are designed to be memorable:

- `\st` - SQL **T**emplates
- `\sr` - SQL **R**efactoring
- `\sc` - SQL **C**TEs
- `\sw` - SQL **W**HERE clauses
- `\sq` - SQL **Q**ueries
- `\se` - SQL **E**xport
- `\sn` - SQL **N**avigation
- `\sf` - SQL **F**ormatting

Within each group, the suffix usually matches the action:
- `e` - **E**xpand/Execute/Export
- `s` - **S**elect/Search
- `l` - **L**ist
- `r` - **R**eload/Run/Rename
- `a` - **A**dd
- `d` - **D**elete
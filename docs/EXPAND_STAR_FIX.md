# Expand Star Fix: Data File Support

**Date**: 2025-10-04
**Issue**: `\sE` (expand SELECT *) didn't work with file hints
**Status**: ✅ Fixed

## Problem

When you had a SQL file with a data file hint:

```sql
-- #! ../data/periodic_table.csv

SELECT *
FROM periodic_table;
```

Running `\sE` would fail with "table not found" because the preview query wasn't passed the data file.

## Root Cause

The `expand_star_smart()` function was executing:

```bash
sql-cli -q "SELECT * FROM periodic_table LIMIT 1" -o json
```

Without the data file, the CLI didn't know where to find `periodic_table`.

## Fix

Now `expand_star_smart()` checks for a data file in state and includes it:

```bash
sql-cli ../data/periodic_table.csv -q "SELECT * FROM periodic_table LIMIT 1" -o json
```

### Code Changes

**File**: `nvim-plugin/lua/sql-cli/results.lua`

**Before**:
```lua
Job:new({
  command = command_path,
  args = {
    '-q', preview_query,
    '-o', 'json'
  },
```

**After**:
```lua
-- Build args with data file if set
local args = {}

-- Add data file first (positional argument)
local data_file = state:get_data_file()
if data_file then
  log.info('expand_star', 'Using data file: ' .. data_file)
  table.insert(args, data_file)
else
  log.warn('expand_star', 'No data file set - query may fail for table references')
end

-- Add query and output format
table.insert(args, '-q')
table.insert(args, preview_query)
table.insert(args, '-o')
table.insert(args, 'json')

Job:new({
  command = command_path,
  args = args,
```

## How It Works Now

### 1. File with Data Hint

```sql
-- #! ../data/periodic_table.csv

SELECT *
FROM periodic_table;
```

1. Open file → hint detected → `state:set_data_file("../data/periodic_table.csv")`
2. Run `\sE` → passes data file to CLI
3. CLI returns columns from periodic_table.csv
4. `SELECT *` expanded successfully ✅

### 2. CSV File Directly

```bash
nvim data/trades.csv
```

1. Open CSV → auto-detected → `state:set_data_file("data/trades.csv")`
2. Run `\sE` on query → passes data file to CLI
3. Works ✅

### 3. Manually Set Data File

```vim
:SqlCliSetDataFile path/to/data.csv
```

Then `\sE` will use that file.

### 4. WEB CTE (No Data File Needed)

```sql
WITH WEB trades AS (
  URL 'http://api.example.com/trades'
  METHOD GET
)
SELECT * FROM trades;
```

1. No data file needed (self-contained)
2. Run `\sE` → executes WEB CTE
3. Works ✅

## Logging

Now logs show whether data file is being used:

**With data file**:
```
[15:30:00] INFO [expand_star] === expand_star_smart called ===
[15:30:00] INFO [expand_star] Got query (length: 45)
[15:30:00] INFO [expand_star] Using data file: ../data/periodic_table.csv
[15:30:00] DEBUG [expand_star] CLI args: {"../data/periodic_table.csv", "-q", "SELECT * FROM periodic_table LIMIT 1", "-o", "json"}
[15:30:00] INFO [expand_star] Executing query to get schema...
[15:30:00] INFO [expand_star] Query executed successfully (exit code 0)
[15:30:00] INFO [expand_star] Extracted 28 columns: AtomicNumber, Element, Symbol, ...
```

**Without data file** (will likely fail):
```
[15:30:00] WARN [expand_star] No data file set - query may fail for table references
[15:30:00] DEBUG [expand_star] CLI args: {"-q", "SELECT * FROM periodic_table LIMIT 1", "-o", "json"}
[15:30:00] ERROR [expand_star] Query execution failed with exit code 1
[15:30:00] ERROR [expand_star] Error message: Table 'periodic_table' not found
```

## Testing

Test with the chemistry example:

```bash
cd sql-cli
nvim examples/chemistry.sql
```

Put cursor on line 3-6 (the simple `SELECT * FROM periodic_table`), press `\sE`:

**Expected result**:
```sql
SELECT
    AtomicNumber
  , Element
  , Symbol
  , AtomicMass
  , NumberofNeutrons
  , NumberofProtons
  , NumberofElectrons
  , Period
  , Group
  , Phase
  , Radioactive
  , Natural
  , Metal
  , Nonmetal
  , Metalloid
  , Type
  , AtomicRadius
  , Electronegativity
  , FirstIonization
  , Density
  , MeltingPoint
  , BoilingPoint
  , NumberOfIsotopes
  , Discoverer
  , Year
  , SpecificHeat
  , NumberofShells
  , NumberofValence
FROM periodic_table;
```

## Benefits

1. **Works with file hints** - Most example files use `-- #! data.csv`
2. **Works with CSV buffers** - Open CSV directly
3. **Works with manual data files** - `:SqlCliSetDataFile`
4. **Works with WEB CTEs** - No file needed
5. **Clear logging** - Know exactly what's being passed to CLI
6. **Consistent with executor** - Same behavior as `\sx` (execute query)

## Related

- [DEBUGGING_WITH_LOGS.md](DEBUGGING_WITH_LOGS.md) - How to debug expand star issues
- [NVIM_SMART_COLUMN_COMPLETION.md](../nvim-plugin/SMART_EXPANSION_README.md) - Smart expansion documentation

# Fuzzy Filter: Exact Match Mode

**Date**: 2025-10-04
**Status**: ✅ Implemented
**Priority**: P0 (Roadmap Item #1)

## Summary

Added **exact substring match mode** to the nvim plugin's fuzzy filter to address the issue of overly permissive search results. Users can now toggle between **EXACT** (strict substring) and **FUZZY** (character sequence) matching modes.

## Problem

The original fuzzy filter was too permissive:
- Pattern `"ape"` would match `"apple"` (a-p-e in sequence)
- Pattern `"app"` would match any text with a, p, p in that order
- This caused unwanted rows to be included in filter results

## Solution

### 1. Exact Match Function

Added `exact_match()` function that performs strict substring matching:

```lua
-- Exact substring match (stricter, like fzf --exact)
local function exact_match(pattern, text)
  pattern = pattern:lower()
  text = text:lower()

  -- Check if pattern exists as exact substring
  local match_pos = text:find(pattern, 1, true)  -- plain text search
  if not match_pos then
    return nil
  end

  -- Score based on position (earlier = better)
  local score = 100 - match_pos

  -- Bonus for exact match
  if text == pattern then
    score = score + 100
  end

  -- Bonus for match at start
  if match_pos == 1 then
    score = score + 50
  end

  return score
end
```

### 2. Configuration

```lua
M.config = {
  match_mode = 'exact'  -- 'exact' or 'fuzzy'
}
```

Default is **'exact'** for stricter filtering.

### 3. Dynamic Switching

Users can toggle between modes with **Ctrl+t** while filtering:

```lua
-- Toggle match mode (exact vs fuzzy)
vim.keymap.set('n', '<C-t>', function()
  M.config.match_mode = M.config.match_mode == 'exact' and 'fuzzy' or 'exact'
  -- Re-apply filter with new mode
end)
```

### 4. Visual Indicator

Window title shows current mode:
- `[EXACT] Filter 100 rows (ESC: close, C-l: clear, C-t: toggle)`
- `[FUZZY] Showing 5/100 rows`

## Usage

### Opening Fuzzy Filter

In a result buffer with table data, open the fuzzy filter (check your keybinding, typically `\sf` or similar).

### Filtering in Exact Mode (Default)

Type a pattern to match as an exact substring:
- `app` → matches "apple", "application", "pineapple"
- `ape` → matches "grape", "grapefruit" (NOT "apple")

### Switching to Fuzzy Mode

Press `Ctrl+t` to toggle to fuzzy mode:
- `ape` → now matches "apple" (a-p-e in sequence)

### Clearing Filter

Press `Ctrl+l` to clear the current pattern.

### Applying Filter

Press `Enter` to keep the filtered results and close the filter window.

### Canceling Filter

Press `Esc` to restore original table and close the filter window.

## Testing

Test script: `nvim-plugin/test_exact_match.lua`

```bash
lua nvim-plugin/test_exact_match.lua
```

All 8 test cases pass, demonstrating:
- Exact mode requires contiguous substring match
- Fuzzy mode matches characters in sequence (gaps allowed)
- Key case: `"ape"` in `"apple"` → exact: NO, fuzzy: YES

## Behavior Comparison

| Pattern | Text         | Exact Match | Fuzzy Match |
|---------|--------------|-------------|-------------|
| "app"   | "apple"      | ✓           | ✓           |
| "app"   | "pineapple"  | ✓           | ✓           |
| "app"   | "grape"      | ✗           | ✗           |
| "ape"   | "grape"      | ✓           | ✓           |
| "ape"   | "apple"      | ✗           | ✓           |

The last row shows the key difference: exact mode prevents unwanted matches.

## Files Modified

- `nvim-plugin/lua/sql-cli/fuzzy_filter.lua`
  - Added `exact_match()` function
  - Added `M.config.match_mode`
  - Modified `apply_filter()` to use configurable matcher
  - Updated UI to show mode indicator
  - Added `Ctrl+t` keybinding for toggling

## Configuration in User Setup

Users can change the default mode in their config:

```lua
require('sql-cli').setup({
  -- If this module is exposed in config
  fuzzy_filter = {
    match_mode = 'fuzzy'  -- or 'exact' (default)
  }
})
```

*(Note: May need to wire this into main config if not already done)*

## Related Documentation

- **ROADMAP_2025.md** - P0-1: Fix Fuzzy Search Strictness
- **PRIORITIZED_TASKS.md** - This week's top priority

## Future Enhancements

- [ ] Make default mode configurable via plugin setup()
- [ ] Add case-sensitive exact match option
- [ ] Regex mode for power users
- [ ] Remember last used mode per session

## Credits

Implements the "stricter fzf mode" requirement from the roadmap, providing behavior similar to fzf's `--exact` flag for absolute matches.

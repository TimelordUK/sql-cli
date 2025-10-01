-- SQL CLI Configuration for Large Datasets
-- Use this config when working with large datasets where wide columns
-- may appear later in the data (e.g., FIX file selectors with 3k+ rows)

require('sql-cli').setup({
  command = "sql-cli",  -- Or full path like "/home/user/sql-cli/target/release/sql-cli"

  -- Split configuration
  split = {
    direction = "vertical",
    size = 0.5,
  },

  -- Output format
  output_format = "table",

  -- Table output settings optimized for large datasets
  table_output = {
    -- Set to 0 for unlimited column width (no truncation)
    -- Or set to a higher value like 150 if you want some limit
    max_col_width = 0,

    -- Set to 0 to scan ALL rows for column width calculation
    -- This ensures wide values appearing late in the dataset are not truncated
    -- Note: This may be slower for very large datasets (10k+ rows)
    col_sample_rows = 0,
  },

  -- Auto-detect features
  auto_detect = {
    csv_files = true,
    data_hints = true,
  },

  -- Output settings
  output = {
    focus_on_run = false,
    clear_on_run = true,
    wrap = false,
    number = false,
  },
})

-- Example use cases:
--
-- 1. FIX file selector with 3k rows:
--    With col_sample_rows = 0, it will scan all 3k rows to find the widest
--    tag values, preventing truncation with "..." in later rows.
--
-- 2. WEB CTE responses with variable-length fields:
--    Unlimited column width ensures full data visibility without truncation.
--
-- 3. Alternative balanced approach:
--    If scanning all rows is too slow, try:
--      max_col_width = 150      -- Reasonable limit
--      col_sample_rows = 1000   -- Sample first 1000 rows
--
-- Performance note:
--   - col_sample_rows = 0 (all rows) is slower but accurate
--   - col_sample_rows = 100 (default) is fast but may truncate late data
--   - Choose based on your dataset size and width distribution

-- SQL CLI Window Functions Helper Module
-- Provides wizards and macros for common window function patterns

local utils = require('sql-cli.utils')
local M = {}

-- Helper to get word under cursor
local function get_word_under_cursor()
  local word = vim.fn.expand('<cword>')
  return word ~= '' and word or nil
end

-- Helper to insert text at cursor position
local function insert_at_cursor(text)
  local row, col = unpack(vim.api.nvim_win_get_cursor(0))
  local line = vim.api.nvim_get_current_line()

  -- Check if text contains newlines
  if text:find("\n") then
    -- Split text into lines
    local lines = {}
    for line_text in text:gmatch("[^\n]*") do
      table.insert(lines, line_text)
    end

    -- First line: append to current line
    lines[1] = line:sub(1, col) .. lines[1]

    -- Last line: prepend rest of original line
    lines[#lines] = lines[#lines] .. line:sub(col + 1)

    -- Replace current line and insert new ones
    vim.api.nvim_buf_set_lines(0, row - 1, row, false, lines)

    -- Position cursor at end of inserted content
    vim.api.nvim_win_set_cursor(0, {row + #lines - 1, #lines[#lines] - #line:sub(col + 1)})
  else
    -- Simple single-line insertion
    local new_line = line:sub(1, col) .. text .. line:sub(col + 1)
    vim.api.nvim_set_current_line(new_line)

    -- Move cursor to end of inserted text
    vim.api.nvim_win_set_cursor(0, {row, col + #text})
  end
end

-- Generate ROW_NUMBER() ranking over a column
function M.row_number_partition()
  local column = get_word_under_cursor()

  vim.ui.input({
    prompt = "Partition by column (Enter for none): ",
    default = column or ""
  }, function(partition_col)
    if partition_col == nil then return end

    vim.ui.input({
      prompt = "Order by column: ",
      default = column or ""
    }, function(order_col)
      if order_col == nil then return end

      vim.ui.select(
        {"DESC", "ASC"},
        { prompt = "Sort order: " },
        function(order_dir)
          if order_dir == nil then return end

          vim.ui.input({
            prompt = "Alias for rank column: ",
            default = partition_col ~= "" and (partition_col .. "_rank") or "rank"
          }, function(alias)
            if alias == nil then return end

            -- Build the window function
            local window_func
            if partition_col ~= "" then
              window_func = string.format(
                "ROW_NUMBER() OVER (PARTITION BY %s ORDER BY %s %s) as %s",
                partition_col, order_col, order_dir, alias
              )
            else
              window_func = string.format(
                "ROW_NUMBER() OVER (ORDER BY %s %s) as %s",
                order_col, order_dir, alias
              )
            end

            -- Insert with proper formatting
            insert_at_cursor(",\n    " .. window_func)
            vim.notify("ROW_NUMBER() window function added", vim.log.levels.INFO)
          end)
        end)
    end)
  end)
end

-- Generate LAG/LEAD for delta calculations
function M.lag_lead_delta()
  local column = get_word_under_cursor()

  vim.ui.select(
    {"LAG (previous value)", "LEAD (next value)", "BOTH (prev and next)"},
    { prompt = "Select function type: " },
    function(choice)
      if choice == nil then return end

      local func_type = choice:match("^(%w+)")

      vim.ui.input({
        prompt = "Column to analyze: ",
        default = column or ""
      }, function(target_col)
        if target_col == nil then return end

        vim.ui.input({
          prompt = "Partition by (Enter for none): ",
          default = ""
        }, function(partition_col)
          if partition_col == nil then return end

          vim.ui.input({
            prompt = "Order by column: ",
            default = partition_col ~= "" and partition_col or target_col
          }, function(order_col)
            if order_col == nil then return end

            vim.ui.input({
              prompt = "Offset (default 1): ",
              default = "1"
            }, function(offset)
              if offset == nil then return end
              offset = offset == "" and "1" or offset

              -- Build the window functions
              local window_clause
              if partition_col ~= "" then
                window_clause = string.format("PARTITION BY %s ORDER BY %s", partition_col, order_col)
              else
                window_clause = string.format("ORDER BY %s", order_col)
              end

              local window_funcs = {}

              if func_type == "LAG" or func_type == "BOTH" then
                table.insert(window_funcs, string.format(
                  "LAG(%s, %s) OVER (%s) as prev_%s",
                  target_col, offset, window_clause, target_col
                ))
                -- Add delta calculation
                table.insert(window_funcs, string.format(
                  "%s - LAG(%s, %s) OVER (%s) as %s_change",
                  target_col, target_col, offset, window_clause, target_col
                ))
              end

              if func_type == "LEAD" or func_type == "BOTH" then
                table.insert(window_funcs, string.format(
                  "LEAD(%s, %s) OVER (%s) as next_%s",
                  target_col, offset, window_clause, target_col
                ))
                if func_type == "LEAD" then
                  -- Add delta calculation for LEAD
                  table.insert(window_funcs, string.format(
                    "LEAD(%s, %s) OVER (%s) - %s as %s_upcoming_change",
                    target_col, offset, window_clause, target_col, target_col
                  ))
                end
              end

              -- Insert with proper formatting
              local text = ",\n    " .. table.concat(window_funcs, ",\n    ")
              insert_at_cursor(text)
              vim.notify(func_type .. " window functions added", vim.log.levels.INFO)
            end)
          end)
        end)
      end)
    end)
end

-- Generate windowed aggregate functions
function M.windowed_aggregate()
  local column = get_word_under_cursor()

  vim.ui.select(
    {"SUM", "AVG", "COUNT", "MIN", "MAX", "STDDEV", "VARIANCE"},
    { prompt = "Select aggregate function: " },
    function(agg_func)
      if agg_func == nil then return end

      vim.ui.input({
        prompt = "Column to aggregate: ",
        default = column or ""
      }, function(target_col)
        if target_col == nil then return end

        vim.ui.input({
          prompt = "Partition by (Enter for cumulative): ",
          default = ""
        }, function(partition_col)
          if partition_col == nil then return end

          -- Helper function to generate and insert aggregate
          local function generate_aggregate_function(window_clause, alias_suffix)
            -- Special handling for COUNT
            local agg_expr
            if agg_func == "COUNT" then
              agg_expr = target_col == "*" and "COUNT(*)" or string.format("COUNT(%s)", target_col)
            else
              agg_expr = string.format("%s(%s)", agg_func, target_col)
            end

            local window_func = string.format(
              "%s OVER (%s) as %s",
              agg_expr, window_clause, alias_suffix
            )

            -- Insert with proper formatting
            insert_at_cursor(",\n    " .. window_func)
            vim.notify(agg_func .. " window function added", vim.log.levels.INFO)
          end

          vim.ui.select(
            {"Running/Cumulative", "Partition Total", "Moving Window"},
            { prompt = "Window type: " },
            function(window_type)
              if window_type == nil then return end

              local window_clause = ""
              local alias_suffix = ""

              if window_type == "Running/Cumulative" then
                -- Cumulative window (unbounded preceding to current row)
                if partition_col ~= "" then
                  vim.ui.input({
                    prompt = "Order by column: ",
                    default = column or ""
                  }, function(order_col)
                    if order_col == nil then return end
                    window_clause = string.format(
                      "PARTITION BY %s ORDER BY %s ROWS UNBOUNDED PRECEDING",
                      partition_col, order_col
                    )
                    alias_suffix = "running_" .. agg_func:lower()
                    generate_aggregate_function(window_clause, alias_suffix)
                  end)
                else
                  vim.ui.input({
                    prompt = "Order by column: ",
                    default = column or ""
                  }, function(order_col)
                    if order_col == nil then return end
                    window_clause = string.format(
                      "ORDER BY %s ROWS UNBOUNDED PRECEDING",
                      order_col
                    )
                    alias_suffix = "cumulative_" .. agg_func:lower()
                    generate_aggregate_function(window_clause, alias_suffix)
                  end)
                end

              elseif window_type == "Partition Total" then
                -- Total for entire partition
                if partition_col == "" then
                  vim.notify("Partition column required for partition totals", vim.log.levels.WARN)
                  return
                end
                window_clause = string.format("PARTITION BY %s", partition_col)
                alias_suffix = partition_col .. "_" .. agg_func:lower()
                generate_aggregate_function(window_clause, alias_suffix)

              elseif window_type == "Moving Window" then
                -- Moving window (e.g., last 3 rows)
                vim.ui.input({
                  prompt = "Window size (rows before and after, e.g., '2' or '2,1'): ",
                  default = "2"
                }, function(window_size)
                  if window_size == nil then return end

                  local preceding, following
                  if window_size:match(",") then
                    preceding, following = window_size:match("(%d+)%s*,%s*(%d+)")
                  else
                    preceding = window_size
                    following = window_size
                  end

                  vim.ui.input({
                    prompt = "Order by column: ",
                    default = column or ""
                  }, function(order_col)
                    if order_col == nil then return end

                    if partition_col ~= "" then
                      window_clause = string.format(
                        "PARTITION BY %s ORDER BY %s ROWS BETWEEN %s PRECEDING AND %s FOLLOWING",
                        partition_col, order_col, preceding, following
                      )
                    else
                      window_clause = string.format(
                        "ORDER BY %s ROWS BETWEEN %s PRECEDING AND %s FOLLOWING",
                        order_col, preceding, following
                      )
                    end
                    alias_suffix = "moving_" .. agg_func:lower()
                    generate_aggregate_function(window_clause, alias_suffix)
                  end)
                end)
              end
            end)
        end)
      end)
    end)
end

-- Generate RANK/DENSE_RANK functions
function M.rank_functions()
  local column = get_word_under_cursor()

  vim.ui.select(
    {"RANK", "DENSE_RANK", "PERCENT_RANK", "NTILE"},
    { prompt = "Select ranking function: " },
    function(rank_func)
      if rank_func == nil then return end

      local buckets = nil

      -- Define continue_with_rank function before using it
      local function continue_with_rank()
        vim.ui.input({
          prompt = "Partition by (Enter for global ranking): ",
          default = ""
        }, function(partition_col)
          if partition_col == nil then return end

          vim.ui.input({
            prompt = "Order by column: ",
            default = column or ""
          }, function(order_col)
            if order_col == nil then return end

            vim.ui.select(
              {"DESC", "ASC"},
              { prompt = "Sort order: " },
              function(order_dir)
                if order_dir == nil then return end

                -- Build the window function
                local window_func
                local window_clause

                if partition_col ~= "" then
                  window_clause = string.format("PARTITION BY %s ORDER BY %s %s",
                    partition_col, order_col, order_dir)
                else
                  window_clause = string.format("ORDER BY %s %s", order_col, order_dir)
                end

                local alias = rank_func:lower()
                if partition_col ~= "" then
                  alias = partition_col .. "_" .. alias
                end

                if rank_func == "NTILE" then
                  window_func = string.format("%s(%s) OVER (%s) as %s",
                    rank_func, buckets, window_clause, alias)
                else
                  window_func = string.format("%s() OVER (%s) as %s",
                    rank_func, window_clause, alias)
                end

                -- Insert with proper formatting
                insert_at_cursor(",\n    " .. window_func)
                vim.notify(rank_func .. " window function added", vim.log.levels.INFO)
              end)
          end)
        end)
      end

      if rank_func == "NTILE" then
        vim.ui.input({
          prompt = "Number of buckets: ",
          default = "4"
        }, function(input_buckets)
          if input_buckets == nil then return end
          buckets = input_buckets
          continue_with_rank()
        end)
      else
        continue_with_rank()
      end
    end)
end

-- Quick window function menu
function M.window_function_wizard()
  vim.ui.select({
    "1. ROW_NUMBER() - Add ranking",
    "2. LAG/LEAD - Calculate deltas",
    "3. Windowed Aggregates - Running totals, averages",
    "4. RANK Functions - RANK, DENSE_RANK, PERCENT_RANK",
    "5. FIRST/LAST VALUE - Get boundary values",
  }, {
    prompt = "Select window function pattern: "
  }, function(choice)
    if choice == nil then return end

    local option = choice:match("^(%d)")
    if option == "1" then
      M.row_number_partition()
    elseif option == "2" then
      M.lag_lead_delta()
    elseif option == "3" then
      M.windowed_aggregate()
    elseif option == "4" then
      M.rank_functions()
    elseif option == "5" then
      M.first_last_value()
    end
  end)
end

-- Generate FIRST_VALUE/LAST_VALUE
function M.first_last_value()
  local column = get_word_under_cursor()

  vim.ui.select(
    {"FIRST_VALUE", "LAST_VALUE", "BOTH"},
    { prompt = "Select value function: " },
    function(func_choice)
      if func_choice == nil then return end

      vim.ui.input({
        prompt = "Column to get value from: ",
        default = column or ""
      }, function(value_col)
        if value_col == nil then return end

        vim.ui.input({
          prompt = "Partition by (Enter for none): ",
          default = ""
        }, function(partition_col)
          if partition_col == nil then return end

          vim.ui.input({
            prompt = "Order by column: ",
            default = column or ""
          }, function(order_col)
            if order_col == nil then return end

            local window_funcs = {}
            local window_clause

            if partition_col ~= "" then
              window_clause = string.format("PARTITION BY %s ORDER BY %s", partition_col, order_col)
            else
              window_clause = string.format("ORDER BY %s", order_col)
            end

            if func_choice == "FIRST_VALUE" or func_choice == "BOTH" then
              table.insert(window_funcs, string.format(
                "FIRST_VALUE(%s) OVER (%s) as first_%s",
                value_col, window_clause, value_col
              ))
            end

            if func_choice == "LAST_VALUE" or func_choice == "BOTH" then
              -- For LAST_VALUE, we typically want the entire partition
              local last_clause = window_clause .. " ROWS BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING"
              table.insert(window_funcs, string.format(
                "LAST_VALUE(%s) OVER (%s) as last_%s",
                value_col, last_clause, value_col
              ))
            end

            -- Insert with proper formatting
            local text = ",\n    " .. table.concat(window_funcs, ",\n    ")
            insert_at_cursor(text)
            vim.notify("Value window functions added", vim.log.levels.INFO)
          end)
        end)
      end)
    end)
end

-- Setup keymaps
function M.setup_keymaps(config)
  local keymaps = config.keymaps or {}

  -- Window function keymaps - all under \srw prefix for organization
  vim.keymap.set("n", "<leader>srw", M.window_function_wizard,
    { desc = "Refactor: Window function wizard", silent = true })
  vim.keymap.set("n", "<leader>srwr", M.row_number_partition,
    { desc = "Refactor: Add ROW_NUMBER() ranking", silent = true })
  vim.keymap.set("n", "<leader>srwl", M.lag_lead_delta,
    { desc = "Refactor: Add LAG/LEAD delta", silent = true })
  vim.keymap.set("n", "<leader>srwa", M.windowed_aggregate,
    { desc = "Refactor: Add windowed aggregate", silent = true })
  vim.keymap.set("n", "<leader>srwk", M.rank_functions,
    { desc = "Refactor: Add RANK functions", silent = true })
  vim.keymap.set("n", "<leader>srwv", M.first_last_value,
    { desc = "Refactor: Add FIRST/LAST VALUE", silent = true })
end

return M
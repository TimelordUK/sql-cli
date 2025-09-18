-- SQL CLI Function Help System Module
-- Functions for providing help and information about SQL functions and schema

local utils = require('sql-cli.utils')

local M = {}

-- Show function or generator help for word under cursor
function M.function_help_at_cursor(config)
  -- Get word under cursor
  local word = vim.fn.expand("<cword>"):upper()

  if word == "" then
    vim.notify("No word under cursor", vim.log.levels.WARN)
    return
  end

  -- Build command to get function help
  local command_path, err = utils.get_command_path(config.command)
  if not command_path then
    vim.notify(err, vim.log.levels.ERROR)
    return
  end

  -- First try as a function
  local cmd = command_path .. " --function-help " .. vim.fn.shellescape(word)

  -- Execute command
  local result = vim.fn.system(cmd)
  local exit_code = vim.v.shell_error

  if exit_code ~= 0 then
    -- Try as a generator
    local gen_cmd = command_path .. " --generator-help " .. vim.fn.shellescape(word)
    local gen_result = vim.fn.system(gen_cmd)
    local gen_exit_code = vim.v.shell_error

    if gen_exit_code == 0 then
      -- It's a generator, show generator help
      M.show_help_in_float(word .. " (Generator)", gen_result)
      return
    end

    -- Neither function nor generator found, try searching
    local search_cmd = command_path .. " --list-functions"
    local all_functions = vim.fn.system(search_cmd)

    -- Also search in generators
    local gen_list_cmd = command_path .. " --list-generators"
    local all_generators = vim.fn.system(gen_list_cmd)

    -- Check if any function or generator contains this word
    local matches = {}
    local gen_matches = {}

    for line in all_functions:gmatch("[^\n]+") do
      if line:upper():find(word) then
        table.insert(matches, line)
      end
    end

    for line in all_generators:gmatch("[^\n]+") do
      if line:upper():find(word) then
        table.insert(gen_matches, line)
      end
    end

    local suggestions = {}
    if #matches > 0 then
      table.insert(suggestions, "Functions:")
      for _, m in ipairs(matches) do
        table.insert(suggestions, "  " .. m)
      end
    end
    if #gen_matches > 0 then
      if #suggestions > 0 then
        table.insert(suggestions, "")
      end
      table.insert(suggestions, "Generators:")
      for _, m in ipairs(gen_matches) do
        table.insert(suggestions, "  " .. m)
      end
    end

    if #suggestions > 0 then
      vim.notify("'" .. word .. "' not found. Similar items:\n" .. table.concat(suggestions, "\n"), vim.log.levels.WARN)
    else
      vim.notify("'" .. word .. "' not found in functions or generators", vim.log.levels.WARN)
    end
    return
  end

  -- Create a floating window to show help
  M.show_help_in_float(word, result)
end

-- List all available SQL functions
function M.list_functions(config)
  -- Build command
  local command_path, err = utils.get_command_path(config.command)
  if not command_path then
    vim.notify(err, vim.log.levels.ERROR)
    return
  end

  local cmd = command_path .. " --list-functions"

  -- Execute command
  local result = vim.fn.system(cmd)
  local exit_code = vim.v.shell_error

  if exit_code ~= 0 then
    vim.notify("Failed to list functions", vim.log.levels.ERROR)
    return
  end

  -- Parse functions into categories
  local categories = {}
  local current_category = nil

  for line in result:gmatch("[^\n]+") do
    if line:match("^%s*$") then
      -- Skip empty lines
    elseif line:match("^Available SQL Functions:") then
      -- Skip header
    elseif line:match("^[A-Z].*:$") then
      -- Category header (e.g., "Aggregate Functions:", "Mathematical Functions:")
      current_category = line:gsub(":$", "")
      categories[current_category] = {}
    elseif current_category and line:match("^%s+") then
      -- Function in current category (indented lines)
      table.insert(categories[current_category], line)
    end
  end

  -- Create a buffer to show functions
  vim.cmd("new")
  local buf = vim.api.nvim_get_current_buf()
  vim.api.nvim_buf_set_name(buf, "[SQL Functions]")

  -- Build content
  local lines = {"# SQL CLI Functions", "", "Press <CR> on any function to see detailed help", ""}

  -- Keep categories in order they appear in output
  local category_order = {}
  for line in result:gmatch("[^\n]+") do
    if line:match("^[A-Z].*:$") then
      local category = line:gsub(":$", "")
      if categories[category] then
        table.insert(category_order, category)
      end
    end
  end

  -- Add categories to content
  for _, category in ipairs(category_order) do
    table.insert(lines, "## " .. category)
    table.insert(lines, "")
    for _, func_line in ipairs(categories[category]) do
      table.insert(lines, func_line)
    end
    table.insert(lines, "")
  end

  -- Set content
  vim.api.nvim_buf_set_lines(buf, 0, -1, false, lines)

  -- Set buffer options
  vim.bo[buf].buftype = "nofile"
  vim.bo[buf].bufhidden = "delete"
  vim.bo[buf].modifiable = false
  vim.bo[buf].filetype = "markdown"

  -- Add keymap to get function help
  vim.keymap.set("n", "<CR>", function()
    local line = vim.fn.getline(".")
    local func_name = line:match("^%s*([A-Z_]+)")
    if func_name then
      M.function_help_at_cursor({command = config.command, func_name = func_name})
    end
  end, { buffer = buf, desc = "Show function help" })
end

-- List all available generator functions
function M.list_generators(config)
  -- Build command
  local command_path, err = utils.get_command_path(config.command)
  if not command_path then
    vim.notify(err, vim.log.levels.ERROR)
    return
  end

  local cmd = command_path .. " --list-generators"

  -- Execute command
  local result = vim.fn.system(cmd)
  local exit_code = vim.v.shell_error

  if exit_code ~= 0 then
    vim.notify("Failed to list generators", vim.log.levels.ERROR)
    return
  end

  -- Parse generators into categories
  local categories = {}
  local current_category = nil

  for line in result:gmatch("[^\n]+") do
    if line:match("^%s*$") then
      -- Skip empty lines
    elseif line:match("^=== Available Generator Functions ===") then
      -- Skip header
    elseif line:match("^[A-Z].*Generators:$") then
      -- Category header (e.g., "Mathematical Generators:")
      current_category = line:gsub(":$", "")
      categories[current_category] = {}
    elseif line:match("^Use:") or line:match("^Example:") then
      -- Skip usage lines
      break
    elseif current_category and line:match("^%s+") then
      -- Generator in current category (indented lines)
      table.insert(categories[current_category], line)
    end
  end

  -- Create a buffer to show generators
  vim.cmd("new")
  local buf = vim.api.nvim_get_current_buf()
  vim.api.nvim_buf_set_name(buf, "[SQL Generators]")

  -- Build content
  local lines = {"# SQL CLI Generator Functions", "", "Press <CR> on any generator to see detailed help", ""}

  -- Keep categories in order
  local category_order = {"Mathematical Generators", "Random Generators", "Utility Generators"}

  -- Add categories to content
  for _, category in ipairs(category_order) do
    if categories[category] then
      table.insert(lines, "## " .. category)
      table.insert(lines, "")
      for _, gen_line in ipairs(categories[category]) do
        table.insert(lines, gen_line)
      end
      table.insert(lines, "")
    end
  end

  table.insert(lines, "## Usage")
  table.insert(lines, "")
  table.insert(lines, "  SELECT * FROM <generator>(<args>)")
  table.insert(lines, "  Example: SELECT * FROM GENERATE_PRIMES(100)")

  -- Set content
  vim.api.nvim_buf_set_lines(buf, 0, -1, false, lines)

  -- Set buffer options
  vim.bo[buf].buftype = "nofile"
  vim.bo[buf].bufhidden = "delete"
  vim.bo[buf].swapfile = false
  vim.bo[buf].modifiable = false
  vim.bo[buf].filetype = "markdown"

  -- Add keymap for showing help on Enter
  vim.keymap.set("n", "<CR>", function()
    local line = vim.api.nvim_get_current_line()
    -- Extract generator name (e.g., "  GENERATE_PRIMES - Generate all prime numbers...")
    local generator = line:match("^%s+([A-Z_]+)%s+-")
    if generator then
      local cmd = command_path .. " --generator-help " .. generator
      local help_result = vim.fn.system(cmd)
      if vim.v.shell_error == 0 then
        M.show_help_in_float(generator, help_result)
      else
        vim.notify("Failed to get help for " .. generator, vim.log.levels.ERROR)
      end
    end
  end, { buffer = buf, desc = "Show generator help" })
end

-- Search for functions matching query
function M.search_functions(config)
  vim.ui.input({ prompt = "Search functions: " }, function(query)
    if not query or query == "" then return end

    -- Build command
    local command_path, err = utils.get_command_path(config.command)
    if not command_path then
      vim.notify(err, vim.log.levels.ERROR)
      return
    end

    local cmd = command_path .. " --list-functions"

    -- Execute command
    local result = vim.fn.system(cmd)
    local exit_code = vim.v.shell_error

    if exit_code ~= 0 then
      vim.notify("Failed to list functions", vim.log.levels.ERROR)
      return
    end

    -- Search for matching functions
    local matches = {}
    local query_upper = query:upper()

    for line in result:gmatch("[^\n]+") do
      if line:upper():find(query_upper) then
        table.insert(matches, line)
      end
    end

    if #matches == 0 then
      vim.notify("No functions found matching: " .. query, vim.log.levels.WARN)
      return
    end

    -- If only one match, show its help directly
    if #matches == 1 then
      local func_name = matches[1]:match("^%s*([A-Z_]+)")
      if func_name then
        local help_cmd = command_path .. " --function-help " .. vim.fn.shellescape(func_name)
        local help_result = vim.fn.system(help_cmd)
        if vim.v.shell_error == 0 then
          M.show_help_in_float(func_name, help_result)
          return
        end
      end
    end

    -- Show matches in a buffer
    vim.cmd("new")
    local buf = vim.api.nvim_get_current_buf()
    vim.api.nvim_buf_set_name(buf, "[Search Results: " .. query .. "]")

    local lines = {
      "# Function Search Results",
      "Query: " .. query,
      "Found " .. #matches .. " matches",
      "",
      "Press <CR> on any function to see detailed help",
      "",
    }

    for _, match in ipairs(matches) do
      table.insert(lines, match)
    end

    -- Set content
    vim.api.nvim_buf_set_lines(buf, 0, -1, false, lines)

    -- Set buffer options
    vim.bo[buf].buftype = "nofile"
    vim.bo[buf].bufhidden = "delete"
    vim.bo[buf].modifiable = false
    vim.bo[buf].filetype = "markdown"

    -- Add keymap to get function help
    vim.keymap.set("n", "<CR>", function()
      local line = vim.fn.getline(".")
      local func_name = line:match("^%s*([A-Z_]+)")
      if func_name then
        local help_cmd = command_path .. " --function-help " .. vim.fn.shellescape(func_name)
        local help_result = vim.fn.system(help_cmd)
        if vim.v.shell_error == 0 then
          M.show_help_in_float(func_name, help_result)
        end
      end
    end, { buffer = buf, desc = "Show function help" })
  end)
end

-- Show column help at cursor (smart detection)
function M.column_help_at_cursor(config, state)
  -- Get word under cursor
  local word = vim.fn.expand("<cword>")

  if word == "" then
    vim.notify("No word under cursor", vim.log.levels.WARN)
    return
  end

  -- First, ensure we have schema information
  local schema_columns = state:get_schema_columns()
  if not schema_columns or #schema_columns == 0 then
    M.load_schema_for_completion(config, state)
    schema_columns = state:get_schema_columns()
  end

  -- Check if word is a column
  local found_column = nil
  local word_lower = word:lower()

  for _, col in ipairs(schema_columns) do
    local col_name = type(col) == "table" and col.name or col
    if col_name and col_name:lower() == word_lower then
      found_column = col
      break
    end
  end

  if found_column then
    -- Show column info from schema
    local data_file = state:get_data_file()
    if data_file then
      local command_path, err = utils.get_command_path(config.command)
      if not command_path then
        vim.notify(err, vim.log.levels.ERROR)
        return
      end

      local cmd = command_path .. " " .. vim.fn.shellescape(data_file) .. " --schema"
      local result = vim.fn.system(cmd)

      if vim.v.shell_error == 0 then
        -- Extract just the column info
        local column_info = {}
        for line in result:gmatch("[^\n]+") do
          local col_name = line:match("%d+%.%s+([%w_]+)")
          if col_name and col_name:lower() == word_lower then
            table.insert(column_info, line)
          end
        end

        if #column_info > 0 then
          M.show_help_in_float("Column: " .. word, table.concat(column_info, "\n"))
          return
        end
      end
    end
  end

  -- If not a column, try function help
  M.function_help_at_cursor(config)
end

-- Show schema information
function M.show_schema(config, state)
  local data_file = state:get_data_file()
  if not data_file then
    -- Try to detect from current buffer
    local bufnr = vim.api.nvim_get_current_buf()
    local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)
    local buf_path = vim.api.nvim_buf_get_name(bufnr)
    local buf_dir = nil
    if buf_path and buf_path ~= "" then
      buf_dir = vim.fn.fnamemodify(buf_path, ":h")
    end
    data_file = utils.detect_data_hint(lines, buf_dir)
    if data_file then
      state:set_data_file(data_file)
    end

    -- Check if current buffer is a CSV
    if not data_file and buf_path:match("%.csv$") then
      data_file = buf_path
      state:set_data_file(data_file)
    end
  end

  if not data_file then
    vim.notify("No data file set or detected", vim.log.levels.WARN)
    return
  end

  -- Run schema-json command
  local command_path, err = utils.get_command_path(config.command)
  if not command_path then
    vim.notify(err, vim.log.levels.ERROR)
    return
  end

  local cmd = command_path .. " " .. vim.fn.shellescape(data_file) .. " --schema-json"
  local result = vim.fn.system(cmd)
  local exit_code = vim.v.shell_error

  if exit_code ~= 0 then
    vim.notify("Failed to get schema: " .. result, vim.log.levels.ERROR)
    return
  end

  -- Parse JSON schema
  local ok, schema = pcall(vim.json.decode, result)
  if ok and schema and schema.columns then
    local schema_columns = {}
    local display_lines = {}
    table.insert(display_lines, "Table: " .. schema.table)
    table.insert(display_lines, "Rows: " .. schema.rows)
    table.insert(display_lines, "Columns: " .. #schema.columns)
    table.insert(display_lines, "")
    table.insert(display_lines, "Column Information:")
    table.insert(display_lines, string.rep("-", 60))

    for i, col in ipairs(schema.columns) do
      table.insert(schema_columns, {
        name = col.name,
        type = col.type
      })

      local null_info = ""
      if col.null_count and col.null_count > 0 then
        local null_percent = math.floor((col.null_count / schema.rows) * 100)
        null_info = string.format(" (%d%% NULL)", null_percent)
      end

      table.insert(display_lines, string.format("%2d. %-20s %s%s",
        i, col.name, col.type, null_info))
    end

    -- Store schema for completion
    state:set_schema_columns(schema_columns)

    -- Show in floating window
    M.show_help_in_float("Schema: " .. schema.table, table.concat(display_lines, "\n"))
  else
    vim.notify("Failed to parse schema", vim.log.levels.ERROR)
  end
end

-- Load schema for completion (called when data file is set)
function M.load_schema_for_completion(config, state)
  local data_file = state:get_data_file()
  if not data_file then
    return
  end

  -- Run schema-json command to get clean JSON output
  local command_path, err = utils.get_command_path(config.command)
  if not command_path then
    return
  end

  local cmd = command_path .. " " .. vim.fn.shellescape(data_file) .. " --schema-json"
  local result = vim.fn.system(cmd)
  local exit_code = vim.v.shell_error

  if exit_code == 0 then
    -- Parse JSON schema
    local ok, schema = pcall(vim.json.decode, result)
    if ok and schema and schema.columns then
      local schema_columns = {}
      for _, col in ipairs(schema.columns) do
        table.insert(schema_columns, {
          name = col.name,
          type = col.type
        })
      end
      state:set_schema_columns(schema_columns)

      if #schema_columns > 0 then
        vim.notify(string.format("Loaded schema: %d columns from %s", #schema_columns, schema.table), vim.log.levels.INFO)
      end
    else
      -- Fallback to old method if JSON parsing fails
      local schema_columns = {}
      local clean_result = utils.strip_ansi_codes(result)
      for line in clean_result:gmatch("[^\n]+") do
        -- Match lines like "1. column_name TYPE"
        local num, col_name, col_type = line:match("%s*(%d+)%.%s+([%w_]+)%s+([%w]+)")
        if col_name and col_type then
          table.insert(schema_columns, {
            name = col_name,
            type = col_type
          })
        end
      end
      state:set_schema_columns(schema_columns)

      if #schema_columns > 0 then
        vim.notify(string.format("Loaded schema: %d columns", #schema_columns), vim.log.levels.INFO)
      end
    end
  end
end

-- Show help text in a floating window
function M.show_help_in_float(title, content)
  -- Strip ANSI codes from content
  content = utils.strip_ansi_codes(content)

  -- Split content into lines
  local lines = {}
  for line in content:gmatch("[^\n]+") do
    table.insert(lines, line)
  end

  -- Create buffer
  local buf = vim.api.nvim_create_buf(false, true)
  vim.api.nvim_buf_set_lines(buf, 0, -1, false, lines)

  -- Set buffer options
  vim.bo[buf].modifiable = false
  vim.bo[buf].buftype = "nofile"
  vim.bo[buf].filetype = ""  -- No filetype to avoid unwanted highlighting

  -- Apply custom syntax highlighting for schema display
  vim.api.nvim_buf_call(buf, function()
    -- Highlight column numbers consistently
    vim.cmd([[syntax match SqlSchemaNumber /^\s*\d\+\./]])
    vim.cmd([[syntax match SqlSchemaColumnName /\d\+\.\s\+\zs\S\+/]])
    vim.cmd([[syntax match SqlSchemaType /\s\+\zs\(String\|Integer\|Float\|Boolean\|DateTime\|Mixed\|Null\)\ze/]])
    vim.cmd([[syntax match SqlSchemaNullPercent /(\d\+% NULL)/]])
    vim.cmd([[syntax match SqlSchemaHeader /^Table:\|^Rows:\|^Columns:/]])
    vim.cmd([[syntax match SqlSchemaSeparator /^-\+$/]])

    -- Define colors
    vim.cmd([[hi def link SqlSchemaNumber Number]])
    vim.cmd([[hi def link SqlSchemaColumnName Identifier]])
    vim.cmd([[hi def link SqlSchemaType Type]])
    vim.cmd([[hi def link SqlSchemaNullPercent Comment]])
    vim.cmd([[hi def link SqlSchemaHeader Title]])
    vim.cmd([[hi def link SqlSchemaSeparator NonText]])
  end)

  -- Calculate window size
  local width = 80
  local height = math.min(#lines, 30)

  -- Get editor dimensions
  local ui = vim.api.nvim_list_uis()[1]
  local row = math.floor((ui.height - height) / 2)
  local col = math.floor((ui.width - width) / 2)

  -- Create floating window
  local opts = {
    relative = "editor",
    width = width,
    height = height,
    row = row,
    col = col,
    style = "minimal",
    border = "rounded",
    title = " " .. title .. " Function Help ",
    title_pos = "center",
  }

  local win = vim.api.nvim_open_win(buf, true, opts)

  -- Set window options
  vim.wo[win].wrap = true
  vim.wo[win].linebreak = true
  vim.wo[win].cursorline = true

  -- Set up keymaps to close window
  vim.keymap.set("n", "q", function()
    vim.api.nvim_win_close(win, true)
  end, { buffer = buf, silent = true })

  vim.keymap.set("n", "<Esc>", function()
    vim.api.nvim_win_close(win, true)
  end, { buffer = buf, silent = true })

  -- Add syntax highlighting for SQL code blocks
  vim.cmd([[
    syntax match SqlFunctionName /^[A-Z_]\+/
    syntax match SqlFunctionCategory /^Category:.*/
    syntax match SqlFunctionArgs /^Arguments:.*/
    syntax match SqlFunctionReturns /^Returns:.*/
    syntax match SqlFunctionExample /^Example:/
    syntax region SqlCodeBlock start=/^  / end=/$/

    highlight SqlFunctionName guifg=#8be9fd ctermfg=14
    highlight SqlFunctionCategory guifg=#50fa7b ctermfg=10
    highlight SqlFunctionArgs guifg=#f1fa8c ctermfg=11
    highlight SqlFunctionReturns guifg=#ff79c6 ctermfg=13
    highlight SqlFunctionExample guifg=#bd93f9 ctermfg=5
    highlight SqlCodeBlock guifg=#f8f8f2 ctermfg=7
  ]])
end

return M
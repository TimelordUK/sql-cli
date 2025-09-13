-- SQL CLI Export Module
-- Multiple export formats for query results

local M = {}

-- Get the full result table from buffer
local function get_table_data(bufnr, table_info)
  local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)
  local data = {
    headers = {},
    rows = {}
  }

  -- Extract headers
  if table_info.header_row then
    local header_line = lines[table_info.header_row]
    for _, col_pos in ipairs(table_info.column_positions) do
      local col_text = header_line:sub(col_pos.start, col_pos.stop)
      col_text = col_text:gsub("^%s*", ""):gsub("%s*$", "")
      table.insert(data.headers, col_text)
    end
  end

  -- Extract data rows
  if table_info.data_start and table_info.data_end then
    for row_idx = table_info.data_start, table_info.data_end do
      local line = lines[row_idx]
      if line and line:match("^|") then
        local row = {}
        for _, col_pos in ipairs(table_info.column_positions) do
          local value = line:sub(col_pos.start, col_pos.stop)
          value = value:gsub("^%s*", ""):gsub("%s*$", "")
          table.insert(row, value)
        end
        table.insert(data.rows, row)
      end
    end
  end

  return data
end

-- Export to HTML table (works great in Outlook/Teams)
function M.yank_as_html(bufnr, table_info)
  local data = get_table_data(bufnr, table_info)

  local html = {'<table border="1" cellpadding="4" cellspacing="0" style="border-collapse: collapse;">'}

  -- Add header row
  table.insert(html, '  <thead>')
  table.insert(html, '    <tr style="background-color: #f0f0f0;">')
  for _, header in ipairs(data.headers) do
    table.insert(html, string.format('      <th>%s</th>', header))
  end
  table.insert(html, '    </tr>')
  table.insert(html, '  </thead>')

  -- Add data rows
  table.insert(html, '  <tbody>')
  for _, row in ipairs(data.rows) do
    table.insert(html, '    <tr>')
    for _, value in ipairs(row) do
      -- Escape HTML special characters
      local escaped = value:gsub("&", "&amp;"):gsub("<", "&lt;"):gsub(">", "&gt;")
      table.insert(html, string.format('      <td>%s</td>', escaped))
    end
    table.insert(html, '    </tr>')
  end
  table.insert(html, '  </tbody>')
  table.insert(html, '</table>')

  local html_str = table.concat(html, '\n')
  vim.fn.setreg('+', html_str)
  vim.fn.setreg('"', html_str)

  vim.notify(string.format("Yanked %d rows as HTML table (paste into Outlook/Teams)", #data.rows), vim.log.levels.INFO)
  return html_str
end

-- Export to Markdown table
function M.yank_as_markdown(bufnr, table_info)
  local data = get_table_data(bufnr, table_info)

  local md = {}

  -- Header row
  local header_line = "| " .. table.concat(data.headers, " | ") .. " |"
  table.insert(md, header_line)

  -- Separator row
  local separators = {}
  for i = 1, #data.headers do
    table.insert(separators, "---")
  end
  table.insert(md, "| " .. table.concat(separators, " | ") .. " |")

  -- Data rows
  for _, row in ipairs(data.rows) do
    table.insert(md, "| " .. table.concat(row, " | ") .. " |")
  end

  local md_str = table.concat(md, '\n')
  vim.fn.setreg('+', md_str)
  vim.fn.setreg('"', md_str)

  vim.notify(string.format("Yanked %d rows as Markdown table", #data.rows), vim.log.levels.INFO)
  return md_str
end

-- Export to Tab-Separated Values (perfect for Excel)
function M.yank_as_tsv(bufnr, table_info)
  local data = get_table_data(bufnr, table_info)

  local tsv = {}

  -- Header row
  table.insert(tsv, table.concat(data.headers, "\t"))

  -- Data rows
  for _, row in ipairs(data.rows) do
    table.insert(tsv, table.concat(row, "\t"))
  end

  local tsv_str = table.concat(tsv, '\n')
  vim.fn.setreg('+', tsv_str)
  vim.fn.setreg('"', tsv_str)

  vim.notify(string.format("Yanked %d rows as TSV (paste into Excel)", #data.rows), vim.log.levels.INFO)
  return tsv_str
end

-- Export to SQL INSERT statements
function M.yank_as_insert(bufnr, table_info, table_name)
  local data = get_table_data(bufnr, table_info)

  -- Ask for table name if not provided
  if not table_name or table_name == "" then
    table_name = "table_name"
  end

  local inserts = {}

  -- Generate INSERT statements
  for _, row in ipairs(data.rows) do
    local values = {}
    for _, value in ipairs(row) do
      -- Detect if value is numeric or needs quotes
      if value == "NULL" or value == "" then
        table.insert(values, "NULL")
      elseif value:match("^%-?%d+%.?%d*$") then
        -- Numeric value
        table.insert(values, value)
      else
        -- String value - escape single quotes
        local escaped = value:gsub("'", "''")
        table.insert(values, string.format("'%s'", escaped))
      end
    end

    local insert = string.format("INSERT INTO %s (%s) VALUES (%s);",
      table_name,
      table.concat(data.headers, ", "),
      table.concat(values, ", ")
    )
    table.insert(inserts, insert)
  end

  local sql_str = table.concat(inserts, '\n')
  vim.fn.setreg('+', sql_str)
  vim.fn.setreg('"', sql_str)

  vim.notify(string.format("Yanked %d INSERT statements for table '%s'", #data.rows, table_name), vim.log.levels.INFO)
  return sql_str
end

-- Generate CREATE TABLE statement from schema
function M.yank_create_table(bufnr, table_info, table_name)
  local data = get_table_data(bufnr, table_info)

  if not table_name or table_name == "" then
    table_name = "table_name"
  end

  local columns = {}

  -- Analyze first few rows to determine column types
  for col_idx, header in ipairs(data.headers) do
    local col_type = "VARCHAR(255)"  -- Default type
    local all_numeric = true
    local all_integer = true
    local max_length = 0

    -- Check first 10 rows to determine type
    for row_idx = 1, math.min(10, #data.rows) do
      local value = data.rows[row_idx][col_idx]
      if value and value ~= "" and value ~= "NULL" then
        max_length = math.max(max_length, #value)

        if not value:match("^%-?%d+%.?%d*$") then
          all_numeric = false
          all_integer = false
        elseif value:match("%.") then
          all_integer = false
        end
      end
    end

    -- Determine column type
    if all_integer then
      col_type = "INTEGER"
    elseif all_numeric then
      col_type = "DECIMAL(10,2)"
    else
      col_type = string.format("VARCHAR(%d)", math.max(max_length * 2, 50))
    end

    -- Clean column name (remove spaces, special chars)
    local clean_name = header:gsub("[^%w_]", "_"):lower()
    table.insert(columns, string.format("  %s %s", clean_name, col_type))
  end

  local create_sql = string.format("CREATE TABLE %s (\n%s\n);",
    table_name,
    table.concat(columns, ",\n")
  )

  vim.fn.setreg('+', create_sql)
  vim.fn.setreg('"', create_sql)

  vim.notify(string.format("Yanked CREATE TABLE statement for '%s'", table_name), vim.log.levels.INFO)
  return create_sql
end

-- Export to CSV (proper escaping)
function M.yank_as_csv(bufnr, table_info)
  local data = get_table_data(bufnr, table_info)

  local csv = {}

  -- Helper to escape CSV values
  local function escape_csv(value)
    if value:match('[,"\n]') then
      return '"' .. value:gsub('"', '""') .. '"'
    end
    return value
  end

  -- Header row
  local headers = {}
  for _, h in ipairs(data.headers) do
    table.insert(headers, escape_csv(h))
  end
  table.insert(csv, table.concat(headers, ","))

  -- Data rows
  for _, row in ipairs(data.rows) do
    local values = {}
    for _, v in ipairs(row) do
      table.insert(values, escape_csv(v))
    end
    table.insert(csv, table.concat(values, ","))
  end

  local csv_str = table.concat(csv, '\n')
  vim.fn.setreg('+', csv_str)
  vim.fn.setreg('"', csv_str)

  vim.notify(string.format("Yanked %d rows as CSV", #data.rows), vim.log.levels.INFO)
  return csv_str
end

-- Show export menu
function M.show_export_menu(bufnr, table_info)
  local options = {
    "1. HTML Table (Outlook/Teams)",
    "2. Markdown Table",
    "3. Tab-Separated (Excel)",
    "4. CSV (Proper escaping)",
    "5. SQL INSERT Statements",
    "6. CREATE TABLE Statement",
  }

  vim.ui.select(options, {
    prompt = "Select export format:",
  }, function(choice)
    if not choice then return end

    local choice_num = tonumber(choice:match("^(%d+)"))
    if choice_num == 1 then
      M.yank_as_html(bufnr, table_info)
    elseif choice_num == 2 then
      M.yank_as_markdown(bufnr, table_info)
    elseif choice_num == 3 then
      M.yank_as_tsv(bufnr, table_info)
    elseif choice_num == 4 then
      M.yank_as_csv(bufnr, table_info)
    elseif choice_num == 5 then
      vim.ui.input({ prompt = "Table name: ", default = "my_table" }, function(name)
        if name then
          M.yank_as_insert(bufnr, table_info, name)
        end
      end)
    elseif choice_num == 6 then
      vim.ui.input({ prompt = "Table name: ", default = "my_table" }, function(name)
        if name then
          M.yank_create_table(bufnr, table_info, name)
        end
      end)
    end
  end)
end

return M
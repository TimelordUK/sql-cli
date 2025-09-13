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
function M.yank_as_html(bufnr, table_info, open_browser)
  local data = get_table_data(bufnr, table_info)

  -- Create full HTML document for browser viewing
  local full_html = {
    '<!DOCTYPE html>',
    '<html>',
    '<head>',
    '  <meta charset="UTF-8">',
    '  <title>SQL Query Results</title>',
    '  <style>',
    '    body { font-family: Arial, sans-serif; margin: 20px; }',
    '    table { border-collapse: collapse; width: auto; }',
    '    th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }',
    '    th { background-color: #4CAF50; color: white; }',
    '    tr:nth-child(even) { background-color: #f2f2f2; }',
    '    tr:hover { background-color: #ddd; }',
    '    .info { margin: 20px 0; padding: 10px; background: #e7f3fe; border-left: 4px solid #2196F3; }',
    '  </style>',
    '</head>',
    '<body>',
    '  <div class="info">',
    '    <strong>Tip:</strong> Select the table below and copy (Ctrl+C/Cmd+C) to paste into Gmail, Outlook, or Teams.',
    '  </div>',
  }

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

  local html_table = table.concat(html, '\n')

  if open_browser then
    -- Add table to full HTML
    table.insert(full_html, html_table)
    table.insert(full_html, '</body>')
    table.insert(full_html, '</html>')

    -- Write to temp file and open in browser
    -- Use a more predictable location for WSL
    local temp_file
    if vim.fn.has('wsl') == 1 then
      -- Save to Windows temp directory for easy access
      temp_file = '/mnt/c/temp/sql_export_' .. os.date('%Y%m%d_%H%M%S') .. '.html'
      -- Ensure directory exists
      vim.fn.system('mkdir -p /mnt/c/temp')
    else
      temp_file = os.tmpname() .. '.html'
    end
    local file = io.open(temp_file, 'w')
    if file then
      file:write(table.concat(full_html, '\n'))
      file:close()

      -- Open in default browser (cross-platform)
      local open_cmd
      if vim.fn.has('mac') == 1 then
        open_cmd = 'open'
      elseif vim.fn.has('unix') == 1 then
        open_cmd = 'xdg-open'
      elseif vim.fn.has('win32') == 1 then
        open_cmd = 'start'
      end

      if open_cmd then
        vim.fn.system(string.format('%s "%s"', open_cmd, temp_file))

        -- For WSL users, provide Windows path
        if vim.fn.has('wsl') == 1 then
          -- File is saved at C:\temp\sql_export_*.html
          local win_path = temp_file:gsub('/mnt/c/', 'C:\\'):gsub('/', '\\')
          vim.notify(string.format("HTML saved to: %s", win_path), vim.log.levels.INFO)
          vim.notify("Open this file in Windows browser, then copy table to Gmail", vim.log.levels.INFO)

          -- Copy the Windows path to clipboard for easy access
          vim.fn.setreg('+', win_path)
        else
          vim.notify(string.format("Opened HTML table in browser - copy from there to paste into Gmail", #data.rows), vim.log.levels.INFO)
        end
      end
    end
  else
    -- Just yank the HTML code
    vim.fn.setreg('+', html_table)
    vim.fn.setreg('"', html_table)
    vim.notify(string.format("Yanked %d rows as HTML table", #data.rows), vim.log.levels.INFO)
  end

  return html_table
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
    "1. Open HTML in Browser (for Gmail/Teams copy)",
    "2. HTML Table Code (raw HTML)",
    "3. Markdown Table",
    "4. Tab-Separated (Excel)",
    "5. CSV (Proper escaping)",
    "6. SQL INSERT Statements",
    "7. CREATE TABLE Statement",
  }

  vim.ui.select(options, {
    prompt = "Select export format:",
  }, function(choice)
    if not choice then return end

    local choice_num = tonumber(choice:match("^(%d+)"))
    if choice_num == 1 then
      M.yank_as_html(bufnr, table_info, true)  -- Open in browser
    elseif choice_num == 2 then
      M.yank_as_html(bufnr, table_info, false) -- Just yank HTML
    elseif choice_num == 3 then
      M.yank_as_markdown(bufnr, table_info)
    elseif choice_num == 4 then
      M.yank_as_tsv(bufnr, table_info)
    elseif choice_num == 5 then
      M.yank_as_csv(bufnr, table_info)
    elseif choice_num == 6 then
      vim.ui.input({ prompt = "Table name: ", default = "my_table" }, function(name)
        if name then
          M.yank_as_insert(bufnr, table_info, name)
        end
      end)
    elseif choice_num == 7 then
      vim.ui.input({ prompt = "Table name: ", default = "my_table" }, function(name)
        if name then
          M.yank_create_table(bufnr, table_info, name)
        end
      end)
    end
  end)
end

return M
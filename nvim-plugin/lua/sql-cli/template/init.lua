-- SQL-CLI Template System Integration
-- Main module for neovim plugin integration

local M = {}
local engine = require('sql-cli.template.engine')
local loader = require('sql-cli.template.loader')
local parser = require('sql-cli.template.parser')

-- Configuration
M.config = {
  template_dirs = {
    vim.fn.stdpath("config") .. "/sql-templates/",
    vim.fn.stdpath("data") .. "/sql-cli/templates/",
    "./templates/",
    "./sql-templates/"
  },
  extensions = {".sql", ".sqlt"},  -- Support both SQL and template files
  max_depth = 10,
  debug = false
}

-- Helper to read file content
function M.read_file(filepath)
  local file = io.open(filepath, "r")
  if not file then return nil end
  local content = file:read("*all")
  file:close()
  return content
end

-- Initialize the template system
function M.setup(opts)
  if opts then
    M.config = vim.tbl_deep_extend("force", M.config, opts)
  end
  engine.setup(M.config)
end

-- Expand macro at cursor position
function M.expand_at_cursor()
  local line = vim.api.nvim_get_current_line()
  local row, col = unpack(vim.api.nvim_win_get_cursor(0))

  -- Find macro/template pattern at cursor
  local patterns = {
    { pattern = "@{([^}]+)}", prefix = "@{", suffix = "}" },
    { pattern = "@([%w_]+)", prefix = "@", suffix = "" }
  }

  local start_pos, end_pos, macro_content, raw_name

  for _, p in ipairs(patterns) do
    local pos = 1
    while pos <= #line do
      local s, e, content = line:find(p.pattern, pos)
      if s and s <= col + 1 and e >= col + 1 then
        start_pos = s
        end_pos = e
        raw_name = content
        macro_content = p.prefix .. content .. p.suffix
        break
      end
      pos = (e or pos) + 1
    end
    if start_pos then break end
  end

  if not start_pos then
    vim.notify("No macro found at cursor position", vim.log.levels.INFO)
    return
  end

  -- Get macros from current file
  local file_macros = parser.parse_macros()

  -- Also load templates from template files
  local all_templates = {}
  for _, dir in ipairs(M.config.template_dirs) do
    local expanded_dir = vim.fn.expand(dir)
    if vim.fn.isdirectory(expanded_dir) == 1 then
      for _, ext in ipairs(M.config.extensions or {".sql", ".sqlt"}) do
        local pattern = expanded_dir .. "*" .. ext
        local found = vim.fn.glob(pattern, false, true)
        for _, file in ipairs(found) do
          local content = M.read_file(file)
          if content then
            local data = loader.parse_content(content)
            if data.templates then
              for name, template in pairs(data.templates) do
                all_templates[name] = template
              end
            end
            -- Also merge macros from template files
            if data.macros then
              for name, macro in pairs(data.macros) do
                if not file_macros[name] then
                  file_macros[name] = macro
                end
              end
            end
          end
        end
      end
    end
  end

  -- Check if this is a template reference using @ shorthand
  if raw_name and not raw_name:match(":") then
    -- Check if it's a template name
    if all_templates[raw_name] then
      -- Convert @TEMPLATE_NAME to @{TEMPLATE:TEMPLATE_NAME}
      macro_content = "@{TEMPLATE:" .. raw_name .. "}"
      vim.notify("Expanding template: " .. raw_name, vim.log.levels.INFO)
    elseif not file_macros[raw_name] then
      -- Not a macro or template
      vim.notify("Unknown macro/template: " .. raw_name .. ". Define with -- MACRO: or -- Template:", vim.log.levels.WARN)
      return
    end
  end

  vim.notify("Starting expansion of: " .. macro_content, vim.log.levels.DEBUG)

  -- Expand the macro/template
  engine.expand(macro_content, {
    macros = file_macros,
    templates = all_templates,
    interactive = true
  }, function(expanded)
    vim.notify("Expansion complete, result length: " .. (expanded and #expanded or 0), vim.log.levels.DEBUG)

    if expanded then
      -- Replace inline using vim.schedule to ensure we're in the right context
      vim.schedule(function()
        -- Check if expanded text has multiple lines
        local lines = vim.split(expanded, "\n", { plain = true })

        if #lines == 1 then
          -- Single line - simple replacement
          local current_line = vim.api.nvim_get_current_line()
          local new_line = current_line:sub(1, start_pos - 1) .. expanded .. current_line:sub(end_pos + 1)
          vim.api.nvim_set_current_line(new_line)
          -- Move cursor to end of replacement
          vim.api.nvim_win_set_cursor(0, {row, start_pos + #expanded - 1})
        else
          -- Multi-line - need to handle differently
          local current_line = vim.api.nvim_get_current_line()
          local before = current_line:sub(1, start_pos - 1)
          local after = current_line:sub(end_pos + 1)

          -- Combine first line with existing content
          lines[1] = before .. lines[1]
          -- Combine last line with remaining content
          lines[#lines] = lines[#lines] .. after

          -- Replace current line and insert additional lines
          vim.api.nvim_buf_set_lines(0, row - 1, row, false, lines)

          -- Move cursor to end of inserted content
          local last_line_num = row + #lines - 1
          local last_line_len = #lines[#lines] - #after
          vim.api.nvim_win_set_cursor(0, {last_line_num, last_line_len})
        end

        vim.notify("Template expanded successfully", vim.log.levels.INFO)
      end)
    else
      vim.notify("Template expansion cancelled or failed", vim.log.levels.WARN)
    end
  end)
end

-- Select and insert template from file
function M.select_template()
  -- Find template files (support both .sql and .sqlt)
  local files = {}
  local extensions = M.config.extensions or {".sql", ".sqlt"}

  for _, dir in ipairs(M.config.template_dirs) do
    local expanded = vim.fn.expand(dir)
    if vim.fn.isdirectory(expanded) == 1 then
      for _, ext in ipairs(extensions) do
        local pattern = expanded .. "*" .. ext
        local found = vim.fn.glob(pattern, false, true)
        for _, file in ipairs(found) do
          -- Check if file contains templates or macros
          local content = M.read_file(file)
          if content and (content:match("Template:") or content:match("Macro:")) then
            table.insert(files, file)
          end
        end
      end
    end
  end

  if #files == 0 then
    vim.notify("No template files found", vim.log.levels.WARN)
    return
  end

  -- If multiple files, let user choose
  local selected_file
  if #files == 1 then
    selected_file = files[1]
  else
    local file_names = {}
    for _, f in ipairs(files) do
      table.insert(file_names, vim.fn.fnamemodify(f, ":t"))
    end

    vim.ui.select(file_names, {
      prompt = "Select template file: "
    }, function(choice, idx)
      if choice and idx then
        selected_file = files[idx]
      end
    end)

    if not selected_file then return end
  end

  -- List templates in selected file
  local templates = loader.list_templates(selected_file)
  if #templates == 0 then
    vim.notify("No templates found in file", vim.log.levels.WARN)
    return
  end

  -- Select template
  local items = {}
  for _, t in ipairs(templates) do
    table.insert(items, t.name .. " - " .. t.description)
  end

  vim.ui.select(items, {
    prompt = "Select template: "
  }, function(choice, idx)
    if choice and idx then
      local template = templates[idx]

      -- Get current file macros and template file macros
      local file_macros = parser.parse_macros()
      local template_macros = loader.get_macros(selected_file)

      -- Merge macros (current file takes precedence)
      local merged_macros = vim.tbl_extend("force", template_macros, file_macros)

      -- Expand template
      local expanded = engine.expand_from_file(selected_file, template.name, {
        macros = merged_macros,
        interactive = true
      })

      -- Insert at cursor
      M.insert_at_cursor(expanded)
    end
  end)
end

-- Insert text at cursor
function M.insert_at_cursor(text)
  if not text then return end

  local row, col = unpack(vim.api.nvim_win_get_cursor(0))
  local lines = vim.split(text, "\n")

  if #lines == 1 then
    -- Single line insertion
    local current_line = vim.api.nvim_get_current_line()
    local new_line = current_line:sub(1, col) .. text .. current_line:sub(col + 1)
    vim.api.nvim_set_current_line(new_line)
    vim.api.nvim_win_set_cursor(0, {row, col + #text})
  else
    -- Multi-line insertion
    vim.api.nvim_put(lines, "c", true, true)
  end
end

-- List all available templates
function M.list_templates()
  local all_templates = {}

  -- Find all template files
  for _, dir in ipairs(M.config.template_dirs) do
    local expanded = vim.fn.expand(dir)
    if vim.fn.isdirectory(expanded) == 1 then
      for _, ext in ipairs(M.config.extensions or {".sql", ".sqlt"}) do
        local pattern = expanded .. "*" .. ext
        local found = vim.fn.glob(pattern, false, true)
        for _, file in ipairs(found) do
          local content = M.read_file(file)
          if content and (content:match("Template:") or content:match("Macro:")) then
            local templates = loader.list_templates(file)
            for _, t in ipairs(templates) do
              table.insert(all_templates, {
                file = vim.fn.fnamemodify(file, ":~:."),
                name = t.name,
                description = t.description
              })
            end
          end
        end
      end
    end
  end

  -- Display in quickfix or floating window
  if #all_templates == 0 then
    vim.notify("No templates found", vim.log.levels.INFO)
    return
  end

  local lines = {"Available Templates:", ""}
  for _, t in ipairs(all_templates) do
    table.insert(lines, string.format("  %s - %s", t.name, t.description))
    table.insert(lines, string.format("    File: %s", t.file))
    table.insert(lines, "")
  end

  -- Create floating window
  local buf = vim.api.nvim_create_buf(false, true)
  vim.api.nvim_buf_set_lines(buf, 0, -1, false, lines)
  vim.api.nvim_buf_set_option(buf, 'modifiable', false)

  local width = math.min(80, vim.o.columns - 4)
  local height = math.min(#lines, vim.o.lines - 4)

  vim.api.nvim_open_win(buf, true, {
    relative = "editor",
    width = width,
    height = height,
    row = math.floor((vim.o.lines - height) / 2),
    col = math.floor((vim.o.columns - width) / 2),
    border = "rounded",
    style = "minimal"
  })
end

-- Create commands
function M.create_commands()
  vim.api.nvim_create_user_command("SqlTemplateExpand", function()
    M.expand_at_cursor()
  end, { desc = "Expand SQL template macro at cursor" })

  vim.api.nvim_create_user_command("SqlTemplateSelect", function()
    M.select_template()
  end, { desc = "Select and insert SQL template" })

  vim.api.nvim_create_user_command("SqlTemplateList", function()
    M.list_templates()
  end, { desc = "List available SQL templates" })

  vim.api.nvim_create_user_command("SqlTemplateReload", function()
    loader.clear_cache()
    vim.notify("Template cache cleared", vim.log.levels.INFO)
  end, { desc = "Reload SQL template cache" })

  vim.api.nvim_create_user_command("SqlTemplateDebug", function(opts)
    local engine = require('sql-cli.template.engine')
    if opts.args == "on" then
      engine.set_debug(true)
      vim.notify("Template debug mode ON", vim.log.levels.INFO)
    elseif opts.args == "off" then
      engine.set_debug(false)
      vim.notify("Template debug mode OFF", vim.log.levels.INFO)
    else
      local status = engine.get_debug() and "ON" or "OFF"
      vim.notify("Template debug mode is " .. status, vim.log.levels.INFO)
    end
  end, {
    desc = "Toggle SQL template debug mode",
    nargs = "?",
    complete = function() return {"on", "off"} end
  })
end

-- Create keymaps
function M.create_keymaps(opts)
  opts = opts or {}
  local prefix = opts.prefix or "<leader>st"

  -- Template operations
  vim.keymap.set("n", prefix .. "e", M.expand_at_cursor, {
    desc = "SQL Template: Expand at cursor",
    silent = true
  })

  vim.keymap.set("n", prefix .. "s", M.select_template, {
    desc = "SQL Template: Select from file",
    silent = true
  })

  vim.keymap.set("n", prefix .. "l", M.list_templates, {
    desc = "SQL Template: List all available",
    silent = true
  })

  vim.keymap.set("n", prefix .. "r", function()
    loader.clear_cache()
    vim.notify("Template cache cleared", vim.log.levels.INFO)
  end, {
    desc = "SQL Template: Reload cache",
    silent = true
  })

  -- Quick access to common operations
  vim.keymap.set("n", prefix .. "m", function()
    -- Quick macro expand (same as expand but with visual feedback)
    local line = vim.api.nvim_get_current_line()
    local col = vim.api.nvim_win_get_cursor(0)[2]

    -- Check if we're on a macro
    if line:match("@{[^}]+}") then
      vim.notify("Expanding macro...", vim.log.levels.INFO)
      M.expand_at_cursor()
    else
      vim.notify("No macro at cursor position", vim.log.levels.WARN)
    end
  end, {
    desc = "SQL Template: Quick macro expand",
    silent = true
  })

  -- Debug mode toggle
  vim.keymap.set("n", prefix .. "d", function()
    local engine = require('sql-cli.template.engine')
    local current = engine.get_debug()
    engine.set_debug(not current)
    vim.notify("Template debug: " .. (engine.get_debug() and "ON" or "OFF"), vim.log.levels.INFO)
  end, {
    desc = "SQL Template: Toggle debug mode",
    silent = true
  })

  -- Test template expansion (for development)
  if M.config.debug then
    vim.keymap.set("n", prefix .. "t", M.test, {
      desc = "SQL Template: Run test",
      silent = true
    })
  end
end

-- Test function for development
function M.test()
  local test_text = [[
URL is @{VAR:BASE_URL}/api
Token: @{VAR:JWT_TOKEN}
Source: @{JPICKER:SOURCES:Select Source}
Date: @{DATE:Select Date}
]]

  local macros = {
    _CONFIG = {
      BASE_URL = "https://api.test.com",
      JWT_TOKEN = "test-token"
    },
    SOURCES = "Bloomberg\nReuters\nInternal"
  }

  local result = engine.test(test_text, {
    ["VAR:BASE_URL"] = "https://api.test.com",
    ["VAR:JWT_TOKEN"] = "test-token",
    ["JPICKER:SOURCES:Select Source"] = '\\"Bloomberg\\"',
    ["DATE:Select Date"] = "DateTime(2025, 9, 25)"
  }, macros)

  print("Test result:\n" .. result)
end

return M
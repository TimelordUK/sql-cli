-- Template Engine Module
-- Main module that orchestrates template expansion

local M = {}
local parser = require('sql-cli.template.parser')
local resolver = require('sql-cli.template.resolver')
local loader = require('sql-cli.template.loader')

-- Configuration
local config = {
  max_depth = 10,
  template_dirs = {
    vim.fn.expand("~/.config/nvim/sql-templates/"),
    vim.fn.expand("~/.local/share/sql-cli/templates/"),
    "./templates/"
  },
  debug = false  -- Set to true to add debug comments during expansion
}

-- Helper to add debug comments
local function add_debug(msg)
  if config.debug then
    return "-- [DEBUG] " .. msg .. "\n"
  end
  return ""
end

-- Expand a template string
function M.expand(text, options, callback)
  options = options or {}
  local macros = options.macros or {}
  local interactive = options.interactive ~= false
  local test_values = options.test_values
  local depth = options.depth or 0
  local debug_output = options.debug_output or ""

  -- Support both sync and async modes
  local is_async = callback ~= nil

  -- Check recursion depth
  if depth >= config.max_depth then
    local err = "Maximum template expansion depth reached. Check for circular references."
    if is_async then
      vim.notify(err, vim.log.levels.ERROR)
      callback(nil)
      return
    else
      error(err)
    end
  end

  -- Initialize resolver with templates if available
  local templates = options.templates
  resolver.init(macros, macros._CONFIG, templates)

  -- Parse patterns
  local parsed = parser.parse_patterns(text)

  if config.debug then
    debug_output = debug_output .. add_debug("Parsing text: " .. text:sub(1, 100))
    debug_output = debug_output .. add_debug("Found " .. #parsed.patterns .. " patterns")
    for i, p in ipairs(parsed.patterns) do
      debug_output = debug_output .. add_debug("  Pattern " .. i .. ": " .. p.type .. " - " .. (p.args or ""))
    end
  end

  -- If no patterns found, return original text
  if #parsed.patterns == 0 then
    if is_async then
      callback(debug_output .. text)
    end
    return debug_output .. text
  end

  -- For synchronous mode (testing)
  if not is_async then
    -- Resolve each pattern synchronously
    local resolved = {}
    for i, pattern in ipairs(parsed.patterns) do
      if test_values then
        resolved[i] = resolver.test_resolve(pattern, test_values)
      else
        resolved[i] = resolver.resolve(pattern, false) -- Non-interactive for sync
      end
    end

    -- Substitute placeholders
    local result = parsed.text
    for i, pattern in ipairs(parsed.patterns) do
      local replacement = resolved[i] or pattern.match
      -- Escape special characters for gsub
      local safe_replacement = replacement:gsub("%%", "%%%%")
      result = result:gsub(pattern.placeholder, safe_replacement)
    end

    -- Check for new patterns (recursive expansion)
    if result:match("@{[^}]+}") then
      result = M.expand(result, {
        macros = macros,
        interactive = false,
        test_values = test_values,
        templates = templates,
        depth = depth + 1
      })
    end

    -- Restore escaped patterns only at the top level
    if depth == 0 then
      result = result:gsub("__ESCAPED_AT_BRACE__", "@{")
    end

    return result
  end

  -- Async mode for interactive use
  local resolved = {}
  local pending = #parsed.patterns
  local completed = 0

  local function check_complete()
    completed = completed + 1
    if completed >= pending then
      -- All patterns resolved, perform substitution
      local result = parsed.text
      for i, pattern in ipairs(parsed.patterns) do
        local replacement = resolved[i] or pattern.match
        -- Escape special characters for gsub
        local safe_replacement = replacement:gsub("%%", "%%%%")
        result = result:gsub(pattern.placeholder, safe_replacement)
      end

      if config.debug then
        debug_output = debug_output .. add_debug("After substitution: " .. result:sub(1, 200))
      end

      -- Check for new patterns (recursive expansion)
      if result:match("@{[^}]+}") then
        if config.debug then
          debug_output = debug_output .. add_debug("Found new patterns, recursing...")
        end
        M.expand(result, {
          macros = macros,
          interactive = interactive,
          test_values = test_values,
          templates = templates,
          depth = depth + 1,
          debug_output = debug_output
        }, function(expanded_result)
          -- Handle nil result (e.g., from max depth error)
          if not expanded_result then
            callback(nil)
            return
          end
          -- Restore escaped patterns only at the top level
          if depth == 0 then
            expanded_result = expanded_result:gsub("__ESCAPED_AT_BRACE__", "@{")
            if config.debug then
              expanded_result = debug_output .. add_debug("Final result after expansion") .. expanded_result
            end
          end
          callback(expanded_result)
        end)
      else
        -- Restore escaped patterns only at the top level
        if depth == 0 then
          result = result:gsub("__ESCAPED_AT_BRACE__", "@{")
          if config.debug then
            result = debug_output .. add_debug("Final result") .. result
          end
        end
        callback(result)
      end
    end
  end

  -- Resolve patterns sequentially (one at a time) for interactive mode
  -- This ensures UI prompts don't interfere with each other
  local current_index = 1

  local function resolve_next()
    if current_index > #parsed.patterns then
      -- All done, check if complete
      return
    end

    local pattern = parsed.patterns[current_index]
    if config.debug then
      debug_output = debug_output .. add_debug("Resolving pattern " .. current_index .. ": " .. pattern.type .. " - " .. (pattern.args or ""))
    end
    resolver.resolve_async(pattern, interactive, function(value)
      if config.debug then
        debug_output = debug_output .. add_debug("  Resolved to: " .. (value and value:sub(1, 100) or "nil"))
      end
      resolved[current_index] = value
      current_index = current_index + 1
      check_complete()
      -- Resolve next pattern after this one completes
      if current_index <= #parsed.patterns then
        vim.schedule(resolve_next)
      end
    end)
  end

  -- Start resolving the first pattern
  if #parsed.patterns > 0 then
    resolve_next()
  else
    -- No patterns to resolve
    callback(parsed.text)
  end
end

-- Expand template from file
function M.expand_from_file(filepath, template_name, options)
  local template_data = loader.get_template(filepath, template_name)
  if not template_data then
    error("Template not found: " .. template_name)
  end

  -- Get macros from the same file
  local macros = loader.get_macros(filepath)

  options = options or {}
  options.macros = macros

  return M.expand(template_data.content, options)
end

-- List available templates
function M.list_templates(filepath)
  return loader.list_templates(filepath)
end

-- Interactive template selector
function M.select_and_expand(filepath)
  local templates = M.list_templates(filepath)
  if #templates == 0 then
    vim.notify("No templates found in " .. filepath, vim.log.levels.WARN)
    return
  end

  local items = {}
  for _, t in ipairs(templates) do
    table.insert(items, t.name .. " - " .. t.description)
  end

  vim.ui.select(items, {
    prompt = "Select template: "
  }, function(choice, idx)
    if choice and idx then
      local template = templates[idx]
      local expanded = M.expand_from_file(filepath, template.name)
      M.insert_at_cursor(expanded)
    end
  end)
end

-- Insert text at cursor position
function M.insert_at_cursor(text)
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

-- Test function for headless expansion
function M.test(text, test_values, macros)
  return M.expand(text, {
    interactive = false,
    test_values = test_values,
    macros = macros or {}
  })
end

-- Find template files
function M.find_template_files()
  local files = {}

  for _, dir in ipairs(config.template_dirs) do
    local expanded = vim.fn.expand(dir)
    if vim.fn.isdirectory(expanded) == 1 then
      local pattern = expanded .. "*.sqlt"
      local found = vim.fn.glob(pattern, false, true)
      for _, file in ipairs(found) do
        table.insert(files, file)
      end
    end
  end

  return files
end

-- Configure template directories
function M.setup(opts)
  if opts.template_dirs then
    config.template_dirs = opts.template_dirs
  end
  if opts.max_depth then
    config.max_depth = opts.max_depth
  end
  if opts.debug ~= nil then
    config.debug = opts.debug
  end
end

-- Set debug mode
function M.set_debug(enabled)
  config.debug = enabled
end

-- Get debug mode status
function M.get_debug()
  return config.debug
end

return M
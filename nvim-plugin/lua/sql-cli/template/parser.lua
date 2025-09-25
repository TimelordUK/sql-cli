-- Template Parser Module
-- Handles parsing of @{TYPE:args} patterns and macro definitions

local M = {}

-- Parse @{TYPE:args} patterns in text
-- Returns: { patterns = {...}, text = "text with placeholders" }
function M.parse_patterns(text)
  local patterns = {}
  local index = 1
  local result_text = text
  local replacements = {}

  -- First pass: handle escaped patterns ($$@{...} -> preserve literally)
  -- Replace $$@{ with a temporary marker to prevent expansion
  local escape_marker = "__ESCAPED_AT_BRACE__"
  result_text = result_text:gsub("%$%$@{", escape_marker)

  -- Process the text character by character to handle quotes properly
  local pos = 1
  while pos <= #result_text do
    -- Skip escaped markers
    if result_text:sub(pos, pos + #escape_marker - 1) == escape_marker then
      pos = pos + #escape_marker
      goto continue
    end

    -- Check for escaped quote before pattern
    local esc_quote_pattern = result_text:match('^(\\"@{[^}]+}\\")', pos)
    local quote_pattern = result_text:match('^("@{[^}]+}")', pos)
    local bare_pattern = result_text:match('^(@{[^}]+})', pos)

    local match, has_escaped_quotes, has_quotes

    if esc_quote_pattern then
      match = esc_quote_pattern:sub(3, -3)  -- Remove \" from both ends
      has_escaped_quotes = true
      has_quotes = false
      pos = pos + #esc_quote_pattern
    elseif quote_pattern then
      match = quote_pattern:sub(2, -2)  -- Remove " from both ends
      has_escaped_quotes = false
      has_quotes = true
      pos = pos + #quote_pattern
    elseif bare_pattern then
      match = bare_pattern
      has_escaped_quotes = false
      has_quotes = false
      pos = pos + #bare_pattern
    else
      pos = pos + 1
      goto continue
    end

    -- Parse the @{TYPE:args} pattern
    local func_type, args = match:match('^@{([^:}]+):?([^}]*)}$')
    if func_type then
      -- Handle LITERAL type - preserves the content as-is
      if func_type == "LITERAL" then
        local full_match = has_escaped_quotes and ('\\"' .. match .. '\\"') or
                          has_quotes and ('"' .. match .. '"') or match
        -- Don't create a pattern for LITERAL, just replace with the content
        table.insert(replacements, {
          original = full_match,
          replacement = args  -- Return the literal content
        })
        goto continue
      end

      local placeholder = "__PATTERN_" .. index .. "__"

      -- Split args by colon
      local arg_list = {}
      if args and args ~= "" then
        for arg in args:gmatch("[^:]+") do
          table.insert(arg_list, arg:match("^%s*(.-)%s*$"))
        end
      end

      local pattern_info = {
        match = match,
        full_match = has_escaped_quotes and ('\\"' .. match .. '\\"') or
                     has_quotes and ('"' .. match .. '"') or match,
        type = func_type,
        args = args,
        arg_list = arg_list,
        placeholder = placeholder,
        has_escaped_quotes = has_escaped_quotes,
        has_quotes = has_quotes,
        index = index
      }

      table.insert(patterns, pattern_info)
      table.insert(replacements, {
        original = pattern_info.full_match,
        replacement = has_escaped_quotes and ('\\"' .. placeholder .. '\\"') or
                     has_quotes and ('"' .. placeholder .. '"') or placeholder
      })

      index = index + 1
    end

    ::continue::
  end

  -- Apply replacements
  for _, r in ipairs(replacements) do
    -- Escape special regex characters in the original text
    local escaped = r.original:gsub("([%^%$%(%)%%%.%[%]%*%+%-%?])", "%%%1")
    result_text = result_text:gsub(escaped, r.replacement)
  end

  -- Don't restore escaped patterns yet - leave them as markers
  -- They will be restored at the very end after all expansions

  return {
    patterns = patterns,
    text = result_text
  }
end

-- Parse file for macro definitions
function M.parse_macros(lines)
  lines = lines or vim.api.nvim_buf_get_lines(0, 0, -1, false)

  local macros = {}
  local current_macro = nil
  local macro_lines = {}

  for _, line in ipairs(lines) do
    -- Check for macro start
    local macro_name = line:match("^%s*%-%-+%s*MACRO:%s*([%w_]+)")
    if macro_name then
      -- Save previous macro if exists
      if current_macro then
        macros[current_macro] = table.concat(macro_lines, "\n")
      end
      current_macro = macro_name
      macro_lines = {}
    elseif line:match("^%s*%-%-+%s*END%s+MACRO") then
      -- Save current macro
      if current_macro then
        macros[current_macro] = table.concat(macro_lines, "\n")
        current_macro = nil
        macro_lines = {}
      end
    elseif current_macro then
      -- Accumulate macro lines (skip comment prefix)
      local content = line:match("^%s*%-%-+%s*(.*)") or line
      table.insert(macro_lines, content)
    end
  end

  -- Parse CONFIG macro specially
  if macros.CONFIG then
    local config = {}
    for line in macros.CONFIG:gmatch("[^\n]+") do
      local key, value = line:match("^%s*([%w_]+)%s*=%s*(.-)%s*$")
      if key and value then
        config[key] = value
      end
    end
    macros._CONFIG = config
  end

  return macros
end

return M
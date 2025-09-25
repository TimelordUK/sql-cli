-- Template Loader Module
-- Loads and manages template files

local M = {}
local parser = require('sql-cli.template.parser')

-- Cache for loaded templates
local template_cache = {}
local macro_cache = {}

-- Load a template file
function M.load_file(filepath)
  local file = io.open(filepath, "r")
  if not file then
    return nil, "Could not open template file: " .. filepath
  end

  local content = file:read("*all")
  file:close()

  return M.parse_content(content)
end

-- Parse template content (supports both -- and ## markers)
function M.parse_content(content)
  local templates = {}
  local macros = {}
  local current_section = nil
  local section_name = nil
  local section_lines = {}
  local section_type = nil
  local description = nil
  local in_macro = false

  for line in content:gmatch("[^\n]+") do
    -- Support both SQL comments (--) and markdown style (##)
    -- Check for template start
    local template_match = line:match("^%-%-+%s*Template:%s+(.+)") or
                          line:match("^##%s+Template:%s+(.+)")
    if template_match then
      -- Save previous section
      if current_section and section_type == "template" then
        templates[current_section] = {
          name = current_section,
          description = description,
          content = table.concat(section_lines, "\n")
        }
      elseif current_section and section_type == "macro" then
        macros[current_section] = table.concat(section_lines, "\n")
      end

      section_name = template_match
      current_section = section_name
      section_type = "template"
      section_lines = {}
      description = nil
      in_macro = false

    -- Check for macro start
    elseif line:match("^%-%-+%s*Macro:%s+(.+)") or line:match("^##%s+Macro:%s+(.+)") then
      -- Save previous section
      if current_section and section_type == "template" then
        templates[current_section] = {
          name = current_section,
          description = description,
          content = table.concat(section_lines, "\n")
        }
      elseif current_section and section_type == "macro" then
        macros[current_section] = table.concat(section_lines, "\n")
      end

      section_name = line:match("^%-%-+%s*Macro:%s+(.+)") or
                    line:match("^##%s+Macro:%s+(.+)")
      current_section = section_name
      section_type = "macro"
      section_lines = {}
      in_macro = true

    -- Check for description
    elseif (line:match("^%-%-+%s*Description:%s+(.+)") or
            line:match("^##%s+Description:%s+(.+)")) and section_type == "template" then
      description = line:match("^%-%-+%s*Description:%s+(.+)") or
                   line:match("^##%s+Description:%s+(.+)")

    -- Check for END MACRO marker
    elseif line:match("^%-%-+%s*END%s+MACRO") and in_macro then
      if current_section and section_type == "macro" then
        macros[current_section] = table.concat(section_lines, "\n")
      end
      current_section = nil
      section_type = nil
      section_lines = {}
      in_macro = false

    -- Skip standalone comment lines (but not in macros/templates)
    elseif line:match("^#[^#]") or line:match("^#$") then
      -- Skip regular markdown comments

    -- For macro content, strip leading comment markers if present
    elseif current_section and section_type == "macro" then
      -- Strip SQL comment prefix for macro lines
      local content = line:match("^%-%-+%s*(.*)") or line
      table.insert(section_lines, content)

    -- Add content lines for templates as-is
    elseif current_section then
      table.insert(section_lines, line)
    end
  end

  -- Save last section
  if current_section and section_type == "template" then
    templates[current_section] = {
      name = current_section,
      description = description,
      content = table.concat(section_lines, "\n")
    }
  elseif current_section and section_type == "macro" then
    macros[current_section] = table.concat(section_lines, "\n")
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

  return {
    templates = templates,
    macros = macros
  }
end

-- Get all available templates
function M.list_templates(filepath)
  local data = M.load_file(filepath)
  if not data then
    return {}
  end

  local list = {}
  for name, template in pairs(data.templates) do
    table.insert(list, {
      name = name,
      description = template.description or "No description"
    })
  end

  table.sort(list, function(a, b) return a.name < b.name end)
  return list
end

-- Get a specific template
function M.get_template(filepath, template_name)
  if not template_cache[filepath] then
    local data = M.load_file(filepath)
    if data then
      template_cache[filepath] = data.templates
      macro_cache[filepath] = data.macros
    end
  end

  if template_cache[filepath] then
    return template_cache[filepath][template_name]
  end
  return nil
end

-- Get macros from template file
function M.get_macros(filepath)
  if not macro_cache[filepath] then
    local data = M.load_file(filepath)
    if data then
      template_cache[filepath] = data.templates
      macro_cache[filepath] = data.macros
    end
  end

  return macro_cache[filepath] or {}
end

-- Clear cache
function M.clear_cache()
  template_cache = {}
  macro_cache = {}
end

return M
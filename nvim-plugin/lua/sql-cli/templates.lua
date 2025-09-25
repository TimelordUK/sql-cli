-- Redirect to new template module
-- This file redirects the old templates module to the new template system

local new_template = require('sql-cli.template')

local M = {}

-- Redirect setup
function M.setup(config)
  return new_template.setup(config)
end

-- Redirect keymap setup (called by main init.lua)
function M.setup_keymaps()
  new_template.create_commands()
  new_template.create_keymaps()
end

-- Keep any legacy functions that might be called
function M.select_template()
  return new_template.select_template()
end

function M.expand_inline_macro()
  return new_template.expand_at_cursor()
end

-- Add any other legacy function redirects as needed

return M
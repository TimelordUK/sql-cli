-- SQL CLI Plugin initialization
-- This file is automatically loaded by Neovim

if vim.g.loaded_sql_cli then
  return
end
vim.g.loaded_sql_cli = true

-- Setup with default configuration
-- Users can call setup() again with custom config
require('sql-cli').setup()
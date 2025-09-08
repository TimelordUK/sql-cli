-- SQL CLI Plugin initialization
-- This file is automatically loaded by Neovim

if vim.g.loaded_sql_cli then
  return
end
vim.g.loaded_sql_cli = true

-- Don't auto-setup, let the user call it from their config
-- This prevents timing issues with lazy.nvim
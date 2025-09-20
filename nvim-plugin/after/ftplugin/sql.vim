" SQL file type plugin additions for SQL CLI integration
" This file is loaded after the standard SQL ftplugin

" Set comment string for SQL files
setlocal commentstring=--\ %s

" Disable default keywordprg behavior to prevent conflicts
setlocal keywordprg=

" Enable SQL CLI keymaps if plugin is loaded
if exists('g:loaded_sql_cli')
  " Buffer-local keymaps for SQL files
  nnoremap <buffer> <LocalLeader>r :SqlCliExecute<CR>
  vnoremap <buffer> <LocalLeader>r :SqlCliExecute<CR>
  nnoremap <buffer> <LocalLeader>p :SqlCliShowPlan<CR>

  " Map K to show function/generator help
  " Access the config directly from the module
  nnoremap <buffer> <silent> K :lua require('sql-cli.functions').function_help_at_cursor(require('sql-cli').config)<CR>
endif
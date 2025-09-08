" SQL file type plugin additions for SQL CLI integration
" This file is loaded after the standard SQL ftplugin

" Set comment string for SQL files
setlocal commentstring=--\ %s

" Enable SQL CLI keymaps if plugin is loaded
if exists('g:loaded_sql_cli')
  " Buffer-local keymaps for SQL files
  nnoremap <buffer> <LocalLeader>r :SqlCliExecute<CR>
  vnoremap <buffer> <LocalLeader>r :SqlCliExecute<CR>
  nnoremap <buffer> <LocalLeader>p :SqlCliShowPlan<CR>
endif
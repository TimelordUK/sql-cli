" SQL CLI Output Syntax File
" Language: SQL CLI Output
" Maintainer: SQL CLI Team

if exists("b:current_syntax")
  finish
endif

" Headers and separators
syn match sqlCliHeader /^--.*$/
syn match sqlCliSeparator /^-\+$/

" Table structure - Unicode box drawing characters
syn match sqlCliTableBorder /[│├┤┌┐└┘─┬┴┼]/

" Table pipes
syn match sqlCliTablePipe /|/

" Numbers (integers and floats)
syn match sqlCliNumber /\<\d\+\(\.\d\+\)\?\>/

" NULL values
syn match sqlCliNull /\<NULL\>/

" Boolean values
syn match sqlCliBoolean /\<\(true\|false\|TRUE\|FALSE\)\>/

" Trading-specific highlighting
syn match sqlCliBuy /\<Buy\>/
syn match sqlCliSell /\<Sell\>/

" Instrument types
syn match sqlCliInstrumentNDS /\<NDS\>/
syn match sqlCliInstrumentNFD /\<NFD\>/
syn match sqlCliInstrumentCDS /\<CDS\>/
syn match sqlCliInstrumentIRS /\<IRS\>/

" Error messages
syn match sqlCliError /^ERROR:.*$/

" Exit code
syn match sqlCliExitCode /^-- Exit code:.*$/

" Quoted strings (CSV values)
syn region sqlCliString start=/"/ end=/"/ skip=/\\"/
syn region sqlCliString start=/'/ end=/'/ skip=/\\'/

" JSON-like structures
syn match sqlCliJsonKey /"\w\+":/

" Define default highlighting
hi def link sqlCliHeader Comment
hi def link sqlCliSeparator NonText
hi def link sqlCliTableBorder Special
hi def link sqlCliTablePipe Special
hi def link sqlCliNumber Number
hi def link sqlCliNull Constant
hi def link sqlCliBoolean Boolean
hi def link sqlCliError Error
hi def link sqlCliExitCode Statement
hi def link sqlCliString String
hi def link sqlCliJsonKey Identifier

" Trading-specific colors
hi def sqlCliBuy guifg=#00ff00 ctermfg=Green gui=bold
hi def sqlCliSell guifg=#ff0000 ctermfg=Red gui=bold

" Instrument type colors
hi def sqlCliInstrumentNDS guifg=#00aaff ctermfg=Cyan
hi def sqlCliInstrumentNFD guifg=#ffaa00 ctermfg=Yellow
hi def sqlCliInstrumentCDS guifg=#aa00ff ctermfg=Magenta
hi def sqlCliInstrumentIRS guifg=#00ffaa ctermfg=Green

let b:current_syntax = "sql-cli-output"
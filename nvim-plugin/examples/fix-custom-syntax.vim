" Custom FIX Message Syntax File
" This can be sourced after the plugin loads to add FIX-specific highlighting
"
" Usage: Place in ~/.config/nvim/after/syntax/sql-cli-output.vim
"        Or source manually: :source /path/to/this/file

" FIX Message Types (resolved from tag 35)
syn keyword fixMsgTypeExecution ExecutionReport
syn keyword fixMsgTypeAllocation AllocationReport AllocationInstruction
syn keyword fixMsgTypeOrder NewOrderSingle OrderCancelRequest OrderCancelReject
syn keyword fixMsgTypeQuote Quote QuoteRequest QuoteCancel
syn keyword fixMsgTypeReject Reject BusinessMessageReject

" Order Status (tag 39) - resolved enum values
syn keyword fixOrderStatusPending PendingNew PendingCancel PendingReplace
syn keyword fixOrderStatusActive New PartiallyFilled
syn keyword fixOrderStatusComplete Filled DoneForDay
syn keyword fixOrderStatusInactive Canceled Stopped Rejected Expired

" Side (tag 54) - Buy/Sell
syn keyword fixSideBuy Buy
syn keyword fixSideSell Sell
syn keyword fixSideOther Cross BuyMinus SellPlus

" Execution Type (tag 150)
syn keyword fixExecTypeNew New PartialFill Fill
syn keyword fixExecTypeCancel Canceled Replaced
syn keyword fixExecTypeReject Rejected Suspended

" Time in Force (tag 59)
syn keyword fixTimeInForce Day GTC IOC FOK GTD

" Order Type (tag 40)
syn keyword fixOrderType Market Limit Stop StopLimit

" Exchange codes (common exchanges)
syn match fixExchangeUS /\<\(NYSE\|NASDAQ\|AMEX\|ARCA\)\>/
syn match fixExchangeEU /\<\(LSE\|XETRA\|EURONEXT\)\>/
syn match fixExchangeAPAC /\<\(HKEX\|TSE\|SGX\)\>/

" FIX Tag numbers (if showing raw tag=value pairs)
" Common tags with context highlighting
syn match fixTagMsgType /\<35=\w\+/
syn match fixTagSide /\<54=[12]/
syn match fixTagSymbol /\<55=\w\+/
syn match fixTagOrdStatus /\<39=\w/
syn match fixTagExecType /\<150=\w/

" Securities/Instruments
syn keyword fixInstrumentType NDS NFD CDS IRS FRA

" Numeric field highlighting (prices, quantities)
syn match fixPrice /\<Price:\s*\d\+\.\d\+/
syn match fixQuantity /\<Qty:\s*\d\+/

" Define colors for FIX message types
hi def fixMsgTypeExecution guifg=#50fa7b ctermfg=Green gui=bold
hi def fixMsgTypeAllocation guifg=#8be9fd ctermfg=Cyan gui=bold
hi def fixMsgTypeOrder guifg=#f1fa8c ctermfg=Yellow gui=bold
hi def fixMsgTypeQuote guifg=#bd93f9 ctermfg=Magenta gui=bold
hi def fixMsgTypeReject guifg=#ff5555 ctermfg=Red gui=bold

" Order Status colors
hi def fixOrderStatusPending guifg=#f1fa8c ctermfg=Yellow
hi def fixOrderStatusActive guifg=#50fa7b ctermfg=Green
hi def fixOrderStatusComplete guifg=#50fa7b ctermfg=Green gui=bold
hi def fixOrderStatusInactive guifg=#6272a4 ctermfg=DarkGray

" Side colors
hi def fixSideBuy guifg=#50fa7b ctermfg=Green gui=bold
hi def fixSideSell guifg=#ff5555 ctermfg=Red gui=bold
hi def fixSideOther guifg=#bd93f9 ctermfg=Magenta

" Execution type colors
hi def fixExecTypeNew guifg=#50fa7b ctermfg=Green
hi def fixExecTypeCancel guifg=#ff5555 ctermfg=Red
hi def fixExecTypeReject guifg=#ff5555 ctermfg=Red gui=bold

" Time in Force
hi def fixTimeInForce guifg=#8be9fd ctermfg=Cyan

" Order Type
hi def fixOrderType guifg=#bd93f9 ctermfg=Magenta

" Exchange highlighting
hi def fixExchangeUS guifg=#bd93f9 ctermfg=Magenta
hi def fixExchangeEU guifg=#ff79c6 ctermfg=Magenta
hi def fixExchangeAPAC guifg=#ffb86c ctermfg=Yellow

" Tag highlighting
hi def fixTagMsgType guifg=#f1fa8c ctermfg=Yellow gui=bold
hi def fixTagSide guifg=#8be9fd ctermfg=Cyan gui=bold
hi def fixTagSymbol guifg=#bd93f9 ctermfg=Magenta gui=bold
hi def fixTagOrdStatus guifg=#50fa7b ctermfg=Green gui=bold
hi def fixTagExecType guifg=#ffb86c ctermfg=Yellow gui=bold

" Instrument types
hi def fixInstrumentType guifg=#8be9fd ctermfg=Cyan

" Numeric fields
hi def fixPrice guifg=#bd93f9 ctermfg=Magenta
hi def fixQuantity guifg=#ffb86c ctermfg=Yellow

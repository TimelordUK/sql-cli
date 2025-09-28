# PowerShell Token Script Execution Options

## Option 1: Using -Command with & operator (RECOMMENDED)
```powershell
powershell.exe -NoProfile -Command "& $HOME\dev\sql-cli\ExportJwt.ps1"
```

## Option 2: Using -ExecutionPolicy Bypass with -File
```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "$HOME\dev\sql-cli\ExportJwt.ps1"
```

## For nvim config, use double backslashes:

### Option 1 in Lua (RECOMMENDED):
```lua
command = 'powershell.exe -NoProfile -Command "& $HOME\\dev\\sql-cli\\ExportJwt.ps1"'
```

### Option 2 in Lua:
```lua
command = 'powershell.exe -NoProfile -ExecutionPolicy Bypass -File "$HOME\\dev\\sql-cli\\ExportJwt.ps1"'
```

## Why the & operator is needed:
- In PowerShell, when using `-Command`, strings are interpreted as commands to execute
- Without `&`, PowerShell treats the path as a string literal, not a script to run
- The `&` (call operator) tells PowerShell to execute the script at that path

## Why -ExecutionPolicy Bypass works:
- Temporarily overrides the system's script execution policy for this single command
- Only affects the PowerShell instance launched by this command
- Does not permanently change system settings
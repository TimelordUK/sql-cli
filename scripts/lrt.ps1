# lrt - most recently modified files, an `ls -alrt` built on sql-cli.
#
# Load it from your PowerShell profile:
#   . C:\path\to\sql-cli\scripts\lrt.ps1
#
# Usage:
#   lrt                  # 20 newest files in the current directory
#   lrt 50 src -r        # 50 newest under src, recursive
#   lrt -g '*.log'       # only files matching a glob
#   lrt -d               # include directories
#   lrt -style utf8      # any style from `sql-cli --list-table-styles`
#
# The default style is `plain` (no borders). It needs a sql-cli built with the
# ANSI width fix (after v1.85.4); on older builds coloured cells misalign in
# every style except `default`, so use `lrt -style default` there.
function lrt {
    param(
        [int]$n = 20,
        [string]$path = '.',
        [switch]$r,
        [string]$g,
        [switch]$d,
        [string]$style = 'plain'
    )
    $p    = $path -replace "'", "''"
    $opts = @()
    if ($r) { $opts += 'RECURSIVE' }
    if ($g) { $opts += "GLOB '$($g -replace "'", "''")'" }
    $dirFilter = if ($d) { '' } else { 'AND is_dir = false' }
    # recursive: show the path relative to the root, not the full \\?\C:\... path
    $root    = ('\\?\' + (Resolve-Path $path).ProviderPath.TrimEnd('\') + '\') -replace "'", "''"
    $nameCol = if ($r) { "REPLACE(path, '$root', '')" } else { 'name' }
    # The date functions work in UTC; shift to local time for display. This uses
    # today's offset, so files from the other side of a DST change show an hour out.
    $off = [int][TimeZoneInfo]::Local.GetUtcOffset((Get-Date)).TotalMinutes

    $sql = @"
WITH f AS (FILE PATH '$p' $($opts -join ' ')),
g AS (
  SELECT $nameCol AS name, ext, size, is_dir,
    DATEADD('minute', $off, modified) AS lm,
    DATEDIFF('second', modified, NOW()) AS secs
  FROM f WHERE depth > 0 $dirFilter
)
SELECT
  FORMAT_DATE(lm, '%Y-%m-%d') AS date,
  FORMAT_DATE(lm, '%H:%M:%S') AS time,
  CASE
    WHEN secs < 60     THEN ANSI_COLOR('bright_green', secs || 's')
    WHEN secs < 3600   THEN ANSI_COLOR('green',  FLOOR(secs / 60) || 'm')
    WHEN secs < 86400  THEN ANSI_COLOR('yellow', FLOOR(secs / 3600) || 'h')
    WHEN secs < 604800 THEN FLOOR(secs / 86400) || 'd'
    ELSE ANSI_COLOR('bright_black', FLOOR(secs / 86400) || 'd')
  END AS age,
  IIF(is_dir, '<dir>', FORMAT_BYTES(size)) AS size,
  CASE
    WHEN is_dir THEN ANSI_COLOR('bright_blue', name)
    WHEN ext IN ('rs', 'py', 'sql', 'ps1', 'cs', 'lua') THEN ANSI_COLOR('cyan', name)
    WHEN ext IN ('md', 'txt') THEN ANSI_COLOR('magenta', name)
    WHEN ext IN ('log', 'csv', 'json') THEN ANSI_COLOR('yellow', name)
    WHEN ext IN ('exe', 'dll', 'zip') THEN ANSI_COLOR('red', name)
    ELSE name
  END AS name
FROM g
ORDER BY secs ASC
LIMIT $n
"@
    # The "# Query completed" footer goes to stderr
    sql-cli -q $sql -o table --table-style $style --max-col-width 0 2>$null
}

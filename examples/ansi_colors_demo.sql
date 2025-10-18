--
-- ANSI Terminal Colors Demo
-- Demonstrates colorful terminal output using ANSI escape codes
--
-- Run with: sql-cli -f examples/ansi_colors_demo.sql
-- Or in Neovim: \sq
--

-- Basic Named Colors
SELECT
    ANSI_COLOR('red', 'ERROR') as red_text,
    ANSI_COLOR('green', 'SUCCESS') as green_text,
    ANSI_COLOR('yellow', 'WARNING') as yellow_text,
    ANSI_COLOR('blue', 'INFO') as blue_text,
    ANSI_COLOR('magenta', 'DEBUG') as magenta_text,
    ANSI_COLOR('cyan', 'TRACE') as cyan_text
;
GO

-- Bright Color Variants
SELECT
    ANSI_COLOR('bright_red', 'CRITICAL') as bright_red,
    ANSI_COLOR('bright_green', 'PASSED') as bright_green,
    ANSI_COLOR('bright_yellow', 'PENDING') as bright_yellow,
    ANSI_COLOR('bright_blue', 'NOTE') as bright_blue,
    ANSI_COLOR('bright_magenta', 'VERBOSE') as bright_magenta,
    ANSI_COLOR('bright_cyan', 'DETAIL') as bright_cyan
;
GO

-- Background Colors
SELECT
    ANSI_BG('red', ' ALERT ') as red_bg,
    ANSI_BG('green', ' ACTIVE ') as green_bg,
    ANSI_BG('yellow', ' CAUTION ') as yellow_bg,
    ANSI_BG('blue', ' INFO ') as blue_bg
;
GO

-- RGB Colors (True Color)
SELECT
    ANSI_RGB(255, 0, 0, 'Pure Red') as pure_red,
    ANSI_RGB(0, 255, 0, 'Pure Green') as pure_green,
    ANSI_RGB(0, 0, 255, 'Pure Blue') as pure_blue,
    ANSI_RGB(255, 165, 0, 'Orange') as orange,
    ANSI_RGB(255, 192, 203, 'Pink') as pink,
    ANSI_RGB(138, 43, 226, 'Blue Violet') as blue_violet
;
GO

-- Text Formatting
SELECT
    ANSI_BOLD('Bold Text') as bold,
    ANSI_ITALIC('Italic Text') as italic,
    ANSI_UNDERLINE('Underlined Text') as underline,
    ANSI_REVERSE('Reversed') as reversed,
    ANSI_STRIKETHROUGH('Deprecated') as strikethrough
;
GO

-- Practical Example: Status Report
SELECT
    'Status Report' as category,
    ANSI_COLOR('bright_green', '✓') || ' ' || 'All tests passed' as status
UNION ALL
SELECT
    'Error Count',
    ANSI_COLOR('bright_red', '0') || ' errors'
UNION ALL
SELECT
    'Warning Count',
    ANSI_COLOR('yellow', '3') || ' warnings'
UNION ALL
SELECT
    'Build Status',
    ANSI_BG('green', ' SUCCESS ') || ' Build completed'
;
GO

-- Color Gradient (RGB)
SELECT
    ANSI_RGB(255, 0, 0, '██') as r255,
    ANSI_RGB(200, 55, 0, '██') as r200,
    ANSI_RGB(150, 105, 0, '██') as r150,
    ANSI_RGB(100, 155, 0, '██') as r100,
    ANSI_RGB(50, 205, 0, '██') as r50,
    ANSI_RGB(0, 255, 0, '██') as g255
;
GO

-- Using with data (example)
SELECT 1 as value
;
GO

WITH data AS (
    SELECT 1 as id, 'Active' as status, 95.5 as score
    UNION ALL
    SELECT 2, 'Warning', 65.3
    UNION ALL
    SELECT 3, 'Error', 45.1
)
SELECT
    id,
    CASE
        WHEN status = 'Active' THEN ANSI_COLOR('green', status)
        WHEN status = 'Warning' THEN ANSI_COLOR('yellow', status)
        WHEN status = 'Error' THEN ANSI_COLOR('red', status)
        ELSE status
    END as colored_status,
    CASE
        WHEN score >= 90 THEN ANSI_COLOR('bright_green', score)
        WHEN score >= 70 THEN ANSI_COLOR('yellow', score)
        ELSE ANSI_COLOR('red', score)
    END as colored_score
FROM data
;
GO

-- Combining colors and formatting
SELECT
    ANSI_BOLD(ANSI_COLOR('red', 'CRITICAL ERROR')) as critical,
    ANSI_ITALIC(ANSI_COLOR('blue', 'Note: Check logs')) as note,
    ANSI_UNDERLINE(ANSI_COLOR('green', 'COMPLETED')) as completed
;
GO

-- Rainbow Effect
SELECT
    ANSI_RGB(255, 0, 0, '●') || ' ' ||
    ANSI_RGB(255, 127, 0, '●') || ' ' ||
    ANSI_RGB(255, 255, 0, '●') || ' ' ||
    ANSI_RGB(0, 255, 0, '●') || ' ' ||
    ANSI_RGB(0, 0, 255, '●') || ' ' ||
    ANSI_RGB(75, 0, 130, '●') || ' ' ||
    ANSI_RGB(148, 0, 211, '●') as rainbow
;
GO

-- Available Named Colors Reference
SELECT
    'Named Colors' as category,
    'black, red, green, yellow, blue, magenta, cyan, white' as basic_colors,
    'bright_* variants for all basic colors, plus gray/grey' as bright_colors
;
GO

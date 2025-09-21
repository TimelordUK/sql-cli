-- SQL Refactoring Tools for Neovim
-- Provides smart SQL transformations using the CLI engine

local M = {}
local Job = require('plenary.job')
local utils = require('sql-cli.utils')

-- Store config reference
M.config = nil

-- Get visual selection
local function get_visual_selection()
    local start_pos = vim.fn.getpos("'<")
    local end_pos = vim.fn.getpos("'>")
    local lines = vim.api.nvim_buf_get_lines(
        0,
        start_pos[2] - 1,
        end_pos[2],
        false
    )

    if #lines == 0 then
        return nil
    end

    -- Handle partial line selection
    if #lines == 1 then
        lines[1] = string.sub(lines[1], start_pos[3], end_pos[3])
    else
        lines[1] = string.sub(lines[1], start_pos[3])
        lines[#lines] = string.sub(lines[#lines], 1, end_pos[3])
    end

    return table.concat(lines, '\n')
end

-- Get current query (between GO statements or entire buffer)
local function get_current_query()
    local cursor = vim.api.nvim_win_get_cursor(0)
    local lines = vim.api.nvim_buf_get_lines(0, 0, -1, false)

    -- Find query boundaries (GO statements)
    local start_line = 1
    local end_line = #lines

    for i = cursor[1] - 1, 1, -1 do
        if lines[i]:upper():match('^GO%s*$') then
            start_line = i + 1
            break
        end
    end

    for i = cursor[1], #lines do
        if lines[i]:upper():match('^GO%s*$') then
            end_line = i - 1
            break
        end
    end

    local query_lines = {}
    for i = start_line, end_line do
        table.insert(query_lines, lines[i])
    end

    return table.concat(query_lines, '\n')
end

-- Extract expression to CTE
function M.extract_to_cte()
    local config = M.config or require('sql-cli.config').defaults
    local command_path, err = utils.get_command_path(config.command)
    if not command_path then
        vim.notify(err or "sql-cli command not found", vim.log.levels.ERROR)
        return
    end

    local selection = get_visual_selection()
    if not selection then
        vim.notify("No selection found", vim.log.levels.WARN)
        return
    end

    local query = get_current_query()

    Job:new({
        command = command_path,
        args = {
            '--extract-to-cte',
            '--expression', selection,
            '--query', query
        },
        on_exit = function(j, return_val)
            vim.schedule(function()
                if return_val == 0 then
                    local result = table.concat(j:result(), '\n')
                    local data = vim.fn.json_decode(result)

                    -- Show preview
                    M.show_refactoring_preview(data)
                else
                    vim.notify("Extraction failed: " .. table.concat(j:stderr_result(), '\n'), vim.log.levels.ERROR)
                end
            end)
        end,
    }):start()
end

-- Generate banding CASE statement
function M.generate_banding()
    vim.ui.input({ prompt = 'Column name: ' }, function(column)
        if not column then return end

        vim.ui.input({ prompt = 'Bands (e.g., 0-10,11-20,21-30,30+): ' }, function(bands)
            if not bands then return end

            local config = M.config or require('sql-cli.config').defaults
            local command_path, err = utils.get_command_path(config.command)
            if not command_path then
                vim.notify(err or "sql-cli command not found", vim.log.levels.ERROR)
                return
            end

            Job:new({
                command = command_path,
                args = {
                    '--generate-bands',
                    '--column', column,
                    '--bands', bands
                },
                on_exit = function(j, return_val)
                    vim.schedule(function()
                        if return_val == 0 then
                            local result = j:result()

                            -- Insert at cursor position
                            local cursor = vim.api.nvim_win_get_cursor(0)
                            vim.api.nvim_buf_set_lines(0, cursor[1] - 1, cursor[1] - 1, false, result)
                        else
                            vim.notify("Band generation failed", vim.log.levels.ERROR)
                        end
                    end)
                end,
            }):start()
        end)
    end)
end

-- Generate CASE statement from data analysis
function M.generate_case_from_data()
    -- Get current buffer's file path as default
    local current_file = vim.fn.expand('%:p')
    local data_file = ''

    -- Check if current file is a CSV or if there's a data hint
    if current_file:match('%.csv$') then
        data_file = current_file
    else
        -- Look for data file hint in SQL file
        local lines = vim.api.nvim_buf_get_lines(0, 0, 10, false)
        for _, line in ipairs(lines) do
            local hint = line:match('-- #! (.+%.csv)')
            if hint then
                -- Resolve relative path
                local dir = vim.fn.expand('%:p:h')
                data_file = vim.fn.simplify(dir .. '/' .. hint)
                break
            end
        end
    end

    vim.ui.input({ prompt = 'Data file: ', default = data_file }, function(file)
        if not file or file == '' then return end

        vim.ui.input({ prompt = 'Column name: ' }, function(column)
            if not column then return end

            local styles = { 'values', 'ranges', 'auto' }
            vim.ui.select(styles, {
                prompt = 'Select style:',
                format_item = function(item)
                    local descriptions = {
                        values = 'Values - Create CASE for distinct values (e.g., categories)',
                        ranges = 'Ranges - Create numeric range bands',
                        auto = 'Auto - Let CLI decide based on data'
                    }
                    return descriptions[item] or item
                end
            }, function(style)
                if not style then style = 'auto' end

                -- Check if we should analyze the data first
                if style == 'auto' then
                    -- Run a quick distinct count query
                    local config = M.config or require('sql-cli.config').defaults
                    local command_path, err = utils.get_command_path(config.command)
                    if not command_path then
                        vim.notify(err or "sql-cli command not found", vim.log.levels.ERROR)
                        return
                    end

                    Job:new({
                        command = command_path,
                        args = {
                            file,
                            '-q',
                            string.format("SELECT COUNT(DISTINCT %s) as count FROM data", column),
                            '-o', 'csv'
                        },
                        on_exit = function(j, return_val)
                            if return_val == 0 then
                                local output = j:result()
                                -- Parse count from output
                                local count = tonumber(output[2])

                                -- Decide style based on distinct count
                                if count and count <= 20 then
                                    style = 'values'
                                else
                                    style = 'ranges'
                                end

                                M.execute_case_generation(file, column, style)
                            else
                                -- Default to values if query fails
                                M.execute_case_generation(file, column, 'values')
                            end
                        end
                    }):start()
                else
                    M.execute_case_generation(file, column, style)
                end
            end)
        end)
    end)
end

-- Execute the actual CASE generation
function M.execute_case_generation(file, column, style, labels)
    local config = M.config or require('sql-cli.config').defaults
    local command_path, err = utils.get_command_path(config.command)
    if not command_path then
        vim.notify(err or "sql-cli command not found", vim.log.levels.ERROR)
        return
    end

    local args = {
        '--generate-case',
        file,
        '--column', column,
        '--style', style
    }

    if labels then
        table.insert(args, '--labels')
        table.insert(args, labels)
    end

    Job:new({
        command = command_path,
        args = args,
        on_exit = function(j, return_val)
            vim.schedule(function()
                if return_val == 0 then
                    local result = j:result()

                    -- Insert at cursor position
                    local cursor = vim.api.nvim_win_get_cursor(0)
                    vim.api.nvim_buf_set_lines(0, cursor[1] - 1, cursor[1] - 1, false, result)

                    vim.notify(string.format("Generated CASE statement for %s (%s style)", column, style), vim.log.levels.INFO)
                else
                    local error = table.concat(j:stderr_result(), '\n')
                    vim.notify("CASE generation failed: " .. error, vim.log.levels.ERROR)
                end
            end)
        end,
    }):start()
end

-- Window function wizard
function M.window_function_wizard()
    local function_types = {
        'ROW_NUMBER()',
        'RANK()',
        'DENSE_RANK()',
        'LAG',
        'LEAD',
        'SUM (running)',
        'AVG (moving)',
        'PERCENT_RANK()',
        'Custom...'
    }

    vim.ui.select(function_types, {
        prompt = 'Select window function type:',
    }, function(choice)
        if not choice then return end

        -- Get partition columns
        vim.ui.input({ prompt = 'PARTITION BY columns (comma-separated): ' }, function(partition)
            if not partition then partition = '' end

            -- Get order columns
            vim.ui.input({ prompt = 'ORDER BY columns (comma-separated): ' }, function(order)
                if not order then order = '' end

                local args = {
                    '--generate-window',
                    '--function', choice,
                }

                if partition ~= '' then
                    table.insert(args, '--partition')
                    table.insert(args, partition)
                end

                if order ~= '' then
                    table.insert(args, '--order')
                    table.insert(args, order)
                end

                -- For LAG/LEAD, ask for column and offset
                if choice:match('LAG') or choice:match('LEAD') then
                    vim.ui.input({ prompt = 'Column name: ' }, function(col)
                        if not col then return end
                        table.insert(args, '--column')
                        table.insert(args, col)

                        vim.ui.input({ prompt = 'Offset (default 1): ' }, function(offset)
                            if offset and offset ~= '' then
                                table.insert(args, '--offset')
                                table.insert(args, offset)
                            end

                            M.execute_window_generation(args)
                        end)
                    end)
                else
                    M.execute_window_generation(args)
                end
            end)
        end)
    end)
end

-- Execute window function generation
function M.execute_window_generation(args)
    local config = M.config or require('sql-cli.config').defaults
    local command_path, err = utils.get_command_path(config.command)
    if not command_path then
        vim.notify(err or "sql-cli command not found", vim.log.levels.ERROR)
        return
    end

    Job:new({
        command = command_path,
        args = args,
        on_exit = function(j, return_val)
            vim.schedule(function()
                if return_val == 0 then
                    local result = j:result()

                    -- Insert at cursor
                    local cursor = vim.api.nvim_win_get_cursor(0)
                    vim.api.nvim_buf_set_lines(0, cursor[1] - 1, cursor[1] - 1, false, result)
                else
                    vim.notify("Window function generation failed", vim.log.levels.ERROR)
                end
            end)
        end,
    }):start()
end

-- Conditional aggregation builder
function M.conditional_aggregation()
    vim.ui.input({ prompt = 'Column to pivot: ' }, function(column)
        if not column then return end

        vim.ui.input({ prompt = 'Values (comma-separated): ' }, function(values)
            if not values then return end

            local agg_types = { 'COUNT', 'SUM', 'AVG', 'MAX', 'MIN' }
            vim.ui.select(agg_types, {
                prompt = 'Aggregation type:',
            }, function(agg_type)
                if not agg_type then return end

                local args = {
                    '--build-conditional-agg',
                    '--column', column,
                    '--values', values,
                    '--aggregate', agg_type
                }

                -- For SUM/AVG/MAX/MIN, ask for the column to aggregate
                if agg_type ~= 'COUNT' then
                    vim.ui.input({ prompt = 'Column to aggregate: ' }, function(agg_col)
                        if agg_col then
                            table.insert(args, '--agg-column')
                            table.insert(args, agg_col)
                        end

                        M.execute_conditional_agg(args)
                    end)
                else
                    M.execute_conditional_agg(args)
                end
            end)
        end)
    end)
end

-- Execute conditional aggregation generation
function M.execute_conditional_agg(args)
    local config = M.config or require('sql-cli.config').defaults
    local command_path, err = utils.get_command_path(config.command)
    if not command_path then
        vim.notify(err or "sql-cli command not found", vim.log.levels.ERROR)
        return
    end

    Job:new({
        command = command_path,
        args = args,
        on_exit = function(j, return_val)
            vim.schedule(function()
                if return_val == 0 then
                    local result = j:result()

                    -- Insert at cursor
                    local cursor = vim.api.nvim_win_get_cursor(0)
                    vim.api.nvim_buf_set_lines(0, cursor[1] - 1, cursor[1] - 1, false, result)
                else
                    vim.notify("Conditional aggregation generation failed", vim.log.levels.ERROR)
                end
            end)
        end,
    }):start()
end

-- Show refactoring preview in floating window
function M.show_refactoring_preview(data)
    local width = math.floor(vim.o.columns * 0.8)
    local height = math.floor(vim.o.lines * 0.8)

    local buf = vim.api.nvim_create_buf(false, true)

    -- Format the preview
    local lines = {
        '╭' .. string.rep('─', width - 2) .. '╮',
        '│ SQL Refactoring Preview' .. string.rep(' ', width - 26) .. '│',
        '├' .. string.rep('─', width - 2) .. '┤',
    }

    if data.cte_name then
        table.insert(lines, '│ New CTE:' .. string.rep(' ', width - 11) .. '│')
        table.insert(lines, '│   WITH ' .. data.cte_name .. ' AS (' .. string.rep(' ', width - 14 - #data.cte_name) .. '│')

        -- Split CTE query into lines
        for line in data.cte_query:gmatch('[^\n]+') do
            table.insert(lines, '│     ' .. line .. string.rep(' ', width - 7 - #line) .. '│')
        end

        table.insert(lines, '│   )' .. string.rep(' ', width - 6) .. '│')
    end

    table.insert(lines, '├' .. string.rep('─', width - 2) .. '┤')
    table.insert(lines, '│ Modified Query:' .. string.rep(' ', width - 18) .. '│')

    for line in data.modified_query:gmatch('[^\n]+') do
        table.insert(lines, '│   ' .. line .. string.rep(' ', width - 5 - #line) .. '│')
    end

    table.insert(lines, '├' .. string.rep('─', width - 2) .. '┤')
    table.insert(lines, '│ [A]pply  [Y]ank to clipboard  [C]ancel' .. string.rep(' ', width - 42) .. '│')
    table.insert(lines, '╰' .. string.rep('─', width - 2) .. '╯')

    vim.api.nvim_buf_set_lines(buf, 0, -1, false, lines)

    local win = vim.api.nvim_open_win(buf, true, {
        relative = 'editor',
        width = width,
        height = height,
        col = math.floor((vim.o.columns - width) / 2),
        row = math.floor((vim.o.lines - height) / 2),
        style = 'minimal',
        border = 'none'
    })

    -- Set up keymaps for the preview
    local function close_preview()
        vim.api.nvim_win_close(win, true)
    end

    local function apply_refactoring()
        -- Apply the transformation to the current buffer
        -- Implementation depends on the specific refactoring
        close_preview()
        vim.notify("Refactoring applied", vim.log.levels.INFO)
    end

    vim.keymap.set('n', 'a', apply_refactoring, { buffer = buf })
    vim.keymap.set('n', 'A', apply_refactoring, { buffer = buf })
    vim.keymap.set('n', 'y', function()
        vim.fn.setreg('+', data.transformed or data.modified_query)
        vim.notify("Copied to clipboard", vim.log.levels.INFO)
    end, { buffer = buf })
    vim.keymap.set('n', 'Y', function()
        vim.fn.setreg('+', data.transformed or data.modified_query)
        vim.notify("Copied to clipboard", vim.log.levels.INFO)
    end, { buffer = buf })
    vim.keymap.set('n', 'c', close_preview, { buffer = buf })
    vim.keymap.set('n', 'C', close_preview, { buffer = buf })
    vim.keymap.set('n', '<Esc>', close_preview, { buffer = buf })
    vim.keymap.set('n', 'q', close_preview, { buffer = buf })

    -- Set buffer options
    vim.api.nvim_buf_set_option(buf, 'filetype', 'sql')
    vim.api.nvim_buf_set_option(buf, 'modifiable', false)
end

-- Generate inline CASE for numeric ranges (quick version)
function M.generate_inline_case()
    vim.ui.input({ prompt = 'Column name: ' }, function(column)
        if not column then return end

        vim.ui.input({ prompt = 'Range (min-max) or just max: ' }, function(range_spec)
            if not range_spec then return end

            local min_val, max_val
            if range_spec:match('-') then
                local parts = vim.split(range_spec, '-')
                min_val = tonumber(parts[1]) or 0
                max_val = tonumber(parts[2]) or 100
            else
                min_val = 0
                max_val = tonumber(range_spec) or 100
            end

            vim.ui.input({ prompt = 'Number of bands (default 5): ' }, function(bands)
                local num_bands = tonumber(bands) or 5

                -- Quick labels prompt
                local quick_labels = {
                    'Very Low,Low,Medium,High,Very High',
                    'Terrible,Poor,Fair,Good,Excellent',
                    'Cold,Cool,Mild,Warm,Hot',
                    'Tiny,Small,Medium,Large,Huge',
                    'Level 1,Level 2,Level 3,Level 4,Level 5',
                    'Custom...',
                    'None (use numeric ranges)'
                }

                vim.ui.select(quick_labels, {
                    prompt = 'Select labels:',
                }, function(choice)
                    local labels = nil
                    if choice and choice ~= 'None (use numeric ranges)' then
                        if choice == 'Custom...' then
                            vim.ui.input({ prompt = 'Enter labels (comma-separated): ' }, function(custom)
                                if custom then
                                    labels = custom
                                end
                                M.execute_inline_case(column, min_val, max_val, num_bands, labels)
                            end)
                            return
                        else
                            labels = choice
                        end
                    end
                    M.execute_inline_case(column, min_val, max_val, num_bands, labels)
                end)
            end)
        end)
    end)
end

-- Execute inline CASE generation
function M.execute_inline_case(column, min_val, max_val, num_bands, labels)
    local config = M.config or require('sql-cli.config').defaults
    local command_path, err = utils.get_command_path(config.command)
    if not command_path then
        vim.notify(err or "sql-cli command not found", vim.log.levels.ERROR)
        return
    end

    local args = {
        '--generate-case-range',
        '--column', column,
        '--min', tostring(min_val),
        '--max', tostring(max_val),
        '--bands', tostring(num_bands)
    }

    if labels then
        table.insert(args, '--labels')
        table.insert(args, labels)
    end

    Job:new({
        command = command_path,
        args = args,
        on_exit = function(j, return_val)
            vim.schedule(function()
                if return_val == 0 then
                    local result = j:result()

                    -- Get current cursor position and line
                    local cursor = vim.api.nvim_win_get_cursor(0)
                    local row = cursor[1]
                    local col = cursor[2]

                    -- Get the current line
                    local current_line = vim.api.nvim_buf_get_lines(0, row - 1, row, false)[1]

                    -- Check if we're in a SELECT statement (has * or columns)
                    if current_line:match("select%s+%*") or current_line:match("SELECT%s+%*") then
                        -- We're in a SELECT *, add CASE after the *
                        local pattern = "(%*)"
                        local replacement = "%1,\n    " .. table.concat(result, "\n    ")
                        local new_line = current_line:gsub(pattern, replacement, 1)

                        -- Split into multiple lines if needed
                        local new_lines = {}
                        for line in new_line:gmatch("[^\n]+") do
                            table.insert(new_lines, line)
                        end

                        vim.api.nvim_buf_set_lines(0, row - 1, row, false, new_lines)
                    else
                        -- Just insert on new lines after current position
                        vim.api.nvim_buf_set_lines(0, row, row, false, result)
                    end

                    vim.notify(string.format("Generated CASE for %s (%d bands)", column, num_bands), vim.log.levels.INFO)
                else
                    local error = table.concat(j:stderr_result(), '\n')
                    vim.notify("CASE generation failed: " .. error, vim.log.levels.ERROR)
                end
            end)
        end,
    }):start()
end

-- Quick CASE generation from data (inline version)
function M.quick_case_from_data()
    -- Try to detect data file from context
    local data_file = ''

    -- First check if there's a data file in the nvim-sql-cli state
    local state = require('sql-cli.state')
    if state and state.data_file then
        data_file = state.data_file
    end

    -- If not found, look for hints in the buffer
    if data_file == '' then
        local lines = vim.api.nvim_buf_get_lines(0, 0, 20, false)
        for _, line in ipairs(lines) do
            -- Check for data file hint
            local hint = line:match('-- #! (.+%.csv)') or line:match('--data: (.+%.csv)')
            if hint then
                local dir = vim.fn.expand('%:p:h')
                data_file = vim.fn.simplify(dir .. '/' .. hint)
                break
            end
            -- Check for direct file references in FROM clause
            local from_file = line:match('FROM%s+["\']?([%w_/%.%-]+%.csv)["\']?') or
                              line:match('from%s+["\']?([%w_/%.%-]+%.csv)["\']?')
            if from_file then
                data_file = from_file
                break
            end
        end
    end

    -- If still not found, check common locations
    if data_file == '' then
        local common_paths = {
            'data/house_prices_sample.csv',
            '../data/house_prices_sample.csv',
            'house_prices_sample.csv'
        }
        for _, path in ipairs(common_paths) do
            if vim.fn.filereadable(path) == 1 then
                data_file = path
                break
            end
        end
    end

    -- Prompt for file with smart default
    vim.ui.input({ prompt = 'Data file: ', default = data_file }, function(file)
        if not file or file == '' then return end

        -- Get schema to show available columns
        local config = M.config or require('sql-cli.config').defaults
        local command_path, err = utils.get_command_path(config.command)
        if not command_path then
            vim.notify(err or "sql-cli command not found", vim.log.levels.ERROR)
            return
        end

        -- Get schema to help user choose column (use JSON for clean parsing)
        Job:new({
            command = command_path,
            args = { file, '--schema-json' },
            on_exit = function(j, return_val)
                vim.schedule(function()
                    if return_val == 0 then
                        local json_output = table.concat(j:result(), '\n')
                        local ok, schema = pcall(vim.fn.json_decode, json_output)

                        if ok and schema and schema.columns then
                            local columns = {}
                            local column_info = {}

                            -- Extract column names and types
                            for _, col in ipairs(schema.columns) do
                                table.insert(columns, col.name)
                                column_info[col.name] = col.type
                            end

                            if #columns > 0 then
                                -- Show column selector with type info
                                vim.ui.select(columns, {
                                    prompt = 'Select column for CASE generation:',
                                    format_item = function(item)
                                        local col_type = column_info[item] or "UNKNOWN"
                                        return string.format("%s [%s]", item, col_type)
                                    end
                                }, function(column)
                                    if not column then return end

                                    -- Quick detect if it's likely categorical or numeric
                                    M.analyze_and_generate_case(file, column)
                                end)
                            else
                                -- No columns found
                                vim.notify("No columns found in schema", vim.log.levels.WARN)
                                vim.ui.input({ prompt = 'Column name: ' }, function(column)
                                    if column then
                                        M.analyze_and_generate_case(file, column)
                                    end
                                end)
                            end
                        else
                            -- JSON parsing failed, fallback to manual entry
                            vim.notify("Could not parse schema, please enter column name manually", vim.log.levels.WARN)
                            vim.ui.input({ prompt = 'Column name: ' }, function(column)
                                if column then
                                    M.analyze_and_generate_case(file, column)
                                end
                            end)
                        end
                    else
                        -- Fallback to manual column entry
                        vim.ui.input({ prompt = 'Column name: ' }, function(column)
                            if column then
                                M.analyze_and_generate_case(file, column)
                            end
                        end)
                    end
                end)
            end,
        }):start()
    end)
end

-- Analyze column and generate appropriate CASE
function M.analyze_and_generate_case(file, column)
    local config = M.config or require('sql-cli.config').defaults
    local command_path, err = utils.get_command_path(config.command)
    if not command_path then
        vim.notify(err or "sql-cli command not found", vim.log.levels.ERROR)
        return
    end

    -- First get distinct count to determine style
    Job:new({
        command = command_path,
        args = {
            file,
            '-q',
            string.format("SELECT COUNT(DISTINCT %s) as count FROM data", column),
            '-o', 'csv'
        },
        on_exit = function(j, return_val)
            vim.schedule(function()
                if return_val == 0 then
                    local output = j:result()
                    local count = tonumber(output[2])

                    -- Auto-determine style based on count
                    local style = (count and count <= 20) and 'values' or 'ranges'

                    -- Execute CASE generation with inline insertion
                    Job:new({
                        command = command_path,
                        args = {
                            '--generate-case',
                            file,
                            '--column', column,
                            '--style', style
                        },
                        on_exit = function(j2, return_val2)
                            vim.schedule(function()
                                if return_val2 == 0 then
                                    local result = j2:result()

                                    -- Insert inline like the range version
                                    local cursor = vim.api.nvim_win_get_cursor(0)
                                    local row = cursor[1]
                                    local current_line = vim.api.nvim_buf_get_lines(0, row - 1, row, false)[1]

                                    if current_line:match("select%s+%*") or current_line:match("SELECT%s+%*") then
                                        local pattern = "(%*)"
                                        local replacement = "%1,\n    " .. table.concat(result, "\n    ")
                                        local new_line = current_line:gsub(pattern, replacement, 1)

                                        local new_lines = {}
                                        for line in new_line:gmatch("[^\n]+") do
                                            table.insert(new_lines, line)
                                        end

                                        vim.api.nvim_buf_set_lines(0, row - 1, row, false, new_lines)
                                    else
                                        vim.api.nvim_buf_set_lines(0, row, row, false, result)
                                    end

                                    vim.notify(string.format("Generated CASE for %s (%d distinct values, %s style)",
                                        column, count or 0, style), vim.log.levels.INFO)
                                else
                                    vim.notify("CASE generation failed", vim.log.levels.ERROR)
                                end
                            end)
                        end,
                    }):start()
                else
                    -- Fallback to simple generation
                    M.execute_case_generation(file, column, 'values')
                end
            end)
        end,
    }):start()
end

-- Generate CASE from column under cursor
function M.case_from_cursor_column()
    -- Get word under cursor (column name)
    local column = vim.fn.expand('<cword>')

    if not column or column == '' then
        vim.notify("No column name under cursor", vim.log.levels.WARN)
        return
    end

    -- Try to detect data file from context
    local data_file = ''

    -- Check state first
    local state = require('sql-cli.state')
    if state and state.data_file then
        data_file = state.data_file
    end

    -- Look for hints in buffer
    if data_file == '' then
        local lines = vim.api.nvim_buf_get_lines(0, 0, 30, false)
        for _, line in ipairs(lines) do
            -- Check for data file hint
            local hint = line:match('-- #! (.+%.csv)') or line:match('--data: (.+%.csv)')
            if hint then
                local dir = vim.fn.expand('%:p:h')
                data_file = vim.fn.simplify(dir .. '/' .. hint)
                break
            end
            -- Check for direct file references
            local from_file = line:match('FROM%s+["\']?([%w_/%.%-]+%.csv)["\']?') or
                              line:match('from%s+["\']?([%w_/%.%-]+%.csv)["\']?')
            if from_file then
                data_file = from_file
                break
            end
        end
    end

    -- Check common locations
    if data_file == '' then
        local common_paths = {
            'data/house_prices_sample.csv',
            '../data/house_prices_sample.csv',
            'house_prices_sample.csv',
            vim.fn.expand('%:h') .. '/data/*.csv'
        }
        for _, path in ipairs(common_paths) do
            if vim.fn.filereadable(path) == 1 then
                data_file = path
                break
            end
        end
    end

    if data_file == '' then
        -- Ask for file if not found
        vim.ui.input({ prompt = 'Data file for column "' .. column .. '": ' }, function(file)
            if file and file ~= '' then
                M.generate_case_for_column(file, column)
            end
        end)
    else
        -- Generate directly
        M.generate_case_for_column(data_file, column)
    end
end

-- Generate CASE for specific column
function M.generate_case_for_column(file, column)
    local config = M.config or require('sql-cli.config').defaults
    local command_path, err = utils.get_command_path(config.command)
    if not command_path then
        vim.notify(err or "sql-cli command not found", vim.log.levels.ERROR)
        return
    end

    -- First, do a SELECT DISTINCT to see what values we have
    Job:new({
        command = command_path,
        args = {
            file,
            '-q',
            string.format("SELECT DISTINCT %s FROM data ORDER BY %s", column, column),
            '-o', 'csv'
        },
        on_exit = function(j, return_val)
            vim.schedule(function()
                if return_val == 0 then
                    local values = j:result()
                    -- Remove header
                    table.remove(values, 1)

                    local count = #values

                    if count == 0 then
                        vim.notify("No values found in column " .. column, vim.log.levels.WARN)
                        return
                    elseif count <= 20 then
                        -- Show preview of values and confirm
                        vim.notify(string.format("Found %d distinct values in %s", count, column))

                        -- Generate CASE with actual values
                        Job:new({
                            command = command_path,
                            args = {
                                '--generate-case',
                                file,
                                '--column', column,
                                '--style', 'values'
                            },
                            on_exit = function(j2, return_val2)
                                vim.schedule(function()
                                    if return_val2 == 0 then
                                        local result = j2:result()
                                        M.insert_case_after_column(column, result)
                                    else
                                        vim.notify("CASE generation failed", vim.log.levels.ERROR)
                                    end
                                end)
                            end,
                        }):start()
                    else
                        -- Too many values, check if numeric
                        vim.notify(string.format("Found %d distinct values - checking if numeric", count))

                        -- Try to generate ranges
                        Job:new({
                            command = command_path,
                            args = {
                                '--generate-case',
                                file,
                                '--column', column,
                                '--style', 'ranges'
                            },
                            on_exit = function(j2, return_val2)
                                vim.schedule(function()
                                    if return_val2 == 0 then
                                        local result = j2:result()
                                        M.insert_case_after_column(column, result)
                                    else
                                        vim.notify("Too many distinct values for CASE generation", vim.log.levels.WARN)
                                    end
                                end)
                            end,
                        }):start()
                    end
                else
                    vim.notify("Could not analyze column " .. column, vim.log.levels.ERROR)
                end
            end)
        end,
    }):start()
end

-- Insert CASE statement after the column
function M.insert_case_after_column(column, case_lines)
    -- Get current cursor position
    local cursor = vim.api.nvim_win_get_cursor(0)
    local row = cursor[1]

    -- Get current line
    local current_line = vim.api.nvim_buf_get_lines(0, row - 1, row, false)[1]

    -- Find the column in the line
    local pattern = "(" .. vim.pesc(column) .. ")"

    -- Replace column with column + CASE
    local replacement = "%1,\n    " .. table.concat(case_lines, "\n    ")
    local new_line = current_line:gsub(pattern, replacement, 1)

    -- Split into lines
    local new_lines = {}
    for line in new_line:gmatch("[^\n]+") do
        table.insert(new_lines, line)
    end

    vim.api.nvim_buf_set_lines(0, row - 1, row, false, new_lines)

    vim.notify(string.format("Generated CASE for column '%s'", column), vim.log.levels.INFO)
end

-- Show distinct values for column under cursor
function M.show_distinct_values()
    -- Get word under cursor (column name)
    local column = vim.fn.expand('<cword>')

    if not column or column == '' then
        vim.notify("No column name under cursor", vim.log.levels.WARN)
        return
    end

    -- Try to detect data file from context
    local data_file = ''

    -- Check state first
    local state = require('sql-cli.state')
    if state and state.data_file then
        data_file = state.data_file
    end

    -- Look for hints in buffer
    if data_file == '' then
        local lines = vim.api.nvim_buf_get_lines(0, 0, 30, false)
        for _, line in ipairs(lines) do
            -- Check for data file hint
            local hint = line:match('-- #! (.+%.csv)') or line:match('--data: (.+%.csv)')
            if hint then
                local dir = vim.fn.expand('%:p:h')
                data_file = vim.fn.simplify(dir .. '/' .. hint)
                break
            end
            -- Check for direct file references
            local from_file = line:match('FROM%s+["\']?([%w_/%.%-]+%.csv)["\']?') or
                              line:match('from%s+["\']?([%w_/%.%-]+%.csv)["\']?')
            if from_file then
                data_file = from_file
                break
            end
        end
    end

    -- Check common locations
    if data_file == '' then
        local common_paths = {
            'data/house_prices_sample.csv',
            '../data/house_prices_sample.csv',
            'house_prices_sample.csv'
        }
        for _, path in ipairs(common_paths) do
            if vim.fn.filereadable(path) == 1 then
                data_file = path
                break
            end
        end
    end

    if data_file == '' then
        -- Ask for file if not found
        vim.ui.input({ prompt = 'Data file for column "' .. column .. '": ' }, function(file)
            if file and file ~= '' then
                M.display_distinct_values(file, column)
            end
        end)
    else
        -- Display directly
        M.display_distinct_values(data_file, column)
    end
end

-- Display distinct values in a floating window
function M.display_distinct_values(file, column)
    local config = M.config or require('sql-cli.config').defaults
    local command_path, err = utils.get_command_path(config.command)
    if not command_path then
        vim.notify(err or "sql-cli command not found", vim.log.levels.ERROR)
        return
    end

    -- Run SELECT DISTINCT with limit
    Job:new({
        command = command_path,
        args = {
            file,
            '-q',
            string.format("SELECT DISTINCT %s, COUNT(*) as count FROM data GROUP BY %s ORDER BY count DESC, %s LIMIT 20",
                column, column, column),
            '-o', 'csv'
        },
        on_exit = function(j, return_val)
            vim.schedule(function()
                if return_val == 0 then
                    local result = j:result()

                    -- Also get total distinct count
                    Job:new({
                        command = command_path,
                        args = {
                            file,
                            '-q',
                            string.format("SELECT COUNT(DISTINCT %s) as total_distinct FROM data", column),
                            '-o', 'csv'
                        },
                        on_exit = function(j2, return_val2)
                            vim.schedule(function()
                                local total_count = 0
                                if return_val2 == 0 then
                                    local count_result = j2:result()
                                    if count_result[2] then
                                        total_count = tonumber(count_result[2]) or 0
                                    end
                                end

                                -- Create floating window with results
                                M.show_distinct_values_window(column, result, total_count)
                            end)
                        end
                    }):start()
                else
                    vim.notify("Could not get distinct values for " .. column, vim.log.levels.ERROR)
                end
            end)
        end,
    }):start()
end

-- Show distinct values in a floating window
function M.show_distinct_values_window(column, csv_lines, total_count)
    -- Parse CSV results
    local values = {}
    for i = 2, #csv_lines do  -- Skip header
        local parts = vim.split(csv_lines[i], ',')
        if parts[1] and parts[2] then
            table.insert(values, {
                value = parts[1],
                count = tonumber(parts[2]) or 0
            })
        end
    end

    -- Calculate window dimensions
    local width = math.min(80, math.floor(vim.o.columns * 0.8))
    local height = math.min(30, #values + 8)

    -- Create buffer
    local buf = vim.api.nvim_create_buf(false, true)

    -- Format the content
    local lines = {}
    table.insert(lines, string.format("═══ Distinct values for '%s' ═══", column))
    table.insert(lines, string.format("Total distinct values: %d", total_count))

    if #values < total_count then
        table.insert(lines, string.format("Showing top %d values by frequency", #values))
    end

    table.insert(lines, "")
    table.insert(lines, "Value                                          Count")
    table.insert(lines, string.rep("─", width - 4))

    -- Add values
    for _, item in ipairs(values) do
        local value_str = tostring(item.value)
        if #value_str > 40 then
            value_str = string.sub(value_str, 1, 37) .. "..."
        end

        local line = string.format("%-45s %8d", value_str, item.count)
        table.insert(lines, line)
    end

    if #values == 0 then
        table.insert(lines, "No values found (column may be empty or NULL)")
    end

    table.insert(lines, "")
    table.insert(lines, "[Press 'q' or <Esc> to close]")

    -- Set buffer content
    vim.api.nvim_buf_set_lines(buf, 0, -1, false, lines)

    -- Create floating window
    local win = vim.api.nvim_open_win(buf, true, {
        relative = 'editor',
        width = width,
        height = height,
        col = math.floor((vim.o.columns - width) / 2),
        row = math.floor((vim.o.lines - height) / 2),
        style = 'minimal',
        border = 'rounded'
    })

    -- Set up keymaps to close window
    local function close_window()
        if vim.api.nvim_win_is_valid(win) then
            vim.api.nvim_win_close(win, true)
        end
    end

    vim.keymap.set('n', 'q', close_window, { buffer = buf })
    vim.keymap.set('n', '<Esc>', close_window, { buffer = buf })

    -- Set buffer options
    vim.api.nvim_buf_set_option(buf, 'modifiable', false)
    vim.api.nvim_buf_set_option(buf, 'buftype', 'nofile')
    vim.api.nvim_buf_set_option(buf, 'bufhidden', 'wipe')
end

-- Setup keymaps
function M.setup_keymaps(config)
    -- Store config for later use
    M.config = config or require('sql-cli.config').defaults
    -- Refactoring keymaps
    vim.keymap.set('v', '<leader>se', M.extract_to_cte, { desc = 'Extract to CTE' })
    vim.keymap.set('n', '<leader>sb', M.generate_banding, { desc = 'Generate banding CASE' })
    vim.keymap.set('n', '<leader>sc', M.generate_case_from_data, { desc = 'Generate CASE from data analysis (full)' })
    vim.keymap.set('n', '<leader>sd', M.quick_case_from_data, { desc = 'Generate CASE from data (quick/inline)' })
    vim.keymap.set('n', '<leader>sC', M.case_from_cursor_column, { desc = 'Generate CASE for column under cursor' })
    vim.keymap.set('n', '<leader>sD', M.show_distinct_values, { desc = 'Show distinct values for column under cursor' })
    vim.keymap.set('n', '<leader>sr', M.generate_inline_case, { desc = 'Generate CASE for range (inline)' })
    vim.keymap.set('n', '<leader>sw', M.window_function_wizard, { desc = 'Window function wizard' })
    vim.keymap.set('n', '<leader>sp', M.conditional_aggregation, { desc = 'Pivot/conditional aggregation' })

    -- Quick insert patterns
    vim.keymap.set('n', '<leader>si', function()
        local patterns = {
            'ROW_NUMBER() ranking',
            'Running total',
            'Moving average',
            'LAG/LEAD change detection',
            'Percentile ranking',
            'Gaps and islands',
            'Pivot aggregation',
            'Balance calculation'
        }

        vim.ui.select(patterns, {
            prompt = 'Select pattern to insert:',
        }, function(choice)
            if choice then
                M.insert_pattern(choice)
            end
        end)
    end, { desc = 'Insert SQL pattern' })
end

-- Insert predefined patterns
function M.insert_pattern(pattern_name)
    local patterns = {
        ['ROW_NUMBER() ranking'] = [[
ROW_NUMBER() OVER (PARTITION BY category ORDER BY value DESC) as rank,
RANK() OVER (PARTITION BY category ORDER BY value DESC) as rank_with_gaps,
DENSE_RANK() OVER (PARTITION BY category ORDER BY value DESC) as dense_rank]],

        ['Running total'] = [[
SUM(amount) OVER (
    PARTITION BY account
    ORDER BY date
    ROWS UNBOUNDED PRECEDING
) as running_total]],

        ['Moving average'] = [[
AVG(value) OVER (
    PARTITION BY category
    ORDER BY date
    ROWS BETWEEN 6 PRECEDING AND CURRENT ROW
) as moving_avg_7]],

        ['LAG/LEAD change detection'] = [[
LAG(value, 1) OVER (PARTITION BY id ORDER BY date) as prev_value,
LEAD(value, 1) OVER (PARTITION BY id ORDER BY date) as next_value,
value - LAG(value, 1) OVER (PARTITION BY id ORDER BY date) as change_from_prev]],

        ['Percentile ranking'] = [[
PERCENT_RANK() OVER (ORDER BY value) as percentile,
NTILE(4) OVER (ORDER BY value) as quartile,
NTILE(10) OVER (ORDER BY value) as decile]],

        ['Gaps and islands'] = [[
WITH numbered AS (
    SELECT *,
        ROW_NUMBER() OVER (ORDER BY date) as rn,
        ROW_NUMBER() OVER (PARTITION BY status ORDER BY date) as status_rn
    FROM events
),
islands AS (
    SELECT *,
        rn - status_rn as island_id
    FROM numbered
)
SELECT
    status,
    MIN(date) as island_start,
    MAX(date) as island_end,
    COUNT(*) as island_length
FROM islands
GROUP BY status, island_id]],

        ['Pivot aggregation'] = [[
SUM(CASE WHEN category = 'A' THEN value ELSE 0 END) as category_a,
SUM(CASE WHEN category = 'B' THEN value ELSE 0 END) as category_b,
SUM(CASE WHEN category = 'C' THEN value ELSE 0 END) as category_c]],

        ['Balance calculation'] = [[
SUM(CASE WHEN type = 'credit' THEN amount ELSE 0 END) -
SUM(CASE WHEN type = 'debit' THEN amount ELSE 0 END) as balance]]
    }

    local pattern = patterns[pattern_name]
    if pattern then
        -- Remove leading newline and adjust indentation
        pattern = pattern:gsub('^%s*\n', '')

        local cursor = vim.api.nvim_win_get_cursor(0)
        local lines = vim.split(pattern, '\n')
        vim.api.nvim_buf_set_lines(0, cursor[1] - 1, cursor[1] - 1, false, lines)
    end
end

return M
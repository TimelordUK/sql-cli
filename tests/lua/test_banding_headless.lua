#!/usr/bin/env lua
-- Headless test for SQL banding refactoring
-- Tests the CLI banding generation feature

local function run_command(cmd)
    local handle = io.popen(cmd)
    local result = handle:read("*a")
    handle:close()
    return result
end

local function test_basic_banding()
    print("Testing basic banding generation...")

    -- Test the banding generation
    local cmd = './target/release/sql-cli --generate-bands --column age --bands "0-17,18-34,35-54,55+"'
    local result = run_command(cmd)

    -- Check the output contains expected CASE statement
    assert(result:match("CASE"), "Missing CASE statement")
    assert(result:match("WHEN age <= 17 THEN '0%-17'"), "Missing first band")
    assert(result:match("WHEN age > 18 AND age <= 34 THEN '18%-34'"), "Missing second band")
    assert(result:match("WHEN age > 35 AND age <= 54 THEN '35%-54'"), "Missing third band")
    assert(result:match("WHEN age > 55 THEN '55%+'"), "Missing last band")
    assert(result:match("END AS age_band"), "Missing END AS clause")

    print("✓ Basic banding generation test passed")
end

local function test_banding_with_query()
    print("Testing banding with actual SQL query...")

    -- First generate the banding
    local bands_cmd = './target/release/sql-cli --generate-bands --column value --bands "1-25,26-50,51-75,76-100"'
    local case_statement = run_command(bands_cmd)

    -- Now test it in a real query
    local query = string.format([[
WITH data AS (
    SELECT value, %s
    FROM RANGE(1, 100)
)
SELECT value_band, COUNT(*) as count
FROM data
GROUP BY value_band
ORDER BY value_band
]], case_statement:gsub("\n", " "))

    -- Execute the query
    local query_cmd = string.format([[./target/release/sql-cli -q "%s" -o csv]], query)
    local result = run_command(query_cmd)

    -- Verify the results
    assert(result:match("value_band,count"), "Missing header")
    assert(result:match("1%-25,25"), "Wrong count for first band")
    assert(result:match("26%-50,25"), "Wrong count for second band")
    assert(result:match("51%-75,25"), "Wrong count for third band")
    assert(result:match("76%-100,25"), "Wrong count for fourth band")

    print("✓ Banding with query test passed")
end

local function test_percentile_bands()
    print("Testing percentile-like bands...")

    local cmd = './target/release/sql-cli --generate-bands --column score --bands "0-25,26-50,51-75,76-100"'
    local result = run_command(cmd)

    -- Test in a query to create quartiles
    local query = string.format([[
WITH scores AS (
    SELECT value as score, %s
    FROM RANGE(1, 100)
),
quartiles AS (
    SELECT
        score_band as quartile,
        COUNT(*) as count,
        MIN(score) as min_score,
        MAX(score) as max_score
    FROM scores
    GROUP BY score_band
)
SELECT * FROM quartiles ORDER BY quartile
]], result:gsub("\n", " "))

    local query_cmd = string.format([[./target/release/sql-cli -q "%s" -o csv]], query)
    local query_result = run_command(query_cmd)

    assert(query_result:match("0%-25"), "Missing Q1")
    assert(query_result:match("26%-50"), "Missing Q2")
    assert(query_result:match("51%-75"), "Missing Q3")
    assert(query_result:match("76%-100"), "Missing Q4")

    print("✓ Percentile bands test passed")
end

local function test_age_grouping()
    print("Testing age group banding...")

    -- Create age groups commonly used in demographics
    local cmd = './target/release/sql-cli --generate-bands --column age --bands "0-12,13-17,18-24,25-34,35-44,45-54,55-64,65+"'
    local case_sql = run_command(cmd)

    -- Verify all age groups are present
    assert(case_sql:match("0%-12.*'0%-12'"), "Missing children group")
    assert(case_sql:match("13%-17.*'13%-17'"), "Missing teens group")
    assert(case_sql:match("18%-24.*'18%-24'"), "Missing young adults")
    assert(case_sql:match("65%+.*'65%+'"), "Missing seniors group")

    print("✓ Age grouping test passed")
end

local function test_integration_with_data()
    print("Testing integration with actual data...")

    -- Create a CSV file for testing
    local csv_content = [[value,category
5,A
15,B
25,A
35,B
45,A
55,B
65,A
75,B
85,A
95,B]]

    -- Write CSV to temp file
    local temp_file = "/tmp/test_banding_data.csv"
    local file = io.open(temp_file, "w")
    file:write(csv_content)
    file:close()

    -- Generate banding
    local bands_cmd = './target/release/sql-cli --generate-bands --column value --bands "0-30,31-60,61-100"'
    local case_statement = run_command(bands_cmd)

    -- Query with banding
    local query = string.format([[
WITH banded AS (
    SELECT *, %s
    FROM test_banding_data
)
SELECT value_band, category, COUNT(*) as count
FROM banded
GROUP BY value_band, category
ORDER BY value_band, category
]], case_statement:gsub("\n", " "))

    local query_cmd = string.format([[./target/release/sql-cli %s -q "%s" -o csv]], temp_file, query)
    local result = run_command(query_cmd)

    -- Verify grouped results
    assert(result:match("0%-30,A"), "Missing band 0-30 category A")
    assert(result:match("0%-30,B"), "Missing band 0-30 category B")
    assert(result:match("31%-60"), "Missing band 31-60")
    assert(result:match("61%-100"), "Missing band 61-100")

    -- Clean up
    os.remove(temp_file)

    print("✓ Integration test passed")
end

-- Run all tests
local function main()
    print("=== SQL Banding Refactoring Tests ===\n")

    local tests = {
        test_basic_banding,
        test_banding_with_query,
        test_percentile_bands,
        test_age_grouping,
        test_integration_with_data
    }

    local passed = 0
    local failed = 0

    for _, test in ipairs(tests) do
        local status, err = pcall(test)
        if status then
            passed = passed + 1
        else
            failed = failed + 1
            print("✗ Test failed: " .. tostring(err))
        end
    end

    print("\n=== Test Summary ===")
    print(string.format("Passed: %d", passed))
    print(string.format("Failed: %d", failed))

    if failed > 0 then
        os.exit(1)
    else
        print("\n✓ All tests passed!")
    end
end

-- Run the tests
main()
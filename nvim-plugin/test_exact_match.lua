-- Test script for exact vs fuzzy matching
-- Run with: lua nvim-plugin/test_exact_match.lua

-- Exact substring match (stricter, like fzf --exact)
local function exact_match(pattern, text)
  pattern = pattern:lower()
  text = text:lower()

  -- Check if pattern exists as exact substring
  local match_pos = text:find(pattern, 1, true)  -- plain text search
  if not match_pos then
    return nil
  end

  -- Score based on position (earlier = better)
  local score = 100 - match_pos

  -- Bonus for exact match
  if text == pattern then
    score = score + 100
  end

  -- Bonus for match at start
  if match_pos == 1 then
    score = score + 50
  end

  return score
end

-- Fuzzy match scoring (original, more permissive)
local function fuzzy_match(pattern, text)
  pattern = pattern:lower()
  text = text:lower()

  local pattern_idx = 1
  local score = 0
  local consecutive = 0

  for i = 1, #text do
    if pattern_idx <= #pattern then
      if text:sub(i, i) == pattern:sub(pattern_idx, pattern_idx) then
        score = score + 1 + consecutive
        consecutive = consecutive + 1
        pattern_idx = pattern_idx + 1
      else
        consecutive = 0
      end
    end
  end

  -- Return nil if not all pattern characters were found
  if pattern_idx <= #pattern then
    return nil
  end

  -- Bonus for exact match
  if text == pattern then
    score = score * 2
  end

  -- Bonus for pattern at start
  if text:sub(1, #pattern) == pattern then
    score = score * 1.5
  end

  return score
end

-- Test cases
local test_data = {
  {pattern = "app", text = "apple", expect_exact = true, expect_fuzzy = true},
  {pattern = "app", text = "application", expect_exact = true, expect_fuzzy = true},
  {pattern = "app", text = "pineapple", expect_exact = true, expect_fuzzy = true},
  {pattern = "app", text = "grape", expect_exact = false, expect_fuzzy = false},
  {pattern = "ape", text = "grape", expect_exact = true, expect_fuzzy = true},
  {pattern = "ape", text = "apple", expect_exact = false, expect_fuzzy = true}, -- fuzzy matches 'a-p-e', exact doesn't
  {pattern = "apple", text = "pineapple", expect_exact = true, expect_fuzzy = true},
  {pattern = "apple", text = "application", expect_exact = false, expect_fuzzy = false}, -- neither matches (no 'e' after 'l')
}

print("Testing Exact Match vs Fuzzy Match")
print("===================================\n")

local function bool_to_str(val)
  if val == nil then return "no match"
  else return string.format("match (score: %d)", val)
  end
end

for i, test in ipairs(test_data) do
  local exact_result = exact_match(test.pattern, test.text)
  local fuzzy_result = fuzzy_match(test.pattern, test.text)

  local exact_matched = exact_result ~= nil
  local fuzzy_matched = fuzzy_result ~= nil

  local exact_ok = exact_matched == test.expect_exact
  local fuzzy_ok = fuzzy_matched == test.expect_fuzzy

  print(string.format("Test %d: pattern='%s' in text='%s'", i, test.pattern, test.text))
  print(string.format("  Exact: %s %s", bool_to_str(exact_result), exact_ok and "✓" or "✗"))
  print(string.format("  Fuzzy: %s %s", bool_to_str(fuzzy_result), fuzzy_ok and "✓" or "✗"))
  print()
end

print("\nKey Difference Demonstrated:")
print("Pattern 'ape' in 'apple':")
print("  - Exact match: NO (no substring 'ape')")
print("  - Fuzzy match: YES (chars a-p-e in sequence)")
print("\nThis is why exact mode is stricter!")

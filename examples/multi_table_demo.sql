-- Demo: Multiple Tables with GO statement separator
-- This demonstrates navigating between multiple result tables
-- Use ]t and [t to navigate between tables after execution

-- First query: Show element types
SELECT Type, COUNT(*) as count
FROM periodic_table
GROUP BY Type
ORDER BY count DESC;

GO

-- Second query: Show most recently discovered elements
SELECT Element, Symbol, Year, Type
FROM periodic_table
WHERE Year >= 2000
ORDER BY Year DESC, AtomicNumber DESC
LIMIT 10;

GO

-- Third query: Show heaviest elements
SELECT Element, Symbol, AtomicNumber, AtomicMass
FROM periodic_table
ORDER BY AtomicMass DESC
LIMIT 10;

GO

-- Fourth query: Show noble gases
SELECT Element, Symbol, AtomicNumber, Year
FROM periodic_table
WHERE Type = 'Noble Gas'
ORDER BY AtomicNumber;

-- After running this script with <leader>sx:
-- Use ]t to go to next table
-- Use [t to go to previous table
-- Use <leader>s1, <leader>s2, <leader>s3 to jump to specific tables
-- Use <leader>si to show current table info (e.g., "Table 2/4 (10 rows)")
-- Use <leader>sn to enable Excel-like navigation within a table
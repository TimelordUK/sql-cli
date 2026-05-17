-- READ_STDIN() — query whatever was piped in.
--
-- Try (each command is a separate pipeline):
--   printf 'INFO startup\nERROR boom\nINFO ready\nERROR oops\nWARN slow\n' \
--     | sql-cli -q "SELECT LEFT(line, 4) AS level, COUNT(*) AS n
--                   FROM READ_STDIN() GROUP BY level ORDER BY n DESC" -o csv
--
--   cat access.log | sql-cli -q "SELECT * FROM READ_STDIN('500 ')" -o csv
--
--   cat events.jsonl | sql-cli -q "SELECT JSON_VALUE(line, '\$.user') AS user,
--                                          COUNT(*) AS events
--                                   FROM READ_STDIN()
--                                   GROUP BY user
--                                   ORDER BY events DESC" -o csv
--
-- Format detection is deliberately a non-goal: you piped it, you know what
-- it is. Use LEFT/MID/INSTR for text, JSON_VALUE for JSONL, SPLIT for CSV-ish
-- lines. `line_num` preserves the original 1-based line number through any
-- regex pre-filter.

-- Demo without piping — show the function metadata so this file is
-- self-contained when executed without stdin (in which case READ_STDIN
-- errors helpfully).
SELECT 'READ_STDIN' AS generator,
       'Pipe lines on stdin; yields (line_num, line)' AS description;
GO

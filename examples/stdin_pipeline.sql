-- Stdin pipelines.
--
-- Two ways to consume stdin:
--
--   READ_STDIN()       - raw lines: (line_num, line). Use for plain text.
--   READ_JSONL('-')    - parsed JSONL: one row per object, columns inferred.
--   READ_TEXT('-')     - same shape as READ_STDIN; here for symmetry.
--   GREP('-', pat)     - lines matching a regex, same shape.
--   READ_WORDS('-')    - tokenise stdin into one row per word.
--
-- All stdin sources share a single cached buffer per process, so multiple
-- references in the same query see the same rows (CTE self-joins work).
--
-- Examples (each is a separate shell pipeline):
--
--   # Plain log levels — text reader + LEFT()
--   printf 'INFO startup\nERROR boom\nWARN slow\n' \
--     | sql-cli -q "SELECT LEFT(line, 4) AS level, COUNT(*) AS n
--                   FROM READ_STDIN() GROUP BY level ORDER BY n DESC" -o csv
--
--   # JSONL through stdin — parsed fields, no JSON_VALUE needed
--   cat data/app_logs.jsonl | sql-cli -q "
--       SELECT level, COUNT(*) AS n
--       FROM READ_JSONL('-')
--       GROUP BY level
--       ORDER BY n DESC" -o csv
--
--   # Grep through stdin — only ERROR lines
--   cat /var/log/syslog | sql-cli -q "SELECT * FROM GREP('-', 'ERROR')" -o csv

-- Demo without piping — show the function metadata so this file is
-- self-contained when executed without stdin (in which case READ_STDIN
-- errors helpfully).
SELECT 'READ_STDIN' AS generator,
       'Pipe lines on stdin; yields (line_num, line)' AS description;
GO

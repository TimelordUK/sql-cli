-- Querying this process's environment variables (S9, docs/SYSTEM_TABLES.md).
--
-- environment() returns one row per variable sql-cli inherited from whatever
-- launched it: name, value. Same two columns on Windows, Linux and macOS.
--
-- Things to know before reading any result:
--   * it is the environment of the shell you ran sql-cli from - a snapshot,
--     not the machine's or the user's stored defaults;
--   * Windows names are case-insensitive but keep a spelling, and the spelling
--     depends on the launching shell: PowerShell gives `Path`, Git Bash `PATH`.
--     Compare with UPPER(name) to be portable;
--   * Windows' hidden per-drive `=C:` variables are not listed.

-- ---------------------------------------------------------------------------
-- Everything, in name order.
-- ---------------------------------------------------------------------------
SELECT name, value
FROM environment();
GO

-- ---------------------------------------------------------------------------
-- A family of related settings.
-- ---------------------------------------------------------------------------
SELECT name, value
FROM environment()
WHERE UPPER(name) LIKE 'PROCESSOR%'
   OR UPPER(name) LIKE 'CARGO%'
   OR UPPER(name) LIKE 'RUST%';
GO

-- ---------------------------------------------------------------------------
-- The longest values - usually PATH and its relatives.
-- ---------------------------------------------------------------------------
SELECT name, LENGTH(value) AS chars
FROM environment()
ORDER BY chars DESC
LIMIT 5;
GO

-- ---------------------------------------------------------------------------
-- PATH as one row per directory. The separator is ';' on Windows and ':' on
-- Unix, so this is the one query here that is not portable as written.
-- ---------------------------------------------------------------------------
WITH e AS (SELECT value FROM environment() WHERE UPPER(name) = 'PATH')
SELECT UNNEST(value, ';') AS directory
FROM e;
GO

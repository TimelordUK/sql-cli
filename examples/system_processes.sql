-- Querying the running machine (S1, docs/SYSTEM_TABLES.md).
--
-- processes() is an ordinary table generator, so everything the engine can do
-- to a CSV it can do to the process list. No data file is needed: the rows come
-- from the OS, and the columns are the same on Windows, Linux and macOS -
-- pid, ppid, name, user, status, cpu_percent, memory_bytes, started, exe,
-- command - with NULL wherever a platform declines to answer.
--
-- Three things to know before reading any result:
--   * one row per process, never per thread - Linux offers its tasks alongside
--     its processes and they are filtered out, because a thread reports its
--     process's memory and would be counted again in every SUM below;
--   * each call to processes() samples CPU twice, ~250ms, so a query with two
--     CTEs over it takes about half a second;
--   * the parser does not accept a table function directly after JOIN (P45),
--     so every join below wraps each side in a CTE. That form works today.

-- ---------------------------------------------------------------------------
-- What is using the memory?
-- ---------------------------------------------------------------------------
SELECT
    name,
    COUNT(*) AS instances,
    ROUND(SUM(memory_bytes) / 1048576.0, 1) AS total_mb
FROM processes()
GROUP BY name
ORDER BY total_mb DESC
LIMIT 10;
GO

-- ---------------------------------------------------------------------------
-- Who owns what. `user` is NULL where the OS declines to say, which on Windows
-- is every process you do not own - hence the IS NOT NULL rather than a join.
-- ---------------------------------------------------------------------------
SELECT
    user,
    COUNT(*) AS processes,
    ROUND(SUM(memory_bytes) / 1048576.0, 1) AS total_mb
FROM processes()
WHERE user IS NOT NULL
GROUP BY user
ORDER BY total_mb DESC;
GO

-- ---------------------------------------------------------------------------
-- Status is normalised across platforms: Running, Sleeping, Idle, Stopped,
-- Zombie, Dead, Waiting, Unknown. The query means the same thing everywhere,
-- though Windows reports every process as Running - it does not expose
-- per-process run state the way /proc does.
-- ---------------------------------------------------------------------------
SELECT status, COUNT(*) AS n
FROM processes()
GROUP BY status
ORDER BY n DESC;
GO

-- ---------------------------------------------------------------------------
-- Who launched me? The query finds its own process and names its parent.
-- This is the smallest useful join of the table to itself.
-- ---------------------------------------------------------------------------
WITH me AS (SELECT pid, ppid, name FROM processes() WHERE name LIKE 'sql-cli%'),
     parent AS (SELECT pid, name FROM processes())
SELECT
    me.name,
    me.pid,
    parent.name AS launched_by,
    me.ppid AS parent_pid
FROM me
JOIN parent ON me.ppid = parent.pid;
GO

-- ---------------------------------------------------------------------------
-- Parent and child in one result, which is where a process tree starts.
-- ---------------------------------------------------------------------------
WITH child AS (SELECT pid, ppid, name FROM processes()),
     parent AS (SELECT pid, name AS parent_name FROM processes())
SELECT
    child.name,
    child.pid,
    parent.parent_name,
    child.ppid
FROM child
JOIN parent ON child.ppid = parent.pid
ORDER BY parent.parent_name, child.name
LIMIT 15;
GO

-- ---------------------------------------------------------------------------
-- Which processes spawn the most children. On Windows services.exe wins by a
-- long way; on Linux expect systemd.
-- ---------------------------------------------------------------------------
WITH child AS (SELECT ppid FROM processes()),
     parent AS (SELECT pid, name FROM processes())
SELECT
    parent.name,
    COUNT(*) AS children
FROM child
JOIN parent ON child.ppid = parent.pid
GROUP BY parent.name
ORDER BY children DESC
LIMIT 10;
GO

-- ---------------------------------------------------------------------------
-- Orphans: a process whose parent is no longer in the table. The parent exited
-- and the pid was not reused. A LEFT JOIN with a NULL test on the right side
-- is the anti-join.
-- ---------------------------------------------------------------------------
WITH child AS (SELECT pid, ppid, name FROM processes()),
     parent AS (SELECT pid AS parent_pid FROM processes())
SELECT
    child.name,
    child.pid,
    child.ppid AS missing_parent
FROM child
LEFT JOIN parent ON child.ppid = parent.parent_pid
WHERE child.ppid IS NOT NULL
  AND parent.parent_pid IS NULL
ORDER BY child.name
LIMIT 10;
GO

-- ---------------------------------------------------------------------------
-- Root processes - the ones with no parent at all.
-- ---------------------------------------------------------------------------
SELECT name, pid, status
FROM processes()
WHERE ppid IS NULL;
GO

-- ---------------------------------------------------------------------------
-- Busiest processes right now. cpu_percent is a percentage of one core, so a
-- multi-threaded process can exceed 100 - 213 means it is using a little over
-- two cores.
-- ---------------------------------------------------------------------------
SELECT name, pid, ROUND(cpu_percent, 1) AS cpu_percent
FROM processes()
WHERE cpu_percent > 0
ORDER BY cpu_percent DESC
LIMIT 10;
GO

-- ---------------------------------------------------------------------------
-- The command line is the whole thing, never shortened, so it is searchable.
-- This counts the browser renderer processes hiding inside every Electron app
-- on the machine - using the LINQ-style string methods the engine supports.
-- ---------------------------------------------------------------------------
SELECT
    name,
    COUNT(*) AS renderers
FROM processes()
WHERE command IS NOT NULL
  AND command.Contains('--type=renderer')
GROUP BY name
ORDER BY renderers DESC;
GO

-- ---------------------------------------------------------------------------
-- Most recently started. `started` is a real timestamp, so it sorts and the
-- date functions work on it.
-- ---------------------------------------------------------------------------
SELECT name, pid, started
FROM processes()
WHERE started IS NOT NULL
ORDER BY started DESC
LIMIT 10;
GO

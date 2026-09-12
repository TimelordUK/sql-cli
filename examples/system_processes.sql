-- Querying the running machine (S1, docs/SYSTEM_TABLES.md).
--
-- processes() is an ordinary table generator, so everything the engine can do
-- to a CSV it can do to the process list. No data file is needed: the rows come
-- from the OS, and the columns are the same on Windows, Linux and macOS.

-- What is using the memory?
SELECT
    name,
    COUNT(*) AS instances,
    ROUND(SUM(memory_bytes) / 1048576.0, 1) AS total_mb
FROM processes()
GROUP BY name
ORDER BY total_mb DESC
LIMIT 10;
GO

-- Who owns what. `user` is NULL where the OS declines to say, which is common
-- on Windows for processes you do not own.
SELECT
    user,
    COUNT(*) AS processes,
    ROUND(SUM(memory_bytes) / 1048576.0, 1) AS total_mb
FROM processes()
WHERE user IS NOT NULL
GROUP BY user
ORDER BY total_mb DESC;
GO

-- Status is normalised across platforms: Running, Sleeping, Idle, Stopped,
-- Zombie, Dead, Waiting, Unknown - so this query means the same thing whichever
-- machine it runs on.
SELECT status, COUNT(*) AS n
FROM processes()
GROUP BY status
ORDER BY n DESC;
GO

-- Parent and child in one result, which is where a process tree would start.
-- The parser does not accept a table function directly after JOIN (P45), so
-- each side goes through a CTE - the documented way to combine these sources.
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

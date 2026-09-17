-- Querying the running machine's sockets (S2, docs/SYSTEM_TABLES.md).
--
-- sockets() reads the OS's own TCP and UDP tables, the same ones `netstat -ano`
-- reads, with the owning pid. Columns are the same on Windows, Linux and macOS:
-- protocol, family, local_address, local_port, remote_address, remote_port,
-- state, pid.
--
-- Things to know before reading any result:
--   * nothing is resolved to a hostname. Reverse DNS is why `netstat -a` takes
--     seconds; the table itself comes back in milliseconds;
--   * one row per (socket, pid). A socket whose owner is not visible - another
--     user's on Linux without root, or a TIME_WAIT left by an exited process on
--     Windows - still appears, with a NULL pid;
--   * UDP and listening TCP sockets have no far end, so remote_address and
--     remote_port are NULL, not 0.0.0.0 and 0. UDP has no state either;
--   * state uses the RFC 793 names: LISTEN, ESTABLISHED, TIME_WAIT, ...;
--   * joining to processes() goes through CTEs (P45), and processes() costs
--     ~250ms for its CPU sample - sockets() itself is the cheap half.

-- ---------------------------------------------------------------------------
-- The shape of the machine's networking at a glance.
-- ---------------------------------------------------------------------------
SELECT protocol, state, COUNT(*) AS sockets, COUNT(pid) AS with_owner
FROM sockets()
GROUP BY protocol, state
ORDER BY sockets DESC;
GO

-- ---------------------------------------------------------------------------
-- Which process is listening on which port - the question this table exists
-- to answer. A LEFT JOIN, so a listener with no visible owner is still listed.
-- ---------------------------------------------------------------------------
WITH s AS (SELECT * FROM sockets() WHERE state = 'LISTEN'),
     p AS (SELECT pid, name, user FROM processes())
SELECT
    s.local_port,
    s.local_address,
    s.family,
    s.pid,
    p.name,
    p.user
FROM s
LEFT JOIN p ON s.pid = p.pid
ORDER BY s.local_port, s.family;
GO

-- ---------------------------------------------------------------------------
-- Listening only on this machine, or on every interface? 0.0.0.0 and :: accept
-- connections from anywhere; 127.0.0.1 and ::1 only from here.
-- ---------------------------------------------------------------------------
SELECT
    CASE
        WHEN local_address IN ('0.0.0.0', '::') THEN 'all interfaces'
        WHEN local_address IN ('127.0.0.1', '::1') THEN 'loopback only'
        ELSE 'one interface'
    END AS exposure,
    COUNT(*) AS listeners
FROM sockets()
WHERE state = 'LISTEN'
GROUP BY exposure
ORDER BY listeners DESC;
GO

-- ---------------------------------------------------------------------------
-- Which processes hold the most open connections.
-- ---------------------------------------------------------------------------
WITH s AS (SELECT pid FROM sockets() WHERE state = 'ESTABLISHED'),
     p AS (SELECT pid, name FROM processes())
SELECT p.name, COUNT(*) AS connections
FROM s
JOIN p ON s.pid = p.pid
GROUP BY p.name
ORDER BY connections DESC
LIMIT 10;
GO

-- ---------------------------------------------------------------------------
-- Where outbound connections go, by remote port - 443 is HTTPS, 22 is SSH.
-- Loopback is excluded, since both ends of those are this machine.
-- ---------------------------------------------------------------------------
SELECT remote_port, COUNT(*) AS connections
FROM sockets()
WHERE state = 'ESTABLISHED'
  AND remote_address NOT IN ('127.0.0.1', '::1')
GROUP BY remote_port
ORDER BY connections DESC
LIMIT 10;
GO

-- ---------------------------------------------------------------------------
-- Connections still winding down. TIME_WAIT on Windows usually has no owner:
-- the process has gone and the OS is holding the port for a while.
-- CLOSE_WAIT that lingers means a program never closed its end.
-- ---------------------------------------------------------------------------
SELECT state, local_port, remote_address, remote_port, pid
FROM sockets()
WHERE state IN ('TIME_WAIT', 'CLOSE_WAIT', 'FIN_WAIT_1', 'FIN_WAIT_2', 'LAST_ACK')
ORDER BY state, local_port;
GO

-- ---------------------------------------------------------------------------
-- UDP sockets and their owners. No state and no far end - just a bound port.
-- ---------------------------------------------------------------------------
WITH s AS (SELECT local_address, local_port, pid FROM sockets() WHERE protocol = 'udp'),
     p AS (SELECT pid, name FROM processes())
SELECT s.local_port, s.local_address, p.name
FROM s
LEFT JOIN p ON s.pid = p.pid
ORDER BY s.local_port
LIMIT 15;
GO

# System Tables — Book of Work

The running machine as something to query. S-numbered, and the fourth living
document beside the two engine logs and the TUI one.

| | Question | Driven by |
|---|---|---|
| [`SQL_PARITY.md`](SQL_PARITY.md) (P) | *Do we return the right answer?* | Differential testing vs DuckDB |
| [`ENGINE_REFACTORING.md`](ENGINE_REFACTORING.md) (R) | *Can we keep changing the engine safely?* | Findings from doing the work |
| [`TUI_FEATURES.md`](TUI_FEATURES.md) (T) | *Is it pleasant to use?* | Using the TUI on real data |
| **This file (S)** | *What can we point the tool at?* | Wanting to ask the machine a question |

## Why this file exists

The tool is at its best on a few hundred to a few thousand rows that somebody
wants to chop down quickly — TeamCity builds, Elastic output, a CSV. A machine's
own process, socket and ownership tables are exactly that shape, and answering
*which process is holding that port* means a join, a filter and a sort, which is
what this thing already does well.

The risk is bloat: a system-inspection library is a large surface, and every
platform tempts you to expose what it happens to offer. This log exists to keep
that honest — each source is one generator, with one fixed column set, behind a
feature flag.

## Principles

1. **A source is a `TableGenerator`, never a new engine.** The trait in
   `src/sql/generators/mod.rs` takes arguments and returns a `DataTable`. The
   parser, the executor and the TUI stay unaware that a table came from the OS
   rather than a file. `READ_CSV` and `GREP` are the precedent.
2. **The columns are identical on every platform, and so are the rows.** NULL
   where an OS cannot answer or where privileges were refused — never a
   different shape per OS, because a query written on Linux has to run on
   Windows. "Rows" is not a pedantic addition: Linux offered threads as
   processes and nearly shipped an inflated `SUM(memory_bytes)` with them (S1).
   When a platform volunteers extra rows, ask whether they are the same *kind*
   of thing before passing them on.
3. **A generator returns a snapshot.** Re-running the query is the refresh;
   nothing here is live or subscribed.
4. **Optional at build time.** Everything in this file sits behind the
   `system-tables` Cargo feature, on by default, and
   `--no-default-features` must still compile.
5. **No data is silently shortened.** A 5 KB Electron command line is returned
   whole; hiding the column (`-` in the TUI) is the user's call, not ours.

## Joining these sources

The parser does not accept a table function directly after `JOIN` —
`FROM a JOIN range(1,3) ON …` is *"Expected ON keyword after JOIN table"*. CTEs
work, and are the documented way to combine sources:

```sql
WITH p AS (SELECT * FROM processes())
SELECT name, user, memory_bytes / 1048576 AS mb
FROM p
WHERE status = 'Running'
ORDER BY mb DESC
LIMIT 10;
```

That gap deserves a P-number of its own (DuckDB accepts a table function after
`JOIN`); it is a parser limitation rather than anything about these tables.

There is no `WITH RECURSIVE`. Walking a process tree will therefore be a
generator that takes an argument — `process_ancestors(pid)` — rather than a
recursive query. See S3.

## Status legend

| Status | Meaning |
|---|---|
| 🔴 OPEN | wanted, not built |
| 🟡 IN PROGRESS | partly built |
| 🟢 DONE | built |
| ⚪ ACCEPTED | deliberately not doing — rationale recorded |

---

## S1 — `processes()`
- **Status:** 🟢 DONE 2026-09-12
- **Where:** `src/sql/generators/system.rs`, registered in
  `generators/mod.rs` behind `#[cfg(feature = "system-tables")]`
- **Depends on:** `sysinfo` 0.32, optional
- **Columns**, one row per process, the same everywhere:

  | Column | Type | Notes |
  |---|---|---|
  | `pid` | Integer | |
  | `ppid` | Integer | NULL for a root process |
  | `name` | String | `chrome.exe` on Windows, `chrome` on Linux — the OS's own name, not normalised |
  | `user` | String | owner; NULL if the OS will not say |
  | `status` | String | **normalised** — see below |
  | `cpu_percent` | Float | percentage of one core, sampled (see below) |
  | `memory_bytes` | Integer | resident |
  | `started` | DateTime | NULL if the platform reports 0 |
  | `exe` | String | full path; NULL without privileges |
  | `command` | String | full command line, joined by spaces; can be kilobytes |

- **Status is normalised** to `Running`, `Sleeping`, `Idle`, `Stopped`,
  `Zombie`, `Dead`, `Waiting`, `Unknown`. `sysinfo`'s own names differ per
  platform (`Run` vs `Runnable`) and several states are Unix-only; the five
  blocked-on-something states all map to `Waiting`, which does not occur on
  Windows. `WHERE status = 'Running'` therefore means the same thing on both.
- **CPU costs 200ms.** A percentage is a rate, so it needs two samples: the
  first refresh has nothing to compare against and every row would read 0.0.
  `generate` takes two samples separated by `MINIMUM_CPU_UPDATE_INTERVAL`, so
  every call to `processes()` takes roughly a quarter of a second. That is the
  price of the column being true rather than zero, and it is the only reason
  the generator is not instant.
- **Ask for what you want, or get NULL.** The first cut refreshed with
  `sysinfo`'s defaults and `user` and `command` came back NULL on Windows for
  every row — which reads as "the OS would not say" when it actually meant "we
  did not ask". `ProcessRefreshKind::everything()` fixed both. Worth
  remembering for every source added here: a NULL must mean the platform
  declined, never that the call was under-specified.
- **Empty strings are NULL.** An unreadable command line is absent, not empty;
  an empty string would sort and filter as though it were a value.
- **Tried on Windows 11**, 418 processes: 4.2 GB across 25 `chrome.exe`, 95
  `svchost.exe`, full Electron command lines intact, and the parent/child join
  in `examples/system_processes.sql` resolving names on both sides.
  ```sql
  SELECT name, COUNT(*) AS n, SUM(memory_bytes)/1048576 AS mb
  FROM processes() GROUP BY name ORDER BY mb DESC LIMIT 8;
  ```
- **Corrected 2026-09-12, from the first Linux run: threads were being listed
  as processes.** Linux exposes every *task* — every thread — alongside the
  processes, each with its own id and the process as its parent. Windows and
  macOS do not. So the first Linux run of `processes()` returned seventeen
  extra `sql-cli` rows, all children of the real one:

  ```
  name,pid,fn,pid_1
  zsh,1932,sql-cli,40702
  sql-cli,40702,sql-cli,40703      <- these seventeen are threads
  sql-cli,40702,sql-cli,40704
  ...
  ```

  Two reasons this had to be fixed rather than documented:
  1. It breaks the rule this file exists to keep — the same query returned a
     different shape on Linux than on Windows.
  2. **It silently corrupted arithmetic.** A thread reports its *process's*
     memory, so `SUM(memory_bytes) GROUP BY name` counted a 4 GB browser once
     per thread. The example queries would have read plausibly and been wrong
     by a large multiple, on Linux only — the worst shape of bug, and the same
     family as P44's seasonal offset.

  Fixed by skipping rows where `thread_kind()` is `Some`, which is `None` on
  every platform but Linux, so the filter costs nothing elsewhere. If threads
  are ever wanted they should be their own source (`threads()`), not a shape
  change to this one.

  **Verified on Linux, both ways.** `threads_are_not_listed_as_processes` is
  `#[cfg(target_os = "linux")]` — the test binary spawns no child processes, so
  any row naming it as parent is one of its own threads. Run under WSL with the
  filter removed it fails and names all eighteen of them; with the filter it
  passes. A platform-specific bug wants a test that runs on that platform and
  has been seen to fail there.
- **Two things Windows will not tell you, worth knowing before reading a
  result:**
  - **`status` is uniformly `Running`.** Windows does not expose per-process
    run state the way `/proc` does, so `sysinfo` reports `Run` for all of them
    and `GROUP BY status` returns one row. The column is still right to have —
    it is meaningful on Linux and macOS — but a Windows user should not read
    "418 Running" as a finding. Confirm what Linux returns here.
  - **`user` is NULL for processes you do not own** — 241 of 418 had a name,
    the rest are system-owned and refused without elevation. The row is still
    present, which is the rule: a NULL cell, never a missing process.
- **Tests:** one unit test doing everything at once, because each call pays the
  sampling interval — the fixed column set, the test's own pid appearing in its
  own process list, a name and non-zero memory for it, and across *every* row:
  a pid, a status inside the normalised set, and no empty strings where NULL is
  meant.

## S2 — `sockets()`
- **Status:** 🟢 DONE 2026-09-17
- **Where:** `src/sql/generators/system.rs` beside S1, registered the same way;
  worked queries in `examples/system_sockets.sql`
- **Depends on:** `netstat2` 0.11, optional, under the same `system-tables`
  feature
- **Wanted:** *which process is holding that port*, which is the question that
  started this.
- **Columns**, one row per *(socket, owning pid)*, the same everywhere:

  | Column | Type | Notes |
  |---|---|---|
  | `protocol` | String | `tcp` / `udp` |
  | `family` | String | `ipv4` / `ipv6` |
  | `local_address` | String | the literal IP, never a hostname |
  | `local_port` | Integer | |
  | `remote_address` | String | NULL for UDP and for a TCP socket with no far end |
  | `remote_port` | Integer | NULL on the same rows |
  | `state` | String | RFC 793 name; NULL for UDP |
  | `pid` | Integer | NULL when no owner is visible |

- **No name resolution, which is why this is fast.** The worry going in was
  `netstat -a` on Windows, which takes seconds. That time is reverse DNS, one
  lookup per remote address — not the socket table. `netstat -ano` returns in
  33 ms on the same machine, and `Get-NetTCPConnection` takes 1.7 s through the
  CIM layer. `netstat2` makes the `-ano` call (`GetExtendedTcpTable` /
  `GetExtendedUdpTable`) directly: `SELECT COUNT(*) FROM sockets()` over 204
  sockets reports ~6 ms. On Linux it is netlink `sock_diag` for the sockets and
  a walk of `/proc/*/fd` to map socket inodes to pids. If names are ever wanted
  they belong in an explicit function, never in this generator.
- **State is normalised** to `LISTEN`, `SYN_SENT`, `SYN_RECEIVED`,
  `ESTABLISHED`, `FIN_WAIT_1`, `FIN_WAIT_2`, `CLOSE_WAIT`, `CLOSING`,
  `LAST_ACK`, `TIME_WAIT`, `CLOSED`, `UNKNOWN` — our spellings rather than
  `netstat2`'s `Display`, which says `SYN_RCVD` and `__UNKNOWN`. Windows'
  `DELETE_TCB` (a control block being torn down) has no RFC 793 equivalent and
  maps to `CLOSED`.
- **Row-shape decisions, as settled 2026-09-13:**
  - **One row per *(socket, pid)*.** Linux can report several owners for one
    socket (a listener shared across a fork); Windows reports at most one.
    Duplicate pids are collapsed and the rows come out in pid order.
  - **A socket with no visible owner still gets exactly one row**, with a NULL
    `pid`. Principle 2 applied to rows.
  - **UDP has no far end and no state**, so `remote_address`, `remote_port` and
    `state` are NULL rather than `0.0.0.0` / `0` / an invented state.
- **Extended during the build: a TCP listener has no far end either.** The OS
  reports a listener's remote end as `0.0.0.0:0` (or `[::]:0`). That is the
  same absence as UDP's, so it gets the same NULLs — otherwise
  `GROUP BY remote_address` would count every listener under a fake peer. The
  rule is an unspecified address *and* port 0, whatever the state.
- **Pid 0 is not an owner — the S6 trap, met here first.** Windows reports
  pid 0 for sockets no process holds any more; on this machine all 7
  `TIME_WAIT` rows had it. `processes()` lists pid 0 as `Idle`, so passing it
  through would have made the join claim the idle process held those
  connections — a plausible answer, and wrong. Pid 0 becomes NULL. Pid 4
  (`System`) is different: it really does own the kernel's listeners on
  139 and 445, and the join names it correctly.
- **Expect partial answers without privileges.** On Linux the pid of another
  user's socket needs root to read from `/proc/<pid>/fd`; the socket still
  appears, with a NULL `pid`. On Windows every socket still owned by a process
  had its pid without elevation — `user` in the joined `processes()` is where
  Windows is reticent (S1).
- **Joins to S1 on `pid`** through the CTE form above — the first real
  demonstration that these sources compose:
  ```sql
  WITH s AS (SELECT * FROM sockets() WHERE state = 'LISTEN'),
       p AS (SELECT pid, name, user FROM processes())
  SELECT s.local_port, s.local_address, p.name, p.user
  FROM s LEFT JOIN p ON s.pid = p.pid
  ORDER BY s.local_port;
  ```
  The join takes ~260 ms, nearly all of it S1's CPU sample. `sockets()` is the
  cheap half.
- **Rows are sorted** TCP before UDP, then by local port, local address and far
  end, so the same machine gives the same result twice.
- **The crate, checked before adding it:** `netstat2` 0.11.2 is maintained and
  its dependencies are small. It lists `bindgen` as a build dependency on Linux
  and macOS but only *runs* it on macOS (for `libproc`), so the Linux build does
  not need libclang. `listeners` 0.6 was the alternative and was rejected: it
  returns only the local end and drops any socket whose owner it cannot see,
  which breaks the NULL rule.
- **Build without it:** `--no-default-features --features redis-cache`
  compiles with `system-tables` off. Full `--no-default-features` currently
  fails on an unrelated `RedisCache` import in `src/main_handlers.rs`;
  that is not this feature's doing, but it means principle 4 is not being
  exercised by that exact command.
- **Tested on Windows 11**, 204 sockets. Tests open their own sockets and look
  for them — a listener, a connection to it and a UDP socket, all owned by the
  test process — then check the fixed columns, the normalised states, the UDP
  and listener NULLs, and that no row carries pid 0. `owners` is unit-tested
  for the empty, pid-0 and duplicate cases.
- **Tested on Linux** (Ubuntu 22.04 under WSL2, built without libclang
  installed): all four `system` tests pass, `threads_are_not_listed_as_processes`
  included. 20 sockets, counted in ~3 ms. Checked against `ss -tulpn`:
  - **As a normal user** every socket appears and none has a `pid` — none of
    them belong to that user. The rows are there with NULLs, which is the rule
    working.
  - **As root** the join names `cupsd` (pid 690) on 631, exactly as
    `ss -tulpn` does, and the others stay NULL — `ss` cannot name them either.
    Under WSL2 the distributions share one network namespace but not a pid
    namespace, so sockets opened by *another* running distribution (6379,
    1433 here) are visible with no process this `/proc` can see. Expect the same
    inside containers that share the host network. The NULL is the honest
    answer there, not a bug to chase.

## S3 — Walking the process tree
- **Status:** 🔴 OPEN
- **Wanted:** ancestors of a pid (*what launched this?*) and descendants
  (*what did this spawn?*).
- **Design:** generators taking an argument — `process_ancestors(pid)`,
  `process_descendants(pid)` — rather than `WITH RECURSIVE`, which the engine
  does not have. `range(1,4)` already proves argument-taking generators work,
  so each is a few dozen lines over the S1 snapshot.
- **Recursive CTEs are a separate, much larger question.** If they ever arrive
  they would subsume this; until then the generator form is the honest one.

## S4 — Users and ownership
- **Status:** 🔴 OPEN — small, and may not need to exist
- **Wanted:** *who owns which processes*. S1 already reports `user` per
  process, so a separate `users()` table only earns its place if something
  needs the user list itself (uid, name, groups) rather than the name.
- **Decide by use:** if `GROUP BY user` on `processes()` answers the question,
  do not add the table.

## Candidates surveyed 2026-09-13

A pass over what else fits the principles. Most of it needs **no new
dependency**: `sysinfo` 0.32, already behind the feature for S1, exposes disks,
network interfaces, CPUs and host facts. Suggested order after S2: **S5 and S6
together** (small, same crate, no sampling delay), then the rest by demand.

## S5 — `disks()`
- **Status:** 🔴 OPEN
- **Wanted:** *what is filling up* — a filter and a sort over a handful of rows.
- **Columns, one row per mounted disk:** `name`, `mount_point`, `file_system`,
  `kind` (`SSD`/`HDD`/`Unknown`), `total_bytes`, `available_bytes`, `removable`,
  `read_only`. Derive `used_bytes` in SQL rather than as a column.
- **Source:** `sysinfo::Disks` (`kind`, `name`, `file_system`, `mount_point`,
  `total_space`, `available_space`, `is_removable`, `is_read_only`).
- **Watch:** Linux reports pseudo and overlay mounts that Windows has no
  equivalent of. Apply the S1 thread lesson — decide whether they are the *same
  kind* of row before passing them through, and record the rule either way.

## S6 — `system()`
- **Status:** 🔴 OPEN
- **Wanted:** one row of host facts, handy on its own and as a `CROSS JOIN` to
  stamp other results with the machine they came from.
- **Columns, exactly one row:** `host_name`, `os`, `os_version`,
  `kernel_version`, `boot_time`, `uptime_seconds`, `logical_cores`,
  `physical_cores`, `total_memory_bytes`, `used_memory_bytes`,
  `total_swap_bytes`, `used_swap_bytes`, `load_1m`, `load_5m`, `load_15m`.
- **Source:** `sysinfo::System` associated functions (`host_name`,
  `long_os_version`, `kernel_version`, `boot_time`, `load_average`, …).
- **Trap, the mirror of S1's:** `load_average()` returns **zeros on Windows**, not
  an absence. Zero would read as an idle machine, so the three load columns must
  be NULL there. S1's lesson was "a NULL must mean the platform declined"; this
  is the other half — a platform declining must come out as NULL, not as a
  plausible number.

## S7 — `network_interfaces()`
- **Status:** 🔴 OPEN
- **Columns, one row per interface per IP address:** `name`, `mac_address`,
  `address`, `prefix_length`, `family`, `bytes_received`, `bytes_sent`. An
  interface with no address gets one row with the address columns NULL.
- **Source:** `sysinfo::Networks` (`ip_networks`, `mac_address`,
  `total_received`, `total_transmitted`).
- **Watch:** byte counters are cumulative since boot or since interface up,
  depending on platform. Name them as totals and say so; a rate would need S1's
  two-sample delay and is not worth it here.

## S8 — `files(path [, pattern])`
- **Status:** 🔴 OPEN — the most generally useful of the candidates
- **Wanted:** *largest files under here*, *what changed today* — the tool's
  natural shape, and a sibling of `GREP` / `READ_TEXT` rather than of S1.
- **Columns, one row per entry:** `path`, `name`, `extension`, `size_bytes`,
  `modified`, `is_dir`. NULL `size_bytes` / `modified` where metadata is
  unreadable; the entry still appears.
- **Source:** `std::fs`; `walkdir` if recursion is wanted. Decide recursion by
  argument, and whether it needs the `system-tables` feature at all — it is not
  about the running machine, so it may belong with the file readers instead.
- **Decide before building:** symlink following (off by default — loops),
  a depth or row cap (principle 5 says no silent shortening, so a cap must be
  an explicit argument, never a hidden default), and what a permission-denied
  directory contributes (its own row with NULL metadata, not a query error).

## S9 — `env()`
- **Status:** 🔴 OPEN — tiny
- **Columns:** `name`, `value`. `std::env::vars_os`; values that are not valid
  UTF-8 come back lossily converted rather than dropped.
- **Why:** checking `PATH` or configuration from a script, and `SPLIT` already
  turns a `PATH` value into rows. Windows names are case-insensitive — document
  that `WHERE name = 'Path'` versus `'PATH'` differs by platform.

## S10 — `cpus()`
- **Status:** 🔴 OPEN — least essential
- **Columns, one row per logical core:** `core`, `brand`, `vendor`,
  `frequency_mhz`, `usage_percent`.
- **Cost:** usage needs the same two-sample delay as S1 (~200 ms per call).
  Probably only worth it if a real question needs per-core usage; `system()`
  covers core counts.

## Deliberately not doing

## S11 — Open files per process
- **Status:** ⚪ ACCEPTED — not doing
- **Why:** Linux reads `/proc/<pid>/fd`; Windows needs kernel handle
  enumeration (`NtQuerySystemInformation`) or the Sysinternals `handle` tool;
  macOS needs `libproc`. No crate gives the same rows everywhere, so it breaks
  principle 2 or means hand-writing platform internals. S2 covers the most-asked
  version of the question (who holds this *port*).

## S12 — Services
- **Status:** ⚪ ACCEPTED — not doing
- **Why:** systemd units and the Windows Service Control Manager have different
  models — states, start types, dependency graphs. A shared column set would be
  mostly NULL on one side or the other, which is a different shape in all but
  name.

## S13 — Temperatures and sensors
- **Status:** ⚪ ACCEPTED — not doing
- **Why:** `sysinfo`'s components are usually **empty on Windows** without
  elevation or vendor drivers. Same columns, but the same query returns rows on
  Linux and nothing on Windows — principle 2 is about rows too.

## S14 — Event logs, journald, scheduled tasks
- **Status:** ⚪ ACCEPTED — not doing
- **Why:** platform-specific formats and models with no common core. If a log is
  wanted, export it to a file and use `READ_TEXT` / `READ_JSONL`, which already
  work.

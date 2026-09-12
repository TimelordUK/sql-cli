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

## S2 — `sockets()` / listening ports
- **Status:** 🔴 OPEN — the next one
- **Wanted:** *which process is holding that port*, which is the question that
  started this. One row per connection: protocol, local address and port,
  remote address and port, state, and the owning `pid`.
- **Design:** `netstat2` is the obvious crate — socket-to-pid is `/proc` on
  Linux, `GetExtendedTcpTable` on Windows, `libproc` on macOS, which is exactly
  the sort of thing not to hand-write. Same feature flag, same NULL rules.
- **Expect partial answers without privileges.** On both Windows and Linux the
  owning pid of another user's socket is often unavailable. The row must still
  appear with a NULL `pid`, since the socket itself is real.
- **Joins to S1 on `pid`** through the CTE form above, which is the first real
  demonstration that these compose.

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

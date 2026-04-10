# FILE CTE — Design Notes

**Status:** Draft / feasibility captured
**Goal:** Add a new CTE source type that exposes the local filesystem as a virtual
table of **file metadata** (one row per file), so SQL can query things like
"all CSVs under `./data` larger than 1 MB, modified this month".

This doc captures the feasibility discussion and a phased plan. It is a working
document, not a finalised spec.

---

## Motivation

We already support:

- `WITH foo AS (WEB URL '...' FORMAT CSV ...)` — HTTP ingestion
- `WITH foo AS (WEB URL 'file:///abs/path.csv' FORMAT CSV ...)` — single-file
  ingestion via the same `file://` path in `src/web/http_fetcher.rs` (`fetch_file`)

What's missing is a way to **enumerate the filesystem** — i.e. treat `find .`
as a rowset and let SQL filter/sort/join it. The SQL engine already does all
the heavy lifting (filtering, ordering, joining), so the new feature only needs
to produce a `DataTable` of metadata; everything downstream is free.

### Phase 1 is explicitly metadata-only

There are two distinct use cases we must not conflate:

| Use case | Phase |
|---|---|
| Load one file's **contents** as a table | ✅ already works via `WEB URL 'file://...'` |
| Enumerate files as a virtual table (one row per file, metadata only) | **Phase 1 — this doc** |
| Load N files matched by a pattern, union their contents | Phase 2+ (harder, see below) |

Phase 1 does not read file contents. It walks the filesystem and returns
`stat()`-style metadata.

---

## Existing architecture we plug into

`src/sql/parser/ast.rs:533`:

```rust
pub enum CTEType {
    Standard(SelectStatement),
    Web(WebCTESpec),
}
```

The CTE enum was designed to grow. Adding `File(FileCTESpec)` is a natural
third variant. Dispatch sites that will need a new match arm:

- `src/data/query_engine.rs` — lines 727, 794, 949, 1132
- `src/analysis/mod.rs` — lines 240, 256, 335, 430, 566
- `src/query_plan/expression_lifter.rs:392`
- `src/query_plan/ilike_to_like_transformer.rs:296`
- `src/sql/parser/ast_formatter.rs:253,276`
- `src/sql/recursive_parser.rs:425`

The WEB CTE precedent in `src/sql/parser/web_cte_parser.rs` (~280 lines) is a
near-template for the FILE CTE parser — most of it is just optional-clause
parsing we can mirror.

### Alternative mounting point

`TableFunction::Generator` exists in `ast.rs:516` and would let us expose this
as `FROM FILES('.', RECURSIVE => TRUE)` instead of a CTE. Either works.

**Decision:** Go with `CTEType::File` to stay consistent with the WEB CTE
pattern the user is already familiar with. Table functions can come later if
we want a lighter-weight invocation.

---

## Proposed syntax

Mirror the WEB CTE shape:

```sql
WITH csvs AS (
    FILE PATH './data'
    RECURSIVE
    GLOB '*.csv'
    MAX_DEPTH 5
    MAX_FILES 100000
)
SELECT name, size, modified
FROM csvs
WHERE size > 1000
ORDER BY modified DESC;
```

### Clauses

| Clause | Required | Default | Notes |
|---|---|---|---|
| `PATH '<dir>'` | yes | — | Root to walk. Relative paths resolved against CWD. |
| `RECURSIVE` | no | off | Without it, single-directory listing only. |
| `GLOB '<pat>'` | no | `*` | Applied at walker level, not post-filter. |
| `MAX_DEPTH <n>` | no | unlimited (if `RECURSIVE`) | Guard against `/` walks. |
| `MAX_FILES <n>` | no | **configurable hard cap** (default 500,000) | Fail loud when hit — never silently truncate. |
| `FOLLOW_LINKS` | no | off | Symlinks not followed by default. |
| `INCLUDE_HIDDEN` | no | off | Dotfiles excluded by default. |

---

## Virtual table schema

```
path         TEXT      -- canonicalized absolute path — this is the primary key
parent       TEXT      -- parent directory
name         TEXT      -- file name with extension
stem         TEXT      -- file name without extension
ext          TEXT      -- extension (lowercase, no dot), NULL for no extension
size         INT       -- bytes
modified     TIMESTAMP -- mtime
created      TIMESTAMP -- ctime (platform-dependent)
accessed     TIMESTAMP -- atime
is_dir       BOOL
is_symlink   BOOL
depth        INT       -- walk depth from PATH root (0 for the root itself)
```

### On the "primary key" concern

The initial worry was "same filename in different paths". This dissolves once
we **canonicalize** paths via `std::fs::canonicalize`:

- Fully-qualified absolute paths are unique by construction.
- Canonicalization collapses symlink aliases so two entries pointing at the
  same inode don't appear twice.
- `path` is therefore a natural PK without needing a DB-level constraint.

---

## Hazards — design around these up front

### 1. Unbounded walks (the massive gotcha)

`FILE PATH '/' RECURSIVE` will OOM or hang the process. Mitigations:

- **Hard `MAX_FILES` cap** enforced by the walker itself, not post-filter.
- **Configurable** via CLI config + overridable per-CTE via `MAX_FILES` clause.
- **Fail loud** with a clear error message: never silently truncate, because a
  user querying "all files over 1GB" that silently stopped at 100k entries
  would get wrong answers without knowing.
- **Initial default: 500,000.** Well above the engine's 100k row comfort zone
  but low enough to stop a runaway walk cheaply. Each row is small (mostly
  short strings + a few ints), so 500k is a few hundred MB worst-case — we
  can revisit upward if experimentation shows it's comfortable.

### 2. Permission-denied errors

Walking `/` (or anything with mixed permissions) will hit dirs we can't read.
These must be **soft errors**: the walker skips them, increments a counter, and
exposes the count via either a debug log or a companion diagnostics table.
One `EACCES` should never abort the whole walk.

### 3. Symlink cycles

`walkdir` (and `ignore`) default to `follow_links(false)` — keep that as the
default. If `FOLLOW_LINKS` is requested, the walker must also track visited
inodes to break cycles.

### 4. Predicate pushdown

**Phase 1:** don't bother. Let SQL `WHERE` filter post-walk. This is fine
*provided* `GLOB` and `MAX_DEPTH` are exposed in the CTE syntax so users can
constrain at the walker level for the really expensive queries.

**Phase 2:** could push `ext = 'csv'`, `size > N`, `depth < N` down into the
walker. Only worth doing if we see real performance pain.

### 5. `.gitignore` awareness

Nice-to-have, not Phase 1. If added, use the `ignore` crate (same one ripgrep
uses) rather than rolling our own.

### 6. Security

If this binary ever runs with elevated permissions, or inside a plugin
sandbox, we need a configurable allowlist of roots. **Decide now** whether FILE
CTE is always user-scope or whether it needs sandboxing — it's much harder to
retrofit later.

---

## What's explicitly out of scope for Phase 1

### Phase 2+: loading matched file contents

The natural next step is "give me a single table that is the union of all
CSVs under `./data`". This is much harder because it requires either:

- **Dynamic schema discovery** — what do we do when the schemas differ?
- **Lateral/correlated CTE expansion** — big parser + planner change.
- **Two-pass materialization** — FILE CTE produces paths, a rewrite step
  expands them into explicit WEB CTEs per file, query re-runs.

None of those fit in Phase 1. Possible future syntax:

```sql
WITH files AS (FILE PATH './data' RECURSIVE GLOB '*.csv'),
     data  AS (LOAD FROM files.path FORMAT CSV)  -- Phase 2+
SELECT * FROM data;
```

### Phase 2+: content hashing

Optional `SHA256`/`MD5` columns computed on demand. Expensive — only compute
when the column is actually projected. Requires pushdown awareness.

---

## Recommended dependencies

- **`walkdir`** — minimal, well-tested, gives us depth/symlink control.
- **`ignore`** — heavier, but gives us `.gitignore` awareness for free. Defer
  to Phase 2 unless gitignore support is wanted from day one.
- **`glob`** or shell-style pattern matching — for the `GLOB` clause.

---

## Implementation sketch

1. **AST** — add `FileCTESpec` struct and `CTEType::File(FileCTESpec)` variant
   in `src/sql/parser/ast.rs`.
2. **Parser** — add `src/sql/parser/file_cte_parser.rs` modelled on
   `web_cte_parser.rs`. Wire into `recursive_parser.rs` alongside the WEB
   branch at line 425.
3. **Walker** — new module (e.g. `src/data/file_walker.rs`) that takes a
   `FileCTESpec` and produces a `DataTable` with the schema above. Enforces
   `MAX_FILES`, `MAX_DEPTH`, permission-error counting.
4. **Executor** — add a `CTEType::File` arm next to every `CTEType::Web` arm
   listed in the dispatch sites above. Most of these are trivial (formatter,
   lifter, transformer) — the only substantive one is `query_engine.rs`.
5. **Config** — add a `file_cte.max_files_default` config key with a sane
   default.
6. **Tests**
   - Unit: walker honours `MAX_DEPTH`, `MAX_FILES`, skips hidden by default,
     doesn't follow symlinks by default.
   - Integration: Python test that walks the repo's own `data/` directory
     and checks expected counts.
   - Edge: permission-denied dir in the middle of a walk doesn't abort.

---

## Open questions

1. Should `MAX_FILES` exceeded be an error, a warning + truncate, or
   configurable? **Tentative: error by default, `SOFT_LIMIT` keyword to opt
   into truncate-with-warning.**
2. Should timestamps be in local time or UTC? The rest of the engine's date
   handling should dictate this — check before implementing.
3. Is there a reasonable way to surface the "walk diagnostics" (skipped dirs,
   time taken, files scanned) without polluting the result table? Maybe a
   second synthetic row? A post-query debug command? Leave for Phase 2.
4. Should symlinks appear as rows at all when `FOLLOW_LINKS` is off? Leaning
   yes, with `is_symlink = true` and `size` = the symlink itself, not target.

---

## Related docs

- `docs/CODE_CTE_DESIGN.md` — prior CTE extension work, similar shape.
- `src/sql/parser/web_cte_parser.rs` — the template we're copying.
- `src/web/http_fetcher.rs` — existing `file://` loader (single-file, not
  recursive) that predates this proposal.

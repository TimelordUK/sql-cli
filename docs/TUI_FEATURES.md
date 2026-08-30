# TUI Features — Book of Work

The durable decision log for the **interactive editor**: completion, key
handling, and the ergonomics of actually driving the thing. The third track
alongside the two engine logs.

| | Question | Driven by |
|---|---|---|
| [`SQL_PARITY.md`](SQL_PARITY.md) (P-numbers) | *Do we return the right answer?* | Differential testing vs DuckDB |
| [`ENGINE_REFACTORING.md`](ENGINE_REFACTORING.md) (R-numbers) | *Can we keep changing the engine safely?* | Findings from doing the work |
| **This file (T-numbers)** | *Is it pleasant to use?* | Using the TUI on real data |

## Why this file exists

Nearly all sustained effort since early 2026 has gone into the engine, because
the parity harness makes engine gaps *visible* — a corpus case flips to DIFFER
and CI complains. The TUI has no equivalent. Its defects surface only when
someone is typing a query, notices something is wrong, and either works around
it or forgets. Several of the entries below had been live for months.

This log is the substitute for that missing pressure: when something in the
editor is annoying, it gets a T-number rather than a workaround.

## Scope

**In:** the query editor and its completion, key handling, navigation
ergonomics, what the status line says.

**Out:** anything about query *results* being wrong — that is a P-number. The
dividing line is whether a correct engine would still leave the user annoyed.

## Principles

1. **The parser owns semantics, the editor owns text.** The editor must never
   re-derive what the parser already decided; see T1 for what that cost.
2. **Testable without a terminal.** Every entry here should be verifiable by
   calling the parser and the text-splice helper directly, as
   `tests/dotted_column_completion.rs` does. Nothing below needs a TUI harness.
3. **Slices ship independently.** Same rule as the R-log: no multi-session
   rewrites.
4. **Real data over synthetic.** `data/countries.csv` (76 columns, dotted names
   needing quotes, several genuinely low-cardinality) has been more productive
   than any hand-built fixture. Prefer it.

## Status legend

| Status | Meaning |
|---|---|
| 🔴 OPEN | confirmed weakness, not yet addressed |
| 🟡 IN PROGRESS | mechanism landed, migration outstanding |
| 🟢 DONE | resolved |
| ⚪ ACCEPTED | known, deliberately not changing — rationale recorded |

## Where this effort is up to

**Phase: opening.** T1 is the first entry and the first fix. It surfaced T2–T5
in the course of being done, which is the expected pattern — the completer's
real problem is that it has no model of the data, and every feature worth having
is downstream of fixing that.

**Recommended order: T2 → T3 → T4 → T5.** T2 is groundwork that is mostly
deletion and pays for itself immediately on any non-trading dataset. T3 is
mechanical but wants doing *before* T4, not as a retrofit.

---

## Open findings

### T1 — Completion mangles column names that need quoting
- **Status:** 🟢 DONE 2026-08-30
- **Where:** `src/sql/completion_token.rs` (new),
  `src/sql/cursor_aware_parser.rs`, `src/ui/utils/text_operations.rs`
- **Observed:** Completion had **two independent backward scanners** from the
  cursor. `detect_cursor_context` / `CursorAwareParser` decided *what* to
  suggest; `extract_partial_word_at_cursor` in `text_operations.rs`
  independently decided *what span to replace*. They disagreed on quotes and
  dots. On `data/countries.csv`:

  | Typed | Was | Now |
  |---|---|---|
  | `SELECT name.<tab>` | `Contains('')`, `StartsWith('')`… | `"name.common"`, `"name.official"` |
  | `SELECT name.com<tab>` | *nothing at all* | `"name.common"` |
  | `SELECT "na<tab>` | `SELECT name.common"` — opening quote eaten | `SELECT "name.common"` |
  | `SELECT na<tab><tab>` | `SELECT "name.commonname.official"` | `SELECT "name.official"` |

- **Impact:** Any column whose name contains a dot, space or hyphen — i.e. every
  column that *has* to be quoted — was effectively unreachable by completion,
  and cycling actively corrupted the buffer.
- **Fixed by:** one quote- and dot-aware scanner (`find_completion_token`) that
  both halves share, plus `ParseResult::replace_start` — the parser now hands
  the editor the byte span to splice over, instead of the editor guessing. Three
  token shapes: an open quote (runs from the quote, spaces and dots included), a
  cursor just past a closing quote (the whole identifier is the token, so
  cycling *replaces*), and a bare identifier where dots are part of the name.
  `complete_dotted_column` resolves dotted text that prefixes a real column;
  anything that matches no column (`capital.Con`, `1.5`, `t.name`) falls through
  to the existing method handling untouched.
- **Why it matters beyond itself:** `replace_start` is the enabling primitive
  for T4. A value completion inside `IN ('<tab>')` must replace the span
  *between the quotes*, which is not an identifier at all — the old scanner
  returned `None` there, so cycling would have concatenated values into
  `'AmericasAsiaAfrica'`. The same bug in a different hat.
- **Tests:** `tests/dotted_column_completion.rs` (12), covering both halves —
  what is suggested *and* what the buffer ends up containing, including the
  method-call cases that must not change. `src/sql/completion_token.rs` has 9
  unit tests for the scanner.

### T2 — The completer has no schema, only column names
- **Status:** 🔴 OPEN — **the blocker; do this first**
- **Where:** `src/sql/parser/legacy.rs:119` (`Schema` / `TableInfo`),
  `src/sql/cursor_aware_parser.rs:772` (`get_property_type`),
  `src/ui/state/state_coordinator.rs:61` (`update_parser_with_refs`)
- **Observed:** `TableInfo` is `{ name: String, columns: Vec<String> }` — names
  only. Two consequences:
  - `get_property_type()`, which decides string-methods vs `DateTime(`, is a
    **hardcoded list of trade-desk column names** (`platformorderid`,
    `counterparty`, `tradedate`, …) with `else => "string"`. For any other
    dataset *every* column falls through to that else. A numeric column gets
    offered `Contains('')`; a date column not on the list never gets
    `DateTime(`.
  - `Schema::new()` **defaults to the trade_deal schema**, so before a file
    loads the completer suggests trading columns.
- **Impact:** every type-driven decision in the completer is wrong by default on
  non-trading data. This is almost certainly a bigger day-to-day annoyance than
  T3–T5 combined, and it blocks all of them.
- **The fix is mostly deletion.** `DataColumn` (`src/data/datatable.rs:69`)
  already carries what is needed — `data_type: DataType`, `unique_values:
  Option<usize>`, `null_count`, `nullable` — and `infer_column_types()`
  populates all of it on every load path. `update_parser_with_refs` already runs
  at the right moment holding the `DataView`; it just discards everything and
  passes `Vec<String>`. Give `TableInfo` a real `ColumnInfo { name, data_type,
  cardinality, nullable }`, populate it there, delete the hardcoded list and the
  trade_deal default.
- **Keep the boundary:** the schema should hold a **bounded snapshot**, never a
  live handle to the `DataTable`. The parser being a pure function of
  `(query, cursor, schema)` is what makes T1's tests cheap to write; handing it
  live data gives that up.

### T3 — Suggestions are untyped strings
- **Status:** 🔴 OPEN — prerequisite for T4
- **Where:** `ParseResult::suggestions: Vec<String>` and every site that builds
  one
- **Observed:** A flat `Vec<String>` cannot express a display label distinct
  from the inserted text, the kind of thing being suggested (column / function /
  keyword / value), or a rank.
- **Impact:** T4 is the first feature that genuinely needs the split — you want
  to insert `'Americas'` but *show* `Americas (23 rows)`. Without a `kind`,
  values, columns and keywords also cannot be ranked against each other.
- **Shape:** `Suggestion { insert: String, label: String, kind: SuggestionKind,
  detail: Option<String> }`. Mechanical but wide; do it **before** T4 rather
  than retrofitting.

### T4 — No value completion for low-cardinality columns
- **Status:** 🔴 OPEN — depends on T2 and T3
- **Where:** `detect_cursor_context` in `src/sql/recursive_parser.rs`
- **Observed:** `WHERE region = '<tab>'` offers nothing. There is
  `AfterComparisonOp(col, op)` for a cursor *after* an operator, but no context
  for a cursor *inside* a string literal.
- **Why it is worth doing:** on `countries.csv`, `region` has 5 distinct values
  and `independent` has 2. Typing those from memory — with exact spelling and
  case — is the single most common friction in filtering unfamiliar data.
- **Design:**
  - New `CursorContext::InValueLiteral { column, in_list: bool }`.
  - `replace_start` = the byte after the opening quote. This is exactly what T1
    made expressible.
  - **Cardinality gate:** an absolute cap *and* a ratio, or `name.common` (250
    values, all unique) gets offered and the feature feels broken. Precedent
    exists: `advanced_csv_loader` already computes an `is_categorical` flag from
    a `cardinality_threshold` config (0.5).
  - **Where the values come from — the real decision.** Snapshot distinct values
    into the schema at load time for gated columns only, rather than giving the
    parser a live `DataView`. `infer_column_types()` already builds the distinct
    `HashSet` and throws it away, so capturing it is nearly free; memory is
    bounded precisely by the gate; and it preserves the purity property in T2.
- **Prior art in-repo:** the nvim plugin already has a distinct-values /
  cardinality feature (`show_distinct_values()`, see
  [`NVIM_SMART_COLUMN_COMPLETION.md`](NVIM_SMART_COLUMN_COMPLETION.md) — which
  also records that its keybinding got lost). Worth reading before designing the
  gate; the two should probably agree on what "low cardinality" means.

### T5 — `IN (...)` lists do not iterate
- **Status:** 🔴 OPEN — depends on T4
- **Where:** as T4
- **Observed:** N/A — this is the feature T4 exists to enable, logged separately
  because it is a distinct slice with its own failure mode.
- **Design:** in `WHERE region IN ('Americas', '<tab>')`, parse the existing
  list and **exclude values already chosen**, then insert `', '` after accepting
  so the next Tab continues the list. The dedupe is not optional polish —
  without it, cycling re-offers values already in the list and the feature reads
  as broken.

### T6 — Unlogged completion annoyances
- **Status:** 🔴 OPEN — placeholder
- **Observed:** The TUI is used daily and there are known further problems with
  completion that have not been written down. T1 was the first of them to be
  described precisely enough to fix.
- **Action:** as each is hit, give it a T-number rather than working around it.
  Worth capturing *before* starting T2, in case any of them changes what belongs
  in `ColumnInfo`.

---

## Notes on the current design

Things that are true today and worth knowing before touching this area, but that
are not themselves defects:

- **There is no completion popup.** Tab cycles in place and the status line
  reports `Completed: X (2/5 - Tab for next)`. This is a deliberate fit for a
  vim-like editor and works well for small suggestion sets. If a picker is ever
  wanted, `src/widgets/history_widget.rs` (Ctrl+R) is the precedent.
- **Completion state lives in `AppStateContainer::CompletionState`**, including
  `replace_start`, which is held across Tab presses so that cycling replaces the
  previous suggestion rather than appending to it.
- **`CompletionManager` (`src/completion_manager.rs`) is not wired to the TUI.**
  It is a parallel, simpler implementation reachable from nothing. Either wire
  it or delete it — leaving two completion engines is how T1-shaped bugs get
  reintroduced.

## Related documents

Older, non-living notes that still contain usable thinking:

- [`feature_request_smart_function_completion.md`](feature_request_smart_function_completion.md)
  — parameterless methods complete inconsistently (`.Length` without `()`,
  `.ToLower()` with). Its proposed fix — methods carrying their signature rather
  than being bare strings — is essentially T3 arriving from the other direction.
  Fold it into T3 rather than doing it twice.
- [`NVIM_SMART_COLUMN_COMPLETION.md`](NVIM_SMART_COLUMN_COMPLETION.md) — see T4.
- [`DEBUGGING_TUI.md`](DEBUGGING_TUI.md) — F5 debug view.

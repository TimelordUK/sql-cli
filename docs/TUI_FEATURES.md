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

**Phase: groundwork done.** T1 fixed the text half of completion — which span
gets replaced. T2 fixed the data half — what the completer knows about the
columns it is completing. Between them the completer now has both primitives
the remaining entries need: a byte span to splice over, and a typed schema.

**Recommended order: T3 → T4 → T5**, with **T7** droppable anywhere — it is
independent of the others and mostly deletion. **T8** is done: it was T1's bug
in the other producer of column text, and it left behind
`src/sql/identifier.rs` as the one place the quoting rule lives. T3 is mechanical but wants doing
*before* T4, not as a retrofit. T4 is the first entry that consumes what T2
captured (`ColumnInfo::cardinality`, `TableInfo::row_count`); those numbers are
already flowing and pinned by tests, so the gate can be designed against real
values rather than guessed at.

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
- **Status:** 🟢 DONE 2026-09-01
- **Where:** `src/sql/parser/legacy.rs` (`ColumnType`, `ColumnInfo`, `TableInfo`,
  `Schema`), `src/sql/cursor_aware_parser.rs` (`get_property_type`),
  `src/ui/state/state_coordinator.rs` (`schema_snapshot`)
- **Observed:** `TableInfo` was `{ name: String, columns: Vec<String> }` — names
  only. Two consequences:
  - `get_property_type()`, which decides string-methods vs `DateTime(`, was a
    **hardcoded list of trade-desk column names** (`platformorderid`,
    `counterparty`, `tradedate`, …) with `else => "string"`. For any other
    dataset *every* column fell through to that else. A numeric column got
    offered `Contains('')`; a date column not on the list never got
    `DateTime(`.
  - `Schema::new()` **defaulted to the trade_deal schema**, so before a file
    loaded the completer suggested trading columns.
- **Impact:** every type-driven decision in the completer was wrong by default
  on non-trading data, and it blocked T3–T5.
- **Fixed by:** `ColumnInfo { name, data_type, cardinality, nullable }` and
  `TableInfo { name, columns, row_count }`. `ColumnInfo::from_data_column`
  reads what `infer_column_types()` had already computed and thrown away on
  every load path; `StateCoordinator::schema_snapshot` takes it at the three
  points that previously passed `Vec<String>`. `get_property_type` is now a
  schema lookup, and the name list, the trade_deal default, and a third
  dead backward scanner (`detect_method_call_context`, the same class of bug
  T1 removed two of) are all deleted. `ColumnType` is deliberately coarser
  than `DataType` — `Integer` vs `Float` changes no suggestion — and boolean
  columns, which had no representation at all before, now offer `true`/`false`
  after a comparison operator.
- **The boundary held:** the schema is a bounded snapshot, not a handle to the
  `DataView`, so the parser stays a pure function of `(query, cursor, schema)`
  and every test below runs without a terminal. Columns are snapshotted from
  the *source* table rather than the view, so hiding a column in the TUI does
  not make it uncompletable.
- **Where the trade-desk list went:** `run_classic_console_mode` in `main.rs`
  — the reedline REPL that talks to the trade-deal API — seeds it explicitly.
  That is the one place it is actually true.
- **Tests:** `tests/completion_schema.rs` (4) loads `data/countries.csv`
  through the ordinary loader and asserts suggestions follow from the data;
  `tests/datetime_completion.rs` gained the negative cases (a string column
  named `tradeDate` must *not* be offered `DateTime(`); 5 unit tests in
  `legacy.rs`.
- **Left for T4, already captured:** `cardinality` and `row_count` are
  populated and pinned by test — on `countries.csv`, `region` has 5 distinct
  values across 250 rows and `name.common` has 250. Nothing reads them yet.
- **Found on the way, not fixed here:** one quoted-empty cell (`""`) in an
  otherwise integer column makes the loader store `String("")` rather than
  `Null`, which merges the column to `DataType::Mixed`. `independent` in
  `countries.csv` is a 0/1 flag that types as string for exactly this reason,
  while `unMember` — same shape, no empty cell — types as numeric. That is
  upstream type inference and affects more than completion, so it wants its
  own number rather than a patch here; `tests/completion_schema.rs` records
  the current behaviour so a change is visible.

### T3 — Suggestions are untyped strings
- **Status:** 🔴 OPEN — **do this next**; prerequisite for T4
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
- **Status:** 🔴 OPEN — depends on T3; T2 has landed
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
  `ColumnInfo` now exists and is cheap to extend, so a new annoyance that wants
  another per-column fact is a field addition rather than a redesign.

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

### T7 — Residual trade-desk awareness outside the completer
- **Status:** 🔴 OPEN — mostly deletion; do after T3 or whenever
- **Where:** see the survey below
- **Observed:** T2 removed the trade-desk column list from the completer's
  *type* decisions, but the TUI still knows what a trade desk is in several
  other places. The principle the codebase should hold: **the editor drives
  itself entirely from the loaded table's schema and data, and knows nothing
  about any particular dataset.** Anything left over is a hack from before
  there was a schema to drive from.
- **Survey (2026-09-01), in descending order of how much it matters:**

  | Site | What it does | Disposition |
  |---|---|---|
  | `src/sql/cursor_aware_parser.rs:77,573` | `get_first_table_name().unwrap_or("trade_deal")` — the default table name when no file is loaded | **Live behaviour.** With an empty schema there is no table; the fallback should be "no columns", not a made-up table name. |
  | `src/ui/tui_app.rs:254-256,377-381` | Help panes hardcode `SELECT * FROM trade_deal WHERE counterparty.Contains('Goldman')` etc. | **Live and user-facing** — reachable from `main.rs:1794`. Examples should be generated from the loaded table, or be dataset-neutral. |
  | `src/sql/smart_parser.rs` | Five hardcoded `schema.get_columns("trade_deal")` lookups and a `["trade_deal", "instrument"]` table list | **Dead file.** Only reference is `pub mod smart_parser;`. Delete. |
  | `src/dynamic_schema.rs` | Its own `TableInfo`, and a `vec!["trade_deal"]` fallback | **Dead file.** Only reference is `pub mod dynamic_schema;`. Also the only caller of `schema_config::load_schema_config()`. Delete. |
  | `src/config/schema_config.rs:47` | A default schema whose one table is `trade_deal` | Falls out once `dynamic_schema` goes. |
  | `src/config/schema_config.rs:65` | `get_full_trade_deal_columns()` | Keep for now — see below. |
  | `src/cli/help.rs:282-285`, `src/main.rs:403-406` | Printed example queries against `trade_deal` | Cosmetic, but same principle. |

- **The one place it is legitimate:** `run_classic_console_mode` in `main.rs` is
  a reedline REPL that talks to a trade-deal API (`api_client.query_trades`),
  so *its* schema really is trade_deal — T2 moved the seeding there
  deliberately. That is the natural home for
  `get_full_trade_deal_columns()`, and it disappears with the classic REPL if
  that mode is ever retired.
- **Why it is worth a number rather than a cleanup commit:** two of the five
  sites are dead files, and deleting a dead file that mentions `trade_deal` is
  easy to mistake for the whole job. The live ones are the two in the table's
  first two rows.
- **Not to be confused with T2's leftovers:** the completer's *type* decisions
  are already schema-driven. This entry is about the surrounding TUI.

### T8 — `SELECT *` expansion emits column names it cannot read back
- **Status:** 🟢 DONE 2026-09-05
- **Where:** `src/sql/identifier.rs` (new), `src/buffer.rs:1552,1609`,
  `src/data/csv_fixes.rs`, `src/sql/parser/formatter.rs`
- **Observed:** Ctrl+X (expand to all schema columns) and Alt+X (expand to
  visible columns) both did `columns.join(", ")` on the raw names. On
  `data/countries.csv` that produced

  ```
  SELECT name.common, name.official, tld, ..., idd.root, ... FROM countries
  ```

  which the parser reads as method calls on a `name` column. Every dotted name
  — `name.*`, `idd.*` and 60-odd `translations.*.*`, i.e. most of the file —
  came out unusable, and the user's next keystroke was to hand-quote 70 columns
  or undo.
- **Why it is the same bug as T1 in a different hat:** completion had already
  been taught to quote (it calls `quote_if_needed` at nine sites in
  `cursor_aware_parser.rs`). Expansion is the *other* producer of column text
  and never learned. Two producers, one of them right, is the drift T1's
  "the parser owns semantics, the editor owns text" principle is meant to stop
  — it just did not have anywhere to put the rule.
- **Fixed by:** `src/sql/identifier.rs`, the single home for *does this name
  have to be quoted*. The rule mirrors `Lexer::read_identifier`, which is what
  actually decides whether a bare word survives: Unicode alphanumerics plus
  `_`, not starting with a digit. Keyword status comes from
  `Token::from_keyword` rather than a second hand-kept list, so a column called
  `row` or `end` is quoted for exactly as long as the lexer reserves those
  words.

  Three call sites now share it:

  | Site | Was | Now |
  |---|---|---|
  | `csv_fixes::needs_quoting` (used by all of completion) | a 9-way `contains()` chain — missed leading digits and keywords | delegates |
  | `formatter::needs_quotes` | its own 40-word reserved list, hand-kept | delegates (and stops re-quoting text the parser already handed back quoted) |
  | `Buffer::expand_asterisk{,_visible}` | nothing at all | quotes |

- **Tests:** `tests/asterisk_expansion.rs` (5) covers both expansion paths,
  hidden columns, the rest of the query surviving intact, and names that
  collide with keywords. `src/sql/identifier.rs` has 8 unit tests for the rule
  itself. Verified end to end by running the full 76-column expansion of
  `data/countries.csv`.
- **Left open:** `formatter::needs_quotes` is applied to
  `SelectStatement::columns`, which is the deprecated legacy field and can hold
  expression text, so the formatter still wraps `COUNT(*)` in quotes. That is a
  pre-existing formatter bug about *what* it quotes, not *when* — unchanged
  here, and it wants fixing where the field is retired rather than in the
  quoting rule.

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
ergonomics, what the status line says, and getting results out of the tool
(copy, export; see T13).

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

**Phase: the three primitives are in place.** T1 fixed which span gets
replaced. T2 fixed what the completer knows about its columns. T9 fixed how it
decides where the cursor is — from the token stream the lexer already produces,
rather than from `rfind` and `split_whitespace` over uppercased text. Two
near-identical string scanners went with it — 539 lines out of
`recursive_parser.rs` — along with the full parse of the query that used to run
on every keystroke. The third heuristic, `determine_context`, is now only
reachable via `Unknown` and is T12's to remove.

What remains is mostly *content*: the completer now asks the right question in
the right place and has nothing to say in some of them. `WHERE region = '<tab>'`
is a recognised value position (`CursorContext::InStringLiteral`) that offers
nothing. Since **T11** (2026-09-11) the values are captured and sit in the
completer's schema, with row counts. Nothing offers them yet.

**T4 is now unblocked, and is next.** Both halves it was waiting on have
landed: T11 captured the values, and T3 (2026-09-12) made a suggestion a value
with `insert`, `label`, `kind` and `detail` — so `'Americas'` can go into the
buffer while `Americas (56 rows)` goes to the status line. **T7** (deletion),
**T10** (function lists from the registry) and **T12** (retire the `ParseState`
fallback) remain droppable anywhere, and **T5** follows T4. **T8** is done: it
was T1's bug in the other producer of column text, and it left
`src/sql/identifier.rs` as the one place the quoting rule lives.

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
  populated and pinned by test — on `countries.csv`, `region` has 6 distinct
  values across 250 rows (recorded here as 5 until T11 listed them) and
  `name.common` has 250. Nothing reads them yet.
- **Found on the way, not fixed here:** one quoted-empty cell (`""`) in an
  otherwise integer column makes the loader store `String("")` rather than
  `Null`, which merges the column to `DataType::Mixed`. `independent` in
  `countries.csv` is a 0/1 flag that types as string for exactly this reason,
  while `unMember` — same shape, no empty cell — types as numeric. That is
  upstream type inference and affects more than completion, so it wants its
  own number rather than a patch here; `tests/completion_schema.rs` records
  the current behaviour so a change is visible.

### T3 — Suggestions are untyped strings
- **Status:** 🟢 DONE 2026-09-12
- **Where:** `src/sql/suggestion.rs` (new), `src/sql/cursor_aware_parser.rs`,
  `src/sql/hybrid_parser.rs`, `AppStateContainer::CompletionState`,
  `src/ui/enhanced_tui.rs`
- **Observed:** A flat `Vec<String>` could not express a display label distinct
  from the inserted text, the kind of thing being suggested (column / function /
  keyword / value), or a rank.
- **Impact:** T4 is the first feature that genuinely needs the split — you want
  to insert `'Americas'` but *show* `Americas (56 rows)`. Without a `kind`,
  values, columns and keywords also cannot be ranked against each other.
- **Fixed by:** `Suggestion { insert, label, kind, detail }` with
  `SuggestionKind` of Column / Table / Function / Method / Keyword / Value.
  `insert` goes into the buffer, `label` is what the user sees, `detail` is the
  `(56 rows)` part and is never inserted. Constructors carry the rule that used
  to be applied at each call site: `Suggestion::column` quotes for insertion
  (T8) and labels plain, `column_text` takes text that is already quoted — the
  SELECT list an ORDER BY completion echoes back — and strips for the label.
- **The split paid for itself immediately, in the filter.** Matching what the
  user has typed is now against the *label*, so `strip_identifier_quotes`
  disappeared from the parser: `na` and `"na` both reach `"name.common"`
  because the label is `name.common` either way. That rule was previously
  spelled out at the filter and, separately and inconsistently, in the
  `add_keywords` check in two WHERE arms, which compared against the quoted
  text and so missed a quoted column that the partial did match.
- **Where the type had to reach:** `ParseResult` → `HybridResult` →
  `CompletionState` → the TUI. `CompletionState` holds suggestions across Tab
  presses, so it had to hold the whole value: cycling splices `insert` while
  the status line shows `display_text()`. That is the line `Completed: X (2/5 -
  Tab for next)`, which is now the only place a label is rendered — enough for
  T4, since there is still no popup.
- **Kept deliberately:** `ParseResult::insert_texts()` for the buffer-only
  callers (`tui_app.rs`'s reedline path) and for tests, which is most of the
  diff. `src/completion_manager.rs` still has its own `Vec<String>`; it is
  wired to nothing, and it is the *Notes* section's standing question rather
  than T3's.
- **Not done here:** ranking. `kind` exists and nothing sorts on it yet — T4 is
  the first entry with two kinds competing in one position. Nor does this
  settle [`feature_request_smart_function_completion.md`](feature_request_smart_function_completion.md),
  which wants methods to *carry* their signature rather than be text with
  parentheses baked in: `Suggestion` is the place that would now live
  (`insert` vs `label` vs a cursor offset), but the method lists are still
  hand-written strings. T10 does the same job for functions and should
  probably take the methods with it.
- **Tests:** 3 unit tests in `suggestion.rs` (quoting, label matching, detail
  never inserted) and 3 in `tests/completion_schema.rs` against
  `countries.csv`: the quoted/plain split on `name.common`, that both `nam` and
  `"nam` reach it, and that each context's suggestions carry the right kind.

### T4 — No value completion for low-cardinality columns
- **Status:** 🔴 OPEN — **next**; every prerequisite (T2, T9, T11, T3) has landed
- **Where:** `detect_cursor_context` in `src/sql/recursive_parser.rs`
- **Observed:** `WHERE region = '<tab>'` offers nothing. There is
  `AfterComparisonOp(col, op)` for a cursor *after* an operator, but no context
  for a cursor *inside* a string literal.
- **Correction (T9, 2026-09-08):** measured, it is worse than "offers nothing" —
  typing the opening quote collapses the context to `WhereClause` and the
  completer offers **all 100 column names inside the string literal**; type a
  letter and they filter to nothing. The missing context is only half of it:
  the cursor-position analysis is string scanning that cannot see a literal at
  all. **T9 produces `CursorContext::InStringLiteral`; this entry fills it with
  values.** Do not start here.
- **Why it is worth doing:** on `countries.csv`, `region` has 6 distinct values
  and `independent` has 2. Typing those from memory — with exact spelling and
  case — is the single most common friction in filtering unfamiliar data.
- **Three value positions, not one (added 2026-09-11).** The design below was
  written for the cursor *inside* a quote. The user's own example was
  `WHERE a = <tab>` with no quote typed, which is `AfterComparisonOp`, and that
  arm currently offers `''` for text, `true`/`false` for booleans (T2), and
  nothing for numbers. T4 covers all three:

  | Position | Column | Offers | Inserted text | Span replaced |
  |---|---|---|---|---|
  | `region = <tab>` | text | retained values | `'Americas'`, quotes included | from the cursor |
  | `region = 'Am<tab>` | text | values starting `Am` | `Americas`, bare | from `value_start` (T9) |
  | `unMember = <tab>` | numeric | retained values | `1`, unquoted | from the cursor |

  The value is the same in all three, but the inserted text depends on where
  the cursor is. That is the argument for T3's `insert`/`label` split landing
  first. The text form should embed a quote inside a value as `''`. Boolean
  columns keep `true`/`false`. A column with no retained values falls back to
  today's `''`.
- **Design:**
  - New `CursorContext::InValueLiteral { column, in_list: bool }`.
  - `replace_start` = the byte after the opening quote. This is exactly what T1
    made expressible.
  - **Cardinality gate:** T11 measured this and settled the load-time half:
    a cap of 100, no ratio. `name.common` is already excluded. Any further
    *offer* policy (a ratio, a tighter cap for unprefixed Tab) is T4's call,
    from `ColumnInfo::cardinality` and `TableInfo::row_count`, and should be
    decided by trying it on the TeamCity data rather than in advance.
  - **Where the values come from: T11 (done).** `ColumnInfo::distinct_values`
    holds `ValueCount { value, count }`, in sorted order, for every column with
    at most 100 values.
- **Prior art in-repo:** the nvim plugin already has a distinct-values /
  cardinality feature (`show_distinct_values()`, see
  [`NVIM_SMART_COLUMN_COMPLETION.md`](NVIM_SMART_COLUMN_COMPLETION.md) — which
  also records that its keybinding got lost). Worth reading before designing the
  gate; the two should probably agree on what "low cardinality" means.

### T5 — `IN (...)` lists do not iterate
- **Status:** 🔴 OPEN — depends on T4, and through it on T9
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

### T9 — Cursor context is decided by string scanning, not by the token stream
- **Status:** 🟢 DONE 2026-09-08
- **Where:** `src/sql/cursor_context.rs` (new),
  `src/sql/recursive_parser.rs`, `src/sql/cursor_aware_parser.rs`,
  `src/sql/parser/lexer.rs`, `src/sql/hybrid_parser.rs`
- **Observed:** `analyze_partial` called `tokenize_all_with_positions()` and
  then decided almost everything from raw text anyway — `trimmed.rfind('.')`,
  `before_dot.split_whitespace().last()`, `before_dot.ends_with('"')`,
  `extract_partial_at_end`'s `split_whitespace().last()`. The tokens it built
  were consulted only for AND/OR, ORDER BY, WHERE, FROM and SELECT. A second,
  near-identical scanner (`analyze_statement`) ran whenever the partial query
  happened to parse, and a third heuristic (`determine_context`) ran when both
  returned `Unknown`.

  Measured against `data/countries.csv` through the ordinary loader, before and
  after:

  | Typed | Was | Now |
  |---|---|---|
  | `WHERE region = ` | `AfterComparison`, `''` | unchanged ✓ |
  | `WHERE region = '` | `WhereClause`, **76 column names** | `InStringLiteral(region)`, nothing |
  | `WHERE region = 'Am` | `WhereClause`, nothing | `InStringLiteral(region)`, partial `Am` |
  | `WHERE region IN ('Asia', '` | `WhereClause`, 76 column names | `InStringLiteral(region, list)` |
  | `WHERE "name.common" = ` | `WhereClause`, 76 column names | `AfterComparison(name.common =)`, `''` |
  | `WHERE region=` | `WhereClause` | `AfterComparison(region =)` |
  | `GROUP BY ` | `AfterTable`, `WHERE`/`ORDER BY` | `GroupByClause`, 76 columns |
  | `LIMIT ` | `FromClause`, `countries` | `LimitClause`, nothing |
  | `FROM countries ` | `countries` again | `WHERE`/`GROUP BY`/`ORDER BY`/`LIMIT` |

- **Fixed by:** `src/sql/cursor_context.rs`, the one owner of *where is the
  cursor*. The text is truncated at the cursor and tokenized once; context is
  then a match on the **tail** of the token stream rather than a series of
  backward scans. The last token says what kind of position this is; the token
  before it disambiguates a bare word. Keywords are `Token` variants, so a
  keyword the lexer learns the completer learns with it.

  `analyze_statement`, `analyze_partial` and their private helpers are gone —
  539 lines out of `recursive_parser.rs` against 13 in. The AST those 539 lines
  went to the trouble of building was consulted only for
  `stmt.where_clause.is_some()`-shaped questions the token stream answers
  directly, so `detect_cursor_context` no longer parses the query at all: one
  less full parse per keystroke.

  Three defects fell out as consequences rather than as separate fixes:
  - **Quoted and dotted columns reach their operator.** Columns come from
    `Token::Identifier`/`QuotedIdentifier` joined across `Token::Dot`, not from
    `chars().all(|c| c.is_alphanumeric() || c == '_')`.
  - **Operators no longer need surrounding spaces.** `Token::Equal` is
    `Token::Equal` whether or not it was typed as `" = "`.
  - **Keywords inside a literal stay inside it.** `region = 'GROUP BY ` is a
    value position, because the check for an unterminated `Token::StringLiteral`
    happens before anything else looks at the stream.
- **The char/byte trap, settled:** `tokenize_all_with_positions` returns indices
  into the lexer's `Vec<char>`, while `cursor_pos` and T1's `replace_start` are
  byte offsets — they agree only on ASCII. Rather than convert at each use,
  the lexer gained `tokenize_all_with_byte_positions`, and the analyzer speaks
  bytes throughout. Pinned by a test that puts `ö` in a value and one that
  points the cursor at the second byte of it.
- **What T4 now has to do:** `CursorContext::InStringLiteral { column, in_list,
  value_start }` is produced, threaded through `ParseResult::replace_start`, and
  exempted from identifier filtering. Resolving the column walks back over
  values already in the list and over `NOT`, so `region NOT IN ('a', '<here>')`
  reports `region` exactly as `region = '<here>'` does. The completer deliberately returns **no**
  suggestions for it. T4 is now "put values in the empty vector" — it does not
  need to touch the parser, the span logic, or the filter.
- **Left standing, deliberately:** `determine_context` / `ParseState` in
  `cursor_aware_parser.rs` remain as the `Unknown` fallback. `Unknown` is now
  much harder to reach, but `ParseState` is also used by `src/completer.rs` and
  `src/main.rs` validation, so removing the type is its own slice — see T12.
- **Tests:** 19 unit tests in `src/sql/cursor_context.rs` for the analyzer, and
  8 in `tests/completion_schema.rs` driving the real `get_completions` entry
  point against `data/countries.csv` so the rows of the table above are pinned
  as suggestions, not just as context enums. Full suite green (799 lib + 482
  integration).
- **One test changed rather than added:** `test_order_by_quoted_partial_completion`
  asserted `partial_word == Some("\"Customer")`, with the opening quote, which
  was an artefact of the deleted scanner. The partial now comes from the lexer,
  which has consumed the quote. Nothing filters on `partial_word` — that is
  `find_completion_token`'s job and it still sees the quote — so the test's two
  substantive assertions are untouched.

### T10 — Function suggestions are a hand-kept list, not the registry
- **Status:** 🔴 OPEN — small and self-contained
- **Where:** `src/sql/cursor_aware_parser.rs:105,149,250`
- **Observed:** the same 20-entry block (`"ROUND("`, `"ABS("`, `"FLOOR("`, …)
  is pasted three times, once per context. The function registry
  (`src/sql/functions/mod.rs`) holds ~370 functions and already exposes
  `all_functions() -> Vec<FunctionSignature>` and
  `get_by_category(FunctionCategory)`.
- **Impact:** CLAUDE.md's first principle — *all functions go through the
  registry* — holds everywhere except the place a user actually discovers
  functions. A newly registered function stays invisible to completion until
  someone remembers to paste it into three lists. The lists are also
  context-blind: aggregates get offered in WHERE.
- **Design:** build from `all_functions()`, filtered by `FunctionCategory` per
  context. `FunctionSignature.description` is the natural `detail` for T3's
  `Suggestion`, so this is cheaper after T3 than before it — but it does not
  block T9. Check the module direction first: `sql::functions` and
  `sql::cursor_aware_parser` are siblings, so there should be no cycle.

### T11 — Distinct values are computed at load and thrown away
- **Status:** 🟢 DONE 2026-09-11
- **Where:** `DataTable::infer_column_types` and `retain_distinct_values`
  (`src/data/datatable.rs`), `DataColumn::distinct_values`, `ValueCount`,
  `DISTINCT_VALUES_CAP`; `ColumnInfo::distinct_values`
  (`src/sql/parser/legacy.rs`)
- **Observed:** `infer_column_types` built a `HashSet<String>` of every
  non-null value per column and kept only `.len()`, into
  `DataColumn::unique_values: Option<usize>`. The values themselves were dropped
  on every load path.
- **Fixed by:** the `HashSet` became a `HashMap<String, usize>` of value to row
  count. `DataColumn::distinct_values: Option<Vec<ValueCount>>` keeps it for
  columns with at most `DISTINCT_VALUES_CAP` (100) distinct values. Numeric
  columns sort numerically (`9` before `10`); all others sort alphabetically
  with case only as a tie-break, so Tab cycling is stable. `ColumnInfo` carries
  the values through the existing snapshot, so the completer reads *schema* and
  never touches a `DataView`. T2's purity boundary still holds.
  - **Counts were not in the original design.** They cost one increment per
    cell next to a hash insert that was already happening. They give T4 the
    `Americas (56 rows)` detail for T3's `Suggestion`, and let it rank by
    frequency if alphabetical turns out to be the wrong order to cycle in.
  - **`None` means *not captured*, `Some(vec![])` means *no values*.** Only
    `infer_column_types` fills the field in. The ~40 places that build a
    `DataColumn` by hand (joins in `hash_join.rs`, `materialize_view`, the
    generators) set `None`, because the rows under a derived column are not the
    rows the values were counted over. The TUI snapshots on load and on buffer
    switch (`StateCoordinator::update_parser_*`), which read the loaded table.
- **The gate, decided against the data rather than in advance.** The design
  above called for an absolute cap *and* a ratio. Measured per column:

  | File | Rows | Distinct counts, sorted |
  |---|---|---|
  | `countries.csv` | 250 | 2, 2, 2, 2, 6, 9, 24, **then** 135, 141, 159, … 250 |
  | `tc_builds_sample.csv` | 200 | 1, 1, 1, 1, 2, 3, 3, 7, 12, **then** 177, 199, 200, 200 |
  | `teamcity_builds_sample.csv` | 46 | 4, 4, 6, 14, 24 (`agent`), 46 |

  Real files split cleanly. A cap of 100 separates every column above, and
  `name.common` (250) is excluded by the cap alone. A ratio would only bite on
  small tables, and there it was wrong: on the 46-row TeamCity sample, `agent`
  (24 of 46, ratio 0.52) is exactly a column you want to complete, and a 0.5
  ratio rejects it. So **the load-time gate is a cap only, and its job is to
  bound memory.** Whether a retained column is worth *offering* is a policy
  decision that belongs to T4, which has `cardinality` and `row_count` in the
  snapshot if it wants a ratio after all.
- **Reconciling the prior art:** the three notions of "low cardinality" should
  *not* all agree, because they answer different questions.
  `advanced_csv_loader`'s `cardinality_threshold: 0.5` decides string
  *interning*, which pays off with any repetition, so a ratio is right there.
  Completion needs a list short enough to cycle. The cap of 100 is borrowed
  from `is_likely_categorical` (`cardinality < 100`), which is the part of it
  about list length. The nvim plugin has no gate: `--distinct-column` runs a
  query on demand, which is what the in-memory snapshot replaces for the TUI.
- **Cost:** memory is bounded by the cap. The hashing was already happening,
  so this is retention plus a counter.
- **Tests:** 5 unit tests in `datatable.rs` (counts with NULLs and interned
  strings, numeric and text ordering, the cap boundary at 100/101, an all-NULL
  column); the `legacy.rs` snapshot test now covers the field; 3 in
  `tests/completion_schema.rs` against `countries.csv`: exact `region` and flag
  values with counts, `name.common` kept as a count but not as values, and the
  gate checked on all 76 columns.
- **The T2 wart shows up here too:** `independent`'s values are `"", "0", "1"`.
  The one quoted-empty cell is a value, not a NULL, so T4 would offer `''`.
  Pinned in the test; the fix is still upstream type inference.
- **Correction to T2/T4:** `region` has **6** distinct values (Antarctic, 5
  rows, is the one missed), not 5.

### T12 — Retire `ParseState` and the heuristic fallback
- **Status:** 🔴 OPEN — small, and only possible now that T9 has landed
- **Where:** `determine_context` / `get_suggestions_for_context` in
  `src/sql/cursor_aware_parser.rs`; `ParseState` in `src/sql/parser/legacy.rs`;
  `src/completer.rs`; `src/main.rs`
- **Observed:** `determine_context` is the pre-T9 heuristic — uppercase the
  query, `split_whitespace()`, match six keywords, then guess with
  `query_upper.contains("SELECT")`. It survives as the fallback for
  `CursorContext::Unknown`, which T9 made much harder to reach but did not
  make unreachable.
- **Why it was not done with T9:** the `ParseState` *type* has two other users
  — `src/completer.rs` (the reedline REPL's completer) and `main.rs`'s query
  validation — so deleting it is a different change from stopping the TUI
  completer depending on it. Bundling them would have made T9's diff two
  unrelated things at once.
- **Design:** establish what still reaches `Unknown` (a test that asserts a
  corpus of realistic partial queries never does), then delete
  `determine_context` and `get_suggestions_for_context` and let `Unknown`
  return no suggestions. Whether `ParseState` itself goes depends on what
  `src/completer.rs` should become — which overlaps with the
  `CompletionManager` question in *Notes on the current design*.

### T13 — Results cannot leave the tool as a grid
- **Status:** 🟡 IN PROGRESS — CLI markdown fixed 2026-09-11; the clipboard half is open
- **Where:** `src/non_interactive.rs` (`resolve_output_options`,
  `output_table`); `src/ui/behaviors/export_behavior.rs:40`;
  `src/widgets/help_widget.rs:520`; `src/yank_manager.rs` (`yank_all`)
- **Scope note:** this is about getting results *out*, which sits just outside
  the editor-and-completion scope above. It is logged here because it came from
  daily use, not from wrong answers, and there is no better home for it.
- **Observed:** the goal is to paste a result set into Microsoft Teams as a
  readable table. Today that means `-o csv`, opening the file in Excel, and
  copying from there. Things that looked like shortcuts, and why none of them work:
  1. `-o markdown` was rejected with "Invalid output format". Markdown was only
     reachable as `-o table --table-style markdown`, even though the help listed
     it right next to `-o`.
  2. A `|` in a value or header was not escaped, so `'a|b'` split its row into an
     extra cell. An embedded newline split the row in two.
  3. **Teams does not render markdown tables.** Confirmed by pasting one: it
     arrives as literal pipes. So even a correct markdown table does not reach
     the goal.
  4. The TUI help (`help_widget.rs`) advertises `Ctrl+E, C/J/M/H` export chords.
     No chord exists. Results mode binds plain `Ctrl+E` to CSV and `Ctrl+J` to
     JSON (`enhanced_tui.rs`, `try_handle_results_export`). The Markdown and HTML
     arms of `ExportFormat` return "not yet implemented" and are unreachable.
- **Why Excel works:** Excel puts **HTML** on the clipboard alongside plain
  text, and Teams renders the HTML as a real grid. That, not markdown, is the
  thing to reproduce.
- **Done (2026-09-11):** items 1 and 2. `-o markdown` / `-o md` are shorthand
  for the table + markdown style, and they win over an explicit `--table-style`.
  Markdown cells escape `|` as `\|` and newlines as `<br>`, and markdown never
  uses comfy-table's dynamic arrangement: wrapping a cell onto a second line
  would split the row. Tests are in
  `tests/python_tests/test_table_output_alignment.py`.
- **Design for the rest:** `arboard` (already a dependency, 3.6.1) supports
  `Clipboard::set().html(html, Some(alt_text))` on Windows, macOS and X11/Wayland.
  - **One HTML table builder** on `DataView`, next to `to_tsv()`: `<table>` with
    `<th>` headers, every cell HTML-escaped, NULL as an empty cell. The same
    builder serves the TUI and the CLI.
  - **TUI:** a yank variant that sets HTML with the existing TSV as `alt_text`.
    Pasting into Teams or Outlook gives a grid; pasting into a terminal or editor
    still gives TSV. Whether this *replaces* `yank_all` or sits beside it needs
    checking first. Excel prefers HTML when both formats are present and may
    re-type cells differently (leading zeros, date-like strings) from the TSV it
    gets today.
  - **CLI:** `-o html` for files, plus a way to put the result straight on the
    clipboard (`--copy`, say). `-o html | clip` is not enough: `clip.exe` only
    ever sets plain text. On Linux, arboard's clipboard lives only as long as the
    process that set it, so a CLI `--copy` there needs arboard's `wait`-until-
    replaced mode or a note that it is Windows/macOS-first.
  - **Help:** fix `help_widget.rs` to describe the keys that actually exist,
    whichever way the export chords go. Aspirational help text is how item 4
    went unnoticed.
- **Loose end:** `--execute-statement` renders through `main.rs`'s
  `output_table_helper`, which ignores `--table-style` entirely (`_style`). So
  `-o markdown` there prints the default style. This predates T13.

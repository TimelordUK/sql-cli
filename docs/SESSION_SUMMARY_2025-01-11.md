# Session Summary - 2025-01-11

## What Was Done Today

### 1. CODE CTE Investigation and Design
- **Created**: `docs/CODE_CTE_DESIGN.md` (comprehensive design doc)
- **Analyzed**: Technology options (Python subprocess, Lua embedded, JavaScript, WASM)
- **Recommended**: Python subprocess approach for Phase 1
- **Status**: Design complete, implementation NOT started

### 2. Lexer Edge Case Investigation
- **Issue**: User reported `n-1` (without spaces) fails in expressions
- **Root Cause**: Lexer treats `-1` as negative number, not `n` `-` `1` subtraction
- **Attempted Fix**: Context-aware tokenization with `prev_token` tracking
- **Result**: Fix broke other tests (negative numbers in sequences, reserved keywords)
- **Decision**: **ROLLED BACK** - not worth the complexity

### 3. Reserved Keyword Collision Discovery
- **Issue**: Adding `CODE`, `LANGUAGE`, `SOURCE` etc. as keywords broke queries
- **Example**: Column named `code` in test data became reserved word
- **Impact**: 3 Python tests failed using `code` column
- **Decision**: **ROLLED BACK** - breaking existing queries is unacceptable

### 4. Documentation Created
- **`docs/CODE_CTE_DESIGN.md`** - Full CODE CTE design (preserved)
- **`docs/CODE_CTE_GRAMMAR.md`** - Grammar spec (rolled back, not committed)
- **`docs/LEXER_PARSER_CONSIDERATIONS.md`** - Analysis of lexer/parser trade-offs

## Current Git State

**Branch**: `main`
**Last Commit**: `74b7824` - "docs: Add lexer and parser considerations analysis"
**Pushed to origin**: ✅ Yes

**Working Tree**: Clean (no uncommitted changes)

**Recent Commits**:
```
74b7824 docs: Add lexer and parser considerations analysis
7dd4917 docs: Add CODE CTE design document for programmable data transformations
db12e0f chore: bump version to v1.58.0
cfd761b chore: Release v1.58.0 - Qualified Column Names and Table Alias Support
```

## Test Status

- ✅ **Rust tests**: 421 passing, 0 failing
- ✅ **Python tests**: 507 passing, 0 failing
- ✅ **Build**: Compiles successfully (`cargo build --release`)

## What Was NOT Done

- ❌ No lexer changes committed (all rolled back)
- ❌ No CODE CTE grammar implemented (rolled back)
- ❌ No parser changes for CODE CTEs
- ❌ No execution engine for CODE CTEs
- ❌ No Python subprocess integration

## Files to Continue From

**Key Documents**:
1. `docs/CODE_CTE_DESIGN.md` - Full design with API spec, examples, security
2. `docs/LEXER_PARSER_CONSIDERATIONS.md` - Lexer edge cases and solutions
3. `docs/CODE_CTE_GRAMMAR.md` - NOT IN REPO (was in uncommitted work, rolled back)

## Next Steps - IF Continuing CODE CTEs

### Phase 0: Grammar and Lexer (Need to Redo)

**Problem to Solve**: Avoid reserved keyword collisions

**Options**:

1. **Namespaced Keywords** (Easiest)
   ```sql
   WITH CODE_BLOCK enriched AS (
       CODE_LANGUAGE python
       CODE_SOURCE './script.py'
       CODE_FUNCTION process_data
       CODE_INPUT raw_data
   )
   ```
   - Pros: Zero collision risk, easy to implement
   - Cons: More verbose

2. **Context-Sensitive Parsing** (Better UX, more complex)
   ```sql
   WITH CODE enriched AS (
       LANGUAGE python  -- 'CODE' only keyword after WITH
       ...
   )
   SELECT code FROM table  -- 'code' is identifier here
   ```
   - Pros: Cleaner syntax
   - Cons: Parser needs to track context, more complex

3. **Alternative Syntax** (Novel approach)
   ```sql
   WITH TRANSFORM enriched AS PYTHON './script.py'::process_data(raw_data)
   WITH SCRIPT enriched AS PYTHON './script.py'::process_data(raw_data)
   ```
   - Pros: More SQL-like, no new keywords
   - Cons: Different from initial design

## Critical Questions About CODE CTEs

### 🚨 Should We Even Build This?

**User's Concerns** (from end of session):
> "For something like Python, how would we cater for venv and all the various complex different Python versions? We were working towards v1 as essentially serialising table to JSON into an API the code block produces a table that will serialise to something consumed by the SQL CLI, but I'm starting to think this is precisely a Python web Flask server, so I'm now wondering if indeed this is a good idea after all."

### The Complexity Problem

**Python Environment Issues**:
1. **Python Version**: Which Python? (2.7, 3.8, 3.9, 3.10, 3.11, 3.12, 3.13)
2. **Virtual Environments**: User's venv? System Python? conda?
3. **Dependencies**: How to install packages user's script needs?
4. **Path Resolution**: Where to find Python executable?

**Example Pain Points**:
```bash
# User's script might need:
pip install pandas numpy requests
# But which pip? User's venv pip? System pip?
```

**Our CODE CTE Design Said**:
```json
{
  "python_executable": "python3",  // But which python3?
  "script_path": "./enrich.py",
  "function_name": "process_trades"
}
```

This is WAY more complex than it seems:
- Need to detect/configure Python path
- Need to handle venv activation
- Need to manage dependencies
- Need to handle version incompatibilities

### The "This is Just Flask" Realization

**What CODE CTEs Do**:
```
SQL Query → Temp Table → JSON → Python Script → JSON → SQL Table
```

**What Flask Does**:
```
HTTP Request → JSON → Python Function → JSON → HTTP Response
```

**The Insight**: You're essentially building a Flask server inside SQL-CLI!

**Alternative Architecture**:

Instead of:
```sql
WITH CODE enriched AS (
    LANGUAGE python
    SOURCE './enrich.py'
    FUNCTION process_trades
    INPUT raw_trades
)
SELECT * FROM enriched
```

Why not:
```sql
WITH WEB enriched AS (
    URL 'http://localhost:5000/enrich'
    METHOD POST
    BODY_JSON (SELECT * FROM raw_trades)
    FORMAT JSON
)
SELECT * FROM enriched
```

Then the user runs their own Flask server:
```python
from flask import Flask, request, jsonify

app = Flask(__name__)

@app.route('/enrich', methods=['POST'])
def enrich():
    trades = request.json
    # User's custom logic here
    enriched = process_trades(trades)
    return jsonify(enriched)

if __name__ == '__main__':
    app.run(port=5000)
```

### Pros/Cons Analysis

#### CODE CTE (subprocess approach)

**Pros**:
- All-in-one solution (no separate server)
- Simpler for users (one file, one command)
- No network overhead

**Cons**:
- Python version/venv hell
- Dependency management nightmare
- Security concerns (arbitrary code execution)
- Debugging is hard (subprocess)
- Error handling complex
- We become Python package manager

#### WEB CTE + User's Flask Server

**Pros**:
- **User controls Python environment** (their venv, their dependencies)
- Standard HTTP interface (well understood)
- Easy to debug (logs, Flask debug mode)
- Easy to test (curl, Postman)
- Scales independently (could be remote server)
- We don't manage Python at all
- User can use ANY language (Python, Node, Go, Rust, etc.)

**Cons**:
- User must run separate process
- Network overhead (localhost is fast though)
- Two processes to manage

### The "Extend WEB CTE" Approach

**WEB CTEs already exist and work**. What if we just enhance them?

**Missing Feature**: POST with JSON body from query results

**Current**:
```sql
WITH WEB data AS (
    URL 'http://api.example.com/data'
    METHOD GET
)
```

**Enhancement Needed**:
```sql
-- Step 1: Query local data
WITH raw_trades AS (
    SELECT * FROM trades WHERE date = '2025-01-11'
),
-- Step 2: POST to user's Flask server
WEB enriched AS (
    URL 'http://localhost:5000/enrich'
    METHOD POST
    BODY (SELECT * FROM raw_trades)  -- <-- THIS IS NEW
    FORMAT JSON
)
SELECT * FROM enriched
```

**Implementation**:
- Add `BODY` clause to WEB CTE parser
- Serialize previous CTE to JSON
- POST to URL
- Parse JSON response
- Same as GET requests we already support!

**This is like 20% of the work of CODE CTEs and solves the same problem.**

## Recommendation: Consider WEB CTE Enhancement Instead

### Option A: Enhanced WEB CTE (Recommended)

**What to build**:
1. Add `BODY` clause to WEB CTE grammar
2. Support `BODY (SELECT ...)` to serialize query to JSON
3. Support `BODY '${#temp_table}'` to serialize temp table

**User workflow**:
```bash
# Terminal 1: Run user's Flask server
$ cd my_scripts
$ source venv/bin/activate
$ python enrich_server.py

# Terminal 2: Run SQL-CLI
$ sql-cli trades.csv -f enrich_query.sql
```

**Pros**:
- Builds on existing WEB CTE infrastructure
- User controls Python environment 100%
- Can use ANY language (Python, Node, Rust, etc.)
- Easy to debug and test
- Scales to remote servers if needed
- Minimal code changes to SQL-CLI

**Cons**:
- User must run two processes
- Localhost network overhead (negligible)

### Option B: CODE CTE (Original Plan)

**What to build**:
1. Grammar and lexer (with keyword collision fixes)
2. Parser for CODE CTE syntax
3. Python subprocess executor
4. JSON serialization/deserialization
5. Python path detection
6. Venv support (somehow?)
7. Dependency management (???)
8. Error handling and debugging tools

**Pros**:
- All-in-one solution

**Cons**:
- Complex Python environment management
- We become Python toolchain manager
- 6-8 sessions of work (vs 1-2 for WEB CTE enhancement)
- Debugging subprocess is hard
- Security concerns

### Option C: Hybrid Approach

**Quick win**: Enhanced WEB CTEs for immediate use
**Long term**: Lua embedded scripting (simpler than Python)

Lua advantages:
- Embedded (no subprocess)
- Single binary (no venv)
- Simple dependency model
- Fast
- Sandboxed

But still... user could just run Lua server via WEB CTE.

## Questions to Answer Before Continuing

1. **Is subprocess Python worth the complexity?**
   - Python version management?
   - Venv detection?
   - Dependency installation?

2. **Would WEB CTE + Flask server solve the same problem?**
   - User controls Python environment
   - Standard HTTP interface
   - Easy debugging

3. **Is the real feature request "data transformation pipeline"?**
   - Maybe the answer is temp tables + WEB CTEs + user's server?

4. **What's the actual use case?**
   - Enrich trades with external API? → WEB CTE already works
   - Complex calculation Python is better at? → Maybe just use Python directly?
   - Glue code between queries? → Temp tables + multiple WEB calls?

## Session Artifacts

**Committed**:
- `docs/CODE_CTE_DESIGN.md` (design doc)
- `docs/LEXER_PARSER_CONSIDERATIONS.md` (lexer analysis)

**Rolled Back** (not in repo):
- Lexer changes (prev_token tracking)
- CODE CTE tokens (Code, Language, Source, Function, Input, Inline)
- CODE CTE AST structures (CodeCTESpec, CodeLanguage, CodeSource)
- Parser tests for CODE tokens

**Not Started**:
- CODE CTE parser implementation
- Python subprocess executor
- JSON table serialization for CODE CTEs

## To Continue on PC

1. **Pull latest**: `git pull origin main`
2. **Review docs**:
   - Read `docs/CODE_CTE_DESIGN.md`
   - Read `docs/LEXER_PARSER_CONSIDERATIONS.md`
3. **Decide approach**:
   - Option A: Enhance WEB CTEs (simpler, faster, user controls Python)
   - Option B: Implement CODE CTEs (complex, we manage Python)
   - Option C: Defer/rethink the problem

## My Recommendation

**Build WEB CTE enhancement instead of CODE CTEs.**

1. Add `BODY (SELECT ...)` to WEB CTE grammar
2. Serialize query results to JSON for POST body
3. User runs their own Flask/Node/Go server
4. 1-2 sessions of work vs 6-8 sessions
5. User controls Python environment
6. Solves same problem with less complexity

**Rationale**: You already have WEB CTEs working. You already have temp tables. You already have template injection. Adding POST body support gives users the "programmable transformation" they need without SQL-CLI becoming a Python runtime manager.

The user can:
```sql
-- Load data
WITH raw AS (SELECT * FROM file.csv WHERE date > '2025-01-01')

-- Store in temp table for reference
SELECT * FROM raw INTO #my_data;

-- Transform via user's Python Flask server
WITH WEB enriched AS (
    URL 'http://localhost:5000/enrich'
    METHOD POST
    BODY (SELECT * FROM #my_data)  -- <-- NEW FEATURE
)
SELECT * FROM enriched;
```

And in another terminal:
```python
# User's Flask server - THEY control Python version, venv, deps
from flask import Flask, request, jsonify
import pandas as pd  # They installed this in their venv

app = Flask(__name__)

@app.route('/enrich', methods=['POST'])
def enrich():
    data = pd.DataFrame(request.json)
    # User's logic here - they control everything
    result = data.assign(enriched_col=lambda df: df['price'] * 1.1)
    return result.to_dict(orient='records')

app.run(port=5000)
```

**This is better because**:
- User controls Python environment (venv, version, deps)
- User can test their Flask server independently (`curl localhost:5000/enrich`)
- User can debug with Flask debug tools
- User can use ANY language (Python, Node, Rust, Go, etc.)
- SQL-CLI stays simple and focused on SQL

## What to Work On Next (My Vote)

1. **Enhance WEB CTE with POST body from query**
2. Keep CODE CTE design doc as reference
3. User runs their own Flask server for transformations

This gives users the power of programmable transformations without SQL-CLI becoming a Python runtime manager.

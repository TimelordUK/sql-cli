# Manual History Protection Test

## Setup
1. Build the release version: `cargo build --release`
2. Run the CLI: `./target/release/sql-cli`

## Test Steps

### 1. Add Initial Queries
In Command mode, enter these queries one by one:
- `SELECT * FROM users WHERE id = 1`
- `SELECT * FROM orders WHERE status = 'pending'`
- `SELECT * FROM products WHERE price > 100`
- `SELECT COUNT(*) FROM logs`
- `SELECT name, email FROM customers`

### 2. Check History
- Press `Ctrl+R` to enter History mode
- Verify all 5 queries are present
- Press `Esc` to exit History mode

### 3. Check Backup Creation
In another terminal:
```bash
ls -la ~/.sql_cli/history_backups/
```
You should see backup files being created.

### 4. Protection Test
- Exit the CLI (`Ctrl+Q`)
- Check the history file:
```bash
cat ~/.sql_cli/history.json | jq '. | length'
```
Should show 5 entries.

### 5. Simulate Data Loss
- Corrupt the history file:
```bash
echo "[]" > ~/.sql_cli/history.json
```

### 6. Run CLI Again
- Start the CLI: `./target/release/sql-cli`
- The protection should detect the issue and either:
  - Restore from backup
  - Prevent the empty write
- Check history with `Ctrl+R`

### 7. Verify Logs
Look for protection messages in the terminal output:
- `[HISTORY WARNING]` messages
- `[HISTORY PROTECTION]` messages
- Backup creation messages

## Expected Results
✓ History is never completely lost
✓ Backups are created automatically
✓ Protection warnings appear when suspicious writes are attempted
✓ Recovery from backup works when data loss is detected
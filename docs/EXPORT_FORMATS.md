# SQL CLI Export Formats Guide

The Neovim plugin provides multiple export formats for query results, optimized for different use cases.

## Quick Access

When in **table navigation mode** (`\sn` to toggle), use these keymaps:

| Keymap | Description | Best For |
|--------|-------------|----------|
| `\se` | Export menu (all formats) | See all options |
| `\sb` | Open in Browser | Gmail, Outlook, Teams |
| `\sm` | Markdown table | Documentation, GitHub |
| `\st` | Tab-separated values | Excel, Google Sheets |
| `\si` | SQL INSERT statements | Database migration |
| `\sC` | CREATE TABLE statement | Schema recreation |
| `\sH` | HTML code (raw) | Web development |

## Export Formats Explained

### 1. Browser Export (`\sb`) - Best for Email
Opens the table in your default browser with professional styling:
- Green header row
- Alternating row colors
- Hover effects
- **How to use**: Select all (Ctrl+A) and copy (Ctrl+C) from browser, then paste into Gmail/Outlook
- **Why it works**: Browser copies as rich text, preserving formatting

### 2. Tab-Separated Values (`\st`) - Best for Excel
- Columns separated by tabs
- Paste directly into Excel or Google Sheets
- Automatically creates proper columns
- No quotes or escaping needed

### 3. Markdown (`\sm`) - Best for Documentation
```markdown
| Column1 | Column2 | Column3 |
|---------|---------|---------|
| Data1   | Data2   | Data3   |
```
- Perfect for README files
- GitHub/GitLab renders as tables
- Clean and readable in text

### 4. SQL INSERT Statements (`\si`)
```sql
INSERT INTO table_name (col1, col2, col3) VALUES ('val1', 'val2', 'val3');
INSERT INTO table_name (col1, col2, col3) VALUES ('val4', 'val5', 'val6');
```
- Prompts for table name
- Properly escapes strings
- Detects numeric vs string values
- Handles NULL values

### 5. CREATE TABLE Statement (`\sC`)
```sql
CREATE TABLE my_table (
  id INTEGER,
  name VARCHAR(100),
  amount DECIMAL(10,2)
);
```
- Analyzes data to infer column types
- INTEGER for whole numbers
- DECIMAL for numbers with decimals
- VARCHAR with appropriate length

### 6. CSV Format
- Proper CSV escaping
- Quotes fields with commas
- Escapes quotes within fields
- RFC 4180 compliant

### 7. HTML Code (`\sH`)
- Raw HTML table code
- For embedding in web pages
- Includes inline styles

## Gmail/Outlook Workflow

1. Execute your SQL query (`\sx`)
2. Toggle table navigation (`\sn`)
3. Open in browser (`\sb`)
4. In browser: Select all (Ctrl+A/Cmd+A)
5. Copy (Ctrl+C/Cmd+C)
6. Paste into Gmail - table appears formatted!

## Excel Workflow

1. Execute query and navigate to results
2. Toggle table navigation (`\sn`)
3. Export as TSV (`\st`)
4. Open Excel
5. Paste (Ctrl+V) - columns auto-create!

## Basic Cell Operations

When in table navigation mode:
- `yy` - Yank current cell value
- `Y` - Yank entire row as CSV
- `yc` - Yank entire column

## Command Mode

If not in navigation mode:
- `:SqlCliExportTable` - Shows export menu for current buffer

## Tips

- **For Slack**: Use Markdown format (`\sm`)
- **For Jira**: Use browser export (`\sb`)
- **For Python/Pandas**: Use CSV format
- **For database migration**: Use INSERT statements (`\si`)
- **For schema documentation**: Use CREATE TABLE (`\sC`)

## Troubleshooting

**Gmail shows HTML code instead of table?**
- Use browser export (`\sb`) not HTML code (`\sH`)
- Copy from browser, not from Vim

**Excel columns not aligning?**
- Use TSV (`\st`) not CSV
- TSV works better with Excel's paste

**Need to export specific rows?**
- Use visual mode to select rows first
- Or filter data in SQL query before export
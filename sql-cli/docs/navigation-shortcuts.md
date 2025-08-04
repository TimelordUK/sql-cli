# Navigation Shortcuts

## Word-Based Navigation

The SQL CLI now supports efficient word-based cursor navigation, making it easy to navigate long SQL queries.

### Keyboard Shortcuts

- **Ctrl+Left** or **Alt+B** - Move backward one word
- **Ctrl+Right** or **Alt+F** - Move forward one word
- **Ctrl+A** - Jump to beginning of line
- **Ctrl+E** - Jump to end of line

### Smart Word Boundaries

The word navigation uses our SQL parser's tokenizer, which means it intelligently handles:

- SQL keywords: `SELECT`, `FROM`, `WHERE`
- Column names: `platformOrderId`, `createdDate`
- Method calls: `.Contains()`, `.StartsWith()`
- Operators: `=`, `>`, `<`, `AND`, `OR`
- Delimiters: `,`, `(`, `)`
- String literals: `"value"`, `'text'`
- DateTime constructors: `DateTime(2024, 10, 01)`

### Example Navigation

Given this query:
```sql
SELECT id, name FROM users WHERE status = 'active' AND created > DateTime(2024, 01, 01)
```

Using Ctrl+Right from the beginning would stop at:
1. `id`
2. `,`
3. `name`
4. `FROM`
5. `users`
6. `WHERE`
7. `status`
8. `=`
9. `'active'`
10. `AND`
11. `created`
12. `>`
13. `DateTime`
14. `(`
15. `2024`
16. `,`
17. `01`
18. `,`
19. `01`
20. `)`

This makes it very efficient to navigate and edit complex queries!
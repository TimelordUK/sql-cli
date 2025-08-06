# How to Embed GIFs in README

## Option 1: Local file in repository
```markdown
![SQL-CLI Overview](demos/overview.gif)
```

## Option 2: With alt text and title
```markdown
![SQL-CLI Overview Demo](demos/overview.gif "SQL-CLI in action")
```

## Option 3: Centered with HTML
```markdown
<p align="center">
  <img src="demos/overview.gif" alt="SQL-CLI Demo" width="800">
</p>
```

## Option 4: Multiple GIFs in a table
```markdown
| Feature | Demo |
|---------|------|
| Overview | ![Overview](demos/overview-optimized.gif) |
| Fuzzy Search | ![Fuzzy Search](demos/fuzzy-filter.gif) |
| Column Navigation | ![Column Nav](demos/column-navigation.gif) |
```

## Option 5: Collapsible sections for large GIFs
```markdown
<details>
<summary>🎬 View Full Demo (6MB)</summary>

![Full SQL-CLI Demo](demos/overview.gif)

</details>
```

## Option 6: Link to GIF (saves README load time)
```markdown
[![SQL-CLI Demo](demos/overview-thumbnail.png)](demos/overview.gif)
```
Or with text link:
```markdown
[📺 View Demo GIF (6MB)](demos/overview.gif)
```

## Option 7: Side-by-side smaller GIFs
```markdown
<table>
<tr>
<td width="50%">

### Fuzzy Search
![Fuzzy Search](demos/fuzzy-small.gif)

</td>
<td width="50%">

### SQL Completion  
![SQL Completion](demos/completion-small.gif)

</td>
</tr>
</table>
```

## Best Practices for GitHub README:

1. **Keep main GIF under 2-3MB** for fast loading
2. **Place it near the top** after the title and badges
3. **Use relative paths** (demos/file.gif not absolute URLs)
4. **Consider a static thumbnail** that links to the full GIF
5. **Multiple smaller GIFs** often work better than one large one

## Example README structure:

```markdown
# SQL-CLI

A powerful SQL interface with vim-style navigation and fuzzy search.

![SQL-CLI Demo](demos/overview-optimized.gif)

## Features

- 🔍 Fuzzy filtering with exact match mode
- ⌨️ Vim-style navigation 
- 📊 Column statistics
- 🔄 SQL autocomplete

## Full Demos

<details>
<summary>Click to view all demos</summary>

### Fuzzy Search
![Fuzzy Search Demo](demos/fuzzy-filter.gif)

### Column Navigation  
![Column Navigation Demo](demos/column-navigation.gif)

### SQL Queries
![SQL Queries Demo](demos/sql-queries.gif)

</details>
```
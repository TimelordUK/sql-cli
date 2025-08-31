# Known Issues

## 🐛 Bugs

### 1. Column Duplication After Unpinning
**Status**: Open  
**Priority**: Low (has workaround)  
**Reported**: 2025-08-15

**Description**: 
When pinning columns and then unpinning them, the columns appear to be duplicated in the view. For example:
- Pin 2 columns → works correctly
- Unpin those columns → end up with 4 columns (2 pinned + 2 regular copies)

**Expected Behavior**: 
Unpinning should restore the original column layout without duplication.

**Workaround**: 
Re-run the query (very fast now) to reset the view.

**Likely Cause**: 
The unpin logic in DataView might be adding columns back to the regular column list when they're already there.

---

## 💡 Future Optimizations

### 1. String Interning for Low-Cardinality Columns
**Status**: Idea  
**Priority**: Medium  
**Impact**: Memory usage

**Description**: 
For columns with very few unique values (e.g., status fields with "active"/"inactive", country codes), we currently store duplicates for every row. With 100k rows and only 2-5 unique values, this wastes significant memory.

**Potential Implementation**:
- Detect low-cardinality columns during data load
- Use string interning (like C#'s String.Intern)
- Store indices/references instead of duplicated strings
- Could reduce memory by 90%+ for these columns

**Example**:
```
100k rows × "United States" (13 bytes) = 1.3MB
vs
100k rows × 1 byte index + 1 × "United States" = 100KB + 13 bytes
```

---

## ✅ Recently Fixed

- DataView performance (588x improvement!)
- Pinned columns visual indicators
- Row vs Cell mode visual distinction
- Navigation through action system
- Vim-style count support (5j, 10k, etc.)

---

## 📝 Notes

The TUI is now production-ready for daily use. Focus is shifting to:
1. Completing key extraction refactor
2. Redux-style state management
3. Then circle back to fix minor bugs and optimizations

### Development Philosophy
- Don't get distracted by minor bugs that have easy workarounds
- The TUI is "good enough" for full-time use now
- Query execution is so fast that re-running is a valid solution
- Focus on architectural improvements first, polish later
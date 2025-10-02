# Function Migration TODO

## Functions Successfully Migrated to Registry
✅ Mathematical Functions:
- ROUND, ABS, FLOOR, CEILING, MOD, QUOTIENT
- SQRT, EXP, LN, LOG, LOG10, POWER
- DEGREES, RADIANS

✅ String Functions:
- MID, UPPER, LOWER, TRIM, TEXTJOIN

✅ Constants:
- TAU, PHI, HBAR (added to existing PI, E, ME)

## Remaining Functions in arithmetic_evaluator.rs

### Date/Time Functions (Complex - need special handling)
These functions require access to system time or complex date parsing:
- **NOW()** - Returns current date and time (needs system time)
- **TODAY()** - Returns current date (needs system date)
- **DATEDIFF(unit, date1, date2)** - Complex date arithmetic
- **DATEADD(unit, amount, date)** - Complex date arithmetic

### Unit Conversion Function (Complex - needs unit database)
- **CONVERT(value, from_unit, to_unit)** - Requires comprehensive unit conversion logic

### Already Removed from arithmetic_evaluator.rs
These have been successfully removed:
- EULER (duplicate of E, which is in registry)
- All MASS_* functions for planets (already in astronomy module)

## Notes
- Date functions may need their own module with proper date parsing
- CONVERT function needs a unit conversion system
- Consider whether date functions should remain in evaluator for performance
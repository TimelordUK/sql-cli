# String Methods Registry Refactoring

## Current State
- String methods are hardcoded in WHERE parser (Contains, StartsWith, EndsWith, ToUpper, ToLower, etc.)
- Each method creates a specific WhereExpr variant
- Evaluator has specific code for each variant

## New Architecture
We've created a registry-based system for string methods that allows:
1. Dynamic registration of new methods
2. Unified evaluation through SqlFunction trait
3. Both function-style and method-style calling

## Implementation Status

### ✅ Completed
1. Created `MethodFunction` trait extending `SqlFunction`
2. Implemented all string methods as registry functions
3. Added method registry to `FunctionRegistry`
4. Methods can be called as functions: `TOUPPER(name)`

### 🔄 In Progress  
Need to connect WHERE parser to use the registry

### 📋 TODO
1. Modify WHERE parser to check registry first
2. Create generic WhereExpr::MethodCall variant
3. Update evaluator to use registry for method evaluation
4. Gradual migration from hardcoded to registry-based

## Migration Strategy

### Phase 1: Parallel Implementation (Current)
- Keep existing hardcoded methods working
- Add registry-based methods alongside
- Both work independently

### Phase 2: Registry Integration
```rust
// In WHERE parser
if let Some(method_func) = registry.get_method(&method_name) {
    // Use registry-based method
    return Ok(WhereExpr::MethodCall {
        column,
        method: method_name,
        args: parsed_args,
    });
} else {
    // Fall back to hardcoded methods
    match method.as_str() {
        "Contains" => { /* existing code */ }
        // ...
    }
}
```

### Phase 3: Evaluator Update
```rust
// In recursive_where_evaluator
WhereExpr::MethodCall { column, method, args } => {
    let method_func = registry.get_method(method)?;
    let column_value = get_column_value(column);
    let result = method_func.evaluate_method(column_value, args)?;
    // Convert result to boolean for WHERE
}
```

### Phase 4: Deprecate Hardcoded
- Remove hardcoded method variants from WhereExpr
- All methods go through registry
- Clean, extensible architecture

## Benefits

1. **Extensibility**: Add new methods without touching parser
2. **Consistency**: Same methods work in SELECT and WHERE
3. **Testing**: Test methods independently
4. **Documentation**: Auto-generated from function signatures
5. **Type Safety**: Compile-time checking via traits

## Example Usage

After full implementation:
```sql
-- Method style (WHERE clause)
SELECT * FROM users WHERE name.Contains('john')
SELECT * FROM users WHERE email.EndsWith('.com')
SELECT * FROM users WHERE status.ToUpper() = 'ACTIVE'

-- Function style (SELECT clause)  
SELECT TOUPPER(name), LENGTH(email) FROM users
SELECT CONTAINS(description, 'important') as has_keyword FROM tasks

-- Both styles work everywhere
SELECT * FROM users WHERE CONTAINS(email, '@gmail')
SELECT name.ToUpper() as uppercase_name FROM users
```

## New Methods to Add

With the registry system, we can easily add:
- `Trim()`, `TrimStart()`, `TrimEnd()`
- `Replace(old, new)`
- `Substring(start, length)`
- `IndexOf(substring)`
- `PadLeft(length, char)`, `PadRight(length, char)`
- `Reverse()`
- `Split(delimiter)` - returns array
- Regular expression methods

## Implementation Notes

### Registry Access
The main challenge is getting the registry to the WHERE parser and evaluator.
Options:
1. Pass registry as parameter (requires API changes)
2. Global/static registry (less flexible)
3. Store in parent context that creates parser

### Type Conversion
Methods return DataValue, but WHERE needs boolean results.
Need conversion rules:
- Boolean values: use directly
- Numeric: 0 is false, non-zero is true
- String: empty is false, non-empty is true
- Null: always false

### Case Sensitivity
The registry handles case-insensitive method lookup via `handles_method()`.
This maintains compatibility with existing case-insensitive mode.
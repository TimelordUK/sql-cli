# SQL CLI Documentation

## Documentation Structure

### 📁 [/refactoring](./refactoring/)
Current refactoring efforts and cleanup progress
- `REFACTORING_PROGRESS.md` - Current TUI refactoring status (7.7k lines, visitor pattern)
- `CLEANUP_PROGRESS.md` - Overall cleanup tracking
- `CLEANUP_SUMMARY.md` - Cleanup achievements summary
- `TUI_DATAVIEW_MIGRATION.md` - TUI to DataView migration notes
- `KEY_EXTRACTION_PLAN.md` - Key handler extraction planning
- `REDUX_ARCHITECTURE_PLAN.md` - Redux-style architecture proposal

### 📁 [/migration](./migration/)
Migration tracking for various subsystems
- `KEY_MIGRATION_*.md` - Key handler migration progress
- `DATAVIEW_COMPLETE.md` - DataView migration completion
- `VIEWPORT_*.md` - Viewport manager refactoring
- `TAB_SWITCHING.md` - Tab switching implementation

### 📁 [/debug](./debug/)
Debugging guides and memory analysis
- `DEBUG_GUIDE.md` - Comprehensive debugging guide
- `DEBUGGING.md` - Debugging tips and tools
- `ACTION_DEBUG_TOOLS.md` - Action system debugging
- `MEMORY_ANALYSIS.md` - Memory usage analysis
- `analyze_memory.md` - Memory profiling notes

### 📁 [/architecture](./architecture/)
Architecture decisions and version histories
- `V*.md` - Version-specific architecture notes (V47-V49)
- `EXECUTE_QUERY_CLEAN.md` - Query execution architecture
- `FUZZY_FILTER_CLEAN.md` - Fuzzy filter implementation
- `PHASE_2_SUMMARY.md` - Phase 2 refactoring summary

### 📁 [/config](./config/)
Configuration documentation

### 📁 [/design](./design/)
Design documents and proposals

### 📄 Root Documentation Files
- `ISSUES.md` - Known issues tracking
- `COLUMN_OPERATIONS_TODO.md` - Column operations roadmap
- `EDITING_TEST_SUMMARY.md` - Test coverage summary

## Key Documents

### 🚀 Active Work
- [Current Refactoring Progress](./refactoring/REFACTORING_PROGRESS.md) - TUI from 10k → 7.7k lines
- [Column Operations Analysis](./COLUMN_OPERATIONS_ANALYSIS.md)
- [Input Behavior Analysis](./INPUT_BEHAVIOR_ANALYSIS.md)

### 📚 Reference
- [Master Refactoring Roadmap](./MASTER_REFACTORING_ROADMAP.md)
- [State Management Refactor](./STATE_MANAGEMENT_REFACTOR.md)
- [DataTable Implementation Strategy](./DATATABLE_IMPLEMENTATION_STRATEGY.md)

### 🎯 Future Work
- [Excel Integration Roadmap](./EXCEL_INTEGRATION_ROADMAP.md)
- [Register System Design](./REGISTER_SYSTEM_DESIGN.md)
- [Event-Driven Architecture](./event-driven-architecture.md)

## Quick Links
- [Features List](./FEATURES.md)
- [Release Notes](./RELEASE_NOTES.md)
- [Critical Issues](./CRITICAL_ISSUES.md)

## Documentation Standards
- Use `.md` extension for all documentation
- Include date in version-specific docs
- Keep active work docs updated
- Archive completed work to appropriate folders
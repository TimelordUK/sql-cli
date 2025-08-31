# Claude Code Agents for SQL CLI

This directory contains specialized agents for the SQL CLI project. Each agent is designed to handle specific types of tasks efficiently.

## Available Agents

### 1. rust-build-fixer
**Purpose**: Fix Rust compilation errors and ensure code formatting compliance  
**Color**: Yellow  
**When to use**: 
- Compilation errors after code changes
- Cargo build/test failures
- Code formatting issues
- Clippy warnings

**Example usage**:
```
Main Agent: "I need to add a new sorting feature"
After implementing: "Let me use the rust-build-fixer agent to ensure it compiles correctly"
```

### 2. rust-test-failure-investigator  
**Purpose**: Investigate and fix failing Rust tests  
**Color**: Blue  
**When to use**:
- Cargo test failures
- Unit test regressions
- Integration test issues
- Test logic problems

**Example usage**:
```
User: "cargo test failed with 3 failures in the data_view module"
Main Agent: "I'll use the rust-test-failure-investigator agent to analyze and fix these failures"
```

### 3. debug-analyzer (NEW)
**Purpose**: Analyze F5 debug output to identify issues and extract insights  
**Color**: Purple  
**When to use**:
- State inconsistencies between components
- Performance bottlenecks in TUI operations  
- Event flow issues in key handling
- Viewport navigation problems
- Search/filter synchronization issues

**Example usage**:
```
User: "The cursor jumps around when I navigate. Here's F5 debug output: [large dump]"
Main Agent: "Let me use the debug-analyzer agent to parse this debug output and identify the root cause"
```

## Debug-Analyzer Agent Capabilities

The debug-analyzer agent is specifically designed for the SQL CLI's F5 debug output and can:

### State Inconsistency Detection
- **Mode Stack Issues**: Detect mode_stack vs current_mode conflicts
- **Search State Conflicts**: Find search pattern/result mismatches  
- **Viewport Synchronization**: Identify crosshair vs viewport boundary issues
- **Column State Problems**: Detect visible/hidden/pinned column inconsistencies
- **Buffer Misalignment**: Find buffer_index vs active data problems

### Performance Bottleneck Analysis  
- **Slow Operations**: Analyze search_time, filter_time durations
- **Memory Pressure**: Examine buffer sizes and cache efficiency
- **Rendering Bottlenecks**: Identify viewport calculation overhead
- **Query Performance**: Analyze SQL parsing/evaluation times

### Event Flow Issue Detection
- **Key Handler Chain**: Trace key events through handler system
- **Action Processing**: Verify actions reach correct handlers
- **State Transitions**: Identify invalid/incomplete transitions
- **Debouncing Issues**: Find search/filter debouncing problems

## Debug Output Sections Analyzed

The debug-analyzer understands these F5 output sections:

```
=== APP STATE CONTAINER DEBUG DUMP ===
MODE INFORMATION:
UI FLAGS: 
HELP STATE:
INPUT STATE:
SEARCH STATE:
FILTER STATE:
COLUMN SEARCH STATE:

=== BUFFER DEBUG DUMP ===
Buffer Information:
VISIBLE COLUMNS:
HIDDEN COLUMNS: 
PINNED COLUMNS:
SORT STATE:

VIEWPORT MANAGER STATE:
PERFORMANCE METRICS:
```

## How to Use the Debug-Analyzer Agent

### Step 1: Reproduce the Issue
1. Run the SQL CLI with your data
2. Reproduce the problematic behavior  
3. Press F5 to generate debug output
4. Copy the complete debug dump

### Step 2: Invoke the Agent
Provide both the problem description and the raw debug output:

```
Problem: "Sort indicators show 'Ascending' but data appears unsorted"
Debug Output: [Paste complete F5 dump here]
```

### Step 3: Review Analysis
The agent will provide:
- **Problem Summary**: Core issue identification
- **Debug Evidence**: Specific problematic values  
- **Root Cause Analysis**: Technical explanation
- **Recommended Actions**: Specific fixes to try
- **Prevention Strategies**: Future improvements

## Integration with Main Agent Workflow

The main agent should delegate to debug-analyzer when:
1. User provides F5 debug output with a problem report
2. TUI behavior doesn't match expected state
3. Performance issues need debug data analysis
4. Complex state synchronization problems occur

This creates an efficient workflow where the main agent focuses on implementation while the debug-analyzer provides specialized debug output interpretation.
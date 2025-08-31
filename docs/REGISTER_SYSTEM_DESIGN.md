# Vim-Style Register System Design

## Overview
A vim-style register system for marking, yanking, and navigating cells in the TUI. This would enhance the data exploration workflow by allowing users to mark multiple cells across large datasets and manage multiple clipboard-like registers.

## User Commands

### Register Selection
- `"a` → select register 'a' for next operation
- `"b` → select register 'b' for next operation
- `"1` through `"9` → numbered registers
- `""` → default/unnamed register

### Marking Commands
- `ma` → mark current cell position as 'a'
- `mb` → mark current cell position as 'b'
- `m1` through `m9` → numbered marks
- `'a` → jump to mark 'a'
- `'b` → jump to mark 'b'

### Yank Operations
- `ya` → yank current cell/row/selection to register 'a'
- `"ayy` → yank current row to register 'a'
- `"ayw` → yank current cell to register 'a'
- `"a5yy` → yank 5 rows to register 'a'

### Paste Operations
- `"ap` → paste from register 'a' (display/export)
- `"bp` → paste from register 'b'

## Architecture Design

### RegisterManager
```rust
pub struct RegisterManager {
    marks: HashMap<char, CellPosition>,      // 'a' -> (row, col)
    registers: HashMap<char, RegisterContent>, // 'a' -> content
    selected_register: Option<char>,         // currently selected register
    default_register: RegisterContent,      // unnamed register
}

#[derive(Clone)]
pub struct CellPosition {
    row: usize,
    col: usize,
    timestamp: SystemTime,  // when mark was created
}

#[derive(Clone)]
pub struct RegisterContent {
    content: String,
    content_type: RegisterType,
    source_position: Option<CellPosition>,
    timestamp: SystemTime,
}

#[derive(Clone)]
pub enum RegisterType {
    Cell(String),           // single cell value
    Row(Vec<String>),       // entire row
    Column(Vec<String>),    // entire column
    Selection(Vec<Vec<String>>), // rectangular selection
    Query(String),          // SQL query text
}

impl RegisterManager {
    pub fn new() -> Self
    pub fn set_mark(&mut self, register: char, position: CellPosition)
    pub fn get_mark(&self, register: char) -> Option<&CellPosition>
    pub fn jump_to_mark(&self, register: char) -> Option<CellPosition>
    pub fn yank_to_register(&mut self, register: char, content: RegisterContent)
    pub fn get_register_content(&self, register: char) -> Option<&RegisterContent>
    pub fn select_register(&mut self, register: char)
    pub fn get_selected_register(&self) -> Option<char>
    pub fn list_marks(&self) -> Vec<(char, CellPosition)>
    pub fn list_registers(&self) -> Vec<(char, &RegisterContent)>
    pub fn clear_register(&mut self, register: char)
    pub fn clear_all_marks(&mut self)
}
```

### ViewportManager Integration
```rust
impl ViewportManager {
    pub fn highlight_marks(&self, marks: &[(char, CellPosition)]) -> Vec<MarkHighlight>
    pub fn render_register_indicators(&self, frame: &mut Frame, area: Rect)
    pub fn get_marked_cells_in_viewport(&self, marks: &HashMap<char, CellPosition>) -> Vec<VisibleMark>
    pub fn navigate_to_mark(&mut self, position: CellPosition) -> Result<()>
}

#[derive(Clone)]
pub struct MarkHighlight {
    visual_row: usize,
    visual_col: usize,
    register: char,
    style: Style,
}

#[derive(Clone)]
pub struct VisibleMark {
    register: char,
    position: CellPosition,
    is_in_viewport: bool,
}
```

### Enhanced Renderer
```rust
impl EnhancedTuiApp {
    fn render_marked_cells(&self, frame: &mut Frame, area: Rect)
    fn render_register_status(&self, frame: &mut Frame, area: Rect)
    fn apply_mark_styling(&self, cell: &Cell, register: Option<char>) -> Cell
}
```

## Visual Indicators

### Cell Highlighting
- Marked cells show small register indicator in corner: `ᵃ ᵇ ¹ ²`
- Different colors for different register types:
  - Letters (a-z): Blue background
  - Numbers (1-9): Green background  
  - Current selection: Yellow highlight

### Status Line Integration
```
[Row 45/1000] [Col 12/25] Registers: a→(45,12) b→(67,8) | Selected: "a
```

### Register Display Panel (F6 or :registers)
```
--- REGISTERS ---
"  (default): "Product Name: Widget X"
a  mark(45,12): "Widget X" 
b  mark(67,8):  "2024-01-15"
1  row(23):     "ID:123, Name:Widget, Price:$49.99"
2  selection:   "3x2 cell range from (10,5)"
--- MARKS ---
a  → Row 45, Col 12  (Product Name)
b  → Row 67, Col 8   (Created Date)  
```

## Implementation Phases

### Phase 1: Basic Register System
1. Implement `RegisterManager` core functionality
2. Add basic mark/yank/register commands to action system
3. Simple visual indicators for marked cells

### Phase 2: ViewportManager Integration  
1. Integrate with viewport for mark highlighting
2. Navigation to marks with viewport updates
3. Mark persistence across data changes

### Phase 3: Enhanced UX
1. Register display panel
2. Status line integration
3. Visual styling and animations
4. Export/import register contents

### Phase 4: Advanced Features
1. Rectangular selections across registers
2. Register history and undo
3. Named registers with descriptions
4. Register-based filtering and search

## Use Cases

### Data Exploration Workflow
```bash
# Mark interesting cells while exploring
ma          # mark current cell as 'a'
/price      # search for price column  
mb          # mark price cell as 'b'
'a          # jump back to mark 'a'
"ayy        # yank current row to register 'a'
'b          # jump to mark 'b'  
"byy        # yank current row to register 'b'
:registers  # view all marked content
```

### Multi-Cell Comparison
```bash
# Compare values across dataset
ma          # mark cell 1
10j         # move down 10 rows  
mb          # mark cell 2
20j         # move down 20 rows
mc          # mark cell 3
'a          # jump to first mark
"ay         # yank to register a
'b          # jump to second mark  
"by         # yank to register b
'c          # jump to third mark
"cy         # yank to register c
:registers  # compare all three values
```

### Data Validation Workflow
```bash
# Mark suspicious data points
ma mb mc md  # mark multiple cells
"ayyyy       # yank 4 rows from mark a location
# Export registers for analysis
:export registers data_validation.txt
```

## Prerequisites
- Key migration system completion
- Action system maturity
- Stable viewport manager
- Enhanced renderer capabilities

## Future Enhancements
- Register sharing between buffers
- Persistent registers across sessions
- Register-based macros and automation
- Integration with external clipboard
- Register-based data export formats

---

**Status:** Design Phase  
**Dependencies:** Key Migration v2 completion  
**Priority:** Medium (post key-migration)  
**Estimated Effort:** 2-3 weeks for full implementation
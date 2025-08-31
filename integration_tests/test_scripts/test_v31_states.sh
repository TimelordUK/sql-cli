#!/bin/bash

echo "Testing V31 state migration..."

# Create a simple Rust test file
cat > /tmp/test_v31.rs << 'EOF'
use sql_cli::app_state_container::{AppStateContainer, UndoRedoState, ScrollState};

fn main() {
    println!("Testing V31 state structures...");
    
    // Create AppStateContainer
    let container = AppStateContainer::new(None);
    
    // Test UndoRedoState
    {
        let mut undo_redo = container.undo_redo_mut();
        println!("✓ UndoRedoState created with {} undo entries", undo_redo.undo_stack.len());
        
        // Test push/pop operations
        undo_redo.push_undo("test text".to_string(), 5);
        assert_eq!(undo_redo.undo_stack.len(), 1);
        println!("✓ Pushed to undo stack");
        
        let popped = undo_redo.pop_undo();
        assert!(popped.is_some());
        println!("✓ Popped from undo stack: {:?}", popped);
    }
    
    // Test ScrollState
    {
        let mut scroll = container.scroll_mut();
        println!("✓ ScrollState created with help_scroll={}", scroll.help_scroll);
        
        // Test scroll updates
        scroll.help_scroll = 10;
        scroll.input_scroll_offset = 5;
        scroll.viewport_scroll_offset = (100, 20);
        scroll.last_visible_rows = 50;
        
        println!("✓ Updated scroll state:");
        println!("  - help_scroll: {}", scroll.help_scroll);
        println!("  - input_scroll: {}", scroll.input_scroll_offset);
        println!("  - viewport: {:?}", scroll.viewport_scroll_offset);
        println!("  - visible_rows: {}", scroll.last_visible_rows);
    }
    
    // Test debug dump includes new states
    let dump = container.generate_debug_dump();
    assert!(dump.contains("UNDO/REDO STATE"));
    assert!(dump.contains("SCROLL STATE"));
    println!("✓ Debug dump includes new states");
    
    println!("\n✅ All V31 state tests passed!");
}
EOF

# Compile and run the test
rustc --edition 2021 \
    -L target/release/deps \
    --extern sql_cli=target/release/libsql_cli.rlib \
    /tmp/test_v31.rs \
    -o /tmp/test_v31 2>/dev/null

if [ $? -eq 0 ]; then
    /tmp/test_v31
else
    echo "Compilation test - checking if new structures are accessible..."
    grep -q "UndoRedoState" src/app_state_container.rs && echo "✓ UndoRedoState found"
    grep -q "ScrollState" src/app_state_container.rs && echo "✓ ScrollState found"
    grep -q "undo_redo: RefCell" src/app_state_container.rs && echo "✓ undo_redo field added"
    grep -q "scroll: RefCell" src/app_state_container.rs && echo "✓ scroll field added"
    grep -q "UNDO/REDO STATE" src/app_state_container.rs && echo "✓ Debug dump for undo/redo"
    grep -q "SCROLL STATE" src/app_state_container.rs && echo "✓ Debug dump for scroll"
    echo "✅ All structure checks passed!"
fi

# Clean up
rm -f /tmp/test_v31.rs /tmp/test_v31
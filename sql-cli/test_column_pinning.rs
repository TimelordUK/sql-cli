// Test file for column pinning feature documentation

fn main() {
    println!("Testing Column Pinning Feature:");
    println!();
    println!("Column pinning allows you to keep important columns visible while scrolling.");
    println!();
    println!("How to use:");
    println!("  p     - Pin/unpin the current column (max 4 pinned columns)");
    println!("  P     - Clear all pinned columns");
    println!();
    println!("Visual indicators:");
    println!("  📌    - Appears before pinned column headers");
    println!("  [*]   - Marks the currently selected column");
    println!();
    println!("Navigation:");
    println!("  h/l   - Move left/right between columns");
    println!("  ^     - Jump to first column");
    println!("  $     - Jump to last column");
    println!();
    println!("Example workflow:");
    println!("1. Navigate to an important column (like 'ID' or 'Name')");
    println!("2. Press 'p' to pin it");
    println!("3. Navigate to another column and pin it");
    println!("4. Scroll horizontally - pinned columns stay visible");
    println!("5. Press 'P' to unpin all columns");
    println!();
    println!("The table header shows:");
    println!("  - Number of pinned columns");
    println!("  - Total visible columns");
    println!("  - Total number of columns");
}

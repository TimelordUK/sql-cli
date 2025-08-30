#!/usr/bin/env python3
"""
Test script for the debug-analyzer agent.
Generates sample F5 debug output and tests the agent's analysis capabilities.
"""

# Sample F5 debug output representing different types of issues
SAMPLE_DEBUG_OUTPUTS = {
    "viewport_navigation_issue": """
=== APP STATE CONTAINER DEBUG DUMP ===

MODE INFORMATION:
  Current Mode: Normal
  Mode Stack: [Normal]

UI FLAGS:
  Debug Enabled: true

HELP STATE:
  Visible: false
  Scroll Offset: 0
  Max Scroll: 100
  Open Count: 0

INPUT STATE:
  Text: 'select * from data'
  Cursor: 17
  Last Query: 'select * from data'

SEARCH STATE:
  [Inactive]

FILTER STATE:
  [Inactive]
  Total Filters: 0
  History Items: 0

COLUMN SEARCH STATE (INACTIVE):

=== BUFFER DEBUG DUMP ===

Buffer Information:
  Type: DataTableBuffer
  Row Count: 1000
  Column Count: 25

VISIBLE COLUMNS (25): [id, name, book, cat, price, qty, status, loc, desc, notes, time, user, ext_id, fees, total, col16, col17, col18, col19, col20, col21, col22, col23, col24, col25]
HIDDEN COLUMNS: []
PINNED COLUMNS: []

SORT STATE:
  Column: None
  Order: None

VIEWPORT MANAGER STATE:
  Crosshair: row=5, col=24
  Viewport: start_row=0, start_col=20
  Terminal: width=80, height=30
  Available columns for display: 5
  Scroll offset: col=20

PERFORMANCE METRICS:
  Last navigation: 250ms ago
  Render time: 15ms
  Buffer operations: 0ms
    """.strip(),
    
    "state_sync_issue": """
=== APP STATE CONTAINER DEBUG DUMP ===

MODE INFORMATION:
  Current Mode: Normal
  Mode Stack: [Normal, Search]

UI FLAGS:
  Debug Enabled: true

SEARCH STATE:
  Pattern: 'admin'
  Matches: 5 found
  Current: 2 of 5
  Search time: 45ms ago

FILTER STATE:
  Pattern: 'status=Avail'
  Filtered Rows: 800
  Case Insensitive: false
  Last Filter: 120ms ago
  Total Filters: 1

=== BUFFER DEBUG DUMP ===

SORT STATE:
  Column: price
  Order: Desc

VIEWPORT MANAGER STATE:
  Crosshair: row=15, col=4
  Current match position: row=25, col=11
  Viewport showing: rows 0-30

PERFORMANCE METRICS:
  Search operation: 125ms
  Filter operation: 89ms
  State sync: 234ms
    """.strip(),
    
    "performance_issue": """
=== APP STATE CONTAINER DEBUG DUMP ===

SEARCH STATE:
  Pattern: 'engineering'
  Matches: 2500 found
  Current: 1250 of 2500
  Search time: 2.5s ago

FILTER STATE:
  Pattern: 'dept like %eng% and salary > 50000'
  Filtered Rows: 15000
  Case Insensitive: true
  Last Filter: 3.2s ago
  Total Filters: 5

=== BUFFER DEBUG DUMP ===

Buffer Information:
  Row Count: 100000
  Column Count: 50

PERFORMANCE METRICS:
  Search operation: 2500ms
  Filter operation: 3200ms
  Render time: 450ms
  Memory usage: 250MB
  Cache hits: 45%
  Cache misses: 55%
    """.strip()
}

def test_debug_analyzer():
    """Test the debug-analyzer agent with different scenarios"""
    
    print("=== Debug Analyzer Agent Test ===\n")
    
    for scenario, debug_output in SAMPLE_DEBUG_OUTPUTS.items():
        print(f"Scenario: {scenario.replace('_', ' ').title()}")
        print("-" * 50)
        
        # Simulate what the main agent would send to debug-analyzer
        problem_description = {
            "viewport_navigation_issue": "User reports cursor jumping to wrong position when navigating right with 'l' key",
            "state_sync_issue": "Search highlights don't match the actual cursor position",
            "performance_issue": "Application becomes very slow when searching large datasets"
        }[scenario]
        
        print(f"Problem: {problem_description}")
        print(f"Debug Output Length: {len(debug_output)} characters")
        print("\nKey sections identified:")
        
        # Simulate basic parsing that the agent would do
        if "VIEWPORT MANAGER STATE" in debug_output:
            print("✓ Found viewport state information")
        if "PERFORMANCE METRICS" in debug_output:
            print("✓ Found performance timing data")
        if "SEARCH STATE" in debug_output:
            print("✓ Found search state information")
        if "Mode Stack:" in debug_output:
            print("✓ Found mode stack information")
            
        print(f"\nAgent would analyze this debug output for {scenario} patterns...")
        print("=" * 60)
        print()

if __name__ == "__main__":
    test_debug_analyzer()
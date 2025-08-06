# VHS script for SQL query capabilities

Output demos/sql-queries.gif

Set FontSize 16
Set Width 1100
Set Height 700
Set Theme "One Dark"

# Start sql-cli
Type "sql-cli"
Enter
Sleep 1s

# Load trade data
Type ":cache load 7"
Enter
Sleep 2s

# Basic select
Type "select * from data limit 5"
Enter
Sleep 2s

# Filtering
Type "select instrumentid, price, quantity from data where price > 100 limit 10"
Enter
Sleep 3s

# Aggregation
Type "select instrumentid, avg(price) as avg_price, sum(quantity) as total_qty from data group by instrumentid limit 10"
Enter
Sleep 3s

# Complex query with CTEs
Type @"with high_value_trades as (
  select * from data where price * quantity > 10000
)
select instrumentid, count(*) as trade_count, avg(price) as avg_price 
from high_value_trades 
group by instrumentid 
order by trade_count desc 
limit 10"@
Enter
Sleep 4s

# Show TUI for results
Type ":tui"
Enter
Sleep 2s

# Navigate results
Type "j" Sleep 200ms
Type "j" Sleep 200ms
Type "l" Sleep 200ms
Type "l" Sleep 200ms

# Exit TUI
Type "q"
Sleep 1s

# Exit sql-cli
Type ":exit"
Enter
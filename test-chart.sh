 ./target/release/sql-cli-chart data/production_vwap_final.csv \
    -q "SELECT snapshot_time, average_price FROM production_vwap_final WHERE filled_quantity > 0" \
    -x snapshot_time \
    -y average_price \
    -t "VWAP Price Over Time"


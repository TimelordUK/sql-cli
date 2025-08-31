 ./target/release/sql-cli-chart data/production_vwap_final.csv \
   -q "SELECT snapshot_time, filled_quantity FROM production_vwap_final WHERE order_type.Contains('client')" \
    -x snapshot_time \
    -y filled_quantity \
    -t "Client Order Fill Volume"


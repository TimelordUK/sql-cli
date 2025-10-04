-- Test data model rendering
local data_model = require('sql-cli.data_model')
local viewport = require('sql-cli.viewport')
local renderer = require('sql-cli.renderer')

-- Create mock JSON data
local json_data = {
  columns = {
    {name = "id", type = "Integer", max_width = 5, alignment = "right"},
    {name = "name", type = "String", max_width = 10, alignment = "left"},
    {name = "value", type = "Float", max_width = 8, alignment = "right"},
  },
  rows = {
    {"1", "Alice", "100.5"},
    {"2", "Bob", "200.3"},
    {"3", "Charlie", "150.7"},
    {"4", "David", "300.2"},
    {"5", "Eve", "250.9"},
  },
  metadata = {
    total_rows = 5,
    query_time_ms = 1.5
  }
}

-- Create data model
local model = data_model.DataModel:new(json_data)
print(string.format("Model: %d rows × %d cols", model.total_rows, model.total_cols))

-- Create viewport
local vp = viewport.Viewport:new(model, {visible_rows = 10, theme = "ascii"})
print(string.format("Viewport at row %d, col %d", vp.current_row, vp.current_col))

-- Test movement
print("\nMoving right...")
vp:move_cursor(0, 1)
print(string.format("Now at row %d, col %d", vp.current_row, vp.current_col))

print("\nMoving down...")
vp:move_cursor(1, 0)
print(string.format("Now at row %d, col %d", vp.current_row, vp.current_col))

-- Test get_visible_data
print("\nGetting visible data...")
local visible = vp:get_visible_data()
print(string.format("Got %d rows", #visible.rows))

print("\nTest completed successfully!")

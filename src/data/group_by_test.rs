#[cfg(test)]
mod tests {
    use super::super::data_view::{DataView, GroupKey};
    use super::super::datatable::{DataColumn, DataRow, DataTable, DataValue};
    use std::sync::Arc;

    #[test]
    fn test_simple_group_by() {
        // Create test data
        let mut table = DataTable::new("trades");
        table.add_column(DataColumn::new("trader"));
        table.add_column(DataColumn::new("book"));
        table.add_column(DataColumn::new("quantity"));
        table.add_column(DataColumn::new("price"));

        // Add test rows
        table
            .add_row(DataRow::new(vec![
                DataValue::String("Alice".to_string()),
                DataValue::String("BookA".to_string()),
                DataValue::Integer(100),
                DataValue::Float(50.5),
            ]))
            .unwrap();

        table
            .add_row(DataRow::new(vec![
                DataValue::String("Bob".to_string()),
                DataValue::String("BookA".to_string()),
                DataValue::Integer(200),
                DataValue::Float(51.0),
            ]))
            .unwrap();

        table
            .add_row(DataRow::new(vec![
                DataValue::String("Alice".to_string()),
                DataValue::String("BookB".to_string()),
                DataValue::Integer(150),
                DataValue::Float(49.5),
            ]))
            .unwrap();

        table
            .add_row(DataRow::new(vec![
                DataValue::String("Bob".to_string()),
                DataValue::String("BookB".to_string()),
                DataValue::Integer(250),
                DataValue::Float(52.0),
            ]))
            .unwrap();

        table
            .add_row(DataRow::new(vec![
                DataValue::String("Alice".to_string()),
                DataValue::String("BookA".to_string()),
                DataValue::Integer(300),
                DataValue::Float(50.0),
            ]))
            .unwrap();

        // Create DataView and group by trader
        let view = DataView::new(Arc::new(table));
        let groups = view.group_by(&vec!["trader".to_string()]).unwrap();

        // Should have 2 groups (Alice and Bob)
        assert_eq!(groups.len(), 2);

        // Check Alice's group
        let alice_key = GroupKey(vec![DataValue::String("Alice".to_string())]);
        let alice_view = groups.get(&alice_key).unwrap();
        assert_eq!(alice_view.get_visible_rows().len(), 3); // Alice has 3 trades

        // Check Bob's group
        let bob_key = GroupKey(vec![DataValue::String("Bob".to_string())]);
        let bob_view = groups.get(&bob_key).unwrap();
        assert_eq!(bob_view.get_visible_rows().len(), 2); // Bob has 2 trades
    }

    #[test]
    fn test_multi_column_group_by() {
        // Create test data
        let mut table = DataTable::new("trades");
        table.add_column(DataColumn::new("trader"));
        table.add_column(DataColumn::new("book"));
        table.add_column(DataColumn::new("quantity"));

        // Add test rows
        table
            .add_row(DataRow::new(vec![
                DataValue::String("Alice".to_string()),
                DataValue::String("BookA".to_string()),
                DataValue::Integer(100),
            ]))
            .unwrap();

        table
            .add_row(DataRow::new(vec![
                DataValue::String("Alice".to_string()),
                DataValue::String("BookA".to_string()),
                DataValue::Integer(200),
            ]))
            .unwrap();

        table
            .add_row(DataRow::new(vec![
                DataValue::String("Alice".to_string()),
                DataValue::String("BookB".to_string()),
                DataValue::Integer(150),
            ]))
            .unwrap();

        table
            .add_row(DataRow::new(vec![
                DataValue::String("Bob".to_string()),
                DataValue::String("BookA".to_string()),
                DataValue::Integer(250),
            ]))
            .unwrap();

        // Create DataView and group by trader AND book
        let view = DataView::new(Arc::new(table));
        let groups = view
            .group_by(&vec!["trader".to_string(), "book".to_string()])
            .unwrap();

        // Should have 3 unique combinations
        assert_eq!(groups.len(), 3);

        // Check (Alice, BookA) group
        let alice_booka_key = GroupKey(vec![
            DataValue::String("Alice".to_string()),
            DataValue::String("BookA".to_string()),
        ]);
        let alice_booka_view = groups.get(&alice_booka_key).unwrap();
        assert_eq!(alice_booka_view.get_visible_rows().len(), 2); // 2 trades

        // Check (Alice, BookB) group
        let alice_bookb_key = GroupKey(vec![
            DataValue::String("Alice".to_string()),
            DataValue::String("BookB".to_string()),
        ]);
        let alice_bookb_view = groups.get(&alice_bookb_key).unwrap();
        assert_eq!(alice_bookb_view.get_visible_rows().len(), 1); // 1 trade

        // Check (Bob, BookA) group
        let bob_booka_key = GroupKey(vec![
            DataValue::String("Bob".to_string()),
            DataValue::String("BookA".to_string()),
        ]);
        let bob_booka_view = groups.get(&bob_booka_key).unwrap();
        assert_eq!(bob_booka_view.get_visible_rows().len(), 1); // 1 trade
    }
}

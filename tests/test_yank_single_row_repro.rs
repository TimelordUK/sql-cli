//! Regression tests for yank reporting "No row selected" on a visible cell.
//!
//! The yank handlers resolve their row from `Buffer::get_selected_row()`. That used to
//! be backed by ratatui's `TableState`, which is only written by row navigation and by
//! query execution - so a freshly loaded file, or a filter that leaves a single row
//! (where j/k can never fire), left it as `None` and `yv`/`yy` refused to copy anything.

use sql_cli::app_state_container::AppStateContainer;
use sql_cli::buffer::{Buffer, BufferAPI};
use sql_cli::data::data_view::DataView;
use sql_cli::data::datatable::{DataColumn, DataRow, DataTable, DataValue};
use sql_cli::ui::state::state_coordinator::StateCoordinator;
use sql_cli::ui::viewport_manager::ViewportManager;
use std::cell::RefCell;
use std::sync::Arc;

fn make_table() -> DataTable {
    let mut table = DataTable::new("versions");
    table.add_column(DataColumn::new("project"));
    table.add_column(DataColumn::new("version"));

    for (p, v) in [("alpha", "1.0.0"), ("nucleus", "2.3.4"), ("beta", "9.9.9")] {
        table
            .add_row(DataRow::new(vec![
                DataValue::String(p.to_string()),
                DataValue::String(v.to_string()),
            ]))
            .unwrap();
    }
    table
}

/// Mirrors `EnhancedTuiApp::new_with_dataview` / `add_dataview_with_refs`:
/// a buffer created straight from a loaded file, with no query executed.
fn make_container() -> AppStateContainer {
    let table = make_table();
    let mut buffer = Buffer::new(1);
    buffer.set_datatable(Some(Arc::new(table.clone())));
    buffer.set_dataview(Some(DataView::new(Arc::new(table))));

    // Default() rather than new(): it uses CommandHistory::default(), which doesn't
    // touch the shared history file and so can't race other tests.
    let mut container = AppStateContainer::default();
    container.buffers_mut().add_buffer(buffer);
    container.buffers_mut().switch_to(0);
    container.update_data_size(3, 2);
    container
}

fn viewport_for(container: &AppStateContainer) -> RefCell<Option<ViewportManager>> {
    RefCell::new(Some(ViewportManager::new(Arc::new(
        container.get_buffer_dataview().unwrap().clone(),
    ))))
}

fn selected_row(container: &AppStateContainer) -> Option<usize> {
    container.current_buffer().unwrap().get_selected_row()
}

#[test]
fn freshly_loaded_file_has_a_selected_row() {
    let container = make_container();
    assert_eq!(
        selected_row(&container),
        Some(0),
        "yank must work immediately after loading a file, without pressing j/k first"
    );
}

#[test]
fn empty_results_have_no_selected_row() {
    let mut container = make_container();
    let vm = viewport_for(&container);

    container.set_fuzzy_filter_pattern("zzzznomatch".to_string());
    let (count, _) = StateCoordinator::apply_fuzzy_filter_with_refs(&mut container, &vm);

    assert_eq!(count, 0);
    assert_eq!(
        selected_row(&container),
        None,
        "with no visible rows there is genuinely nothing to yank"
    );
}

#[test]
fn fuzzy_filter_to_single_row_keeps_a_selected_row() {
    let mut container = make_container();
    let vm = viewport_for(&container);

    container.set_fuzzy_filter_pattern("nucleus".to_string());
    let (count, _) = StateCoordinator::apply_fuzzy_filter_with_refs(&mut container, &vm);

    assert_eq!(count, 1);
    assert_eq!(selected_row(&container), Some(0));
}

#[test]
fn text_filter_to_single_row_keeps_a_selected_row() {
    let mut container = make_container();
    let vm = viewport_for(&container);

    let count = StateCoordinator::apply_text_filter_with_refs(&mut container, &vm, "nucleus");

    assert_eq!(count, 1, "text filter should narrow to the single match");
    assert_eq!(
        selected_row(&container),
        Some(0),
        "the 'f' filter must land on the first match like the fuzzy filter does"
    );
}

#[test]
fn selection_is_clamped_to_the_filtered_view() {
    let mut container = make_container();
    let vm = viewport_for(&container);

    // User navigated down to the last row before filtering.
    container.set_selected_row(Some(2));

    let count = StateCoordinator::apply_text_filter_with_refs(&mut container, &vm, "nucleus");
    assert_eq!(count, 1);

    let row = selected_row(&container).expect("one visible row means one selectable row");

    // This is the lookup YankManager::yank_cell performs; it must hit the real cell
    // rather than running off the end of the narrowed view and copying "NULL".
    let view = container.get_buffer_dataview().unwrap();
    assert_eq!(view.row_count(), 1);
    assert_eq!(
        view.get_cell_value(row, 1),
        Some("2.3.4".to_string()),
        "yank must read the visible row, not a stale index"
    );
}

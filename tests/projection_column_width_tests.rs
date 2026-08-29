//! Regression tests for column widths under a narrower-than-source projection.
//!
//! `DataView` projections keep the original `DataTable` and record source column
//! indices in `visible_columns`. Anything keyed by visual position - notably the
//! column width cache - must translate those indices first. Using a source index
//! directly reads past the end of the width vector and silently falls back to
//! DEFAULT_COL_WIDTH (15), truncating wide values such as "27/08/2026 11:59:12".

use std::sync::Arc;

use sql_cli::data::data_view::DataView;
use sql_cli::data::datatable::{DataColumn, DataRow, DataTable, DataValue};
use sql_cli::ui::viewport_manager::ViewportManager;

const STARTED: &str = "27/08/2026 11:59:12";
const FINISHED: &str = "27/08/2026 12:04:31";

/// A wide export (like a TeamCity build API dump) where the interesting columns
/// sit past the width of the projection that selects them.
fn wide_source_table() -> DataTable {
    let headers = [
        "BuildTypeId",
        "Number",
        "State",
        "Status",
        "StatusText",
        "Branch",
        "Agent",
        "TriggeredBy",
        "Comment",
        "Project",
        "Job",
        "JobId",
        "Started",
        "Finished",
        "DurationSec",
    ];

    let mut table = DataTable::new("tc");
    for header in headers {
        table.add_column(DataColumn::new(header));
    }

    for i in 0..20 {
        table
            .add_row(DataRow::new(vec![
                DataValue::String(format!("bt_{i}")),
                DataValue::String(i.to_string()),
                DataValue::String("finished".into()),
                DataValue::String("SUCCESS".into()),
                DataValue::String("Success".into()),
                DataValue::String("main".into()),
                DataValue::String(format!("agent-{i}")),
                DataValue::String("scheduler".into()),
                DataValue::String(String::new()),
                DataValue::String("ServerOps".into()),
                DataValue::String("DeployUpdate".into()),
                DataValue::String(format!("job-{i}")),
                DataValue::String(STARTED.into()),
                DataValue::String(FINISHED.into()),
                DataValue::String("319".into()),
            ]))
            .unwrap();
    }

    table
}

/// SELECT Project, Job, JobId, Started, Finished, DurationSec FROM tc
fn projected_view() -> DataView {
    DataView::new(Arc::new(wide_source_table())).with_columns(vec![9, 10, 11, 12, 13, 14])
}

#[test]
fn visual_index_of_column_maps_source_indices_to_display_positions() {
    let view = projected_view();

    assert_eq!(view.visual_index_of_column(9), Some(0), "Project");
    assert_eq!(view.visual_index_of_column(12), Some(3), "Started");
    assert_eq!(view.visual_index_of_column(14), Some(5), "DurationSec");

    // Columns outside the projection have no visual position
    assert_eq!(view.visual_index_of_column(0), None, "BuildTypeId");
    assert_eq!(view.visual_index_of_column(99), None, "out of range");
}

#[test]
fn projected_datetime_columns_are_not_truncated() {
    let mut vm = ViewportManager::new(Arc::new(projected_view()));
    vm.update_terminal_size(200, 30);

    let (headers, _rows, widths) = vm.get_visual_display(200, &[]);

    for (name, value) in [("Started", STARTED), ("Finished", FINISHED)] {
        let pos = headers
            .iter()
            .position(|h| h == name)
            .unwrap_or_else(|| panic!("{name} column missing from {headers:?}"));

        assert!(
            widths[pos] >= value.len() as u16,
            "{name} width {} truncates {value:?} ({} chars); widths={widths:?}",
            widths[pos],
            value.len()
        );
    }
}

#[test]
fn projected_widths_match_the_equivalent_unprojected_view() {
    // The same six columns, but as the only columns in the source table, so that
    // visual and DataTable indices coincide. Widths must agree either way.
    let mut narrow = DataTable::new("tc_narrow");
    for header in ["Project", "Job", "JobId", "Started", "Finished", "DurationSec"] {
        narrow.add_column(DataColumn::new(header));
    }
    for i in 0..20 {
        narrow
            .add_row(DataRow::new(vec![
                DataValue::String("ServerOps".into()),
                DataValue::String("DeployUpdate".into()),
                DataValue::String(format!("job-{i}")),
                DataValue::String(STARTED.into()),
                DataValue::String(FINISHED.into()),
                DataValue::String("319".into()),
            ]))
            .unwrap();
    }

    let mut wide_vm = ViewportManager::new(Arc::new(projected_view()));
    wide_vm.update_terminal_size(200, 30);
    let (wide_headers, _, wide_widths) = wide_vm.get_visual_display(200, &[]);

    let mut narrow_vm = ViewportManager::new(Arc::new(DataView::new(Arc::new(narrow))));
    narrow_vm.update_terminal_size(200, 30);
    let (narrow_headers, _, narrow_widths) = narrow_vm.get_visual_display(200, &[]);

    assert_eq!(wide_headers, narrow_headers);
    assert_eq!(
        wide_widths, narrow_widths,
        "projection changed column widths: {wide_widths:?} vs {narrow_widths:?}"
    );
}

// Pure table rendering function that depends only on TableRenderContext
// This is completely decoupled from TUI internals

use crate::app_state_container::SelectionMode;
use crate::ui::table_render_context::TableRenderContext;
use ratatui::{
    layout::Constraint,
    prelude::*,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};

/// Render a table using only the provided context
/// This function has no dependencies on TUI internals
pub fn render_table(f: &mut Frame, area: Rect, ctx: &TableRenderContext) {
    // Handle empty results
    if ctx.row_count == 0 {
        let empty = Paragraph::new("No results found")
            .block(Block::default().borders(Borders::ALL).title("Results"))
            .style(Style::default().fg(Color::Yellow));
        f.render_widget(empty, area);
        return;
    }

    // Build header row
    let header = build_header_row(ctx);

    // Build data rows
    let rows = build_data_rows(ctx);

    // Calculate column widths for the table widget
    let widths = calculate_column_widths(ctx);

    // Create and render the table widget
    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Results ({} rows)", ctx.row_count)),
        )
        .column_spacing(1)
        .row_highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(table, area);
}

/// Build the header row with sort indicators and column selection
fn build_header_row(ctx: &TableRenderContext) -> Row<'static> {
    let mut header_cells: Vec<Cell> = Vec::new();

    // Add row number header if enabled
    if ctx.show_row_numbers {
        header_cells.push(
            Cell::from("#").style(
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
        );
    }

    // Add data headers with separator between pinned and scrollable columns
    let mut last_was_pinned = false;
    for (visual_pos, header) in ctx.column_headers.iter().enumerate() {
        let is_pinned = ctx.is_pinned_column(visual_pos);

        // Add separator if transitioning from pinned to non-pinned
        if last_was_pinned && !is_pinned && ctx.pinned_count > 0 {
            // Add a visual separator column
            header_cells.push(
                Cell::from("│").style(
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                ),
            );
        }

        // Get sort indicator
        let sort_indicator = ctx.get_sort_indicator(visual_pos);

        // Check if this is the current column
        let is_crosshair = ctx.is_selected_column(visual_pos);
        let column_indicator = if is_crosshair { " [*]" } else { "" };

        // Add pin indicator for pinned columns
        let pin_indicator = if is_pinned { "📌 " } else { "" };

        // Determine styling
        let mut style = if is_pinned {
            // Pinned columns get special styling
            Style::default()
                .bg(Color::Rgb(40, 40, 80)) // Darker blue background
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            // Regular columns
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        };

        if is_crosshair {
            // Current column gets yellow text
            style = style.fg(Color::Yellow).add_modifier(Modifier::UNDERLINED);
        }

        header_cells.push(
            Cell::from(format!(
                "{}{}{}{}",
                pin_indicator, header, sort_indicator, column_indicator
            ))
            .style(style),
        );

        last_was_pinned = is_pinned;
    }

    Row::new(header_cells)
}

/// Build the data rows with appropriate styling
fn build_data_rows(ctx: &TableRenderContext) -> Vec<Row<'static>> {
    ctx.data_rows
        .iter()
        .enumerate()
        .map(|(row_idx, row_data)| {
            let mut cells: Vec<Cell> = Vec::new();

            // Add row number if enabled
            if ctx.show_row_numbers {
                let row_num = ctx.row_viewport.start + row_idx + 1;
                cells.push(
                    Cell::from(row_num.to_string()).style(Style::default().fg(Color::DarkGray)),
                );
            }

            // Check if this is the current row
            let is_current_row = ctx.is_selected_row(row_idx);

            // Add data cells with separator between pinned and scrollable
            let mut last_was_pinned = false;
            for (col_idx, val) in row_data.iter().enumerate() {
                let is_pinned = ctx.is_pinned_column(col_idx);

                // Add separator if transitioning from pinned to non-pinned
                if last_was_pinned && !is_pinned && ctx.pinned_count > 0 {
                    cells.push(Cell::from("│").style(Style::default().fg(Color::DarkGray)));
                }

                let is_selected_column = ctx.is_selected_column(col_idx);
                let mut cell = Cell::from(val.clone());

                // Apply fuzzy filter highlighting
                if !is_current_row && ctx.cell_matches_filter(val) {
                    cell = cell.style(Style::default().fg(Color::Magenta));
                }

                // Apply background for pinned columns
                if is_pinned && !is_current_row {
                    cell = cell.style(Style::default().bg(Color::Rgb(20, 20, 40)));
                }

                // Apply selection styling based on mode
                cell = match ctx.selection_mode {
                    SelectionMode::Cell if is_current_row && is_selected_column => {
                        // Cell mode: Only highlight the specific cell
                        cell.style(
                            Style::default()
                                .bg(Color::Yellow)
                                .fg(Color::Black)
                                .add_modifier(Modifier::BOLD),
                        )
                    }
                    SelectionMode::Row if is_current_row => {
                        // Row mode: Highlight entire row with special crosshair cell
                        if is_selected_column {
                            cell.style(
                                Style::default()
                                    .bg(Color::Yellow)
                                    .fg(Color::Black)
                                    .add_modifier(Modifier::BOLD),
                            )
                        } else if is_pinned {
                            cell.style(Style::default().bg(Color::Rgb(60, 80, 120)))
                        } else {
                            cell.style(Style::default().bg(Color::Rgb(70, 70, 70)))
                        }
                    }
                    _ if is_selected_column => {
                        // Column highlight (not in current row)
                        if is_pinned {
                            cell.style(Style::default().bg(Color::Rgb(40, 60, 100)))
                        } else {
                            cell.style(Style::default().bg(Color::Rgb(50, 50, 50)))
                        }
                    }
                    _ if is_pinned => {
                        // Pinned column gets subtle blue tint
                        cell.style(Style::default().bg(Color::Rgb(20, 30, 50)))
                    }
                    _ => cell,
                };

                cells.push(cell);
                last_was_pinned = is_pinned;
            }

            // Apply row highlighting
            let row_style = if is_current_row {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            Row::new(cells).style(row_style)
        })
        .collect()
}

/// Calculate column widths for the table
fn calculate_column_widths(ctx: &TableRenderContext) -> Vec<Constraint> {
    let mut widths: Vec<Constraint> = Vec::new();

    // Add row number column width if enabled
    if ctx.show_row_numbers {
        widths.push(Constraint::Length(8)); // Fixed width for row numbers
    }

    // Add widths for visible data columns with separator
    let mut last_was_pinned = false;
    for (idx, &width) in ctx.column_widths.iter().enumerate() {
        let is_pinned = ctx.is_pinned_column(idx);

        // Add separator width if transitioning from pinned to non-pinned
        if last_was_pinned && !is_pinned && ctx.pinned_count > 0 {
            widths.push(Constraint::Length(1)); // Separator column
        }

        widths.push(Constraint::Length(width));
        last_was_pinned = is_pinned;
    }

    widths
}

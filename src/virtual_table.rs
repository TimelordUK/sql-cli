use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Row, StatefulWidget, Table, Widget},
};
use serde_json::Value;

/// A table widget that only renders visible rows for performance
pub struct VirtualTable<'a> {
    headers: Vec<&'a str>,
    data: &'a [Value],
    widths: Vec<Constraint>,
    block: Option<Block<'a>>,
    header_style: Style,
    row_style: Style,
    highlight_style: Style,
    highlight_symbol: &'a str,
}

impl<'a> VirtualTable<'a> {
    pub fn new(headers: Vec<&'a str>, data: &'a [Value], widths: Vec<Constraint>) -> Self {
        Self {
            headers,
            data,
            widths,
            block: None,
            header_style: Style::default().fg(Color::Yellow),
            row_style: Style::default(),
            highlight_style: Style::default().add_modifier(Modifier::REVERSED),
            highlight_symbol: ">> ",
        }
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    pub fn header_style(mut self, style: Style) -> Self {
        self.header_style = style;
        self
    }

    pub fn highlight_style(mut self, style: Style) -> Self {
        self.highlight_style = style;
        self
    }
}

#[derive(Default, Clone)]
pub struct VirtualTableState {
    /// Current offset (first visible row)
    pub offset: usize,
    /// Currently selected row (absolute index)
    pub selected: usize,
    /// Number of visible rows (calculated during render)
    pub visible_rows: usize,
}

impl VirtualTableState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn select(&mut self, index: usize) {
        self.selected = index;
    }

    pub fn scroll_up(&mut self, amount: usize) {
        self.selected = self.selected.saturating_sub(amount);

        // If we're moving within the visible window, no need to adjust offset
        if self.selected >= self.offset && self.selected < self.offset + self.visible_rows {
            return;
        }

        // If selected moved above the visible area, scroll up
        if self.selected < self.offset {
            self.offset = self.selected;
        }
    }

    pub fn scroll_down(&mut self, amount: usize, total_rows: usize) {
        self.selected = (self.selected + amount).min(total_rows.saturating_sub(1));

        // If we're moving within the visible window, no need to adjust offset
        if self.selected >= self.offset && self.selected < self.offset + self.visible_rows {
            return;
        }

        // If selected moved below the visible area, scroll down
        if self.selected >= self.offset + self.visible_rows {
            self.offset = self.selected.saturating_sub(self.visible_rows - 1);
        }
    }

    pub fn page_up(&mut self) {
        let page_size = self.visible_rows.saturating_sub(1);
        self.scroll_up(page_size);
    }

    pub fn page_down(&mut self, total_rows: usize) {
        let page_size = self.visible_rows.saturating_sub(1);
        self.scroll_down(page_size, total_rows);
    }

    pub fn goto_top(&mut self) {
        self.selected = 0;
        self.offset = 0;
    }

    pub fn goto_bottom(&mut self, total_rows: usize) {
        self.selected = total_rows.saturating_sub(1);
        // Position the viewport so the last page is visible
        // This ensures the cursor is at the bottom row of a full viewport
        if total_rows > self.visible_rows {
            self.offset = total_rows.saturating_sub(self.visible_rows);
        } else {
            self.offset = 0;
        }
    }
}

impl<'a> StatefulWidget for VirtualTable<'a> {
    type State = VirtualTableState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Calculate the inner area (accounting for borders)
        let inner_area = if let Some(ref block) = self.block {
            let inner = block.inner(area);
            block.render(area, buf);
            inner
        } else {
            area
        };

        // Calculate how many rows we can display (account for header)
        let available_height = inner_area.height.saturating_sub(2); // 1 for header, 1 for header margin
        state.visible_rows = available_height as usize;

        // Only create rows for visible data
        let end_row = (state.offset + state.visible_rows).min(self.data.len());
        let visible_slice = &self.data[state.offset..end_row];

        // Create header
        let header_cells: Vec<ratatui::widgets::Cell> = self
            .headers
            .iter()
            .map(|h| ratatui::widgets::Cell::from(*h).style(self.header_style))
            .collect();
        let header = Row::new(header_cells).height(1).bottom_margin(1);

        // Create only visible rows
        let rows: Vec<Row> = visible_slice
            .iter()
            .enumerate()
            .map(|(idx, record)| {
                let absolute_idx = state.offset + idx;
                let is_selected = absolute_idx == state.selected;

                let cells: Vec<ratatui::widgets::Cell> = self
                    .headers
                    .iter()
                    .map(|field| {
                        if let Some(obj) = record.as_object() {
                            match obj.get(*field) {
                                Some(Value::String(s)) => ratatui::widgets::Cell::from(s.as_str()),
                                Some(Value::Number(n)) => {
                                    ratatui::widgets::Cell::from(n.to_string())
                                }
                                Some(Value::Bool(b)) => ratatui::widgets::Cell::from(b.to_string()),
                                Some(Value::Null) => ratatui::widgets::Cell::from("NULL")
                                    .style(Style::default().fg(Color::Gray)),
                                Some(v) => ratatui::widgets::Cell::from(v.to_string()),
                                None => ratatui::widgets::Cell::from(""),
                            }
                        } else {
                            ratatui::widgets::Cell::from("")
                        }
                    })
                    .collect();

                let mut row = Row::new(cells).height(1);
                if is_selected {
                    row = row.style(self.highlight_style);
                }
                row
            })
            .collect();

        // Calculate selected row relative to visible area
        let relative_selected = if state.selected >= state.offset && state.selected < end_row {
            Some(state.selected - state.offset)
        } else {
            None
        };

        // Create a minimal table state for rendering
        let mut table_state = ratatui::widgets::TableState::default();
        table_state.select(relative_selected);

        // Render the table with only visible rows
        let table = Table::new(rows, self.widths.clone())
            .header(header)
            .highlight_symbol(self.highlight_symbol);

        <Table as StatefulWidget>::render(table, inner_area, buf, &mut table_state);
    }
}

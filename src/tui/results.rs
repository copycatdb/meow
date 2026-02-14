//! Results table pane with scrolling.

use crate::app::{App, FocusPane};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};

/// Draw the results pane.
pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let focused = app.focus == FocusPane::Results;
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let result = &app.result;

    // Title with row count and timing
    let title = if let Some(ref err) = result.error {
        format!(" Results — Error: {} ", err)
    } else if result.rows.is_empty() && result.columns.is_empty() {
        " Results ".to_string()
    } else {
        format!(
            " Results — {} rows  {}ms ",
            result.rows.len(),
            result.elapsed_ms
        )
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(border_style);

    if result.columns.is_empty() {
        // No results yet
        let msg = if let Some(ref err) = result.error {
            err.clone()
        } else if app.query_running {
            "Running query...".to_string()
        } else {
            "No results. Press Ctrl+Enter to run a query.".to_string()
        };
        let paragraph = Paragraph::new(msg)
            .block(block)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(paragraph, area);
        return;
    }

    // Build header
    let header_cells: Vec<Cell> = result
        .columns
        .iter()
        .map(|c| Cell::from(c.as_str()).style(Style::default().fg(Color::Cyan).bold()))
        .collect();
    let header = Row::new(header_cells).height(1);

    // Build rows with scroll offset
    let visible_rows: Vec<Row> = result
        .rows
        .iter()
        .skip(app.result_scroll)
        .map(|row_data| {
            let cells: Vec<Cell> = row_data
                .iter()
                .map(|val| Cell::from(val.as_str()))
                .collect();
            Row::new(cells)
        })
        .collect();

    // Column widths — auto-size based on content
    let widths: Vec<Constraint> = result
        .columns
        .iter()
        .enumerate()
        .map(|(i, col)| {
            let max_data = result
                .rows
                .iter()
                .map(|r| r.get(i).map(|s| s.len()).unwrap_or(0))
                .max()
                .unwrap_or(0);
            let w = col.len().max(max_data).min(40) as u16 + 2;
            Constraint::Length(w)
        })
        .collect();

    let table = Table::new(visible_rows, &widths)
        .header(header)
        .block(block)
        .row_highlight_style(Style::default().bg(Color::Rgb(49, 50, 68)));

    frame.render_widget(table, area);
}

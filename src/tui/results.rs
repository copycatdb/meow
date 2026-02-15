//! Results table pane with vertical and horizontal scrolling.

use crate::app::{App, FocusPane};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};

/// Draw the results pane.
pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    if app.expanded_mode && !app.result.columns.is_empty() && app.result.error.is_none() {
        draw_expanded(frame, app, area);
    } else {
        draw_table(frame, app, area);
    }
}

/// Draw results in expanded (vertical record) mode.
fn draw_expanded(frame: &mut Frame, app: &App, area: Rect) {
    let focused = app.focus == FocusPane::Results;
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let result = &app.result;
    let title = format!(
        " Results (expanded) — {} rows  {}ms ",
        result.rows.len(),
        result.elapsed_ms
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(border_style);

    // Build expanded text lines
    let max_col_width = result.columns.iter().map(|c| c.len()).max().unwrap_or(0);
    let mut lines: Vec<ratatui::text::Line> = Vec::new();
    for (i, row) in result.rows.iter().enumerate() {
        let sep = format!("-[ RECORD {} ]{}", i + 1, "-".repeat(20));
        lines.push(ratatui::text::Line::from(ratatui::text::Span::styled(
            sep,
            Style::default().fg(Color::Cyan),
        )));
        for (j, col) in result.columns.iter().enumerate() {
            let val = row.get(j).map(|s| s.as_str()).unwrap_or("");
            lines.push(ratatui::text::Line::from(format!(
                "{:>width$} | {}",
                col,
                val,
                width = max_col_width
            )));
        }
    }

    let text = ratatui::text::Text::from(lines);
    let paragraph = Paragraph::new(text)
        .block(block)
        .scroll((app.result_scroll as u16, 0));
    frame.render_widget(paragraph, area);
}

/// Draw the results as a normal table.
fn draw_table(frame: &mut Frame, app: &App, area: Rect) {
    let focused = app.focus == FocusPane::Results;
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let result = &app.result;

    // Title with row count, timing, and scroll hint
    let title = if let Some(ref err) = result.error {
        format!(" Results — Error: {} ", err)
    } else if result.rows.is_empty() && result.columns.is_empty() {
        " Results ".to_string()
    } else {
        let col_info = if result.columns.len() > 1 {
            format!(
                " (cols {}-{}/{})",
                app.result_col_scroll + 1,
                result
                    .columns
                    .len()
                    .min(app.result_col_scroll + visible_col_count(app, area)),
                result.columns.len()
            )
        } else {
            String::new()
        };
        format!(
            " Results — {} rows  {}ms{} ",
            result.rows.len(),
            result.elapsed_ms,
            col_info
        )
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(border_style);

    if result.columns.is_empty() {
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

    let col_offset = app.result_col_scroll;

    // Compute column widths for ALL columns (needed for slicing)
    let all_widths: Vec<u16> = result
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
            col.len().max(max_data).min(50) as u16 + 2
        })
        .collect();

    // Figure out how many columns fit in the available width (minus borders)
    let available_width = area.width.saturating_sub(2); // borders
    let mut total_w = 0u16;
    let mut visible_end = col_offset;
    for (i, &w) in all_widths.iter().enumerate().skip(col_offset) {
        let next = total_w + w;
        if next > available_width && visible_end > col_offset {
            break;
        }
        total_w = next;
        visible_end = i + 1;
    }

    // Slice columns
    let visible_cols = col_offset..visible_end;
    let widths: Vec<Constraint> = visible_cols
        .clone()
        .map(|i| Constraint::Length(all_widths[i]))
        .collect();

    // Build header (visible columns only)
    let header_cells: Vec<Cell> = visible_cols
        .clone()
        .map(|i| {
            Cell::from(result.columns[i].as_str()).style(Style::default().fg(Color::Cyan).bold())
        })
        .collect();
    let header = Row::new(header_cells).height(1);

    // Build rows with vertical scroll, horizontal slice
    let visible_rows: Vec<Row> = result
        .rows
        .iter()
        .skip(app.result_scroll)
        .map(|row_data| {
            let cells: Vec<Cell> = visible_cols
                .clone()
                .map(|i| Cell::from(row_data.get(i).map(|s| s.as_str()).unwrap_or("")))
                .collect();
            Row::new(cells)
        })
        .collect();

    let table = Table::new(visible_rows, &widths)
        .header(header)
        .block(block)
        .row_highlight_style(Style::default().bg(Color::Rgb(49, 50, 68)));

    frame.render_widget(table, area);
}

/// Estimate how many columns are visible from the current scroll offset.
fn visible_col_count(app: &App, area: Rect) -> usize {
    let available = area.width.saturating_sub(2) as usize;
    let mut total = 0;
    let mut count = 0;
    for i in app.result_col_scroll..app.result.columns.len() {
        let max_data = app
            .result
            .rows
            .iter()
            .map(|r| r.get(i).map(|s| s.len()).unwrap_or(0))
            .max()
            .unwrap_or(0);
        let w = app.result.columns[i].len().max(max_data).min(50) + 2;
        total += w;
        if total > available && count > 0 {
            break;
        }
        count += 1;
    }
    count.max(1)
}

//! Results table pane with vertical and horizontal scrolling.

use crate::app::{App, FocusPane};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};

/// Draw the results pane.
pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let columns = app.result.columns_for(app.current_result_set);
    if app.expanded_mode && !columns.is_empty() && app.result.error.is_none() {
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

    let rs_idx = app.current_result_set;
    let columns = app.result.columns_for(rs_idx);
    let rows = app.result.rows_for(rs_idx);
    let set_indicator = result_set_indicator(app);
    let title = format!(
        " Results (expanded){} — {} rows  {}ms ",
        set_indicator,
        rows.len(),
        app.result.elapsed_ms
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(border_style);

    // Build expanded text lines
    let max_col_width = columns.iter().map(|c| c.len()).max().unwrap_or(0);
    let mut lines: Vec<ratatui::text::Line> = Vec::new();
    for (i, row) in rows.iter().enumerate() {
        let sep = format!("-[ RECORD {} ]{}", i + 1, "-".repeat(20));
        lines.push(ratatui::text::Line::from(ratatui::text::Span::styled(
            sep,
            Style::default().fg(Color::Cyan),
        )));
        for (j, col) in columns.iter().enumerate() {
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

    let rs_idx = app.current_result_set;
    let columns = app.result.columns_for(rs_idx);
    let rows = app.result.rows_for(rs_idx);

    // Title with row count, timing, and scroll hint
    let title = if let Some(ref err) = app.result.error {
        format!(" Results — Error: {} ", err)
    } else if rows.is_empty() && columns.is_empty() {
        " Results ".to_string()
    } else {
        let set_indicator = result_set_indicator(app);
        let col_info = if columns.len() > 1 {
            format!(
                " (cols {}-{}/{})",
                app.result_col_scroll + 1,
                columns
                    .len()
                    .min(app.result_col_scroll + visible_col_count(app, area)),
                columns.len()
            )
        } else {
            String::new()
        };
        format!(
            " Results{} — {} rows  {}ms{} ",
            set_indicator,
            rows.len(),
            app.result.elapsed_ms,
            col_info
        )
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(border_style);

    if columns.is_empty() {
        let msg = if let Some(ref err) = app.result.error {
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
    let all_widths: Vec<u16> = columns
        .iter()
        .enumerate()
        .map(|(i, col)| {
            let max_data = rows
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
        .map(|i| Cell::from(columns[i].as_str()).style(Style::default().fg(Color::Cyan).bold()))
        .collect();
    let header = Row::new(header_cells).height(1);

    // Build rows with vertical scroll, horizontal slice
    let visible_rows: Vec<Row> = rows
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

/// Build a result set indicator string like " — Set 1/3" when there are multiple sets.
fn result_set_indicator(app: &App) -> String {
    if app.result.result_sets.len() > 1 {
        format!(
            " — Set {}/{}",
            app.current_result_set + 1,
            app.result.result_sets.len()
        )
    } else {
        String::new()
    }
}

/// Estimate how many columns are visible from the current scroll offset.
fn visible_col_count(app: &App, area: Rect) -> usize {
    let columns = app.result.columns_for(app.current_result_set);
    let rows = app.result.rows_for(app.current_result_set);
    let available = area.width.saturating_sub(2) as usize;
    let mut total = 0;
    let mut count = 0;
    for (i, col) in columns.iter().enumerate().skip(app.result_col_scroll) {
        let max_data = rows
            .iter()
            .map(|r| r.get(i).map(|s| s.len()).unwrap_or(0))
            .max()
            .unwrap_or(0);
        let w = col.len().max(max_data).min(50) + 2;
        total += w;
        if total > available && count > 0 {
            break;
        }
        count += 1;
    }
    count.max(1)
}

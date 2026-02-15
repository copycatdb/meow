//! Status bar showing connection info, timing, and row count.

use crate::app::App;
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

/// Draw the status bar.
pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let left = format!(" {} | {} ", app.connection_info, app.current_database);
    let right = if app.query_running {
        " ⏳ Running... ".to_string()
    } else if !app.result.columns_for(app.current_result_set).is_empty() {
        let set_info = if app.result.result_sets.len() > 1 {
            format!(
                "Set {}/{} | ",
                app.current_result_set + 1,
                app.result.result_sets.len()
            )
        } else {
            String::new()
        };
        format!(
            " {}{} rows | {}ms ",
            set_info,
            app.result.rows_for(app.current_result_set).len(),
            app.result.elapsed_ms
        )
    } else {
        String::new()
    };

    // Pad middle
    let total_width = area.width as usize;
    let padding = total_width.saturating_sub(left.len() + right.len());
    let status = format!("{}{}{}", left, " ".repeat(padding), right);

    let paragraph =
        Paragraph::new(status).style(Style::default().fg(Color::White).bg(Color::Rgb(49, 50, 68)));
    frame.render_widget(paragraph, area);
}

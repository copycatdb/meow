//! SQL query editor pane with syntax highlighting.

use crate::app::{App, FocusPane};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders};

/// SQL keywords for basic syntax highlighting.
const SQL_KEYWORDS: &[&str] = &[
    "SELECT", "FROM", "WHERE", "INSERT", "UPDATE", "DELETE", "CREATE", "DROP", "ALTER", "TABLE",
    "INTO", "VALUES", "SET", "JOIN", "LEFT", "RIGHT", "INNER", "OUTER", "ON", "AND", "OR", "NOT",
    "NULL", "IS", "IN", "LIKE", "BETWEEN", "ORDER", "BY", "GROUP", "HAVING", "LIMIT", "TOP",
    "DISTINCT", "AS", "UNION", "ALL", "EXISTS", "CASE", "WHEN", "THEN", "ELSE", "END", "BEGIN",
    "COMMIT", "ROLLBACK", "EXEC", "EXECUTE", "DECLARE", "USE", "GO", "WITH", "ASC", "DESC",
    "COUNT", "SUM", "AVG", "MIN", "MAX", "CAST", "CONVERT",
];

/// Draw the SQL editor pane.
pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let focused = app.focus == FocusPane::Editor;
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" SQL Editor ")
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);
    frame.render_widget(&app.editor, inner);
}

/// Check if a word is a SQL keyword (case-insensitive).
pub fn is_sql_keyword(word: &str) -> bool {
    SQL_KEYWORDS.iter().any(|kw| kw.eq_ignore_ascii_case(word))
}

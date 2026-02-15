//! Main UI layout and rendering.

use crate::app::App;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

use super::{autocomplete, editor, results, sidebar, statusbar};

/// Draw the entire TUI.
pub fn draw(frame: &mut Frame, app: &App) {
    let size = frame.area();

    // Main layout: title bar, content, status bar, keybindings
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // title bar
            Constraint::Min(5),    // content
            Constraint::Length(1), // status bar
            Constraint::Length(1), // key bindings
        ])
        .split(size);

    // Title bar
    let title = Paragraph::new(format!(
        " 🐱 meow — connected to {} ({})",
        app.connection_info, app.current_database
    ))
    .style(Style::default().fg(Color::White).bg(Color::Rgb(30, 30, 46)));
    frame.render_widget(title, chunks[0]);

    // Content area: sidebar | (editor / results)
    if app.sidebar_visible {
        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(22), // sidebar
                Constraint::Min(30),    // editor + results
            ])
            .split(chunks[1]);

        sidebar::draw(frame, app, content_chunks[0]);
        draw_editor_results(frame, app, content_chunks[1]);
    } else {
        draw_editor_results(frame, app, chunks[1]);
    }

    // Status bar
    statusbar::draw(frame, app, chunks[2]);

    // Key bindings bar
    let keys_text = if app.result.result_sets.len() > 1 {
        " Ctrl+Enter: Run │ Tab: Switch Pane │ [/]: Prev/Next Set │ Ctrl+D: Sidebar │ Ctrl+Q: Quit │ F1: Help"
    } else {
        " Ctrl+Enter: Run │ Tab: Switch Pane │ Ctrl+D: Sidebar │ Ctrl+Q: Quit │ F1: Help"
    };
    let keys = Paragraph::new(keys_text).style(
        Style::default()
            .fg(Color::DarkGray)
            .bg(Color::Rgb(30, 30, 46)),
    );
    frame.render_widget(keys, chunks[3]);

    // Help overlay
    if app.show_help {
        draw_help_overlay(frame, size);
    }

    // Autocomplete popup overlay
    if app.autocomplete.active && !app.autocomplete.suggestions.is_empty() {
        draw_autocomplete(frame, app, size);
    }
}

/// Draw the editor and results split vertically.
fn draw_editor_results(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(45), // editor
            Constraint::Percentage(55), // results
        ])
        .split(area);

    editor::draw(frame, app, chunks[0]);
    results::draw(frame, app, chunks[1]);
}

/// Draw the help overlay.
fn draw_help_overlay(frame: &mut Frame, area: Rect) {
    let help_area = centered_rect(60, 70, area);
    frame.render_widget(Clear, help_area);

    let help_text = vec![
        "🐱 meow — Key Bindings",
        "",
        "  Ctrl+Enter / F5    Execute query",
        "  Tab                Cycle focus (Editor → Results → Sidebar)",
        "  Ctrl+D             Toggle sidebar",
        "  Ctrl+L             Clear editor",
        "  Ctrl+Q             Quit",
        "  F1                 Toggle this help",
        "",
        "  Results pane:",
        "    ↑/↓              Scroll results",
        "    [ / ]            Previous / next result set",
        "",
        "  Sidebar:",
        "    ↑/↓              Navigate",
        "    Enter            Expand/collapse",
        "",
        "  Press F1 to close",
    ];

    let paragraph = Paragraph::new(help_text.join("\n"))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Help ")
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .style(Style::default().fg(Color::White).bg(Color::Rgb(30, 30, 46)))
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, help_area);
}

/// Create a centered rectangle.
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Draw the autocomplete popup near the cursor.
fn draw_autocomplete(frame: &mut Frame, app: &App, area: Rect) {
    let max_items = 8usize;
    let suggestions = &app.autocomplete.suggestions;
    let count = suggestions.len().min(max_items);
    if count == 0 {
        return;
    }

    // Figure out cursor position in the terminal.
    // The editor is inside content area. We approximate:
    // row 0 = title bar, then content starts at row 1.
    // If sidebar visible, editor starts at x=22+1 (border), else x=1.
    // Editor area starts at row 1 (title) + 1 (border).
    let cursor = app.editor.cursor();
    let editor_x_offset: u16 = if app.sidebar_visible { 23 } else { 1 };
    // Line numbers take ~4 chars, plus 1 border
    let line_num_width: u16 = 5;
    let cursor_x = editor_x_offset + line_num_width + cursor.1 as u16;
    // Title bar (1) + editor border (1) + cursor row - scroll offset
    let cursor_y = 2 + cursor.0 as u16;

    // Position popup below cursor
    let popup_y = (cursor_y + 1).min(area.height.saturating_sub(count as u16 + 2));
    let popup_x = cursor_x.min(area.width.saturating_sub(22));

    let width = 20u16;
    let height = count as u16 + 2; // +2 for borders

    let popup_area = Rect::new(
        popup_x.min(area.width.saturating_sub(width)),
        popup_y.min(area.height.saturating_sub(height)),
        width.min(area.width),
        height.min(area.height),
    );

    frame.render_widget(Clear, popup_area);

    let items: Vec<Line> = suggestions
        .iter()
        .take(max_items)
        .enumerate()
        .map(|(i, kw)| {
            if i == app.autocomplete.selected {
                Line::from(*kw).style(Style::default().fg(Color::Black).bg(Color::Cyan))
            } else {
                Line::from(*kw).style(Style::default().fg(Color::White))
            }
        })
        .collect();

    let popup = Paragraph::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .style(Style::default().bg(Color::Rgb(40, 40, 60))),
    );

    frame.render_widget(popup, popup_area);
}

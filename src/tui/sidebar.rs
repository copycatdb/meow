//! Object browser sidebar pane.

use crate::app::{self, App, FocusPane};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

/// Draw the sidebar object browser.
pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let focused = app.focus == FocusPane::Sidebar;
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Objects ")
        .border_style(border_style);

    let flat = app::flatten_tree(&app.objects);
    if flat.is_empty() {
        let msg = Paragraph::new("  Loading...")
            .block(block)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(msg, area);
        return;
    }

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines: Vec<Line> = flat
        .iter()
        .enumerate()
        .map(|(i, (depth, name, expanded, has_children))| {
            let indent = "  ".repeat(*depth as usize);
            let icon = if *has_children {
                if *expanded { "▾ " } else { "▸ " }
            } else {
                "  "
            };
            let style = if i == app.sidebar_scroll && focused {
                Style::default().fg(Color::Cyan).bg(Color::Rgb(49, 50, 68))
            } else {
                match depth {
                    0 => Style::default().fg(Color::Yellow),
                    1 => Style::default().fg(Color::Green),
                    _ => Style::default().fg(Color::White),
                }
            };
            Line::from(Span::styled(format!("{}{}{}", indent, icon, name), style))
        })
        .collect();

    let paragraph = Paragraph::new(lines).scroll((0, 0));
    frame.render_widget(paragraph, inner);
}

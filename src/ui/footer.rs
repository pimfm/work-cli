use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::{App, ViewMode};

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let mut spans = Vec::new();

    match &app.view_mode {
        ViewMode::Items => {
            spans.push(hint("↑↓", "navigate"));
            spans.push(hint("→", "agents"));
            spans.push(hint("d", "dispatch"));
            spans.push(hint("m", "auto mode"));
            spans.push(hint("r", "refresh"));
            spans.push(hint("q", "quit"));
        }
        ViewMode::Agents => {
            spans.push(hint("↑↓", "navigate"));
            spans.push(hint("→", "detail"));
            spans.push(hint("←", "items"));
            spans.push(hint("q", "quit"));
        }
        ViewMode::AgentDetail(_) => {
            spans.push(hint("↑↓", "scroll"));
            spans.push(hint("←", "agents"));
            spans.push(hint("q", "quit"));
        }
    }

    // Auto mode indicator
    if app.auto_mode {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            " AUTO ",
            Style::default()
                .fg(ratatui::style::Color::Black)
                .bg(ratatui::style::Color::Green),
        ));
    }

    // Flash message
    if let Some((msg, _)) = &app.flash_message {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            msg,
            Style::default().fg(ratatui::style::Color::Yellow),
        ));
    }

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line);
    f.render_widget(paragraph, area);
}

fn hint(key: &str, desc: &str) -> Span<'static> {
    Span::styled(
        format!(" {key}:{desc} "),
        Style::default().fg(ratatui::style::Color::DarkGray),
    )
}

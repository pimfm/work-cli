use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use crate::ui::theme::agent_color;
use crate::model::agent::AgentName;

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    if !app.input_active {
        return;
    }

    let input = &app.input_buffer;
    let cursor = app.input_cursor;

    // Build styled input with cursor
    let mut spans = Vec::new();

    // Highlight @agent prefix if present
    if let Some(agent_name) = detect_agent_prefix(input) {
        let prefix = format!("@{} ", agent_name.as_str());
        spans.push(Span::styled(
            prefix.clone(),
            Style::default().fg(agent_color(agent_name)),
        ));
        let rest: String = input.chars().skip(prefix.len()).collect();
        spans.push(Span::raw(rest));
    } else {
        spans.push(Span::raw(input.clone()));
    }

    let title = if detect_agent_prefix(input).is_some() {
        " Message Agent "
    } else if input.is_empty() {
        " Command — @agent msg | new task title "
    } else {
        " New Task "
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ratatui::style::Color::Yellow))
        .title(title);

    let paragraph = Paragraph::new(Line::from(spans)).block(block);
    f.render_widget(paragraph, area);

    // Position cursor
    let x = area.x + 1 + cursor as u16;
    let y = area.y + 1;
    f.set_cursor_position((x.min(area.x + area.width - 2), y));
}

fn detect_agent_prefix(input: &str) -> Option<AgentName> {
    if !input.starts_with('@') {
        return None;
    }
    let after_at = &input[1..];
    for name in AgentName::ALL {
        let prefix = name.as_str();
        if after_at.starts_with(prefix)
            && after_at
                .chars()
                .nth(prefix.len())
                .map_or(true, |c| c == ' ')
        {
            return Some(name);
        }
    }
    None
}

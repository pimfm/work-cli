use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::App;
use crate::model::agent::AgentName;
use crate::ui::theme::event_color;

pub fn render(f: &mut Frame, area: Rect, app: &App, agent_name: AgentName) {
    let events = app.agent_events(agent_name);

    let visible_height = area.height.saturating_sub(2) as usize;
    let max_scroll = events.len().saturating_sub(visible_height);
    let scroll = app.agent_log_scroll.min(max_scroll);

    let lines: Vec<Line> = events
        .iter()
        .skip(scroll)
        .take(visible_height)
        .map(|event| {
            // Parse timestamp for display
            let time = event
                .timestamp
                .get(11..19)
                .unwrap_or(&event.timestamp);

            let date = event
                .timestamp
                .get(..10)
                .unwrap_or("");

            let mut spans = vec![
                Span::styled(
                    format!("{date} {time} "),
                    Style::default().fg(ratatui::style::Color::DarkGray),
                ),
                Span::styled(
                    format!("{:<12}", event.event),
                    Style::default().fg(event_color(&event.event)),
                ),
            ];

            if let Some(msg) = &event.message {
                spans.push(Span::raw(format!(" {msg}")));
            } else if let Some(title) = &event.work_item_title {
                spans.push(Span::styled(
                    format!(" {title}"),
                    Style::default().fg(ratatui::style::Color::White),
                ));
            }

            Line::from(spans)
        })
        .collect();

    let title = format!(
        " {} {} Activity ",
        agent_name.emoji(),
        agent_name.display_name()
    );

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(ratatui::style::Color::Cyan))
                .title(title),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

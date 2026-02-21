use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::App;
use crate::model::agent::AgentName;
use crate::model::chat::ChatSender;
use crate::ui::theme::agent_color;

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let visible_height = area.height.saturating_sub(2) as usize;

    // Build lines from chat messages
    let mut all_lines: Vec<Line> = Vec::new();

    for msg in &app.chat_messages {
        let mut header_spans = vec![
            Span::styled(
                format!("{} ", msg.timestamp),
                Style::default().fg(ratatui::style::Color::DarkGray),
            ),
        ];

        match &msg.sender {
            ChatSender::User => {
                header_spans.push(Span::styled(
                    "you",
                    Style::default()
                        .fg(ratatui::style::Color::White)
                        .add_modifier(Modifier::BOLD),
                ));

                // Check if message targets an agent
                if let Some(name) = extract_agent_target(&msg.text) {
                    header_spans.push(Span::styled(
                        format!(" → {}", name.display_name()),
                        Style::default().fg(agent_color(name)),
                    ));
                }
            }
            ChatSender::Agent(name) => {
                header_spans.push(Span::styled(
                    format!("{} {}", name.emoji(), name.display_name()),
                    Style::default()
                        .fg(agent_color(*name))
                        .add_modifier(Modifier::BOLD),
                ));
            }
            ChatSender::System => {
                header_spans.push(Span::styled(
                    "system",
                    Style::default()
                        .fg(ratatui::style::Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ));
            }
        }

        all_lines.push(Line::from(header_spans));

        // Message body - wrap into lines
        let body = match &msg.sender {
            ChatSender::User => strip_agent_prefix(&msg.text),
            _ => msg.text.clone(),
        };

        for text_line in body.lines() {
            let color = match &msg.sender {
                ChatSender::User => ratatui::style::Color::White,
                ChatSender::Agent(_) => ratatui::style::Color::Rgb(0xCC, 0xCC, 0xCC),
                ChatSender::System => ratatui::style::Color::Yellow,
            };
            all_lines.push(Line::from(Span::styled(
                format!("  {text_line}"),
                Style::default().fg(color),
            )));
        }

        // Blank line between messages
        all_lines.push(Line::raw(""));
    }

    // Show loading indicator if waiting for agent response
    if app.waiting_for_response {
        all_lines.push(Line::from(Span::styled(
            "  thinking...",
            Style::default()
                .fg(ratatui::style::Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        )));
    }

    // Auto-scroll to bottom
    let total = all_lines.len();
    let skip = total.saturating_sub(visible_height);
    let visible_lines: Vec<Line> = all_lines.into_iter().skip(skip).take(visible_height).collect();

    let msg_count = app.chat_messages.len();
    let title = if msg_count > 0 {
        format!(" Chat ({msg_count}) ")
    } else {
        " Chat — press : to start ".to_string()
    };

    let paragraph = Paragraph::new(visible_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(ratatui::style::Color::Magenta))
                .title(title),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

fn extract_agent_target(text: &str) -> Option<AgentName> {
    if !text.starts_with('@') {
        return None;
    }
    let after_at = &text[1..];
    for name in AgentName::ALL {
        if after_at.starts_with(name.as_str()) {
            return Some(name);
        }
    }
    None
}

fn strip_agent_prefix(text: &str) -> String {
    if !text.starts_with('@') {
        return text.to_string();
    }
    let after_at = &text[1..];
    for name in AgentName::ALL {
        let prefix = name.as_str();
        if after_at.starts_with(prefix) {
            let rest = &after_at[prefix.len()..];
            return rest.trim_start().to_string();
        }
    }
    text.to_string()
}

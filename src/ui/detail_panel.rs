use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::App;
use crate::ui::theme::priority_color;

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    if app.items.is_empty() {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(ratatui::style::Color::Cyan))
            .title(" Details ");
        f.render_widget(block, area);
        return;
    }

    let item = &app.items[app.selected_item];
    let mut lines: Vec<Line> = Vec::new();

    if let Some(status) = &item.status {
        lines.push(Line::from(vec![
            Span::styled("Status: ", Style::default().fg(ratatui::style::Color::Gray)),
            Span::raw(status),
        ]));
    }

    if let Some(priority) = &item.priority {
        lines.push(Line::from(vec![
            Span::styled(
                "Priority: ",
                Style::default().fg(ratatui::style::Color::Gray),
            ),
            Span::styled(priority, Style::default().fg(priority_color(priority))),
        ]));
    }

    if !item.labels.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("Labels: ", Style::default().fg(ratatui::style::Color::Gray)),
            Span::raw(item.labels.join(", ")),
        ]));
    }

    if let Some(team) = &item.team {
        lines.push(Line::from(vec![
            Span::styled("Team: ", Style::default().fg(ratatui::style::Color::Gray)),
            Span::raw(team),
        ]));
    }

    if let Some(url) = &item.url {
        lines.push(Line::from(vec![
            Span::styled("URL: ", Style::default().fg(ratatui::style::Color::Gray)),
            Span::styled(url, Style::default().fg(ratatui::style::Color::Blue)),
        ]));
    }

    if let Some(desc) = &item.description {
        lines.push(Line::raw(""));
        let truncated: String = desc.chars().take(300).collect();
        lines.push(Line::raw(truncated));
    }

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(ratatui::style::Color::Cyan))
                .title(" Details "),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

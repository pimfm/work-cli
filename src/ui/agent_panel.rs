use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use crate::app::{App, ViewMode};
use crate::model::agent::AgentStatus;
use crate::ui::theme::{agent_color, status_color};

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let agents = app.store.get_all();
    let in_agent_view = matches!(app.view_mode, ViewMode::Agents);

    let items: Vec<ListItem> = agents
        .iter()
        .enumerate()
        .map(|(i, agent)| {
            let selected = in_agent_view && i == app.selected_agent;

            let emoji = Span::styled(
                format!("{} ", agent.name.emoji()),
                Style::default().fg(agent_color(agent.name)),
            );

            let name_style = if selected {
                Style::default()
                    .fg(ratatui::style::Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(agent_color(agent.name))
            };
            let name = Span::styled(
                format!("{} ", agent.name.display_name()),
                name_style,
            );

            let status = Span::styled(
                format!("{}", agent.status),
                Style::default().fg(status_color(agent.status)),
            );

            let mut spans = vec![emoji, name, status];

            // Elapsed time for working agents
            if agent.status == AgentStatus::Working || agent.status == AgentStatus::Provisioning {
                if let Some(started_at) = &agent.started_at {
                    if let Ok(start) = chrono::DateTime::parse_from_rfc3339(started_at) {
                        let elapsed = chrono::Utc::now().signed_duration_since(start);
                        let mins = elapsed.num_minutes();
                        let secs = elapsed.num_seconds() % 60;
                        spans.push(Span::styled(
                            format!(" {mins:02}:{secs:02}"),
                            Style::default().fg(ratatui::style::Color::Gray),
                        ));
                    }
                }
            }

            // Work item title
            if let Some(title) = &agent.work_item_title {
                let max_len = area.width.saturating_sub(30) as usize;
                let truncated: String = title.chars().take(max_len).collect();
                spans.push(Span::styled(
                    format!(" {truncated}"),
                    Style::default().fg(ratatui::style::Color::White),
                ));
            }

            // Error message
            if let Some(error) = &agent.error {
                spans.push(Span::styled(
                    format!(" {error}"),
                    Style::default().fg(ratatui::style::Color::Red),
                ));
            }

            // Idle tagline
            if agent.status == AgentStatus::Idle {
                let p = crate::model::personality::personality(agent.name);
                spans.push(Span::styled(
                    format!(" — {}", p.tagline),
                    Style::default().fg(ratatui::style::Color::DarkGray),
                ));
            }

            ListItem::new(Line::from(spans))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(ratatui::style::Color::Cyan))
            .title(" Agents "),
    );

    f.render_widget(list, area);
}

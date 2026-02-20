use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use crate::app::App;
use crate::ui::theme::{agent_color, source_color};

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let items: Vec<ListItem> = app
        .items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let selected = i == app.selected_item;

            // Agent emoji if assigned
            let agent_indicator = app
                .assigned_agent(&item.id)
                .map(|name| {
                    Span::styled(
                        format!("{} ", name.emoji()),
                        Style::default().fg(agent_color(name)),
                    )
                })
                .unwrap_or_else(|| Span::raw("  "));

            let id_span = Span::styled(
                format!("{} ", item.id),
                Style::default().fg(source_color(&item.source)),
            );

            // Truncate title to fit
            let max_title = area.width.saturating_sub(20) as usize;
            let title: String = item.title.chars().take(max_title).collect();
            let title_style = if selected {
                Style::default()
                    .fg(ratatui::style::Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let title_span = Span::styled(title, title_style);

            let source_span = Span::styled(
                format!(" [{}]", item.source),
                Style::default().fg(source_color(&item.source)),
            );

            let line = Line::from(vec![agent_indicator, id_span, title_span, source_span]);
            ListItem::new(line)
        })
        .collect();

    let title = if app.loading {
        " Work Items (loading...) "
    } else {
        " Work Items "
    };

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(ratatui::style::Color::Cyan))
            .title(title),
    );

    f.render_widget(list, area);
}

pub mod agent_detail;
pub mod agent_panel;
pub mod detail_panel;
pub mod footer;
pub mod item_list;
pub mod theme;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

use crate::app::{App, ViewMode};

pub fn render(f: &mut Frame, app: &App) {
    let size = f.area();

    // Split: main content + footer
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(6), Constraint::Length(1)])
        .split(size);

    let main_area = vertical[0];
    let footer_area = vertical[1];

    match &app.view_mode {
        ViewMode::Items => {
            // Items (50%) + Detail (25%) + Agents (25%)
            let horizontal = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(50),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                ])
                .split(main_area);

            item_list::render(f, horizontal[0], app);
            detail_panel::render(f, horizontal[1], app);
            agent_panel::render(f, horizontal[2], app);
        }
        ViewMode::Agents => {
            // Items (40%) + Agents (60%)
            let horizontal = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
                .split(main_area);

            item_list::render(f, horizontal[0], app);
            agent_panel::render(f, horizontal[1], app);
        }
        ViewMode::AgentDetail(name) => {
            // Agent detail takes full width
            agent_detail::render(f, main_area, app, *name);
        }
    }

    footer::render(f, footer_area, app);
}

pub mod agent_detail;
pub mod agent_panel;
pub mod board_picker;
pub mod chat_panel;
pub mod command_bar;
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

    // Determine bottom bar height: command bar (3) when input active, else footer (1)
    let bottom_height = if app.input_active { 3 } else { 1 };

    // Determine if chat panel should be visible
    let show_chat = !app.chat_messages.is_empty() || app.input_active;

    // Split: main content + chat (optional) + bottom bar
    let vertical = if show_chat {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(6),        // main content
                Constraint::Length(12),     // chat panel
                Constraint::Length(bottom_height), // footer or command bar
            ])
            .split(size)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(6),                // main content
                Constraint::Length(0),              // no chat
                Constraint::Length(bottom_height),  // footer or command bar
            ])
            .split(size)
    };

    let main_area = vertical[0];
    let chat_area = vertical[1];
    let bottom_area = vertical[2];

    match &app.view_mode {
        ViewMode::BoardSelection => {
            board_picker::render(f, main_area, app);
        }
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

    // Chat panel
    if show_chat {
        chat_panel::render(f, chat_area, app);
    }

    // Bottom bar: command bar when input active, footer otherwise
    if app.input_active {
        command_bar::render(f, bottom_area, app);
    } else {
        footer::render(f, bottom_area, app);
    }
}

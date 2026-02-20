use ratatui::style::Color;

use crate::model::agent::{AgentName, AgentStatus};

pub fn source_color(source: &str) -> Color {
    match source {
        "Linear" => Color::Rgb(0x5E, 0x6A, 0xD2),
        "Trello" => Color::Rgb(0x00, 0x79, 0xBF),
        "Jira" => Color::Rgb(0x00, 0x52, 0xCC),
        "GitHub" => Color::White,
        _ => Color::Gray,
    }
}

pub fn priority_color(priority: &str) -> Color {
    match priority {
        "Urgent" => Color::Red,
        "High" => Color::Yellow,
        "Medium" => Color::Blue,
        "Low" => Color::Gray,
        _ => Color::Gray,
    }
}

pub fn agent_color(name: AgentName) -> Color {
    match name {
        AgentName::Ember => Color::Rgb(0xFF, 0x70, 0x43),
        AgentName::Flow => Color::Rgb(0x4F, 0xC3, 0xF7),
        AgentName::Tempest => Color::Rgb(0xCE, 0x93, 0xD8),
        AgentName::Terra => Color::Rgb(0x81, 0xC7, 0x84),
    }
}

pub fn status_color(status: AgentStatus) -> Color {
    match status {
        AgentStatus::Idle => Color::Gray,
        AgentStatus::Provisioning => Color::Yellow,
        AgentStatus::Working => Color::Cyan,
        AgentStatus::Done => Color::Green,
        AgentStatus::Error => Color::Red,
    }
}

pub fn event_color(event: &str) -> Color {
    match event {
        "dispatched" => Color::Blue,
        "provisioning" => Color::Yellow,
        "working" => Color::Cyan,
        "done" => Color::Green,
        "error" => Color::Red,
        "retry" => Color::Yellow,
        "max-retries" => Color::Red,
        "released" => Color::Gray,
        _ => Color::White,
    }
}

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::PathBuf;

use crate::config::data_dir;
use crate::model::agent::AgentName;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentEvent {
    pub timestamp: String,
    pub agent: AgentName,
    pub event: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_item_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_item_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

fn log_path() -> PathBuf {
    data_dir().join("agent-activity.jsonl")
}

pub fn append_event(event: &AgentEvent) -> Result<()> {
    let path = log_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    let line = serde_json::to_string(event)?;
    writeln!(file, "{line}")?;
    Ok(())
}

pub fn read_events(agent: Option<AgentName>, limit: Option<usize>) -> Vec<AgentEvent> {
    let path = log_path();
    if !path.exists() {
        return Vec::new();
    }
    let contents = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut events: Vec<AgentEvent> = contents
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| serde_json::from_str(line).ok())
        .filter(|e: &AgentEvent| agent.map_or(true, |a| e.agent == a))
        .collect();

    if let Some(limit) = limit {
        let len = events.len();
        if len > limit {
            events = events.split_off(len - limit);
        }
    }

    events
}

pub fn new_event(
    agent: AgentName,
    event_type: &str,
    work_item_id: Option<&str>,
    work_item_title: Option<&str>,
    message: Option<&str>,
) -> AgentEvent {
    AgentEvent {
        timestamp: chrono::Utc::now().to_rfc3339(),
        agent,
        event: event_type.to_string(),
        work_item_id: work_item_id.map(String::from),
        work_item_title: work_item_title.map(String::from),
        message: message.map(String::from),
    }
}

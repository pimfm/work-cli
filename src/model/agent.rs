use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentName {
    Ember,
    Flow,
    Tempest,
    Terra,
}

impl AgentName {
    pub const ALL: [AgentName; 4] = [
        AgentName::Ember,
        AgentName::Flow,
        AgentName::Tempest,
        AgentName::Terra,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            AgentName::Ember => "ember",
            AgentName::Flow => "flow",
            AgentName::Tempest => "tempest",
            AgentName::Terra => "terra",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            AgentName::Ember => "Ember",
            AgentName::Flow => "Flow",
            AgentName::Tempest => "Tempest",
            AgentName::Terra => "Terra",
        }
    }

    pub fn emoji(&self) -> &'static str {
        match self {
            AgentName::Ember => "\u{1F468}\u{200D}\u{1F692}",
            AgentName::Flow => "\u{1F3C4}\u{200D}\u{2640}\u{FE0F}",
            AgentName::Tempest => "\u{1F9DD}\u{200D}\u{2640}\u{FE0F}",
            AgentName::Terra => "\u{1F469}\u{200D}\u{1F33E}",
        }
    }
}

impl fmt::Display for AgentName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentStatus {
    Idle,
    Provisioning,
    Working,
    Done,
    Error,
}

impl fmt::Display for AgentStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AgentStatus::Idle => f.write_str("idle"),
            AgentStatus::Provisioning => f.write_str("provisioning"),
            AgentStatus::Working => f.write_str("working"),
            AgentStatus::Done => f.write_str("done"),
            AgentStatus::Error => f.write_str("error"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub name: AgentName,
    pub status: AgentStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_item_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_item_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worktree_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(default)]
    pub retry_count: u32,
}

impl Agent {
    pub fn new(name: AgentName) -> Self {
        Self {
            name,
            status: AgentStatus::Idle,
            work_item_id: None,
            work_item_title: None,
            branch: None,
            worktree_path: None,
            pid: None,
            started_at: None,
            error: None,
            retry_count: 0,
        }
    }
}

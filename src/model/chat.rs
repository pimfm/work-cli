use crate::model::agent::AgentName;

#[derive(Debug, Clone)]
pub enum ChatSender {
    User,
    Agent(AgentName),
    System,
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub sender: ChatSender,
    pub text: String,
    pub timestamp: String,
}

impl ChatMessage {
    pub fn user(text: impl Into<String>) -> Self {
        Self {
            sender: ChatSender::User,
            text: text.into(),
            timestamp: chrono::Utc::now().format("%H:%M:%S").to_string(),
        }
    }

    pub fn agent(name: AgentName, text: impl Into<String>) -> Self {
        Self {
            sender: ChatSender::Agent(name),
            text: text.into(),
            timestamp: chrono::Utc::now().format("%H:%M:%S").to_string(),
        }
    }

    pub fn system(text: impl Into<String>) -> Self {
        Self {
            sender: ChatSender::System,
            text: text.into(),
            timestamp: chrono::Utc::now().format("%H:%M:%S").to_string(),
        }
    }
}

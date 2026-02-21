pub mod github;
pub mod jira;
pub mod linear;
pub mod trello;

use anyhow::Result;
use async_trait::async_trait;

use crate::config::AppConfig;
use crate::model::work_item::WorkItem;

pub struct BoardInfo {
    pub id: String,
    pub name: String,
    pub source: String,
}

#[async_trait]
pub trait Provider: Send + Sync {
    fn name(&self) -> &str;
    async fn fetch_items(&self) -> Result<Vec<WorkItem>>;
    async fn list_boards(&self) -> Result<Vec<BoardInfo>>;
    fn set_board_filter(&mut self, _board_id: String) {}
    async fn move_to_done(&self, _source_id: &str) -> Result<()> {
        Ok(())
    }
    async fn move_to_in_progress(&self, _source_id: &str) -> Result<()> {
        Ok(())
    }
    /// Create a new work item in the provider. Returns None if provider doesn't support creation.
    async fn create_item(&self, _title: &str, _description: Option<&str>) -> Result<Option<WorkItem>> {
        Ok(None)
    }
}

#[cfg(test)]
pub mod tests;

pub fn create_providers(config: &AppConfig) -> Vec<Box<dyn Provider>> {
    let mut providers: Vec<Box<dyn Provider>> = Vec::new();

    if let Some(cfg) = &config.linear {
        providers.push(Box::new(linear::LinearProvider::new(cfg.api_key.clone())));
    }
    if let Some(cfg) = &config.trello {
        providers.push(Box::new(trello::TrelloProvider::new(
            cfg.api_key.clone(),
            cfg.token.clone(),
        )));
    }
    if let Some(cfg) = &config.jira {
        providers.push(Box::new(jira::JiraProvider::new(
            cfg.domain.clone(),
            cfg.email.clone(),
            cfg.api_token.clone(),
        )));
    }
    if let Some(cfg) = &config.github {
        providers.push(Box::new(github::GitHubProvider::new(cfg.owner.clone())));
    }

    providers
}

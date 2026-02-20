pub mod github;
pub mod jira;
pub mod linear;
pub mod trello;

use anyhow::Result;
use async_trait::async_trait;

use crate::config::AppConfig;
use crate::model::work_item::WorkItem;

#[async_trait]
pub trait Provider: Send + Sync {
    fn name(&self) -> &str;
    async fn fetch_items(&self) -> Result<Vec<WorkItem>>;
}

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

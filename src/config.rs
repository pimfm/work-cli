use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Default)]
pub struct AppConfig {
    pub linear: Option<LinearConfig>,
    pub trello: Option<TrelloConfig>,
    pub jira: Option<JiraConfig>,
    pub github: Option<GitHubConfig>,
    pub agents: Option<AgentsConfig>,
}

#[derive(Debug, Deserialize)]
pub struct LinearConfig {
    pub api_key: String,
}

#[derive(Debug, Deserialize)]
pub struct TrelloConfig {
    pub api_key: String,
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct JiraConfig {
    pub domain: String,
    pub email: String,
    pub api_token: String,
}

#[derive(Debug, Deserialize)]
pub struct GitHubConfig {
    pub owner: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct AgentsConfig {
    pub repo_root: Option<String>,
}

fn config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".localpipeline")
        .join("config.toml")
}

pub fn data_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".localpipeline")
}

pub fn load_config() -> Result<AppConfig> {
    let path = config_path();
    if !path.exists() {
        return Ok(AppConfig::default());
    }
    let contents = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read config from {}", path.display()))?;
    let config: AppConfig =
        toml::from_str(&contents).with_context(|| "Failed to parse config.toml")?;
    Ok(config)
}

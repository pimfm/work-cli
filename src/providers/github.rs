use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;

use super::Provider;
use crate::model::work_item::WorkItem;

pub struct GitHubProvider {
    owner: String,
}

impl GitHubProvider {
    pub fn new(owner: String) -> Self {
        Self { owner }
    }
}

#[derive(Deserialize)]
struct GhIssue {
    number: u64,
    title: String,
    body: Option<String>,
    state: Option<String>,
    url: Option<String>,
    #[serde(default)]
    labels: Vec<GhLabel>,
    repository: Option<GhRepo>,
}

#[derive(Deserialize)]
struct GhLabel {
    name: String,
}

#[derive(Deserialize)]
struct GhRepo {
    #[serde(rename = "nameWithOwner")]
    name_with_owner: String,
}

#[async_trait]
impl Provider for GitHubProvider {
    fn name(&self) -> &str {
        "GitHub"
    }

    async fn fetch_items(&self) -> Result<Vec<WorkItem>> {
        let output = tokio::process::Command::new("gh")
            .args([
                "search",
                "issues",
                "--assignee",
                &self.owner,
                "--state",
                "open",
                "--json",
                "number,title,body,state,url,labels,repository",
                "--limit",
                "50",
            ])
            .output()
            .await
            .context("Failed to run gh CLI")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("gh search issues failed: {stderr}");
        }

        let issues: Vec<GhIssue> =
            serde_json::from_slice(&output.stdout).context("Failed to parse gh output")?;

        let items = issues
            .into_iter()
            .map(|issue| {
                let description = issue
                    .body
                    .filter(|b| !b.trim().is_empty())
                    .map(|b| b.chars().take(500).collect::<String>());
                let labels = issue.labels.into_iter().map(|l| l.name).collect();
                let team = issue.repository.map(|r| r.name_with_owner);

                WorkItem {
                    id: format!("#{}", issue.number),
                    title: issue.title,
                    description,
                    status: issue.state,
                    priority: None,
                    labels,
                    source: "GitHub".into(),
                    team,
                    url: issue.url,
                }
            })
            .collect();

        Ok(items)
    }
}

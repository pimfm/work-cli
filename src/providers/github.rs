use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;

use super::{BoardInfo, Provider};
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
                    source_id: issue.url.clone(),
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

    async fn list_boards(&self) -> Result<Vec<BoardInfo>> {
        Ok(vec![])
    }

    async fn create_item(
        &self,
        title: &str,
        description: Option<&str>,
    ) -> Result<Option<WorkItem>> {
        // Detect the current repo using gh
        let repo_output = tokio::process::Command::new("gh")
            .args(["repo", "view", "--json", "nameWithOwner"])
            .output()
            .await
            .context("Failed to run gh CLI to detect repo")?;

        if !repo_output.status.success() {
            // Not in a git repo or gh not configured — skip
            return Ok(None);
        }

        let repo_info: serde_json::Value =
            serde_json::from_slice(&repo_output.stdout).context("Failed to parse gh repo view")?;
        let repo = repo_info
            .get("nameWithOwner")
            .and_then(|v| v.as_str())
            .context("No nameWithOwner in gh repo view output")?;

        // Build the gh issue create command
        let mut cmd_args = vec![
            "issue".to_string(),
            "create".to_string(),
            "--repo".to_string(),
            repo.to_string(),
            "--title".to_string(),
            title.to_string(),
        ];

        if let Some(desc) = description {
            cmd_args.push("--body".to_string());
            cmd_args.push(desc.to_string());
        }

        let output = tokio::process::Command::new("gh")
            .args(&cmd_args)
            .output()
            .await
            .context("Failed to run gh issue create")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("gh issue create failed: {stderr}");
        }

        // gh issue create outputs the URL of the new issue
        let url = String::from_utf8_lossy(&output.stdout).trim().to_string();

        // Extract the issue number from the URL (e.g., https://github.com/owner/repo/issues/42)
        let number = url
            .rsplit('/')
            .next()
            .unwrap_or("?")
            .to_string();

        let item = WorkItem {
            id: format!("#{number}"),
            source_id: Some(url.clone()),
            title: title.to_string(),
            description: description.map(|d| d.chars().take(500).collect()),
            status: Some("open".to_string()),
            priority: None,
            labels: Vec::new(),
            source: "GitHub".into(),
            team: Some(repo.to_string()),
            url: Some(url),
        };

        Ok(Some(item))
    }

    async fn move_to_done(&self, source_id: &str) -> Result<()> {
        // source_id is the issue URL, close it via gh CLI
        let output = tokio::process::Command::new("gh")
            .args(["issue", "close", source_id])
            .output()
            .await
            .context("Failed to run gh CLI")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("gh issue close failed: {stderr}");
        }

        Ok(())
    }

    async fn move_to_in_progress(&self, source_id: &str) -> Result<()> {
        let output = tokio::process::Command::new("gh")
            .args(["issue", "edit", source_id, "--add-label", "in-progress"])
            .output()
            .await
            .context("Failed to run gh CLI")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("gh issue edit failed: {stderr}");
        }

        Ok(())
    }
}

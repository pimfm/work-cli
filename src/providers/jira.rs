use anyhow::{Context, Result};
use async_trait::async_trait;
use base64::Engine;
use serde::Deserialize;

use super::{BoardInfo, Provider};
use crate::model::work_item::WorkItem;
use crate::util::adf::extract_text_from_adf;

pub struct JiraProvider {
    base_url: String,
    auth_header: String,
    client: reqwest::Client,
}

impl JiraProvider {
    pub fn new(domain: String, email: String, api_token: String) -> Self {
        let creds = format!("{email}:{api_token}");
        let encoded = base64::engine::general_purpose::STANDARD.encode(creds);
        Self {
            base_url: format!("https://{domain}.atlassian.net"),
            auth_header: format!("Basic {encoded}"),
            client: reqwest::Client::new(),
        }
    }
}

#[derive(Deserialize)]
struct SearchResponse {
    issues: Vec<JiraIssue>,
}

#[derive(Deserialize)]
struct JiraIssue {
    key: String,
    fields: IssueFields,
}

#[derive(Deserialize)]
struct IssueFields {
    summary: Option<String>,
    description: Option<serde_json::Value>,
    status: Option<StatusField>,
    priority: Option<PriorityField>,
    #[serde(default)]
    labels: Vec<String>,
    project: Option<ProjectField>,
}

#[derive(Deserialize)]
struct StatusField {
    name: String,
}

#[derive(Deserialize)]
struct PriorityField {
    name: String,
}

#[derive(Deserialize)]
struct ProjectField {
    name: String,
}

#[async_trait]
impl Provider for JiraProvider {
    fn name(&self) -> &str {
        "Jira"
    }

    async fn fetch_items(&self) -> Result<Vec<WorkItem>> {
        let jql = "assignee=currentUser() AND statusCategory!=Done ORDER BY priority ASC";
        let url = format!(
            "{}/rest/api/3/search?jql={}&maxResults=50&fields=summary,description,status,priority,labels,project",
            self.base_url,
            urlencoding::encode(jql)
        );

        let resp = self
            .client
            .get(&url)
            .header("Authorization", &self.auth_header)
            .header("Accept", "application/json")
            .send()
            .await
            .context("Jira API request failed")?;

        let search: SearchResponse = resp.json().await.context("Failed to parse Jira response")?;

        let items = search
            .issues
            .into_iter()
            .map(|issue| {
                let description = issue
                    .fields
                    .description
                    .as_ref()
                    .and_then(|d| extract_text_from_adf(d))
                    .map(|d| d.chars().take(500).collect::<String>());

                let url = format!("{}/browse/{}", self.base_url, issue.key);

                WorkItem {
                    id: issue.key.clone(),
                    source_id: Some(issue.key),
                    title: issue.fields.summary.unwrap_or_default(),
                    description,
                    status: issue.fields.status.map(|s| s.name),
                    priority: issue.fields.priority.map(|p| p.name),
                    labels: issue.fields.labels,
                    source: "Jira".into(),
                    team: issue.fields.project.map(|p| p.name),
                    url: Some(url),
                }
            })
            .collect();

        Ok(items)
    }

    async fn list_boards(&self) -> Result<Vec<BoardInfo>> {
        Ok(vec![])
    }

    async fn move_to_done(&self, source_id: &str) -> Result<()> {
        // Get available transitions for this issue
        let url = format!(
            "{}/rest/api/3/issue/{}/transitions",
            self.base_url, source_id
        );

        let resp: serde_json::Value = self
            .client
            .get(&url)
            .header("Authorization", &self.auth_header)
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to fetch Jira transitions")?
            .json()
            .await?;

        // Find a transition to the "done" status category
        let transition_id = resp
            .get("transitions")
            .and_then(|t| t.as_array())
            .and_then(|transitions| {
                transitions.iter().find_map(|t| {
                    let category = t.pointer("/to/statusCategory/key")?.as_str()?;
                    if category == "done" {
                        t.get("id")?.as_str().map(|s| s.to_string())
                    } else {
                        None
                    }
                })
            })
            .context("No transition to Done status found")?;

        // Execute the transition
        let body = serde_json::json!({
            "transition": { "id": transition_id }
        });

        self.client
            .post(&url)
            .header("Authorization", &self.auth_header)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to transition Jira issue to Done")?;

        Ok(())
    }
}

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;

use super::{BoardInfo, Provider};
use crate::model::work_item::WorkItem;

pub struct LinearProvider {
    api_key: String,
    client: reqwest::Client,
}

impl LinearProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
        }
    }
}

const QUERY: &str = r#"{
  viewer {
    assignedIssues(
      filter: { state: { type: { nin: ["completed", "canceled"] } } }
      first: 50
    ) {
      nodes {
        id identifier title description priority url
        state { name }
        team { name }
        labels { nodes { name } }
      }
    }
  }
}"#;

#[derive(Deserialize)]
struct GqlResponse {
    data: Option<GqlData>,
}

#[derive(Deserialize)]
struct GqlData {
    viewer: Viewer,
}

#[derive(Deserialize)]
struct Viewer {
    #[serde(rename = "assignedIssues")]
    assigned_issues: IssueConnection,
}

#[derive(Deserialize)]
struct IssueConnection {
    nodes: Vec<Issue>,
}

#[derive(Deserialize)]
struct Issue {
    id: String,
    identifier: String,
    title: String,
    description: Option<String>,
    priority: Option<u8>,
    url: Option<String>,
    state: Option<State>,
    team: Option<Team>,
    labels: Option<LabelConnection>,
}

#[derive(Deserialize)]
struct State {
    name: String,
}

#[derive(Deserialize)]
struct Team {
    name: String,
}

#[derive(Deserialize)]
struct LabelConnection {
    nodes: Vec<Label>,
}

#[derive(Deserialize)]
struct Label {
    name: String,
}

fn map_priority(p: Option<u8>) -> Option<String> {
    match p {
        Some(1) => Some("Urgent".into()),
        Some(2) => Some("High".into()),
        Some(3) => Some("Medium".into()),
        Some(4) => Some("Low".into()),
        _ => None,
    }
}

#[async_trait]
impl Provider for LinearProvider {
    fn name(&self) -> &str {
        "Linear"
    }

    async fn fetch_items(&self) -> Result<Vec<WorkItem>> {
        let body = serde_json::json!({ "query": QUERY });
        let resp = self
            .client
            .post("https://api.linear.app/graphql")
            .header("Authorization", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Linear API request failed")?;

        let gql: GqlResponse = resp.json().await.context("Failed to parse Linear response")?;
        let data = gql.data.context("No data in Linear response")?;

        let items = data
            .viewer
            .assigned_issues
            .nodes
            .into_iter()
            .map(|issue| {
                let description = issue
                    .description
                    .map(|d| d.chars().take(500).collect::<String>());
                let labels = issue
                    .labels
                    .map(|lc| lc.nodes.into_iter().map(|l| l.name).collect())
                    .unwrap_or_default();

                WorkItem {
                    id: issue.identifier,
                    source_id: Some(issue.id),
                    title: issue.title,
                    description,
                    status: issue.state.map(|s| s.name),
                    priority: map_priority(issue.priority),
                    labels,
                    source: "Linear".into(),
                    team: issue.team.map(|t| t.name),
                    url: issue.url,
                }
            })
            .collect();

        Ok(items)
    }

    async fn list_boards(&self) -> Result<Vec<BoardInfo>> {
        Ok(vec![])
    }

    async fn move_to_done(&self, source_id: &str) -> Result<()> {
        // Find the issue's team and its completed workflow state
        let query = r#"query($id: String!) {
          issue(id: $id) {
            team {
              states(filter: { type: { eq: "completed" } }) {
                nodes { id name }
              }
            }
          }
        }"#;

        let body = serde_json::json!({
            "query": query,
            "variables": { "id": source_id }
        });

        let resp: serde_json::Value = self
            .client
            .post("https://api.linear.app/graphql")
            .header("Authorization", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Linear API request failed")?
            .json()
            .await?;

        let state_id = resp
            .pointer("/data/issue/team/states/nodes/0/id")
            .and_then(|v| v.as_str())
            .context("No completed state found for issue's team")?
            .to_string();

        // Update the issue state
        let mutation = r#"mutation($id: String!, $stateId: String!) {
          issueUpdate(id: $id, input: { stateId: $stateId }) {
            success
          }
        }"#;

        let body = serde_json::json!({
            "query": mutation,
            "variables": { "id": source_id, "stateId": state_id }
        });

        self.client
            .post("https://api.linear.app/graphql")
            .header("Authorization", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to update Linear issue state")?;

        Ok(())
    }

    async fn create_item(&self, title: &str, description: Option<&str>) -> Result<Option<WorkItem>> {
        // First get the viewer's first team
        let team_query = r#"{ viewer { teams(first: 1) { nodes { id name } } } }"#;
        let body = serde_json::json!({ "query": team_query });

        let resp: serde_json::Value = self
            .client
            .post("https://api.linear.app/graphql")
            .header("Authorization", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Linear API request failed")?
            .json()
            .await?;

        let team_id = resp
            .pointer("/data/viewer/teams/nodes/0/id")
            .and_then(|v| v.as_str())
            .context("No team found for Linear user")?
            .to_string();

        let team_name = resp
            .pointer("/data/viewer/teams/nodes/0/name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        // Create the issue
        let mutation = r#"mutation($title: String!, $teamId: String!, $description: String) {
          issueCreate(input: { title: $title, teamId: $teamId, description: $description }) {
            success
            issue { id identifier title description url state { name } }
          }
        }"#;

        let mut variables = serde_json::json!({
            "title": title,
            "teamId": team_id,
        });
        if let Some(desc) = description {
            variables["description"] = serde_json::Value::String(desc.to_string());
        }

        let body = serde_json::json!({
            "query": mutation,
            "variables": variables,
        });

        let resp: serde_json::Value = self
            .client
            .post("https://api.linear.app/graphql")
            .header("Authorization", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to create Linear issue")?
            .json()
            .await?;

        let issue = resp.pointer("/data/issueCreate/issue")
            .context("No issue in create response")?;

        let item = WorkItem {
            id: issue.get("identifier").and_then(|v| v.as_str()).unwrap_or("?").to_string(),
            source_id: issue.get("id").and_then(|v| v.as_str()).map(String::from),
            title: title.to_string(),
            description: description.map(String::from),
            status: issue.pointer("/state/name").and_then(|v| v.as_str()).map(String::from),
            priority: None,
            labels: Vec::new(),
            source: "Linear".into(),
            team: Some(team_name),
            url: issue.get("url").and_then(|v| v.as_str()).map(String::from),
        };

        Ok(Some(item))
    }

    async fn move_to_in_progress(&self, source_id: &str) -> Result<()> {
        let query = r#"query($id: String!) {
          issue(id: $id) {
            team {
              states(filter: { type: { eq: "started" } }) {
                nodes { id name }
              }
            }
          }
        }"#;

        let body = serde_json::json!({
            "query": query,
            "variables": { "id": source_id }
        });

        let resp: serde_json::Value = self
            .client
            .post("https://api.linear.app/graphql")
            .header("Authorization", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Linear API request failed")?
            .json()
            .await?;

        let state_id = resp
            .pointer("/data/issue/team/states/nodes/0/id")
            .and_then(|v| v.as_str())
            .context("No 'started' state found for issue's team")?
            .to_string();

        let mutation = r#"mutation($id: String!, $stateId: String!) {
          issueUpdate(id: $id, input: { stateId: $stateId }) {
            success
          }
        }"#;

        let body = serde_json::json!({
            "query": mutation,
            "variables": { "id": source_id, "stateId": state_id }
        });

        self.client
            .post("https://api.linear.app/graphql")
            .header("Authorization", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to update Linear issue to In Progress")?;

        Ok(())
    }
}

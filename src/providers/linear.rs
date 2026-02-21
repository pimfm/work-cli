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
}

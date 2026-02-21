use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;

use super::{BoardInfo, Provider};
use crate::model::work_item::WorkItem;

pub struct TrelloProvider {
    api_key: String,
    token: String,
    client: reqwest::Client,
    board_id: Option<String>,
}

impl TrelloProvider {
    pub fn new(api_key: String, token: String) -> Self {
        Self {
            api_key,
            token,
            client: reqwest::Client::new(),
            board_id: None,
        }
    }

    fn auth_params(&self) -> [(&str, &str); 2] {
        [("key", &self.api_key), ("token", &self.token)]
    }
}

#[derive(Deserialize)]
struct Member {
    id: String,
}

#[derive(Deserialize)]
struct Board {
    id: String,
    name: String,
}

#[derive(Deserialize)]
struct TrelloList {
    id: String,
    name: String,
}

#[derive(Deserialize)]
struct TrelloLabel {
    name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Card {
    id: String,
    name: String,
    desc: Option<String>,
    short_url: Option<String>,
    id_list: Option<String>,
    id_board: Option<String>,
    labels: Option<Vec<TrelloLabel>>,
}

const EXCLUDED_LISTS: &[&str] = &["done", "in review"];

#[async_trait]
impl Provider for TrelloProvider {
    fn name(&self) -> &str {
        "Trello"
    }

    async fn fetch_items(&self) -> Result<Vec<WorkItem>> {
        let base = "https://api.trello.com/1";

        // Get member ID
        let member: Member = self
            .client
            .get(format!("{base}/members/me"))
            .query(&self.auth_params())
            .send()
            .await
            .context("Trello members/me failed")?
            .json()
            .await?;

        let (boards, cards) = if let Some(bid) = &self.board_id {
            // Board-filtered: fetch only cards and board info for the specific board
            let board_fut = self
                .client
                .get(format!("{base}/boards/{bid}"))
                .query(&self.auth_params())
                .query(&[("fields", "id,name")])
                .send();

            let cards_fut = self
                .client
                .get(format!("{base}/boards/{bid}/cards"))
                .query(&self.auth_params())
                .query(&[(
                    "fields",
                    "id,name,desc,shortUrl,idList,labels,idBoard",
                )])
                .send();

            let (board_resp, cards_resp) = tokio::try_join!(board_fut, cards_fut)?;
            let board: Board = board_resp.json().await?;
            let cards: Vec<Card> = cards_resp.json().await?;
            (vec![board], cards)
        } else {
            // Unfiltered: fetch all boards and cards
            let boards_fut = self
                .client
                .get(format!("{base}/members/{}/boards", member.id))
                .query(&self.auth_params())
                .query(&[("fields", "id,name"), ("filter", "open")])
                .send();

            let cards_fut = self
                .client
                .get(format!("{base}/members/{}/cards", member.id))
                .query(&self.auth_params())
                .query(&[(
                    "fields",
                    "id,name,desc,shortUrl,idList,labels,idBoard",
                )])
                .send();

            let (boards_resp, cards_resp) = tokio::try_join!(boards_fut, cards_fut)?;
            let boards: Vec<Board> = boards_resp.json().await?;
            let cards: Vec<Card> = cards_resp.json().await?;
            (boards, cards)
        };

        let board_map: HashMap<String, String> =
            boards.into_iter().map(|b| (b.id, b.name)).collect();

        // Fetch lists for each board that has cards
        let board_ids: Vec<String> = cards
            .iter()
            .filter_map(|c| c.id_board.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        let mut list_map: HashMap<String, String> = HashMap::new();
        for board_id in &board_ids {
            let lists: Vec<TrelloList> = self
                .client
                .get(format!("{base}/boards/{board_id}/lists"))
                .query(&self.auth_params())
                .query(&[("fields", "id,name")])
                .send()
                .await?
                .json()
                .await?;
            for list in lists {
                list_map.insert(list.id, list.name);
            }
        }

        let items = cards
            .into_iter()
            .filter(|card| {
                if let Some(list_id) = &card.id_list {
                    if let Some(list_name) = list_map.get(list_id) {
                        let lower = list_name.to_lowercase();
                        return !EXCLUDED_LISTS.iter().any(|ex| lower == *ex);
                    }
                }
                true
            })
            .map(|card| {
                let status = card
                    .id_list
                    .as_ref()
                    .and_then(|id| list_map.get(id))
                    .cloned();
                let team = card
                    .id_board
                    .as_ref()
                    .and_then(|id| board_map.get(id))
                    .cloned();
                let labels = card
                    .labels
                    .unwrap_or_default()
                    .into_iter()
                    .filter(|l| !l.name.is_empty())
                    .map(|l| l.name)
                    .collect();
                let description = card
                    .desc
                    .filter(|d| !d.trim().is_empty())
                    .map(|d| d.chars().take(500).collect::<String>());

                WorkItem {
                    id: card.id[..8.min(card.id.len())].to_string(),
                    source_id: Some(card.id.clone()),
                    title: card.name,
                    description,
                    status,
                    priority: None,
                    labels,
                    source: "Trello".into(),
                    team,
                    url: card.short_url,
                }
            })
            .collect();

        Ok(items)
    }

    async fn list_boards(&self) -> Result<Vec<BoardInfo>> {
        let base = "https://api.trello.com/1";

        let member: Member = self
            .client
            .get(format!("{base}/members/me"))
            .query(&self.auth_params())
            .send()
            .await
            .context("Trello members/me failed")?
            .json()
            .await?;

        let boards: Vec<Board> = self
            .client
            .get(format!("{base}/members/{}/boards", member.id))
            .query(&self.auth_params())
            .query(&[("fields", "id,name"), ("filter", "open")])
            .send()
            .await?
            .json()
            .await?;

        Ok(boards
            .into_iter()
            .map(|b| BoardInfo {
                id: b.id,
                name: b.name,
                source: "Trello".into(),
            })
            .collect())
    }

    fn set_board_filter(&mut self, board_id: String) {
        self.board_id = Some(board_id);
    }

    async fn move_to_done(&self, source_id: &str) -> Result<()> {
        let base = "https://api.trello.com/1";

        // Get the card's board ID
        let card: Card = self
            .client
            .get(format!("{base}/cards/{source_id}"))
            .query(&self.auth_params())
            .query(&[("fields", "idBoard")])
            .send()
            .await
            .context("Failed to fetch Trello card")?
            .json()
            .await?;

        let board_id = card
            .id_board
            .context("Card has no board ID")?;

        // Get the board's lists and find one named "Done"
        let lists: Vec<TrelloList> = self
            .client
            .get(format!("{base}/boards/{board_id}/lists"))
            .query(&self.auth_params())
            .query(&[("fields", "id,name")])
            .send()
            .await?
            .json()
            .await?;

        let done_list = lists
            .iter()
            .find(|l| l.name.eq_ignore_ascii_case("done"))
            .context("No 'Done' list found on board")?;

        // Move card to Done list
        self.client
            .put(format!("{base}/cards/{source_id}"))
            .query(&self.auth_params())
            .query(&[("idList", &done_list.id)])
            .send()
            .await
            .context("Failed to move Trello card to Done")?;

        Ok(())
    }

    async fn create_item(
        &self,
        title: &str,
        description: Option<&str>,
    ) -> Result<Option<WorkItem>> {
        let board_id = match &self.board_id {
            Some(id) => id.clone(),
            None => return Ok(None), // No board selected — can't create
        };

        let base = "https://api.trello.com/1";

        // Get the board's lists and find a suitable one for new cards
        let lists: Vec<TrelloList> = self
            .client
            .get(format!("{base}/boards/{board_id}/lists"))
            .query(&self.auth_params())
            .query(&[("fields", "id,name")])
            .send()
            .await
            .context("Failed to fetch Trello board lists")?
            .json()
            .await?;

        // Prefer "Todo"/"To Do"/"Backlog", fall back to the first list
        let target_list = lists
            .iter()
            .find(|l| {
                let lower = l.name.to_lowercase();
                lower == "todo" || lower == "to do" || lower == "backlog"
            })
            .or_else(|| lists.first())
            .context("Board has no lists — cannot create card")?;

        let list_id = &target_list.id;
        let list_name = &target_list.name;

        // Create the card
        let mut params: Vec<(&str, &str)> = vec![
            ("key", &self.api_key),
            ("token", &self.token),
            ("idList", list_id),
            ("name", title),
        ];
        let desc_str;
        if let Some(d) = description {
            desc_str = d.to_string();
            params.push(("desc", &desc_str));
        }

        let card: Card = self
            .client
            .post(format!("{base}/cards"))
            .query(&params)
            .send()
            .await
            .context("Failed to create Trello card")?
            .json()
            .await
            .context("Failed to parse Trello create card response")?;

        let item = WorkItem {
            id: card.id[..8.min(card.id.len())].to_string(),
            source_id: Some(card.id),
            title: card.name,
            description: card
                .desc
                .filter(|d| !d.trim().is_empty())
                .map(|d| d.chars().take(500).collect()),
            status: Some(list_name.clone()),
            priority: None,
            labels: card
                .labels
                .unwrap_or_default()
                .into_iter()
                .filter(|l| !l.name.is_empty())
                .map(|l| l.name)
                .collect(),
            source: "Trello".into(),
            team: None,
            url: card.short_url,
        };

        Ok(Some(item))
    }

    async fn move_to_in_progress(&self, source_id: &str) -> Result<()> {
        let base = "https://api.trello.com/1";

        let card: Card = self
            .client
            .get(format!("{base}/cards/{source_id}"))
            .query(&self.auth_params())
            .query(&[("fields", "idBoard")])
            .send()
            .await
            .context("Failed to fetch Trello card")?
            .json()
            .await?;

        let board_id = card
            .id_board
            .context("Card has no board ID")?;

        let lists: Vec<TrelloList> = self
            .client
            .get(format!("{base}/boards/{board_id}/lists"))
            .query(&self.auth_params())
            .query(&[("fields", "id,name")])
            .send()
            .await?
            .json()
            .await?;

        let in_progress_list = lists
            .iter()
            .find(|l| {
                let lower = l.name.to_lowercase();
                lower == "in progress" || lower == "doing" || lower == "in-progress"
            })
            .context("No 'In Progress' or 'Doing' list found on board")?;

        self.client
            .put(format!("{base}/cards/{source_id}"))
            .query(&self.auth_params())
            .query(&[("idList", &in_progress_list.id)])
            .send()
            .await
            .context("Failed to move Trello card to In Progress")?;

        Ok(())
    }
}

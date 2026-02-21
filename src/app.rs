use std::time::Instant;

use tokio::sync::mpsc;

use crate::agents::dispatch;
use crate::agents::log::{append_event, new_event, read_events, AgentEvent};
use crate::agents::retry::MAX_RETRIES;
use crate::agents::store::AgentStore;
use crate::config::{self, AppConfig, BoardMapping};
use crate::event::KeyAction;
use crate::model::agent::{AgentName, AgentStatus};
use crate::model::work_item::WorkItem;
use crate::providers::{self, BoardInfo, Provider};

#[derive(Debug, Clone)]
pub enum Action {
    Key(KeyAction),
    Tick,
    WorkItemsLoaded(Vec<WorkItem>),
    FetchError(String),
    #[allow(dead_code)]
    PollAgents,
    AgentProcessExited(AgentName, bool),
    Quit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewMode {
    BoardSelection,
    Items,
    Agents,
    AgentDetail(AgentName),
}

pub struct App {
    pub items: Vec<WorkItem>,
    pub selected_item: usize,
    pub view_mode: ViewMode,
    pub selected_agent: usize,
    pub agent_log_scroll: usize,
    pub auto_mode: bool,
    pub loading: bool,
    pub flash_message: Option<(String, Instant)>,
    pub store: AgentStore,
    pub repo_root: String,
    pub should_quit: bool,
    pub action_tx: mpsc::UnboundedSender<Action>,
    pub available_boards: Vec<BoardInfo>,
    pub selected_board: usize,
    pub project_dir: String,
    providers: Vec<Box<dyn Provider>>,
    dispatched_item_ids: std::collections::HashSet<String>,
}

impl App {
    pub fn new(
        config: &AppConfig,
        store: AgentStore,
        action_tx: mpsc::UnboundedSender<Action>,
    ) -> Self {
        let repo_root = config
            .agents
            .as_ref()
            .and_then(|a| a.repo_root.clone())
            .unwrap_or_else(|| {
                std::env::current_dir()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            });

        let project_dir = std::env::current_dir()
            .ok()
            .and_then(|p| p.canonicalize().ok())
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let mut providers = providers::create_providers(config);

        // Check board mappings for current directory
        let mappings = config::load_board_mappings();
        let has_mapping = if let Some(mapping) = mappings.get(&project_dir) {
            // Apply board filter to the matching provider
            for provider in &mut providers {
                if provider.name() == mapping.source {
                    provider.set_board_filter(mapping.board_id.clone());
                }
            }
            true
        } else {
            false
        };

        let view_mode = if has_mapping {
            ViewMode::Items
        } else {
            ViewMode::BoardSelection
        };

        Self {
            items: Vec::new(),
            selected_item: 0,
            view_mode,
            selected_agent: 0,
            agent_log_scroll: 0,
            auto_mode: false,
            loading: !has_mapping,
            flash_message: None,
            store,
            repo_root,
            should_quit: false,
            action_tx,
            available_boards: Vec::new(),
            selected_board: 0,
            project_dir,
            providers,
            dispatched_item_ids: std::collections::HashSet::new(),
        }
    }

    pub async fn update(&mut self, action: Action) {
        // Clear flash message after 3 seconds
        if let Some((_, t)) = &self.flash_message {
            if t.elapsed().as_secs() >= 3 {
                self.flash_message = None;
            }
        }

        match action {
            Action::Key(key) => self.handle_key(key).await,
            Action::Tick => self.handle_tick().await,
            Action::WorkItemsLoaded(items) => {
                self.items = items;
                self.loading = false;
                if self.selected_item >= self.items.len() && !self.items.is_empty() {
                    self.selected_item = self.items.len() - 1;
                }
            }
            Action::FetchError(msg) => {
                self.loading = false;
                self.flash_message = Some((format!("Fetch error: {msg}"), Instant::now()));
            }
            Action::PollAgents => {
                let _ = self.store.reload();
            }
            Action::AgentProcessExited(name, success) => {
                let _ = self.store.reload();
                if success {
                    // Move work item to done in source system
                    if let Some(agent) = self.store.get_agent(name) {
                        if let Some(item_id) = agent.work_item_id.clone() {
                            if let Some(item) = self.items.iter().find(|i| i.id == item_id) {
                                self.move_item_to_done(item.clone()).await;
                            }
                        }
                    }
                    let _ = self.store.mark_done(name);
                } else {
                    let _ = self.store.mark_error(name, "Process failed");
                }
            }
            Action::Quit => {
                self.should_quit = true;
            }
        }
    }

    async fn handle_key(&mut self, key: KeyAction) {
        match key {
            KeyAction::Up => match &self.view_mode {
                ViewMode::BoardSelection => {
                    if self.selected_board > 0 {
                        self.selected_board -= 1;
                    }
                }
                ViewMode::Items => {
                    if self.selected_item > 0 {
                        self.selected_item -= 1;
                    }
                }
                ViewMode::Agents => {
                    if self.selected_agent > 0 {
                        self.selected_agent -= 1;
                    }
                }
                ViewMode::AgentDetail(_) => {
                    if self.agent_log_scroll > 0 {
                        self.agent_log_scroll -= 1;
                    }
                }
            },
            KeyAction::Down => match &self.view_mode {
                ViewMode::BoardSelection => {
                    if !self.available_boards.is_empty()
                        && self.selected_board < self.available_boards.len() - 1
                    {
                        self.selected_board += 1;
                    }
                }
                ViewMode::Items => {
                    if !self.items.is_empty() && self.selected_item < self.items.len() - 1 {
                        self.selected_item += 1;
                    }
                }
                ViewMode::Agents => {
                    if self.selected_agent < AgentName::ALL.len() - 1 {
                        self.selected_agent += 1;
                    }
                }
                ViewMode::AgentDetail(_) => {
                    self.agent_log_scroll += 1;
                }
            },
            KeyAction::Select => {
                if self.view_mode == ViewMode::BoardSelection && !self.available_boards.is_empty() {
                    self.select_board().await;
                }
            }
            KeyAction::Right => match &self.view_mode {
                ViewMode::BoardSelection => {}
                ViewMode::Items => {
                    self.view_mode = ViewMode::Agents;
                    self.selected_agent = 0;
                }
                ViewMode::Agents => {
                    let agent_name = AgentName::ALL[self.selected_agent];
                    self.view_mode = ViewMode::AgentDetail(agent_name);
                    self.agent_log_scroll = 0;
                }
                ViewMode::AgentDetail(_) => {}
            },
            KeyAction::Left => match &self.view_mode {
                ViewMode::BoardSelection => {}
                ViewMode::Items => {}
                ViewMode::Agents => {
                    self.view_mode = ViewMode::Items;
                }
                ViewMode::AgentDetail(_) => {
                    self.view_mode = ViewMode::Agents;
                }
            },
            KeyAction::Dispatch => {
                if self.view_mode == ViewMode::Items {
                    self.dispatch_selected().await;
                }
            }
            KeyAction::ToggleAutoMode => {
                self.auto_mode = !self.auto_mode;
                let status = if self.auto_mode { "ON" } else { "OFF" };
                self.flash_message =
                    Some((format!("Auto mode: {status}"), Instant::now()));
            }
            KeyAction::Refresh => {
                self.refresh_items().await;
            }
        }
    }

    async fn handle_tick(&mut self) {
        let _ = self.store.reload();

        // Auto-release done agents
        let done_agents: Vec<AgentName> = self
            .store
            .get_all()
            .iter()
            .filter(|a| a.status == AgentStatus::Done)
            .map(|a| a.name)
            .collect();
        for name in done_agents {
            let _ = append_event(&new_event(name, "released", None, None, None));
            let _ = self.store.release(name);
        }

        // Auto-retry errored agents
        let errored_agents: Vec<AgentName> = self
            .store
            .get_all()
            .iter()
            .filter(|a| a.status == AgentStatus::Error)
            .map(|a| a.name)
            .collect();
        for name in errored_agents {
            let retry_count = self.store.increment_retry(name).unwrap_or(0);
            if retry_count <= MAX_RETRIES {
                let _ = append_event(&new_event(
                    name,
                    "retry",
                    None,
                    None,
                    Some(&format!("Retry {retry_count}/{MAX_RETRIES}")),
                ));
                // Re-dispatch with same work item if we have it
                if let Some(agent) = self.store.get_agent(name) {
                    if let (Some(item_id), Some(_item_title)) =
                        (agent.work_item_id.clone(), agent.work_item_title.clone())
                    {
                        if let Some(item) = self.items.iter().find(|i| i.id == item_id) {
                            let item = item.clone();
                            let _ = dispatch::dispatch(
                                name,
                                &item,
                                &self.repo_root,
                                &mut self.store,
                                self.action_tx.clone(),
                            )
                            .await;
                        } else {
                            // Item not in list anymore, just release
                            let _ = self.store.release(name);
                        }
                    }
                }
            } else {
                let _ = append_event(&new_event(
                    name,
                    "max-retries",
                    None,
                    None,
                    Some("Max retries reached"),
                ));
                let _ = self.store.release(name);
            }
        }

        // Auto mode: dispatch to free agents
        if self.auto_mode {
            self.auto_dispatch().await;
        }
    }

    async fn auto_dispatch(&mut self) {
        loop {
            let free_agent = self.store.next_free_agent();
            let free_agent = match free_agent {
                Some(a) => a,
                None => break,
            };

            // Find next unassigned item
            let next_item = self
                .items
                .iter()
                .find(|item| !self.dispatched_item_ids.contains(&item.id))
                .cloned();

            match next_item {
                Some(item) => {
                    self.dispatched_item_ids.insert(item.id.clone());
                    let _ = dispatch::dispatch(
                        free_agent,
                        &item,
                        &self.repo_root,
                        &mut self.store,
                        self.action_tx.clone(),
                    )
                    .await;
                }
                None => break,
            }
        }
    }

    async fn dispatch_selected(&mut self) {
        if self.items.is_empty() {
            return;
        }
        let item = self.items[self.selected_item].clone();

        let free_agent = self.store.next_free_agent();
        match free_agent {
            Some(agent_name) => {
                self.dispatched_item_ids.insert(item.id.clone());
                match dispatch::dispatch(
                    agent_name,
                    &item,
                    &self.repo_root,
                    &mut self.store,
                    self.action_tx.clone(),
                )
                .await
                {
                    Ok(_) => {
                        self.flash_message = Some((
                            format!(
                                "{} dispatched to {}",
                                item.id,
                                agent_name.display_name()
                            ),
                            Instant::now(),
                        ));
                    }
                    Err(e) => {
                        self.flash_message =
                            Some((format!("Dispatch failed: {e}"), Instant::now()));
                    }
                }
            }
            None => {
                self.flash_message = Some(("All agents busy".into(), Instant::now()));
            }
        }
    }

    pub async fn fetch_boards(&mut self) {
        self.loading = true;
        let mut all_boards = Vec::new();
        for provider in &self.providers {
            match provider.list_boards().await {
                Ok(boards) => all_boards.extend(boards),
                Err(e) => {
                    let _ = self
                        .action_tx
                        .send(Action::FetchError(format!("{}: {e}", provider.name())));
                }
            }
        }
        self.available_boards = all_boards;
        self.selected_board = 0;
        self.loading = false;
    }

    async fn select_board(&mut self) {
        let board = &self.available_boards[self.selected_board];
        let mapping = BoardMapping {
            board_id: board.id.clone(),
            board_name: board.name.clone(),
            source: board.source.clone(),
        };

        // Save mapping
        if let Err(e) = config::save_board_mapping(&self.project_dir, &mapping) {
            self.flash_message = Some((format!("Failed to save mapping: {e}"), Instant::now()));
            return;
        }

        // Apply board filter to the matching provider
        for provider in &mut self.providers {
            if provider.name() == mapping.source {
                provider.set_board_filter(mapping.board_id.clone());
            }
        }

        self.flash_message = Some((format!("Board: {}", mapping.board_name), Instant::now()));
        self.view_mode = ViewMode::Items;
        self.refresh_items().await;
    }

    pub async fn refresh_items(&mut self) {
        self.loading = true;
        let tx = self.action_tx.clone();

        let mut all_items = Vec::new();
        let mut errors = Vec::new();

        // Fetch from all providers (we need to do this on the current task since providers aren't Send-safe with references)
        for provider in &self.providers {
            match provider.fetch_items().await {
                Ok(items) => all_items.extend(items),
                Err(e) => errors.push(format!("{}: {e}", provider.name())),
            }
        }

        if !errors.is_empty() {
            let _ = tx.send(Action::FetchError(errors.join("; ")));
        }
        let _ = tx.send(Action::WorkItemsLoaded(all_items));
    }

    pub fn agent_events(&self, name: AgentName) -> Vec<AgentEvent> {
        read_events(Some(name), Some(200))
    }

    async fn move_item_to_done(&mut self, item: WorkItem) {
        if let Some(source_id) = &item.source_id {
            for provider in &self.providers {
                if provider.name() == item.source {
                    match provider.move_to_done(source_id).await {
                        Ok(_) => {
                            self.flash_message = Some((
                                format!("{} moved to done", item.id),
                                Instant::now(),
                            ));
                        }
                        Err(e) => {
                            self.flash_message = Some((
                                format!("Failed to move {} to done: {e}", item.id),
                                Instant::now(),
                            ));
                        }
                    }
                    break;
                }
            }
        }
    }

    pub fn assigned_agent(&self, item_id: &str) -> Option<AgentName> {
        self.store.get_all().iter().find_map(|a| {
            if a.work_item_id.as_deref() == Some(item_id)
                && matches!(
                    a.status,
                    AgentStatus::Working | AgentStatus::Provisioning | AgentStatus::Done
                )
            {
                Some(a.name)
            } else {
                None
            }
        })
    }
}

use std::time::Instant;

use tokio::sync::mpsc;

use crate::agents::dispatch;
use crate::agents::log::{append_event, clear_events, new_event, read_events, AgentEvent};
use crate::agents::message;
use crate::agents::retry::MAX_RETRIES;
use crate::agents::store::AgentStore;
use crate::config::{self, AppConfig, BoardMapping};
use crate::event::KeyAction;
use crate::model::agent::{AgentName, AgentStatus};
use crate::model::chat::ChatMessage;
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
    AgentResponse(AgentName, String),
    AgentResponseError(AgentName, String),
    TaskCreated(WorkItem),
    TaskCreateError(String),
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

    // Input & chat state
    pub input_active: bool,
    pub input_buffer: String,
    pub input_cursor: usize,
    pub chat_messages: Vec<ChatMessage>,
    #[allow(dead_code)]
    pub chat_scroll: usize,
    pub waiting_for_response: bool,
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
            input_active: false,
            input_buffer: String::new(),
            input_cursor: 0,
            chat_messages: Vec::new(),
            chat_scroll: 0,
            waiting_for_response: false,
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
            Action::Key(key) => {
                if self.input_active {
                    self.handle_input_key(key).await;
                } else {
                    self.handle_key(key).await;
                }
            }
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
            Action::AgentResponse(name, response) => {
                self.waiting_for_response = false;
                self.chat_messages.push(ChatMessage::agent(name, response));
            }
            Action::AgentResponseError(name, error) => {
                self.waiting_for_response = false;
                self.chat_messages.push(ChatMessage::system(format!(
                    "{} error: {}",
                    name.display_name(),
                    error
                )));
            }
            Action::TaskCreated(item) => {
                self.chat_messages
                    .push(ChatMessage::system(format!("Task created: {}", item.title)));
                self.items.push(item);
                // In auto mode, it will be picked up on next tick
                if !self.auto_mode {
                    self.flash_message = Some(("New task added — press d to dispatch".into(), Instant::now()));
                }
            }
            Action::TaskCreateError(msg) => {
                self.chat_messages
                    .push(ChatMessage::system(format!("Failed to create task: {msg}")));
            }
            Action::Quit => {
                self.should_quit = true;
            }
        }
    }

    async fn handle_input_key(&mut self, key: KeyAction) {
        match key {
            KeyAction::Escape => {
                self.input_active = false;
                self.input_buffer.clear();
                self.input_cursor = 0;
            }
            KeyAction::Select => {
                // Enter submits the input
                let input = self.input_buffer.clone();
                self.input_buffer.clear();
                self.input_cursor = 0;
                self.input_active = false;
                if !input.trim().is_empty() {
                    self.process_command(input).await;
                }
            }
            KeyAction::Backspace => {
                if self.input_cursor > 0 {
                    self.input_cursor -= 1;
                    self.input_buffer.remove(self.input_cursor);
                }
            }
            KeyAction::Left => {
                if self.input_cursor > 0 {
                    self.input_cursor -= 1;
                }
            }
            KeyAction::Right => {
                if self.input_cursor < self.input_buffer.len() {
                    self.input_cursor += 1;
                }
            }
            KeyAction::Char(c) => {
                self.input_buffer.insert(self.input_cursor, c);
                self.input_cursor += 1;
            }
            KeyAction::Tab => {
                // Auto-complete agent names
                self.autocomplete_agent();
            }
            _ => {}
        }
    }

    fn autocomplete_agent(&mut self) {
        if !self.input_buffer.starts_with('@') {
            return;
        }
        let partial = &self.input_buffer[1..];
        for name in AgentName::ALL {
            if name.as_str().starts_with(partial) && partial.len() < name.as_str().len() {
                self.input_buffer = format!("@{} ", name.as_str());
                self.input_cursor = self.input_buffer.len();
                return;
            }
        }
    }

    async fn process_command(&mut self, input: String) {
        if input.starts_with('@') {
            self.process_agent_message(input).await;
        } else {
            self.process_task_creation(input).await;
        }
    }

    async fn process_agent_message(&mut self, input: String) {
        // Parse @agent_name message
        let after_at = &input[1..];
        let mut target_agent = None;
        let mut agent_message = "";

        for name in AgentName::ALL {
            let prefix = name.as_str();
            if after_at.starts_with(prefix) {
                let rest = &after_at[prefix.len()..];
                if rest.is_empty() || rest.starts_with(' ') {
                    target_agent = Some(name);
                    agent_message = rest.trim();
                    break;
                }
            }
        }

        let agent_name = match target_agent {
            Some(n) => n,
            None => {
                self.chat_messages.push(ChatMessage::system(
                    "Unknown agent. Use @ember, @flow, @tempest, or @terra".to_string(),
                ));
                return;
            }
        };

        if agent_message.is_empty() {
            self.chat_messages.push(ChatMessage::system(format!(
                "Send a message: @{} <your message>",
                agent_name.as_str()
            )));
            return;
        }

        // Add user message to chat
        self.chat_messages.push(ChatMessage::user(input.clone()));

        // Determine work directory and task context
        let agent = self.store.get_agent(agent_name);
        let work_dir;
        let task_context;
        let is_working;

        if let Some(agent) = agent {
            is_working = agent.status == AgentStatus::Working;
            work_dir = agent
                .worktree_path
                .clone()
                .unwrap_or_else(|| self.repo_root.clone());
            task_context = agent.work_item_title.clone();
        } else {
            is_working = false;
            work_dir = self.repo_root.clone();
            task_context = None;
        }

        // Check if the message is feedback for a working/done/error agent
        let is_feedback = agent.map_or(false, |a| {
            matches!(
                a.status,
                AgentStatus::Working | AgentStatus::Done | AgentStatus::Error
            )
        });

        self.waiting_for_response = true;

        if is_working {
            // Agent is busy — tell user and queue the feedback
            self.chat_messages.push(ChatMessage::system(format!(
                "{} is currently working. Sending feedback that will be applied when done...",
                agent_name.display_name()
            )));
        }

        let tx = self.action_tx.clone();
        let msg = agent_message.to_string();
        let ctx = task_context.clone();

        // Log the interaction
        let _ = append_event(&new_event(
            agent_name,
            "user-message",
            None,
            task_context.as_deref(),
            Some(agent_message),
        ));

        if is_feedback && !is_working {
            // Apply feedback directly — agent can make changes
            let wd = work_dir.clone();
            let tc = ctx.unwrap_or_else(|| "No specific task".to_string());
            tokio::spawn(async move {
                match message::apply_feedback(agent_name, &msg, &wd, &tc).await {
                    Ok(response) => {
                        let _ = tx.send(Action::AgentResponse(agent_name, response));
                    }
                    Err(e) => {
                        let _ = tx.send(Action::AgentResponseError(
                            agent_name,
                            e.to_string(),
                        ));
                    }
                }
            });
        } else {
            // Send message and get response (read-only conversation)
            let wd = work_dir.clone();
            let ctx_str = ctx.as_deref().map(|s| s.to_string());
            tokio::spawn(async move {
                match message::message_agent(
                    agent_name,
                    &msg,
                    &wd,
                    ctx_str.as_deref(),
                )
                .await
                {
                    Ok(response) => {
                        let _ = tx.send(Action::AgentResponse(agent_name, response));
                    }
                    Err(e) => {
                        let _ = tx.send(Action::AgentResponseError(
                            agent_name,
                            e.to_string(),
                        ));
                    }
                }
            });
        }
    }

    async fn process_task_creation(&mut self, input: String) {
        let title = input.trim().to_string();
        if title.is_empty() {
            return;
        }

        self.chat_messages.push(ChatMessage::user(format!("New task: {title}")));

        // Create a local work item immediately
        let local_item = WorkItem {
            id: format!("LOCAL-{}", self.items.len() + 1),
            source_id: None,
            title: title.clone(),
            description: None,
            status: Some("Todo".to_string()),
            priority: None,
            labels: Vec::new(),
            source: "Local".to_string(),
            team: None,
            url: None,
        };

        // Try to create in the active provider
        let tx = self.action_tx.clone();
        let mut created_in_provider = false;

        for provider in &self.providers {
            match provider.create_item(&title, None).await {
                Ok(Some(item)) => {
                    let _ = tx.send(Action::TaskCreated(item));
                    created_in_provider = true;
                    break;
                }
                Ok(None) => continue, // Provider doesn't support create
                Err(e) => {
                    let _ = tx.send(Action::TaskCreateError(format!(
                        "{}: {}",
                        provider.name(),
                        e
                    )));
                    // Fall through to add locally
                }
            }
        }

        if !created_in_provider {
            // Add as local item
            let _ = tx.send(Action::TaskCreated(local_item));
        }
    }

    async fn handle_key(&mut self, key: KeyAction) {
        match key {
            KeyAction::ActivateInput => {
                self.input_active = true;
                self.input_buffer.clear();
                self.input_cursor = 0;
            }
            // Also allow entering input mode by just typing a character
            // when not in a view that uses single-char shortcuts
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
            KeyAction::Left | KeyAction::Escape => match &self.view_mode {
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
                let status = if self.auto_mode { "AUTO" } else { "MANUAL" };
                self.flash_message = Some((format!("Mode: {status}"), Instant::now()));
                // Log mode change for all agents to see
                let _ = append_event(&new_event(
                    AgentName::ALL[0],
                    "mode-change",
                    None,
                    None,
                    Some(&format!("Switched to {status} mode")),
                ));
            }
            KeyAction::Refresh => {
                self.refresh_items().await;
            }
            KeyAction::ClearAgent => {
                if matches!(self.view_mode, ViewMode::Agents | ViewMode::AgentDetail(_)) {
                    let agent_name = match &self.view_mode {
                        ViewMode::AgentDetail(name) => *name,
                        _ => AgentName::ALL[self.selected_agent],
                    };
                    self.clear_agent(agent_name).await;
                }
            }
            KeyAction::ClearLogs => {
                if let ViewMode::AgentDetail(agent_name) = self.view_mode {
                    let _ = clear_events(agent_name);
                    self.agent_log_scroll = 0;
                    self.flash_message = Some((
                        format!("Cleared logs for {}", agent_name.display_name()),
                        Instant::now(),
                    ));
                    let _ = append_event(&new_event(
                        agent_name,
                        "logs-cleared",
                        None,
                        None,
                        Some("Activity log cleared"),
                    ));
                }
            }
            // Ignore unhandled keys in normal mode
            KeyAction::Char(_) | KeyAction::Backspace | KeyAction::Tab => {}
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

        // Auto-retry and auto-dispatch only in auto mode
        if self.auto_mode {
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

            // Auto-dispatch to free agents
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
                    if dispatch::dispatch(
                        free_agent,
                        &item,
                        &self.repo_root,
                        &mut self.store,
                        self.action_tx.clone(),
                    )
                    .await
                    .is_ok()
                    {
                        self.move_item_to_in_progress(&item).await;
                    }
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
                        self.move_item_to_in_progress(&item).await;
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

    async fn clear_agent(&mut self, agent_name: AgentName) {
        if let Some(agent) = self.store.get_agent(agent_name) {
            if agent.status == AgentStatus::Idle {
                self.flash_message = Some((
                    format!("{} is already idle", agent_name.display_name()),
                    Instant::now(),
                ));
                return;
            }

            // Kill process if running
            if let Some(pid) = agent.pid {
                unsafe {
                    libc::kill(pid as i32, libc::SIGTERM);
                }
            }

            let work_title = agent.work_item_title.clone();
            let work_id = agent.work_item_id.clone();

            // Remove item from dispatched set so it can be re-assigned
            if let Some(item_id) = &work_id {
                self.dispatched_item_ids.remove(item_id);
            }

            // Release the agent
            let _ = self.store.release(agent_name);
            let _ = append_event(&new_event(
                agent_name,
                "cleared",
                work_id.as_deref(),
                work_title.as_deref(),
                Some("Agent cleared by user"),
            ));

            self.flash_message = Some((
                format!("{} cleared", agent_name.display_name()),
                Instant::now(),
            ));
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

    async fn move_item_to_in_progress(&mut self, item: &WorkItem) {
        if let Some(source_id) = &item.source_id {
            for provider in &self.providers {
                if provider.name() == item.source {
                    if let Err(e) = provider.move_to_in_progress(source_id).await {
                        self.flash_message = Some((
                            format!("Failed to move {} to in-progress: {e}", item.id),
                            Instant::now(),
                        ));
                    }
                    break;
                }
            }
        }
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

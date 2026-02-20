use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::config::data_dir;
use crate::model::agent::{Agent, AgentName, AgentStatus};

#[derive(Debug, Serialize, Deserialize)]
struct StoreData {
    agents: HashMap<String, Agent>,
}

impl Default for StoreData {
    fn default() -> Self {
        let mut agents = HashMap::new();
        for name in AgentName::ALL {
            agents.insert(name.as_str().to_string(), Agent::new(name));
        }
        StoreData { agents }
    }
}

pub struct AgentStore {
    path: PathBuf,
    data: StoreData,
}

impl AgentStore {
    pub fn new() -> Result<Self> {
        let path = data_dir().join("agents.json");
        let data = if path.exists() {
            let contents = std::fs::read_to_string(&path)
                .with_context(|| format!("Failed to read {}", path.display()))?;
            serde_json::from_str(&contents).unwrap_or_default()
        } else {
            StoreData::default()
        };
        let mut store = Self { path, data };
        store.clean_stale_processes();
        Ok(store)
    }

    fn save(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(&self.data)?;
        std::fs::write(&self.path, json)?;
        Ok(())
    }

    fn clean_stale_processes(&mut self) {
        for agent in self.data.agents.values_mut() {
            if let Some(pid) = agent.pid {
                if !is_process_alive(pid) {
                    agent.status = AgentStatus::Error;
                    agent.error = Some("Process exited unexpectedly".into());
                    agent.pid = None;
                }
            }
        }
        let _ = self.save();
    }

    pub fn get_all(&self) -> Vec<&Agent> {
        AgentName::ALL
            .iter()
            .filter_map(|name| self.data.agents.get(name.as_str()))
            .collect()
    }

    pub fn get_agent(&self, name: AgentName) -> Option<&Agent> {
        self.data.agents.get(name.as_str())
    }

    pub fn update_agent(&mut self, name: AgentName, f: impl FnOnce(&mut Agent)) -> Result<()> {
        if let Some(agent) = self.data.agents.get_mut(name.as_str()) {
            f(agent);
            self.save()?;
        }
        Ok(())
    }

    pub fn next_free_agent(&self) -> Option<AgentName> {
        AgentName::ALL
            .iter()
            .find(|name| {
                self.data
                    .agents
                    .get(name.as_str())
                    .map(|a| a.status == AgentStatus::Idle)
                    .unwrap_or(false)
            })
            .copied()
    }

    pub fn mark_provisioning(
        &mut self,
        name: AgentName,
        work_item_id: &str,
        work_item_title: &str,
        branch: &str,
        worktree_path: &str,
    ) -> Result<()> {
        self.update_agent(name, |agent| {
            agent.status = AgentStatus::Provisioning;
            agent.work_item_id = Some(work_item_id.into());
            agent.work_item_title = Some(work_item_title.into());
            agent.branch = Some(branch.into());
            agent.worktree_path = Some(worktree_path.into());
            agent.started_at = Some(chrono::Utc::now().to_rfc3339());
            agent.error = None;
        })
    }

    pub fn mark_working(&mut self, name: AgentName, pid: u32) -> Result<()> {
        self.update_agent(name, |agent| {
            agent.status = AgentStatus::Working;
            agent.pid = Some(pid);
        })
    }

    pub fn mark_done(&mut self, name: AgentName) -> Result<()> {
        self.update_agent(name, |agent| {
            agent.status = AgentStatus::Done;
            agent.pid = None;
        })
    }

    pub fn mark_error(&mut self, name: AgentName, error: &str) -> Result<()> {
        self.update_agent(name, |agent| {
            agent.status = AgentStatus::Error;
            agent.error = Some(error.into());
            agent.pid = None;
        })
    }

    pub fn increment_retry(&mut self, name: AgentName) -> Result<u32> {
        let mut count = 0;
        self.update_agent(name, |agent| {
            agent.retry_count += 1;
            count = agent.retry_count;
        })?;
        Ok(count)
    }

    pub fn release(&mut self, name: AgentName) -> Result<()> {
        self.update_agent(name, |agent| {
            *agent = Agent::new(name);
        })
    }

    pub fn reload(&mut self) -> Result<()> {
        if self.path.exists() {
            let contents = std::fs::read_to_string(&self.path)?;
            self.data = serde_json::from_str(&contents).unwrap_or_default();
        }
        self.clean_stale_processes();
        Ok(())
    }
}

fn is_process_alive(pid: u32) -> bool {
    unsafe { libc::kill(pid as i32, 0) == 0 }
}

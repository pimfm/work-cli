use anyhow::{Context, Result};
use std::path::Path;
use std::process::Stdio;
use tokio::sync::mpsc;

use super::branch::{branch_name, worktree_path};
use super::claude_md::write_claude_md;
use super::claude_prompt::build_prompt;
use super::log::{append_event, new_event};
use super::store::AgentStore;
use crate::app::Action;
use crate::model::agent::AgentName;
use crate::model::work_item::WorkItem;

pub async fn dispatch(
    agent_name: AgentName,
    item: &WorkItem,
    repo_root: &str,
    store: &mut AgentStore,
    action_tx: mpsc::UnboundedSender<Action>,
) -> Result<()> {
    let branch = branch_name(agent_name, &item.id, &item.title);
    let wt_path = worktree_path(repo_root, agent_name);

    // Mark provisioning
    store.mark_provisioning(agent_name, &item.id, &item.title, &branch, &wt_path)?;
    let _ = append_event(&new_event(
        agent_name,
        "dispatched",
        Some(&item.id),
        Some(&item.title),
        None,
    ));

    // Git operations
    run_git(repo_root, &["fetch", "origin", "main"]).await?;

    // Clean up existing worktree
    let wt = Path::new(&wt_path);
    if wt.exists() {
        let _ = run_git(repo_root, &["worktree", "remove", &wt_path, "--force"]).await;
        if wt.exists() {
            tokio::fs::remove_dir_all(&wt_path).await.ok();
        }
    }
    let _ = run_git(repo_root, &["worktree", "prune"]).await;

    // Create branch (force if exists)
    if run_git(repo_root, &["branch", &branch, "origin/main"])
        .await
        .is_err()
    {
        run_git(repo_root, &["branch", "-f", &branch, "origin/main"]).await?;
    }

    // Create worktree
    run_git(repo_root, &["worktree", "add", &wt_path, &branch]).await?;

    // Write CLAUDE.md
    write_claude_md(Path::new(&wt_path), agent_name)?;

    // Build prompt
    let prompt = build_prompt(item, agent_name);

    // Set up log file
    let log_dir = crate::config::data_dir().join("logs");
    std::fs::create_dir_all(&log_dir)?;
    let log_file_path = log_dir.join(format!("agent-{}.log", agent_name.as_str()));
    let log_file = std::fs::File::create(&log_file_path)?;

    // Spawn claude process
    let child = tokio::process::Command::new("claude")
        .args(["-p", &prompt, "--dangerously-skip-permissions"])
        .current_dir(&wt_path)
        .stdin(Stdio::null())
        .stdout(Stdio::from(log_file.try_clone()?))
        .stderr(Stdio::from(log_file))
        .spawn()
        .context("Failed to spawn claude")?;

    let pid = child.id().unwrap_or(0);
    store.mark_working(agent_name, pid)?;
    let _ = append_event(&new_event(
        agent_name,
        "working",
        Some(&item.id),
        Some(&item.title),
        None,
    ));

    // Monitor process in background
    let item_id = item.id.clone();
    let item_title = item.title.clone();
    tokio::spawn(async move {
        let result = child.wait_with_output().await;
        match result {
            Ok(output) if output.status.success() => {
                let _ = append_event(&new_event(
                    agent_name,
                    "done",
                    Some(&item_id),
                    Some(&item_title),
                    None,
                ));
                let _ = action_tx.send(Action::AgentProcessExited(agent_name, true));
            }
            Ok(output) => {
                let msg = format!("Exit code: {}", output.status);
                let _ = append_event(&new_event(
                    agent_name,
                    "error",
                    Some(&item_id),
                    Some(&item_title),
                    Some(&msg),
                ));
                let _ = action_tx.send(Action::AgentProcessExited(agent_name, false));
            }
            Err(e) => {
                let msg = format!("Process error: {e}");
                let _ = append_event(&new_event(
                    agent_name,
                    "error",
                    Some(&item_id),
                    Some(&item_title),
                    Some(&msg),
                ));
                let _ = action_tx.send(Action::AgentProcessExited(agent_name, false));
            }
        }
    });

    Ok(())
}

async fn run_git(cwd: &str, args: &[&str]) -> Result<()> {
    let output = tokio::process::Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .await
        .with_context(|| format!("Failed to run git {}", args.join(" ")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git {} failed: {}", args.join(" "), stderr);
    }
    Ok(())
}

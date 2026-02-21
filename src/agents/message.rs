use anyhow::{Context, Result};
use std::process::Stdio;

use crate::model::agent::AgentName;
use crate::model::personality::personality;

/// Send a message to an agent and get a response.
/// Spawns a short-lived claude process with the message as prompt.
/// If the agent has a worktree, runs in that directory.
pub async fn message_agent(
    agent_name: AgentName,
    message: &str,
    work_dir: &str,
    task_context: Option<&str>,
) -> Result<String> {
    let p = personality(agent_name);

    let prompt = if let Some(ctx) = task_context {
        format!(
            r#"You are {name}, an agent in a team dashboard CLI called "work".
Your personality: {tagline} — {focus}

You are currently working on: {ctx}

The user has sent you this message:
{message}

Respond concisely and helpfully. If you need more information from the user, ask clearly.
If you're given feedback on your work, acknowledge it and explain what you'll do.
If asked a question, answer directly.
Keep responses under 200 words."#,
            name = agent_name.display_name(),
            tagline = p.tagline,
            focus = p.focus,
            ctx = ctx,
            message = message,
        )
    } else {
        format!(
            r#"You are {name}, an agent in a team dashboard CLI called "work".
Your personality: {tagline} — {focus}

The user has sent you this message:
{message}

Respond concisely and helpfully. If you need more information from the user, ask clearly.
Keep responses under 200 words."#,
            name = agent_name.display_name(),
            tagline = p.tagline,
            focus = p.focus,
            message = message,
        )
    };

    let output = tokio::process::Command::new("claude")
        .args(["-p", &prompt, "--output-format", "text"])
        .current_dir(work_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .context("Failed to spawn claude for agent message")?;

    if output.status.success() {
        let response = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(response)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Agent response failed: {stderr}")
    }
}

/// Build a prompt for an agent to apply feedback and make changes.
/// This spawns claude with --dangerously-skip-permissions so it can edit files.
pub async fn apply_feedback(
    agent_name: AgentName,
    feedback: &str,
    work_dir: &str,
    task_context: &str,
) -> Result<String> {
    let p = personality(agent_name);

    let prompt = format!(
        r#"You are {name}, an agent working on: {ctx}
Your personality: {tagline} — {focus}

The user has given you this feedback:
{feedback}

Apply this feedback to the codebase. Make the necessary changes, test them, commit and push.
After making changes, briefly summarize what you did."#,
        name = agent_name.display_name(),
        tagline = p.tagline,
        focus = p.focus,
        ctx = task_context,
        feedback = feedback,
    );

    let output = tokio::process::Command::new("claude")
        .args(["-p", &prompt, "--dangerously-skip-permissions", "--output-format", "text"])
        .current_dir(work_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .context("Failed to spawn claude for feedback application")?;

    if output.status.success() {
        let response = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(response)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Feedback application failed: {stderr}")
    }
}

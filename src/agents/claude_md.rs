use anyhow::Result;
use std::path::Path;

use crate::model::agent::AgentName;
use crate::model::personality::personality;

pub fn write_claude_md(worktree_path: &Path, agent_name: AgentName) -> Result<()> {
    let p = personality(agent_name);
    let traits = p.traits.join(", ");

    let content = format!(
        r#"# work pipeline

## Project Overview
A terminal dashboard CLI (`work`) that aggregates work items from Trello, Linear, Jira, and GitHub.
Built with Rust and Ratatui (terminal UI).

## Tech Stack
- **Language**: Rust (edition 2021)
- **UI**: Ratatui + Crossterm
- **Async**: Tokio
- **HTTP**: reqwest
- **Build**: cargo
- **Test**: cargo test

## Conventions
- Models in `src/model/`, providers in `src/providers/`, UI in `src/ui/`
- Agent infrastructure in `src/agents/`
- Use `anyhow` for error handling, `thiserror` for custom errors
- Use `serde` for serialization/deserialization
- Config stored at `~/.localpipeline/config.toml`
- Agent state stored at `~/.localpipeline/agents.json`
- Activity log at `~/.localpipeline/agent-activity.jsonl`

## Testing
- Run: `cargo test`

## Commit Format
- Short imperative subject line (e.g., "Add login validation")
- Reference the work item ID in the commit body

## Agent Identity
You are **{display}**, an autonomous agent working in a git worktree.
Your changes will be pushed directly to main.

### Personality: {tagline}
- **Focus**: {focus}
- **Traits**: {traits}
- **Working style**: {system_prompt}
"#,
        display = agent_name.display_name(),
        tagline = p.tagline,
        focus = p.focus,
        traits = traits,
        system_prompt = p.system_prompt,
    );

    std::fs::write(worktree_path.join("CLAUDE.md"), content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claude_md_includes_personality_for_all_agents() {
        let dir = tempfile::tempdir().unwrap();
        for name in AgentName::ALL {
            write_claude_md(dir.path(), name).unwrap();
            let content = std::fs::read_to_string(dir.path().join("CLAUDE.md")).unwrap();
            let p = personality(name);
            assert!(
                content.contains("**Focus**:"),
                "{name} CLAUDE.md missing Focus field"
            );
            assert!(
                content.contains("**Traits**:"),
                "{name} CLAUDE.md missing Traits field"
            );
            assert!(
                content.contains("**Working style**:"),
                "{name} CLAUDE.md missing Working style field"
            );
            assert!(
                content.contains(p.tagline),
                "{name} CLAUDE.md missing tagline"
            );
            assert!(
                content.contains(p.focus),
                "{name} CLAUDE.md missing focus content"
            );
            assert!(
                content.contains(p.system_prompt),
                "{name} CLAUDE.md missing system prompt"
            );
            assert!(
                content.contains(name.display_name()),
                "{name} CLAUDE.md missing display name"
            );
        }
    }

    #[test]
    fn claude_md_includes_project_conventions() {
        let dir = tempfile::tempdir().unwrap();
        write_claude_md(dir.path(), AgentName::Ember).unwrap();
        let content = std::fs::read_to_string(dir.path().join("CLAUDE.md")).unwrap();
        assert!(content.contains("src/agents/"), "missing agents convention");
        assert!(
            content.contains("config.toml"),
            "missing config path convention"
        );
        assert!(
            content.contains("agents.json"),
            "missing agent state convention"
        );
        assert!(content.contains("thiserror"), "missing thiserror convention");
    }
}

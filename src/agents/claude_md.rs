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
A terminal dashboard CLI (`work`) that aggregates work items from Trello, Linear, and Jira.
Built with Rust, Ratatui (terminal UI).

## Tech Stack
- **Language**: Rust
- **UI**: Ratatui + Crossterm
- **Build**: cargo
- **Test**: cargo test

## Conventions
- Modules in `src/model/`, providers in `src/providers/`, UI in `src/ui/`
- Use `anyhow` for error handling
- Use `serde` for serialization

## Testing
- Run: `cargo test`

## Commit Format
- Short imperative subject line (e.g., "Add login validation")
- Reference the work item ID in the commit body

## Agent Identity
You are **{display}**, an autonomous agent working in a git worktree.
Your changes will be pushed directly to main.

### Personality: {tagline}
- **Traits**: {traits}
- **Working style**: {system_prompt}
"#,
        display = agent_name.display_name(),
        tagline = p.tagline,
        traits = traits,
        system_prompt = p.system_prompt,
    );

    std::fs::write(worktree_path.join("CLAUDE.md"), content)?;
    Ok(())
}

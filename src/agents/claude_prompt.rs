use crate::model::agent::AgentName;
use crate::model::personality::personality;
use crate::model::work_item::WorkItem;

pub fn build_prompt(item: &WorkItem, agent_name: AgentName) -> String {
    let p = personality(agent_name);
    let labels = if item.labels.is_empty() {
        "none".to_string()
    } else {
        item.labels.join(", ")
    };

    format!(
        r#"You are agent "{agent}" working on the following task. Your personality: {tagline}.

# {title}
- ID: {id}
- Source: {source}
- URL: {url}
- Priority: {priority}
- Labels: {labels}
- Status: {status}
- Team: {team}

## Description
{description}

## Instructions
1. Read CLAUDE.md in the project root for conventions and context.
2. Implement the task described above.
3. Write tests for your changes.
4. Run `cargo test` and ensure all tests pass.
5. Commit your changes with a message referencing {id}.
6. Run `git fetch origin main && git rebase origin/main`.
7. Run `git push origin HEAD:main`.

Work autonomously. Do not ask for clarification — make reasonable decisions.

## Personality: {tagline}
- Traits: {traits}
- Working style: {system_prompt}"#,
        agent = agent_name.display_name(),
        tagline = p.tagline,
        title = item.title,
        id = item.id,
        source = item.source,
        url = item.url.as_deref().unwrap_or("n/a"),
        priority = item.priority.as_deref().unwrap_or("n/a"),
        labels = labels,
        status = item.status.as_deref().unwrap_or("n/a"),
        team = item.team.as_deref().unwrap_or("n/a"),
        description = item.description.as_deref().unwrap_or("No description provided."),
        traits = p.traits.join(", "),
        system_prompt = p.system_prompt,
    )
}

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
- Focus: {focus}
- Traits: {traits}
- Working style: {system_prompt}"#,
        agent = agent_name.display_name(),
        tagline = p.tagline,
        focus = p.focus,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn test_item() -> WorkItem {
        WorkItem {
            id: "TEST-1".to_string(),
            title: "Test task".to_string(),
            description: Some("A test description".to_string()),
            status: Some("Todo".to_string()),
            priority: None,
            labels: vec!["bug".to_string()],
            source: "trello".to_string(),
            team: Some("TestTeam".to_string()),
            url: Some("https://example.com".to_string()),
        }
    }

    #[test]
    fn prompt_includes_focus_for_all_agents() {
        let item = test_item();
        for name in AgentName::ALL {
            let prompt = build_prompt(&item, name);
            let p = personality(name);
            assert!(
                prompt.contains("Focus:"),
                "{name} prompt missing Focus field"
            );
            assert!(
                prompt.contains(p.focus),
                "{name} prompt missing focus content"
            );
        }
    }

    #[test]
    fn prompt_includes_personality_section() {
        let item = test_item();
        let prompt = build_prompt(&item, AgentName::Ember);
        let p = personality(AgentName::Ember);
        assert!(prompt.contains(&format!("Personality: {}", p.tagline)));
        assert!(prompt.contains("Traits:"));
        assert!(prompt.contains("Working style:"));
        assert!(prompt.contains(r#"You are agent "Ember""#));
    }
}

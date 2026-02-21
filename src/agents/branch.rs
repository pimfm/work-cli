use crate::model::agent::AgentName;

pub fn slugify(title: &str) -> String {
    let slug: String = title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    let trimmed = slug.trim_matches('-').to_string();
    trimmed.chars().take(40).collect()
}

pub fn branch_name(agent: AgentName, item_id: &str, title: &str) -> String {
    let slug = slugify(title);
    let short_id = &item_id[..item_id.len().min(8)];
    format!("agent/{}/{}-{}", agent.as_str(), short_id, slug)
}

pub fn worktree_path(repo_root: &str, agent: AgentName) -> String {
    let mut parts: Vec<&str> = repo_root.rsplitn(2, '/').collect();
    parts.reverse();
    if parts.len() == 2 {
        format!("{}/agent-{}", parts[0], agent.as_str())
    } else {
        format!("{}/agent-{}", repo_root, agent.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Add login validation"), "add-login-validation");
        assert_eq!(slugify("Fix bug #42!"), "fix-bug--42");
    }

    #[test]
    fn test_branch_name() {
        let name = branch_name(AgentName::Ember, "LIN-42", "Add login");
        assert_eq!(name, "agent/ember/LIN-42-add-login");
    }

    #[test]
    fn test_worktree_path() {
        let path = worktree_path("/Users/pim/fm/workflow/main", AgentName::Ember);
        assert_eq!(path, "/Users/pim/fm/workflow/agent-ember");
    }
}

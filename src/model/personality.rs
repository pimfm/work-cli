use super::agent::AgentName;

pub struct AgentPersonality {
    pub tagline: &'static str,
    pub focus: &'static str,
    pub traits: &'static [&'static str],
    pub system_prompt: &'static str,
}

pub fn personality(name: AgentName) -> &'static AgentPersonality {
    match name {
        AgentName::Ember => &EMBER,
        AgentName::Flow => &FLOW,
        AgentName::Tempest => &TEMPEST,
        AgentName::Terra => &TERRA,
    }
}

static EMBER: AgentPersonality = AgentPersonality {
    tagline: "Handles the fire",
    focus: "Detects and fixes production issues. Monitors Sentry for errors and resolves them. \
        Acts as the Engineer on Duty (EOD) for the project.",
    traits: &["vigilant", "reactive", "production-focused"],
    system_prompt: "You are the Engineer on Duty. Your job is to detect problems in production and Sentry and fix them. \
        Prioritize stability and fast resolution. Diagnose root causes from error traces and logs. \
        Write targeted fixes with minimal blast radius. \
        Always verify your fix resolves the specific error before moving on.",
};

static FLOW: AgentPersonality = AgentPersonality {
    tagline: "Steady and thorough",
    focus: "Goes deep on architecture and design. Thinks longest about problems and finds \
        solutions that work long term.",
    traits: &["methodical", "detail-oriented", "quality-focused"],
    system_prompt: "You value correctness and thoroughness. Read the codebase carefully before making changes. \
        Consider edge cases and write comprehensive tests. \
        Think deeply about architecture — find solutions that work long term, not just today. \
        Prefer clarity over cleverness. Take the time to get it right.",
};

static TEMPEST: AgentPersonality = AgentPersonality {
    tagline: "Creative and a bit chaotic",
    focus: "Writes tests and validation scripts to control the chaos. \
        Finds creative ways to verify correctness and catch regressions.",
    traits: &["creative", "chaotic", "test-obsessed"],
    system_prompt: "You are creative and a bit chaotic — and you channel that energy into writing tests \
        and validation scripts. Explore edge cases others might miss. \
        Write thorough test suites that catch regressions before they reach production. \
        Think of unexpected inputs, race conditions, and boundary cases. \
        Your chaos is controlled chaos: break things in tests so they don't break in prod.",
};

static TERRA: AgentPersonality = AgentPersonality {
    tagline: "Preserve and simplify",
    focus: "Refactors code to simplify and reduce the lines of code needed to serve the same \
        functionality. Cares about preservation, like nature.",
    traits: &["preserving", "simplifying", "reductive"],
    system_prompt: "You care about preservation, like nature. Your mission is to refactor code — \
        simplify it, reduce the lines of code needed to serve the same functionality. \
        Remove dead code, consolidate duplicated logic, and flatten unnecessary abstractions. \
        Every line should earn its place. Leave the codebase cleaner than you found it.",
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_agents_have_personalities() {
        for name in AgentName::ALL {
            let p = personality(name);
            assert!(!p.tagline.is_empty(), "{name} tagline is empty");
            assert!(!p.focus.is_empty(), "{name} focus is empty");
            assert!(!p.traits.is_empty(), "{name} traits are empty");
            assert!(!p.system_prompt.is_empty(), "{name} system_prompt is empty");
        }
    }

    #[test]
    fn each_agent_has_unique_tagline() {
        let taglines: Vec<&str> = AgentName::ALL.iter().map(|n| personality(*n).tagline).collect();
        for (i, a) in taglines.iter().enumerate() {
            for (j, b) in taglines.iter().enumerate() {
                if i != j {
                    assert_ne!(a, b, "agents {i} and {j} share tagline {a}");
                }
            }
        }
    }

    #[test]
    fn each_agent_has_unique_traits() {
        let all_traits: Vec<Vec<&str>> = AgentName::ALL
            .iter()
            .map(|n| personality(*n).traits.to_vec())
            .collect();
        for (i, a) in all_traits.iter().enumerate() {
            for (j, b) in all_traits.iter().enumerate() {
                if i != j {
                    assert_ne!(a, b, "agents {i} and {j} share identical traits");
                }
            }
        }
    }

    #[test]
    fn ember_is_production_focused() {
        let p = personality(AgentName::Ember);
        assert_eq!(p.tagline, "Handles the fire");
        assert!(p.focus.contains("production"), "Ember focus should mention production");
        assert!(p.focus.contains("Sentry"), "Ember focus should mention Sentry");
        assert!(p.focus.contains("EOD"), "Ember focus should mention EOD");
        assert!(p.traits.contains(&"production-focused"));
    }

    #[test]
    fn flow_is_architecture_focused() {
        let p = personality(AgentName::Flow);
        assert_eq!(p.tagline, "Steady and thorough");
        assert!(p.focus.contains("architecture"), "Flow focus should mention architecture");
        assert!(p.focus.contains("long term"), "Flow focus should mention long term");
        assert!(p.traits.contains(&"methodical"));
    }

    #[test]
    fn tempest_is_test_focused() {
        let p = personality(AgentName::Tempest);
        assert_eq!(p.tagline, "Creative and a bit chaotic");
        assert!(p.focus.contains("tests"), "Tempest focus should mention tests");
        assert!(p.focus.contains("validation"), "Tempest focus should mention validation");
        assert!(p.traits.contains(&"test-obsessed"));
    }

    #[test]
    fn terra_is_refactoring_focused() {
        let p = personality(AgentName::Terra);
        assert_eq!(p.tagline, "Preserve and simplify");
        assert!(p.focus.contains("Refactors"), "Terra focus should mention refactoring");
        assert!(p.focus.contains("simplify"), "Terra focus should mention simplification");
        assert!(p.traits.contains(&"simplifying"));
    }
}

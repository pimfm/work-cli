use anyhow::{bail, Result};

use crate::config;
use crate::providers;

/// Parse CLI args for `work add` and create the task in the mapped provider.
pub async fn handle_add(args: &[String]) -> Result<()> {
    let (title, description) = parse_add_args(args)?;

    let config = config::load_config()?;
    let mut providers = providers::create_providers(&config);

    if providers.is_empty() {
        bail!("No providers configured. Add credentials to ~/.localpipeline/config.toml");
    }

    // Determine current project directory and apply board mapping
    let project_dir = std::env::current_dir()
        .ok()
        .and_then(|p| p.canonicalize().ok())
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let mappings = config::load_board_mappings();
    let mapping = mappings.get(&project_dir);

    if let Some(mapping) = mapping {
        // Apply board filter so providers know which board/project to target
        for provider in &mut providers {
            if provider.name() == mapping.source {
                provider.set_board_filter(mapping.board_id.clone());
            }
        }
    }

    // Try the mapped provider first, then fall back to others
    let desc = description.as_deref();
    let mut created = false;
    let mut last_error = None;

    // Sort providers: mapped provider first
    let provider_order: Vec<usize> = if let Some(mapping) = mapping {
        let mut order: Vec<usize> = Vec::new();
        // Mapped provider first
        for (i, p) in providers.iter().enumerate() {
            if p.name() == mapping.source {
                order.push(i);
            }
        }
        // Then the rest
        for (i, p) in providers.iter().enumerate() {
            if p.name() != mapping.source {
                order.push(i);
            }
        }
        order
    } else {
        (0..providers.len()).collect()
    };

    for idx in provider_order {
        let provider = &providers[idx];
        match provider.create_item(&title, desc).await {
            Ok(Some(item)) => {
                println!("Created in {}: {} — {}", item.source, item.id, item.title);
                if let Some(url) = &item.url {
                    println!("  {url}");
                }
                created = true;
                break;
            }
            Ok(None) => continue, // Provider doesn't support create
            Err(e) => {
                last_error = Some(format!("{}: {e}", provider.name()));
                continue;
            }
        }
    }

    if !created {
        if let Some(err) = last_error {
            bail!("Failed to create task: {err}");
        } else {
            bail!("No provider supports task creation. Configure Linear, Trello, or GitHub in ~/.localpipeline/config.toml");
        }
    }

    Ok(())
}

/// Parse `work add` arguments into (title, optional description).
///
/// Supported forms:
///   work add "My task title"
///   work add My task title
///   work add "My task" -d "The description"
///   work add "My task" --desc "The description"
pub fn parse_add_args(args: &[String]) -> Result<(String, Option<String>)> {
    if args.is_empty() {
        bail!("Usage: work add <title> [-d <description>]\n\nExamples:\n  work add \"Fix the login bug\"\n  work add \"Fix the login bug\" -d \"Users can't log in with SSO\"");
    }

    let mut title_parts: Vec<String> = Vec::new();
    let mut description: Option<String> = None;
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "-d" | "--desc" | "--description" => {
                i += 1;
                if i < args.len() {
                    description = Some(args[i].clone());
                } else {
                    bail!("Missing value for -d/--desc flag");
                }
            }
            _ => {
                title_parts.push(args[i].clone());
            }
        }
        i += 1;
    }

    let title = title_parts.join(" ");
    if title.is_empty() {
        bail!("Task title cannot be empty");
    }

    Ok((title, description))
}

pub fn print_help() {
    println!("work — terminal dashboard for work items\n");
    println!("USAGE:");
    println!("  work              Launch the TUI dashboard");
    println!("  work add <title>  Create a new task and sync to your project management tool");
    println!();
    println!("ADD OPTIONS:");
    println!("  -d, --desc <text>  Set a description for the task");
    println!();
    println!("EXAMPLES:");
    println!("  work add \"Fix the login bug\"");
    println!("  work add \"Fix login\" -d \"Users can't log in with SSO\"");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(strs: &[&str]) -> Vec<String> {
        strs.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn parse_simple_title() {
        let (title, desc) = parse_add_args(&args(&["Fix the login bug"])).unwrap();
        assert_eq!(title, "Fix the login bug");
        assert_eq!(desc, None);
    }

    #[test]
    fn parse_multi_word_title() {
        let (title, desc) = parse_add_args(&args(&["Fix", "the", "login", "bug"])).unwrap();
        assert_eq!(title, "Fix the login bug");
        assert_eq!(desc, None);
    }

    #[test]
    fn parse_title_with_description_short_flag() {
        let (title, desc) =
            parse_add_args(&args(&["Fix login", "-d", "Users can't log in"])).unwrap();
        assert_eq!(title, "Fix login");
        assert_eq!(desc, Some("Users can't log in".to_string()));
    }

    #[test]
    fn parse_title_with_description_long_flag() {
        let (title, desc) =
            parse_add_args(&args(&["Fix login", "--desc", "SSO is broken"])).unwrap();
        assert_eq!(title, "Fix login");
        assert_eq!(desc, Some("SSO is broken".to_string()));
    }

    #[test]
    fn parse_title_with_description_full_flag() {
        let (title, desc) =
            parse_add_args(&args(&["Fix login", "--description", "SSO is broken"])).unwrap();
        assert_eq!(title, "Fix login");
        assert_eq!(desc, Some("SSO is broken".to_string()));
    }

    #[test]
    fn parse_empty_args_fails() {
        let result = parse_add_args(&args(&[]));
        assert!(result.is_err());
    }

    #[test]
    fn parse_only_flag_no_title_fails() {
        let result = parse_add_args(&args(&["-d", "some description"]));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }

    #[test]
    fn parse_missing_desc_value_fails() {
        let result = parse_add_args(&args(&["My task", "-d"]));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing value"));
    }

    #[test]
    fn parse_desc_between_title_words() {
        // Weird but should work: title words around the flag
        let (title, desc) =
            parse_add_args(&args(&["Fix", "-d", "urgent fix needed", "login", "bug"])).unwrap();
        assert_eq!(title, "Fix login bug");
        assert_eq!(desc, Some("urgent fix needed".to_string()));
    }

    #[test]
    fn parse_preserves_special_characters() {
        let (title, desc) = parse_add_args(&args(&[
            "Add @mention support & <html> escaping",
            "-d",
            "Handle edge cases: <script>, '\"quotes\"', and &&",
        ]))
        .unwrap();
        assert_eq!(title, "Add @mention support & <html> escaping");
        assert_eq!(
            desc,
            Some("Handle edge cases: <script>, '\"quotes\"', and &&".to_string())
        );
    }

    #[test]
    fn parse_unicode_title() {
        let (title, _desc) = parse_add_args(&args(&["修复登录 bug 🐛"])).unwrap();
        assert_eq!(title, "修复登录 bug 🐛");
    }
}

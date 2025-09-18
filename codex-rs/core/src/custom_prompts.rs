use codex_protocol::custom_prompts::CustomPrompt;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use tokio::fs;

/// Return the default prompts directory: `$CODEX_HOME/prompts`.
/// If `CODEX_HOME` cannot be resolved, returns `None`.
pub fn default_prompts_dir() -> Option<PathBuf> {
    crate::config::find_codex_home()
        .ok()
        .map(|home| home.join("prompts"))
}

/// Discover prompt files in the given directory, returning entries sorted by name.
/// Non-files are ignored. If the directory does not exist or cannot be read, returns empty.
pub async fn discover_prompts_in(dir: &Path) -> Vec<CustomPrompt> {
    discover_prompts_in_excluding(dir, &HashSet::new()).await
}

/// Discover prompt files in the given directory, excluding any with names in `exclude`.
/// Returns entries sorted by name. Non-files are ignored. Missing/unreadable dir yields empty.
pub async fn discover_prompts_in_excluding(
    dir: &Path,
    exclude: &HashSet<String>,
) -> Vec<CustomPrompt> {
    let mut out: Vec<CustomPrompt> = Vec::new();
    let mut entries = match fs::read_dir(dir).await {
        Ok(entries) => entries,
        Err(_) => return out,
    };

    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        let is_file = entry
            .file_type()
            .await
            .map(|ft| ft.is_file() || ft.is_symlink())
            .unwrap_or(false);
        if !is_file {
            continue;
        }
        // Only include Markdown files with a .md extension.
        let is_md = path
            .extension()
            .and_then(|s| s.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("md"))
            .unwrap_or(false);
        if !is_md {
            continue;
        }
        let Some(name) = path
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
        else {
            continue;
        };
        if exclude.contains(&name) {
            continue;
        }
        let raw_content = match fs::read_to_string(&path).await {
            Ok(s) => s,
            Err(_) => continue,
        };
        let (description, argument_hint, content) = parse_frontmatter(&raw_content);
        out.push(CustomPrompt {
            name,
            path,
            content,
            category: None,
            argument_hint,
            description,
            template_args: None,
            template_syntax: None,
        });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

/// Discover prompts including subdirectories with namespace support
pub async fn discover_prompts_with_directories(base_dir: &Path) -> Vec<CustomPrompt> {
    let mut prompts = Vec::new();

    // Scan root level prompts (flat commands)
    let root_prompts = discover_prompts_in_excluding(base_dir, &HashSet::new()).await;
    for mut prompt in root_prompts {
        prompt.category = None; // Root level prompts have no category
        prompts.push(prompt);
    }

    // Scan subdirectories (namespaced commands)
    if let Ok(mut entries) = fs::read_dir(base_dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            if entry
                .file_type()
                .await
                .map(|ft| ft.is_dir() || ft.is_symlink())
                .unwrap_or(false)
            {
                let dir_name = entry.file_name().to_string_lossy().to_string();
                let subdir_prompts =
                    discover_prompts_in_excluding(&entry.path(), &HashSet::new()).await;

                for mut prompt in subdir_prompts {
                    prompt.name = format!("{}:{}", dir_name, prompt.name);
                    prompt.category = Some(dir_name.clone());
                    prompts.push(prompt);
                }
            }
        }
    }

    prompts.sort_by(|a, b| a.name.cmp(&b.name));
    prompts
}

/// Discover prompts from both global and project-level directories
pub async fn discover_prompts_with_project_support(
    global_dir: &Path,
    project_cwd: &Path,
) -> Vec<CustomPrompt> {
    let mut prompts = Vec::new();
    let mut exclude = HashSet::new();

    // 1. Global prompts (existing behavior)
    prompts.extend(discover_prompts_in_excluding(global_dir, &exclude).await);

    // Build exclusion set from global prompts
    for prompt in &prompts {
        exclude.insert(prompt.name.clone());
    }

    // 2. Project prompts (new behavior)
    if let Some(git_root) = crate::git_info::get_git_repo_root(project_cwd) {
        let project_prompt_dir = git_root.join(".codex/prompts");
        let project_prompts = discover_prompts_with_directories(&project_prompt_dir).await;

        // Project prompts override global ones (later sources take precedence)
        prompts.retain(|p| !project_prompts.iter().any(|pp| pp.name == p.name));
        prompts.extend(project_prompts);
    }

    prompts.sort_by(|a, b| a.name.cmp(&b.name));
    prompts
}

/// Parse frontmatter from a markdown file content
/// Returns (description, argument_hint, content_without_frontmatter)
fn parse_frontmatter(content: &str) -> (Option<String>, Option<String>, String) {
    // Check if content starts with frontmatter delimiter (handle both Unix and Windows line endings)
    let skip_bytes = if content.starts_with("---\n") {
        4
    } else if content.starts_with("---\r\n") {
        5
    } else {
        return (None, None, content.to_string());
    };

    // Find the closing frontmatter delimiter (handle both line ending types)
    let content_after_start = &content[skip_bytes..];
    let (closing_delimiter, delimiter_len) = if content_after_start.contains("\n---\n") {
        ("\n---\n", 5)
    } else if content_after_start.contains("\r\n---\r\n") {
        ("\r\n---\r\n", 7)
    } else {
        return (None, None, content.to_string());
    };

    if let Some(end_pos) = content_after_start.find(closing_delimiter) {
        let frontmatter = &content_after_start[..end_pos];
        let body_content = &content_after_start[end_pos + delimiter_len..];

        // Trim leading newline/CRLF from body content if present
        let body_content = body_content
            .strip_prefix("\r\n")
            .or_else(|| body_content.strip_prefix('\n'))
            .unwrap_or(body_content);

        // Helper function to parse a field value from frontmatter
        let parse_field = |field_name: &str| -> Option<String> {
            frontmatter
                .lines()
                .find(|line| line.trim_start().starts_with(&format!("{field_name}:")))
                .and_then(|line| {
                    let colon_pos = line.find(':')?;
                    let value = line[colon_pos + 1..].trim();
                    // Handle quoted values
                    let value = if (value.starts_with('"') && value.ends_with('"'))
                        || (value.starts_with('\'') && value.ends_with('\''))
                    {
                        &value[1..value.len() - 1]
                    } else {
                        value
                    };
                    if value.is_empty() {
                        None
                    } else {
                        Some(value.to_string())
                    }
                })
        };

        // Parse description and argument-hint from frontmatter
        let description = parse_field("description");
        let argument_hint = parse_field("argument-hint");

        // Return only the body content (frontmatter completely removed)
        (description, argument_hint, body_content.to_string())
    } else {
        // No closing delimiter found, treat as regular content
        (None, None, content.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[tokio::test]
    async fn empty_when_dir_missing() {
        let tmp = tempdir().expect("create TempDir");
        let missing = tmp.path().join("nope");
        let found = discover_prompts_in(&missing).await;
        assert!(found.is_empty());
    }

    #[tokio::test]
    async fn discovers_and_sorts_files() {
        let tmp = tempdir().expect("create TempDir");
        let dir = tmp.path();
        fs::write(dir.join("b.md"), b"b").unwrap();
        fs::write(dir.join("a.md"), b"a").unwrap();
        fs::create_dir(dir.join("subdir")).unwrap();
        let found = discover_prompts_in(dir).await;
        let names: Vec<String> = found.into_iter().map(|e| e.name).collect();
        assert_eq!(names, vec!["a", "b"]);
    }

    #[tokio::test]
    async fn excludes_builtins() {
        let tmp = tempdir().expect("create TempDir");
        let dir = tmp.path();
        fs::write(dir.join("init.md"), b"ignored").unwrap();
        fs::write(dir.join("foo.md"), b"ok").unwrap();
        let mut exclude = HashSet::new();
        exclude.insert("init".to_string());
        let found = discover_prompts_in_excluding(dir, &exclude).await;
        let names: Vec<String> = found.into_iter().map(|e| e.name).collect();
        assert_eq!(names, vec!["foo"]);
    }

    #[tokio::test]
    async fn skips_non_utf8_files() {
        let tmp = tempdir().expect("create TempDir");
        let dir = tmp.path();
        // Valid UTF-8 file
        fs::write(dir.join("good.md"), b"hello").unwrap();
        // Invalid UTF-8 content in .md file (e.g., lone 0xFF byte)
        fs::write(dir.join("bad.md"), vec![0xFF, 0xFE, b'\n']).unwrap();
        let found = discover_prompts_in(dir).await;
        let names: Vec<String> = found.into_iter().map(|e| e.name).collect();
        assert_eq!(names, vec!["good"]);
    }

    #[test]
    fn test_parse_frontmatter_with_argument_hint() {
        let content = "---\ndescription: \"test\"\nargument-hint: <subject>\n---\n\nHello world";
        let (description, argument_hint, filtered_content) = parse_frontmatter(content);
        assert_eq!(description, Some("test".to_string()));
        assert_eq!(argument_hint, Some("<subject>".to_string()));
        assert_eq!(filtered_content, "Hello world");
    }

    #[test]
    fn test_parse_frontmatter_filters_out_unwanted_fields() {
        let content = "---\ndescription: \"test\"\nargument-hint: <subject>\nallowed-tools: Read\nmodel: claude-opus\n---\n\nHello world";
        let (description, argument_hint, filtered_content) = parse_frontmatter(content);
        assert_eq!(description, Some("test".to_string()));
        assert_eq!(argument_hint, Some("<subject>".to_string()));
        assert_eq!(filtered_content, "Hello world");
    }

    #[test]
    fn test_parse_frontmatter_no_argument_hint() {
        let content = "---\ndescription: \"test\"\n---\n\nHello world";
        let (description, argument_hint, filtered_content) = parse_frontmatter(content);
        assert_eq!(description, Some("test".to_string()));
        assert_eq!(argument_hint, None);
        assert_eq!(filtered_content, "Hello world");
    }

    #[test]
    fn test_parse_frontmatter_only_unwanted_fields() {
        let content = "---\nallowed-tools: Read\nmodel: claude-opus\n---\n\nHello world";
        let (description, argument_hint, filtered_content) = parse_frontmatter(content);
        assert_eq!(description, None);
        assert_eq!(argument_hint, None);
        assert_eq!(filtered_content, "Hello world");
    }

    #[test]
    fn test_parse_frontmatter_no_frontmatter() {
        let content = "Hello world";
        let (description, argument_hint, filtered_content) = parse_frontmatter(content);
        assert_eq!(description, None);
        assert_eq!(argument_hint, None);
        assert_eq!(filtered_content, "Hello world");
    }

    #[test]
    fn test_parse_frontmatter_incomplete() {
        let content = "---\ndescription: \"test\"\nno closing delimiter";
        let (description, argument_hint, filtered_content) = parse_frontmatter(content);
        assert_eq!(description, None);
        assert_eq!(argument_hint, None);
        assert_eq!(filtered_content, content);
    }

    #[test]
    fn test_parse_frontmatter_real_world_example() {
        let content = r#"---
description: "Analyze plans for potential blocking issues by examining codebase, dependencies, and related documents"
argument-hint: <plan_file(s)_or_NNNN>
allowed-tools: Read(./**), Task, Bash(git:*, find:*, grep:*), Glob, Grep
model: claude-opus-4-1
---

You are tasked with analyzing implementation plans for potential blocking issues by examining the codebase, technical dependencies, related summaries, and research documents.

**Plan input provided:** $1"#;

        let (description, argument_hint, filtered_content) = parse_frontmatter(content);

        assert_eq!(description, Some("Analyze plans for potential blocking issues by examining codebase, dependencies, and related documents".to_string()));
        assert_eq!(argument_hint, Some("<plan_file(s)_or_NNNN>".to_string()));
        assert_eq!(filtered_content, "You are tasked with analyzing implementation plans for potential blocking issues by examining the codebase, technical dependencies, related summaries, and research documents.\n\n**Plan input provided:** $1");
    }

    #[test]
    fn test_parse_frontmatter_windows_line_endings() {
        let content = "---\r\ndescription: \"Analyze plans for potential blocking issues\"\r\nargument-hint: <plan_file>\r\nmodel: claude-opus-4-1\r\n---\r\n\r\nYou are tasked with analyzing implementation plans.\r\n\r\n**Plan input provided:** $1";

        let (description, argument_hint, filtered_content) = parse_frontmatter(content);

        assert_eq!(description, Some("Analyze plans for potential blocking issues".to_string()));
        assert_eq!(argument_hint, Some("<plan_file>".to_string()));
        assert_eq!(filtered_content, "You are tasked with analyzing implementation plans.\r\n\r\n**Plan input provided:** $1");
    }
}

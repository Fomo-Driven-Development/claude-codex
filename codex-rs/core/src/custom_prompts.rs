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
        let content = match fs::read_to_string(&path).await {
            Ok(s) => s,
            Err(_) => continue,
        };
        out.push(CustomPrompt {
            name,
            path,
            content,
            category: None,
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
}

---
date: 2025-09-17T23:16:53-05:00
git_commit: c9505488a120299b339814d73f57817ee79e114f
branch: main
repository: codex
topic: "How can we create a custom project level config which would allow for project level prompts similar to claude codes project level slash commands?"
tags: [research, configuration, slash-commands, prompts, project-config, custom-prompts, toml, automatic-discovery, directory-commands]
status: complete
last_updated: 2025-09-17
last_updated_note: "Added follow-up research for automatic .codex/prompts discovery and /directory:command syntax"
---

# Research: Creating Custom Project-Level Configuration for Project-Specific Prompts

**Date**: 2025-09-17T23:16:53-05:00
**Git Commit**: c9505488a120299b339814d73f57817ee79e114f
**Branch**: main
**Repository**: codex

## Research Question

How can we create a custom project level config which would allow for project level prompts similar to Claude Code's project level slash commands?

## Summary

Based on comprehensive research of the Codex CLI codebase, creating custom project-level configuration for project-specific prompts is achievable by extending the existing configuration system. The current implementation provides all necessary building blocks: TOML-based configuration with Serde validation, profile system, CLI overrides, and custom prompt discovery. The recommended approach involves extending the configuration schema to support project-specific prompt directories and leveraging the existing custom prompt system.

## Detailed Findings

### Current Slash Command Architecture

Claude Code implements a two-tier slash command system:

- **Built-in Commands**: Defined in `codex-rs/tui/src/slash_command.rs` as static enum variants (Model, Approvals, New, Init, etc.)
- **Custom Prompts**: Markdown files discovered from `$CODEX_HOME/prompts/` directory via `codex-rs/core/src/custom_prompts.rs`

The system combines both types in the command popup (`codex-rs/tui/src/bottom_pane/command_popup.rs`), with built-ins taking precedence for name conflicts.

**Key Implementation Details**:
- Custom prompts are loaded at session start from `~/.codex/prompts/*.md`
- Filename (minus `.md`) becomes the slash command name
- File content is sent as the message when command is selected
- No variable substitution or templating currently supported

### Configuration Extension Patterns

The codebase demonstrates several robust patterns for extending configuration:

**1. Profile-Based Extension** (`codex-rs/core/src/config_profile.rs`):
```rust
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct ConfigProfile {
    pub experimental_instructions_file: Option<PathBuf>,
    // ... other profile fields
}
```

**2. Dynamic CLI Overrides** (`codex-rs/common/src/config_override.rs`):
- Supports dotted-path notation: `-c prompts.directories='["./team-prompts"]'`
- TOML/JSON value parsing with string fallback
- Deep nested object creation capabilities

**3. HashMap-Based Extension Points** (`codex-rs/core/src/config.rs`):
```rust
#[serde(default)]
pub mcp_servers: HashMap<String, McpServerConfig>,
#[serde(default)]
pub model_providers: HashMap<String, ModelProviderInfo>,
```

### Project-Level Configuration Patterns

Current project-level configuration is minimal:

**Trust-Based Project Settings** (`codex-rs/core/src/config.rs:734-736`):
```toml
[projects."/path/to/project"]
trust_level = "trusted"
```

**Project Documentation Discovery** (`codex-rs/core/src/project_doc.rs`):
- Discovers `AGENTS.md` files from Git root to current directory
- No configuration file discovery for project-specific settings
- All configuration flows through global `~/.codex/config.toml`

### Prompt Handling and Template Systems

**Current Limitations**:
- No variable substitution (no `${variable}` or `{{variable}}` replacement)
- No conditional templates or inheritance
- Static file loading without context injection
- Simple markdown files without structured metadata

**Extension Points**:
- Base instructions override via `experimental_instructions_file` in profiles
- Custom prompt directory scanning in `codex-rs/core/src/custom_prompts.rs`
- Configuration-driven prompt behavior through profiles

## Code References

- `codex-rs/tui/src/slash_command.rs:15-35` - Built-in slash command definitions
- `codex-rs/core/src/custom_prompts.rs:44-85` - Custom prompt discovery mechanism
- `codex-rs/tui/src/bottom_pane/command_popup.rs:145-165` - Slash command integration
- `codex-rs/core/src/config.rs:604-708` - Main configuration structure with extension points
- `codex-rs/common/src/config_override.rs:42-77` - CLI override parsing for dynamic config
- `codex-rs/core/src/config_profile.rs:11-23` - Profile system for grouped settings
- `codex-rs/core/src/project_doc.rs:109-174` - Project documentation discovery

## Architecture Insights

**Configuration Design Philosophy**:
1. **Security-First**: Explicit trust management prevents automatic execution of project-specific code
2. **Global Config Preference**: Single source of truth in `~/.codex/config.toml` avoids configuration sprawl
3. **Profile-Based Flexibility**: Named configuration sets enable project-specific behavior without per-project files
4. **Extension-Friendly**: Serde defaults and HashMap patterns enable backward-compatible schema evolution

**Slash Command Integration Points**:
1. **Discovery Phase**: Custom prompts loaded from configured directories
2. **Rendering Phase**: Command popup combines built-ins and custom prompts
3. **Execution Phase**: Content submission without preprocessing

## Implementation Recommendations

### Option 1: Profile-Based Project Prompts (Recommended)

Extend the profile system to support project-specific prompt directories:

**Configuration Schema Addition**:
```rust
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct ConfigProfile {
    // ... existing fields
    pub custom_prompt_directories: Option<Vec<PathBuf>>,
    pub project_prompts: Option<HashMap<String, ProjectPromptConfig>>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct ProjectPromptConfig {
    pub path: PathBuf,
    pub description: Option<String>,
    pub category: Option<String>,
}
```

**Usage Example**:
```toml
[profiles.team-project]
model = "gpt-4"
custom_prompt_directories = ["./team-prompts", "./docs/prompts"]

[profiles.team-project.project_prompts.code-review]
path = "./prompts/code-review-template.md"
description = "Team code review checklist"
category = "development"
```

**CLI Usage**:
```bash
# Activate project profile
codex --profile team-project

# Override prompt directories
codex -c profiles.team-project.custom_prompt_directories='["./custom"]'
```

### Option 2: Project Configuration Files

Add support for project-specific `.codex.toml` files:

**Implementation Changes**:
1. Modify `codex-rs/core/src/config.rs` to discover project config files
2. Add project config schema with prompt-specific fields
3. Update configuration merging to include project layer
4. Extend trust system to validate project config usage

**Configuration Hierarchy**:
```
CLI overrides → Profile config → Project .codex.toml → Global config.toml → Defaults
```

### Option 3: Directory-Based Prompt Discovery

Extend the current custom prompt system to support multiple directories:

**Configuration Addition**:
```toml
[prompts]
directories = [
    "~/.codex/prompts",           # Global prompts
    "./team-prompts",             # Project team prompts
    "./docs/codex-prompts"        # Documentation prompts
]
exclude_patterns = ["*.draft.md", "README.md"]
```

**Implementation**:
- Modify `codex-rs/core/src/custom_prompts.rs` to scan multiple directories
- Add directory precedence rules (later directories override earlier ones)
- Support glob patterns for exclusion

### Recommended Implementation Path

**Phase 1: Profile Extension (Low Risk)**
1. Add `custom_prompt_directories` field to `ConfigProfile`
2. Modify prompt discovery to check profile-specific directories
3. Test with existing profile system

**Phase 2: Enhanced Metadata (Medium Risk)**
1. Add prompt metadata support (description, category)
2. Enhance command popup to display metadata
3. Support prompt organization and filtering

**Phase 3: Template System (High Risk)**
1. Add variable substitution to prompt content
2. Support context injection (git info, project name)
3. Implement template inheritance and includes

## Related Research

- [RESEARCH_0001_17-09-2025_wed_technical-deep-dive-configuration-management.md](./RESEARCH_0001_17-09-2025_wed_technical-deep-dive-configuration-management.md) - Comprehensive configuration system analysis

## Open Questions

1. **Security Model**: How should project-specific prompts be validated and trusted?
2. **Template Complexity**: What level of templating is appropriate without over-engineering?
3. **Performance Impact**: How does multi-directory prompt discovery affect startup time?
4. **UI/UX Design**: How should project prompts be visually distinguished from global ones?
5. **Migration Path**: How can existing custom prompts be migrated to new project-specific system?
6. **Workspace Integration**: Should project prompts be discovered relative to Git root or current directory?

## Follow-up Research 2025-09-17T23:30:53-05:00

### Research Questions

1. **Automatic Discovery**: Would it be possible to have it just discover a `.codex/prompts` directory at the project level without having to edit a config file?
2. **Directory Commands**: Would it be possible to add `/directory:command` in the CLI which matches the `.codex/prompts/directory/command.md` found in the project level dir?

### Summary of Follow-up Findings

**YES to both questions** - The existing codebase provides excellent patterns and infrastructure for both automatic `.codex/prompts` discovery and `/directory:command` syntax. Both features can be implemented with minimal changes to existing code.

### Detailed Analysis

#### Automatic `.codex/prompts` Discovery - HIGHLY FEASIBLE

**Existing Foundation**:
- **Git Repository Detection**: `codex-rs/core/src/git_info.rs:get_git_repo_root()` provides reliable project boundary detection
- **Project Documentation Pattern**: `codex-rs/core/src/project_doc.rs` already implements automatic discovery of `AGENTS.md` files from Git root to current directory
- **Custom Prompt Infrastructure**: `codex-rs/core/src/custom_prompts.rs:discover_prompts_in()` handles `.md` file scanning with proper filtering

**Implementation Strategy**:
```rust
// Extend existing custom prompts discovery
pub async fn discover_prompts_with_project_support(
    global_dir: &Path,
    project_cwd: &Path,
) -> Vec<CustomPrompt> {
    let mut prompts = Vec::new();

    // 1. Global prompts (existing behavior)
    prompts.extend(discover_prompts_in(global_dir).await);

    // 2. Project prompts (new behavior)
    if let Some(git_root) = crate::git_info::get_git_repo_root(project_cwd) {
        let project_prompt_dir = git_root.join(".codex/prompts");
        prompts.extend(discover_prompts_in(&project_prompt_dir).await);
    }

    // 3. Project overrides global (later sources take precedence)
    deduplicate_by_name(prompts)
}
```

**Integration Points**:
- **Main Handler**: `codex-rs/core/src/codex.rs:1435-1452` - `Op::ListCustomPrompts` handler needs single line change
- **Discovery Function**: `codex-rs/core/src/custom_prompts.rs:44-85` - Extend to scan multiple directories
- **No Configuration Required**: Uses existing Git detection, no config file changes needed

#### `/directory:command` Syntax - MODERATELY FEASIBLE

**Current Slash Command Parsing**:
- **Entry Point**: `codex-rs/tui/src/bottom_pane/command_popup.rs:75-80` - Command token extraction
- **Current Logic**: `token.split_whitespace().next().unwrap_or("")` - Takes first whitespace-delimited token
- **Limitation**: No support for colon-separated syntax

**Required Changes for Directory Commands**:

**1. Enhanced Token Parsing**:
```rust
// In command_popup.rs:on_composer_text_change()
pub(crate) fn on_composer_text_change(&mut self, text: String) {
    let first_line = text.lines().next().unwrap_or("");

    if let Some(stripped) = first_line.strip_prefix('/') {
        let token = stripped.trim_start();
        let cmd_token = token.split_whitespace().next().unwrap_or("");

        // NEW: Parse directory:command syntax
        self.command_filter = if cmd_token.contains(':') {
            cmd_token.to_string() // Keep full directory:command
        } else {
            cmd_token.to_string() // Existing flat command
        };
    }
}
```

**2. Directory-Based Custom Prompt Discovery**:
```rust
// Extend CustomPrompt structure
pub struct CustomPrompt {
    pub name: String,           // "directory:command" for nested
    pub display_name: String,   // "command" for UI display
    pub category: Option<String>, // "directory" for organization
    pub path: PathBuf,
    pub content: String,
}

// Enhanced discovery with directory structure
pub async fn discover_directory_prompts(base_dir: &Path) -> Vec<CustomPrompt> {
    let mut prompts = Vec::new();

    // Scan subdirectories
    if let Ok(mut entries) = fs::read_dir(base_dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            if entry.file_type().await.map(|ft| ft.is_dir()).unwrap_or(false) {
                let dir_name = entry.file_name().to_string_lossy().to_string();
                let subdir_prompts = discover_prompts_in(&entry.path()).await;

                // Convert flat prompts to directory:command format
                for mut prompt in subdir_prompts {
                    prompt.name = format!("{}:{}", dir_name, prompt.name);
                    prompt.category = Some(dir_name.clone());
                    prompts.push(prompt);
                }
            }
        }
    }

    prompts
}
```

**3. Enhanced Command Matching**:
```rust
// In command_popup.rs - enhance fuzzy matching for directory commands
pub(crate) fn update_command_list(&mut self) {
    let filter = &self.command_filter;
    let mut out: Vec<(CommandItem, Option<Vec<usize>>, i32)> = Vec::new();

    // Handle directory:command syntax
    if filter.contains(':') {
        // Match against full directory:command names
        for (idx, prompt) in self.prompts.iter().enumerate() {
            if let Some((indices, score)) = fuzzy_match(&prompt.name, filter) {
                out.push((CommandItem::UserPrompt(idx), Some(indices), score));
            }
        }
    } else {
        // Existing matching logic for flat commands
        // ... existing code
    }
}
```

### Hierarchical Command Patterns in Codebase

**Existing Models**:
- **CLI Subcommands**: `codex-rs/cli/src/main.rs` - `codex mcp add`, `codex login status`
- **MCP Hierarchy**: `codex-rs/cli/src/mcp_cmd.rs` - Nested command structure
- **Arg0 Dispatch**: `codex-rs/arg0/src/lib.rs` - Executable name-based routing

**Best Model for Directory Commands**:
The MCP subcommand pattern (`McpSubcommand::Add`, `McpSubcommand::List`) provides the clearest model for hierarchical command organization.

### Implementation Roadmap

#### Phase 1: Automatic `.codex/prompts` Discovery (LOW RISK)
1. **Extend Discovery Function**: Modify `discover_prompts_in()` to accept multiple directories
2. **Add Git Integration**: Use `get_git_repo_root()` to find project `.codex/prompts`
3. **Update Call Sites**: Modify `Op::ListCustomPrompts` to use enhanced discovery
4. **Test Project Detection**: Verify behavior in Git repos, worktrees, and non-Git directories

#### Phase 2: Directory Structure Support (MEDIUM RISK)
1. **Enhanced Metadata**: Add `category` field to `CustomPrompt` structure
2. **Directory Scanning**: Implement recursive discovery in subdirectories
3. **Name Collision Handling**: Implement precedence rules (project > global, subdirectory > root)
4. **UI Grouping**: Enhance command popup to show directory categories

#### Phase 3: `/directory:command` Syntax (MEDIUM RISK)
1. **Parser Extension**: Modify token extraction to preserve colon syntax
2. **Fuzzy Matching**: Enhance matching algorithm for qualified names
3. **UI Updates**: Update command popup to display hierarchical commands
4. **Validation**: Add error handling for malformed directory:command syntax

### Code Integration Points

**Primary Files to Modify**:
1. `codex-rs/core/src/custom_prompts.rs:44-85` - Enhanced discovery with Git integration
2. `codex-rs/tui/src/bottom_pane/command_popup.rs:75-80` - Directory:command parsing
3. `codex-rs/core/src/codex.rs:1435-1452` - Update ListCustomPrompts handler

**Supporting Infrastructure** (Already Exists):
- `codex-rs/core/src/git_info.rs:26-42` - Git repository detection
- `codex-rs/core/src/project_doc.rs:109-174` - Project file discovery patterns
- `codex-rs/common/src/fuzzy_match.rs` - Fuzzy matching algorithm

### Example Directory Structure

```
my-project/
├── .git/
├── .codex/
│   └── prompts/
│       ├── review.md          # Available as /review
│       ├── docs/
│       │   ├── api.md         # Available as /docs:api
│       │   └── readme.md      # Available as /docs:readme
│       └── testing/
│           ├── unit.md        # Available as /testing:unit
│           └── integration.md # Available as /testing:integration
└── src/
```

**Available Commands**:
- `/review` - Flat command from root
- `/docs:api` - Directory command
- `/docs:readme` - Directory command
- `/testing:unit` - Directory command
- `/testing:integration` - Directory command

### Benefits of This Approach

1. **Zero Configuration**: Works automatically when `.codex/prompts` directory exists
2. **Backward Compatible**: Existing flat prompts continue to work
3. **Intuitive Structure**: Maps directly to filesystem organization
4. **Project Isolation**: Prompts are scoped to Git repository boundaries
5. **Hierarchical Organization**: Supports team organization patterns
6. **Git Integration**: Respects existing project boundary detection

### Risk Assessment

**Automatic Discovery**: **LOW RISK**
- Leverages proven patterns from AGENTS.md discovery
- Uses existing Git detection infrastructure
- Graceful degradation when `.codex/prompts` doesn't exist

**Directory Commands**: **MEDIUM RISK**
- Requires parser changes to core slash command handling
- UI changes for hierarchical command display
- Potential edge cases with colon in command names
- Fuzzy matching complexity for qualified names

Both features are **highly feasible** and would significantly enhance project-specific prompt capabilities while maintaining the existing user experience.
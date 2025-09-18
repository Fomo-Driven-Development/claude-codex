# Project-Level Custom Prompts Implementation Plan

## Overview

Implementation of automatic `.codex/prompts` directory discovery and `/directory:command` syntax support for project-specific custom prompts in the Codex CLI. This enables teams to maintain project-scoped prompts without configuration changes while supporting hierarchical organization through directory-based namespaces.

## Current State Analysis

**Current Custom Prompt System:**
- Single global directory: `$CODEX_HOME/prompts/*.md`
- Flat command structure: `/command-name`
- Discovery via `custom_prompts.rs:discover_prompts_in()`
- Handler in `codex.rs:1435-1452` calls single directory scan
- Command parsing in `command_popup.rs:75-80` uses whitespace tokenization

**Existing Infrastructure:**
- Robust Git repository detection: `git_info.rs:get_git_repo_root()`
- Project file discovery pattern: `project_doc.rs:discover_project_doc_paths()`
- Fuzzy matching algorithm: `fuzzy_match.rs`
- Well-tested exclusion and collision handling

**Key Constraints:**
- Existing prompts must continue working unchanged
- Built-in commands take precedence over custom prompts
- Security-conscious design prevents arbitrary project code execution
- Performance impact must be minimal

## Desired End State

**Automatic Discovery:**
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
│           └── unit.md        # Available as /testing:unit
└── src/
```

**Command Usage:**
- Flat commands: `/review`, `/deploy`
- Directory commands: `/docs:api`, `/testing:unit`
- Project prompts override global ones with same names
- No configuration required - works when `.codex/prompts` exists

**Verification:**
1. Create `.codex/prompts/` directory with markdown files
2. Subdirectories create namespaced commands automatically
3. Commands appear in slash command popup with fuzzy matching
4. Project prompts take precedence over global prompts

### Key Discoveries:

- `project_doc.rs:discover_project_doc_paths()` provides perfect template for Git-aware directory traversal
- `custom_prompts.rs:discover_prompts_in_excluding()` already supports exclusion for collision handling
- `command_popup.rs:on_composer_text_change()` needs minimal changes for colon syntax
- `fuzzy_match.rs` supports full command name matching without modification

## What We're NOT Doing

- Variable substitution or templating in prompt content
- Project-specific configuration files (`.codex.toml`)
- Authentication or permission systems for project prompts
- Prompt metadata beyond directory categories
- Complex prompt inheritance or includes
- UI redesign beyond directory grouping indicators
- Migration tools for existing global prompts

## Implementation Approach

**Three-phase approach following research document risk assessment:**

1. **Phase 1: Automatic Discovery (LOW RISK)** - Leverage existing Git detection and discovery patterns
2. **Phase 2: Directory Structure Support (MEDIUM RISK)** - Extend prompt metadata and scanning
3. **Phase 3: Directory Command Syntax (MEDIUM RISK)** - Modify command parsing and fuzzy matching

**Key Strategy:**
- Reuse proven patterns from `project_doc.rs` for Git-aware discovery
- Extend existing structures rather than replacing them
- Maintain backward compatibility at every step
- Graceful degradation when `.codex/prompts` doesn't exist

## Phase 1: Automatic `.codex/prompts` Discovery

### Overview

Extend custom prompt discovery to automatically scan project-level `.codex/prompts` directories using Git repository boundaries, following the established pattern from AGENTS.md discovery.

### Changes Required:

#### 1. Enhanced Custom Prompt Discovery

**File**: `codex-rs/core/src/custom_prompts.rs`
**Changes**: Add multi-directory discovery function with Git integration

```rust
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
        let project_prompts = discover_prompts_in_excluding(&project_prompt_dir, &HashSet::new()).await;

        // Project prompts override global ones (later sources take precedence)
        prompts.retain(|p| !project_prompts.iter().any(|pp| pp.name == p.name));
        prompts.extend(project_prompts);
    }

    prompts.sort_by(|a, b| a.name.cmp(&b.name));
    prompts
}
```

#### 2. Update Main Handler

**File**: `codex-rs/core/src/codex.rs` (lines 1435-1452)
**Changes**: Replace single directory discovery with project-aware discovery

```rust
Op::ListCustomPrompts => {
    let sub_id = sub.id.clone();

    let custom_prompts: Vec<CustomPrompt> =
        if let Some(global_dir) = crate::custom_prompts::default_prompts_dir() {
            crate::custom_prompts::discover_prompts_with_project_support(
                &global_dir,
                &config.cwd
            ).await
        } else {
            // Fallback: scan project directory only if global dir unavailable
            if let Some(git_root) = crate::git_info::get_git_repo_root(&config.cwd) {
                let project_dir = git_root.join(".codex/prompts");
                crate::custom_prompts::discover_prompts_in(&project_dir).await
            } else {
                Vec::new()
            }
        };

    let event = Event {
        id: sub_id,
        msg: EventMsg::ListCustomPromptsResponse(ListCustomPromptsResponseEvent {
            custom_prompts,
        }),
    };
    sess.send_event(event).await;
}
```

### Success Criteria:

#### Automated Verification:

- [x] Code compiles successfully: `cargo build`
- [x] Type checking passes: `cargo check`
- [x] Unit tests pass: `cargo test custom_prompts`
- [ ] Integration tests pass: `cargo test discover_prompts_with_project_support`

#### Manual Verification:

- [ ] Global prompts continue working unchanged
- [ ] Project prompts discovered automatically when `.codex/prompts` exists
- [ ] Project prompts override global ones with same names
- [ ] No prompts shown when neither global nor project directories exist
- [ ] Git repository detection works in subdirectories
- [ ] Performance impact negligible during prompt discovery

---

## Phase 2: Directory Structure Support

### Overview

Extend prompt metadata and discovery to support subdirectories within `.codex/prompts`, creating hierarchical organization with directory-based categories.

### Changes Required:

#### 1. Enhanced Prompt Structure

**File**: `codex-rs/protocol/src/custom_prompts.rs`
**Changes**: Add category field to CustomPrompt structure

```rust
#[derive(Serialize, Deserialize, Debug, Clone, TS)]
pub struct CustomPrompt {
    pub name: String,           // "command" or "directory:command"
    pub path: PathBuf,          // full path to the file
    pub content: String,        // file contents
    pub category: Option<String>, // directory name for organization
}
```

#### 2. Directory-Aware Discovery

**File**: `codex-rs/core/src/custom_prompts.rs`
**Changes**: Add recursive directory scanning with namespace support

```rust
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
            if entry.file_type().await.map(|ft| ft.is_dir()).unwrap_or(false) {
                let dir_name = entry.file_name().to_string_lossy().to_string();
                let subdir_prompts = discover_prompts_in_excluding(&entry.path(), &HashSet::new()).await;

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
```

#### 3. Update Discovery Integration

**File**: `codex-rs/core/src/custom_prompts.rs`
**Changes**: Modify project discovery to use directory-aware scanning

```rust
// Update discover_prompts_with_project_support to use new directory function
if let Some(git_root) = crate::git_info::get_git_repo_root(project_cwd) {
    let project_prompt_dir = git_root.join(".codex/prompts");
    let project_prompts = discover_prompts_with_directories(&project_prompt_dir).await;
    // ... rest of logic
}
```

### Success Criteria:

#### Automated Verification:

- [x] Code compiles successfully: `cargo build`
- [x] Unit tests pass: `cargo test discover_prompts_with_directories`
- [ ] Namespace parsing tests pass: `cargo test directory_command_parsing`
- [x] Category metadata correctly populated

#### Manual Verification:

- [ ] Subdirectory prompts discovered with correct namespace format
- [ ] Root level prompts continue working unchanged
- [ ] Category information properly attached to prompts
- [ ] Directory names become command namespaces
- [ ] Empty subdirectories ignored gracefully

---

## Phase 3: Directory Command Syntax Support

### Overview

Modify command parsing and fuzzy matching to support `/directory:command` syntax, enabling users to invoke namespaced prompts through the command popup.

### Changes Required:

#### 1. Enhanced Command Parsing

**File**: `codex-rs/tui/src/bottom_pane/command_popup.rs` (lines 75-80)
**Changes**: Preserve colon syntax in command token extraction

```rust
pub(crate) fn on_composer_text_change(&mut self, text: String) {
    let first_line = text.lines().next().unwrap_or("");

    if let Some(stripped) = first_line.strip_prefix('/') {
        let token = stripped.trim_start();
        let cmd_token = token.split_whitespace().next().unwrap_or("");

        // Enhanced: Preserve colon syntax for directory commands
        self.command_filter = cmd_token.to_string();
    } else {
        self.command_filter.clear();
    }

    // Reset selection based on filtered list
    let matches_len = self.filtered_items().len();
    self.state.clamp_selection(matches_len);
    self.state.ensure_visible(matches_len, MAX_POPUP_ROWS.min(matches_len));
}
```

#### 2. Enhanced Fuzzy Matching

**File**: `codex-rs/tui/src/bottom_pane/command_popup.rs` (lines 144-153)
**Changes**: Support full command name matching for directory syntax

```rust
// In filtered() method - enhance user prompt matching
for (idx, p) in self.prompts.iter().enumerate() {
    // Match against full command name (supports directory:command syntax)
    if let Some((indices, score)) = fuzzy_match(&p.name, filter) {
        out.push((CommandItem::UserPrompt(idx), Some(indices), score));
    }
}
```

#### 3. UI Display Enhancement

**File**: `codex-rs/tui/src/bottom_pane/command_popup.rs` (lines 207-221)
**Changes**: Enhanced display for directory commands in popup

```rust
CommandItem::UserPrompt(i) => GenericDisplayRow {
    name: format!("/{}", self.prompts[i].name),
    match_indices: indices.map(|v| v.into_iter().map(|i| i + 1).collect()),
    is_current: false,
    description: match &self.prompts[i].category {
        Some(category) => Some(format!("send saved prompt ({})", category)),
        None => Some("send saved prompt".to_string()),
    },
},
```

### Success Criteria:

#### Automated Verification:

- [x] Code compiles successfully: `cargo build`
- [x] Command parsing tests pass: `cargo test command_popup`
- [ ] Fuzzy matching tests pass: `cargo test fuzzy_match_directory_commands`
- [x] UI rendering tests pass: `cargo test command_display`

#### Manual Verification:

- [ ] `/directory:command` syntax recognized in command popup
- [ ] Fuzzy matching works for both flat and directory commands
- [ ] Command completion correctly handles colon syntax
- [ ] Directory commands display category information
- [ ] Tab completion works for directory commands
- [ ] Error handling for malformed directory syntax

---

## Test Plan Reference

**Related Test Plan**: `.strategic-claude-basic/plan/TEST_0001_17-09-2025_wed_project-level-custom-prompts.md`

**Testing Focus Areas:**
- **File System Operations**: Directory discovery, Git detection, edge cases
- **Command Parsing**: Colon syntax, fuzzy matching, completion
- **Integration**: Multi-directory discovery, precedence rules, performance
- **Backward Compatibility**: Existing prompts continue working unchanged

Comprehensive test coverage includes unit tests for each component, integration tests for cross-system functionality, and manual verification of user workflows.

## Performance Considerations

**Discovery Performance:**
- Git repository detection adds minimal overhead (single directory traversal)
- Subdirectory scanning limited to single level depth
- Prompt caching mechanisms remain unchanged
- File system operations use existing async patterns

**Command Parsing Performance:**
- Colon syntax detection adds no overhead to existing commands
- Fuzzy matching algorithm unchanged, works with longer command names
- UI rendering optimized for category display

**Startup Impact:**
- Project discovery happens during existing prompt loading phase
- No additional discovery calls during normal operation
- Graceful degradation when Git or directories unavailable

## Migration Notes

**Backward Compatibility:**
- All existing global prompts continue working unchanged
- No breaking changes to existing CLI workflows
- Project prompts are additive enhancement
- Graceful fallback when `.codex/prompts` doesn't exist

**Adoption Path:**
1. Teams can immediately create `.codex/prompts` directory
2. Flat prompts work immediately (Phase 1)
3. Directory organization available after Phase 2
4. Advanced syntax available after Phase 3

**Data Migration:**
- No migration required for existing global prompts
- Teams can gradually move shared prompts to project directories
- Global prompts serve as fallback for prompts not in project

## References

- Related research: `.strategic-claude-basic/research/RESEARCH_0002_17-09-2025_wed_project-level-custom-prompts.md`
- Project doc discovery pattern: `codex-rs/core/src/project_doc.rs:109-174`
- Git detection implementation: `codex-rs/core/src/git_info.rs:26-42`
- Custom prompt discovery: `codex-rs/core/src/custom_prompts.rs:17-74`
- Command popup parsing: `codex-rs/tui/src/bottom_pane/command_popup.rs:68-93`
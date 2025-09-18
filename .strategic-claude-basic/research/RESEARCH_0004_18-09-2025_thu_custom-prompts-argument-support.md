---
date: 2025-09-18T08:35:20-05:00
git_commit: c9505488a120299b339814d73f57817ee79e114f
branch: main
repository: codex
topic: "How to extend Codex's custom command system to support arguments and template processing"
tags: [research, custom-prompts, commands, templates, arguments, parsing]
status: complete
last_updated: 2025-09-18
---

# Research: How to extend Codex's custom command system to support arguments and template processing

**Date**: 2025-09-18T08:35:20-05:00
**Git Commit**: c9505488a120299b339814d73f57817ee79e114f
**Branch**: main
**Repository**: codex

## Research Question

How can we fix Codex's custom command system to support arguments? Currently, `/research "subject"` ignores the arguments completely and only uses the command name. The system needs to be extended to support parameterized templates with variable substitution.

## Summary

Codex's custom command system is currently designed as a simple static content injection system, not a template processing system. The issue stems from three architectural limitations:

1. **Parser limitation**: Only extracts the first token from `/command args` syntax (`command_popup.rs:76`)
2. **No template processing**: Custom prompts return raw file content without substitution (`chat_composer.rs:436-437`)
3. **Simple data structure**: `CustomPrompt` struct has no argument/template metadata fields

**Solution approach**: Extend the system with shlex-based argument parsing, Askama template processing (already used in codebase), and enhanced CustomPrompt structure to support parameterized templates.

## Detailed Findings

### Current System Architecture

#### Command Parsing Limitation
**File**: `codex-rs/tui/src/bottom_pane/command_popup.rs:76`
```rust
let cmd_token = token.split_whitespace().next().unwrap_or("");
```
The parser intentionally extracts only the first token after `/`, discarding all arguments. This is by design to show help for `/clear something` as if it were just `/clear`.

#### Custom Prompt Execution
**File**: `codex-rs/tui/src/bottom_pane/chat_composer.rs:436-437`
```rust
CommandItem::UserPrompt(_) => {
    if let Some(contents) = prompt_content {
        return (InputResult::Submitted(contents), true);
    }
}
```
Custom prompts bypass all processing and submit raw file content directly to the agent.

#### Data Structure
**File**: `codex-rs/protocol/src/custom_prompts.rs:7-12`
```rust
pub struct CustomPrompt {
    pub name: String,             // "command" or "directory:command"
    pub path: PathBuf,            // full path to the file
    pub content: String,          // file contents
    pub category: Option<String>, // directory name for organization
}
```
No fields exist for argument definitions or template metadata.

### Existing Template Processing Patterns

#### Askama Template Engine (Recommended)
**File**: `codex-rs/core/src/codex/compact.rs`
The codebase already uses Askama for structured templating:
```rust
#[derive(Template)]
#[template(path = "compact/history_bridge.md", escape = "none")]
struct HistoryBridgeTemplate<'a> {
    user_messages_text: &'a str,
    summary_text: &'a str,
}
```
Template syntax uses `{{ variable_name }}` for substitution.

#### Placeholder Replacement System
**File**: `codex-rs/tui/src/bottom_pane/chat_composer.rs`
The TUI implements sophisticated placeholder replacement for paste handling:
```rust
for (placeholder, actual) in &self.pending_pastes {
    if text.contains(placeholder) {
        text = text.replace(placeholder, actual);
    }
}
```

### Command Argument Parsing Patterns

#### Shlex Library Usage
**File**: `codex-rs/core/src/parse_command.rs:5-6`
The codebase extensively uses shlex for shell-style argument parsing:
```rust
use shlex::split as shlex_split;

let tokens = shlex_split(input).unwrap_or_else(||
    input.split_whitespace().map(|s| s.to_string()).collect()
);
```
This provides robust handling of quoted arguments: `/command arg1 "arg with spaces" arg3`

#### Command Line Processing
**File**: `codex-rs/cli/src/main.rs:28-47`
Structured argument parsing using Clap for complex CLI commands.

## Architecture Design for Template Support

### Phase 1: Enhanced Argument Parsing

**Modify**: `codex-rs/tui/src/bottom_pane/command_popup.rs:68-93`

Replace single-token extraction with full argument parsing:
```rust
pub(crate) fn on_composer_text_change(&mut self, text: String) {
    let first_line = text.lines().next().unwrap_or("");

    if let Some(stripped) = first_line.strip_prefix('/') {
        // Parse full command line with arguments
        let tokens = shlex::split(stripped).unwrap_or_else(||
            stripped.split_whitespace().map(String::from).collect()
        );

        self.command_filter = tokens.first().unwrap_or("").to_string();
        self.command_args = tokens[1..].to_vec(); // Store arguments
    }
}
```

### Phase 2: Enhanced Data Structure

**Modify**: `codex-rs/protocol/src/custom_prompts.rs:7-12`

Extend CustomPrompt with template metadata:
```rust
#[derive(Serialize, Deserialize, Debug, Clone, TS)]
pub struct CustomPrompt {
    pub name: String,
    pub path: PathBuf,
    pub content: String,
    pub category: Option<String>,
    // New fields for argument support
    pub template_args: Option<Vec<TemplateArg>>,
    pub template_syntax: Option<TemplateSyntax>,
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
pub struct TemplateArg {
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
    pub default_value: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
pub enum TemplateSyntax {
    Simple,    // {arg1}, {arg2}
    Askama,    // {{ arg1 }}, {{ arg2 }}
}
```

### Phase 3: Template Processing Engine

**Create**: `codex-rs/core/src/template_processor.rs`

Implement template substitution with multiple syntax support:
```rust
pub fn process_template(
    content: &str,
    args: &[String],
    syntax: TemplateSyntax
) -> Result<String, TemplateError> {
    match syntax {
        TemplateSyntax::Simple => process_simple_template(content, args),
        TemplateSyntax::Askama => process_askama_template(content, args),
    }
}
```

### Phase 4: Execution Integration

**Modify**: `codex-rs/tui/src/bottom_pane/chat_composer.rs:436-437`

Add template processing before content submission:
```rust
CommandItem::UserPrompt(_) => {
    if let Some(contents) = prompt_content {
        let processed = if let Some(args) = self.command_args {
            template_processor::process_template(&contents, &args, prompt.template_syntax)?
        } else {
            contents
        };
        return (InputResult::Submitted(processed), true);
    }
}
```

## Code References

- `codex-rs/tui/src/bottom_pane/command_popup.rs:76` - Current token parsing limitation
- `codex-rs/tui/src/bottom_pane/chat_composer.rs:436-437` - Custom prompt execution without processing
- `codex-rs/protocol/src/custom_prompts.rs:7-12` - CustomPrompt data structure
- `codex-rs/core/src/parse_command.rs:5-6` - Shlex parsing patterns
- `codex-rs/core/src/codex/compact.rs` - Askama template usage example
- `codex-rs/tui/src/slash_command.rs` - Built-in slash command definitions
- `codex-rs/core/src/custom_prompts.rs:114-142` - Prompt discovery system

## Architecture Insights

1. **Separation of Concerns**: Built-in slash commands use enum dispatch while custom prompts use file-based content. Template support should bridge this gap.

2. **Existing Infrastructure**: The codebase has robust patterns for:
   - Shell-style argument parsing (shlex)
   - Template processing (Askama)
   - File-based configuration (custom prompts discovery)

3. **Integration Points**: Changes needed across 4 layers:
   - **UI Layer**: Enhanced command popup and composer
   - **Protocol Layer**: Extended CustomPrompt structure
   - **Core Layer**: Template processing engine
   - **Configuration**: Backward compatibility for existing prompts

4. **Template Syntax Options**:
   - **Simple**: `{arg1}`, `{arg2}` - easier for users
   - **Askama**: `{{ arg1 }}`, `{{ arg2 }}` - more powerful but complex

## Implementation Strategy

### Phase 1: Foundation (Low Risk)
- Add template processing utilities using existing patterns
- Extend CustomPrompt structure with optional template fields
- Maintain backward compatibility for existing prompts

### Phase 2: Parsing Enhancement (Medium Risk)
- Modify command popup to capture arguments
- Update chat composer to pass arguments to template processor
- Add argument validation and error handling

### Phase 3: Advanced Features (High Value)
- Support for named arguments: `/research --subject="AI" --depth=detailed`
- Argument completion and validation in UI
- Template syntax highlighting in prompt files

### Phase 4: Documentation & Migration
- Update prompt documentation with template examples
- Provide migration guide for existing prompts
- Add template validation during prompt discovery

## Related Research

- [RESEARCH_0002_17-09-2025_wed_project-level-custom-prompts.md](RESEARCH_0002_17-09-2025_wed_project-level-custom-prompts.md) - Project-level custom prompt structure
- [RESEARCH_0001_17-09-2025_wed_technical-deep-dive-configuration-management.md](RESEARCH_0001_17-09-2025_wed_technical-deep-dive-configuration-management.md) - Configuration system architecture

## Open Questions

1. **Template Syntax Choice**: Should we use simple `{arg}` syntax or full Askama `{{ arg }}` templates?
2. **Argument Validation**: How strict should validation be for missing/extra arguments?
3. **Backward Compatibility**: How to handle existing `.md` prompt files during transition?
4. **UI/UX**: Should argument hints be shown in the command popup during typing?
5. **Error Handling**: How to gracefully handle template processing errors in the UI?
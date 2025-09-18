# Custom Prompts Argument Support Implementation Plan

## Overview

Extend Codex's custom command system to support arguments and template processing. Currently, `/research "subject"` ignores arguments and only uses the command name. This implementation will enable parameterized templates with variable substitution using existing infrastructure patterns.

## Current State Analysis

**Root Problem**: Three architectural limitations prevent argument support:

1. **Parser strips arguments** (`command_popup.rs:76`): `token.split_whitespace().next()` extracts only the first token
2. **No template processing** (`chat_composer.rs:436-437`): Custom prompts return raw file content without substitution
3. **Simple data structure** (`custom_prompts.rs:7-12`): `CustomPrompt` struct lacks argument metadata fields

**Existing Infrastructure Ready for Leverage**:
- **Askama templates**: Already used with `{{ variable }}` syntax in `compact.rs`
- **Shlex parsing**: Extensively used for shell-style argument handling in `parse_command.rs`
- **String replacement**: Placeholder patterns exist in `chat_composer.rs` for paste handling

## Desired End State

Users can create parameterized custom prompts and use them with arguments:

```bash
# Command usage
/research "AI safety in autonomous vehicles"
/debug --error="segfault" --context="production"

# Template file (.codex/prompts/research.md)
Research the following topic in detail: {{ subject }}

Focus on these key areas:
- Current state of the art
- Technical challenges
- Recent developments
```

**Verification**:
- Arguments are parsed with shell-style quoting support
- Template variables are substituted correctly
- Backward compatibility maintained for existing prompts
- Error handling for missing/invalid arguments

### Key Discoveries:

- **Template syntax consistency**: Use `{{ variable }}` to match existing Askama patterns (`compact.rs:33-38`)
- **Argument parsing infrastructure**: Shlex parsing with fallback already implemented (`parse_command.rs:76-78`)
- **Command enumeration pattern**: Clear separation between built-in and custom commands (`command_popup.rs:16-21`)
- **Error handling patterns**: Result-based processing with graceful fallbacks throughout codebase

## What We're NOT Doing

- **Complex control flow**: No conditionals, loops, or advanced template features (use simple variable substitution)
- **Built-in command changes**: No modifications to existing slash commands like `/model`, `/diff`
- **UI/UX redesign**: Minimal changes to command popup appearance and behavior
- **Migration tools**: No automatic conversion of existing prompts (backward compatibility only)
- **Advanced validation**: No schema validation or type checking for arguments (basic presence checks only)

## Implementation Approach

**Strategy**: Extend existing patterns rather than creating new systems. Use Askama for template processing (already integrated), shlex for argument parsing (already used), and maintain the current CustomPrompt file-based approach with optional template metadata.

**Philosophy**: Incremental enhancement with graceful degradation - existing prompts continue working unchanged while new prompts can opt into argument support.

## Phase 1: Enhanced Argument Parsing

### Overview

Modify command parsing to capture and store arguments while maintaining backward compatibility with existing command filtering and completion.

### Changes Required:

#### 1. Command Popup Enhancement

**File**: `codex-rs/tui/src/bottom_pane/command_popup.rs`
**Changes**: Replace single-token extraction with full argument parsing

```rust
// Replace line 76: let cmd_token = token.split_whitespace().next().unwrap_or("");
// With full argument parsing:

let tokens = shlex::split(stripped).unwrap_or_else(||
    stripped.split_whitespace().map(String::from).collect()
);

self.command_filter = tokens.first().unwrap_or(&String::new()).clone();
self.command_args = tokens.get(1..).unwrap_or(&[]).to_vec();
```

**Additional field needed**:
```rust
pub struct CommandPopup {
    // existing fields...
    pub(crate) command_args: Vec<String>, // Store parsed arguments
}
```

#### 2. Argument Storage Structure

**File**: `codex-rs/tui/src/bottom_pane/command_popup.rs`
**Changes**: Add argument storage and accessor methods

```rust
impl CommandPopup {
    pub(crate) fn new(/* existing params */) -> Self {
        Self {
            // existing initialization...
            command_args: Vec::new(),
        }
    }

    pub(crate) fn current_arguments(&self) -> &[String] {
        &self.command_args
    }
}
```

### Success Criteria:

#### Automated Verification:

- [x] Code compiles successfully: `cargo build -p codex-tui`
- [x] No clippy warnings: `just fix -p tui`
- [x] Formatting is correct: `just fmt`

#### Manual Verification:

- [x] Command popup still filters commands correctly with `/mo` → shows model command
- [x] Arguments are captured: `/research "test subject"` stores `["test subject"]` in command_args
- [x] Quoted arguments work: `/debug "error message" flag` captures both arguments correctly
- [x] Existing command completion behavior unchanged

---

## Phase 2: Template Processing Engine

### Overview

Create a template processing system that supports both simple variable substitution and optional Askama template processing for custom prompts.

### Changes Required:

#### 1. Template Processor Module

**File**: `codex-rs/core/src/template_processor.rs` (new file)
**Changes**: Create template processing utilities

```rust
use askama::Template;
use std::collections::HashMap;

#[derive(Debug)]
pub enum TemplateError {
    MissingVariable(String),
    ProcessingError(String),
}

pub enum TemplateSyntax {
    Simple,    // {arg1}, {arg2}
    Askama,    // {{ arg1 }}, {{ arg2 }}
}

pub fn process_simple_template(
    content: &str,
    args: &[String]
) -> Result<String, TemplateError> {
    let mut result = content.to_string();
    for (i, arg) in args.iter().enumerate() {
        let placeholder = format!("{{{}}}", i);
        result = result.replace(&placeholder, arg);
    }
    Ok(result)
}

pub fn process_askama_template(
    content: &str,
    args: &HashMap<String, String>
) -> Result<String, TemplateError> {
    // Use existing Askama patterns from compact.rs
    // Template processing with {{ variable }} syntax
}
```

#### 2. CustomPrompt Extension

**File**: `codex-rs/protocol/src/custom_prompts.rs`
**Changes**: Extend CustomPrompt with optional template metadata

```rust
#[derive(Serialize, Deserialize, Debug, Clone, TS)]
pub struct CustomPrompt {
    pub name: String,
    pub path: PathBuf,
    pub content: String,
    pub category: Option<String>,
    // New fields for template support
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
    Simple,
    Askama,
}
```

### Success Criteria:

#### Automated Verification:

- [x] Core crate compiles: `cargo build -p codex-core`
- [x] Protocol crate compiles: `cargo build -p codex-protocol`
- [x] Template processor tests pass: `cargo test -p codex-core template_processor`
- [x] Type definitions generate: `cargo build --all-features`

#### Manual Verification:

- [x] Simple template substitution works: `{0}` → first argument
- [x] Askama template processing works: `{{ variable }}` → argument value
- [x] Error handling works for missing variables
- [x] Template metadata is properly serialized/deserialized

---

## Phase 3: Execution Integration

### Overview

Integrate template processing into the command execution flow, connecting argument parsing to template substitution while maintaining backward compatibility.

### Changes Required:

#### 1. Chat Composer Integration

**File**: `codex-rs/tui/src/bottom_pane/chat_composer.rs`
**Changes**: Add template processing before content submission

```rust
// Replace lines 436-437 with template processing:
CommandItem::UserPrompt(idx) => {
    if let Some(contents) = prompt_content {
        let processed_content = if let Some(args) = self.popup.current_arguments() {
            if !args.is_empty() {
                // Get the prompt metadata to determine template syntax
                let prompt = &self.popup.prompts()[*idx];
                template_processor::process_template(
                    &contents,
                    args,
                    prompt.template_syntax.unwrap_or(TemplateSyntax::Simple)
                ).unwrap_or(contents) // Fallback to original on error
            } else {
                contents
            }
        } else {
            contents
        };
        return (InputResult::Submitted(processed_content), true);
    }
}
```

#### 2. Error Handling Integration

**File**: `codex-rs/tui/src/bottom_pane/chat_composer.rs`
**Changes**: Add graceful error handling for template processing

```rust
impl ChatComposer {
    fn process_custom_prompt_with_args(
        &self,
        content: &str,
        prompt: &CustomPrompt,
        args: &[String]
    ) -> String {
        match template_processor::process_template(content, args, prompt.template_syntax) {
            Ok(processed) => processed,
            Err(err) => {
                // Log error and return original content
                eprintln!("Template processing error: {:?}", err);
                content.to_string()
            }
        }
    }
}
```

### Success Criteria:

#### Automated Verification:

- [x] TUI crate compiles: `cargo build -p codex-tui`
- [x] All integration tests pass: `cargo test -p codex-tui`
- [x] No regression in existing slash commands: `cargo test --all-features`

#### Manual Verification:

- [x] Custom prompts with arguments work end-to-end: `/research "AI safety"` → template substitution
- [x] Existing custom prompts without arguments continue working unchanged
- [x] Error cases handled gracefully: missing arguments don't crash
- [x] Built-in slash commands unaffected: `/model`, `/diff` work normally

---

## Test Plan Reference

**Related Test Plan**: `.strategic-claude-basic/plan/TEST_0002_18-09-2025_thu_custom-prompts-argument-support.md`

Testing covers argument parsing validation, template processing correctness, error handling, backward compatibility, and UI snapshot testing for command popup behavior. All testing implementation details and coverage requirements are specified in the dedicated test plan.

## Performance Considerations

**Template Compilation**: Consider caching compiled Askama templates for frequently used prompts to avoid re-compilation overhead.

**Argument Parsing**: Shlex parsing adds minimal overhead and is already used extensively throughout the codebase.

**Memory Usage**: Storing command arguments in CommandPopup adds negligible memory usage (Vec<String> for a few arguments).

## Migration Notes

**Backward Compatibility**: Existing `.md` custom prompt files continue working unchanged - they are treated as static content when no arguments are provided.

**Optional Enhancement**: Users can add template syntax to existing prompts by simply using `{{ variable }}` or `{0}` syntax in their content.

**No Migration Required**: No data migration or file format changes needed.

## References

- Related research: `.strategic-claude-basic/research/RESEARCH_0004_18-09-2025_thu_custom-prompts-argument-support.md`
- Askama patterns: `codex-rs/core/src/codex/compact.rs:33-38`
- Shlex usage: `codex-rs/core/src/parse_command.rs:76-78`
- Command parsing: `codex-rs/tui/src/bottom_pane/command_popup.rs:76`
- Custom prompt execution: `codex-rs/tui/src/bottom_pane/chat_composer.rs:436-437`
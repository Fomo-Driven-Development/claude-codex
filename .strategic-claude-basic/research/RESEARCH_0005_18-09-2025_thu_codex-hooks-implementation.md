---
date: 2025-09-18T11:12:06-05:00
git_commit: 3279a55f48a4b5e3fbedc49a9dabe196800b952d
branch: claude-codex
repository: codex
topic: "How can we get .codex/hooks/ in codex to work like .claude/hooks in claude code? Look up the anthropic docs to look at all the features then lets focus on the stop hook. We should be able to use a simliar approach as the .claude/settings.json file. Lets figure out how we can replicate that in codex."
tags: [research, codebase, hooks, configuration, claude-code, rust]
status: complete
last_updated: 2025-09-18
---

# Research: Implementing .codex/hooks/ to Work Like .claude/hooks in Claude Code

**Date**: 2025-09-18T11:12:06-05:00
**Git Commit**: 3279a55f48a4b5e3fbedc49a9dabe196800b952d
**Branch**: claude-codex
**Repository**: codex

## Research Question

How can we get .codex/hooks/ in codex to work like .claude/hooks in claude code? Look up the anthropic docs to look at all the features then lets focus on the stop hook. We should be able to use a simliar approach as the .claude/settings.json file. Lets figure out how we can replicate that in codex.

## Summary

**Key Finding**: The current .claude/settings.json hook system is implemented externally to the Rust codex codebase by the Claude client itself. The Rust codebase only provides basic notification infrastructure. To implement .codex/hooks/, we need to add native hook processing to the Rust codebase using existing configuration patterns.

**Implementation Focus**: Implement Stop hook support by extending the existing notification system (`config.rs:1027-1051`) to execute hooks when sessions complete. Use TOML configuration format with `$CODEX_PROJECT_DIR` environment variable for portable hook paths.

## Detailed Findings

### Claude Code Hook System (External Documentation)

Claude Code provides 9 distinct hook types:
- **PreToolUse** - Runs before tool calls (can block them)
- **PostToolUse** - Runs after tool calls complete
- **UserPromptSubmit** - Runs when user submits a prompt
- **Notification** - Runs when Claude Code sends notifications
- **Stop** - Runs when Claude Code finishes responding
- **SubagentStop** - Runs when subagent tasks complete
- **PreCompact** - Runs before context compaction
- **SessionStart** - Runs when session starts/resumes
- **SessionEnd** - Runs when session ends

**Stop Hook Specifics**:
- Triggered when main Claude agent finishes responding
- Does NOT run if stopped due to user interrupt
- Cannot block session termination
- Receives JSON with `session_id`, `transcript_path`, `cwd`, `hook_event_name`
- Useful for cleanup, logging, session state saving

### Current .claude/settings.json Implementation

**File**: `.claude/settings.json`

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash|Write|Edit|MultiEdit",
        "hooks": [{"type": "command", "command": "script.py"}]
      }
    ],
    "Stop": [
      {
        "matcher": "*",
        "hooks": [{"type": "command", "command": "stop-session-notify.py"}]
      }
    ]
  }
}
```

**Critical Discovery**: This configuration is processed by the external Claude client, NOT by the Rust codex codebase. The hook scripts exist at `.codex/hooks/strategic/` but are not executed by the Rust code.

### Existing Rust Hook Infrastructure

**File**: `codex-rs/core/src/config.rs:1027-1051`

The Rust codebase has basic notification infrastructure:

```rust
fn maybe_notify(&self, notification: UserNotification) {
    let Some(notify_command) = &self.notify else { return; };
    if notify_command.is_empty() { return; }

    let mut command = std::process::Command::new(&notify_command[0]);
    if notify_command.len() > 1 {
        command.args(&notify_command[1..]);
    }
    command.arg(json);

    if let Err(e) = command.spawn() {
        warn!("failed to spawn notifier '{}': {e}", notify_command[0]);
    }
}
```

**Hook Execution Points**:
- Agent turn completion: `codex-rs/core/src/codex.rs:1881-1885`
- Session drop: `codex-rs/core/src/codex.rs:1054-1058`

### Configuration Patterns in Codex

**Primary Pattern**: TOML with Serde deserialization (`config.rs`)

```rust
#[derive(Deserialize, Debug, Clone, Default, PartialEq)]
pub struct ConfigToml {
    pub model: Option<String>,
    pub approval_policy: Option<AskForApproval>,
    pub mcp_servers: HashMap<String, McpServerConfig>,
}
```

**Project-Level Pattern**: From custom prompts (`custom_prompts.rs`)

```rust
// Load global configs first, then project-level from .codex/ in git root
if let Some(git_root) = crate::git_info::get_git_repo_root(project_cwd) {
    let project_prompt_dir = git_root.join(".codex/prompts");
    // Project configs override global ones
}
```

**Command Configuration Pattern**: From MCP servers

```rust
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct McpServerConfig {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: Option<HashMap<String, String>>,
}
```

## Code References

- `codex-rs/core/src/config.rs:1027-1051` - Basic notification infrastructure
- `codex-rs/core/src/codex.rs:1881-1885` - Agent turn completion hook point
- `codex-rs/core/src/codex.rs:1054-1058` - Session drop hook point
- `codex-rs/core/src/custom_prompts.rs:125-145` - Project-level config pattern
- `codex-rs/core/src/config_types.rs:85-92` - Command execution config pattern
- `.claude/settings.json:23-33` - Stop hook configuration example
- `.codex/hooks/strategic/stop-session-notify.py` - Stop hook implementation

## Architecture Insights

### Current State
1. **External Hook Processing**: .claude/settings.json hooks are processed by Claude client, not Rust codebase
2. **Basic Notification System**: Rust has simple notification command execution
3. **Infrastructure Exists**: Hook scripts exist at `.codex/hooks/strategic/` but unused by Rust
4. **Configuration Patterns Ready**: Established patterns for TOML config, project-level overrides, command execution

### Implementation Strategy

**Option 1: Extend Existing Notification System (Recommended)**
- Add hooks configuration to `ConfigToml` similar to `mcp_servers`
- Use TOML format: `.codex/config.toml` with `[hooks]` section
- Implement hook matching and execution in existing notification pipeline
- Support both global (`CODEX_HOME/config.toml`) and project-level (`.codex/config.toml`)

**Option 2: Separate Hooks Configuration**
- Create `.codex/hooks.toml` configuration file
- Implement dedicated hook processing system
- More separation but duplicates configuration patterns

### Proposed Configuration Format (Stop Hook Focus)

```toml
# In .codex/config.toml or $CODEX_HOME/config.toml
[hooks.Stop]
[[hooks.Stop.matchers]]
pattern = "*"  # Wildcard matches all sessions
command = "$CODEX_PROJECT_DIR/.codex/hooks/strategic/stop-session-notify.py"
timeout = 60  # Optional timeout in seconds
```

The Stop hook configuration is simpler since:
- No approval/deny decisions needed (fire-and-forget)
- Pattern matching is typically "*" (all sessions)
- Focus is on notification/logging, not control flow

### Stop Hook Processing Flow

1. **Configuration Loading**: Load hooks.Stop config from TOML during startup
2. **Event Triggering**: Hook into session completion point (`codex.rs:1881-1885`)
3. **Environment Setup**: Set `$CODEX_PROJECT_DIR` to git root or cwd
4. **Command Execution**: Execute hook script with JSON input via stdin
5. **Fire and Forget**: No response processing needed for Stop hooks

## Implementation Requirements

### Stop Hook Implementation Steps

1. **Add Hook Configuration Structure** (`config_types.rs`)
   - Create `HookConfig` struct with command, timeout fields
   - Create `StopHookConfig` struct with matchers array
   - Add `hooks: Option<HooksConfig>` to `ConfigToml`

2. **Extend Configuration Loading** (`config.rs`)
   - Parse `[hooks.Stop]` section from TOML
   - Support `$CODEX_PROJECT_DIR` variable expansion
   - Merge project-level and global configurations

3. **Implement Hook Execution** (`hooks.rs` - new file)
   - Create `execute_stop_hooks()` function
   - Set environment variables (CODEX_PROJECT_DIR, CODEX_HOME, CODEX_SESSION_ID)
   - Pass JSON input via stdin with session_id, cwd, transcript_path
   - Handle command timeout (default 60 seconds)

4. **Integration Point** (`codex.rs:1881-1885`)
   - Call `execute_stop_hooks()` after agent turn completion
   - Pass session context (id, cwd, transcript path if available)
   - Fire-and-forget execution (don't wait for response)

5. **Testing**
   - Test with existing `.codex/hooks/strategic/stop-session-notify.py`
   - Verify environment variable expansion
   - Test project-level vs global configuration precedence

## Related Research

- RESEARCH_0001: Configuration management patterns
- RESEARCH_0002: Project-level custom prompts (configuration precedence model)

## Decisions Made

1. **Configuration Format**: TOML (consistent with main config)
2. **Hook Response Format**: Not applicable for Stop hooks (fire-and-forget)
3. **Environment Variables**: Yes, support `$CODEX_PROJECT_DIR` for portable paths
4. **Migration Path**: Not addressing at this time
5. **Tool Integration**: Not addressing PreToolUse hooks in this phase

## Stop Hook JSON Input Format

The Stop hook will receive this JSON structure via stdin:
```json
{
  "hook_event_name": "Stop",
  "session_id": "unique-session-id",
  "cwd": "/path/to/project",
  "transcript_path": "/path/to/transcript.txt"  // Optional
}
```

This matches the format expected by existing hook scripts like `stop-session-notify.py`.

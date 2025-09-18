# Codex Hooks Implementation Plan

## Overview

Implement native hook execution in the codex-rs Rust codebase to support .codex/hooks/ functionality similar to Claude Code's .claude/hooks system. This plan focuses on extending the existing notification infrastructure to execute Stop hooks when sessions complete, using TOML configuration that follows established patterns in the codebase.

## Current State Analysis

**Existing Infrastructure:**
- Basic notification system exists at `codex-rs/core/src/codex.rs:1027-1051` with `maybe_notify()` function
- TOML configuration patterns established for complex structures like `mcp_servers`
- Project-level configuration discovery from `.codex/` directories already implemented
- Hook scripts exist at `.codex/hooks/strategic/` but are unused by Rust code

**What's Missing:**
- Hook configuration structure in TOML format
- Hook execution engine with JSON input/output
- Integration with session lifecycle events
- Environment variable expansion support

**Key Constraints:**
- Must follow existing configuration patterns (HashMap<String, ConfigStruct>)
- Must maintain fire-and-forget execution for Stop hooks
- Must support project-level configuration overrides
- Must be compatible with existing hook script interfaces

### Key Discoveries:

- Notification infrastructure ready for extension at `codex-rs/core/src/codex.rs:1027-1051`
- Configuration patterns well-established in `codex-rs/core/src/config_types.rs:85-92`
- Project-level config discovery pattern at `codex-rs/core/src/custom_prompts.rs:125-145`
- Hook execution points identified at `codex-rs/core/src/codex.rs:1881-1885`
- JSON interface documented in existing scripts at `.codex/hooks/strategic/stop-session-notify.py`

## Desired End State

Users can configure Stop hooks in their `.codex/config.toml` or global `$CODEX_HOME/config.toml` files that execute when sessions complete. The hooks receive session context via JSON stdin and can perform cleanup, logging, or notification tasks.

**Verification**: A configured Stop hook executes when a session completes, receives proper JSON input with session context, and existing `.codex/hooks/strategic/stop-session-notify.py` works without modification.

## What We're NOT Doing

- **Pre/Post Tool Use hooks** - Focusing only on Stop hooks initially
- **Hook approval/blocking mechanisms** - Stop hooks are fire-and-forget
- **Migration from .claude/settings.json** - That system remains external to Rust
- **Complex pattern matching** - Using simple wildcard matching for Stop hooks
- **Hook chaining or dependencies** - Each hook executes independently

## Implementation Approach

Extend the existing notification system by adding hooks configuration to the TOML structure and creating a hook execution engine that follows established command execution patterns. This leverages existing infrastructure rather than building parallel systems.

## Phase 1: Configuration Structure

### Overview

Add hooks configuration structures to the TOML system using existing patterns for command execution and configuration merging.

### Changes Required:

#### 1. Hook Configuration Types

**File**: `codex-rs/core/src/config_types.rs`
**Changes**: Add hook configuration structures following McpServerConfig pattern

```rust
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct HookConfig {
    pub command: String,

    #[serde(default)]
    pub args: Vec<String>,

    #[serde(default)]
    pub env: Option<HashMap<String, String>>,

    #[serde(default)]
    pub timeout_ms: Option<u64>,

    #[serde(default = "default_hook_enabled")]
    pub enabled: bool,
}

fn default_hook_enabled() -> bool {
    true
}

#[derive(Deserialize, Debug, Clone, PartialEq, Default)]
pub struct HooksConfig {
    #[serde(default)]
    pub stop: HashMap<String, HookConfig>,
}
```

#### 2. Main Configuration Integration

**File**: `codex-rs/core/src/config.rs`
**Changes**: Add hooks field to ConfigToml and Config structs

```rust
// In ConfigToml struct
#[serde(default)]
pub hooks: Option<HooksConfig>,

// In Config struct
pub hooks: HooksConfig,

// In Config::load_from_base_config_with_overrides
hooks: cfg.hooks.unwrap_or_default(),
```

### Success Criteria:

#### Automated Verification:

- [ ] Code compiles successfully: `cargo build -p codex-core`
- [ ] Type checking passes: `cargo check -p codex-core`
- [ ] Configuration loads without errors: Unit test for TOML parsing

#### Manual Verification:

- [ ] TOML configuration structure follows established patterns
- [ ] Hook configuration can be loaded from both global and project config files
- [ ] Configuration merging works as expected

---

## Phase 2: Stop Hook Implementation

### Overview

Implement Stop hook execution by extending the UserNotification enum and adding hook execution to session completion events.

### Changes Required:

#### 1. User Notification Extension

**File**: `codex-rs/core/src/user_notification.rs`
**Changes**: Add SessionStopped notification type

```rust
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub(crate) enum UserNotification {
    #[serde(rename_all = "kebab-case")]
    AgentTurnComplete {
        turn_id: String,
        input_messages: Vec<String>,
        last_assistant_message: Option<String>,
    },
    #[serde(rename_all = "kebab-case")]
    SessionStopped {
        session_id: String,
        cwd: String,
        transcript_path: Option<String>,
    },
}
```

#### 2. Session Integration

**File**: `codex-rs/core/src/codex.rs`
**Changes**: Add Stop hook notification to session completion at line 1881-1885

```rust
// Add after existing AgentTurnComplete notification
sess.maybe_notify_stop_hooks(UserNotification::SessionStopped {
    session_id: sess.conversation_id.to_string(),
    cwd: sess.cwd.to_string_lossy().to_string(),
    transcript_path: None, // TODO: Add transcript path if available
});
```

#### 3. Hook Execution Method

**File**: `codex-rs/core/src/codex.rs`
**Changes**: Add maybe_notify_stop_hooks method to Session impl

```rust
fn maybe_notify_stop_hooks(&self, notification: UserNotification) {
    for (hook_name, hook_config) in &self.config.hooks.stop {
        if !hook_config.enabled {
            continue;
        }

        self.execute_hook(hook_name, hook_config, &notification);
    }
}

fn execute_hook(&self, hook_name: &str, hook_config: &HookConfig, notification: &UserNotification) {
    let Ok(json) = serde_json::to_string(notification) else {
        error!("failed to serialize hook notification for {hook_name}");
        return;
    };

    let mut command = std::process::Command::new(&hook_config.command);
    command.args(&hook_config.args);
    command.stdin(std::process::Stdio::piped());

    // Set environment variables
    if let Some(env) = &hook_config.env {
        command.envs(env);
    }

    // Set CODEX_PROJECT_DIR for path expansion
    if let Some(git_root) = crate::git_info::get_git_repo_root(&self.cwd) {
        command.env("CODEX_PROJECT_DIR", git_root);
    }

    // Fire-and-forget execution with JSON input via stdin
    if let Ok(mut child) = command.spawn() {
        if let Some(stdin) = child.stdin.take() {
            let _ = std::io::Write::write_all(&mut std::io::BufWriter::new(stdin), json.as_bytes());
        }
    } else {
        warn!("failed to spawn hook '{}': {}", hook_name, hook_config.command);
    }
}
```

### Success Criteria:

#### Automated Verification:

- [ ] Code compiles successfully: `cargo build -p codex-core`
- [ ] UserNotification serializes correctly: Unit test for JSON output
- [ ] Hook execution doesn't block session completion: Integration test

#### Manual Verification:

- [ ] Stop hooks execute when sessions complete
- [ ] JSON input matches expected format from existing scripts
- [ ] Hook execution is fire-and-forget (doesn't wait for completion)
- [ ] Environment variables are set correctly

---

## Phase 3: Hook Execution Engine

### Overview

Implement robust hook command execution with timeout handling, environment variable expansion, and proper error handling.

### Changes Required:

#### 1. Environment Variable Expansion

**File**: `codex-rs/core/src/codex.rs`
**Changes**: Add command and args expansion for $CODEX_PROJECT_DIR

```rust
fn expand_hook_command(&self, command: &str) -> String {
    let git_root = crate::git_info::get_git_repo_root(&self.cwd)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| self.cwd.to_string_lossy().to_string());

    command.replace("$CODEX_PROJECT_DIR", &git_root)
}

fn expand_hook_args(&self, args: &[String]) -> Vec<String> {
    let git_root = crate::git_info::get_git_repo_root(&self.cwd)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| self.cwd.to_string_lossy().to_string());

    args.iter()
        .map(|arg| arg.replace("$CODEX_PROJECT_DIR", &git_root))
        .collect()
}
```

#### 2. Timeout Handling

**File**: `codex-rs/core/src/codex.rs`
**Changes**: Add timeout support to hook execution

```rust
fn execute_hook_with_timeout(&self, hook_config: &HookConfig, json: &str) {
    let timeout = Duration::from_millis(hook_config.timeout_ms.unwrap_or(60000)); // Default 60s

    let expanded_command = self.expand_hook_command(&hook_config.command);
    let expanded_args = self.expand_hook_args(&hook_config.args);

    let mut command = std::process::Command::new(expanded_command);
    command.args(expanded_args);
    command.stdin(std::process::Stdio::piped());
    command.stdout(std::process::Stdio::null());
    command.stderr(std::process::Stdio::null());

    // Set environment variables
    if let Some(env) = &hook_config.env {
        command.envs(env);
    }

    // Spawn with timeout handling
    if let Ok(mut child) = command.spawn() {
        if let Some(stdin) = child.stdin.take() {
            let _ = std::io::Write::write_all(&mut std::io::BufWriter::new(stdin), json.as_bytes());
        }

        // Fire-and-forget with timeout (don't wait)
        std::thread::spawn(move || {
            let _ = child.wait_timeout(timeout);
        });
    }
}
```

### Success Criteria:

#### Automated Verification:

- [ ] Environment variable expansion works: Unit test for $CODEX_PROJECT_DIR
- [ ] Timeout handling doesn't block execution: Unit test
- [ ] Command expansion handles edge cases: Unit test

#### Manual Verification:

- [ ] Hook commands execute with expanded paths
- [ ] Timeout prevents hung processes
- [ ] Error handling logs appropriately without crashing

---

## Phase 4: Project-Level Configuration Support

### Overview

Enable project-level .codex/config.toml files to override global hook configurations using the established pattern from custom prompts.

### Changes Required:

#### 1. Configuration Discovery

**File**: `codex-rs/core/src/config.rs`
**Changes**: Add hook discovery following custom prompts pattern

```rust
fn discover_hooks_with_project_support(
    global_hooks: &HooksConfig,
    project_cwd: &Path,
) -> HooksConfig {
    let mut hooks = global_hooks.clone();

    // Project-level hooks override global ones
    if let Some(git_root) = crate::git_info::get_git_repo_root(project_cwd) {
        let project_config_path = git_root.join(".codex/config.toml");
        if let Ok(contents) = std::fs::read_to_string(&project_config_path) {
            if let Ok(project_config) = toml::from_str::<ConfigToml>(&contents) {
                if let Some(project_hooks) = project_config.hooks {
                    // Merge project hooks with global ones
                    for (name, config) in project_hooks.stop {
                        hooks.stop.insert(name, config);
                    }
                }
            }
        }
    }

    hooks
}
```

#### 2. Integration with Config Loading

**File**: `codex-rs/core/src/config.rs`
**Changes**: Call hook discovery in main config loading

```rust
// In Config::load_from_base_config_with_overrides
let hooks = discover_hooks_with_project_support(
    &cfg.hooks.unwrap_or_default(),
    &resolved_cwd
);
```

### Success Criteria:

#### Automated Verification:

- [ ] Project-level config overrides global config: Integration test
- [ ] Configuration merging works correctly: Unit test
- [ ] Git root discovery works: Unit test

#### Manual Verification:

- [ ] .codex/config.toml hooks override global settings
- [ ] Project-specific hooks execute correctly
- [ ] Configuration precedence follows expected pattern

---

## Phase 5: Testing & Integration

### Overview

Verify the implementation works with existing hook scripts and provide comprehensive testing coverage.

### Changes Required:

#### 1. Integration Testing

**File**: `codex-rs/core/tests/hooks_integration_test.rs` (new file)
**Changes**: Create integration tests for hook execution

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_stop_hook_execution() {
        // Test that Stop hooks execute with correct JSON format
    }

    #[test]
    fn test_project_level_hook_override() {
        // Test project-level configuration override
    }

    #[test]
    fn test_environment_variable_expansion() {
        // Test $CODEX_PROJECT_DIR expansion
    }
}
```

#### 2. Example Configuration

**File**: `codex-rs/example-configs/hooks-config.toml` (new file)
**Changes**: Provide example hook configuration

```toml
[hooks.stop.session-notify]
command = "$CODEX_PROJECT_DIR/.codex/hooks/strategic/stop-session-notify.py"
timeout_ms = 30000
enabled = true

[hooks.stop.cleanup]
command = "echo"
args = ["Session completed: ${session_id}"]
enabled = false
```

### Success Criteria:

#### Automated Verification:

- [ ] All unit tests pass: `cargo test -p codex-core`
- [ ] Integration tests pass: `cargo test -p codex-core hooks_integration`
- [ ] Example configuration loads: `cargo test -p codex-core config_loading`
- [ ] Linting passes: `just fix -p codex-core`

#### Manual Verification:

- [ ] Existing `.codex/hooks/strategic/stop-session-notify.py` works without modification
- [ ] Hook execution appears in logs when sessions complete
- [ ] Project-level hooks override global configuration as expected
- [ ] JSON input format matches existing script expectations

---

## Performance Considerations

- **Fire-and-forget execution**: Hooks don't block session completion
- **Timeout protection**: Default 60-second timeout prevents hung processes
- **Minimal overhead**: Hook execution only occurs on session completion events
- **Configuration caching**: Hook configuration loaded once during session startup

## Migration Notes

- **Backward compatibility**: Existing `.claude/settings.json` hooks continue to work via Claude client
- **Script compatibility**: Existing hook scripts work without modification
- **Gradual adoption**: Users can migrate hooks from `.claude/settings.json` to `.codex/config.toml` at their own pace

## References

- Related research: `.strategic-claude-basic/research/RESEARCH_0005_18-09-2025_thu_codex-hooks-implementation.md`
- Configuration pattern: `codex-rs/core/src/config_types.rs:85-92`
- Notification system: `codex-rs/core/src/codex.rs:1027-1051`
- Project-level pattern: `codex-rs/core/src/custom_prompts.rs:125-145`
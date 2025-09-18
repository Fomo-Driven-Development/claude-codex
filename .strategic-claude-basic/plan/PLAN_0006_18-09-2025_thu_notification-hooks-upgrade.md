---
date: 2025-09-18T15:01:51-05:00
git_commit: 387f13900b50dc3a6a8116e332fad82d7b81b6a6
branch: claude-codex
repository: claude-codex
topic: "Notification Hooks Upgrade Implementation Plan"
tags: [plan, implementation, hooks, notifications, user-notification, tui, core]
status: draft
last_updated: 2025-09-18
---

# Notification Hooks Upgrade Implementation Plan

## Overview

Implement notification hooks in the codex hook system to trigger external commands when:
1. **Tool Permission Requests**: Agent needs user approval to execute tools (after modal display)
2. **Idle Timeout**: User input has been idle for 60 seconds (fixed/configurable)

This extends the existing event-driven hook architecture with two new notification events while leveraging the proven execution infrastructure.

## Current State Analysis

**Existing Hook Infrastructure**:
- Event-driven architecture with `UserNotification` enum supports `AgentTurnComplete` and `AgentTurnStopped`
- Robust execution pipeline with JSON communication, timeout handling, variable expansion
- Project-level configuration discovery (`.codex/config.toml` overrides global)
- Working examples in `.strategic-claude-basic/core/hooks/`

**Current Notification Infrastructure**:
- TUI approval modals for exec (`chatwidget.rs:538-550`) and patch requests (`chatwidget.rs:555-577`)
- OSC 9 desktop notifications for terminal focus detection
- Existing notification types: `AgentTurnComplete`, `ExecApprovalRequested`, `EditApprovalRequested`

**No Architecture Decision Records** found - implementation will proceed without ADR constraints.

## Desired End State

After implementation completion:

1. **Hook Configuration**: New `[hooks.notification]` section in config supports permission and idle hooks
2. **Event Triggers**: Notification hooks trigger after approval modals display and on 60-second idle timeout
3. **JSON Payload**: Hooks receive structured notification data via stdin with session context
4. **Backward Compatibility**: Existing stop hooks continue working unchanged
5. **Configuration**: Idle timeout configurable but defaults to 60 seconds per Claude Code specification

### Key Discoveries:

- Hook execution infrastructure in `codex.rs:1055-1162` can be extended without modification
- TUI approval handlers already have perfect integration points after modal display
- Existing `tokio::time` patterns in frame scheduling provide idle timeout foundation
- Project-level hook discovery automatically works for notification hooks

## What We're NOT Doing

- Modifying existing stop hook behavior or configuration format
- Adding hook execution before approval modal display (confirmed: after only)
- Creating separate permission types for exec vs patch (confirmed: single notification type)
- Adding user warnings for hook failures (confirmed: fire-and-forget only)
- Breaking changes to existing `UserNotification` serialization format

## Implementation Approach

**Three-Phase Approach**:
1. **Core Extensions**: Extend notification types and hook configuration structures
2. **TUI Integration**: Add trigger points for permission requests and idle timeout detection
3. **Configuration & Examples**: Update config templates and provide working examples

**Key Design Decisions** (per user requirements):
- Hooks execute **after** approval modal display
- **Single notification hook type** with event details in JSON payload
- **60-second fixed idle timeout** with optional configuration override
- **Fire-and-forget execution** - failures don't affect approval flow

## Phase 1: Core Notification Infrastructure

### Overview

Extend the core notification system to support tool permission requests and idle timeout events. This phase focuses on the foundational data structures and hook execution integration.

### Changes Required:

#### 1. UserNotification Enum Extension

**File**: `codex-rs/core/src/user_notification.rs`
**Changes**: Add two new notification variants for permission requests and idle timeout

```rust
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub(crate) enum UserNotification {
    // ... existing variants

    #[serde(rename_all = "kebab-case")]
    ToolPermissionRequest {
        session_id: String,
        cwd: String,
        approval_type: String, // "exec" | "patch"
        tool_name: String,     // "Bash" | "ApplyPatch"
        command: Option<String>,
        changes: Option<Vec<String>>,
        reason: Option<String>,
    },

    #[serde(rename_all = "kebab-case")]
    PromptIdleTimeout {
        session_id: String,
        cwd: String,
        idle_duration_seconds: u64,
    },
}
```

#### 2. Hook Configuration Extension

**File**: `codex-rs/core/src/config_types.rs`
**Changes**: Add notification hooks to `HooksConfig` structure

```rust
#[derive(Deserialize, Debug, Clone, PartialEq, Default)]
pub struct HooksConfig {
    #[serde(default)]
    pub stop: HashMap<String, HookConfig>,

    #[serde(default)]
    pub notification: HashMap<String, HookConfig>,
}
```

#### 3. Hook Execution Extension

**File**: `codex-rs/core/src/codex.rs`
**Changes**: Add notification hook execution method alongside existing stop hooks

```rust
// Add after maybe_notify_stop_hooks (around line 1063)
fn maybe_notify_notification_hooks(
    &self,
    hooks_config: &crate::config_types::HooksConfig,
    turn_context: &TurnContext,
    notification: UserNotification,
) {
    for (hook_name, hook_config) in &hooks_config.notification {
        if !hook_config.enabled {
            continue;
        }
        self.execute_hook(hook_name, hook_config, &notification, turn_context);
    }
}
```

#### 4. Idle Timeout Configuration

**File**: `codex-rs/core/src/config_types.rs`
**Changes**: Add idle timeout configuration to main config

```rust
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct Config {
    // ... existing fields

    /// Idle timeout in seconds. Defaults to 60, set to 0 to disable.
    #[serde(default = "default_idle_timeout")]
    pub idle_timeout_seconds: u64,
}

fn default_idle_timeout() -> u64 {
    60
}
```

### Success Criteria:

#### Automated Verification:

- [ ] Code compiles successfully: `cargo build -p codex-core`
- [ ] Type checking passes in core: `cargo check -p codex-core`
- [ ] Unit tests pass: `cargo test -p codex-core`
- [ ] Hook configuration deserialization works with new notification section

#### Manual Verification:

- [ ] `UserNotification` enum serializes new variants to correct JSON format
- [ ] Hook configuration loading includes notification hooks in discovery
- [ ] New notification hook execution method integrated with existing infrastructure
- [ ] No breaking changes to existing stop hook functionality

---

## Phase 2: TUI Integration and Event Triggers

### Overview

Integrate notification hook triggers into the TUI layer at the identified integration points: after approval modal display and for idle timeout detection.

### Changes Required:

#### 1. App Event Extension

**File**: `codex-rs/tui/src/app_event.rs`
**Changes**: Add notification hook event for communication between TUI and core

```rust
#[derive(Debug)]
pub(crate) enum AppEvent {
    // ... existing variants

    /// Execute notification hooks with the provided data
    ExecuteNotificationHooks {
        notification: codex_core::user_notification::UserNotification,
        turn_context: TurnContext,
    },
}
```

#### 2. Approval Request Hook Triggers

**File**: `codex-rs/tui/src/chatwidget.rs`
**Changes**: Add hook triggers after approval modal display

```rust
// In handle_exec_approval_now (after line 544)
self.notify(Notification::ExecApprovalRequested { command });

// NEW: Trigger notification hook
if let Some(conversation_id) = &self.conversation_id {
    let notification = UserNotification::ToolPermissionRequest {
        session_id: conversation_id.to_string(),
        cwd: self.config.cwd.to_string_lossy().to_string(),
        approval_type: "exec".to_string(),
        tool_name: "Bash".to_string(),
        command: Some(ev.command.join(" ")),
        changes: None,
        reason: ev.reason.clone(),
    };
    self.trigger_notification_hook(notification);
}

// In handle_apply_patch_approval_now (after line 577)
self.notify(Notification::EditApprovalRequested { cwd, changes });

// NEW: Trigger notification hook
if let Some(conversation_id) = &self.conversation_id {
    let notification = UserNotification::ToolPermissionRequest {
        session_id: conversation_id.to_string(),
        cwd: self.config.cwd.to_string_lossy().to_string(),
        approval_type: "patch".to_string(),
        tool_name: "ApplyPatch".to_string(),
        command: None,
        changes: Some(ev.changes.keys().map(|p| p.to_string_lossy().to_string()).collect()),
        reason: ev.reason.clone(),
    };
    self.trigger_notification_hook(notification);
}
```

#### 3. Idle Timeout Detection

**File**: `codex-rs/tui/src/app.rs`
**Changes**: Add idle timeout detection to main event loop

```rust
// Add to App struct (around line 60)
last_user_input: Arc<AtomicU64>, // epoch milliseconds
idle_timeout_task: Option<tokio::task::JoinHandle<()>>,

// In run() method, modify event loop (around line 147)
let mut last_input_time = std::time::Instant::now();

loop {
    tokio::select! {
        // ... existing event handling

        // NEW: Idle timeout detection
        _ = tokio::time::sleep(Duration::from_secs(60)), if config.idle_timeout_seconds > 0 => {
            if last_input_time.elapsed().as_secs() >= config.idle_timeout_seconds {
                self.handle_idle_timeout().await?;
                last_input_time = std::time::Instant::now(); // Reset to avoid repeated notifications
            }
        }
    }
}

// Add helper method
async fn handle_idle_timeout(&mut self) -> Result<()> {
    if let Some(session_id) = self.chat_widget.conversation_id() {
        let notification = UserNotification::PromptIdleTimeout {
            session_id: session_id.to_string(),
            cwd: self.config.cwd.to_string_lossy().to_string(),
            idle_duration_seconds: self.config.idle_timeout_seconds,
        };

        // Send to core for hook execution
        self.submit_op(Op::ExecuteNotificationHooks { notification }).await?;
    }
    Ok(())
}
```

#### 4. Hook Trigger Helper

**File**: `codex-rs/tui/src/chatwidget.rs`
**Changes**: Add notification hook trigger helper method

```rust
impl ChatWidget {
    fn trigger_notification_hook(&mut self, notification: UserNotification) {
        // Create minimal turn context for hooks
        let turn_context = TurnContext {
            cwd: self.config.cwd.clone(),
            // ... other required fields
        };

        let _ = self.app_event_tx.send(AppEvent::ExecuteNotificationHooks {
            notification,
            turn_context,
        });
    }
}
```

### Success Criteria:

#### Automated Verification:

- [ ] TUI compiles successfully: `cargo build -p codex-tui`
- [ ] Type checking passes: `cargo check -p codex-tui`
- [ ] Integration tests pass: `cargo test -p codex-tui`
- [ ] Event communication works between TUI and core layers

#### Manual Verification:

- [ ] Approval modals display normally with hook triggers working in background
- [ ] Idle timeout detection activates after 60 seconds of inactivity
- [ ] User input resets idle timer correctly
- [ ] Hook triggers fire after (not before) approval modal display
- [ ] No impact on existing approval workflow or user experience

---

## Phase 3: Configuration & Integration

### Overview

Complete the integration by adding configuration examples, updating templates, and ensuring project-level hook discovery works correctly for notification hooks.

### Changes Required:

#### 1. Configuration Template Updates

**File**: `codex-rs/example-configs/hooks-config.toml`
**Changes**: Add notification hook examples

```toml
# Notification hooks - triggered on permission requests and idle timeout
[hooks.notification.tool-permission]
command = "notify-send"
args = ["Claude Code", "Permission needed: {{approval_type}}"]
timeout_ms = 5000
enabled = true

[hooks.notification.idle-timeout]
command = "$CODEX_PROJECT_DIR/.codex/hooks/idle-notify.sh"
timeout_ms = 10000
enabled = true

[hooks.notification.general-notify]
command = "$CODEX_PROJECT_DIR/.strategic-claude-basic/core/hooks/notification-hook.py"
timeout_ms = 30000
enabled = true
```

#### 2. Hook Discovery Integration

**File**: `codex-rs/core/src/config.rs`
**Changes**: Ensure notification hooks are included in project-level discovery

```rust
// In discover_hooks_with_project_support (around line 1996)
if let Some(project_hooks) = project_config.hooks {
    // Merge stop hooks
    for (name, config) in project_hooks.stop {
        hooks.stop.insert(name, config);
    }

    // NEW: Merge notification hooks
    for (name, config) in project_hooks.notification {
        hooks.notification.insert(name, config);
    }
}
```

#### 3. Op Integration for Hook Execution

**File**: `codex-rs/core/src/codex.rs`
**Changes**: Add operation handler for notification hook execution

```rust
// Add to Op enum handling (around line 800)
Op::ExecuteNotificationHooks { notification } => {
    if let Some(hooks_config) = &config.hooks {
        self.maybe_notify_notification_hooks(hooks_config, &turn_context, notification);
    }
}
```

#### 4. Working Hook Example

**File**: `.strategic-claude-basic/core/hooks/notification-hook.py`
**Changes**: Update existing notification hook to handle new event types

```python
def handle_tool_permission_request(hook_data):
    """Handle tool permission request notifications."""
    approval_type = hook_data.get("approval_type", "unknown")
    tool_name = hook_data.get("tool_name", "unknown")

    title = f"{PROJECT_TITLE}: Permission Required"
    message = f"Claude needs permission to use {tool_name}"

    if approval_type == "exec":
        command = hook_data.get("command", "")
        message += f"\nCommand: {command}"
    elif approval_type == "patch":
        changes = hook_data.get("changes", [])
        message += f"\nFiles: {', '.join(changes[:3])}"
        if len(changes) > 3:
            message += f" (+{len(changes)-3} more)"

    send_notification(message, title, priority="high", tags=["warning", project_tag])

def handle_idle_timeout(hook_data):
    """Handle idle timeout notifications."""
    duration = hook_data.get("idle_duration_seconds", 60)

    title = f"{PROJECT_TITLE}: Session Idle"
    message = f"Claude session idle for {duration}s"

    send_notification(message, title, priority="default", tags=["clock", project_tag])
```

### Success Criteria:

#### Automated Verification:

- [ ] Full system builds successfully: `cargo build --all-features`
- [ ] All tests pass: `cargo test --all-features`
- [ ] Configuration parsing works with notification hooks: `cargo test config`
- [ ] Hook discovery includes notification hooks in project configs

#### Manual Verification:

- [ ] Example configuration loads without errors
- [ ] Project-level notification hooks override global ones correctly
- [ ] Working notification hook example processes new event types
- [ ] End-to-end flow works: approval request → hook trigger → external notification
- [ ] Idle timeout detection works with configurable duration

---

## Test Plan Reference

**Related Test Plan**: `.strategic-claude-basic/plan/TEST_0006_18-09-2025_thu_notification-hooks-upgrade.md`

Testing will validate notification hook triggers, JSON payload structure, configuration loading, timeout behavior, and integration with existing approval workflows. Detailed test scenarios, automation strategy, and validation criteria are covered in the dedicated test plan.

## Performance Considerations

- **Idle Timeout**: Uses efficient `tokio::select!` with 60-second sleep intervals, minimal CPU impact
- **Hook Execution**: Fire-and-forget execution prevents blocking approval workflows
- **Memory**: Notification payloads are small JSON structures, negligible memory impact
- **Event Loop**: Hook triggers use existing event system, no additional overhead

## Migration Notes

- **Backward Compatibility**: Existing stop hooks continue working unchanged
- **Configuration**: New `[hooks.notification]` section is optional, defaults to empty
- **Graceful Degradation**: If notification hooks fail, approval workflow continues normally
- **Project Discovery**: Existing project-level hook discovery automatically includes notification hooks

## References

- Related research: `.strategic-claude-basic/research/RESEARCH_0006_18-09-2025_thu_notification-hooks-upgrade.md`
- Existing hook patterns: `codex-rs/core/src/codex.rs:1055-1162`
- TUI approval handling: `codex-rs/tui/src/chatwidget.rs:538-577`
- Configuration discovery: `codex-rs/core/src/config.rs:1976-1996`
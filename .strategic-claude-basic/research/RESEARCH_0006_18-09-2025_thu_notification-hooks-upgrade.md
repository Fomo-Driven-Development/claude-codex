---
date: 2025-09-18T14:48:02-05:00
git_commit: 6051751b4390427e51b18fa9336260475005635f
branch: claude-codex
repository: claude-codex
topic: "Upgrading the new hook system to add the Notification hook to codex"
tags: [research, codebase, hooks, notifications, user-notification, tui, core]
status: complete
last_updated: 2025-09-18
---

# Research: Upgrading the new hook system to add the Notification hook to codex

**Date**: 2025-09-18T14:48:02-05:00
**Git Commit**: 6051751b4390427e51b18fa9336260475005635f
**Branch**: claude-codex
**Repository**: claude-codex

## Research Question

How to upgrade the new hook system to add the Notification hook to codex, specifically to trigger when:
1. Claude needs permission to use a tool (e.g., "Claude needs your permission to use Bash")
2. The prompt input has been idle for at least 60 seconds (e.g., "Claude is waiting for your input")

Reference: https://docs.claude.com/en/docs/claude-code/hooks#notification

## Summary

The codex hook system currently supports **stop hooks** (triggered when agent turns complete) with a robust execution infrastructure. Adding notification hooks requires extending the existing architecture to support two new event types: tool permission requests and idle timeout notifications. The implementation involves:

1. **Extending UserNotification enum** with `ToolPermissionRequest` and `PromptIdleTimeout` variants
2. **Adding notification hook configuration** to the existing `HooksConfig` structure
3. **Integrating trigger points** in the TUI layer where approval requests occur and idle timeouts are detected
4. **Leveraging existing hook execution** infrastructure for command execution, variable expansion, and timeout handling

## Detailed Findings

### Current Hook System Architecture

The hook system is built around the `UserNotification` enum in `codex-rs/core/src/user_notification.rs:8-27` with two event types:
- `AgentTurnComplete` - When agent finishes a turn
- `AgentTurnStopped` - When session stops (for hooks)

**Hook Configuration Structure** (`codex-rs/core/src/config_types.rs:28-53`):
```rust
pub struct HookConfig {
    pub command: String,
    pub args: Vec<String>,
    pub env: Option<HashMap<String, String>>,
    pub timeout_ms: Option<u64>,
    pub enabled: bool,
}

pub struct HooksConfig {
    pub stop: HashMap<String, HookConfig>,
}
```

**Hook Execution Infrastructure** (`codex-rs/core/src/codex.rs:1055-1162`):
- JSON serialization of event data passed via stdin
- Environment variable expansion (`$CODEX_PROJECT_DIR`)
- Timeout handling (default 60 seconds)
- Fire-and-forget execution with proper cleanup

### Notification Infrastructure Analysis

The codebase has comprehensive notification infrastructure across multiple layers:

**Permission Request Notifications** (`codex-rs/tui/src/chatwidget.rs`):
- `on_exec_approval_request()` - Lines 333-340: Command execution approval
- `on_apply_patch_approval_request()` - Lines 342-349: File modification approval
- `handle_exec_approval_now()` - Lines 538-550: Displays command approval modal
- `handle_apply_patch_approval_now()` - Lines 555-577: Displays patch approval modal

**Desktop Notification System** (`codex-rs/tui/src/tui.rs`):
- OSC 9 escape sequence implementation for terminal notifications
- `PostNotification` command - Lines 564-581
- Terminal focus detection for notification triggering

**Notification Types** (`codex-rs/tui/src/chatwidget.rs:1485-1489`):
```rust
enum Notification {
    AgentTurnComplete,
    ExecApprovalRequested,
    EditApprovalRequested,
}
```

### Integration Points for Notification Hooks

**1. UserNotification Enum Extension** (`codex-rs/core/src/user_notification.rs`):
```rust
pub(crate) enum UserNotification {
    // Existing variants...

    #[serde(rename_all = "kebab-case")]
    ToolPermissionRequest {
        tool_name: String,
        command: String,
        reason: Option<String>,
        session_id: String,
        cwd: String,
    },

    #[serde(rename_all = "kebab-case")]
    PromptIdleTimeout {
        session_id: String,
        cwd: String,
        idle_duration_seconds: u64,
    },
}
```

**2. Hook Configuration Extension** (`codex-rs/core/src/config_types.rs:48-52`):
```rust
pub struct HooksConfig {
    pub stop: HashMap<String, HookConfig>,
    pub notification: HashMap<String, HookConfig>,  // NEW
}
```

**3. Tool Permission Hook Triggers** (`codex-rs/tui/src/chatwidget.rs`):
- Line 544: In `handle_exec_approval_now()` - Trigger hook for command approval
- Lines 574-577: In `handle_apply_patch_approval_now()` - Trigger hook for patch approval

**4. Idle Timeout Detection** (`codex-rs/tui/src/app.rs`):
- Integration needed in main event loop around lines 65-200
- 60-second idle timeout detection with reset on user input

**5. Hook Execution Extension** (`codex-rs/core/src/codex.rs`):
- Add `maybe_notify_notification_hooks()` method alongside existing `maybe_notify_stop_hooks()`
- Leverage existing `execute_hook_with_timeout()` infrastructure

### Existing Hook Examples

The system includes several working hook implementations:

**Strategic Claude Hooks** (`.strategic-claude-basic/core/hooks/`):
- `notification-hook.py` - General notification handling with ntfy integration
- `stop-session-notify.py` - Session completion notifications
- `notifications.py` - Shared notification utilities

**Hook Configuration Example** (`codex-rs/example-configs/hooks-config.toml`):
```toml
[hooks.stop.session-notify]
command = "$CODEX_PROJECT_DIR/.codex/hooks/strategic/stop-session-notify.py"
timeout_ms = 30000
enabled = true
```

## Code References

- `codex-rs/core/src/user_notification.rs:8-27` - UserNotification enum definition
- `codex-rs/core/src/config_types.rs:28-53` - Hook configuration structures
- `codex-rs/core/src/codex.rs:1055-1162` - Hook execution infrastructure
- `codex-rs/tui/src/chatwidget.rs:333-349` - Permission request handling
- `codex-rs/tui/src/chatwidget.rs:538-577` - Approval modal implementations
- `codex-rs/tui/src/tui.rs:224-233` - Desktop notification system
- `codex-rs/tui/src/app.rs:65-200` - Main event loop integration point
- `.strategic-claude-basic/core/hooks/notification-hook.py` - Example notification hook
- `codex-rs/example-configs/hooks-config.toml` - Hook configuration example

## Architecture Insights

**Event-Driven Design**: The hook system follows an event-driven pattern where lifecycle events trigger hook execution. This design is extensible and can accommodate notification hooks alongside existing stop hooks.

**JSON-Based Communication**: Hook data is serialized to JSON and passed via stdin, providing a language-agnostic interface for hook implementations.

**Project-Level Configuration**: The system supports both global and project-level hook configurations with automatic discovery in `.codex/config.toml`.

**Environment Variable Expansion**: Built-in support for `$CODEX_PROJECT_DIR` expansion enables portable hook configurations across different project structures.

**Timeout and Cleanup**: Robust process management with configurable timeouts and proper cleanup prevents hanging processes.

**Terminal Integration**: Deep integration with terminal notification systems (OSC 9) provides immediate user feedback when the terminal is unfocused.

## Related Research

- [RESEARCH_0005_18-09-2025_thu_codex-hooks-implementation.md](RESEARCH_0005_18-09-2025_thu_codex-hooks-implementation.md) - Original hooks implementation research

## Open Questions

1. **Hook Priority**: Should notification hooks execute before or after approval modal display?
2. **Idle Timeout Granularity**: Is 60 seconds appropriate, or should it be configurable?
3. **Hook Failure Handling**: How should notification hook failures affect the user approval flow?
4. **Event Deduplication**: Should multiple rapid permission requests trigger multiple hooks or be batched?
5. **Cross-Platform Compatibility**: How should notification hooks work across different terminal environments?
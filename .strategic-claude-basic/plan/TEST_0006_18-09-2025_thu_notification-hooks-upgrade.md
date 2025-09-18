---
date: 2025-09-18T15:01:51-05:00
git_commit: 387f13900b50dc3a6a8116e332fad82d7b81b6a6
branch: claude-codex
repository: claude-codex
topic: "Notification Hooks Upgrade Test Plan"
tags: [test, plan, hooks, notifications, validation, quality-assurance]
status: draft
last_updated: 2025-09-18
---

# Notification Hooks Upgrade Test Plan

## Overview

Comprehensive testing strategy for notification hooks implementation that validates trigger behavior, JSON payload structure, configuration handling, and integration with existing approval workflows. Testing ensures robustness of the fire-and-forget execution model and proper event handling.

## Implementation Plan Reference

**Related Implementation Plan**: `.strategic-claude-basic/plan/PLAN_0006_18-09-2025_thu_notification-hooks-upgrade.md`

The implementation plan covers extending the hook system with notification events for tool permission requests and idle timeout detection, leveraging existing hook execution infrastructure.

## Current Test Coverage Analysis

**Existing Hook Tests**:
- `codex-rs/core/src/user_notification.rs:33-64` - UserNotification serialization tests
- Hook configuration parsing tests in core module
- Limited integration testing for hook execution

**Coverage Gaps**:
- No tests for notification-specific hook behavior
- No idle timeout detection tests
- No TUI layer notification trigger tests
- Missing integration tests for approval workflow with hooks

**Testing Infrastructure**:
- Rust: `cargo test` with unit and integration test support
- Existing snapshot testing with `insta` for TUI validation
- Mock/stub patterns established for external dependencies

## Test Strategy

### Test Types Required:

- **Unit Tests**: UserNotification variants, hook configuration parsing, event serialization
- **Integration Tests**: TUI → Core hook trigger flow, configuration discovery, hook execution
- **End-to-End Tests**: Complete approval workflows with hook execution
- **Performance Tests**: Idle timeout accuracy, hook execution timing
- **Security Tests**: Hook configuration validation, JSON payload sanitization

### Testing Approach:

**Layered Testing Strategy**:
1. **Core Layer**: Validate notification types, hook execution, configuration
2. **TUI Layer**: Test trigger points, event handling, timeout detection
3. **Integration**: End-to-end workflows with real hook execution
4. **Behavioral**: User scenarios and edge case handling

## What We're NOT Testing

- External notification service reliability (ntfy, notify-send, etc.)
- OS-specific desktop notification behavior
- Network-dependent hook execution
- Performance under extreme load (>1000 hooks per minute)
- Concurrent hook execution stress testing

## Phase 1: Core Notification System Tests

### Overview

Validate the foundational notification types, hook configuration structures, and execution infrastructure extensions.

### Test Coverage Requirements:

#### 1. UserNotification Enum Validation

**Files Under Test**: `codex-rs/core/src/user_notification.rs`
**Test File**: `codex-rs/core/src/user_notification.rs` (test module)

**Test Cases**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_permission_request_serialization() {
        let notification = UserNotification::ToolPermissionRequest {
            session_id: "session123".to_string(),
            cwd: "/home/user/project".to_string(),
            approval_type: "exec".to_string(),
            tool_name: "Bash".to_string(),
            command: Some("ls -la".to_string()),
            changes: None,
            reason: Some("List files for analysis".to_string()),
        };

        let json = serde_json::to_string(&notification).unwrap();
        assert_eq!(
            json,
            r#"{"type":"tool-permission-request","session-id":"session123","cwd":"/home/user/project","approval-type":"exec","tool-name":"Bash","command":"ls -la","changes":null,"reason":"List files for analysis"}"#
        );
    }

    #[test]
    fn test_prompt_idle_timeout_serialization() {
        let notification = UserNotification::PromptIdleTimeout {
            session_id: "session456".to_string(),
            cwd: "/home/user/project".to_string(),
            idle_duration_seconds: 60,
        };

        let json = serde_json::to_string(&notification).unwrap();
        assert_eq!(
            json,
            r#"{"type":"prompt-idle-timeout","session-id":"session456","cwd":"/home/user/project","idle-duration-seconds":60}"#
        );
    }

    #[test]
    fn test_patch_approval_notification() {
        let notification = UserNotification::ToolPermissionRequest {
            session_id: "session789".to_string(),
            cwd: "/home/user/project".to_string(),
            approval_type: "patch".to_string(),
            tool_name: "ApplyPatch".to_string(),
            command: None,
            changes: Some(vec!["src/main.rs".to_string(), "README.md".to_string()]),
            reason: Some("Update application logic".to_string()),
        };

        let serialized = serde_json::to_string(&notification).unwrap();
        let deserialized: UserNotification = serde_json::from_str(&serialized).unwrap();
        assert_eq!(notification, deserialized);
    }
}
```

**Coverage Requirements**:

- [ ] Happy path serialization for both notification types
- [ ] Edge cases: empty/null fields, special characters in strings
- [ ] JSON structure matches kebab-case naming convention
- [ ] Deserialization round-trip validation

#### 2. Hook Configuration Extensions

**Files Under Test**: `codex-rs/core/src/config_types.rs`
**Test File**: `codex-rs/core/src/config_types.rs` (test module)

**Test Cases**:

```rust
#[test]
fn test_hooks_config_with_notification_hooks() {
    let toml = r#"
    [hooks.stop.session-notify]
    command = "echo"
    args = ["session complete"]
    enabled = true

    [hooks.notification.tool-permission]
    command = "notify-send"
    args = ["Permission needed"]
    timeout_ms = 5000
    enabled = true

    [hooks.notification.idle-timeout]
    command = "/path/to/idle-script.sh"
    enabled = false
    "#;

    let config: HooksConfig = toml::from_str(toml).unwrap();

    assert_eq!(config.stop.len(), 1);
    assert_eq!(config.notification.len(), 2);
    assert!(config.notification["tool-permission"].enabled);
    assert!(!config.notification["idle-timeout"].enabled);
    assert_eq!(config.notification["tool-permission"].timeout_ms, Some(5000));
}

#[test]
fn test_idle_timeout_configuration() {
    let toml = r#"
    idle_timeout_seconds = 120
    "#;

    let config: Config = toml::from_str(toml).unwrap();
    assert_eq!(config.idle_timeout_seconds, 120);
}
```

#### 3. Hook Execution Extension

**Files Under Test**: `codex-rs/core/src/codex.rs`
**Test File**: `codex-rs/core/tests/hook_execution.rs`

**Test Cases**:

```rust
#[test]
fn test_notification_hook_execution() {
    let mut hooks_config = HooksConfig::default();
    hooks_config.notification.insert(
        "test-hook".to_string(),
        HookConfig {
            command: "echo".to_string(),
            args: vec!["notification received".to_string()],
            env: None,
            timeout_ms: Some(1000),
            enabled: true,
        }
    );

    let notification = UserNotification::ToolPermissionRequest {
        session_id: "test-session".to_string(),
        cwd: "/tmp".to_string(),
        approval_type: "exec".to_string(),
        tool_name: "Bash".to_string(),
        command: Some("test command".to_string()),
        changes: None,
        reason: None,
    };

    // Test hook execution (verify process spawning, not blocking)
    let turn_context = create_test_turn_context();
    session.maybe_notify_notification_hooks(&hooks_config, &turn_context, notification);

    // Should return immediately (fire-and-forget)
}
```

### Test Data and Fixtures:

**Test Data Requirements**:

- Sample notification payloads for all event types
- Valid and invalid hook configurations
- Mock turn context with realistic paths

**Test Environment Setup**:

- Mock file system for configuration discovery tests
- Temporary directories for hook execution tests
- Environment variable isolation for path expansion tests

### Success Criteria:

#### Automated Verification:

- [ ] Core unit tests pass: `cargo test -p codex-core`
- [ ] Serialization tests validate JSON structure: `cargo test user_notification`
- [ ] Configuration parsing tests pass: `cargo test config_types`
- [ ] Hook execution tests verify non-blocking behavior: `cargo test hook_execution`

#### Manual Verification:

- [ ] JSON payloads match Claude Code notification hook specification
- [ ] Configuration backwards compatibility maintained
- [ ] Hook execution follows existing patterns (timeout, environment, etc.)
- [ ] No performance regression in existing hook functionality

---

## Phase 2: TUI Integration and Trigger Tests

### Overview

Validate notification hook triggers in the TUI layer, including approval request handling and idle timeout detection.

### Integration Test Strategy:

#### 1. Approval Request Hook Triggers

**Integration Scope**: TUI approval handlers → Core hook execution
**Test Scenarios**:

- **Scenario 1**: Exec approval request triggers notification hook after modal display
- **Scenario 2**: Patch approval request triggers notification hook with file change details
- **Scenario 3**: Multiple rapid approval requests don't cause hook conflicts
- **Scenario 4**: Hook failures don't block approval workflow

**Files Under Test**: `codex-rs/tui/src/chatwidget.rs`
**Test File**: `codex-rs/tui/tests/approval_hook_integration.rs`

```rust
#[tokio::test]
async fn test_exec_approval_triggers_notification_hook() {
    let mut test_widget = create_test_chatwidget().await;
    let (hook_tx, mut hook_rx) = tokio::sync::mpsc::channel(10);

    // Configure notification hook
    test_widget.config.hooks.notification.insert(
        "test-hook".to_string(),
        create_test_hook_config()
    );

    // Simulate exec approval request
    let approval_event = ExecApprovalRequestEvent {
        id: "test-approval".to_string(),
        command: vec!["ls".to_string(), "-la".to_string()],
        reason: Some("List files".to_string()),
    };

    test_widget.on_exec_approval_request(approval_event).await;

    // Verify hook was triggered
    let hook_event = tokio::time::timeout(
        Duration::from_millis(100),
        hook_rx.recv()
    ).await.unwrap().unwrap();

    match hook_event {
        AppEvent::ExecuteNotificationHooks { notification, .. } => {
            if let UserNotification::ToolPermissionRequest {
                approval_type, tool_name, command, ..
            } = notification {
                assert_eq!(approval_type, "exec");
                assert_eq!(tool_name, "Bash");
                assert_eq!(command, Some("ls -la".to_string()));
            } else {
                panic!("Expected ToolPermissionRequest notification");
            }
        }
        _ => panic!("Expected ExecuteNotificationHooks event"),
    }
}

#[tokio::test]
async fn test_patch_approval_triggers_notification_hook() {
    // Similar test for patch approval with file changes
}

#[tokio::test]
async fn test_hook_trigger_timing_after_modal() {
    // Verify hooks execute after modal display, not before
}
```

#### 2. Idle Timeout Detection

**Integration Scope**: App event loop → Idle detection → Hook execution
**Test Scenarios**:

- **Scenario 1**: 60-second idle period triggers timeout notification
- **Scenario 2**: User input resets idle timer correctly
- **Scenario 3**: Multiple idle timeouts don't accumulate
- **Scenario 4**: Configurable timeout duration works correctly

**Files Under Test**: `codex-rs/tui/src/app.rs`
**Test File**: `codex-rs/tui/tests/idle_timeout_integration.rs`

```rust
#[tokio::test]
async fn test_idle_timeout_detection() {
    let mut app = create_test_app_with_timeout(60).await;
    let (hook_tx, mut hook_rx) = tokio::sync::mpsc::channel(10);

    // Start app event loop in background
    let app_handle = tokio::spawn(async move {
        app.run_for_test_duration(Duration::from_secs(65)).await
    });

    // Wait for idle timeout
    let hook_event = tokio::time::timeout(
        Duration::from_secs(70),
        hook_rx.recv()
    ).await.unwrap().unwrap();

    match hook_event {
        AppEvent::ExecuteNotificationHooks { notification, .. } => {
            if let UserNotification::PromptIdleTimeout {
                idle_duration_seconds, ..
            } = notification {
                assert_eq!(idle_duration_seconds, 60);
            } else {
                panic!("Expected PromptIdleTimeout notification");
            }
        }
        _ => panic!("Expected ExecuteNotificationHooks event"),
    }
}

#[tokio::test]
async fn test_idle_timer_reset_on_input() {
    let mut app = create_test_app_with_timeout(60).await;

    // Simulate user input after 30 seconds
    tokio::time::sleep(Duration::from_secs(30)).await;
    app.handle_key_event(KeyEvent::from(KeyCode::Char('a'))).await;

    // Wait another 40 seconds (total 70, but reset at 30)
    tokio::time::sleep(Duration::from_secs(40)).await;

    // Should not have triggered idle timeout yet
    assert!(!app.has_triggered_idle_timeout());
}

#[tokio::test]
async fn test_configurable_idle_timeout() {
    let mut app = create_test_app_with_timeout(30).await; // 30-second timeout

    tokio::time::sleep(Duration::from_secs(35)).await;
    assert!(app.has_triggered_idle_timeout());
}
```

**Mock Strategy**:

- Mock hook execution to capture hook events without external process spawning
- Mock time advancement for timeout testing
- Stub session state for consistent test data

### Success Criteria:

#### Automated Verification:

- [ ] TUI integration tests pass: `cargo test -p codex-tui --test approval_hook_integration`
- [ ] Idle timeout tests pass: `cargo test -p codex-tui --test idle_timeout_integration`
- [ ] Event flow validation: Hook triggers reach core layer correctly
- [ ] Timing tests: Hooks execute after modal display, not before

#### Manual Verification:

- [ ] Approval workflows continue normally with hook triggers active
- [ ] Idle timeout detection works with real user input patterns
- [ ] Hook failures don't affect TUI responsiveness
- [ ] Multiple rapid approvals handled gracefully

---

## Phase 3: End-to-End Workflow Tests

### Overview

Validate complete user workflows from approval request through hook execution, including configuration loading and real external process execution.

### Integration Test Strategy:

#### 1. Complete Approval Workflow

**Integration Scope**: Configuration → TUI → Core → External Hook Process
**Test Scenarios**:

- **Scenario 1**: Complete exec approval with external notification hook
- **Scenario 2**: Complete patch approval with file change notification
- **Scenario 3**: Project-level hooks override global hooks correctly
- **Scenario 4**: Hook configuration errors don't break approval flow

**Test Implementation**:

```rust
#[tokio::test]
async fn test_complete_exec_approval_workflow() {
    // Setup test environment with real config file
    let temp_dir = create_temp_project_with_config().await;
    let config_path = temp_dir.path().join(".codex/config.toml");

    write_test_notification_config(&config_path, r#"
    [hooks.notification.test-exec]
    command = "echo"
    args = ["exec-approval", "{{session_id}}", "{{approval_type}}"]
    timeout_ms = 1000
    enabled = true
    "#).await;

    let mut app = create_test_app_with_config(&config_path).await;

    // Trigger exec approval
    let approval_request = create_exec_approval_request();
    app.handle_approval_request(approval_request).await;

    // Wait for hook execution and capture output
    let hook_output = capture_hook_execution_output(Duration::from_secs(2)).await;

    assert!(hook_output.contains("exec-approval"));
    assert!(hook_output.contains("exec")); // approval_type
}

#[tokio::test]
async fn test_project_level_hook_override() {
    let temp_dir = create_temp_project_with_config().await;

    // Create global config
    let global_config = temp_dir.path().join(".codex/global.toml");
    write_test_notification_config(&global_config, r#"
    [hooks.notification.test-hook]
    command = "echo"
    args = ["global-hook"]
    enabled = true
    "#).await;

    // Create project config that overrides
    let project_config = temp_dir.path().join(".codex/config.toml");
    write_test_notification_config(&project_config, r#"
    [hooks.notification.test-hook]
    command = "echo"
    args = ["project-hook"]
    enabled = true
    "#).await;

    let app = create_test_app_with_configs(&global_config, &project_config).await;

    // Trigger notification and verify project hook executed
    let hook_output = trigger_and_capture_notification(&app).await;
    assert!(hook_output.contains("project-hook"));
    assert!(!hook_output.contains("global-hook"));
}
```

#### 2. Configuration Discovery and Validation

**Test Focus**: Configuration file discovery, parsing, and error handling

```rust
#[test]
fn test_notification_hook_config_parsing() {
    let valid_configs = vec![
        // Basic notification hook
        r#"
        [hooks.notification.basic]
        command = "notify-send"
        args = ["Test notification"]
        enabled = true
        "#,

        // With environment variables
        r#"
        [hooks.notification.with-env]
        command = "$CODEX_PROJECT_DIR/hooks/notify.sh"
        env = { "PROJECT_NAME" = "test-project" }
        timeout_ms = 30000
        enabled = true
        "#,

        // Disabled hook
        r#"
        [hooks.notification.disabled]
        command = "echo"
        enabled = false
        "#,
    ];

    for config_toml in valid_configs {
        let config: HooksConfig = toml::from_str(config_toml).unwrap();
        assert!(!config.notification.is_empty());
    }
}

#[test]
fn test_invalid_hook_configs() {
    let invalid_configs = vec![
        // Missing command
        r#"
        [hooks.notification.invalid]
        args = ["test"]
        enabled = true
        "#,

        // Invalid timeout
        r#"
        [hooks.notification.invalid-timeout]
        command = "echo"
        timeout_ms = "not-a-number"
        "#,
    ];

    for config_toml in invalid_configs {
        assert!(toml::from_str::<HooksConfig>(config_toml).is_err());
    }
}
```

### Success Criteria:

#### Automated Verification:

- [ ] End-to-end tests pass: `cargo test --test e2e_notification_hooks`
- [ ] Configuration tests pass: `cargo test config_discovery`
- [ ] Real hook execution completes within timeout: `cargo test hook_execution_timing`
- [ ] Project override behavior validated: `cargo test project_hook_override`

#### Manual Verification:

- [ ] Complete user workflows function with hooks active
- [ ] External notification services receive correct payloads
- [ ] Configuration errors provide helpful feedback
- [ ] Hook execution doesn't interfere with normal codex operation

---

## Test Infrastructure Requirements

### Test Framework and Tools:

- **Unit Testing**: Rust `cargo test` with `tokio-test` for async testing
- **Integration Testing**: Custom test harnesses for TUI and core interaction
- **Mocking/Stubbing**: `mockall` for trait mocking, manual mocks for process execution
- **Test Data Management**: Temporary directory management with `tempfile`
- **Coverage Reporting**: `cargo-tarpaulin` with 90% coverage target for new code

### CI/CD Integration:

- **Test Execution**: All tests run on PR and main branch commits
- **Coverage Reports**: Posted to PR comments with coverage delta
- **Test Result Artifacts**: JUnit XML for test result visualization
- **Failure Notifications**: Slack/email alerts for test failures in main branch

## Test Data Management

### Test Data Strategy:

- **Data Generation**: Builder patterns for test notification objects
- **Data Cleanup**: Automatic cleanup of temporary files and processes
- **Data Isolation**: Each test uses isolated temporary directories
- **Sensitive Data Handling**: No real credentials or personal data in tests

### Fixtures and Mocks:

- **Static Fixtures**: Sample configuration files for various hook scenarios
- **Dynamic Fixtures**: Generated notification payloads with randomized session IDs
- **External Service Mocks**: Mock HTTP servers for webhook testing
- **Process Mocks**: Captured process execution for hook command validation

## Performance and Load Testing

### Performance Requirements:

- **Hook Trigger Latency**: < 10ms from approval to hook execution start
- **Idle Timeout Accuracy**: ± 1 second of configured timeout duration
- **Memory Usage**: No memory leaks during continuous hook execution
- **CPU Impact**: < 5% CPU increase during idle timeout monitoring

### Load Testing Strategy:

- **Load Scenarios**: 100 approval requests per minute with hooks enabled
- **Stress Testing**: 1000 hooks executed concurrently (within timeout limits)
- **Endurance Testing**: 24-hour continuous operation with periodic approvals
- **Spike Testing**: Sudden burst of 50 approvals in 1 second

## Security Testing

### Security Test Requirements:

- **Input Validation**: Malicious JSON payloads don't cause injection
- **Command Injection**: Hook commands with shell metacharacters handled safely
- **File System Access**: Hook execution respects sandbox boundaries
- **Environment Variables**: No sensitive data leaked through hook environment

### Security Test Approach:

Validate that hook execution maintains the existing security model:
- Hook commands run with same privileges as codex process
- No additional file system access beyond existing patterns
- Environment variable expansion doesn't expose sensitive data
- JSON payloads are sanitized and don't contain executable content

## Test Maintenance and Evolution

### Test Maintenance Strategy:

- **Test Update Process**: Tests updated alongside implementation changes in same PR
- **Test Deprecation**: Remove tests when features are deprecated with 2-release notice
- **Test Performance**: Keep test suite under 5 minutes total execution time
- **Test Documentation**: Each test includes docstring explaining purpose and validation

## References

- Related implementation plan: `.strategic-claude-basic/plan/PLAN_0006_18-09-2025_thu_notification-hooks-upgrade.md`
- Existing hook tests: `codex-rs/core/src/user_notification.rs:33-64`
- TUI testing patterns: `codex-rs/tui/tests/`
- Rust testing documentation: https://doc.rust-lang.org/book/ch11-00-testing.html
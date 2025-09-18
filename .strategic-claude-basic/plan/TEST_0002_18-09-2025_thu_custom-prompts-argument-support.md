# Custom Prompts Argument Support Test Plan

## Overview

Comprehensive testing strategy for the custom prompts argument support feature, covering argument parsing, template processing, error handling, backward compatibility, and user interface validation. The testing approach ensures robust functionality while maintaining existing system reliability.

## Implementation Plan Reference

**Related Implementation Plan**: `.strategic-claude-basic/plan/PLAN_0002_18-09-2025_thu_custom-prompts-argument-support.md`

The implementation plan covers extending the custom command system with argument parsing (shlex-based), template processing (Askama and simple substitution), and integration into the chat composer. This test plan validates all implementation phases and integration points.

## Current Test Coverage Analysis

**Existing Test Infrastructure**:
- **Unit tests**: Custom prompt discovery in `codex-rs/core/src/custom_prompts.rs`
- **Snapshot tests**: UI rendering with `insta` crate for command popup and chat composer
- **Integration tests**: Chat widget functionality in `codex-rs/tui/src/chatwidget/tests.rs`
- **Template tests**: Askama template processing in various modules

**Coverage Gaps Identified**:
- No argument parsing tests for custom prompts (only built-in commands)
- No template variable substitution tests for custom content
- No error handling tests for malformed templates
- Limited end-to-end testing of command-to-execution flow

## Test Strategy

### Test Types Required:

- **Unit Tests**: Argument parsing, template processing, CustomPrompt structure extensions
- **Integration Tests**: Command popup to chat composer flow, template processor integration
- **End-to-End Tests**: Full user workflow from command entry to message submission
- **Performance Tests**: Template processing overhead, command parsing performance
- **Security Tests**: Malicious template content handling, argument injection prevention

### Testing Approach:

Follow existing codebase patterns using `tempfile`, `tokio::test`, `insta` snapshots, and `pretty_assertions`. Create isolated test environments for file-based custom prompts and mock external dependencies for consistent test execution.

## What We're NOT Testing

- **Advanced template features**: No testing for conditionals, loops, or complex Askama features (out of scope)
- **UI/UX design validation**: No user experience testing beyond functional behavior
- **Performance optimization**: No micro-benchmarking of individual functions
- **Cross-platform compatibility**: Testing on single platform (existing CI handles multi-platform)
- **Built-in command changes**: No testing of modifications to existing slash commands

## Phase 1: Unit Tests for Core Components

### Overview

Validate individual components work correctly in isolation: argument parsing, template processing, and data structure enhancements.

### Test Coverage Requirements:

#### 1. Argument Parsing (`command_popup.rs`)

**Files Under Test**: `codex-rs/tui/src/bottom_pane/command_popup.rs`
**Test File**: `codex-rs/tui/src/bottom_pane/command_popup.rs` (existing test module)

**Test Cases**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_argument_parsing_basic() {
        let mut popup = CommandPopup::new(/* params */);
        popup.on_composer_text_change("/research hello world".to_string());

        assert_eq!(popup.command_filter, "research");
        assert_eq!(popup.command_args, vec!["hello", "world"]);
    }

    #[test]
    fn test_argument_parsing_quoted() {
        let mut popup = CommandPopup::new(/* params */);
        popup.on_composer_text_change("/research \"AI safety\" simple".to_string());

        assert_eq!(popup.command_filter, "research");
        assert_eq!(popup.command_args, vec!["AI safety", "simple"]);
    }

    #[test]
    fn test_argument_parsing_empty() {
        let mut popup = CommandPopup::new(/* params */);
        popup.on_composer_text_change("/research".to_string());

        assert_eq!(popup.command_filter, "research");
        assert_eq!(popup.command_args, Vec::<String>::new());
    }

    #[test]
    fn test_argument_parsing_malformed_quotes() {
        let mut popup = CommandPopup::new(/* params */);
        popup.on_composer_text_change("/research \"unclosed quote".to_string());

        assert_eq!(popup.command_filter, "research");
        // Should fall back to simple whitespace splitting
        assert_eq!(popup.command_args.len(), 1);
    }
}
```

**Coverage Requirements**:

- [ ] Basic space-separated arguments
- [ ] Quoted arguments with spaces
- [ ] Empty argument list
- [ ] Malformed quotes (fallback behavior)
- [ ] Special characters in arguments
- [ ] Multiple quoted arguments
- [ ] Mixed quoted and unquoted arguments

#### 2. Template Processing (`template_processor.rs`)

**Files Under Test**: `codex-rs/core/src/template_processor.rs`
**Test File**: `codex-rs/core/src/template_processor.rs` (new test module)

**Test Cases**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_simple_template_processing() {
        let content = "Research the topic: {0}. Focus on {1}.";
        let args = vec!["AI safety".to_string(), "technical aspects".to_string()];

        let result = process_simple_template(content, &args).unwrap();
        assert_eq!(result, "Research the topic: AI safety. Focus on technical aspects.");
    }

    #[test]
    fn test_askama_template_processing() {
        let content = "Subject: {{ subject }}\nContext: {{ context }}";
        let mut args = HashMap::new();
        args.insert("subject".to_string(), "testing".to_string());
        args.insert("context".to_string(), "unit test".to_string());

        let result = process_askama_template(content, &args).unwrap();
        assert_eq!(result, "Subject: testing\nContext: unit test");
    }

    #[test]
    fn test_template_missing_variable() {
        let content = "Missing: {1}";
        let args = vec!["only one arg".to_string()];

        let result = process_simple_template(content, &args).unwrap();
        assert_eq!(result, "Missing: {1}"); // Should leave unmatched placeholders
    }

    #[test]
    fn test_template_error_handling() {
        let content = "Invalid template: {{ unclosed";
        let args = HashMap::new();

        match process_askama_template(content, &args) {
            Err(TemplateError::ProcessingError(_)) => (),
            _ => panic!("Expected ProcessingError"),
        }
    }
}
```

**Coverage Requirements**:

- [ ] Simple placeholder substitution (`{0}`, `{1}`)
- [ ] Askama variable substitution (`{{ variable }}`)
- [ ] Missing variables handling
- [ ] Template syntax errors
- [ ] Empty content and empty arguments
- [ ] Special characters in variables
- [ ] Multiple variable occurrences

#### 3. CustomPrompt Structure Extension

**Files Under Test**: `codex-rs/protocol/src/custom_prompts.rs`
**Test File**: `codex-rs/protocol/src/custom_prompts.rs` (extend existing tests)

**Test Cases**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_prompt_serialization() {
        let prompt = CustomPrompt {
            name: "test".to_string(),
            path: PathBuf::from("/test.md"),
            content: "Test content".to_string(),
            category: None,
            template_args: Some(vec![TemplateArg {
                name: "subject".to_string(),
                description: Some("Research subject".to_string()),
                required: true,
                default_value: None,
            }]),
            template_syntax: Some(TemplateSyntax::Askama),
        };

        let serialized = serde_json::to_string(&prompt).unwrap();
        let deserialized: CustomPrompt = serde_json::from_str(&serialized).unwrap();

        assert_eq!(prompt.name, deserialized.name);
        assert_eq!(prompt.template_args.is_some(), deserialized.template_args.is_some());
    }

    #[test]
    fn test_backward_compatibility() {
        // Test that existing prompts without template fields still work
        let old_prompt_json = r#"{
            "name": "old",
            "path": "/old.md",
            "content": "Old content",
            "category": null
        }"#;

        let prompt: CustomPrompt = serde_json::from_str(old_prompt_json).unwrap();
        assert!(prompt.template_args.is_none());
        assert!(prompt.template_syntax.is_none());
    }
}
```

### Test Data and Fixtures:

**Test Data Requirements**:

- Sample custom prompt files with various template syntaxes
- JSON fixtures for CustomPrompt serialization testing
- Command line input samples for argument parsing

**Test Environment Setup**:

- Temporary directories for custom prompt files (`tempfile::tempdir()`)
- Mock file system structure for prompt discovery
- Isolated test configuration to avoid side effects

### Success Criteria:

#### Automated Verification:

- [ ] Unit tests pass: `cargo test -p codex-core template_processor`
- [ ] TUI unit tests pass: `cargo test -p codex-tui command_popup`
- [ ] Protocol unit tests pass: `cargo test -p codex-protocol custom_prompts`
- [ ] Coverage threshold met: 95% line coverage for new code

#### Manual Verification:

- [ ] All edge cases covered in test specifications
- [ ] Error conditions properly tested
- [ ] Test output provides clear failure diagnostics
- [ ] Test execution completes under 5 seconds per module

---

## Phase 2: Integration Tests

### Overview

Validate component interactions work correctly: command popup to chat composer flow, template processing integration, and end-to-end argument handling.

### Integration Test Strategy:

#### 1. Command Flow Integration

**Integration Scope**: Command popup argument parsing → Chat composer template processing → Message submission
**Test Scenarios**:

- **Normal operation**: User types command with arguments, template processes correctly, message submitted
- **Error conditions**: Template processing fails, graceful fallback to original content
- **Edge cases**: Empty arguments, malformed templates, missing prompt files

**Test Implementation**:

```rust
#[tokio::test]
async fn test_custom_prompt_with_arguments_integration() {
    let (mut chat, mut rx, _op_rx) = make_chatwidget_manual();

    // Create test prompt file with template
    let temp_dir = tempdir().unwrap();
    let prompt_file = temp_dir.path().join("research.md");
    fs::write(&prompt_file, "Research topic: {{ subject }}").unwrap();

    // Simulate user typing command with arguments
    chat.bottom_pane.set_text("/research \"AI safety\"".to_string());
    chat.bottom_pane.handle_key_event(KeyEvent::from(KeyCode::Enter));

    // Verify processed message
    let event = rx.recv().await.unwrap();
    match event {
        AppEvent::SubmitMessage(msg) => {
            assert_eq!(msg, "Research topic: AI safety");
        }
        _ => panic!("Expected SubmitMessage event"),
    }
}

#[tokio::test]
async fn test_backward_compatibility_integration() {
    let (mut chat, mut rx, _op_rx) = make_chatwidget_manual();

    // Create old-style prompt without templates
    let temp_dir = tempdir().unwrap();
    let prompt_file = temp_dir.path().join("simple.md");
    fs::write(&prompt_file, "Simple static content").unwrap();

    // Use without arguments
    chat.bottom_pane.set_text("/simple".to_string());
    chat.bottom_pane.handle_key_event(KeyEvent::from(KeyCode::Enter));

    // Verify original content preserved
    let event = rx.recv().await.unwrap();
    match event {
        AppEvent::SubmitMessage(msg) => {
            assert_eq!(msg, "Simple static content");
        }
        _ => panic!("Expected SubmitMessage event"),
    }
}
```

**Mock Strategy**:

- Mock file system for prompt discovery using `tempfile`
- Mock app event channels for message capture
- Stub external dependencies (no network calls)

#### 2. Error Handling Integration

**Integration Scope**: Template processing errors → User feedback → Graceful degradation
**Test Scenarios**:

- **Template syntax errors**: Invalid Askama syntax falls back to original content
- **Missing arguments**: Required template variables handled gracefully
- **File system errors**: Prompt file access failures handled

### Success Criteria:

#### Automated Verification:

- [ ] Integration tests pass: `cargo test -p codex-tui integration`
- [ ] Error scenarios validated: `cargo test -p codex-tui error_handling`
- [ ] Performance within limits: Template processing < 10ms per prompt
- [ ] Memory usage stable: No memory leaks in repeated operations

#### Manual Verification:

- [ ] End-to-end workflows function correctly from UI to submission
- [ ] Error scenarios provide appropriate user feedback
- [ ] System remains stable under error conditions
- [ ] Backward compatibility verified with existing prompts

---

## Phase 3: UI and Snapshot Testing

### Overview

Validate user interface behavior and visual consistency using snapshot testing for command popup appearance and behavior changes.

### UI Test Strategy:

#### 1. Command Popup Snapshot Tests

**Files Under Test**: Command popup rendering with arguments
**Test Implementation**:

```rust
#[test]
fn test_command_popup_with_arguments_snapshot() {
    let terminal = setup_test_terminal();
    let mut popup = CommandPopup::new(/* params */);

    // Test command filtering with arguments
    popup.on_composer_text_change("/res \"test subject\"".to_string());
    popup.render(terminal.area(), terminal.buffer_mut());

    assert_snapshot!(terminal.backend(), @"command_popup_with_args");
}

#[test]
fn test_argument_completion_snapshot() {
    let terminal = setup_test_terminal();
    let mut popup = CommandPopup::new(/* params */);

    // Test argument display in completion
    popup.on_composer_text_change("/research ".to_string());
    popup.render(terminal.area(), terminal.buffer_mut());

    assert_snapshot!(terminal.backend(), @"command_popup_argument_hint");
}
```

#### 2. Chat Composer Integration Snapshots

**Files Under Test**: Chat composer behavior with custom prompt arguments
**Test Coverage**:

- Command entry state with arguments
- Template processing feedback
- Error state display

### Success Criteria:

#### Automated Verification:

- [ ] Snapshot tests pass: `cargo test -p codex-tui snapshot`
- [ ] UI regressions detected: `cargo insta test`
- [ ] Visual consistency maintained across scenarios

#### Manual Verification:

- [ ] Command popup appearance unchanged for existing commands
- [ ] Argument display clear and consistent
- [ ] Error states visually appropriate

---

## Test Infrastructure Requirements

### Test Framework and Tools:

- **Unit Testing**: Standard Rust `cargo test` with `tokio::test` for async
- **Integration Testing**: `tokio::test` with mock channels and temporary file systems
- **Mocking/Stubbing**: `tempfile` for file system, custom mocks for event channels
- **Test Data Management**: Static fixtures and dynamic generation with `tempfile`
- **Coverage Reporting**: `cargo tarpaulin` for coverage analysis
- **Snapshot Testing**: `insta` crate for UI regression testing

### CI/CD Integration:

- **Test Execution**: All test phases run on pull request and main branch pushes
- **Coverage Reports**: Coverage reports uploaded to code coverage service
- **Test Result Artifacts**: Snapshot differences preserved for manual review
- **Failure Notifications**: Test failures block merge and notify maintainers

## Test Data Management

### Test Data Strategy:

- **Data Generation**: Dynamic prompt files created per test with `tempfile`
- **Data Cleanup**: Automatic cleanup via `tempfile` RAII patterns
- **Data Isolation**: Each test uses separate temporary directories
- **Sensitive Data Handling**: No sensitive data in tests (use placeholder content)

### Fixtures and Mocks:

- **Static Fixtures**: Sample prompt files with various template syntaxes
- **Dynamic Fixtures**: Generated prompt content for specific test scenarios
- **External Service Mocks**: No external services involved in custom prompts
- **Database Mocks**: No database interaction for file-based custom prompts

## Performance and Load Testing

### Performance Requirements:

- **Response time**: Template processing < 10ms per prompt
- **Throughput**: Command parsing handles > 100 commands/second
- **Resource usage**: Memory usage growth < 1MB per 1000 processed prompts
- **Scalability**: Performance linear with prompt file count

### Load Testing Strategy:

- **Load Scenarios**: Rapid command entry, large template files, many arguments
- **Stress Testing**: Memory pressure with large argument lists
- **Endurance Testing**: Repeated template processing over time
- **Spike Testing**: Sudden burst of command processing

## Security Testing

### Security Test Requirements:

- **Input Validation**: Malicious arguments and template content handling
- **Template Injection**: Prevention of code execution through templates
- **File System Access**: Restricted access to authorized prompt directories
- **Data Protection**: No sensitive data leakage through template variables

### Security Test Approach:

Test malicious input handling including script injection attempts, path traversal in prompt files, and template content that attempts code execution. Validate that shlex parsing prevents command injection and template processing is sandboxed.

## Test Maintenance and Evolution

### Test Maintenance Strategy:

- **Test Update Process**: Tests updated with implementation changes in same PR
- **Test Deprecation**: Remove tests for deprecated functionality after migration
- **Test Performance**: Regular review of test execution time and optimization
- **Test Documentation**: Inline comments explain complex test scenarios

## References

- Related implementation plan: `.strategic-claude-basic/plan/PLAN_0002_18-09-2025_thu_custom-prompts-argument-support.md`
- Similar test implementation: `codex-rs/tui/src/chatwidget/tests.rs`
- Testing framework documentation: `insta` crate docs, `tokio::test` patterns
- Existing test patterns: `codex-rs/core/src/custom_prompts.rs` test module
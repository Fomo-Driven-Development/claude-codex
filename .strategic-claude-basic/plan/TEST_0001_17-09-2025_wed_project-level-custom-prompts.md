# Project-Level Custom Prompts Test Plan

## Overview

Comprehensive testing strategy for automatic `.codex/prompts` directory discovery and `/directory:command` syntax support. This test plan ensures robust validation of project-specific custom prompts while maintaining backward compatibility with existing functionality.

## Implementation Plan Reference

**Related Implementation Plan**: `.strategic-claude-basic/plan/PLAN_0001_17-09-2025_wed_project-level-custom-prompts.md`

The implementation plan covers three phases: automatic discovery, directory structure support, and directory command syntax. This test plan validates each phase with appropriate unit, integration, and end-to-end testing.

## Current Test Coverage Analysis

**Existing Test Infrastructure:**
- `custom_prompts.rs` - Comprehensive unit tests for discovery mechanisms
- `command_popup.rs` - Tests for command filtering, parsing, and collision handling
- `git_info.rs` - Extensive tests for Git detection, repository boundaries, worktrees
- `project_doc.rs` - Good coverage for Git-aware project file discovery
- Established patterns for temporary directory testing and async test execution

**Test Infrastructure Quality:**
- Robust use of `tempfile::TempDir` for isolated test environments
- Async test patterns with `#[tokio::test]`
- Comprehensive edge case coverage (missing files, Git boundaries, malformed data)
- Mock Git repository creation utilities in `git_info.rs`

**Coverage Gaps:**
- No existing tests for multi-directory prompt discovery
- No tests for directory-based command namespacing
- No integration tests between Git detection and prompt discovery
- No tests for command parsing with colon syntax

## Test Strategy

### Test Types Required:

- **Unit Tests**: Custom prompt discovery, command parsing, Git integration, directory scanning
- **Integration Tests**: Multi-directory discovery, command popup integration, Git boundary detection
- **End-to-End Tests**: Complete user workflows from directory creation to command execution
- **Performance Tests**: Directory scanning performance, startup impact measurement
- **Security Tests**: Path traversal prevention, malicious directory name handling

### Testing Approach:

**Phase-Aligned Testing**: Each test phase corresponds to implementation phases, ensuring incremental validation of functionality as it's built.

**Existing Pattern Reuse**: Leverage established test utilities from `git_info.rs` for Git repository creation and `custom_prompts.rs` for directory setup patterns.

**Comprehensive Edge Case Coverage**: Focus on Git boundary conditions, file system edge cases, and command parsing variations.

## What We're NOT Testing

- Variable substitution or templating (not implemented)
- Project-specific configuration files (out of scope)
- Authentication or permission systems (not applicable)
- Complex prompt inheritance or includes (not implemented)
- Performance under extreme load (beyond reasonable use cases)
- Non-standard Git configurations (worktrees handled separately)
- UI visual testing (command popup display only functionally tested)

## Phase 1: Automatic Discovery Unit Tests

### Overview

Validate automatic `.codex/prompts` directory discovery functionality, Git integration, and precedence rules between global and project prompts.

### Test Coverage Requirements:

#### 1. Project-Level Discovery Core Logic

**Files Under Test**: `codex-rs/core/src/custom_prompts.rs`
**Test File**: `codex-rs/core/src/custom_prompts.rs` (extended test module)

**Test Cases**:

```rust
#[tokio::test]
async fn discover_prompts_with_project_support_no_git_repo() {
    // Test behavior when no Git repository exists
    let tmp = tempdir().expect("create TempDir");
    let global_dir = tmp.path().join("global");
    let project_dir = tmp.path().join("project");

    fs::create_dir_all(&global_dir).unwrap();
    fs::write(global_dir.join("global.md"), "global prompt").unwrap();

    let prompts = discover_prompts_with_project_support(&global_dir, &project_dir).await;
    assert_eq!(prompts.len(), 1);
    assert_eq!(prompts[0].name, "global");
}

#[tokio::test]
async fn discover_prompts_with_project_support_project_overrides_global() {
    // Test project prompts take precedence over global ones
    let tmp = tempdir().expect("create TempDir");
    setup_git_repo(&tmp).await;

    let global_dir = tmp.path().join("global");
    let project_prompts = tmp.path().join(".codex/prompts");

    fs::create_dir_all(&global_dir).unwrap();
    fs::create_dir_all(&project_prompts).unwrap();

    fs::write(global_dir.join("shared.md"), "global version").unwrap();
    fs::write(project_prompts.join("shared.md"), "project version").unwrap();

    let prompts = discover_prompts_with_project_support(&global_dir, tmp.path()).await;
    let shared_prompt = prompts.iter().find(|p| p.name == "shared").unwrap();
    assert_eq!(shared_prompt.content, "project version");
}

#[tokio::test]
async fn discover_prompts_with_project_support_git_subdirectory() {
    // Test discovery works from Git repository subdirectories
    let tmp = tempdir().expect("create TempDir");
    setup_git_repo(&tmp).await;

    let project_prompts = tmp.path().join(".codex/prompts");
    let subdirectory = tmp.path().join("nested/deep/path");

    fs::create_dir_all(&project_prompts).unwrap();
    fs::create_dir_all(&subdirectory).unwrap();
    fs::write(project_prompts.join("project.md"), "project prompt").unwrap();

    let prompts = discover_prompts_with_project_support(&PathBuf::new(), &subdirectory).await;
    assert_eq!(prompts.len(), 1);
    assert_eq!(prompts[0].name, "project");
}
```

**Coverage Requirements**:

- [ ] Project discovery when Git repository exists
- [ ] Fallback to global-only when no Git repository
- [ ] Project prompts override global prompts with same names
- [ ] Discovery works from Git repository subdirectories
- [ ] Graceful handling when `.codex/prompts` doesn't exist
- [ ] Empty project directory returns only global prompts
- [ ] Git repository detection edge cases (bare repos, worktrees)

#### 2. Handler Integration

**Files Under Test**: `codex-rs/core/src/codex.rs`
**Test File**: `codex-rs/core/tests/integration/custom_prompts_integration.rs` (new file)

**Test Cases**:

```rust
#[tokio::test]
async fn list_custom_prompts_includes_project_level() {
    // Test Op::ListCustomPrompts handler includes project prompts
    let tmp = tempdir().expect("create TempDir");
    setup_test_environment(&tmp).await;

    let config = create_test_config(&tmp);
    let response = handle_list_custom_prompts_op(&config).await;

    assert!(response.custom_prompts.iter().any(|p| p.name == "project-specific"));
    assert!(response.custom_prompts.iter().any(|p| p.name == "global-fallback"));
}
```

### Test Data and Fixtures:

**Test Data Requirements**:

- Git repository creation utilities (reuse from `git_info.rs`)
- Mock global and project prompt directories
- Sample markdown files with varying content sizes
- Edge case scenarios (empty files, non-UTF8 content, symlinks)

**Test Environment Setup**:

- Temporary directory creation for each test
- Git repository initialization utilities
- Helper functions for creating nested directory structures
- Mock configuration objects with appropriate paths

### Success Criteria:

#### Automated Verification:

- [ ] All unit tests pass: `cargo test custom_prompts`
- [ ] Integration tests pass: `cargo test integration::custom_prompts`
- [ ] Code coverage threshold met: `cargo tarpaulin --ignore-tests`
- [ ] No memory leaks in async operations: `cargo test --features=leak-detection`

#### Manual Verification:

- [ ] Test output clearly indicates which prompts are global vs project
- [ ] Edge case handling is properly validated
- [ ] Test execution time is reasonable (< 5 seconds per test file)
- [ ] Test isolation prevents cross-test interference

---

## Phase 2: Directory Structure Integration Tests

### Overview

Validate subdirectory scanning, namespace creation, and category metadata assignment for hierarchical prompt organization.

### Integration Test Strategy:

#### 1. Directory-Aware Discovery Integration

**Integration Scope**: Custom prompt discovery with directory scanning and namespace generation
**Test Scenarios**:

- **Normal Operation**: Nested directory structure with mixed flat and namespaced prompts
- **Error Conditions**: Permissions issues, deeply nested directories, circular symlinks
- **Edge Cases**: Empty subdirectories, special characters in directory names, case sensitivity

**Test Cases**:

```rust
#[tokio::test]
async fn discover_prompts_with_directories_mixed_structure() {
    let tmp = tempdir().expect("create TempDir");
    setup_complex_prompt_structure(&tmp).await;
    /*
    .codex/prompts/
    ├── root.md              # Flat command: /root
    ├── docs/
    │   ├── api.md          # Namespaced: /docs:api
    │   └── guide.md        # Namespaced: /docs:guide
    └── testing/
        └── unit.md         # Namespaced: /testing:unit
    */

    let prompts = discover_prompts_with_directories(tmp.path().join(".codex/prompts")).await;

    // Validate flat commands
    assert!(prompts.iter().any(|p| p.name == "root" && p.category.is_none()));

    // Validate namespaced commands
    assert!(prompts.iter().any(|p| p.name == "docs:api" && p.category == Some("docs".to_string())));
    assert!(prompts.iter().any(|p| p.name == "testing:unit" && p.category == Some("testing".to_string())));
}

#[tokio::test]
async fn directory_commands_with_special_characters() {
    // Test handling of special characters in directory names
    let tmp = tempdir().expect("create TempDir");
    let prompts_dir = tmp.path().join(".codex/prompts");
    let special_dir = prompts_dir.join("team-scripts");

    fs::create_dir_all(&special_dir).unwrap();
    fs::write(special_dir.join("deploy.md"), "deployment script").unwrap();

    let prompts = discover_prompts_with_directories(&prompts_dir).await;
    assert!(prompts.iter().any(|p| p.name == "team-scripts:deploy"));
}
```

**Mock Strategy**:

- File system operations stubbed for permission testing
- Directory traversal mocked for performance testing
- Git detection mocked for isolated directory testing

#### 2. Command Popup Integration

**Integration Scope**: Directory prompts integration with command popup filtering and display
**Test Scenarios**:

- Command popup correctly displays both flat and namespaced prompts
- Fuzzy matching works for directory commands
- Category information properly displayed in descriptions

### Success Criteria:

#### Automated Verification:

- [ ] Directory structure tests pass: `cargo test discover_prompts_with_directories`
- [ ] Namespace generation tests pass: `cargo test directory_namespace`
- [ ] Category metadata tests pass: `cargo test prompt_categories`
- [ ] Performance benchmarks met: `cargo bench directory_discovery`

#### Manual Verification:

- [ ] Complex directory structures handled correctly
- [ ] Empty subdirectories gracefully ignored
- [ ] Directory names with special characters work properly
- [ ] Category information correctly assigned and displayed

---

## Phase 3: Command Syntax End-to-End Tests

### Overview

Validate `/directory:command` syntax parsing, fuzzy matching, and command execution through complete user workflows.

### End-to-End Test Strategy:

#### 1. Command Parsing Workflow

**Test Scenarios**:

```rust
#[tokio::test]
async fn command_popup_directory_syntax_parsing() {
    // Test complete workflow: typing -> filtering -> selection -> execution
    let mut popup = CommandPopup::new(create_test_prompts_with_directories());

    // Test parsing of directory command
    popup.on_composer_text_change("/docs:a".to_string());
    let filtered = popup.filtered_items();

    // Should match "docs:api" command
    assert!(filtered.iter().any(|item| match item {
        CommandItem::UserPrompt(i) => popup.prompt_name(*i) == Some("docs:api"),
        _ => false,
    }));
}

#[tokio::test]
async fn fuzzy_matching_directory_commands() {
    // Test fuzzy matching works with colon syntax
    let mut popup = CommandPopup::new(create_test_prompts_with_directories());

    popup.on_composer_text_change("/d:ap".to_string());
    let filtered = popup.filtered_items();

    // Should fuzzy match "docs:api"
    assert!(filtered.len() > 0);
    assert!(popup.prompt_name(0) == Some("docs:api"));
}
```

#### 2. Complete User Workflow

**Test File**: `codex-rs/core/tests/integration/end_to_end_workflows.rs` (new file)

**Test Cases**:

```rust
#[tokio::test]
async fn complete_directory_command_workflow() {
    // Test: Create project -> Add prompts -> Discover -> Execute
    let tmp = tempdir().expect("create TempDir");
    setup_git_repo(&tmp).await;

    // 1. Create project structure
    let prompts_dir = tmp.path().join(".codex/prompts/docs");
    fs::create_dir_all(&prompts_dir).unwrap();
    fs::write(prompts_dir.join("api.md"), "API documentation prompt").unwrap();

    // 2. Test discovery
    let config = create_test_config_with_cwd(tmp.path());
    let prompts = discover_project_prompts(&config).await;
    assert!(prompts.iter().any(|p| p.name == "docs:api"));

    // 3. Test command popup integration
    let mut popup = CommandPopup::new(prompts);
    popup.on_composer_text_change("/docs:api".to_string());
    let selected = popup.selected_item();

    match selected {
        Some(CommandItem::UserPrompt(idx)) => {
            assert_eq!(popup.prompt_content(idx), Some("API documentation prompt"));
        }
        _ => panic!("Expected UserPrompt selection"),
    }
}
```

### Success Criteria:

#### Automated Verification:

- [ ] End-to-end workflow tests pass: `cargo test end_to_end_workflows`
- [ ] Command parsing integration tests pass: `cargo test command_popup_integration`
- [ ] Fuzzy matching tests pass: `cargo test fuzzy_match_directory`
- [ ] UI integration tests pass: `cargo test ui_integration`

#### Manual Verification:

- [ ] Complete user workflows function from start to finish
- [ ] Command completion works correctly for directory syntax
- [ ] Error messages are clear for malformed directory commands
- [ ] Performance is acceptable for large numbers of prompts

---

## Test Infrastructure Requirements

### Test Framework and Tools:

- **Unit Testing**: Rust's built-in `#[test]` and `#[tokio::test]` for async operations
- **Integration Testing**: Custom integration test modules with shared utilities
- **Mocking/Stubbing**: Manual mocking using test utilities (following existing patterns)
- **Test Data Management**: `tempfile::TempDir` for isolated test environments
- **Coverage Reporting**: `cargo tarpaulin` for coverage analysis

### CI/CD Integration:

- **Test Execution**: All tests run on `cargo test` in CI pipeline
- **Coverage Reports**: Coverage thresholds enforced for new code
- **Test Result Artifacts**: JUnit XML output for CI dashboard integration
- **Failure Notifications**: Test failures block PR merging

## Test Data Management

### Test Data Strategy:

- **Data Generation**: Helper functions create mock Git repositories and prompt structures
- **Data Cleanup**: `tempfile::TempDir` automatically cleans up test directories
- **Data Isolation**: Each test uses isolated temporary directories
- **Sensitive Data Handling**: No sensitive data in tests (all test data is synthetic)

### Fixtures and Mocks:

- **Static Fixtures**: Template markdown files for testing various prompt structures
- **Dynamic Fixtures**: Git repositories created per test with appropriate history
- **External Service Mocks**: Git commands stubbed for controlled testing scenarios
- **Database Mocks**: Not applicable (file system operations only)

## Performance and Load Testing

### Performance Requirements:

- **Discovery Time**: Project prompt discovery < 100ms for typical projects
- **Command Filtering**: Real-time filtering with < 10ms response for typical prompt counts
- **Memory Usage**: No significant memory overhead compared to existing global-only discovery
- **Startup Impact**: Project discovery adds < 50ms to application startup time

### Load Testing Strategy:

- **Large Directory Testing**: 1000+ prompts across 50+ directories
- **Deep Nesting Testing**: Directory structures 10+ levels deep
- **Concurrent Access**: Multiple discovery operations in parallel
- **Git Repository Size**: Large repositories with extensive history

**Performance Test Cases**:

```rust
#[tokio::test]
async fn performance_large_prompt_collection() {
    let tmp = tempdir().expect("create TempDir");
    create_large_prompt_structure(&tmp, 1000, 50).await;

    let start = Instant::now();
    let prompts = discover_prompts_with_project_support(&PathBuf::new(), tmp.path()).await;
    let duration = start.elapsed();

    assert!(duration < Duration::from_millis(200));
    assert_eq!(prompts.len(), 1000);
}
```

## Security Testing

### Security Test Requirements:

- **Path Traversal Prevention**: Malicious directory names cannot escape project boundaries
- **Symlink Safety**: Symlinks outside project directory are safely ignored
- **Input Validation**: Directory and file names properly sanitized
- **Git Boundary Enforcement**: Prompts cannot be discovered outside Git repository

### Security Test Approach:

**Path Traversal Testing**:

```rust
#[tokio::test]
async fn security_prevents_path_traversal() {
    let tmp = tempdir().expect("create TempDir");
    setup_git_repo(&tmp).await;

    // Attempt to create malicious directory structure
    let prompts_dir = tmp.path().join(".codex/prompts");
    let malicious_dir = prompts_dir.join("../../../etc");

    // This should fail to create or be safely ignored
    let _ = fs::create_dir_all(&malicious_dir);
    fs::write(malicious_dir.join("passwd.md"), "malicious").unwrap_or(());

    let prompts = discover_prompts_with_project_support(&PathBuf::new(), tmp.path()).await;

    // Should not include any prompts from outside project boundaries
    assert!(!prompts.iter().any(|p| p.path.components().any(|c| c.as_os_str() == "etc")));
}
```

## Test Maintenance and Evolution

### Test Maintenance Strategy:

- **Test Update Process**: Tests updated alongside implementation changes with peer review
- **Test Deprecation**: Deprecated functionality tests removed after transition period
- **Test Performance**: Test suite execution time monitored, slow tests optimized or parallelized
- **Test Documentation**: Each test includes clear comments explaining the scenario being validated

**Test Organization**:

- Unit tests remain co-located with implementation files
- Integration tests organized by feature area in `tests/` directory
- Shared test utilities extracted to common modules
- Performance tests separated into `benches/` directory

## References

- Related implementation plan: `.strategic-claude-basic/plan/PLAN_0001_17-09-2025_wed_project-level-custom-prompts.md`
- Existing test patterns: `codex-rs/core/src/custom_prompts.rs:76-127`
- Git testing utilities: `codex-rs/core/src/git_info.rs:488-938`
- Project discovery tests: `codex-rs/core/src/project_doc.rs:176-350`
- Command popup tests: `codex-rs/tui/src/bottom_pane/command_popup.rs:235-333`
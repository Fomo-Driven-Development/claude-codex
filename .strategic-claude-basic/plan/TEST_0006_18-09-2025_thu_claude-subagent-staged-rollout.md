# Claude Subagent Staged Rollout Test Plan

## Overview

Define validation coverage for the staged subagent rollout, focusing on agent definition loading, controller-driven orchestration, protocol metadata, and MVP frontend surfacing.

## Implementation Plan Reference

**Related Implementation Plan**: `.strategic-claude-basic/plan/PLAN_0006_18-09-2025_thu_claude-subagent-staged-rollout.md`

The implementation plan introduces markdown agent discovery, controller-level subagent orchestration, protocol metadata, and UI tagging for a staged MVP release.

## Current Test Coverage Analysis

- No existing tests cover agent discovery or subagent orchestration paths.
- Protocol serialization tests (`codex-rs/protocol/src/protocol.rs`) focus on existing event variants.
- TUI snapshot tests (`cargo insta`) cover baseline history cells but not labeled multi-agent output.
- Exec mode lacks regression tests for multi-task shutdown behavior.

## Test Strategy

### Test Types Required:

- **Unit Tests**: Agent loader parsing, orchestrator helpers, protocol metadata encoding.
- **Integration Tests**: Controller spawning flow with inherited approvals/sandbox policies.
- **End-to-End Tests**: TUI snapshot tests simulating parent + child transcripts; exec CLI harness tests for multi-agent completion.
- **Performance Tests**: Monitor orchestrator spawn latency under sequential invocations (lightweight benchmarks).
- **Security Tests**: Validate agent file parsing rejects unsafe frontmatter and enforces sandbox inheritance.

### Testing Approach:

Combine focused Rust unit tests with targeted integration harnesses that simulate tool calls. Use snapshot testing in `codex-tui` for rendering validation and scripted exec runs to confirm CLI tagging.

## What We're NOT Testing

- Automatic delegation heuristics (not implemented in MVP).
- True parallel task execution within a single `Session`.
- Seatbelt self-spawning flows (out of scope until future stages).

## Phase 1: Agent Loader Validation

### Overview

Ensure markdown agents load correctly from user and project scopes and that invalid definitions surface actionable errors.

### Test Coverage Requirements:

#### 1. Loader Unit Tests

**Files Under Test**: `codex-rs/core/src/agents/mod.rs`
**Test File**: `codex-rs/core/tests/agents_loader_tests.rs`

**Test Cases**:

```rust
#[test]
fn project_scope_overrides_user_scope() { /* ... */ }

#[test]
fn invalid_frontmatter_returns_error() { /* ... */ }

#[test]
fn tools_field_defaults_to_inherit_all() { /* ... */ }
```

**Coverage Requirements**:

- [ ] Happy path for merged project/user directories.
- [ ] Missing required fields (`name`, `description`) produce errors.
- [ ] Tool scope parsing handles explicit allowlists and inheritance.
- [ ] Model override optionality respected.

### Test Data and Fixtures:

**Test Data Requirements**:

- Temporary directories with `.codex/agents/*.md` fixtures.
- Sample markdown files with valid and invalid YAML frontmatter.

**Test Environment Setup**:

- Use `tempfile::TempDir` to isolate filesystem fixtures.
- No external services required.

### Success Criteria:

#### Automated Verification:

- [ ] `cargo test -p codex-core agents_loader_tests` passes.
- [ ] `cargo clippy -p codex-core` yields no warnings for new tests.

#### Manual Verification:

- [ ] Error messages in TUI align with loader failures seen in unit tests.

---

## Phase 2: Orchestrator & Protocol Integration

### Overview

Validate controller-level subagent spawning, approval inheritance, and protocol metadata serialization.

### Integration Test Strategy:

#### 1. Subagent Orchestration

**Integration Scope**: `SubagentOrchestrator`, `ConversationManager`, approval system.
**Test Scenarios**:

- Successful spawn with inherited approvals.
- Spawn request with unknown agent returns error.
- Parallel spawn requests queue and complete cleanly.

**Mock Strategy**:

- Use in-memory `ConversationManager::with_auth` in tests.
- Stub model responses to trigger `spawn_sub_agent` tool call.

#### 2. Protocol Metadata Serialization

**Integration Scope**: New `SubTaskMetadata` payloads in `TaskStartedEvent`, etc.
**Test Scenarios**:

- Serialization/deserialization preserves parent/child identifiers.
- Legacy clients ignoring metadata continue to parse events.

### Success Criteria:

#### Automated Verification:

- [ ] `cargo test -p codex-core orchestrator_integration` covers spawn flows.
- [ ] `cargo test -p codex-protocol subtask_metadata` validates new structs.
- [ ] `cargo test --doc -p codex-core` ensures documentation examples compile.

#### Manual Verification:

- [ ] Observed telemetry counters match expected spawn counts during manual runs.
- [ ] Approval prompts appear under the correct agent labels in the TUI.

---

## Phase 3: Frontend Rendering & CLI Behavior

### Overview

Confirm the TUI and exec clients render tagged messages and honor multi-agent completion rules.

### Test Coverage Requirements:

#### 1. TUI Snapshot Tests

**Files Under Test**: `codex-rs/tui/src/history_cell.rs`, `codex-rs/tui/src/chatwidget.rs`
**Test File**: `codex-rs/tui/tests/subagent_rendering.rs`

**Test Cases**:

```rust
#[test]
fn snapshot_parent_and_child_transcript() {
    // Render sample events and assert insta snapshot
}
```

**Coverage Requirements**:

- [ ] Snapshot includes parent and child labeled entries.
- [ ] Approval prompts route to labeled modal.
- [ ] Regression snapshot for legacy single-agent flow remains unchanged.

#### 2. Exec Harness Tests

**Files Under Test**: `codex-rs/exec/src/event_processor_with_human_output.rs`
**Test File**: `codex-rs/exec/tests/subagent_cli.rs`

**Test Cases**:

```rust
#[test]
fn waits_for_all_subtasks_before_shutdown() { /* ... */ }

#[test]
fn prefixes_output_with_agent_label() { /* ... */ }
```

**Coverage Requirements**:

- [ ] CLI remains running until final child `TaskComplete` received.
- [ ] Output lines include `[agent]` labels.

### Success Criteria:

#### Automated Verification:

- [ ] `cargo test -p codex-tui` passes with updated snapshots (use `cargo insta accept -p codex-tui` when ready).
- [ ] `cargo test -p codex-exec` covers new shutdown logic.
- [ ] `just fix -p codex-tui` and `just fix -p codex-exec` yield no lint errors.

#### Manual Verification:

- [ ] Visual inspection of TUI ensures no layout regressions.
- [ ] CLI output remains readable under streaming updates.

---

## Test Infrastructure Requirements

### Test Framework and Tools:

- **Unit Testing**: Rust `#[test]` modules with `tempfile`, `serde_yaml`.
- **Integration Testing**: Tokio async tests leveraging `ConversationManager::with_auth`.
- **Mocking/Stubbing**: Lightweight fake model clients for orchestrator tests.
- **Test Data Management**: Inline markdown fixtures + temp directories.
- **Coverage Reporting**: Optional `cargo llvm-cov` run after major changes.

### CI/CD Integration:

- **Test Execution**: Add `cargo test -p codex-core -p codex-tui -p codex-exec -p codex-protocol` jobs to CI matrix.
- **Coverage Reports**: Publish `llvm-cov` summary for subagent modules.
- **Test Result Artifacts**: Store `cargo insta` snapshots for review.
- **Failure Notifications**: Alert core team Slack channel on failure.

## Test Data Management

### Test Data Strategy:

- **Data Generation**: Generate markdown fixtures dynamically to avoid stale files.
- **Data Cleanup**: Use RAII temp dirs so cleanup occurs automatically.
- **Data Isolation**: One temp dir per test to prevent bleed-over.
- **Sensitive Data Handling**: Avoid embedding real credentials; use mock tool names.

### Fixtures and Mocks:

- **Static Fixtures**: Optional sample markdown stored under `codex-rs/core/tests/fixtures/agents`.
- **Dynamic Fixtures**: Programmatically build per-test agent files.
- **External Service Mocks**: Stub MCP tool responses when testing tool scope filtering.
- **Database Mocks**: Not required (no DB integration).

## Performance and Load Testing

### Performance Requirements:

- Controller spawn latency under 200 ms for sequential child runs.
- No noticeable TUI render lag with up to three concurrent subagents.
- Exec output flushing remains within current benchmarks.

### Load Testing Strategy:

- **Load Scenarios**: Repeated spawn of lightweight subagents in integration harness.
- **Stress Testing**: Simulate failure loops to ensure orchestrator tears down cleanly.
- **Endurance Testing**: Run nightly job spawning 100 child conversations sequentially.
- **Spike Testing**: Burst spawn requests to confirm controller serialization.

## Security Testing

### Security Test Requirements:

- Reject frontmatter that attempts to escape directories via relative paths.
- Ensure sandbox inheritance prevents child from escalating privileges.
- Validate tool allowlists prevent disallowed MCP tool access.

### Security Test Approach:

Use unit tests with malicious markdown samples and integration tests verifying sandbox flags on child conversations.

## Test Maintenance and Evolution

### Test Maintenance Strategy:

- Update snapshots whenever UI tagging changes (document in PR checklists).
- Add new orchestrator scenarios as future stages (auto-delegation) land.
- Track test performance; refactor long-running integration tests into async tasks.
- Document test cases in `docs/testing/subagents.md` for future contributors.

## References

- Implementation plan: `.strategic-claude-basic/plan/PLAN_0006_18-09-2025_thu_claude-subagent-staged-rollout.md`
- Research source: `.strategic-claude-basic/research/RESEARCH_0006_18-09-2025_thu_claude-code-subagent-parity.md`
- Conversation manager tests inspiration: `codex-rs/core/tests/conversation_manager_tests.rs`

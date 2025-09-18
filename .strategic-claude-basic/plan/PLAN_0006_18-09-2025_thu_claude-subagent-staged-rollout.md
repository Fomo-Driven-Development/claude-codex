# Claude Subagent Staged Rollout Implementation Plan

## Overview

Deliver the first milestone toward Claude Code subagent parity in Codex by introducing discoverable agent definitions, controller-orchestrated child conversations, and MVP frontend surfacing while preserving the single-task invariants inside `codex-core`.

## Current State Analysis

- `Session::set_task` enforces a single `AgentTask`, aborting any prior work (`codex-rs/core/src/codex.rs:533-547`).
- Tool exposure flows through `get_openai_tools`, so new delegation hooks must be wired there (`codex-rs/core/src/openai_tools.rs:500-560`).
- TUI and exec front-ends toggle global running state on `TaskStarted/TaskComplete` and assume one active transcript (`codex-rs/tui/src/chatwidget.rs:249-276`, `codex-rs/exec/src/event_processor_with_human_output.rs:171-210`).
- Protocol events only carry a free-form `id`, providing no metadata for parent/child relationships (`codex-rs/protocol/src/protocol.rs:409-470`).
- Project doc discovery already lists `AGENTS.md` files in summaries, offering a foundation for agent UX (`codex-rs/core/src/project_doc.rs:1-120`, `codex-rs/tui/src/history_cell.rs:1104-1147`).

## Desired End State

Codex loads markdown-based agent definitions from project and user scopes, exposes them through a new `/agents` workflow, lets the controller spawn child conversations in response to a `spawn_sub_agent` tool call (with inherited approvals and sandbox policy), and tags frontend output with lightweight subtask metadata. Automatic delegation and rich multiplexing remain future stages.

### Key Discoveries:

- Single-task invariant lives in `Session::set_task`, so orchestration must occur above session state (`codex-rs/core/src/codex.rs:533-538`).
- `ConversationManager::new_conversation` already supports spawning sibling conversations asynchronously (`codex-rs/core/src/conversation_manager.rs:17-120`).
- Approval queues and sandbox settings are stored on `Session::state`, meaning controller-spawned children must capture and reuse the parent’s configuration (`codex-rs/core/src/codex.rs:270-320`).
- TUI’s slash-command modal is extensible via `SlashCommand` enums for an `/agents` entry point (`codex-rs/tui/src/slash_command.rs:1-160`).
- Exec output can be tagged by prepending metadata before shutdown triggers (`codex-rs/exec/src/event_processor_with_human_output.rs:171-210`).

## What We're NOT Doing

- Automatic multi-agent delegation within the model planner.
- Concurrent task support inside `Session` or changes to task abort semantics.
- Rich multi-pane UI for simultaneous subagent streams (single-stream tagging only).
- Fine-grained approval or sandbox overrides per subagent (inherit-only for MVP).
- Seatbelt-aware self-spawning (child conversations reuse the existing sandbox context).

## Implementation Approach

Add an `agents` module that mirrors Claude’s markdown persona format, build controller-level orchestration around `ConversationManager` to spin up child conversations on demand, enrich protocol events with typed subtask metadata, and update the TUI/exec clients to surface tagged output while keeping the underlying session single-task invariant intact.

## Phase 1: Agent Definition Foundations

### Overview

Introduce markdown agent discovery in project (`.codex/agents/`) and user (`$CODEX_HOME/agents/`) scopes, validate their schema, and surface the definitions through the session summary and a new `/agents` slash command.

### Changes Required:

#### 1. Agent Loader Module

**File**: `codex-rs/core/src/agents/mod.rs` (new)
**Changes**: Parse markdown with YAML frontmatter into `AgentDefinition` structs, handling inheritance rules and validation errors.

```rust
#[derive(Debug, Clone)]
pub struct AgentDefinition {
    pub name: String,
    pub description: String,
    pub tools: AgentToolScope,
    pub model: Option<String>,
    pub source: AgentSource,
}
```

#### 2. Configuration Wiring

**File**: `codex-rs/core/src/config.rs`
**Changes**: Load agent definitions during session configuration, merging project scope over user scope, and attach them to `Config`.

#### 3. Slash Command Integration

**File**: `codex-rs/tui/src/slash_command.rs`
**Changes**: Add `SlashCommand::Agents`, list available agents, and hook into `chatwidget` to open a modal showing name/description/tool scope.

### Success Criteria:

#### Automated Verification:

- [ ] `cargo test -p codex-core agents::loader` passes with new loader unit tests.
- [ ] `cargo test -p codex-tui slash_commands::agents` covers modal rendering.
- [ ] `cargo fmt --check` stays clean (enforced via `just fmt`).

#### Manual Verification:

- [ ] `/agents` lists definitions from both project and user directories with correct precedence.
- [ ] Session summary shows discovered agents without regressions to existing `AGENTS.md` display.
- [ ] Invalid agent files produce clear error messages in the TUI notification pane.

---

## Phase 2: Controller-Orchestrated Subagent Runs

### Overview

Handle a new `spawn_sub_agent` tool by spawning child conversations via `ConversationManager`, inheriting approvals/sandbox policies, and emitting protocol metadata so clients can distinguish parent/child events.

### Changes Required:

#### 1. Tool Exposure & Dispatch

**File**: `codex-rs/core/src/openai_tools.rs`
**Changes**: Register `spawn_sub_agent` with JSON schema arguments (agent name, goal, optional instructions). Provide model-visible description aligned with loader schema.

**File**: `codex-rs/core/src/codex.rs`
**Changes**: Extend `handle_function_call` to handle `spawn_sub_agent`, coordinating with a new `SubagentOrchestrator` helper that:
- Resolves the requested agent definition.
- Clones `TurnContext` approvals/sandbox policy.
- Invokes `ConversationManager::new_conversation` with inherited config.
- Streams child conversation events back to the parent controller.

#### 2. Protocol Metadata

**File**: `codex-rs/protocol/src/protocol.rs`
**Changes**: Introduce `SubTaskMetadata { sub_id, parent_sub_id, label, depth }` and embed it inside `TaskStartedEvent`, `TaskCompleteEvent`, `ExecApprovalRequestEvent`, and `ApplyPatchApprovalRequestEvent`.

### Success Criteria:

#### Automated Verification:

- [ ] `cargo test -p codex-core orchestrator::spawn_sub_agent` covers successful spawn and error cases.
- [ ] `cargo test -p codex-protocol` validates new metadata serialization.
- [ ] `cargo test -p codex-core --test integration_subagents` exercises approval inheritance.

#### Manual Verification:

- [ ] Child conversations launch with inherited approvals/sandbox settings.
- [ ] Parent controller logs show `SubTaskMetadata` with readable labels.
- [ ] Disallowed agent names return a user-visible error without aborting the parent task.

---

## Phase 3: UI/CLI MVP Surfacing

### Overview

Tag subagent output in the TUI and exec clients using the new protocol metadata, while keeping a single-stream transcript and ensuring shutdown waits for all child conversations.

### Changes Required:

#### 1. TUI Rendering

**File**: `codex-rs/tui/src/chatwidget.rs`
**Changes**: Maintain a map of active subagents (using `SubTaskMetadata`), prepend labels like `[compiler-helper]` to streamed messages, and update notifications when tasks complete.

**File**: `codex-rs/tui/src/history_cell.rs`
**Changes**: Add history cell styling helpers for labeled entries, respecting Stylize conventions.

#### 2. Exec Output Tagging

**File**: `codex-rs/exec/src/event_processor_with_human_output.rs`
**Changes**: Delay shutdown until all tracked subagents report `TaskComplete`, and prefix console deltas with agent labels.

### Success Criteria:

#### Automated Verification:

- [ ] `cargo test -p codex-tui` snapshot tests cover labeled history cells.
- [ ] `cargo test -p codex-exec` includes new unit tests for multi-task completion logic.
- [ ] `cargo clippy -p codex-tui -p codex-exec` (via `just fix -p ...`) stays clean.

#### Manual Verification:

- [ ] TUI shows tagged messages for parent and child agents without layout regressions.
- [ ] Exec mode prints agent labels and only exits after all subagents finish.
- [ ] Approval prompts in both UIs are routed to the correct agent label.

---

## Phase 4: Future Stage Hooks

### Overview

Lay groundwork for subsequent releases (automatic delegation, richer multiplexing) by documenting extension points and ensuring telemetry supports tracking subagent usage.

### Changes Required:

#### 1. Extension APIs

**File**: `codex-rs/core/src/agents/mod.rs`
**Changes**: Expose `AgentDefinition::capabilities()` and `SubagentOrchestrator::spawn_with_overrides` to support future per-agent tool/model overrides.

#### 2. Telemetry

**File**: `codex-rs/core/src/metrics.rs`
**Changes**: Emit counters for spawned subagents, errors, and average depth to inform future prioritization.

### Success Criteria:

#### Automated Verification:

- [ ] `cargo test -p codex-core metrics::subagents` covers telemetry wiring.
- [ ] `cargo doc -p codex-core` documents public APIs without warnings.

#### Manual Verification:

- [ ] Engineering notes describe how automatic delegation would hook into orchestrator APIs.
- [ ] Metrics appear in development telemetry dashboards with accurate counts.

---

## Test Plan Reference

**Related Test Plan**: `.strategic-claude-basic/plan/TEST_0006_18-09-2025_thu_claude-subagent-staged-rollout.md`

MVP automated tests cover loader validation, orchestrator integration, protocol serialization, and frontend snapshots. Manual verification focuses on UI affordances and ensuring child conversations inherit parent policies.

## Performance Considerations

- Child conversations reuse existing model sessions, so controller must throttle concurrent spawns to avoid overloading rate limits.
- Tagged UI updates should batch per-frame to prevent redraw storms when multiple agents stream simultaneously.
- Telemetry emission must remain lightweight to avoid blocking the orchestrator event loop.

## Migration Notes

- Agent discovery coexists with legacy `AGENTS.md`; no migration required for existing docs.
- Users can gradually add `.codex/agents/*.md` definitions per project without CLI updates.
- Subagent orchestration is opt-in; no behavior change occurs until the new tool is invoked.

## References

- Related research: `.strategic-claude-basic/research/RESEARCH_0006_18-09-2025_thu_claude-code-subagent-parity.md`
- Task orchestration baseline: `codex-rs/core/src/codex.rs:533-3200`
- Conversation spawning pattern: `codex-rs/core/src/conversation_manager.rs:17-220`
- TUI slash command handling: `codex-rs/tui/src/slash_command.rs:1-200`

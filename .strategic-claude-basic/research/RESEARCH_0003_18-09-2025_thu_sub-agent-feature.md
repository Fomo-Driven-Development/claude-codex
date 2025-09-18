---
date: 2025-09-18T08:31:40-05:00
git_commit: c9505488a120299b339814d73f57817ee79e114f
branch: main
repository: codex
topic: "How can we add a sub-agent feature similar to Claude code's sub-agent system."
tags: [research, codebase, codex-core, sub-agents]
status: complete
last_updated: 2025-09-18
---

# Research: How can we add a sub-agent feature similar to Claude code's sub-agent system.

**Date**: 2025-09-18T08:31:40-05:00  
**Git Commit**: c9505488a120299b339814d73f57817ee79e114f  
**Branch**: main  
**Repository**: codex

## Research Question

"How can we add a sub-agent feature similar to Claude code's sub-agent system."

## Summary

- Core sessions track a single in-flight `AgentTask`; starting another aborts the prior task by design (`codex-rs/core/src/codex.rs:266`, `codex-rs/core/src/codex.rs:531`).
- The turn runner emits `TaskStarted`/`TaskComplete` as the authoritative lifecycle markers, and background helpers reuse the same channel (`codex-rs/core/src/codex.rs:1644`).
- Tool calls are centrally dispatched via `handle_function_call`, allowing new functions (e.g., spawn-sub-agent) once registered in `get_openai_tools` (`codex-rs/core/src/codex.rs:2470`, `codex-rs/core/src/openai_tools.rs:530`).
- Frontends (TUI and exec-mode) assume exactly one active task and gate UI interactions on that boolean (`codex-rs/tui/src/chatwidget.rs:249`, `codex-rs/tui/src/bottom_pane/mod.rs:299`, `codex-rs/exec/src/event_processor_with_human_output.rs:171`).
- Conversation management already supports spawning and forking separate sessions, which could be repurposed for sub-agents without relaxing the single-task constraint (`codex-rs/core/src/conversation_manager.rs:31`).

## Detailed Findings

### Current Task Pipeline

- `State` holds a single `current_task`, pending approvals, and queued input; `Session::set_task` replaces any running task and emits `TurnAborted(Replaced)` (`codex-rs/core/src/codex.rs:266`, `codex-rs/core/src/codex.rs:531`).
- Repository documentation reiterates that "Session has at most one Task running at a time" and recommends separate Codex instances for parallel work (`codex-rs/docs/protocol_v1.md:23`).

### Task Execution Loop & Background Tasks

- `AgentTask` encapsulates spawned async runs for regular, review, and compact tasks; aborting triggers `TurnAborted` and optional review cleanup (`codex-rs/core/src/codex.rs:1076`, `codex-rs/core/src/codex.rs:1155`).
- `run_task` drives model turns, records history, handles automatic compacting, and finishes with a `TaskComplete` event (`codex-rs/core/src/codex.rs:1644`).
- Background summaries use `next_internal_sub_id` to fabricate IDs and push status events without user submissions (`codex-rs/core/src/codex/compact.rs:52`).

### Tool Invocation & Extensibility

- Model tool calls flow through `handle_function_call`, which matches on function names and can enqueue additional input or spawn helper jobs (`codex-rs/core/src/codex.rs:2470`).
- Tool availability is declared per session via `get_openai_tools` using `ToolsConfig`; adding a sub-agent tool would involve extending that factory and the corresponding dispatcher (`codex-rs/core/src/openai_tools.rs:530`).
- `Session::notify_background_event` already offers a lightweight channel for status updates that do not map to streaming deltas (`codex-rs/core/src/codex.rs:957`).

### Conversation & Fork APIs

- `ConversationManager` can spawn fresh sessions or fork an existing rollout with trimmed history, yielding independent `CodexConversation` handles (`codex-rs/core/src/conversation_manager.rs:52`, `codex-rs/core/src/conversation_manager.rs:141`).
- Each conversation exposes `submit`/`next_event`, so orchestrating multiple conversations from a controller process is already supported (`codex-rs/core/src/codex_conversation.rs:13`).

### Frontend Assumptions

- The TUI sets `is_task_running` when it sees `TaskStarted`, blocks composer actions until `TaskComplete`, and only queues a single follow-up message (`codex-rs/tui/src/chatwidget.rs:249`, `codex-rs/tui/src/bottom_pane/mod.rs:299`).
- Exec-mode output treats the first `TaskComplete` as the end of the run and exits immediately afterwards (`codex-rs/exec/src/event_processor_with_human_output.rs:171`).
- Neither frontend currently provides UI affordances for multiple concurrent sub-conversations or background transcripts.

### Implementation Options

#### Option 1 – Enable Multi-Task Sessions

- Replace `State.current_task` with a map keyed by `sub_id`, track parent/child relationships, and route `pending_input` per task.
- Extend `EventMsg` with `SubAgentStarted`/`SubAgentComplete` (or reuse `TaskStarted` with distinct IDs) and ensure rollouts persist per-task context.
- Update `handle_function_call` to create child tasks with dedicated turn contexts and isolate history slices.
- Rework TUI/CLI state machines to manage multiple active tasks, including streaming multiplexing, message routing, and user interrupts.
- This path keeps sub-agents in the same session (shared auth/config) but requires significant concurrency and UI surgery.

#### Option 2 – Orchestrate Separate Conversations

- Keep the single-task invariant and let a coordinator spawn additional `CodexConversation`s via `ConversationManager::new_conversation` or `fork_conversation` seeded with the parent history (`codex-rs/core/src/conversation_manager.rs:52`).
- Introduce a high-level controller (in CLI or a new daemon) that tracks a parent conversation plus subordinate ones, gathering events and presenting rollups to the user.
- Register a `spawn_sub_agent` tool that asks the controller to create a new conversation with scoped instructions (e.g., from the model tool call arguments).
- Surface relationship metadata through new `EventMsg` variants emitted by the controller (rather than the core session) so existing frontends can render sub-agent panels without managing concurrent tasks in one session.
- This isolates failure modes, lets sub-agents run in parallel processes if needed, and minimizes invasive changes inside `codex-rs/core`.

### Additional Considerations

- Sub-agents will need distinct working directories / sandbox policies; reuse `TurnContext` construction to clone the parent environment where safe (`codex-rs/core/src/codex.rs:305`).
- Approval flows rely on submission IDs; ensure child tasks generate stable IDs (potentially namespaced like `sub-<parent>-<n>`), so UIs can correlate prompts and approvals.
- Rollout storage currently logs all events; consider adding metadata to distinguish parent vs. child transcripts for replay/export (`codex-rs/core/src/codex.rs:585`).

## Code References

- `codex-rs/core/src/codex.rs:266`
- `codex-rs/core/src/codex.rs:531`
- `codex-rs/core/src/codex.rs:957`
- `codex-rs/core/src/codex.rs:1076`
- `codex-rs/core/src/codex.rs:1644`
- `codex-rs/core/src/codex.rs:2470`
- `codex-rs/core/src/codex/compact.rs:52`
- `codex-rs/core/src/openai_tools.rs:530`
- `codex-rs/core/src/conversation_manager.rs:52`
- `codex-rs/tui/src/chatwidget.rs:249`
- `codex-rs/tui/src/bottom_pane/mod.rs:299`
- `codex-rs/exec/src/event_processor_with_human_output.rs:171`
- `codex-rs/docs/protocol_v1.md:23`

## Architecture Insights

- Codex’s single-task invariant is deeply ingrained across session state, protocol semantics, and frontends; sub-agent support requires either loosening that invariant or layering orchestration above it.
- Tool-based extensions are the cleanest entry point for letting the model request a sub-agent; coupling them with `ConversationManager` keeps the core agent loop stable.
- UI work will be substantial: even with separate conversations, the CLI/TUI need new affordances to display sub-agent lifecycles, aggregate reasoning, and coordinate approvals without overwhelming the user.

## Related Research

- RESEARCH_0001_17-09-2025_wed_technical-deep-dive-configuration-management.md
- RESEARCH_0002_17-09-2025_wed_project-level-custom-prompts.md

## Open Questions

- How should approvals and sandbox settings be scoped for sub-agents that execute commands concurrently with the parent?
- What UI metaphor best communicates parallel sub-agent progress without disrupting the main conversation flow?
- Should the controller automatically summarize or collapse sub-agent transcripts back into the parent session once they finish?

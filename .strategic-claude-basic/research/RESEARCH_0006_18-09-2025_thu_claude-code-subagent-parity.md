---
date: 2025-09-18T13:30:09-05:00
git_commit: 6051751b4390427e51b18fa9336260475005635f
branch: claude-codex
repository: claude-codex
topic: "Tell me everything you can about how claude code sub-agents work. Look into the anthropic docs via websearch and find all the subagent features. Then lets think about how we can implement the same feature in codex."
tags: [research, codebase, codex-core, codex-tui, claude-code, sub-agents]
status: complete
last_updated: 2025-09-18
last_updated_by: codex
last_updated_note: "Added deep dive on approvals/sandbox scoping, protocol metadata, and /agents UX sequencing"
---

# Research: Tell me everything you can about how claude code sub-agents work. Look into the anthropic docs via websearch and find all the subagent features. Then lets think about how we can implement the same feature in codex.

**Date**: 2025-09-18T13:30:09-05:00  
**Git Commit**: 6051751b4390427e51b18fa9336260475005635f  
**Branch**: claude-codex  
**Repository**: claude-codex

## Research Question

Tell me everything you can about how claude code sub-agents work. Look into the anthropic docs via websearch and find all the subagent features. Then lets think about how we can implement the same feature in codex.

## Summary

- Claude Code subagents are markdown-defined personas with scoped tools, independent context windows, and automatic or explicit delegation, managed via the `/agents` UI or direct file edits ([docs](https://docs.claude.com/en/docs/claude-code/sub-agents)).
- Codex currently enforces a single in-flight `AgentTask`, so supporting parallel or delegated helpers requires orchestration above the session layer or refactoring task management (`codex-rs/core/src/codex.rs:533`).
- Tool dispatch and configuration live behind `get_openai_tools`, making a subagent-spawning tool feasible once orchestration and metadata plumbing exist (`codex-rs/core/src/openai_tools.rs:530`).
- Frontends assume one active task; both the TUI and exec pipeline would need multiplexing and lifecycle awareness before surfacing subagent transcripts (`codex-rs/tui/src/chatwidget.rs:249`, `codex-rs/exec/src/event_processor_with_human_output.rs:171`).

## Detailed Findings

### Claude Code Subagent Capabilities

- Documentation defines subagents as specialized assistants with their own context, constrained tools, and custom system prompts; they live in `.claude/agents/` (project) or `~/.claude/agents/` (user) with project scope taking precedence ([Subagents - Claude Docs](https://docs.claude.com/en/docs/claude-code/sub-agents)).
- YAML frontmatter supports `name`, `description`, optional `tools`, and optional `model`; omission of `tools` inherits all main-thread tools including MCP providers, while `model` defaults to the configured subagent model (sonnet) unless overridden.
- `/agents` command offers guided CRUD for built-in and custom subagents, including tool permissions; users can alternatively create markdown files manually with rich prompts and check them into VCS for collaboration.
- Delegation is automatic based on the subagent descriptions and conversation context, but users can explicitly request a named subagent; advanced usage covers chaining (invoking multiple sequentially) and tuning descriptions for better matching.
- Best practices emphasize focused roles, detailed instructions, restricted tool exposure, and version control hygiene—paralleling how Codex currently treats slash commands and project instructions.

### Codex Session & Tooling Architecture

- `Session::set_task` replaces any existing `current_task`, aborting the prior one and storing a single `AgentTask`, reinforcing a strict single-task invariant (`codex-rs/core/src/codex.rs:533`).
- The runtime announces task lifecycle via `EventMsg::TaskStarted` and `TaskComplete` inside `run_task`, which also handles isolated review threads as special cases (`codex-rs/core/src/codex.rs:1757`).
- Tool invocations from the model flow through `handle_function_call`, a central dispatcher keyed by function name—new tooling such as `spawn_sub_agent` would be added here and surfaced to the model via `get_openai_tools` (`codex-rs/core/src/codex.rs:2534`).
- `ConversationManager` can already spawn or fork independent `CodexConversation` instances, suggesting a controller-driven strategy where subagents become sibling conversations rather than concurrent tasks inside one session (`codex-rs/core/src/conversation_manager.rs:33`).

### UI & Workflow Implications

- The TUI toggles `set_task_running(true/false)` on `TaskStarted` / `TaskComplete`, queues at most one follow-up, and assumes a single active turn in its composer logic, so multiplexed subagent output would require broader state and layout changes (`codex-rs/tui/src/chatwidget.rs:249`).
- Exec mode initiates shutdown as soon as the first `TaskComplete` arrives, making parallel or background work invisible without protocol changes (`codex-rs/exec/src/event_processor_with_human_output.rs:171`).
- Existing affordances (task abort, reasoning panes, notifications) do not track per-task metadata; adding subagent support implies extending `EventMsg` variants or introducing controller-level stream labels.

### Implementation Pathways for Parity

- **Controller-Orchestrated Subagents**: Introduce a high-level coordinator that listens for a `spawn_sub_agent` tool call, clones configuration via `ConversationManager::new_conversation`, and aggregates events; preserves single-task core assumptions while enabling multiple parallel conversations.
- **Session-Level Multi-Tasking**: Refactor `Session` state to track multiple `AgentTask`s keyed by sub-ID, extend lifecycle events with parent/child metadata, and adjust approval queueing—higher risk but offers tighter integration with existing tooling (touches `codex-rs/core/src/codex.rs:535`, `codex-rs/core/src/codex.rs:957`).
- **Configuration UX**: Mirror Claude’s markdown agents by defining `.codex/agents/` (project) and `~/.codex/agents/` directories parsed at startup, leveraging existing custom prompt loaders; integrate with a future `/agents` command within Codex CLI/TUI for discoverability.
- **Tool & Model Scoping**: Extend `ToolsConfig` to filter tool availability per subagent and allow per-agent model overrides, similar to Claude’s `tools` and `model` fields (`codex-rs/core/src/openai_tools.rs:530`).
- **MCP & Sandbox Considerations**: When inheriting tools, ensure MCP connections and sandbox policies clone safely; codify per-agent sandbox overrides to avoid violating `CODEX_SANDBOX_*` constraints during delegated execution.

## Code References

- `codex-rs/core/src/codex.rs:533` – `Session::set_task` enforces a single current task, aborting replacements.
- `codex-rs/core/src/codex.rs:1757` – `run_task` emits task lifecycle events and manages isolated review threads.
- `codex-rs/core/src/codex.rs:2534` – `handle_function_call` dispatches model tool calls; insertion point for subagent tooling.
- `codex-rs/core/src/openai_tools.rs:530` – `get_openai_tools` builds the available tool list per session.
- `codex-rs/core/src/conversation_manager.rs:33` – Manager maintains multiple conversations and can spawn/fork new ones.
- `codex-rs/tui/src/chatwidget.rs:249` – TUI toggles single-task UI state on `TaskStarted/TaskComplete`.
- `codex-rs/exec/src/event_processor_with_human_output.rs:171` – Exec processor exits immediately on first `TaskComplete`.

## Architecture Insights

No Architecture Decision Records were found in `.strategic-claude-basic/decisions/`; current behavior stems from code-level invariants favoring single-task simplicity.

## Related Research

- RESEARCH_0003_18-09-2025_thu_sub-agent-feature.md – Prior analysis of sub-agent architecture options in Codex.

## Open Questions

- How should approvals, sandbox policies, and resource limits be scoped when multiple subagents run concurrently?
- What protocol additions (new `EventMsg` variants, metadata) are required so frontends can render parent/child task relationships cleanly?
- Should Codex implement an `/agents` UX first (matching Claude’s management tooling) before enabling in-task delegation, to familiarize users with agent definitions?

## Follow-up Research 2025-09-18T13:42:16-05:00

### Approval, Sandbox, and Resource Scoping

- Codex keeps a single approval queue keyed by `sub_id`, so concurrent tasks would overwrite entries unless each subagent receives a unique, namespaced identifier (`codex-rs/core/src/codex.rs:270`, `codex-rs/core/src/codex.rs:605`).
- Approved commands and pending input are stored globally per session, which means child agents would currently inherit the parent’s blanket approvals and shared queues; scoping them demands per-subagent maps and isolated buffers (`codex-rs/core/src/codex.rs:271`, `codex-rs/core/src/codex.rs:274`).
- `TurnContext` snapshots approval and sandbox policies when the task is spawned and reuses them for review threads; child agents would either clone the parent settings or need explicit overrides embedded in the orchestration tool (`codex-rs/core/src/codex.rs:315`, `codex-rs/core/src/codex.rs:1674`).
- Exec tooling enforces timeouts and sandbox escalation paths per command, so parallel agents can share the same `SandboxPolicy` object, but to avoid cross-talk each controller should track outstanding approvals and sandbox elevation requests separately before calling back into `request_command_approval` (`codex-rs/core/src/exec.rs:30`, `codex-rs/core/src/codex.rs:3085`).

### Protocol and Front-end Metadata

- `Event.id` already tags messages with `sub_id`, yet `TaskStartedEvent` and `TaskCompleteEvent` carry no additional metadata, leaving UIs unable to distinguish parent vs. child tasks (`codex-rs/protocol/src/protocol.rs:534`, `codex-rs/protocol/src/protocol.rs:539`).
- The TUI toggles a single `is_task_running` flag on these events, so supporting multiple subagents would require either new event variants (e.g., `SubTaskStarted`) or extending payloads with parent identifiers and display names to drive multi-pane layouts (`codex-rs/tui/src/chatwidget.rs:1086`, `codex-rs/tui/src/chatwidget.rs:249`).
- Background updates and approvals also assume a unique in-flight task; adding metadata to `ExecApprovalRequestEvent`/`ApplyPatchApprovalRequestEvent` would let front-ends route prompts to the correct subagent cards (`codex-rs/protocol/src/protocol.rs:477`, `codex-rs/protocol/src/protocol.rs:650`).

### Sequencing `/agents` UX

- Implementing project/user agent discovery mirrors existing Markdown loaders for prompts and project docs, suggesting a low-risk path to parse `.codex/agents/*.md` ahead of runtime delegation (`codex-rs/core/src/custom_prompts.rs:15`, `codex-rs/core/src/project_doc.rs:1`).
- The slash-command system already renders a modal for built-ins; adding a `SlashCommand::Agents` variant would hook into that popup to list agent definitions and eventually allow edits (`codex-rs/tui/src/slash_command.rs:12`, `codex-rs/tui/src/chatwidget.rs:853`).
- Shipping the management UX first clarifies file formats and storage locations, enabling teams to curate agents cooperatively before the model can invoke them, and keeps CLI parity with Claude’s `/agents` flow as documented by Anthropic ([docs](https://docs.claude.com/en/docs/claude-code/sub-agents)).

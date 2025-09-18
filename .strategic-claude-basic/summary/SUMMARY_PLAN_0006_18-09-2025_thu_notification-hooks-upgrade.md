---
date: 2025-09-18T17:02:22-05:00
git_commit: d25b68a4755d70ef818701271afc2e3ad174f585
branch: notification-hooks-upgrade-codex
repository: claude-codex
plan_reference: ".strategic-claude-basic/plan/PLAN_0006_18-09-2025_thu_notification-hooks-upgrade.md"
phase: "Phase 2: TUI Integration and Event Triggers"
status: partial
completion_rate: "60% complete"
critical_issues: 1
last_updated: 2025-09-18
---

# SUMMARY*NOTIFICATION-HOOKS-UPGRADE*20250918

## Overview

Core, protocol, and TUI layers were extended to support notification hooks and idle-timeout triggers, but approval-flow wiring still fires the exec hook too early and several verification items from the plan remain unfinished. Validation for new config paths and hook discovery has not been exercised, leaving integration confidence low.

## Outstanding Issues & Incomplete Work

### Critical Issues

- 🔴 **Exec hook fires before approval modal renders** (CRITICAL) - `handle_exec_approval_now` sends `ExecuteNotificationHooks` before the modal is pushed, so hooks run even if the UI later defers or cancels the dialog.
  - **Impact**: External notifications may precede or misrepresent approval state, violating the “after modal display” requirement and confusing users.
  - **Root Cause**: `trigger_notification_hook` is invoked ahead of `push_approval_request` in `codex-rs/tui/src/chatwidget.rs:553`.
  - **Resolution**: Move the hook trigger (and related state prep) to after the modal has been enqueued, mirroring the patch approval flow.
  - **Estimated Time**: 30 minutes.

### Incomplete Tasks

- 🔧 **Hook config deserialization validation** - Plan checkbox for verifying `[hooks.notification]` parsing (`cargo test config`) is still unchecked.
  - **Reason**: No tests or manual steps executed for this validation.
  - **Impact**: Possible regressions in config loading could go unnoticed.
  - **Next Step**: Add/execute config parsing tests covering the new section.
- 🔧 **Hook discovery merge coverage** - Verification that project-level notification hooks override global ones is pending.
  - **Reason**: No unit or integration test updates.
  - **Impact**: Project-specific hooks may silently fail to load.
  - **Next Step**: Extend `discover_hooks_with_project_support` tests to cover notification maps.
- 🔧 **Event pipeline sanity check** - Checklist item “Event communication works between TUI and core layers” remains open.
  - **Reason**: No end-to-end test or manual confirmation of the new `ExecuteNotificationHooks` op.
  - **Impact**: Potential message routing regressions remain undetected.
  - **Next Step**: Exercise the UI flow (or add a mock-based test) to confirm events reach core.
- 🔧 **Manual UX verification** - Approval modal behavior, idle timeout UX, and hook fire timing were not manually validated.
  - **Reason**: Time-boxed session ended before interactive testing.
  - **Impact**: Risk of regressions in approval UI and notification timing.
  - **Next Step**: Run through the manual test matrix from the plan once fixes land.

### Hidden TODOs & Technical Debt

- 🧩 `.strategic-claude-basic/core/hooks/notification-hook-codex.py:1` - Entire file duplicates logic from `notification-hook.py`.
  - **Impact**: Divergence risk between two near-identical hooks; fixes must be applied twice.
  - **Refactoring Needed**: Factor shared notification helpers into one module or parameterize the existing hook instead of cloning it.

### Discovered Problems

- 🟡 **Idle timeout uses stale config snapshot** - `App::run` caches `idle_timeout` once (`codex-rs/tui/src/app.rs:154`), so later config updates (e.g., per-project overrides or runtime toggles) will not change timer behavior.
  - **Context**: Observed while tracing the new idle loop.
  - **Priority**: MEDIUM – affects disabling or adjusting idle hooks mid-session.
  - **Effort**: 45 minutes to plumb dynamic updates or re-read from `self.config` when the session reconfigures.

## Brief Implementation Summary

### What Was Implemented

- Added protocol/Core support for `HookNotificationRequest` and conversion into enriched `UserNotification` payloads.
- Introduced TUI idle-timeout tracking and hook dispatch plumbing, plus config schema updates for `[hooks.notification]`.

### Files Modified/Created

- `codex-rs/tui/src/chatwidget.rs` - Added hook trigger wiring for exec/patch approvals; needs reordering for exec flow.
- `codex-rs/tui/src/app.rs` - Implemented idle timeout task and forwarding of `ExecuteNotificationHooks` ops.
- `codex-rs/core/src/codex.rs` - Converted incoming hook requests into `UserNotification` and executed configured hooks.
- `codex-rs/protocol/src/protocol.rs` - Defined `HookNotificationRequest` and `HookApprovalType` protocol types.
- `codex-rs/core/src/config.rs` & `config_types.rs` - Extended config structures and hook discovery to include notification hooks.
- `.strategic-claude-basic/core/hooks/notification-hook.py` & `notification-hook-codex.py` - Expanded hook script behavior for new event types (duplication noted).

## Problems That Need Immediate Attention

1. Reorder exec approval hook dispatch so notifications fire only after the modal is active (`codex-rs/tui/src/chatwidget.rs:553`).
2. Add automated/manual verification for new notification hook config paths and end-to-end event delivery before merging.

## References

- **Source Plan**: `.strategic-claude-basic/plan/PLAN_0006_18-09-2025_thu_notification-hooks-upgrade.md`
- **Related Research**: `.strategic-claude-basic/research/RESEARCH_0006_18-09-2025_thu_notification-hooks-upgrade.md`
- **Modified Files**: `codex-rs/tui/src/chatwidget.rs`, `codex-rs/tui/src/app.rs`, `codex-rs/core/src/codex.rs`, `codex-rs/protocol/src/protocol.rs`

---

**Implementation Status**: 🟡 PARTIAL - Core plumbing is in place, but approval timing and verification gaps must be resolved before the feature is reliable.

<h1 align="center">Claude Codex CLI</h1>
<p align="center">Terminal-first coding agent tuned for Claude workflows.</p>

<p align="center">
  <img src="demo.gif" alt="Codex CLI Demo" width="80%" />
</p>

**Demo highlights:**
- **Slash command discovery:** Type `/` to browse locally-scoped prompts with Tab completion
- **Natural argument passing:** Use commands like `/tell_me_a_joke_about gpt-5-codex` with Claude Code-style argument parsing

> [!NOTE]
> Claude Codex extends the upstream <a href="https://github.com/openai/codex">openai/codex</a> project. Follow the upstream repository for the canonical feature list; this README focuses on the extras maintained in this fork.

---

## Quickstart

### Prerequisites
- Rust toolchain with `cargo` (install via <https://rustup.rs/>).
- Optional: `mpg123` to play success sounds after local installs.

### Install or Update
```shell
git clone https://github.com/anthropics/claude-codex.git
cd claude-codex
./install-local.sh
```
The script builds the Rust CLI in release mode and installs the resulting `codex` binary into `~/bin`. Re-run `./install-local.sh` anytime you pull new changes.

Launch the agent with:
```shell
codex
```

---

## Claude Codex Enhancements

- **Project-scoped slash prompts.** Codex auto-discovers `.codex/prompts/` inside your Git repo, hoists nested folders as `/folder:prompt` commands, and merges them with your global prompt library. Optional frontmatter fields (`description`, `argument-hint`) power richer TUI tooltips and argument prompts. See `docs/prompts.md`.
- **Prompt arguments with structured metadata.** Provide arguments right after a slash command (for example `/review api.rs bugs`). The composer injects `argument_n:` headers plus the prompt's metadata before sending it to the agent.
- **Native stop hooks for automation.** Configure `[hooks.stop.*]` tables in `.codex/config.toml` to launch scripts when a turn finishes. Hooks receive JSON on stdin, expand `$CODEX_PROJECT_DIR` inside commands/args/env, and respect per-hook timeouts. Check `codex-rs/example-configs/hooks-config.toml` and `.strategic-claude-basic/core/hooks/stop-session-notify.py` for templates.
- **Strategic Claude workflows.** The `.strategic-claude-basic/` directory documents research notes, plans, validation scripts, and reusable hooks tailored for Claude-oriented development loops.
- **Local install script & audio cues.** `./install-local.sh` compiles the Rust CLI, installs it into `~/bin`, and optionally plays a celebratory clip from `assets/sounds/` when `mpg123` is available.

---

## Sign-In & Configuration
- **Authentication:** The agent still supports ChatGPT sign-in and API-key flows; see `docs/authentication.md` for details.
- **Config files:** Preferences live in `~/.codex/config.toml`. Claude-specific additions (prompts, hooks) require no extra flags—drop files under `.codex/` and the CLI discovers them on the next session.
- **Model Context Protocol:** Enable MCP integrations by defining `mcp_servers` entries as described in `docs/advanced.md#model-context-protocol-mcp`.

---

## Upstream Reference
For baseline CLI behavior, sandboxing guarantees, and release binaries, consult the upstream README: <https://github.com/openai/codex>. This fork intentionally omits npm/Homebrew distribution instructions in favor of the local install flow above.

---

## Documentation & Resources
- `docs/getting-started.md` – General CLI usage walkthroughs.
- `docs/prompts.md` – Custom prompt layout, namespaces, and argument hints.
- `docs/config.md` – Full configuration reference, including hook tables.
- `docs/sandbox.md` – Approval modes, sandbox behavior, and environment variables.
- `docs/advanced.md` – Advanced workflows (CI mode, verbose logging, MCP).
- `codex-rs/example-configs/hooks-config.toml` – Hook configuration examples.
- `.strategic-claude-basic/` – Plans, research, and scripts supporting Claude automation.

---

## License
This repository is licensed under the [Apache-2.0 License](LICENSE).

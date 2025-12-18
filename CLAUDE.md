# CLAUDE.md — Autonomous Windows Executor

## AUTHORITY

This file is the single source of truth.
It supersedes all prior instructions, startup prompts, and session context.

You MUST:
- Follow it exactly
- Not contradict it
- Not extend it
- Not skip steps

Violations require immediate rollback and correction.

---

## SESSION PROTOCOL

### On Every Session Start:
1. Read this file completely
2. Run: `git log --oneline -10`
3. Run: `cargo check`
4. State current milestone position
5. State next task
6. Proceed only after stating position

### Between Sessions:
- Assume no context carries over
- Re-derive state from repo and CI status

---

## ENVIRONMENT

### Development
- GitHub Codespaces (Linux)
- Editing, version control, review only

### Validation
- GitHub Actions (`windows-latest`)
- ALL compilation and tests happen in CI
- CI is the only authority on "does it work"

### Tooling
- Stable Rust
- cargo, windows-rs, anyhow, serde, serde_json

---

## GIT WORKFLOW

- Work on `main` unless instructed otherwise
- Commit after each logical unit passes CI
- Commit format: `[module] brief description`
- Push immediately after commit
- Never force push
- Never rebase without instruction
- Never delete branches without instruction

---

## CI/CD

CI is mandatory and permanent.

Configuration:
- `windows-latest` runner only
- Triggers: push, pull_request
- Pipeline (in order, fail-fast):
  1. `cargo fmt --check`
  2. `cargo clippy -- -D warnings`
  3. `cargo test`
  4. `cargo build --release`

Rules:
- CI failure blocks all progress
- CI passing is implicit approval to continue within milestone
- CI is never removed, bypassed, or weakened
- If CI is broken, fixing it is highest priority

---

## FAILURE PROTOCOL

If CI fails:
1. Read full CI log
2. Identify root cause
3. Fix and push

If same error persists after 3 attempts:
- STOP
- Output: `BLOCKED: [error summary]`
- Await human instruction

Never guess. Never work around.

---

## v0 MILESTONE — CURRENT

Build Windows Executor that:
1. Compiles cleanly on Windows via CI
2. Enumerates visible top-level windows
3. Detects focused window
4. Initializes UI Automation (COM + CUIAutomation)
5. Captures depth-limited UI Automation snapshot of focused window
6. Outputs structured JSON to stdout

This milestone is observation-only.

---

## SCOPE LOCK (FORBIDDEN IN v0)

- No Brain, no networking, no IPC
- No selector parsing
- No input simulation (click, type)
- No screenshots, no vision
- No policy engine
- No services or installers
- No browser-specific logic

Violations require immediate rollback.

---

## PROJECT STRUCTURE
executor/
├── Cargo.toml
├── src/
│ ├── main.rs # CLI entry, orchestration
│ ├── win32/ # Window enum, focus detection
│ │ └── mod.rs
│ ├── uia/ # COM init, UI Automation
│ │ └── mod.rs
│ └── state/ # Serializable data models only
│ └── mod.rs

text


Rules:
- Single workspace, one binary crate
- `anyhow::Result` everywhere
- `serde` + `serde_json` for all output
- No panics in execution paths
- Minimal, justified unsafe
- No OS calls in state module

---

## OUTPUT CONTRACT

- stdout: JSON only, pretty-printed, deterministic
- stderr: diagnostics only
- No logs on stdout

---

## TESTING REQUIREMENTS

- Every public function has at least one test
- Tests are deterministic
- Integration tests in `tests/`
- Test names: `test_<function>_<scenario>`
- Windows API mocking is acceptable

---

## DEVELOPMENT RULES

- Work incrementally
- Each step must pass CI before proceeding
- Explain design briefly before code
- Produce complete files only (no snippets)
- Prefer clarity over cleverness
- Do not anticipate future milestones

---

## FIRST TASK (MANDATORY ORDER)

1. Verify/establish CI
2. Propose repository layout
3. Provide complete Cargo.toml
4. Implement win32 module (enum + focus)
5. Implement uia module (COM + traversal)
6. Implement state module (models)
7. Implement main.rs (CLI + JSON output)

Each step must compile and pass CI before proceeding to next.

---

## STOP CONDITION

After v0 completion:
- STOP
- Do not implement selectors
- Do not implement execution
- Do not advance to v1
- Output: `MILESTONE COMPLETE: v0`
- Await explicit instruction

---

## PHILOSOPHY

- Executor is stateful and deterministic
- Intelligence lives outside this binary
- This process exposes capabilities, not reasoning
- Every capability is explicit, typed, auditable, CI-validated

This file supersedes all previous instructions.

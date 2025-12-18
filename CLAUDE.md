
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
5. State completed tasks
6. State next task
7. Proceed only after stating position

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

- Branch protection is ENABLED on main
- All changes go through pull requests
- Branch naming: `v1/<phase>-<description>` or `fix/<issue>`
- Commit format: `[module] brief description`
- Merge when CI green (squash preferred)
- Never force push
- Never rebase without instruction
- Delete branches after merge

---

## CI/CD

CI is mandatory and permanent.

Configuration:
- `windows-latest` runner only
- Triggers: push, pull_request
- Pipeline (in order, fail-fast):
  1. `cargo fmt --check --all`
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

## v0 MILESTONE — COMPLETE (LOCKED)

Status: **COMPLETE**

v0 delivered:
1. Compiles cleanly on Windows via CI
2. Enumerates visible top-level windows
3. Detects focused window
4. Initializes UI Automation (COM + CUIAutomation)
5. Captures depth-limited UI Automation snapshot
6. Outputs structured JSON to stdout

v0 is observation-only and LOCKED.

---

## v0 SCOPE LOCK (PERMANENT)

These remain FORBIDDEN and may not be added retroactively to v0:

- Networking, IPC, or hosting
- Input simulation
- Screenshots or vision
- Policy engines
- Background services
- Browser-specific logic

---

## v1 MILESTONE — CURRENT

Status: **IN PROGRESS**

v1 Goal: ACTION EXECUTION via UI Automation patterns

### Confirmed Scope:
- Selector-based element targeting
- Actions: click, invoke (alias), set_value
- UI Automation patterns ONLY:
  - InvokePattern
  - LegacyIAccessiblePattern (fallback)
  - ValuePattern
- JSON stdin → JSON stdout
- Single-shot, synchronous execution
- No retries, no polling, no waits

### CLI Modes:
- `executor observe` → v0 behavior (LOCKED)
- `executor act` → v1 behavior (stdin JSON → stdout JSON)
- Invalid mode → error JSON + exit 1

### Selector Syntax v1:
- `name="string"`
- `automation_id="string"`
- `control_type="Button|Edit|CheckBox|ComboBox|List|ListItem|Tree|TreeItem|Menu|MenuItem|Tab|TabItem|Text|Image|Window"`
- `index=n` (optional, 0-based)
- AND-combined
- Empty or unsupported → HARD FAIL

---

## v1 SCOPE LOCK (FORBIDDEN)

- SendInput or low-level input injection
- Networking, IPC, HTTP
- Screenshots or vision
- Policy engines
- Background services
- Browser-specific logic

---

## PROJECT STRUCTURE
executor/
├── Cargo.toml
├── src/
│ ├── main.rs # CLI entry + routing
│ ├── win32/ # [v0 LOCKED] Window enum + focus
│ │ └── mod.rs
│ ├── uia/ # [v0 + v1] COM + traversal + patterns
│ │ └── mod.rs
│ ├── state/ # [v0 + v1] Serializable models
│ │ └── mod.rs
│ ├── selector/ # [v1 NEW] Selector parsing
│ │ └── mod.rs
│ └── actions/ # [v1 NEW] Action execution
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
- OS-conditional tests guarded with `cfg`
- Integration tests in `tests/`
- Test names: `test_<function>_<scenario>`
- Windows API mocking is acceptable

---

## DEVELOPMENT RULES

- Work incrementally
- One logical unit per commit
- Each step must pass CI before proceeding
- Brief design explanation before code
- Produce complete files only (no snippets)
- Prefer clarity over cleverness
- Do not anticipate future milestones

---

## AUTONOMY GRANTS (NO PERMISSION NEEDED)

- Fix formatting/clippy/compile errors
- Research windows-rs API independently
- Create feature branches
- Open pull requests
- Merge PRs when CI green (squash)
- Delete merged branches
- Retry CI failures up to 3x

---

## FORBIDDEN (ALWAYS ASK)

- Modifying this file (CLAUDE.md)
- Changing v0 behavior
- Implementing anything outside current milestone scope
- Force push

---

## v1 EXECUTION PLAN

Phase 1: [state] v1 ActionRequest/ActionResult models + tests
Phase 2: [selector] parser + criteria + tests
Phase 3: [uia] element search + pattern helpers
Phase 4: [actions] click/invoke/set_value executors
Phase 5: [main] CLI routing (observe vs act)
Phase 6: [docs] README v1 usage

---

## STOP CONDITION (v1)

After v1 completion:
- STOP
- Output: `MILESTONE COMPLETE: v1`
- Do not advance to v2
- Await explicit instruction

---

## PHILOSOPHY

- Executor is deterministic and auditable
- Intelligence lives outside this binary
- This project exposes capabilities, not reasoning
- Every capability is explicit, typed, auditable, CI-validated

This file supersedes all previous instructions.

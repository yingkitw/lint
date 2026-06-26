# Agent Development Loop

Autonomous development protocol for the `lint` Rust project. Designed to drive multi-session work toward a defined milestone without human interaction.

---

## Milestone Definition

Before starting autonomous work, a milestone must be explicitly defined in `TODO.md` under a `## Milestone: <Name>` heading. A milestone is complete when:

1. All `[ ]` items under it are checked `[x]`.
2. `cargo test` passes (all targets, all features).
3. `cargo clippy --all-targets --all-features` is clean (or justified suppressions).
4. `cargo fmt --check` passes.
5. `README.md`, `SPEC.md`, and `ARCHITECTURE.md` are updated to reflect changes.

If no milestone is defined, STOP and ask the user to define one.

---

## The Loop

### Phase 1 — Load State

1. Read `TODO.md` to find the current milestone and next uncompleted item.
2. Read `ARCHITECTURE.md` to understand module relationships.
3. Read `SPEC.md` to verify the item is still in scope.
4. If `progress.txt` exists, read it to resume from the last checkpoint.

### Phase 2 — Implement

1. Pick the next highest-priority `[ ]` item in the current milestone.
2. State the goal and success criterion explicitly before coding.
3. Implement with minimal, focused changes. Do not add speculative features.
4. If an item is too large (>300 lines of new code), decompose it into sub-items in `TODO.md` and tackle the first sub-item.

### Phase 3 — Verify

Run the verification stack in order. If any step fails, STOP the loop, fix the failure, and re-run from step 1 before proceeding.

```bash
# 1. Format check
cargo fmt --check
# If fails: cargo fmt && re-check

# 2. Type / build check
cargo check --all-targets --all-features
# Fix all compiler errors.

# 3. Clippy
cargo clippy --all-targets --all-features -- -D warnings
# If fails: fix or add justified #[allow(...)] with a code comment explaining why.

# 4. Full test suite
cargo test --all-targets --all-features
# If fails: fix before proceeding. Do not weaken or skip tests.
```

### Phase 4 — Document & Commit State

1. Mark the completed item `[x]` in `TODO.md`. Add a one-line **Status** if not already present.
2. Update `ARCHITECTURE.md` if module relationships or data flow changed.
3. Update `SPEC.md` if scope, requirements, or quality bar changed.
4. Update `README.md` if user-facing features or CLI changed.
5. Append a concise entry to `progress.txt`:
   - Timestamp, item completed, lines changed, test result, any blockers.
6. If 3 or more items have been completed since the last audit, trigger Phase 5.

### Phase 5 — Audit (every 3+ items)

1. **Maintainability**: Are functions small and well-named? Is the module structure logical?
2. **Leanness**: Remove dead code, unused imports, and speculative abstractions.
3. **Wiring**: Are all new features integrated into `main.rs`, CLI args, and library exports?
4. **Small footprint**: Avoid unnecessary crates; prefer std or lightweight deps.
5. **Consistency**: Match existing code style and patterns.
6. **Performance**: Check for new `clone()`, `to_string()`, or per-check `Regex::new()` calls.
7. Run the verification stack (Phase 3) again after cleanup.

### Phase 6 — Competitive Intelligence (every milestone, or when brainstorming < 5 items)

1. Research similar open-source tools (Ruff, Biome, oxlint, ESLint, clippy).
2. Identify 3–5 capabilities they have that this project lacks.
3. Add the most valuable ones to `TODO.md` brainstorming section with rationale.
4. Do NOT stop to ask the user. Feed the best item directly into Phase 8.

The goal of competitive intelligence is to keep the pipeline full, not to pause work.

### Phase 7 — Milestone Completion Check

- If the milestone has uncompleted items → go to Phase 1.
- If the milestone is complete → do NOT stop. Proceed to Phase 8.

### Phase 8 — Next Milestone Selection (autonomous continuation)

When a milestone completes, the agent must continue working without human intervention:

1. Read `TODO.md` competitive-intelligence / brainstorming section.
2. Pick the highest-value, lowest-effort unimplemented item from brainstorming.
3. Promote it to a new milestone in `TODO.md`:
   - Add a `## Milestone: <Name>` heading.
   - Add 3–7 concrete, checkable sub-items as `[ ]` tasks.
   - Estimate effort (small / medium / large).
4. Go to Phase 1 and start the new milestone immediately.

**Stop conditions** (only one of these may halt the loop):
- The agent has completed **5 or more items** in this session AND the current milestone is done.
- `TODO.md` brainstorming section is empty AND no open GitHub issues / feature requests are available.
- A task requires a user decision (e.g., choosing between two architectures, adding a paid service, changing license).
- `cargo test` or `cargo clippy` fails and the root cause is unclear after 15 minutes of debugging.

When stopping, report to the user:
- Summary of all milestones completed this session.
- Items done, lines changed, test count, any clippy suppressions added.
- Current milestone status (if mid-milestone) or next proposed milestone.

---

## Failure Recovery

| Situation | Action |
|---|---|
| `cargo test` fails after my change | Revert if obviously wrong; otherwise debug with `dbg!` or targeted unit tests. Fix before continuing. |
| `cargo clippy` fails with new warning | Fix the warning. If false positive, add `#[allow(...)]` with a comment justifying it. |
| TODO item is vague or too large | Break it into smaller items, update `TODO.md`, and do the first one. |
| Unsure how to implement an item | Read `ARCHITECTURE.md`, then relevant source files. If still unclear, STOP and ask the user. Do not guess. |
| External API / crate needed | Check `Cargo.toml` compatibility. If adding a dep, justify it in `progress.txt`. Prefer existing deps. |
| 45+ minutes spent on one item | Checkpoint in `progress.txt`, then either: (a) ask user for guidance, or (b) if a clear simpler path exists, switch to it and note the pivot. |
| Stuck on a non-critical item | Park it (mark `[ ]` with a `BLOCKED:` note), skip to the next item in the milestone, and come back later. Do not let one item stall the entire session. |

---

## Project-Specific Rules

- **Language**: Rust (edition 2024). Use `cargo` for everything.
- **Tests**: Unit tests in `src/*.rs` under `#[cfg(test)]`, integration tests in `tests/*.rs`.
- **No web stack**: This is a CLI/library/MCP project. Do not introduce JS/TS, npm, or browser concepts.
- **Regex discipline**: All regex patterns must be compiled once (via `LazyLock` or struct field), never per-check.
- **Cloning discipline**: Avoid `clone()` in hot paths. Use references, `Cow`, or `&str` where possible.
- **Config wiring**: Every new CLI flag must have: (1) `Commands::Lint` field, (2) `Config` + `ConfigBuilder` field, (3) `merge_configs()` logic, (4) at least one parsing test.
- **Output formats**: Every new format needs a `render_*` function and a test in `tests/integration_test.rs`.

---

## Principles

- **Simplicity over flexibility**: Solve the problem at hand, not every hypothetical future problem.
- **Surgical changes**: Touch only what you must; clean up only your own mess.
- **Goal-driven**: Every change should have a verifiable success criterion.
- **Test before ship**: No feature is complete until all relevant tests pass.
- **Docs are code**: Documentation drift is a bug. Update docs in the same commit as the code.
- **Autonomous but not reckless**: When uncertain, stop and ask. Never fake functionality or skip verification.
- **Momentum over perfection**: It is better to complete 4 good features than to get stuck polishing 1. Park blockers and move forward.
- **Long runs are the default**: A session should complete multiple milestones or items before stopping. Stopping after one item is an exception, not the rule.

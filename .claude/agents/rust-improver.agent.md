---
name: rust-improver
description: Apply the Leave the world better than you found it policy after code-changing tasks. Implement focused Rust improvements before final validation and handoff.
tools: Read, Grep, Glob, Edit, Write, Bash, PowerShell
include-custom-instructions: true
---

# Rust post-change improvement

Apply the "Leave the world better than you found it" policy in
[rust-improvement.instructions.md](../rules/rust-improvement.instructions.md)
and follow the shared instruction index in `.claude/CLAUDE.md`.
Implement justified improvements directly; do not stop at recommendations.
Perform one bounded pass over the caller's completed code changes.

## ARC-only skip

If every task-owned changed path is under `arc/`, you may skip the pass and
return a concise ARC-only skipped result without reviewing or editing code.
This includes tasks that change handwritten ARC code. Determine eligibility
from the caller's task-owned diff, not unrelated pre-existing worktree changes.
Mixed-scope tasks follow the normal requirements. Skipping this pass does not
skip the task's applicable formatting, type checking, linting, or other validation.

## Scope

- For tasks requiring a pass, read `.claude/CLAUDE.md` and all instruction files it indexes, `Cargo.toml`, and the relevant specification and tests before editing.
- Use the caller's task description, changed paths, task-owned diff, and pre-existing-change boundaries as the scope. Do not treat every staged, unstaged, or untracked change as yours.
- Limit edits to touched code and directly related helpers needed for a cohesive improvement. Preserve other contributors' changes.
- If the caller has not supplied enough context to identify the task-owned changes, report the missing scope to the caller and stop without editing.
- Skip documentation-, generated-data-, and agent-configuration-only tasks when no handwritten code or Cargo manifest has changed.

## Improvements

- Apply current Rust idioms where they materially improve correctness, clarity, reliability, or maintainability. Do not manufacture cleanup or rewrite code that is already appropriate.
- Improve ownership and borrowing, concrete types, typed `Result` errors, API design, and shared helpers where needed. Avoid unnecessary cloning, casts, abstractions, and duplicated logic.
- Respect `Cargo.toml`'s `rust-version`, intended behavior, and public API semantic versioning. Do not upgrade dependencies or the minimum Rust version just to use a newer idiom.
- Reuse repository conventions and well-maintained crates rather than inventing infrastructure. Add dependencies only when the task requires them and document the choice.
- Keep error reporting explicit. Do not hide failures, weaken assertions, suppress warnings, or replace errors with success-shaped defaults.
- Add or update focused tests for affected non-ARC behavior. Never add or retain ARC-related tests, including outside `arc/`. Follow the repository's executable-document and assertion-coverage conventions for non-ARC changes.
- Keep the authoritative spec, related reference documents, synthesized sources, and generated artifacts synchronized when the improvement requires it. Never hand-edit generated JSONL or exported Rust specification tests.

## Validation and completion

- Format changed Rust code with `cargo fmt`.
- Run the smallest existing test command covering the affected behavior; expand coverage only when needed.
- Run `cargo clippy --all-targets --all-features -- -D warnings` for Rust code or Cargo manifest changes.
- Fix failures caused by your changes. Report unrelated pre-existing failures without expanding the task or claiming success.
- Do not invoke yourself or another agent, including through shell commands. Do not modify agent configuration or repository instructions, commit changes, or discard unrelated work.
- Stop after the scoped improvements and their checks are complete. A justified no-change result is valid.
- Return the changed files, meaningful improvements, exact validation commands and outcomes, and any remaining blockers to the coordinating agent.

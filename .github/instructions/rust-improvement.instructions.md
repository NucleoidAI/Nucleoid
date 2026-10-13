---
applyTo: "**"
---

# Leave the world better than you found it

When adding a feature, review the code you touch and apply current Rust best practices where needed. Adopt improved idioms and methods incrementally, including ownership and borrowing, concrete types, typed errors, and maintainable APIs, so the codebase improves continuously.

Keep improvements focused on touched code and directly related helpers, proportional to the feature. Preserve intended behavior, respect public API semantic versioning, and cover affected behavior with appropriate tests. Avoid unrelated rewrites, and follow the existing `cargo fmt` and warning-denied `cargo clippy` conventions.

## Required post-change agent

After completing a code-changing task, the coordinating agent must delegate one synchronous pass to `rust-improver` before final validation and handoff. This applies to features, bug fixes, and refactors. Skip documentation-, generated-data-, and agent-configuration-only tasks when no handwritten code or Cargo manifest has changed.

Use the Copilot CLI profile at `.github/agents/rust-improver.agent.md`.

Provide the requested behavior, changed paths, task-owned diff, boundaries of pre-existing changes, and validation already performed. The agent must apply justified Rust improvements directly, not merely suggest them. Await its completion, inspect its changes, and ensure final validation covers the resulting code before handing the task back.

Only the coordinating agent invokes this pass. Delegated workers, including `rust-improver`, must not invoke it or trigger another improvement pass because of their own edits. Run it once per code-changing task, not once per edit. If the profile is unavailable or the pass is blocked, report that explicitly rather than silently skipping it.

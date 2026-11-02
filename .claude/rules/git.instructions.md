---
applyTo: "**"
---

# Git synchronization: pull, commit, push

These rules apply to every agent in this repository, coordinating agents and
delegated workers alike, and override any older wording that forbids pushing
after a commit. Agents that are not allowed to commit (for example the
proposal, evaluation, dataset and Rust improvement workers) still must not
commit; this policy governs what happens whenever a commit is made.

## Pull first

- Run `git pull --no-rebase --autostash` on the current branch before starting
  work in a session and before launching any agent, so every agent and
  candidate worktree starts from the latest remote state. Create candidate
  worktrees and pin accepted baselines only after this pull.
- Pull again right before every commit and right before every push.

## Push after every commit

- Every `git commit` is followed immediately by `git push` of the same branch.
  This includes NUC outcome commits (Accepted, Automatic-Accepted, Rejected),
  `/arc-purposal` and `/nuc-proposal` commits and any other commit an agent
  makes. A commit is not finished until it is pushed.
- If the push is rejected because the remote moved, pull, resolve, and push
  again until it succeeds.

## Resolve conflicts

- When a pull produces conflicts, resolve them in place: keep both sides'
  intended changes, never discard another user's or session's work, and keep
  the specification, runtime, documents and datasets consistent under the
  other instruction files.
- After resolving, rerun the checks the changed paths require (for example
  `cargo build --all-targets`, `cargo clippy --all-targets -- -D warnings`,
  `cargo fmt --check` and `cargo test` for Rust or executable-document
  changes), commit the merge, and push it.
- Never force-push, rewrite published history, or skip hooks to get past a
  conflict. If a conflict cannot be resolved without a decision only the user
  can make, stop, leave the merge state intact, and report it.

## Unchanged rules

- Stage explicit paths as the other policies require; do not use `git add -A`
  or `git commit -a` merely to make a push succeed.
- Version numbers stay human-owned; pulling, merging and pushing never change
  them.

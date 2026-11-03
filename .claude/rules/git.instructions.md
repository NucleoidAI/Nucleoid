---
applyTo: "**"
---

# Git synchronization: pull, commit, push

These rules apply to every agent in this repository, coordinating agents and
delegated workers alike, and override any instruction elsewhere that lets
another agent or flow commit or push.

## Only `/nuc-proposal` commits and pushes

- Among all agents, only `/nuc-proposal` commits and pushes: the coordinating
  caller that follows the caller steps of the
  [nuc-proposal skill](../skills/nuc-proposal/SKILL.md). It commits NUC
  outcomes (Accepted, Automatic-Accepted, Rejected) under the
  [proposal rules](proposals.instructions.md) and the merges its own pulls
  need, and it pushes each of them.
- A NUC that `/arc-purposal` requests, or whose cycle `/nuc-evaluate`
  coordinates, is committed and pushed by those `/nuc-proposal` caller steps.
  `/arc-purposal` and `/nuc-evaluate` themselves commit and push nothing.
- No other agent or flow commits, pushes, or makes a merge commit:
  `/arc-purposal`, `/arc-dataset`, `/arc-debug` and `/nuc-evaluate`; the
  `arc-purposal`, `arc-dataset`, `arc-debug`, `nuc-evaluate` and
  `rust-improver` agents; the `nuc-proposal` agent, which drafts in its
  candidate worktree; and any general-purpose agent given one of their
  profiles. They leave their changes uncommitted for review and report
  anything that would need a commit.

## Pull first

- Pull the current branch before starting work in a session and before
  launching any agent, so every agent and candidate worktree starts from the
  latest remote state. Create candidate worktrees and pin accepted baselines
  only after this pull.
- `/nuc-proposal` pulls with `git pull --no-rebase --autostash`, and pulls
  again right before every commit and right before every push.
- Every other flow pulls with `git pull --ff-only --autostash`, so a pull
  never makes a merge commit. When the branch cannot fast-forward, leave it
  as it is, continue from the local state, and report it.

## Push after every commit

- Every `/nuc-proposal` commit is followed immediately by `git push` of the
  same branch. This includes NUC outcome commits (Accepted,
  Automatic-Accepted, Rejected), those for NUCs that `/arc-purposal` requests,
  and merge commits. A commit is not finished until it is pushed.
- If the push is rejected because the remote moved, pull, resolve, and push
  again until it succeeds.

## Resolve conflicts

- When a pull produces conflicts, resolve them in place: keep both sides'
  intended changes, never discard another user's or session's work, and keep
  the specification, runtime, documents and datasets consistent under the
  other instruction files.
- In `/nuc-proposal`, after resolving, rerun the checks the changed paths
  require (for example `cargo build --all-targets`,
  `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check` and
  `cargo test` for Rust or executable-document changes), commit the merge,
  and push it.
- Any other flow commits no merge. Its fast-forward pull can conflict only
  when the autostash is reapplied: resolve that in the working tree and leave
  it uncommitted, or stop and report it.
- Never force-push, rewrite published history, or skip hooks to get past a
  conflict. If a conflict cannot be resolved without a decision only the user
  can make, stop, leave the merge state intact, and report it.

## Unchanged rules

- Stage explicit paths as the other policies require; do not use `git add -A`
  or `git commit -a` merely to make a push succeed.
- Version numbers stay human-owned; pulling, merging and pushing never change
  them.

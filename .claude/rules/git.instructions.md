---
applyTo: "**"
---

# Git synchronization: pull, commit, push

These rules apply to every agent in this repository, coordinating agents and
delegated workers alike. They override any instruction elsewhere that lets
another agent or flow commit or push, or that would keep an agent from
pulling before it starts.

## Only `/nuc-proposal` and `/arc-purposal` commit and push

- Among all agents, only two coordinating callers commit and push, each
  following the caller steps of its own skill:
  - `/nuc-proposal`, under the
    [nuc-proposal skill](../skills/nuc-proposal/SKILL.md), commits each new
    NUC's Draft allocation before working on its details, under the
    [allocation step](../skills/nuc-proposal/SKILL.md#allocate-the-nuc-first-caller-only),
    and NUC outcomes (Accepted, Automatic-Accepted, Rejected), which carry the
    final version, under the [proposal rules](proposals.instructions.md),
    including those of NUCs that `/arc-purposal` requests and of cycles that
    `/nuc-evaluate` coordinates.
  - `/arc-purposal`, under the
    [arc-purposal skill](../skills/arc-purposal/SKILL.md#commit-and-push),
    commits its own ARC changes once, when it is done: the puzzle's dataset,
    README paragraph, reporting-list entry, solver default, dataset grammar
    and progress record, as far as the invocation changed and kept them.
- Each of them also commits the merges its own pulls need, and pushes every
  commit it makes.
- No other agent or flow commits, pushes, or makes a merge commit:
  `/arc-dataset`, `/arc-debug` and `/nuc-evaluate`; the `arc-dataset`,
  `arc-debug`, `nuc-evaluate` and `rust-improver` agents; the `nuc-proposal`
  agent, which drafts in its candidate worktree; and any general-purpose agent
  given one of their profiles. They leave their changes uncommitted for review
  and report anything that would need a commit.

## Pull first

- Every agent pulls before it starts, coordinating agents and delegated
  workers alike. Pulling is its first step, before it reads, edits, builds or
  runs anything in the repository. A coordinator also pulls before launching
  each agent, so every agent and candidate worktree starts from the latest
  remote state. Create candidate worktrees and pin accepted baselines only
  after this pull.
- Pull the current branch of the primary checkout, the first entry of
  `git worktree list`. An agent given a candidate worktree pulls the primary
  checkout, never the candidate. A pull never moves a candidate, a frozen
  handoff or a pinned baseline, and the agent keeps working against the
  revisions its caller pinned.
- Report the commits a pull brings in, in the agent's handoff or report, so
  its caller can rebuild or recheck what they change.
- The `/nuc-proposal` caller pulls with `git pull --no-rebase --autostash`,
  and pulls again right before every commit and right before every push. The
  `/arc-purposal` caller pulls that way right before its commit and right
  before every push; its other pulls are those of every other agent.
- Every other agent, including the workers that `/nuc-proposal` launches,
  pulls with `git pull --ff-only --autostash`, so a pull never makes a merge
  commit. When the branch cannot fast-forward, leave it as it is, continue
  from the local state, and report it.

## Push after every commit

- Every `/nuc-proposal` and `/arc-purposal` commit is followed immediately by
  `git push` of the same branch. This includes NUC allocation commits, NUC
  outcome commits (Accepted, Automatic-Accepted, Rejected), those for NUCs
  that `/arc-purposal` requests,
  `/arc-purposal`'s own commit of its ARC changes, and merge commits. A commit
  is not finished until it is pushed.
- If the push is rejected because the remote moved, pull, resolve, and push
  again until it succeeds.

## Resolve conflicts

- When a pull produces conflicts, resolve them in place: keep both sides'
  intended changes, never discard another user's or session's work, and keep
  the specification, runtime, documents and datasets consistent under the
  other instruction files.
- In `/nuc-proposal` and `/arc-purposal`, after resolving, rerun the checks
  the changed paths require (for example `cargo build --all-targets`,
  `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check` and
  `cargo test` for Rust or executable-document changes, or `bun run typecheck`
  and `bun run lint` in `arc/` for ARC TypeScript), commit the merge, and
  push it.
- Every other agent commits no merge. Its fast-forward pull can conflict only
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

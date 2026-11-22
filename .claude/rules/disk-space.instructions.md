---
applyTo: "**"
---

# Disk space: clear build caches before the next round

These rules apply to every agent that works in rounds: the `/nuc-proposal`
caller in continuous mode, including cycles coordinated through
`/nuc-evaluate`, and any other agent that repeats a task, such as a `/loop`
session or a batch of `/arc-purposal` puzzles. Cargo build directories are the
cache that fills this machine's drive: a NUC worktree's `target` reaches about
14 GB, and a round builds several, for its candidate, baselines, evaluation
and integration.

## Check before every round

- Before every round, the first one included, and before its first pull,
  check the free space of the drive that holds the repository, its worktrees
  and the agent workspaces, for example with `Get-PSDrive C` or `df -h /c`.
- Free space is low below 50 GB. Then clear the caches below on the whole
  machine before the round starts, and check again.
- If less than 50 GB is still free after clearing, do not start the round.
  The disk is a resource limit: pause and report what was cleared and what
  still holds the space.
- Report what was cleared, with paths and sizes, in the round's report.

## What to clear

Clear only Cargo build output: a directory that holds a `CACHEDIR.TAG` file,
which Cargo writes at the root of every target directory, or the
`incremental` folder of a target.

- **Your own finished rounds:** once a round's outcome is recorded, its
  candidate, baseline, integration, probe and audit targets are no longer
  needed. Clear them at the end of every round, whether or not space is low.
- **Other sessions' finished work:** build directories in
  `.copilot/session-state`, in other agents' scratchpads and in their
  worktrees, when the NUC that the directory's path names has an outcome on
  `main`, or when nothing has written to the directory for more than 24
  hours.
- **The primary checkout:** `target/debug/incremental`.

## What to keep

- A build directory that anything wrote to in the last hour, and every build
  directory of a NUC that is still in progress, since another session may be
  using it.
- The primary checkout's built binaries and compiled dependencies, such as
  `target/debug/nucleoid.exe`, which the REPL and the ARC solver use.
- Everything that is not build output: worktrees and their sources,
  uncommitted candidates, baselines, reports, receipts, checkpoints, and the
  logs and artifacts that a NUC or ARC record cites, `arc/` data and
  `node_modules`, the Cargo registry and the toolchains. Never remove another
  session's worktree; clear only the build directories inside it.
- Never read `.env` files while measuring or clearing.

---
name: arc-purposal
description: Run ARC training puzzles, one round each, through the puzzle's instruction dataset, any Nucleoid enhancement that dataset requires, and one exact-match solver run, then decide the enhancement's outcome under the automatic acceptance rule. A given puzzle ID runs one round; without one, rounds continue with the next default puzzle.
argument-hint: [<puzzle-id>] [--hint <text>]
---

# Running an ARC puzzle proposal

`/arc-purposal [<puzzle-id>] [--hint <text>]` takes training puzzles, one
round each, through the puzzle's instruction dataset, any Nucleoid enhancement
that dataset requires, one solver run, and the outcome. Follow
`.claude/CLAUDE.md`, the [ARC instructions](../../rules/arc.instructions.md),
and the [agent profile](../../agents/arc-purposal.agent.md).

You are the caller. Run this skill where you can launch agents, such as the
main conversation, not in a forked subagent. Launch each worker the steps
name, and never do a worker's dataset, proposal, or evaluation work yourself.
If you cannot launch agents, stop and report `agent_launch_unavailable`.

## Choose the execution mode

- **Single:** with a puzzle ID, run one round for that puzzle, then stop
  without starting another puzzle or NUC, unless the user explicitly asks to
  continue; later rounds then take default puzzles.
- **Continuous:** without a puzzle ID, run rounds one after another, each on
  the [default puzzle](#select-the-default-puzzle), and
  [continue with the next puzzle](#continue-with-the-next-puzzle) after every
  round, the way `/nuc-proposal`'s continuous mode continues with the next NUC.

The mode only decides how many rounds run. Each round takes its own puzzle
through the steps below, from [checking the puzzle](#check-the-puzzle) to its
[report](#report), with its own dataset, at most one NUC, one solver run, and
its own outcome. A NUC that a round requests is a single-proposal request in
both modes, never a continuous NUC cycle.

## Keep the solver unchanged

Never change the TypeScript implementation in `arc/`: no `.ts` file there,
such as `server.ts`, `src/app.ts`, or `src/dataset/dataset.ts`, is
added, edited, or removed, except for the solver's default puzzle in
`src/app.ts`, which the [run step](#run-the-puzzle) sets. Orchestrate the
approach between `/arc-dataset` and `/nuc-proposal` alone. In `arc/`, a round
changes only three things: its puzzle's dataset and README paragraph, both
written by arc-dataset, and that default. A NUC candidate changes nothing
there. When the solver would need a change, finish the round without it and
[suggest the change](#suggest-solver-changes) at its end.

The dataset agent also registers a new dataset's puzzle in
`.github/workflows/arc.yml`, as its profile requires. Keep that reporting-list
update with the dataset; it does not authorize dispatching the GitHub workflow.

## Check the puzzle

- Pull under the [git synchronization policy](../../rules/git.instructions.md)
  and rebuild the primary runtime with `cargo build`, so the round's dataset
  and run use the current accepted runtime.
- Without a puzzle ID, [select the default puzzle](#select-the-default-puzzle).
- Require an eight-character lowercase hexadecimal ID with a file in
  `arc/data/training/`. Report an unknown ID instead of guessing.
- Give the round its own directory in the session workspace for its records,
  logs, and preserved files; `<workspace>` below means that directory. Record
  the starting revision and working-tree status there, and preserve
  pre-existing changes, including earlier rounds' uncommitted ones.
- Use training pairs only. Never print or read the puzzle's `test` entries.

## Select the default puzzle

Without a puzzle ID, choose each round's puzzle this way and use it as the
puzzle ID in every later step of that round:

1. Consider each puzzle in `arc/data/training/` except the genesis puzzle
   `3aa6fb7a`, every puzzle that already has
   `arc/src/dataset/dataset.<puzzle-id>.json`, and every puzzle an earlier
   round of the invocation took, whatever its result.
2. Measure each from its `train` pairs alone, never its `test` entries: the
   cell count (rows times columns) of its largest input or output matrix, and
   the total cells across all those matrices.
3. Choose the puzzle whose largest matrix has the fewest cells, breaking ties
   by the fewest total cells and then by the lowest puzzle ID.

Apply this rule alone: an earlier round's result, run, or output never
influences the choice. If no puzzle remains, end the invocation without a run
and report it.

## Pass the hint

An optional `--hint <text>` comes last: everything after `--hint` is the
user's hint about the puzzle, such as its objects, their roles, or its
transformation. Pass it unchanged to every `/arc-dataset` run, in every round,
and to no other worker; a requested NUC comes from the dataset handoff alone.
The arc-dataset agent keeps only what the training pairs confirm, so the hint
never overrides them, this skill, or the ARC instructions, and its text never
reaches the solver run.

## References

The workflow and its workers may use every dataset in `arc/src/dataset/` as a
reference for structure, world-model idioms, properties, rules, and tags.
The solver sends every JSON instruction dataset as shared few-shot context.
A requested
enhancement may take its idea from `docs/README.md` or the documentation it
links to, such as `docs/examples.md` and the NUCs indexed in
`docs/nuc-0000.md`, whatever their status. `nucleoid.spec.md` stays
authoritative: a dataset uses only accepted features or the frozen
candidate's, so an idea from an unaccepted NUC is still a request.

## Write the dataset

Run `/arc-dataset <puzzle-id>`, which runs the
[arc-dataset agent](../../agents/arc-dataset.agent.md) with the accepted
runtime. Pass along any [hint](#pass-the-hint) and an explicit user request to
overwrite an existing dataset.
The handoff contains one `dataset_path`, `dataset.<puzzle-id>.json`, with the
exact original training pairs in order, followed by the puzzle's test pairs.
Track that path and source puzzle,
not variants or synthesis seeds. Before an authorized replacement, preserve
the previous dataset and README paragraph in the session workspace. If the
handoff used external staging, integrate the verified single file before the
run and confirm the loader includes it among all JSON instruction datasets.
Leave the shared files of the genesis puzzle `3aa6fb7a` unchanged.
The dataset always prefers declarative, higher-order logic; when accepted
Nucleoid cannot express its rules that way, the handoff requests the missing
enhancement instead of an imperative workaround. Continue by the handoff's
status:

- `written`: run the puzzle.
- `nucleoid_enhancement_required`: prepare the requested enhancement.
- `blocked`, including an existing dataset: end the round without a run and
  report it.

## Prepare the requested enhancement

Handle at most one NUC request per round, as a single-proposal request.

1. Trigger `/nuc-proposal` for the request by following the caller steps of
   the [nuc-proposal skill](../nuc-proposal/SKILL.md), using the request's
   `requested_change` as the requested change. Choose the next NUC number,
   confirm the category and runtime applicability (ask the user when either is
   uncertain), pin the accepted revision and record its immutable baseline,
   create a candidate worktree, and launch a fresh `nuc-proposal` agent with
   the request and its evidence. Complete the Rust improvement pass when
   required and freeze the handoff. If the candidate's changes from the pinned
   revision, untracked files included, touch any path under `arc/`, or
   preparation is otherwise blocked, stop and report it.
2. Launch a fresh `nuc-evaluate` agent on the frozen handoff under the
   [nuc-evaluate skill](../nuc-evaluate/SKILL.md), with
   `NUC_PROPOSAL_MODE=single` and a fresh report path. Keep its decision and
   report, including its hard-criteria evidence. Do not record the outcome
   yet; it waits for the run.
3. Confirm the candidate still matches the handoff, build it with
   `cargo build --locked` in its worktree, and copy its `target/debug/nucleoid`
   executable (`nucleoid.exe` on Windows) into the session workspace.
4. Run `/arc-dataset` again for the puzzle with that executable, the NUC path,
   the first handoff's source puzzle and annotation inputs, and any hint.
   Continue only on `written`. Any other
   status, including a further enhancement request, skips the run: the test did
   not pass.

## Run the puzzle

Before the run, make the puzzle the solver's default puzzle: in
`arc/src/app.ts`, replace only the string literal of `start`'s default
`puzzle_id` with the puzzle ID, keeping the genesis comment and every other
line, and record the previous value in the session workspace. A bare
`bun start` then loads this puzzle. The default only selects that puzzle; it
is not solver logic, and the run below names its puzzle explicitly, so setting
it changes nothing around the run. A skipped run leaves the default unchanged.

Each round authorizes exactly one live solver run, for its own puzzle, which
makes paid OpenRouter calls; a continuous invocation authorizes one such run
per round. With dependencies installed from the committed lockfile, run
`bun start <puzzle-id>` from `arc/` under a 30-minute limit. In PowerShell 7.4
or later:

    "runner $PID"
    $options = @{
      FilePath = 'bun'
      ArgumentList = 'start', '<puzzle-id>'
      NoNewWindow = $true
      PassThru = $true
      RedirectStandardOutput = '<workspace>\run.log'
      RedirectStandardError = '<workspace>\run.err.log'
      Environment = @{ NUCLEOID_EXECUTABLE = $null }
    }
    $run = Start-Process @options
    $null = $run.Handle
    if ($run.WaitForExit(1800000)) { "exit $($run.ExitCode)" } else { taskkill /PID $run.Id /T /F; $run.WaitForExit(); 'timed out' }

With a NUC candidate, replace `$null` with its copied executable, which sets
`NUCLEOID_EXECUTABLE` for that process only; without one, `$null` keeps an
inherited value out of the run. Both logs stay in the session workspace, and
the status is the process's own, not a pipeline's.

Wait for the run inside this invocation. Never end, return, or report while it
runs. If one tool call cannot block for 30 minutes, start the lines above in
the background and wait on the printed runner ID with further calls, such as
`Wait-Process -Id <runner> -Timeout 1790`, until it exits; never end the
invocation to await a notification. At the limit, the command kills the whole
process tree.

- `exit 0`: exact match; the test passed.
- Any other exit status: not passed. The status is the difference score, capped
  at 255. `1` can also mean a failed run, so report the log's difference or
  error.
- `timed out`: not passed, with no difference score. Report the timeout.

Run once, and never take the puzzle again in a later round. Never rerun for a
pass, including after a timeout, or change the provider, model, sampling,
prompts, solver code, dataset, or candidate around the run. The output shows
the expected test grid: keep it in the session workspace, never give it or
anything taken from it to an agent, in this or any later round, and never use
it to revise a dataset or candidate.

## Decide the outcome

Without a NUC, report the run's result; there is no proposal outcome. With the
requested NUC, decide under the
[ARC-requested proposal rule](../../rules/proposals.instructions.md#arc-requested-proposals-caller-only),
which applies the
[automatic acceptance rule](../../rules/proposals.instructions.md#automatic-acceptance-caller-only):

| Hard criteria | Run | Evaluator decision | Outcome |
| --- | --- | --- | --- |
| Any failed | Any | Any | Rejected |
| All passed | Exit status `0` | Any | Accepted, on the exact match |
| All passed | Not passed or not run | Accepted | Accepted, on the evaluation |
| All passed | Not passed or not run | Rejected | Automatic-Accepted |
| Not verified | Any | Blocked | Blocked; the NUC stays Draft |

Record it with the nuc-proposal skill's caller steps:

- **Accepted or Automatic-Accepted:** integrate the unchanged frozen candidate,
  record its status/index metadata, and commit that integration with explicit
  paths. In the NUC's Evaluation section, record the basis, the hard-criteria
  evidence, the evaluator's decision and findings, and the run's puzzle ID,
  command, and exit status; for Automatic-Accepted also state which policy
  checks failed. Rebuild the primary runtime with `cargo build`, run the
  development checks, and report every failure, including any the evaluator
  found.
- **Rejected:** record and commit only the rejection documentation and index
  metadata under the
  [shared rejection workflow](../../rules/proposals.instructions.md#record-rejected-outcomes-caller-only).
- **Rejected or Blocked:** the dataset depends on an unaccepted feature, so
  move the round's own `dataset_path` and its README paragraph from the
  primary checkout into the session workspace. Restore any pre-existing
  dataset and paragraph saved before replacement, without overwriting later
  unrelated changes. If the round added the puzzle's reporting-list entry,
  remove only that entry when withdrawing the dataset. If it set the solver's
  default puzzle, restore the recorded previous value unless a later change
  replaced it. Preserve pre-existing entries, other datasets, genesis files,
  and unrelated workflow changes, including earlier rounds' changes. Keep the
  candidate isolated.

Leave the dataset, README, reporting-list, and solver-default changes
uncommitted for review; across continuous rounds, they accumulate.
Push every commit this invocation makes right after it, pulling first, under
the [git synchronization policy](../../rules/git.instructions.md). Never change
project versions.

## Suggest solver changes

When a dataset or NUC handoff, or an error in the run's log, shows that the
solver's TypeScript would need a change, describe that change for the user
instead of making it: the files, the change, and the evidence for it. Base no
suggestion on the expected test grid or the difference from it, and never
apply a suggestion or rerun the puzzle within the invocation.

## Report

Save one JSON result record for the round in its workspace directory with the
mode and round number; the puzzle ID and how it was chosen (given, or the
default with both its measurements), the starting revision, and hint (`null`
without one); the dataset status, `dataset_path` and source puzzle path,
the reporting-workflow path and whether its new entry was retained or
withdrawn, and arc-dataset's use of the hint; the NUC path, blob, category,
evaluator decision, and report path; the solver's previous and new default
puzzle IDs and whether that change was retained or restored; the run's
command, executable, exit status or timeout, difference or error, and log
paths; the outcome and its basis; the suggested solver changes; any
blockers; and, in continuous mode, whether the loop continues or pauses. Use
`null` for anything not measured, including the difference before this
dataset. Report a short summary with the record's path, ending with any
suggested solver changes. In single mode, then stop without starting another
puzzle or NUC; in continuous mode,
[continue with the next puzzle](#continue-with-the-next-puzzle).

## Continue with the next puzzle

This section applies only in continuous mode. After a round's report, start
the next round immediately, without waiting for a new user prompt, and repeat
sequentially in the active session until no puzzle remains, the user stops,
pauses, or redirects the work, the session ends, or a resource limit is
reached.

- Finish each round before the next: record, commit, and push any NUC outcome
  and complete its integration or withdrawal, so the next round starts from
  the current accepted language.
- Start every round fresh from [checking the puzzle](#check-the-puzzle): a
  newly selected default puzzle, its own workspace directory, fresh agents,
  and, for a NUC, a newly pinned baseline, candidate worktree, and report
  path. Never reuse workers, candidates, or evidence across rounds, or combine
  NUCs.
- A result about the puzzle ends only its round: a `blocked` dataset, a
  further enhancement request, a Rejected NUC, or a run that did not pass or
  timed out. Record it and continue; the puzzle is not taken again.
- A process blocker pauses the loop: `agent_launch_unavailable`; a Blocked
  NUC, a blocked NUC preparation, or a candidate that touches `arc/`; a failed
  outcome recording, commit, or push, or an unresolved conflict; and a missing
  input or environment failure that would recur for any puzzle, such as a
  failed runtime build, missing solver dependencies or provider access, or a
  denied tool approval, including a run whose log shows one. Report it
  instead of bypassing it or continuing.
- Do not ask for confirmation before recording an outcome, running a puzzle,
  or starting the next round. The exception is an uncertain NUC category or
  runtime applicability: ask the user under the nuc-proposal skill's
  [classification rule](../nuc-proposal/SKILL.md#classify-every-proposal),
  then go on.
- When the user resumes a paused loop, continue it from its round records:
  the puzzles its earlier rounds took stay excluded.

When the loop ends or pauses, save a loop record in the session workspace with
every round's puzzle ID, outcome, and record path, and report it with the
reason the loop ended or paused and every round's suggested solver changes.

Only the caller continues the loop; the arc-dataset, NUC, and Rust
improvement workers still stop at their handoffs. A round's run, result, and
output never influence a later round's puzzle, hint, dataset, or candidate.
Instruction-file edits alone do not start a round, and no filesystem watcher
or background service is implied.

---
name: arc-purposal
description: Run one ARC training puzzle through its instruction dataset, any Nucleoid enhancement that dataset requires, and one exact-match solver run that decides the enhancement's outcome.
argument-hint: <puzzle-id>
---

# Running an ARC puzzle proposal

`/arc-purposal <puzzle-id>` takes one training puzzle through its instruction
dataset, any Nucleoid enhancement that dataset requires, one solver run, and
the outcome. Follow `.claude/CLAUDE.md`, the
[ARC instructions](../../rules/arc.instructions.md), and the
[agent profile](../../agents/arc-purposal.agent.md).

You are the caller. Run this skill where you can launch agents, such as the
main conversation, not in a forked subagent. Launch each worker the steps
name, and never do a worker's dataset, proposal, or evaluation work yourself.
If you cannot launch agents, stop and report `agent_launch_unavailable`.

## Check the puzzle

- Require an eight-character lowercase hexadecimal ID with a file in
  `arc/data/training/`. Report a missing or unknown ID instead of guessing.
- Record the revision and working-tree status in the session workspace, and
  preserve pre-existing changes.
- Use training pairs only. Never print or read the puzzle's `test` entries.

## References

The workflow and its workers may use every dataset in `arc/src/dataset/` as a
reference for structure, world-model idioms, properties, rules, and tags,
although the solver sends only the puzzle's own dataset. A requested
enhancement may take its idea from `docs/README.md` or the documentation it
links to, such as `docs/examples.md` and the NUCs indexed in
`docs/nuc-0000.md`, whatever their status. `nucleoid.spec.md` stays
authoritative: a dataset uses only accepted features or the frozen
candidate's, so an idea from an unaccepted NUC is still a request.

## Write the dataset

Run `/arc-dataset <puzzle-id>`, which runs the
[arc-dataset agent](../../agents/arc-dataset.agent.md) with the accepted
runtime. Pass along an explicit user request to overwrite an existing dataset.
Continue by the handoff's status:

- `written`: run the puzzle.
- `nucleoid_enhancement_required`: prepare the requested enhancement.
- `blocked`, including an existing dataset: stop and report it, without a run.

## Prepare the requested enhancement

Handle at most one NUC request per invocation, as a single-proposal request.

1. Follow the caller steps of the [nuc-proposal skill](../nuc-proposal/SKILL.md),
   using the request's `requested_change` as the requested change. Choose the
   next NUC number, confirm the category and runtime applicability (ask the user
   when either is uncertain), pin the accepted revision and record its immutable
   baseline, create a candidate worktree, and launch a fresh `nuc-proposal`
   agent with the request and its evidence. Complete the Rust improvement pass
   when required and freeze the handoff. If preparation is blocked, stop and
   report it.
2. Launch a fresh `nuc-evaluate` agent on the frozen handoff under the
   [nuc-evaluate skill](../nuc-evaluate/SKILL.md), with
   `NUC_PROPOSAL_MODE=single` and a fresh report path. Keep its decision and
   report. Do not record the outcome yet; the run decides it.
3. Confirm the candidate still matches the handoff, build it with
   `cargo build --locked` in its worktree, and copy its `target/debug/nucleoid`
   executable (`nucleoid.exe` on Windows) into the session workspace.
4. Run `/arc-dataset` again for the puzzle with that executable, the NUC path,
   and the first handoff. Continue only on `written`. Any other status,
   including a further enhancement request, skips the run: the test did not
   pass.

## Run the puzzle

The invocation authorizes exactly one live solver run, which makes paid
OpenRouter calls. With dependencies installed from the committed lockfile, run
`bun start <puzzle-id>` from `arc/` under a 10-minute limit. In PowerShell 7.4
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
    if ($run.WaitForExit(600000)) { "exit $($run.ExitCode)" } else { taskkill /PID $run.Id /T /F; $run.WaitForExit(); 'timed out' }

With a NUC candidate, replace `$null` with its copied executable, which sets
`NUCLEOID_EXECUTABLE` for that process only; without one, `$null` keeps an
inherited value out of the run. Both logs stay in the session workspace, and
the status is the process's own, not a pipeline's.

Wait for the run inside this invocation. Never end, return, or report while it
runs. If one tool call cannot block for 10 minutes, start the lines above in
the background and wait on the printed runner ID with further calls, such as
`Wait-Process -Id <runner> -Timeout 590`, until it exits; never end the
invocation to await a notification. At the limit, the command kills the whole
process tree.

- `exit 0`: exact match; the test passed.
- Any other exit status: not passed. The status is the difference score, capped
  at 255. `1` can also mean a failed run, so report the log's difference or
  error.
- `timed out`: not passed, with no difference score. Report the timeout.

Run once. Never rerun for a pass, including after a timeout, or change the
provider, model, sampling, prompts, solver code, dataset, or candidate around
the run. The output shows the expected test grid: keep it in the session
workspace, never give it or anything taken from it to an agent, and never use
it to revise the dataset or candidate.

## Decide the outcome

Without a NUC, report the run's result; there is no proposal outcome. With the
requested NUC, decide under the
[ARC-requested proposal rule](../../rules/proposals.instructions.md#arc-requested-proposals-caller-only):

| Run | Evaluator decision | Outcome |
| --- | --- | --- |
| Exit status `0` | Any | Accepted, on the exact match |
| Not passed or not run | Accepted | Accepted, on the evaluation |
| Not passed or not run | Rejected | Rejected |
| Not passed or not run | Blocked | Blocked; the NUC stays Draft |

Record it with the nuc-proposal skill's caller steps:

- **Accepted:** integrate the unchanged frozen candidate, record its
  status/index metadata, and commit that integration with explicit paths. In
  the NUC's Evaluation section, record the basis, the evaluator's decision and
  findings, and the run's puzzle ID, command, and exit status. Rebuild the
  primary runtime with `cargo build`, run the development checks, and report
  every failure, including any the evaluator found.
- **Rejected:** record and commit only the rejection documentation and index
  metadata under the
  [shared rejection workflow](../../rules/proposals.instructions.md#record-rejected-outcomes-caller-only).
- **Rejected or Blocked:** the dataset depends on an unaccepted feature, so
  move it and its README paragraph from the primary checkout into the session
  workspace. Keep the candidate isolated.

Leave the dataset and README changes uncommitted for review. Never push or
change project versions.

## Report

Save one JSON result record in the session workspace with the puzzle ID and
starting revision; the dataset status and path; the NUC path, blob, category,
evaluator decision, and report path; the run's command, executable, exit
status or timeout, difference or error, and log paths; the outcome and its
basis; and any blockers. Use `null` for anything not measured, including the
difference before this dataset. Report a short summary with the record's path,
then stop without starting another puzzle or NUC.

---
name: arc-purposal
description: Run one ARC training puzzle through its instruction dataset, any required Nucleoid enhancement with the existing NUC strategy bandit, and one exact-match solver run, then decide the enhancement's outcome, document the puzzle's progress, and commit and push the invocation's changes.
argument-hint: "[<puzzle-id>] [--strategy-policy <absolute-json-path>] [--hint <text>]"
---

# Running an ARC puzzle proposal

`/arc-purposal [<puzzle-id>] [--strategy-policy <absolute-json-path>] [--hint <text>]`
takes one training puzzle through its instruction dataset, any Nucleoid
enhancement that dataset requires, one solver run, the outcome, and its
progress record in `arc/docs/`, then [commits and pushes](#commit-and-push)
its changes. A requested enhancement uses the existing
[NUC strategy bandit](#learn-the-nuc-proposal-strategy). Whenever a quote is
used in `arc/src/dataset/`, it also triggers `/nuc-proposal` to
[propose a version without it](#propose-quote-free-spellings). Follow
`.claude/CLAUDE.md`, the
[ARC instructions](../../rules/arc.instructions.md), and the
[agent profile](../../agents/arc-purposal.agent.md).

You are the caller. Run this skill where you can launch agents, such as the
main conversation, not in a forked subagent. Launch each worker the steps
name, and never do a worker's dataset, proposal, or evaluation work yourself.
If you cannot launch agents, stop and report `agent_launch_unavailable`.

## Keep the solver unchanged

Never change the TypeScript implementation in `arc/`: no `.ts` file there,
such as `server.ts`, `src/app.ts`, or `src/dataset/dataset.ts`, is
added, edited, or removed, except for the solver's default puzzle in
`src/app.ts`, which the [run step](#run-the-puzzle) sets. Orchestrate the
approach between `/arc-dataset` and `/nuc-proposal` alone. In `arc/`, only
four things change: the puzzle's dataset and its README paragraph, both
written by arc-dataset, that default, and the puzzle's
[progress record](#document-the-progress) in `arc/docs/`. A NUC candidate
changes nothing there. When the solver would need a change, finish the process without it and
[suggest the change](#suggest-solver-changes) at the end.

The dataset agent also registers a new dataset's puzzle in
`.github/workflows/arc.yml`, as its profile requires. Keep that reporting-list
update with the dataset; it does not authorize dispatching the GitHub workflow.

## Check the puzzle

- Pull under the [git synchronization policy](../../rules/git.instructions.md)
  and rebuild the primary runtime with `cargo build`, so the dataset and run
  use the current accepted runtime.
- Without a puzzle ID, [select the default puzzle](#select-the-default-puzzle).
- Require an eight-character lowercase hexadecimal ID with a file in
  `arc/data/training/`. Report an unknown ID instead of guessing.
- Record the revision and working-tree status in the session workspace, and
  preserve pre-existing changes.
- Use training pairs only. Never print or read the puzzle's `test` entries.

## Select the default puzzle

When the invocation names no puzzle ID, choose one this way and use it as the
puzzle ID in every later step:

1. Consider each puzzle in `arc/data/training/` except the genesis puzzle
   `3aa6fb7a` and every puzzle that already has
   `arc/src/dataset/dataset.<puzzle-id>.json`.
2. Measure each from its `train` pairs alone, never its `test` entries: the
   cell count (rows times columns) of its largest input or output matrix, and
   the total cells across all those matrices.
3. Choose the puzzle whose largest matrix has the fewest cells, breaking ties
   by the fewest total cells and then by the lowest puzzle ID.

If no puzzle remains, stop and report it without a run.

## Pass the hint

An optional `--hint <text>` comes last: everything after `--hint` is the
user's hint about the puzzle, such as its objects, their roles, or its
transformation. Pass it unchanged to every `/arc-dataset` run and to no other
worker; a requested NUC comes from the dataset handoff alone. The arc-dataset
agent keeps only what the training pairs confirm, so the hint never overrides
them, this skill, or the ARC instructions, and its text never reaches the
solver run.

## References

The workflow and its workers may use every dataset in `arc/src/dataset/` as a
reference for structure, world-model idioms, properties, rules, and tags.
The solver sends every JSON instruction dataset as shared few-shot context,
including the running puzzle's own annotated test pairs. That is intended
while the datasets are prepared as few-shot examples, under the
[ARC instructions](../../rules/arc.instructions.md#honest-evaluation-and-bounded-changes):
neither report it as a leak nor suggest a solver change that excludes them.
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

Always go with the declarative, higher-order, and most expressive solution,
and pass this requirement to every `/arc-dataset` run. Among the formulations
the training pairs confirm, the dataset takes the one that says the most with
the least: each relation stated once over every instance of a class,
higher-order operations over class populations, such as `filter`, `map`,
`reduce`, `some`, `every`, `find`, and reasoning selections, wherever they
express a rule, and no rule or property that only restates what the others
already determine. When accepted Nucleoid cannot express the rules that way,
the handoff requests the missing enhancement instead of an imperative
workaround. Check a `written` handoff's rules against this requirement before
the run, and send any that fall short back to the arc-dataset worker.

Design every solution with gestalt perception, and pass this requirement to
every `/arc-dataset` run too. Start by zooming out to the big picture: each
grid as a whole, the figures and groups it shows, the relations between them,
and how the whole changes from input to output. Write the declarative
statements at that level, about the objects seen first, such as whole shapes,
patterns, and regions. Do not start from pixels. A model goes down to finer
objects, single cells included, only when the big-picture formulation cannot
express what the training pairs show; a pixel-level solution is then allowed,
but the big picture is always the starting point. When you check a `written`
handoff, send back one that starts from pixels without showing why the big
picture falls short, and never steer a worker toward a finer model than the
training pairs need.

Write every color as a letter, and pass this requirement to every
`/arc-dataset` run too. The dataset follows the exact pattern of the puzzle's
pairs in `arc/data/training/`, cell for cell and shape for shape, but never
writes a color as its number. `0` stays `0`, and every other color becomes a
letter of `ABCDEFGHI`, ranked by how often it occurs across all of the
dataset's input and output matrices, training and test pairs together: the
most used color is `A`, the next `B`, and so on, as `Matrix.silhouette` in
`arc/src/lib/Matrix.ts` ranks a matrix, with a tie going to the color read
first. The letters are the dataset's own ranking, not a fixed map from
numbers to letters, so a color keeps one letter in every pair of its dataset.
The same letters replace the colors in the instance masks and the world model:
color properties hold letters, such as `color='B'`, and rules name colors by
letter, such as `if $Obj.color is C:`. The arc-dataset profile's
[letter rule](../../agents/arc-dataset.agent.md#write-colors-as-letters)
defines the ranking in full. Check a `written` handoff against it before the
run, and send back one that writes a color as its number or ranks its letters
another way.

Write no quote that accepted Nucleoid can do without, and pass this
requirement to every `/arc-dataset` run too. A quote is a single- or
double-quoted string literal that the runtime lexer reads in a code field,
`declarations` or an instance's `implementation`; the JSON file's own quotes
and `#` comments are not quotes. The dataset writes a literal as a bare word
wherever the run's runtime, given the options `arc/src/lib/nucleoid.ts`
passes, reads that word as the same value, as NUC 86 and NUC 93 read `C` in
`if $Obj.color is C:` and `$Obj.fill_color is C`. The arc-dataset profile's
older examples that quote such letters do not override this. Have the
handoff list every quote it keeps, with the actual result of its bare
spelling. Check that list before the run, and send back a handoff that keeps
a quote a bare word would replace. The quotes that remain go to the
[quote-free proposal](#propose-quote-free-spellings).

Continue by the handoff's status:

- `written`: run the puzzle.
- `nucleoid_enhancement_required`: prepare the requested enhancement.
- `blocked`, including an existing dataset: stop and report it, without a run.

## Learn the NUC proposal strategy

Every ARC-requested NUC uses the existing category-conditioned UCB1 learner
under the [NUC caller protocol](../nuc-proposal/SKILL.md#select-and-consume-a-proposal-strategy)
and [command/schema contract](../../../CONTRIBUTING.md#reward-driven-proposal-strategies).
Do not add another bandit, reward formula, strategy registry, or ARC learner.
The ordered arms remain `reuse_first` and `evidence_first`; learning chooses
the proposal approach, never the feature, category, puzzle, or solver settings.

**Policy input.** Reuse the caller's latest retained policy path and raw blob
pin, including after resuming from its saved session learning result. When
there is no retained head, require `--strategy-policy <absolute-json-path>`
pointing to an explicitly supplied empty-history genesis or latest checkpoint
with all its immutable ancestors and evidence. Use the exact genesis schema
in the command contract; never create a genesis automatically. The file must
be an existing absolute external `.json` path, outside the primary checkout
and candidate worktree. Pin supplied raw bytes with
`git hash-object --no-filters` before selection.

Place `--strategy-policy` before any trailing `--hint`. It is a caller-only
input, not part of the dataset hint or solver arguments. If a head is already
retained, an explicitly supplied policy must match its path and pin; never
fork an older checkpoint or silently replace/reset history. A missing policy
blocks the requested enhancement with `strategy_policy_required`, before
drafting or a solver run. Invalid, changed, stale, or incomplete state is a
process blocker, not a fallback to unconfigured learning.

Without a requested NUC, report learning as `not_applicable`: do not select an
arm, create a trial, or update a checkpoint. A dataset-only run needs no policy.

**Before drafting.** After fixing the NUC path, confirmed category, single
mode, runtime applicability, accepted revision and immutable baseline:

- Allocate a unique trial ID and fresh external session `.json` paths for the
  selection, rubric, assessment, evaluator report, checkpoint, and learning
  result. All artifacts are create-once; retain them and every policy ancestor.
- Freeze and pin a caller-owned usefulness rubric under the
  [existing scoring schema](../../../CONTRIBUTING.md#advisory-usefulnesscost-scoring).
  Bind its criterion ID, NUC path and accepted baseline. Derive the observable
  language-level benefit from the dataset handoff's `requested_change` and
  unsupported operation, not from a proposed implementation or ARC outcome.
  Use general-purpose Nucleoid evidence, never ARC grids, puzzle fixtures,
  held-out answers, or new ARC automated tests.
- Set `NUC_PROPOSAL`, `NUC_PROPOSAL_BASELINE`, `NUC_PROPOSAL_MODE=single`,
  confirmed runtime inputs when applicable, `NUC_STRATEGY_CATEGORY`,
  `NUC_STRATEGY_TRIAL_ID`, `NUC_STRATEGY_POLICY`, its original
  `NUC_STRATEGY_POLICY_REVISION`, and fresh `NUC_STRATEGY_SELECTION`.
  Preserve `NUC_PROPOSAL_RUBRIC` and `NUC_PROPOSAL_RUBRIC_REVISION` through both
  handoffs. Unset old selection pins, checkpoint outputs, assessments and
  report pins; never rely on environment state from another tool process.

Actually run the selector before dispatching the proposer:

```console
cargo test --test proposals select_proposal_strategy -- --ignored --exact --nocapture
```

Check success and pin `NUC_STRATEGY_SELECTION_REVISION` with
`git hash-object --no-filters` before drafting. Give the fresh proposer the
unchanged policy, receipt, rubric and pins, trial bindings, selected arm and
its actionable instructions. Require concrete evidence that it executed the
selected approach in its frozen handoff; a label alone is insufficient.
Never reselect, backfill attribution, or let the proposer score its own trial.

**Independent update, before the ARC run.** Pass the unchanged rubric and all
four attribution inputs (`NUC_STRATEGY_POLICY`, `NUC_STRATEGY_POLICY_REVISION`,
`NUC_STRATEGY_SELECTION`, `NUC_STRATEGY_SELECTION_REVISION`) to the fresh
evaluator with the frozen handoff, original trial/category/mode/runtime inputs,
and fresh report, assessment and checkpoint paths. Request an independent
assessment of the frozen criterion after its mandatory checks, with actual
evidence for all six check groups. Missing or unknown usefulness is explicitly
unscored, not neutral zero; never invent evidence to obtain a reward.

Require the evaluator to save and verify the actual `evaluate_proposal`
report, pin `NUC_PROPOSAL_REPORT_REVISION`, set fresh
`NUC_STRATEGY_CHECKPOINT`, and actually run the existing update command under
the [evaluator protocol](../nuc-evaluate/SKILL.md#update-the-attributed-strategy-policy):

```console
cargo test --test proposals learn_proposal_strategy -- --ignored --exact --nocapture
```

Use only the verifier's existing advisory reward: demonstrated usefulness
with its evaluator-time discount, independently assessed neutral `0`, or
genuine core/known external rejection `-1`. A legitimate unscored result
retains the parent head without an observation or checkpoint. A learner error
pauses the invocation without changing the saved independent decision.

Verify the returned checkpoint path/raw pin, parent, receipt/report bindings
and update or skip reason. Persist a new session learning-result JSON with
those artifacts and the retained head before any ARC run. Advance the caller's
one retained head only on a successful update; otherwise keep its original
path and pin. A later separately authorized NUC uses that head, not the genesis.

Never send the ARC run's output to the evaluator or learner. Its exact match,
difference, exit status, timeout and eventual NUC acceptance are not reward
inputs and cannot revise the already-saved score or checkpoint. The bandit
does not change acceptance criteria, train LLM weights, or authorize another
proposal or solver run. Do not claim improved proposal quality from one trial.

## Prepare the requested enhancement

Handle at most one NUC request per invocation, as a single-proposal request.
The only other NUC an invocation prepares is its
[quote-free NUC](#propose-quote-free-spellings), after the run.

1. Trigger `/nuc-proposal` for the request by following the caller steps of
   the [nuc-proposal skill](../nuc-proposal/SKILL.md), using the request's
   `requested_change` as the requested change. Choose the next NUC number,
   confirm the category and runtime applicability (ask the user when either is
   uncertain), pin the accepted revision and record its immutable baseline,
   and create a candidate worktree. Complete the
   [pre-draft strategy selection](#learn-the-nuc-proposal-strategy) before
   launching a fresh `nuc-proposal` agent with the request, its evidence, and
   the pinned strategy/rubric handoff. Complete the Rust improvement pass when
   required and freeze the handoff. If the candidate's changes from the pinned
   revision, untracked files included, touch any path under `arc/`, or
   preparation is otherwise blocked, stop and report it.
2. Launch a fresh `nuc-evaluate` agent on the frozen handoff under the
   [nuc-evaluate skill](../nuc-evaluate/SKILL.md), with
   `NUC_PROPOSAL_MODE=single`, the unchanged scoring/strategy inputs, and fresh
   report, assessment and checkpoint paths. Complete and persist the
   [independent strategy update](#learn-the-nuc-proposal-strategy) before
   proceeding; a learning error stops the invocation without a run. Keep its
   decision and report, including its hard-criteria evidence. Do not record
   the proposal outcome yet; it waits for the run.
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

The invocation authorizes exactly one live solver run, which makes paid
OpenRouter calls. With dependencies installed from the committed lockfile, run
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

Run once. Never rerun for a pass, including after a timeout, or change the
provider, model, sampling, prompts, solver code, dataset, or candidate around
the run. The output shows the expected test grid: keep it in the session
workspace, never give it or anything taken from it to an agent, and never use
it to revise the dataset or candidate.

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

Record it with the nuc-proposal skill's caller steps, which alone commit and
push it:

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
  move the invocation-owned `dataset_path` and its README paragraph from the
  primary checkout into the session workspace. Restore any pre-existing
  dataset and paragraph saved before replacement, without overwriting later
  unrelated changes. If this invocation added the puzzle's reporting-list entry,
  remove only that entry when withdrawing the dataset. If it set the solver's
  default puzzle, restore the recorded previous value unless a later change
  replaced it. Preserve pre-existing entries, other datasets, genesis files,
  and unrelated workflow changes. Keep the candidate isolated.

Those caller steps push the requested NUC's outcome commit right after it,
pulling first, under the
[git synchronization policy](../../rules/git.instructions.md). That commit
holds only the NUC's own records and integration; the dataset, README,
reporting-list, solver-default, and progress-record changes wait for this
invocation's [own commit](#commit-and-push) at the end. Never change project
versions.

## Propose quote-free spellings

Whenever a quote is used in `arc/src/dataset/`, trigger `/nuc-proposal` to
propose a version without it. After the outcome is decided and recorded, list
every [quote](#write-the-dataset) in the datasets that `arc/src/dataset/`
holds at `HEAD` and in this invocation's kept dataset. Check each one's bare
spelling with the accepted runtime and the options `arc/src/lib/nucleoid.ts`
passes:

- **Already bare:** the bare word reads the same value, so the dataset should
  drop the quote. This invocation's dataset was checked before the run.
  Report another dataset's quote as a respelling for a later invocation,
  without making it.
- **Behind an option:** an accepted NUC reads the bare word only under an
  option the solver does not pass, such as NUC 94's `--conditional-words`.
  [Suggest](#suggest-solver-changes) passing that option instead of a NUC.
- **Already proposed:** a Draft or Rejected NUC already proposes the bare
  spelling, whether it is recorded in `docs/nuc-0000.md` or is still in a
  candidate worktree that `git worktree list` shows. Report that NUC instead
  of starting a duplicate.
- **Not expressible:** no accepted, Draft or Rejected NUC writes it without
  quotes.

When any quote is not expressible, prepare one quote-free NUC for all of them.
Use steps 1 and 2 of
[Prepare the requested enhancement](#prepare-the-requested-enhancement), with
its own [strategy trial](#learn-the-nuc-proposal-strategy) from the retained
head, except that its outcome waits for no run.

- **Requested change:** write each listed quote without quotes, reading the
  same value, and keep every accepted form and meaning. That usually means a
  new opt-in option that composes with the accepted ones, as NUC 94 added.
- **Evidence:** each quote's dataset, field and code line, with a minimal
  general-purpose program and its actual result under the accepted runtime
  and the solver's options. Never give it an ARC grid.
- **Blockers:** a missing or invalid policy, or any other blocker, stops only
  this NUC. Report it and finish the invocation.

The quote-free NUC is never run against the puzzle. The invocation's one run
is over, and the datasets keep their quotes until the solver passes the
option the NUC adds. Its independent strategy update therefore precedes its
outcome, which follows the
[automatic acceptance rule](../../rules/proposals.instructions.md#automatic-acceptance-caller-only)
directly:

- A failed hard criterion rejects it.
- Otherwise the evaluator's acceptance makes it Accepted, and any other
  decision makes it Automatic-Accepted.
- Without an evaluation result it is Blocked and stays Draft.

Record the outcome with the nuc-proposal skill's caller steps, which commit
and push it. When it is accepted with an option, suggest passing that option
in `arc/src/lib/nucleoid.ts`, and list the quotes to respell once the solver
does.

## Suggest solver changes

When a dataset or NUC handoff, the [quote check](#propose-quote-free-spellings),
or an error in the run's log shows that the solver's TypeScript would need a
change, describe that change for the user
instead of making it: the files, the change, and the evidence for it. Base no
suggestion on the expected test grid or the difference from it, and never
apply a suggestion or rerun the puzzle within the invocation.

## Document the progress

After the outcome is decided and recorded, including any NUC commit and push,
document the invocation in the puzzle's progress record,
`arc/docs/ARC-NUC-<puzzle-id>.md`, even when it ended without a run. Create
the record when the puzzle has none; otherwise update it in place, keeping its
earlier runs, decisions, and history. Write it yourself, because it carries
the run's result, and never point a worker to it. Follow the existing records,
such as `arc/docs/ARC-NUC-0ca9ddb6.md`:

- The title `# ARC-NUC-<puzzle-id> -- <Name>`, where the name is a few
  title-case words for the transformation, then the records' opening sentence.
- A status table: the puzzle; the instruction dataset, linked with its pair
  and instance counts, or why it is absent or was withdrawn; the output
  canvas; each requested NUC, linked with its outcome and basis, or `None`;
  the solver changes, `None` or suggested and not applied; the solver runs;
  and the date updated, written like `2-Nov-2026`.
- `Puzzle`: the transformation the training pairs show.
- `Gestalt overview`: the puzzle zoomed out to the big picture, as the
  [gestalt requirement](#write-the-dataset) reads it, in five bold-labeled
  points: **Whole grid**, what each input and output grid shows at a glance;
  **Figures**, the figures and groups seen first; **Relations**, how they
  relate to each other or to the canvas; **Change**, how the whole changes
  from input to output; and **World model**, the level of the dataset's
  objects and, when they are finer than the figures seen first, why the big
  picture fell short. Draw it from the training pairs and the handoff's
  big-picture formulation. Every invocation writes this section, or brings it
  up to date with the current dataset, and leaves earlier models to the
  dataset's history. When no dataset was kept, say so under **World model**
  and give the level the handoff proposed.
- `Dataset`: the objects, their properties, and the rules in a `nuc` block,
  linking the dataset's README paragraph for the rest. When the invocation
  replaced a dataset, say what changed and why, such as the user's hint.
- A `Decision` section for a requested NUC: the gap, the evidence programs
  and their actual errors, and what the NUC adds. Include the selected
  proposal strategy and whether learning updated, was unscored, or blocked,
  without copying session artifacts or claiming an ARC-derived reward.
- `Runs`: one entry per run, never removed, with its date, command, runtime
  (the accepted revision or the NUC candidate), exit status or timeout,
  difference or error, and elapsed time. Without a run, say that the
  difference is unknown, not zero.
- `Open items`: blockers, open questions, the suggested solver changes, and
  the quotes in the puzzle's dataset that wait for them or for a respelling.

Describe the puzzle and dataset from the training pairs and the arc-dataset
handoff; never read the puzzle's `test` entries for the record. From the run's
output, copy only its exit status, difference or error, and elapsed time,
never a grid or cell. Then [commit and push](#commit-and-push) the record
with the dataset.

## Commit and push

When the invocation is done, after the progress record, commit its changes
and push them under the
[git synchronization policy](../../rules/git.instructions.md). This is its
one commit of its own, separate from a requested NUC's outcome commit, which
the nuc-proposal caller steps already made and pushed. An invocation that
pauses, or stops before it writes the progress record, commits nothing.

1. Pull with `git pull --no-rebase --autostash` right before committing, and
   resolve any conflict under the policy, keeping every other session's work.
   A conflict on the solver's default puzzle keeps the value the working tree
   held before the pull: this invocation's puzzle ID, or a later session's.
2. Stage, by explicit path, only what this invocation changed and kept in the
   primary checkout: `arc/src/dataset/dataset.<puzzle-id>.json`, its
   paragraph in `arc/README.md`, its entry in `.github/workflows/arc.yml`,
   the default puzzle in `arc/src/app.ts` while it names this puzzle, and
   `arc/docs/ARC-NUC-<puzzle-id>.md`. A withdrawn dataset, and the paragraph
   and entry withdrawn with it, stay out. When a path also holds changes this
   invocation did not make, such as the pre-existing changes recorded at the
   start or another session's, stage only this invocation's hunks, for example
   by applying its own diff with `git apply --cached`; if they cannot be
   separated, leave the path out and report it. Never use `git add -A`,
   `git add .`, or `git commit -a`, and never stage session artifacts, run
   logs, `arc/logs/`, or anything from a candidate worktree.
3. Commit with an imperative subject that names the puzzle and what changed,
   as the existing ARC commits do, such as
   `Add ARC dataset for puzzle <puzzle-id>` for a new dataset or
   `Rework ARC 27a28665 dataset to whole-block shapes` for a replaced one. In
   the body, state what the dataset models, the files changed, any requested
   NUC with its outcome, and the run's exit status and difference or error,
   or that there was no run. Copy nothing else from the run's output, never a
   grid or cell.
4. Verify that the commit holds only the intended paths and hunks, pull again
   the same way, and push the branch right away. When the push is rejected
   because the remote moved, pull, resolve, and push again until it succeeds.
   Never force-push, rewrite published history, or skip hooks.

When nothing is left to commit, make no empty commit, and report why.

## Report

Save one JSON result record in the session workspace with the puzzle ID and
how it was chosen (given, or the default with both its measurements), the
starting revision, and hint (`null` without one); the dataset status,
`dataset_path` and source puzzle path,
the reporting-workflow path and whether its new entry was retained or
withdrawn, and arc-dataset's use of the hint; the NUC path, blob, category,
evaluator decision, and report path; the solver's previous and new default
puzzle IDs and whether that change was retained or restored; the run's
command, executable, exit status or timeout, difference or error, and log
paths; the outcome and its basis; the suggested solver changes; the progress
record's path and whether it was created or updated; the commit's hash,
paths, and push result, or why nothing was committed; and any blockers. Record
the quote check as each quote's dataset, field, code line and result, and the
quote-free NUC with the same fields as a requested NUC, or why there is none.
Use `strategy_learning`, once per requested NUC or once without one, to record
`not_applicable`, `blocked`, `updated`,
`unscored`, or `error`, with its reason; the policy input path/raw pin, trial
ID, selected arm and consumption evidence; rubric/assessment, selection and
evaluator report paths/pins; the verified advisory reward or unscored reason;
the checkpoint path/raw pin when created; and the learning-result path and
retained head path/raw pin for the next authorized invocation. Never substitute
the ARC score or proposal outcome for the advisory reward. Use
`null` for anything not measured, including the difference before this
dataset. Report a short summary with both records' paths and the pushed
commit, ending with any suggested solver changes, then stop without starting
another puzzle or NUC.

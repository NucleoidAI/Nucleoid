# ARC-NUC-ed36ccf7 -- Pattern Quarter Turn

Progress record for ARC training puzzle `ed36ccf7`: its instruction dataset,
the solver and Nucleoid changes it needed, its runs, and what is still open.

| Item | Status |
| --- | --- |
| Puzzle | [`data/training/ed36ccf7.json`](../data/training/ed36ccf7.json) |
| Instruction dataset | [`dataset.ed36ccf7.json`](../src/dataset/dataset.ed36ccf7.json): the puzzle's 4 original training pairs and its test pair, 5 pattern instances, one per pair; version 2, with the standard entries and plain fixed anchors, replaces version 1 of `f4cf680` and is committed with this record |
| Output canvas | `EXACT` |
| Nucleoid enhancements | [NUC 85](../../docs/nuc-0085.md), Fixed Cross-Property Class Anchors: Automatic-Accepted, because it meets every hard criterion while the run was no exact match and the independent evaluation rejected it; proposal strategy `reuse_first`, learning updated |
| Solver changes | None; suggested changes await review, not applied |
| Solver runs | 2 local runs on 5-Nov-2026: version 1 exit 1 before any model call (`OPENROUTER_API_KEY is required`); version 2 on the NUC 85 candidate exit 7, difference 7 |
| Updated | 5-Nov-2026 |

## Puzzle

Each training input is a 3 by 3 canvas holding one pattern of a single color
on a background of zero cells: 7 cells of color 9, 5 of color 6, 5 of color 9,
and 5 of color 2. Not every pattern is connected. The second is two bars, the
full top row and the left two cells of the bottom row, and the fourth is a
lone top-left cell beside a hook of four cells along the right column and the
bottom row. The output keeps the canvas and turns the whole grid a quarter
turn counterclockwise, so the cell in row r and column c moves to row 2 - c
and column r: the right column becomes the top row, and every color keeps its
cell count. Of the square's eight turns and mirrors, only this turn maps every
training input to its output. The third pattern, an L symmetric across the
main diagonal, also fits a top-bottom mirror, but each of the other three
pairs fits only the quarter turn. Every training pattern's box spans its
3 by 3 canvas before and after the turn. The output keeps the input's canvas,
so the dataset is `EXACT`; every training output's pattern spans its canvas,
so `FIT` would frame the same training grids.

`/arc-purposal` chose this puzzle by default: among the 382 training puzzles
other than the genesis puzzle `3aa6fb7a` and those with a dataset, its largest
training matrix has the fewest cells, 9, with 72 cells in all its training
matrices. `6e02f1e3` and `794b24be` also have a largest matrix of 9 cells, but
90 and 180 cells in all, so no other puzzle tied on both measures. `ed36ccf7`
had tied on both with [`3c9b0459`](ARC-NUC-3c9b0459.md),
[`74dd1130`](ARC-NUC-74dd1130.md), [`9565186b`](ARC-NUC-9565186b.md), and
[`a85d4709`](ARC-NUC-a85d4709.md), which earlier invocations took.

## Gestalt overview

- **Whole grid.** A 3 by 3 background canvas holds one pattern of a single
  color.
- **Figures.** One figure per grid, the whole pattern, whether or not its
  cells touch.
- **Relations.** The pattern relates to the canvas center, which it turns
  about.
- **Change.** The whole grid turns a quarter turn counterclockwise, so the
  right column becomes the top row.
- **World model.** One object per pair, the whole pattern, with the standard
  entries `x`, `y`, `width`, `height`, and `orientation`. The turn sets the
  orientation to 270 and moves `x` and `y` about the canvas center, while
  `width` and `height`, the pattern's own size, stay as they are.

## Dataset

Version 2 holds the four original training pairs in source order, followed at
the same level by the puzzle's test pair, which the same rules annotate. Each
grid's whole pattern is one instance of the standard
`Obj(x, y, width, height, orientation)`, tagged `l9100`, and every
implementation passes the pattern box's `width` and `height` after `y`, 3 by 3
in every training pair. The class takes no color, parts list, canvas size, or
matrix of cells: the pattern's cells travel unchanged in its masks. Three
unconditional class-level rules turn every pattern about the canvas center,
each reading the input state with a plain fixed `@first` anchor:

```nuc
$Obj.orientation = ($Obj.orientation@first + 270) % 360
$Obj.x = $Obj.y@first
$Obj.y = canvas_width - $Obj.x@first - $Obj.width@first
```

A counterclockwise quarter turn is 270 degrees clockwise, as
`dataset.6150a2bd.json` adds 180 for its half turn. The box's top row becomes
the turned box's left column, and its right column becomes the turned box's
top row, as far below the top edge as the box was left of the right edge. No
rule changes `width` or `height`: the turn is the orientation's job, so each
output state keeps the input's size, the user's standard for every dataset.
The `x` and `y` rules each read the other's fixed anchor, a cycle that the
accepted runtime before [NUC 85](../../docs/nuc-0085.md) rejected; NUC 85 lets
such fixed-only cycles stand, so version 2 needs a runtime that includes it.
The declarations carry no `#` comments, and the rules create no property. The
[README](../README.md#instruction-datasets) describes the dataset in full.

The caller checked the handoff against the declarative, higher-order, and
gestalt requirements: each relation is stated once over every pattern, no
property restates another, and the plain anchors replace version 1's `.value`
snapshots. It replayed the four training pairs on the NUC 85 candidate, and
all 123 checks passed: grids, masks, boxes, implementations with their sizes,
merges, and input and output snapshots with the rules declared before and
after the instance. The accepted runtime at `491512d` rejects the same rules
with `TypeError: Circular Dependency (line 17, column 1)`.

### Version 2 and the hints

On 5-Nov-2026 the user reran `/arc-purposal ed36ccf7` with the question
"isn't width also changed when object rotated?". Version 1 left `width`
unchanged and had no height, which matched the training pairs only because
every training box is 3 by 3. The first rework answered the question by
swapping width and height with the turn, `$Obj.width = $Obj.height@first.value`
and `$Obj.height = $Obj.width@first.value`, read through `.value` as version
1's `x` and `y` were. The user asked whether that was the right strategy. The
caller found the swap right for the box but the `.value` reads only a
workaround: plain fixed anchors of other properties fail with
`TypeError: Circular Dependency`, because NUC 55 exempts only a rule's own
anchor, and `.value` drops the read's tracking. The user chose to request the
enhancement, so the worker withdrew that draft, restored version 1, and
returned the request.

While the enhancement was prepared, the user set a standard with a second
hint: "Add widht and heigth as a standard entry like class Obj(x: int, y: int,
width: int, height: int, orientation: int ...) and do not change alone with
orientation changes unless object shape changed as well. If object is getting
larger or change always refer to @first like $Obj.width = $Obj.widht@first +
1". Version 2 follows it: the size is a standard entry, the turn leaves it
unchanged, and every rule that computes a changed value reads `@first`. The
training pairs confirm the hint and contradict nothing. The worker fixed the
rules from the training pairs and the hints before it read the test pair, and
the same rules reproduce the test pair. That pair had been visible in version
1's file and README paragraph, which this rework replaced; the change from
version 1 comes from the hints and NUC 85, not from the test pair.

The same day the user asked to apply the standard to every dataset. The other
22 datasets moved to the standard entries in this invocation, as the
"Standard entries" sections of their records describe, and the arc-dataset
profile now states the standard.

### Versions

| Version | Source | Class | Rules |
| --- | --- | --- | --- |
| 1 | `/arc-purposal` default pick, committed in `f4cf680` | `Obj(x, y, orientation, width)` | the turn, with `x` and `y` reading each other's `@first` anchor through `.value` |
| 2 | `/arc-purposal ed36ccf7` with the two hints, on NUC 85 | `Obj(x, y, width, height, orientation)` | the same turn with plain `@first` anchors and no size rule |

Version 1's rules read both anchors with `.value`, because the accepted
runtime rejected the plain anchors as a cycle:

```nuc
$Obj.orientation = ($Obj.orientation@first + 270) % 360
$Obj.x = $Obj.y@first.value
$Obj.y = canvas_width - $Obj.x@first.value - $Obj.width
```

Both versions start from the big picture, each grid's whole pattern as one
object whether or not its cells touch, and never go down to pieces or single
cells. One choice the training pairs cannot settle remains: every training
box spans its canvas, so turning the pattern in place, with `orientation`
alone, as `dataset.3c9b0459.json` states its half turn, also reproduces every
training output. Both versions turn it about the canvas center instead, as
`dataset.6150a2bd.json` does, because these patterns lie on a background of
zero cells, so their boxes need not span the canvas, and a turn of the whole
grid moves a box that does not. Version 1's worker fixed that choice from the
training pairs before it read the test pair. Each training grid holds one
pattern and relates it to nothing else, so the rules need no population
operation such as `filter` or `some`.

The worker rewrote the dataset and its README paragraph. The puzzle's entry in
the **ARC puzzle report** workflow's list, its 19th of 20 puzzles, was already
there. The solver's default puzzle in `src/app.ts` already named `ed36ccf7`,
set by `dd0cd5d`, so this invocation left it unchanged. After the run,
`521aa3c` set the default to the genesis puzzle `3aa6fb7a`, a later change
this record keeps.

## Decision: fixed cross-property anchors (NUC 85)

- **Gap.** A quarter turn needs `x` to take the input `y` and `y` to take a
  value computed from the input `x`, so each rule reads the other's fixed
  `@first` anchor. The accepted runtime treated a rule's read of another
  property's anchor as a dependency on that property, so the two rules form a
  class cycle: `$Obj.x = $Obj.y@first` with
  `$Obj.y = canvas_width - $Obj.x@first - $Obj.width@first` raised
  `TypeError: Circular Dependency (line 17, column 1)`. NUC 55 exempts only a
  rule's own fixed anchor, and `.value` breaks the cycle by freezing the read.
- **Evidence.** The worker reduced the gap to general programs on the accepted
  runtime: a two-property `Box` swap, a `Point` coordinate swap, a rectangle's
  quarter turn in a square, and a three-property rotation mixing `@0` and
  `@first` each raised `TypeError: Circular Dependency`, at lines 5, 5, 8 and
  7. Their `.value` spellings gave the needed values, and a tracked reader
  beside a `.value` reader showed that the snapshot stays stale after the read
  slot is recreated. Cycles with a live or moving read had to keep their error.
- **What NUC 85 adds.** A class cycle made only of fixed history reads of the
  same instance's properties, `@first`, `@0`, or a literal `@n`, is no longer a
  circular dependency, and those reads stay tracked. A cycle with a live,
  moving or other edge still raises the error.
- **Numbering.** The proposal was drafted as NUC 83. Before its handoff,
  another session recorded NUC 83 and NUC 84, so the caller renumbered it to 85
  and re-pinned it to the accepted revision `491512d`, with a fresh baseline,
  rubric and strategy selection. The superseded trial was never evaluated.
- **Preparation.** The Rust improvement pass found that an early prototype's
  queue let a rule skip any path back to itself, so a reasoning-selection
  cycle stopped erroring and another grew memory without bound. The proposer
  fixed it before the frozen handoff and pinned both programs in regression
  tests.
- **Strategy and learning.** A user-authorized empty-history genesis policy
  selected `reuse_first`, the cold-start first arm, and the proposer
  generalized NUC 55's exemption, tracking frame and graph-node storage instead
  of adding a mechanism. Learning updated: the independent evaluator's
  advisory reward was -1, because one of its behavior checks failed although
  usefulness was demonstrated, and the learner wrote one checkpoint from it.
  Nothing from this puzzle's run fed the reward.
- **Outcome.** The independent evaluator found every hard criterion passed:
  the build, warning-denied Clippy and formatting; the full `cargo test`, with
  2,574 tests passed and every accepted test present; and backward
  compatibility, with 9,254 accepted programs unchanged apart from a
  `Date.now()` case and every changed result a former `Circular Dependency`.
  Its policy decision was Rejected: fixed-only cycles whose rules build fresh
  objects never settle, although NUC 85 says such cycles settle, and cascades
  across thousands of instances slow down sharply. The run below was no exact
  match, so NUC 85 is Automatic-Accepted, integrated unchanged by `3592d4b`;
  its Evaluation section records the findings.

## Runs

| Date | Dataset version | Runtime | Exit status | Difference | Elapsed |
| --- | --- | --- | --- | --- | --- |
| 5-Nov-2026 07:50 UTC | 1, one pattern per object | Accepted `161e65d`, local | 1 | Unknown: `Error: OPENROUTER_API_KEY is required` | 0.7s |
| 5-Nov-2026 23:11 UTC | 2, standard entries | NUC 85 candidate on `491512d`, local | 7 | 7 | 2m 29.4s |

The first run was `bun start ed36ccf7` from `arc/`, run once by
`/arc-purposal` with a 30-minute limit and `NUCLEOID_EXECUTABLE` unset, after
`bun install --frozen-lockfile` reported no changes. It used the primary
checkout's debug runtime (`nucleoid 1.0.0`), built with `cargo build` at
`161e65d`, and the committed solver at `161e65d` with only its default puzzle
changed. It stopped in the analyzer's first model call,
`analyzer.declarations`, where `src/lib/openrouter.ts` raised
`OPENROUTER_API_KEY is required`, before any model request, prediction, or
score. Its difference is therefore unknown, not zero, and the solver wrote no
`logs/ed36ccf7.log.json`. The solver reported no elapsed time, so elapsed is
the process's run from 07:50:24.3 to 07:50:25.0 UTC. The hashes of the
runtime, the dataset, and `src/app.ts` were the same before and after the run.
The invocation authorizes one run, so none was repeated.

The second run was `bun start ed36ccf7` from `arc/`, run once by
`/arc-purposal` with a 30-minute limit and `NUCLEOID_EXECUTABLE` set to a copy
of the frozen NUC 85 candidate's debug build (`nucleoid 1.0.0`), after
`bun install --frozen-lockfile` reported no changes. The solver was the
committed `491512d`, its default puzzle already `ed36ccf7`, and its few-shot
context held every instruction dataset in the working tree: version 2 of this
dataset and the 22 others moved to the standard entries, all uncommitted at
the time. The run ended in a mismatch with exit status 7 and a difference of
7, in 2m 29.4s by the solver's own report; the process ran from 23:11:32.8 to
23:14:02.8 UTC. Its log records no error. The hashes of the candidate runtime,
the dataset, and `src/app.ts` were the same before and after the run, and no
commit landed during it. The invocation authorizes one run, so none was
repeated.

## Open items

- **A run sees the answer.** `src/lib/llm.ts` sends
  `dataset.document(puzzle_id)` as few-shot context, and
  `src/dataset/dataset.ts` discards `puzzle_id`. The second run's prompts
  therefore held version 2's annotated test pair, expected output included,
  and an exact match would not have shown that the turn was inferred from the
  training pairs. Its mismatch is not explained here, because the run's log
  shows the expected grid and this record does not read it; `/arc-debug
  ed36ccf7` can investigate the log separately. Suggested, not applied: keep
  the running puzzle's test pairs out of `document(puzzle_id)`.
- **Messages are not JSON-only.** `src/lib/llm.ts`, `src/analyzer.ts`, and
  `src/visualizer.ts` still wrap JSON in text labels such as `few_shot:`,
  `control_id:`, `train_dataset:`, and `json_format:`, while the ARC
  instructions require every message to parse as one JSON document.
  Suggested, not applied: build each message with `JSON.stringify` of one
  data object.
- **NUC 85's settlement claim.** NUC 85 is accepted with a claim its
  evaluation disproved: a fixed-only cycle whose rules build fresh objects,
  such as `$Pair.a = {v: $Pair.b@first}` with `$Pair.b = {v: $Pair.a@first}`,
  reaches the 100,000-step limit and raises
  `TypeError("Propagation did not settle")` instead of settling, and one input
  that wakes the cycles of thousands of instances runs far slower than the
  accepted equivalent. A follow-up proposal should make such cycles settle or
  document the limit, and bound the cascade cost. This dataset's rules read
  numbers, so they settle.
- **The standard-entries migration.** At the user's request, the other 22
  datasets, their README paragraphs, the README's Analysis examples, 18
  progress records and the arc-dataset profile moved to the standard entries
  in this invocation, and `521aa3c` committed them with this dataset. Every
  migrated snapshot replays offline on the accepted runtime and on NUC 85, but
  no solver run has measured the migrated datasets yet. The genesis
  dataset `dataset.core.json` also moves its grown corners' origins, so its
  output states now place each 3 by 3 box where its output mask starts.
- **Comments in declarations.** This dataset and seven others carry none,
  following `.claude/rules/arc.instructions.md`, while the other 15 datasets
  carry `#` comments, which the migration kept and updated. Decide which
  wins; neither choice changes any snapshot.
- **Workflow timeout wording.** The arc-dataset profile still asks for a job
  timeout of at least 30 minutes per listed puzzle plus 20 minutes, 620
  minutes for the 20 puzzles now listed, beyond GitHub Actions' 360-minute job
  limit. Since `56b093c` the workflow runs the puzzles in batches of
  `ARC_CONCURRENCY`, 5, under a 150-minute job timeout: four batches of 30
  minutes plus 20 make 140 minutes. Suggested, not applied: state the batch
  budget in the profile.
- **Resolved.** At `491512d` `bun run typecheck` and `bun run lint` pass, so
  the lint failures of `161e65d` are gone. NUC 85 answers the earlier question
  whether the fixed anchors of other properties should stop counting as
  dependencies.

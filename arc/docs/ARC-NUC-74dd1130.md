# ARC-NUC-74dd1130 -- Main-Diagonal Mirror

Progress record for ARC training puzzle `74dd1130`: its instruction dataset,
the solver and Nucleoid changes it needed, its runs, and what is still open.

| Item | Status |
| --- | --- |
| Puzzle | [`data/training/74dd1130.json`](../data/training/74dd1130.json) |
| Instruction dataset | [`dataset.74dd1130.json`](../src/dataset/dataset.74dd1130.json): the puzzle's 4 original training pairs and its test pair, 5 mosaic instances, one per pair; committed with this record; shared orientation comment added on 6-Nov-2026; descriptive `implementation` comments added on 7-Nov-2026; colors written as letters on 8-Nov-2026; mirror axis derived from each mosaic's input box with `@first` on 8-Nov-2026 |
| Output canvas | `EXACT` |
| Nucleoid enhancements | None requested, so no proposal strategy was selected and strategy learning did not apply; the rule is one unconditional class-level rule of accepted Nucleoid, using the class-level `@first` anchor of [NUC 55](../../docs/nuc-0055.md) |
| Solver changes | None applied; on 8-Nov-2026, validating the model's declarations and rejecting a `null` input state in `src/analyzer.ts` were suggested and not applied; earlier suggestions await review |
| Solver runs | 2 local runs: 5-Nov-2026, exit 0, an exact match, with this dataset's test pair in its few-shot context; 8-Nov-2026, of the dynamic-axis version, exit 1, a failed run with no prediction |
| Updated | 8-Nov-2026 |

## Puzzle

Each training input is a 3 by 3 canvas filled by one mosaic: every cell holds
a color, and none holds the background color 0. The four mosaics use three
colors each: 1, 2, and 5; 2, 5, and 6; 5, 8, and 9; and 1, 2, and 6. The
output keeps the canvas and mirrors the whole mosaic across its main
diagonal, the line from the top-left corner to the bottom-right corner, so the
cell in row r and column c moves to row c and column r. Each input row becomes
the output column of the same index, the three cells on the diagonal stay in
place, and every color keeps its cell count. Of the square's eight turns and
mirrors, only this flip maps each training input to its output, and no
training mosaic looks the same under any of the other seven, so each pair
confirms the flip on its own. No turn reproduces any training output, because
a mirror reverses the mosaic's handedness and a turn never does. The output
keeps the input's 3 by 3 canvas, so the dataset is `EXACT`; with no zero cell
to crop, `FIT` would frame the same grids.

`/arc-purposal` chose this puzzle by default: among the 385 training puzzles
other than the genesis puzzle `3aa6fb7a` and those with a dataset, its largest
training matrix has the fewest cells, 9, with 72 cells in all its training
matrices. Three other puzzles, `9565186b`, `a85d4709`, and `ed36ccf7`, tie on
both measures, and `74dd1130` has the lowest ID among them. It was one of the
puzzles tied with [`3c9b0459`](ARC-NUC-3c9b0459.md), the previous default.

## Gestalt overview

- **Whole grid.** A 3 by 3 canvas filled by one mosaic of three colors, with
  no background cell.
- **Figures.** One figure per grid, the whole mosaic.
- **Relations.** The mosaic relates to the canvas's main diagonal, from the
  top-left corner to the bottom-right. No training mosaic looks the same
  under another turn or mirror of the square, so each pair shows the flip on
  its own.
- **Change.** The whole mosaic flips across that diagonal, so its rows
  become its columns.
- **World model.** One object per pair, the whole mosaic, whose flip is its
  `mirror_axis`: the line from its input box's top-left cell to its
  bottom-right cell, read with `@first`. No finer model was needed.

## Dataset

The dataset holds the four original training pairs in source order, followed
at the same level by the puzzle's test pair, which the same rule annotates.
Every mosaic is one instance of `Obj(x, y, width, height, orientation)` with
only the standard entries, tagged `l9100`, a single shape, as in
`dataset.28bf18c6.json` and `dataset.3c9b0459.json`. The class takes no
color, cell list, canvas size, or matrix of cells: the mosaic's cells travel
unchanged in its masks. One unconditional class-level rule states the flip
and derives its axis from the mosaic's own input box:

```nuc
class Obj(x: int, y: int, width: int, height: int, orientation: int):
$Obj.mirror_axis = {start: {x: $Obj.x@first, y: $Obj.y@first}, end: {x: $Obj.x@first + $Obj.width@first - 1, y: $Obj.y@first + $Obj.height@first - 1}}
```

`mirror_axis` is the line the mosaic is mirrored across, and it is absent
from every input state. `start` is the top-left cell of the mosaic's input
box and `end` its bottom-right cell, as canvas columns and rows, both read
with `@first`. So the axis is the mosaic's own main diagonal wherever its box
lies: from 0 and 0 to 2 and 2 in every pair, and the cells on it stay in
place.

`orientation` cannot carry the change, because it is a clockwise turn and no
turn reproduces a training output, so it stays 0 in both states. The rule
creates no other property and leaves `x`, `y`, `width`, `height`, and the
tag unchanged: a flip across its own diagonal keeps a square box in place.
Each output state adds only `mirror_axis` to its input state, as
`{"end":{"x":2,"y":2},"start":{"x":0,"y":0}}` in the runtime's key order.

The declarations and implementations carry no `#` comments, and each instance
has the six keys the datasets have shared since `d0e3eed`. The
[README](../README.md#instruction-datasets) describes the dataset in full,
and [Dynamic mirror axis](#dynamic-mirror-axis) below says what changed from
the constant `135`, and why.

For the first version, the worker started from the big picture, the whole
mosaic as the object, and that formulation explained every training pair, so
it never went down to rows,
color regions, or single cells. It weighed two alternatives. A turn plus a
mirror flag would state one change with two properties and a convention for
their order. Rules that swap the box's origin, `$Obj.x = $Obj.y@first` and
`$Obj.y = $Obj.x@first`, reproduce every training output too, but every
mosaic's origin is 0 and 0, so they would only return the input origin, the
reason `dataset.3c9b0459.json` also leaves its position rules out. The worker
fixed the rule from the training pairs before it read the test pair and
reported that the same rule reproduces it. Each training grid holds one mosaic
and relates it to nothing else, so the rule needs no population operation such
as `filter` or `some`. This is the first dataset that states a mirror as a
transformation: `dataset.44f52bb0.json` only tests whether a pattern is mirror
symmetric, and `dataset.28bf18c6.json` repeats its shape rather than mirroring
it.

The caller checked the handoff against the declarative, higher-order, and
gestalt requirements, found that it met them, and sent nothing back. It
confirmed the line-orientation convention in `dataset.line.json`. It also
replayed the four training pairs on the accepted runtime: 98 checks of grids,
keys, the flip, mask shapes and cells, bounding boxes, merges,
implementations, orientations, and input and output snapshots, all passing.
The worker wrote the dataset, its README paragraph, and the puzzle's entry in
the **ARC puzzle report** workflow's list, its 16th puzzle. The caller made the
puzzle the solver's default in `src/app.ts`, in place of `3c9b0459`, before the
run.

### Standard entries

On 5-Nov-2026, at the user's request, every ARC dataset moved to the standard
object entries `x`, `y`, `width`, `height`, and `orientation`. `width` and
`height` are the column and row counts of the object's input bounding box. An
orientation change alone leaves them unchanged, and a rule changes them only
when the shape itself changes size, reading the input with `@first`. Every
implementation now passes the input box's `width` and `height` after `y`, and
the class and rules read:

```nuc
class Obj(x: int, y: int, width: int, height: int, orientation: int):
$Obj.mirror_axis = 135
```

No rule changed: the mirror stays in `mirror_axis`, and every box stays 3 by
3. The migration worker regenerated every snapshot with the accepted runtime
at `491512d`, and the caller replayed every pair offline against the unchanged
grids, masks, positions and instance names. No solver run has measured this
version yet.

### Orientation convention

On 6-Nov-2026 the user asked `/arc-purposal` to make `orientation` consistent
across all 29 instruction datasets, with a reference of eight directed 3-cell
lines, each of which points toward its head. Every state now takes the
direction its own shape points, by the convention in the
[README](../README.md#3-extract-instances): a line toward its head or along
its axis, a corner or chevron toward its vertex, and anything else 0, while a
turn adds its angle. Every dataset's first declaration also carries one shared
comment line, directly above its class, that states the convention for the
model:

```nuc
# Orientation is the direction an object points, in degrees clockwise from north (0 up, 90 right, 180 down, 270 left)
```

This dataset's labels already followed the convention, as the labeller
confirmed: each mosaic has no direction, so both states are 0, and
`mirror_axis` keeps 135, the axis value that the convention gives the falling
diagonal. Its README paragraph now cites that shared convention instead of the
line orientations of `dataset.line.json`, whose lines now point toward their
endpoints. Only the comment line changed, so the class, the rules, the grids,
masks, positions, instance names, implementations, and snapshots are as
before, and the gestalt overview above still holds.

The arc-dataset worker made the change in an external staging directory with
the accepted runtime at `d3413e6` (last runtime change `7096171`), and every
snapshot reruns byte for byte on it. The caller's own offline check passed on
the staged file and again after integration: the pairs, grids, masks,
positions, and instance names are as before, every state's orientation matches
the convention's labeller, every snapshot replays on that runtime, and
`document()` loads the dataset with the other 28. No NUC was requested, so no
proposal strategy was selected and strategy learning did not apply. At the
user's choice, no solver run has measured this version.

### Letter colors

On 8-Nov-2026, at the user's request, every instruction dataset started
writing its colors as letters instead of numbers. `0` stays `0`, and every
other color is a letter of `ABCDEFGHI`, ranked by how often it occurs across
all of the dataset's input and output matrices, training and test pairs
together: `A` for the most used, then `B`, and so on, a tie going to the color
read first, as `Matrix.silhouette` ranks one matrix. This dataset writes its
colors as letters ranked by count: 2 as `A`, 5 as `B`, 9 as `C`, 1 as `D`, 6
as `E`, 4 as `F`, 8 as `G`, and 3 as `H`. Mapping the letters back reproduces
the puzzle's pairs cell for cell.

The class and rules hold no color, so only the pair grids and both instance
masks changed; the declarations and implementations are unchanged.

Every snapshot was regenerated with the accepted runtime at `adc282a` and
equals its earlier snapshot byte for byte, so the rules behave as before;
instance names, positions, sizes, orientations, and tags are unchanged. `Matrix.silhouette` now also takes letters, so the
orientation call's few-shot crops read exactly as before. No solver run has
measured this version, so its difference is unknown, not zero.

### Dynamic mirror axis

On 8-Nov-2026 the user asked to rework this dataset to use "more dynamic with
@first" instead of the constant `= 135`, adding that it had been done that
way before. Earlier the same day, the user had asked to "dynamic set
$Obj.mirror_axis instead of 135 direcly". The worker of that attempt proposed
the same endpoint rule, but concurrent edits blocked its handoff before
publication. The note recording it, `713cc5a`, was later reverted by the
user in `653ec5d`. Another session's uncommitted revision applied the rule
too, but it also restored the removed instance keys and comments, and it was
reverted.

A fresh arc-dataset worker, explicitly authorized to replace this dataset,
kept the whole mosaic as the figure. It saved the rule, fixed from the
training pairs, before it extracted the test pair from the puzzle, and the
test pair changed nothing. It disclosed that the committed dataset it
reworked already held the annotated test pair, so it had seen those grids
before. It adopted the earlier attempt's rule unchanged after weighing these
alternatives, measured on the training pairs alone:

- The constant `135`, `(orientation@first + 135) % 360`, and
  `90 + 45 * height@first / width@first` each reproduce all 4 training pairs.
  But each states the same constant, which gives only the axis's direction,
  not where the line lies.
- The same ends without `@first` reproduce them too, but they are not
  anchored to the input state.
- `[[x, y], [x, y]]` lists leave the order of x and y to a convention.
- Exclusive corners, ending at 3 and 3, name a point outside the canvas's
  cells.
- The canvas's diagonal, from `canvas_width` and `canvas_height`, reads the
  canvas rather than the figure.
- Rules that also swap `x` and `y` only restate the 0 and 0 origin.
- Rows turned into columns would be finer than the training pairs need.
- The other diagonal, both center lines, the origin swap alone, no rule, and
  each of the three turns reproduce no training pair.

Only the rule and the five output snapshots changed. The grids, masks,
letters, instance names, implementations, and input snapshots are
byte-identical to the previous version, and the worker's recount confirmed
that the letters still follow the ranking above. The worker worked in an
external directory and changed nothing in the repository. It staged three
things:

- the dataset;
- its README paragraph's new text;
- a fix to the `dataset.6e02f1e3.json` paragraph, which had cited this
  dataset's `135` as the way it names its axis.

All 42 of its checks passed. Among them:

- every snapshot reruns, also in reverse order and with the rule after the
  instances;
- mirroring across each axis reproduces every output;
- every crop passes `Matrix.silhouette`.

The caller checked the handoff against the declarative, higher-order,
gestalt, and letter requirements and found that it met them. The pull before
integration brought in `453a0da`, "NUC 92 List Comprehensions", a parser
change. So the caller rebuilt the runtime and reran its own checks of the
staged file on the new build. All 24 dataset checks and all 4 mirror checks
passed: the letter checks covered the whole file, and every other check read
the training pairs alone. The mirror checks confirm that:

- reflecting each training input mask across its axis gives its output mask;
- the axis runs from each input box's top-left to its bottom-right corner;
- the box, orientation, and tag stay unchanged.

After integration, `document()` loads the dataset as the 15th of the 31 JSON
instruction datasets, with every entry matching its file, and all 536 crops
of the 31 datasets pass `Matrix.silhouette`. No NUC was requested, so no
proposal strategy was selected and strategy learning did not apply. The
caller made the puzzle the solver's default in `src/app.ts`, in place of
`6fa7a44f`, before the run.

## Runs

| Date | Dataset version | Runtime | Exit status | Difference | Elapsed |
| --- | --- | --- | --- | --- | --- |
| 5-Nov-2026 05:46 UTC | 1, one mosaic per object | Accepted `a605465`, local | 0 | 0, exact match | 1m 55.1s |
| 8-Nov-2026 14:14 UTC | Dynamic mirror axis | Accepted `453a0da`, local | 1 | None: the run failed before a prediction | 51.7s, runner's clock |

The run was `bun start 74dd1130` from `arc/`, run once by `/arc-purposal` with
a 30-minute limit and `NUCLEOID_EXECUTABLE` unset, after
`bun install --frozen-lockfile` reported no changes. It used the primary
checkout's debug runtime (`nucleoid 1.0.0`), built with `cargo build` at
`a605465`, and the committed solver at `a605465` with only its default puzzle
changed. The solver reported an exact match, a difference of 0, and an elapsed
time of 1m 55.1s, and wrote `logs/74dd1130.log.json`, which Git ignores.
Unlike the runs of [25d8a9c8](ARC-NUC-25d8a9c8.md),
[25ff71a9](ARC-NUC-25ff71a9.md), and [3c9b0459](ARC-NUC-3c9b0459.md), it
reached the model and scored a prediction. The hashes of the runtime, the
dataset, and `src/app.ts` were the same before and after the run. The
invocation authorizes one run, so none was repeated. The puzzle has not been in
an **ARC puzzle report** workflow run.

The match does not show that the solver inferred the flip from the training
pairs alone: as the first open item explains, the run's few-shot context held
this dataset, its annotated test pair and expected output included.

None of these runs measured the orientation version of 6-Nov-2026: the user
chose offline verification for the batch, so its difference is unknown, not
zero.

At the user's choice, the second run tested the dynamic-axis version: one
`bun start 74dd1130` from `arc/`, with a 30-minute limit and
`NUCLEOID_EXECUTABLE` unset, after `bun install --frozen-lockfile` reported
no changes. The setup was:

- **Runtime:** the primary checkout's debug runtime (`nucleoid 1.0.0`),
  rebuilt with `cargo build` at `453a0da`, whose runtime sources last changed
  in that commit.
- **Solver:** the one committed at `453a0da`, with only its default puzzle
  changed.
- **Model:** the committed `qwen/qwen3.8-27b`, pinned to the `deepinfra/bf16`
  provider with fallbacks and caching off. Each of the run's 5 model
  requests reported 0 cached input tokens.
- **Few-shot context:** `--only-puzzle-dataset` was off, so the few-shot
  context held all 31 instruction datasets, this dataset's reworked version
  and its annotated test pair among them.

The run failed with exit status 1 before it made a prediction, so it has no
difference score, and the solver printed no elapsed time. By the runner's
clock it lasted 51.7 seconds, from 14:14:11.6 to 14:15:03.3 UTC. The model
returned its class declaration split into one array element per line, so
`declarations[0]` held only the class header. `analyzer.value` builds each
input state from `declarations[0]` and the instance's implementation alone.
Nucleoid returns `null` for a program whose class has no body, as an offline
replay of that program confirms. So the first training instance's input
state was `null`, which the solver accepted. Step 1.12, the second
instance's orientation call, then failed with
`TypeError: Cannot read properties of null (reading 'orientation')` at
`src/analyzer.ts:111`.

The error log holds bun's command line, that error, and the script's exit
line. The run wrote no `logs/74dd1130.log.json`; the file there is from the
5-Nov-2026 run. The runtime's hash and every instruction dataset's hash were
the same before and after the run, and no commit reached the primary
checkout or the remote during it. The invocation authorized one run, so none
was repeated, and nothing was revised from its output. `bun run typecheck`
and `bun run lint` passed with the new default after the run.

## Open items

- **A malformed declarations response passes unchecked.** Suggested, not
  applied, in `src/analyzer.ts`:
  - Validate the `declarations` that `analyzer.declarations` returns before
    any code runs. Every element must be a complete statement, so reject a
    response whose class body arrives as separate indented elements. For
    example, run `declarations[0]` alone after the canvas variables and
    require that it declares the class. Answer a failure with a JSON error
    code and the field path, retried within the call's budget, as the ARC
    instructions require.
  - Have `analyzer.value` treat an `input_value` that is not an object with
    the instance's `id` as an error, rather than passing `null` on.
    `visualizer.value` builds its states the same way and would need the
    same check.

  The evidence is the 8-Nov-2026 run alone, not its expected grid. Its first
  input state came back `null` from a header-only `declarations[0]`, the
  next step failed with a `TypeError`, and an offline replay of the
  header-only program gives `null`.
- **Dynamic-axis version unscored.** The one run of the reworked version
  failed before a prediction, so its difference is unknown, not zero.
- **A run sees the answer.** Resolved on 6-Nov-2026: the user decided this is
  intended, because the datasets are being prepared as few-shot examples and
  runs at this stage are not yet held-out tests. The ARC instructions now say
  so, and the suggested change to keep the running puzzle's test pairs out of
  `document(puzzle_id)` is withdrawn. `src/lib/llm.ts` sends
  `dataset.document(puzzle_id)` as few-shot context, and
  `src/dataset/dataset.ts` discards `puzzle_id`. This run's prompts therefore
  held this dataset's annotated test pair, expected output included, so its
  exact match does not show that the flip, or `mirror_axis`, a property no
  other dataset uses, was learned from the training pairs.
- **Messages are not JSON-only.** `src/lib/llm.ts`, `src/analyzer.ts`, and
  `src/visualizer.ts` wrap JSON in text labels such as `few_shot:`,
  `control_id:`, `train_dataset:`, and `json_format:`, while the ARC
  instructions require every message to parse as one JSON document.
  Suggested, not applied: build each message with `JSON.stringify` of one
  data object.
- **Lint fails on untouched solver files.** Resolved: at `453a0da`,
  `bun run lint` and `bun run typecheck` both pass. At `a605465`,
  `bun run lint` reported 28 `prettier/prettier` errors, 4 in
  `src/analyzer.ts` and 24 in `src/visualizer.ts`.
- **Comments in declarations.** Resolved on 6-Nov-2026: the user decided that
  natural-language `#` comments may reach the model only as part of
  `declarations`, and the ARC instructions now say so. Datasets with and
  without such comments both follow them, so nothing needs to change. On
  7-Nov-2026 the user also allowed them in `implementation`, and each
  implementation in this dataset opened with comment lines that described
  its object until `ac49bb3` removed every dataset comment on 8-Nov-2026.
- **Workflow timeout.** Resolved on 6-Nov-2026: at the user's request, the
  arc-dataset profile now asks for the batch budget the workflow needs,
  `ceil(listed puzzles / ARC_CONCURRENCY) * 30 + 20` minutes, within GitHub
  Actions' 360-minute job limit. The workflow's 150-minute timeout covers its
  20 listed puzzles.
- **Orientation version unmeasured.** No solver run has measured the
  6-Nov-2026 orientation version, so whether consistent orientation labels
  change this puzzle's result is unknown.
- **Letter version unmeasured.** No solver run has scored the 8-Nov-2026
  letter version: the dynamic-axis run carried its letters but failed before
  a prediction. The solver still sends this puzzle's own grids as numbers,
  while every few-shot dataset writes its colors as letters.

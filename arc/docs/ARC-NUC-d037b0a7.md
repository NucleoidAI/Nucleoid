# ARC-NUC-d037b0a7 -- Downward Bars

Progress record for ARC training puzzle `d037b0a7`: its instruction dataset,
the solver and Nucleoid changes it needed, its runs, and what is still open.

| Item | Status |
| --- | --- |
| Puzzle | [`data/training/d037b0a7.json`](../data/training/d037b0a7.json) |
| Instruction dataset | [`dataset.d037b0a7.json`](../src/dataset/dataset.d037b0a7.json): the puzzle's 3 original training pairs and its test pair, 11 point instances (3, 3, 2, and 3 per pair); version 2, which reads the canvas height from the solver's `canvas_height`, committed in `f771c15` and moved to the standard entries in `521aa3c`, replacing version 1, which `81c0208` committed; shared orientation comment added on 6-Nov-2026; descriptive `implementation` comments added on 7-Nov-2026 |
| Output canvas | `EXACT` |
| Nucleoid enhancements | None requested |
| Solver changes | On 4-Nov-2026, at the user's request, every `nucleoid.run` call in `src/analyzer.ts` and `src/visualizer.ts` declares `canvas_width` and `canvas_height` at the start of its program; one suggested change awaits review, not applied |
| Solver runs | 1 local run on 3-Nov-2026 for version 1, exit 0; version 2 not run; none of the current version |
| Updated | 6-Nov-2026 |

## Puzzle

Each training input is a 3 by 3 canvas with two or three single-cell points
of different colors, no two in the same column. The output keeps the canvas
and extends every point straight down, in its own color, to the bottom row. A
point on the top row becomes a three-cell bar, a point on the middle row a
two-cell bar, and a point on the bottom row stays a single cell. The bottom
comes from the canvas, not from the other points: in the second and third
training pairs the lowest point is on the middle row, yet the bars reach the
bottom row. The third training output keeps an empty right column, which
`FIT` would crop, so the dataset is `EXACT`.

## Gestalt overview

- **Whole grid.** A 3 by 3 background canvas with two or three single-cell
  points, each of its own color and no two in the same column.
- **Figures.** Each point is a whole figure: its own color sets it apart, and
  it touches no other point through an edge.
- **Relations.** None between points. Each relates only to the canvas's
  bottom edge, which every bar reaches even where no point is on the bottom
  row.
- **Change.** Every point extends straight down in its own color to the
  bottom row, becoming a vertical bar.
- **World model.** One object per point, the figure seen first, whose bar
  takes its height from the canvas.

## Dataset

The dataset holds the three original training pairs in source order, followed
by the original test pair at the same level. Every nonzero input cell is one
instance of `Obj(x, y, orientation, color)`, tagged `l7544`, a single point,
as in `dataset.0ca9ddb6.json`; no object carries the canvas size or a cell
matrix. The rules read the canvas's height from `canvas_height`, which the
solver declares at the start of every program. Two unconditional class-level
rules turn every point into a bar:

```nuc
$Obj.tag = 'l6953'
$Obj.bar_height = canvas_height - $Obj.y
```

`l6953` is a vertical bar, as in `dataset.9af7a82c.json`. `bar_height` is
absent from the input states. `x` and `y` keep the point's position, which is
the bar's top cell, and orientation is 0 in both states. Each bar depends only
on its own point and the canvas, so no rule relates objects to each other.
Like the other ten datasets since `e1bdb5a`, the declarations carry `#`
comments; the native snapshots are the same without them. The
[README](../README.md#instruction-datasets) describes the dataset in full.

The arc-dataset worker fixed the rules from the training pairs before reading
the test pair, and the three training outputs already matched by then. The
same rules reproduce the test pair. `arc/README.md` already held a paragraph
for this dataset from `feef5f8`, although no dataset file was ever committed.
That paragraph already described this model, so the worker kept it and added
only that the rules also set the output tag. It also added `d037b0a7` to the
**ARC puzzle report** workflow's `ARC_PUZZLES` list.

### Versions

| Version | Source | Instances | Rules |
| --- | --- | --- | --- |
| 1 | `/arc-purposal d037b0a7`, committed in `81c0208` | One per nonzero input cell, a point tagged `l7544` with `color` and `canvas_height`: 11 | The output tag and `$Obj.bar_height = $Obj.canvas_height - $Obj.y` |
| 2 | The user's request on 4-Nov-2026 that every `nucleoid.run` call declare `canvas_width` and `canvas_height`, so that datasets need not carry them | The same points, without `canvas_height`: 11 | The two rules above |

Version 2 removes `canvas_height` from the class, the 11 implementations, and
their snapshots, and its rule reads the solver's variable instead of the
property. Regenerated with the accepted runtime, built at `6d3acec`, from
programs that start with the same canvas declarations as the solver's
`nucleoid.run` calls, every input and output state equals version 1's
without `canvas_height`, including each `bar_height`, so the rules still
reproduce every pair. Instance names, masks, pair grids, and the output canvas
are unchanged.

### Gestalt review

On 5-Nov-2026 the dataset was checked against the gestalt requirement, which
`/arc-purposal` and the arc-dataset profile added on 4-Nov-2026, after this
dataset was written: start from the big picture, the figures each grid shows,
and go down to single cells only when that cannot express what the training
pairs show. Version 2 meets it. Each nonzero cell of a training input has a
color of its own and touches no other through an edge, so each point is a
whole figure, not a piece of a larger shape, and each grows into its own bar.
Nothing needs to change.

### Standard entries

On 5-Nov-2026, at the user's request, every ARC dataset moved to the standard
object entries `x`, `y`, `width`, `height`, and `orientation`. `width` and
`height` are the column and row counts of the object's input bounding box. An
orientation change alone leaves them unchanged, and a rule changes them only
when the shape itself changes size, reading the input with `@first`. Every
implementation now passes the input box's `width` and `height` after `y`, and
the class and rules read:

```nuc
class Obj(x: int, y: int, width: int, height: int, orientation: int, color: int):
$Obj.tag = 'l6953'
$Obj.height = canvas_height - $Obj.y@first
```

`bar_height` became the standard `height`: `$Obj.height = canvas_height -
$Obj.y@first`, which reads the point's input row. The migration worker
regenerated every snapshot with the accepted runtime at `491512d`, and the
caller replayed every pair offline against the unchanged grids, masks,
positions and instance names. No solver run has measured this version yet.

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
confirmed: each point is 0, and so is its output, a vertical bar along its
axis or the point itself. Only the comment line changed, so the class, the
rules, the grids, masks, positions, instance names, implementations, and
snapshots are as before, and the gestalt overview above still holds.

The arc-dataset worker made the change in an external staging directory with
the accepted runtime at `d3413e6` (last runtime change `7096171`), and every
snapshot reruns byte for byte on it. The caller's own offline check passed on
the staged file and again after integration: the pairs, grids, masks,
positions, and instance names are as before, every state's orientation matches
the convention's labeller, every snapshot replays on that runtime, and
`document()` loads the dataset with the other 28. No NUC was requested, so no
proposal strategy was selected and strategy learning did not apply. At the
user's choice, no solver run has measured this version.

## Runs

| Date | Dataset version | Runtime | Exit status | Difference | Elapsed |
| --- | --- | --- | --- | --- | --- |
| 3-Nov-2026 06:15 UTC | 1, one point per nonzero cell | Accepted `e1bdb5a` | 0 | 0 | 3m 3.8s |

The run was `bun start d037b0a7` from `arc/`, run once by `/arc-purposal` with
a 30-minute limit. It used the primary checkout's debug runtime
(`nucleoid 1.0.0`), the same binary that produced the dataset's snapshots.
`cargo build` could not replace that binary, because an open `nucleoid.exe`
REPL locked it. It had been built after the last runtime commit, `4c57af8`
(NUC 75), and since that build only uncommitted REPL edits in `src/terminal/`
and an uncommitted `rand` dependency had changed, so it ran the accepted
runtime. Its hash was the same before and after the run. Elapsed is the
solver's own report. The puzzle has not been in an **ARC puzzle report**
workflow run. Version 2 has not been run; measuring it takes a new
`/arc-purposal d037b0a7` invocation, which authorizes its one run.

None of these runs measured the orientation version of 6-Nov-2026: the user
chose offline verification for the batch, so its difference is unknown, not
zero.

## Open items

- **The run saw the answer.** Resolved on 6-Nov-2026: the user decided this is
  intended, because the datasets are being prepared as few-shot examples and
  runs at this stage are not yet held-out tests. The ARC instructions now say
  so, and the suggested change to keep the running puzzle's test pairs out of
  `document(puzzle_id)` is withdrawn. `src/lib/llm.ts` sends
  `document(puzzle_id)` as few-shot context, and `src/dataset/dataset.ts`
  discards `puzzle_id`. The prompt therefore held this puzzle's annotated test
  pair, expected output included, and the exact match does not show that the
  rule was inferred from the training pairs.
- **Canvas height, resolved by version 2.** In version 1 the model had to
  fill in `canvas_height` for the test instance, although `value()` in
  `src/visualizer.ts` sends only the cropped mask with `input_x` and
  `input_y`, not the test canvas's size. Since 4-Nov-2026 every `nucleoid.run`
  call declares `canvas_width` and `canvas_height` at the start of its
  program, from the test input in the visualizer, and version 2 reads
  `canvas_height` from there, so the model no longer supplies it. This
  replaces the earlier suggestion to pass the test input's row and column
  counts to `visualizer.value`. `dataset.6150a2bd.json` and
  `dataset.44f52bb0.json` read the variables the same way.
- **Messages are not JSON-only.** `src/lib/llm.ts`, `src/analyzer.ts`, and
  `src/visualizer.ts` wrap JSON in text labels such as `few_shot:`,
  `train_dataset:`, and `input_x:`, while the ARC instructions require every
  message to parse as one JSON document. Suggested, not applied: build each
  message with `JSON.stringify` of one data object.
- **Comments in declarations.** Resolved on 6-Nov-2026: the user decided that
  natural-language `#` comments may reach the model only as part of
  `declarations`, and the ARC instructions now say so. Datasets with and
  without such comments both follow them, so nothing needs to change. On
  7-Nov-2026 the user also allowed them in `implementation`, and each
  implementation in this dataset now opens with comment lines that describe
  its object.
- **Stale README text.** `arc/README.md` still describes
  `dataset.0d3d703e.json`, `dataset.27a28665.json`, and
  `dataset.44f52bb0.json`, which do not exist. Its GitHub Actions paragraph
  names four puzzles, while the workflow now lists eight.
- **Committed.** `f771c15` committed version 2, its README paragraph, the
  solver change in `src/analyzer.ts`, `src/visualizer.ts`, and `src/app.ts`,
  and this record's update. Version 1,
  its README paragraph, the workflow entry, and this record's first version
  were committed in `81c0208`.
- **Orientation version unmeasured.** No solver run has measured the
  6-Nov-2026 orientation version, so whether consistent orientation labels
  change this puzzle's result is unknown.

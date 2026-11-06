# ARC-NUC-a85d4709 -- Marker Columns Color Their Rows

Progress record for ARC training puzzle `a85d4709`: its instruction dataset,
the solver and Nucleoid changes it needed, its runs, and what is still open.

| Item | Status |
| --- | --- |
| Puzzle | [`data/training/a85d4709.json`](../data/training/a85d4709.json) |
| Instruction dataset | [`dataset.a85d4709.json`](../src/dataset/dataset.a85d4709.json): the puzzle's 4 original training pairs and its test pair, 15 marker instances, 3 in each pair; committed with this record |
| Output canvas | `EXACT` |
| Nucleoid enhancements | None requested, so no proposal strategy was selected and strategy learning did not apply |
| Solver changes | None; earlier suggested changes still await review, not applied |
| Solver runs | 1 local run on 5-Nov-2026, exit 6, a difference of 6, not an exact match, with this dataset's test pair in its few-shot context |
| Updated | 6-Nov-2026 |

## Puzzle

Each training input is a 3 by 3 canvas with one gray cell, color 5, in each
row and zero cells everywhere else. The output keeps the canvas and paints
every row a single color, chosen by the column of that row's gray cell: 2 for
the left column, 4 for the middle column, and 3 for the right column, so no
output cell is zero. The column alone decides. In the first pair the gray
cells run diagonally, in the second they all sit in the right column, and the
third and fourth hold the same chevron one column apart, yet each column gives
the same color in every pair where it holds a gray cell. The output keeps the
input's 3 by 3 canvas, so the dataset is `EXACT`; with no zero cell in any
output, `FIT` would frame the same grids.

`/arc-purposal` chose this puzzle by default: among the 383 training puzzles
other than the genesis puzzle `3aa6fb7a` and those with a dataset, its largest
training matrix has the fewest cells, 9, with 72 cells in all its training
matrices. `ed36ccf7` ties on both measures, and `a85d4709` has the lower ID.
Both had also tied with [`9565186b`](ARC-NUC-9565186b.md), which the previous
invocation took.

## Gestalt overview

- **Whole grid.** A 3 by 3 background canvas with one gray cell, color 5, in
  each row; the output is painted throughout.
- **Figures.** The three gray cells touch through edges or corners and form
  one figure from top to bottom: a diagonal line, a vertical line, or a
  right-pointing chevron.
- **Relations.** Each row's gray cell sits in a column, and the column
  decides the row's color; the figure's shape does not, since the two
  chevrons color their rows differently.
- **Change.** Every row is painted solid in the color its gray cell's column
  selects: 2 for the left column, 4 for the middle, and 3 for the right.
- **World model.** One object per gray cell, one level below the whole
  figure. The figure does not change as one, since each of its rows takes its
  own color, and its shape does not decide the colors, so the model went down
  to the row, where each row holds one figure, its marker.

## Dataset

The dataset holds the four original training pairs in source order, followed
at the same level by the puzzle's test pair, which the same rules annotate.
Each object is one gray cell, the marker of its row: an instance of
`Obj(x, y, orientation)`, tagged `l7544`, a single point, as in
`dataset.0ca9ddb6.json` and `dataset.d037b0a7.json`. The class takes no
descriptive property. Every marker is gray, which the rules never read, and
its column is its `x`; the class has no color, list, canvas size, or matrix of
cells. Four unconditional class-level rules turn each marker into its row's
bar:

```nuc
$Obj.tag = 'l3058'
$Obj.fill_color = [2, 4, 3][$Obj.x@first]
$Obj.bar_width = canvas_width
$Obj.x = 0
```

`l3058` is the horizontal bar of `dataset.d631b094.json` and
`dataset.25d8a9c8.json`. `fill_color` indexes a list of one color per column,
from left to right, with the marker's input column, anchored with `@first`
because the last rule moves `x` to the left edge. `bar_width` reads the
solver's `canvas_width`, as `dataset.d037b0a7.json` reads `canvas_height`,
and `y` keeps the marker's row. The rules create only `fill_color` and
`bar_width`, which input states lack. The declarations carry no `#` comments.
The [README](../README.md#instruction-datasets) describes the dataset in full.

The worker started from the big picture. In each training input the three
gray cells touch through edges or corners and form one figure from the top row
to the bottom: a diagonal line, a vertical line, and the same right-pointing
chevron in two positions. That fell short in two ways:

- The figure does not change as one: each of its rows takes its own color,
  which no scalar property of a single figure object can state.
- Its shape does not decide the colors: the two chevrons turn their rows 2, 4,
  and 2, and 4, 3, and 4.

The model therefore went down to the row, the unit in which every output
changes, and each row holds one figure, its marker. Each bar depends only on
its own marker, and the training pairs show no relation between markers, so no
population operation appears. The bar's length comes from the canvas, not the
other markers: in the second pair every marker sits in the right column, yet
every bar spans its whole row. The training pairs also fit two other readings,
because every canvas is 3 columns wide. The second pair's vertical line could
be one object that widens into a 3 by 3 block, and the colors could follow the
canvas edges rather than the columns. The worker took one marker per row and a
lookup by column in every pair. It fixed the rules from the training pairs
before it read the test pair and reported that the same rules reproduce it.

The caller checked the handoff against the declarative, higher-order, and
gestalt requirements and the user's ARC conventions, found that it met them,
and sent nothing back. It replayed the four training pairs on the accepted
runtime, and all 233 checks passed. They covered:

- grids and keys;
- mask shapes and cells, bounding boxes, and merges;
- implementations and instance names;
- input and output snapshots and the output positions.

The caller also ran each training pair with its three instances in every
order, the rules declared before and after them. All 48 programs gave every
marker its snapshot. `document()` includes the dataset as the 15th of the 21
JSON instruction datasets, in filename order.

The worker wrote the dataset, its README paragraph, and the puzzle's entry in
the **ARC puzzle report** workflow's list, its 18th puzzle. The caller made
the puzzle the solver's default in `src/app.ts`, in place of `9565186b`,
before the run.

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
$Obj.tag = 'l3058'
$Obj.fill_color = [2, 4, 3][$Obj.x@first]
$Obj.width = canvas_width
$Obj.x = 0
```

`bar_width` became the standard `width`: `$Obj.width = canvas_width`, and the
bar stays one row tall. The migration worker regenerated every snapshot with
the accepted runtime at `491512d`, and the caller replayed every pair offline
against the unchanged grids, masks, positions and instance names. No solver
run has measured this version yet.

## Runs

| Date | Dataset version | Runtime | Exit status | Difference | Elapsed |
| --- | --- | --- | --- | --- | --- |
| 5-Nov-2026 06:29 UTC | 1, one marker per row | Accepted `9318a95`, local | 6 | 6, mismatch | 5m 19.6s |

The run was `bun start a85d4709` from `arc/`, run once by `/arc-purposal` with
a 30-minute limit and `NUCLEOID_EXECUTABLE` unset, after
`bun install --frozen-lockfile` reported no changes. It used the primary
checkout's debug runtime (`nucleoid 1.0.0`), current under `cargo build` at
`9318a95`, whose runtime sources last changed in `a605465`. The solver was the
committed one at `9318a95`, with only its default puzzle changed. The solver
reported a mismatch: a difference of 6, the number of cells that
`Matrix.difference` finds unequal, and an elapsed time of 5m 19.6s. Its error
output held only the exit code, and it wrote `logs/a85d4709.log.json`, which
Git ignores. The hashes of the runtime, the dataset, and `src/app.ts` were the
same before and after the run. The invocation authorizes one run, so none was
repeated, and nothing was changed from its output. The puzzle has not been in
an **ARC puzzle report** workflow run.

The run missed even though its few-shot context held this dataset, its
annotated test pair and expected output included, as the second open item
explains.

## Open items

- **Unexplained mismatch.** The invocation read only the run's exit status,
  difference, error output, and elapsed time, because the log shows the
  expected test grid, so why the prediction differs is unknown. A separate
  `/arc-debug a85d4709` can look for abnormalities in the pipeline stages.
  Any dataset revision must come from the training pairs, never from the
  run's output.
- **A run sees the answer.** Resolved on 6-Nov-2026: the user decided this is
  intended, because the datasets are being prepared as few-shot examples and
  runs at this stage are not yet held-out tests. The ARC instructions now say
  so, and the suggested change to keep the running puzzle's test pairs out of
  `document(puzzle_id)` is withdrawn. As in earlier records, `src/lib/llm.ts`
  sends `dataset.document(puzzle_id)` as few-shot context, and
  `src/dataset/dataset.ts` discards `puzzle_id`, so this run's prompts held
  this dataset's annotated test pair, expected output included.
- **Messages are not JSON-only.** `src/lib/llm.ts`, `src/analyzer.ts`, and
  `src/visualizer.ts` still wrap JSON in text labels such as `few_shot:`,
  `control_id:`, `train_dataset:`, and `json_format:`, while the ARC
  instructions require every message to parse as one JSON document.
  Suggested, not applied: build each message with `JSON.stringify` of one
  data object.
- **Lint fails on untouched solver files.** At `9318a95`, `bun run lint`
  reports 28 `prettier/prettier` errors, all in `src/analyzer.ts` and
  `src/visualizer.ts`, which this invocation did not change. `src/app.ts`
  lints clean with the new default, and `bun run typecheck` passes.
  Suggested, not applied: reformat both files with Prettier, a
  formatting-only change.
- **Workflow timeout.** Resolved on 6-Nov-2026: at the user's request, the
  arc-dataset profile now asks for the batch budget the workflow needs,
  `ceil(listed puzzles / ARC_CONCURRENCY) * 30 + 20` minutes, within GitHub
  Actions' 360-minute job limit. The workflow's 150-minute timeout covers its
  20 listed puzzles.

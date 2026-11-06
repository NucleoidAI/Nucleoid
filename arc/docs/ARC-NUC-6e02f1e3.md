# ARC-NUC-6e02f1e3 -- Color Diversity Lines

Progress record for ARC training puzzle `6e02f1e3`: its instruction dataset,
the solver and Nucleoid changes it needed, its runs, and what is still open.

| Item | Status |
| --- | --- |
| Puzzle | [`data/training/6e02f1e3.json`](../data/training/6e02f1e3.json) |
| Instruction dataset | [`dataset.6e02f1e3.json`](../src/dataset/dataset.6e02f1e3.json): the puzzle's 5 original training pairs and its test pair, 6 mosaic instances, one per pair; committed with this record. An earlier invocation the same day verified a different model offline and did not publish it |
| Output canvas | `EXACT`, inferred from the training pairs |
| Nucleoid enhancements | None requested, so no proposal strategy was selected and strategy learning did not apply; the rules use the accepted `distinct` list query of [NUC 71](../../docs/nuc-0071.md) |
| Solver changes | None; suggested changes await review, not applied |
| Solver runs | 1 local run on 5-Nov-2026, exit 1 before any model call (`OPENROUTER_API_KEY is required`); difference unknown |
| Updated | 6-Nov-2026 |

## Puzzle

Each training input is a completely filled 3 by 3 mosaic. The number of
distinct categorical colors determines the direction of one gray line:
one color produces a horizontal bar across the top, two produce a
northwest-to-southeast diagonal, and three produce a
southwest-to-northeast diagonal. The gray foreground is color 5 and the
output background is color 0. The original 3 by 3 canvas stays intact,
including the two empty rows beneath a horizontal bar, so the model uses
`EXACT`, not `FIT`.

`/arc-purposal` chose this puzzle by default twice on 5-Nov-2026. Both times
it was the first of 381 eligible puzzles: its largest training input or output
matrix has 9 cells, and all its training matrices total 90 cells. `794b24be`
also has a largest matrix of 9 cells, but 180 cells in all. The first
invocation started from `f4cf6803` and published no dataset, so the puzzle
stayed eligible. The second started from `c57cbd6`. Both times the working
tree was clean and the pulls brought in no commits.

## Gestalt overview

- **Whole grid.** A 3 by 3 canvas filled by a mosaic of one, two, or three
  colors; the output is one gray line on the background.
- **Figures.** One figure per grid, the whole mosaic. Its color regions are
  not figures of their own here, since the output line follows none of them.
- **Relations.** Only the mosaic's variety matters: how many distinct colors
  it holds.
- **Change.** The mosaic gives way to one gray line, color 5, across the
  whole canvas: the top row for one color, the diagonal from the top-left
  corner for two, and the diagonal from the bottom-left corner for three.
- **World model.** One object per pair, the whole mosaic, with `parts`
  listing its colors as data; its line is one tag whose orientation gives the
  axis.

## Dataset

Each grid's whole mosaic is one instance of `Obj(x, y, orientation, parts)`,
tagged `l9100`, a single shape, as in `dataset.28bf18c6.json` and
`dataset.3c9b0459.json`. `parts` is the flat list of the mosaic's cell colors
in reading order, as in `dataset.5582e5ca.json`; it is the only descriptive
property, so no input property names the color count or the line. Three
unconditional class-level rules turn the mosaic into its line:

```nuc
$Obj.tag = 'l6074'
$Obj.orientation = {1: 90, 2: 135, 3: 45}[$Obj.parts.distinct().length]
$Obj.fill_color = 5
```

`distinct` keeps each of the mosaic's colors once, so its length is the number
of colors. The object literal states each count's axis once: 90 for the
horizontal top row, 135 for the main diagonal, and 45 for the other diagonal.
The new tag `l6074` names a straight line of cells across the whole canvas,
along the axis its orientation gives, so the rules state no length. Every line's
box starts at the canvas origin, where the mosaic's box starts, so `x` and `y`
stay unchanged. The rules create only `fill_color`, which is absent from input
states. The [README](../README.md#instruction-datasets) describes the dataset in
full.

The worker started from the big picture, the whole mosaic as one figure, and
stayed at that level. It rejected color regions as objects, because the output
line follows none of them. The training pairs alone also fit counting
edge-connected pieces instead of colors, since every training color forms one
piece. The worker counted colors and froze the rules, with their hash, before
it extracted the test pair.

This model differs from the unpublished one below. It uses one line tag whose
orientation gives the axis, where the earlier model used separate bar and
diagonal tags. It drops `bar_width` and `line_length`, which only restated the
canvas width. And it looks the axis up by count in an object literal instead of
indexing a list with the count minus one.

The caller checked the handoff against the declarative, higher-order, and
gestalt requirements and found that it met them: each relation is stated once
over every mosaic, no property restates another, and no rule reaches below the
whole figure. It sent nothing back. Its own replay of the five training pairs
on the accepted runtime passed all 84 checks in 15 Nucleoid runs:

- grids and key order;
- mask shapes, cells, bounding boxes, and merges;
- implementations and orientations;
- input and output snapshots, with the rules declared before and after the
  instance;
- declarations without `#` comments, and instance names unique across all
  datasets.

The worker's own checks also passed, among them `document()` loading the
dataset in filename order.

The worker wrote the dataset, its README paragraph, and the puzzle's entry in
the **ARC puzzle report** workflow's list, its 20th puzzle. The caller made the
puzzle the solver's default in `src/app.ts`, in place of `ed36ccf7`, before
the run.

### Earlier unpublished model

The first invocation on 5-Nov-2026 also started from the entire mosaic as one
pattern with a row-major `parts` list, and verified this declaration set
offline for all six original pairs:

```nuc
class Obj(x: int, y: int, orientation: int, parts: list):
    this.x = x
    this.y = y
    this.orientation = orientation
    this.tag = 'l9100'
    this.parts = parts

$Obj.fill_color = 5
$Obj.orientation = [90, 135, 45][$Obj.parts.distinct().length - 1]

if $Obj.orientation == 90:
    $Obj.tag = 'l3058'
    $Obj.bar_width = canvas_width

if $Obj.orientation == 45 or $Obj.orientation == 135:
    $Obj.tag = 'l8831'
    $Obj.line_length = canvas_width
```

It reused `l3058` for the horizontal bar and proposed `l8831` for a
one-cell-thick diagonal. Its model and evidence stayed in that invocation's
session workspace. No canonical dataset, README paragraph, or reporting-list
entry was created.

### Publication blocker

The first invocation's worker returned `blocked` on the workflow timeout
wording. The arc-dataset profile asks for a job timeout of at least thirty
minutes per listed puzzle plus twenty minutes for setup and reporting, within
GitHub Actions' job limit. Read serially, adding this puzzle to the nineteen
listed would require 620 minutes, beyond the 360-minute hosted job limit.

The workflow already runs the puzzles in batches of `ARC_CONCURRENCY`, 5, and
the README asks for a timeout budget sufficient for all batches. The second
invocation sized the timeout by batch, as the `ed36ccf7` invocation did:
twenty puzzles take four batches of 30 minutes plus 20, or 140 minutes. That
fits the existing 150-minute timeout, so the timeout was left unchanged.

### Standard entries

On 5-Nov-2026, at the user's request, every ARC dataset moved to the standard
object entries `x`, `y`, `width`, `height`, and `orientation`. `width` and
`height` are the column and row counts of the object's input bounding box. An
orientation change alone leaves them unchanged, and a rule changes them only
when the shape itself changes size, reading the input with `@first`. Every
implementation now passes the input box's `width` and `height` after `y`, and
the class and rules read:

```nuc
class Obj(x: int, y: int, width: int, height: int, orientation: int, parts: list):
$Obj.tag = 'l6074'
$Obj.orientation = {1: 90, 2: 135, 3: 45}[$Obj.parts.distinct().length]
if $Obj.orientation == 90:
    $Obj.height = 1
$Obj.fill_color = 5
```

One conditional rule sets `$Obj.height = 1` when `$Obj.orientation == 90`,
giving the top-row line its one-row height, while the diagonals stay 3 by 3.
The migration worker regenerated every snapshot with the accepted runtime at
`491512d`, and the caller replayed every pair offline against the unchanged
grids, masks, positions and instance names. No solver run has measured this
version yet.

## Runs

| Date | Dataset version | Runtime | Exit status | Difference | Elapsed |
| --- | --- | --- | --- | --- | --- |
| 5-Nov-2026 09:37 UTC | 1, one mosaic per object | Accepted `c57cbd6`, local | 1 | Unknown: `Error: OPENROUTER_API_KEY is required` | 0.5s |

The first invocation on 5-Nov-2026 started no run, because its dataset handoff
was blocked. It left the solver's default at `ed36ccf7`.

The run was `bun start 6e02f1e3` from `arc/`, run once by `/arc-purposal` with
a 30-minute limit and `NUCLEOID_EXECUTABLE` unset. Before it,
`bun install --frozen-lockfile` reported no changes. It used the primary
checkout's debug runtime (`nucleoid 1.0.0`), built with `cargo build` at
`c57cbd6`, and the committed solver at `c57cbd6` with only its default puzzle
changed.

The run stopped in the analyzer's first model call, `analyzer.declarations`,
where `src/lib/openrouter.ts` raised `OPENROUTER_API_KEY is required`. That was
before any model request, prediction, or score, so the difference is unknown,
not zero. The solver wrote no `logs/6e02f1e3.log.json` and reported no elapsed
time. The elapsed time is the process's run, from 09:37:48.5 to 09:37:49.0 UTC.

The runtime, the dataset, and `src/app.ts` had the same hashes before and after
the run. The invocation authorizes one run, so none was repeated. The puzzle
has not yet been in an **ARC puzzle report** workflow run.

## Open items

- **No model credential.** As in the runs of
  [ed36ccf7](ARC-NUC-ed36ccf7.md), [25d8a9c8](ARC-NUC-25d8a9c8.md),
  [25ff71a9](ARC-NUC-25ff71a9.md), and [3c9b0459](ARC-NUC-3c9b0459.md), the
  solver process had no `OPENROUTER_API_KEY` value, so it stopped before its
  first model request. Measuring this dataset takes a separately authorized
  run with the credential in the process environment. No environment file or
  credential was read.
- **A run would see the answer.** Resolved on 6-Nov-2026: the user decided
  this is intended, because the datasets are being prepared as few-shot
  examples and runs at this stage are not yet held-out tests. The ARC
  instructions now say so, and the suggested change to keep the running
  puzzle's test pairs out of `document(puzzle_id)` is withdrawn.
  `src/lib/llm.ts` sends `dataset.document(puzzle_id)` as few-shot context,
  and `src/dataset/dataset.ts` discards `puzzle_id`. Any run of this puzzle
  therefore holds this dataset's annotated test pair in its prompts, expected
  output included, and an exact match would not show that the rule was
  inferred from the training pairs.
- **Messages are not JSON-only.** `src/lib/llm.ts`, `src/analyzer.ts`, and
  `src/visualizer.ts` still wrap JSON in text labels such as `few_shot:` and
  `control_id:`, while the ARC instructions require every message to parse as
  one JSON document. Suggested, not applied: build each message with
  `JSON.stringify` of one data object.
- **Lint fails on untouched solver files.** At `c57cbd6`, `bun run lint`
  reports 28 `prettier/prettier` errors, 4 in `src/analyzer.ts` and 24 in
  `src/visualizer.ts`, which this invocation did not change. `src/app.ts`
  lints clean with the new default, and `bun run typecheck` passes.
  Suggested, not applied: reformat both files with Prettier, a
  formatting-only change.
- **Workflow timeout wording.** Resolved on 6-Nov-2026: at the user's request,
  the arc-dataset profile now asks for the batch budget the workflow needs,
  `ceil(listed puzzles / ARC_CONCURRENCY) * 30 + 20` minutes, within GitHub
  Actions' 360-minute job limit. The workflow's 150-minute timeout covers its
  20 listed puzzles.

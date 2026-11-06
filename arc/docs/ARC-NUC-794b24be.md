# ARC-NUC-794b24be -- Cell Count Tally

Progress record for ARC training puzzle `794b24be`: its instruction dataset,
the solver and Nucleoid changes it needed, its runs, and what is still open.

| Item | Status |
| --- | --- |
| Puzzle | [`data/training/794b24be.json`](../data/training/794b24be.json) |
| Instruction dataset | [`dataset.794b24be.json`](../src/dataset/dataset.794b24be.json): the puzzle's 10 original training pairs and its 2 test pairs, 12 pattern instances, one per pair; committed with this record; orientation labelled by shape and the shared orientation comment added on 6-Nov-2026 |
| Output canvas | `EXACT` |
| Nucleoid enhancements | None requested; the dataset uses accepted features only |
| Solver changes | None; one suggested change awaits review, not applied |
| Solver runs | 1 local run on 6-Nov-2026: exit 0, difference 0; none of the current version |
| Updated | 6-Nov-2026 |

## Puzzle

Each training input is a 3 by 3 canvas holding one to four blue cells of
color 1 on a background of zero cells: 1, 2, 2, 2, 1, 3, 3, 4, 4, and 4 cells
in the ten training pairs. Each output keeps the 3 by 3 canvas and replaces
the blue cells with as many red cells of color 2 in its top-left corner. One
cell gives a point, two and three cells a bar along the top row from the
left, and four cells a T, the full top row with the middle cell of the second
row. Where the blue cells lie, and whether they touch, makes no difference.
The output keeps the input's canvas, so the dataset is `EXACT`.

## Gestalt overview

- **Whole grid.** A 3 by 3 background canvas with one to four blue cells
  scattered over it; the output is the same canvas with a small red figure in
  its top-left corner.
- **Figures.** All the blue cells read as one group, whether or not they
  touch. The output's red cells form one tally: a point, a bar along the top
  row, or a T.
- **Relations.** Only the group's size matters, not its shape or position:
  three different two-cell groups give the same bar, and three different
  four-cell groups give the same T.
- **Change.** The whole group is replaced by one red tally of as many cells
  in the top-left corner: a bar filled from the left along the top row that,
  once it spans the canvas, grows a stem down from its middle cell.
- **World model.** One object per pair, the whole blue group, the figure seen
  first, measured by its scalar `cell_count`. Nothing finer was needed: one
  object per blue cell would tie each blue cell to a particular red cell,
  which no training pair shows.

## Dataset

The single dataset holds the ten original training pairs in source order,
followed by the two original test pairs at the same level. Each pair's blue
cells form one pattern object, whether or not they touch, as in
[`dataset.d631b094.json`](../src/dataset/dataset.d631b094.json). Each is an
`Obj(x, y, width, height, orientation, cell_count)` tagged `l9100`, where
`width` and `height` are its input bounding box and `cell_count` its number
of cells, with no color, list, or matrix property. Six unconditional class
rules turn it into the tally:

```nuc
class Obj(x: int, y: int, width: int, height: int, orientation: int, cell_count: int):
$Obj.tag = 'l5714'
$Obj.fill_color = 2
$Obj.width = min of [$Obj.cell_count, canvas_width]
$Obj.height = 1 + $Obj.cell_count - $Obj.width
$Obj.x = 0
$Obj.y = 0
```

The bar holds the first cells, up to the canvas's width, which the rule reads
from the solver's `canvas_width` with the noun query `min of`
([NUC 75](../../docs/nuc-0075.md)); every further cell adds one row of stem.
`height` reads the width the rules set, so it takes no `@first` anchor, and
the rules give the same states in any order. `fill_color` is absent before
the rules run, and `l5714`, the tally, is a new tag. Each grid holds a single
pattern and no relation between objects, so no population operation appears.
The [README](../README.md#instruction-datasets) describes the dataset in full.

The arc-dataset worker checked the dataset offline with the accepted runtime:
every grid equals its original, the full-size 3 by 3 masks merge back into
every input and output, every snapshot reruns byte for byte, and the loader
includes the dataset among all 24 instruction datasets in filename order.

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

Six of the twelve instances changed. In the inputs, two cells that touch at a
corner are a diagonal line without a head, 45 when it rises to the right and
135 when it falls, and the seventh training pattern is a three-cell corner
that points toward its bottom-right vertex, 135. In the outputs, a tally of
two or three cells is a horizontal bar, 90, while a single cell and the
four-cell T stay 0. One new unconditional rule sets the tally's orientation
from its own box, with the conditional expression of
[NUC 59](../../docs/nuc-0059.md) and the `is` of
[NUC 86](../../docs/nuc-0086.md). The rules now read:

```nuc
class Obj(x: int, y: int, width: int, height: int, orientation: int, cell_count: int):
$Obj.tag = 'l5714'
$Obj.fill_color = 2
$Obj.width = min of [$Obj.cell_count, canvas_width]
$Obj.height = 1 + $Obj.cell_count - $Obj.width
$Obj.orientation = 90 if $Obj.height is 1 and $Obj.width > 1 else 0
$Obj.x = 0
$Obj.y = 0
```

The grids, masks, positions, instance names, tags, and every other property
are as before; only the orientation values and the new rule changed, so the
world model keeps its level.

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

| Date | Command | Runtime | Exit status | Difference | Elapsed |
| --- | --- | --- | --- | --- | --- |
| 6-Nov-2026 10:43 UTC | `bun start 794b24be` | Accepted runtime at `2da76da` (last runtime change `3592d4b`), local | 0 | 0 | 4m 43.5s |

The run was `bun start 794b24be` from `arc/`, run once by `/arc-purposal`
with a 30-minute limit and `NUCLEOID_EXECUTABLE` unset, after
`bun install --frozen-lockfile` reported no changes. It used the primary
checkout's debug runtime (`nucleoid 1.0.0`), current with `cargo build` at
`2da76da`, and the committed solver at `2da76da` with only its default puzzle
changed. The model settings were the committed `qwen/qwen3.8-27b`, pinned to
the `deepinfra/bf16` provider. An uncommitted edit of `src/lib/openrouter.ts`
that switched to `qwen/qwen3.8-flash` without the provider pin appeared while
the dataset was being written, and the user reverted it before the run.
`--only-puzzle-dataset` was off, so the few-shot context held all 24
instruction datasets, this dataset's annotated test pairs among them, as the
ARC instructions intend while the datasets are prepared as few-shot examples.

The run ended in an exact match, exit status 0 and difference 0, in 4m 43.5s
by the solver's own report; the process ran from 10:43:16.6 to 10:48:00.8
UTC. Its error log holds only bun's command line. The runtime's hash was the
same before and after the run, and no commit landed during it. The invocation
authorizes one run, so none was repeated. The puzzle joined the **ARC puzzle
report** workflow with this dataset, so no workflow run is recorded yet.

None of these runs measured the orientation version of 6-Nov-2026: the user
chose offline verification for the batch, so its difference is unknown, not
zero.

## Open items

- **Messages are not JSON-only.** Since `f58e26a`, `src/lib/llm.ts` sends
  its system messages, the few-shot context and the control ID, as JSON. The
  user and assistant messages that `src/analyzer.ts` and `src/visualizer.ts`
  build still wrap JSON in text labels such as `train_dataset:`,
  `declarations:`, and `json_format:`, while the ARC instructions require
  every message to parse as one JSON document. Suggested, not applied: build
  each of those messages with `JSON.stringify` of one data object. This
  dataset does not depend on it.
- **Orientation version unmeasured.** No solver run has measured the
  6-Nov-2026 orientation version, so whether consistent orientation labels
  change this puzzle's result is unknown.

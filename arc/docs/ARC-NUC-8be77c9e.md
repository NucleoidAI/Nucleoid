# ARC-NUC-8be77c9e -- Grid Stacked Over Its Mirror Image

Progress record for ARC training puzzle `8be77c9e`: its instruction dataset,
the solver and Nucleoid changes it needed, its runs, and what is still open.

| Item | Status |
| --- | --- |
| Puzzle | [`data/training/8be77c9e.json`](../data/training/8be77c9e.json) |
| Instruction dataset | [`dataset.8be77c9e.json`](../src/dataset/dataset.8be77c9e.json): the puzzle's 3 original training pairs and its test pair, 4 pattern instances, 1 in each pair; committed with this record; shared orientation comment added on 6-Nov-2026 |
| Output canvas | `EXACT` |
| Nucleoid enhancements | None requested, so no proposal strategy was selected and strategy learning did not apply; the rules use the `@first` anchors of [NUC 53](../../docs/nuc-0053.md) and [NUC 55](../../docs/nuc-0055.md) |
| Solver changes | None applied; one suggested and not applied, an output canvas framed on the world model's canvas size; the JSON-only message question carried over from earlier records stays open |
| Solver runs | 1 local run on 6-Nov-2026: exit 3, difference 3; none of the current version |
| Updated | 6-Nov-2026 |

## Puzzle

Each training input is a 3 by 3 canvas holding one pattern of color-1 cells
on a background of zero cells: 5, 4, and 2 cells in the three training
pairs. Each output is a 6 by 3 canvas, twice as tall, that stacks the input
over its top-bottom mirror image: the input reappears unchanged in the top
three rows, and its row r reappears as row 5 - r, so the output reads the
same from the bottom up as from the top down. The mirror line is the
canvas's bottom edge, not the pattern's: the first training pattern lies in
the top two rows above an empty bottom row, so two empty rows separate it
from its mirror image, while the second and third patterns reach the bottom
edge and meet theirs. The output keeps the input's 3 columns and grows only
downward, so the dataset is `EXACT`.

`/arc-purposal` chose this puzzle by default: among the 377 training puzzles
other than the genesis puzzle `3aa6fb7a` and those with a dataset, its
largest training matrix has the fewest cells, 18, with 81 cells in all its
training matrices. `c9e6f938` ties on both measurements and follows it by
ID; the next, `c8f0f002`, has 18 and 90.

## Gestalt overview

- **Whole grid.** A small 3 by 3 canvas with one color-1 pattern; the output
  is the same grid on top and its reflection below, a 6 by 3 canvas that is
  symmetric about its horizontal center line.
- **Figures.** The whole color-1 pattern is the figure seen first, one per
  grid, including where its cells touch only at a corner, as the second
  training pattern's top-right cell does.
- **Relations.** The pattern sits against the canvas's edges: on the top row
  above an empty bottom row in the first pair, and against the bottom edge
  below an empty top row in the second and third. The canvas's bottom edge
  is the line the pattern is mirrored across.
- **Change.** The canvas doubles downward, and the whole grid is joined by
  its reflection across its bottom edge, so each pattern gains its mirror
  image below, keeping its color, its columns, and its place.
- **World model.** One object per grid, the whole pattern, the figure seen
  first. Its box grows down to hold the pattern and its reflection, so
  nothing finer was needed. The worker set aside mirroring across the
  pattern's own bottom edge, which fails the first training pair, and a plain
  copy or a half turn placed below, which fail all three, before it read the
  test pair.

## Dataset

The single dataset holds the three original training pairs in source order,
followed at the same level by the puzzle's test pair, which the same rules
annotate. Each object is one whole pattern: an instance of
`Obj(x, y, width, height, orientation)` with only the standard entries, whose
initial tag `l9100` is the single shape of `dataset.28bf18c6.json` and
`dataset.25ff71a9.json`. The class takes no descriptive property: the
pattern's cells travel in its masks, and the rules read the canvas's height
from the solver's `canvas_height`, so there is no color, parts list, canvas
size, or matrix of cells. Two unconditional class-level rules turn every
pattern into the figure it forms with its reflection:

```nuc
class Obj(x: int, y: int, width: int, height: int, orientation: int):
$Obj.tag = 'l7511'
$Obj.height = 2 * (canvas_height - $Obj.y@first)
```

`l7511`, a new tag, is a shape over its top-bottom mirror image, symmetric
about the horizontal center line of its box. `height` becomes twice the
distance from the pattern's top row to the canvas's bottom edge, so the
mirror line lies at the box's center: 6 rows for the first training pattern,
which starts on the top row, and 4 for the second and third, which start one
row down. The rule reads `canvas_height` as `dataset.d037b0a7.json` grows its
bars with `canvas_height - $Obj.y@first`, anchored with `@first` to read the
pattern's input row. `x`, `y`, `width`, and `orientation` stay as they were:
the reflection keeps the pattern's columns, the figure starts on the
pattern's top row, and the reflection is part of the figure `l7511` names,
not a turn. The rules create no property and state no `mirror_axis`, the
property of `dataset.74dd1130.json`, because the tag and the growing `height`
already determine the axis. Each grid holds a single pattern and shows no
relation between objects, so no population operation appears. The
declarations carry no `#` comments. The
[README](../README.md#instruction-datasets) describes the dataset in full.

The caller checked the handoff against the declarative, higher-order, and
gestalt requirements and found that it met them, so it sent nothing back. It
then ran its own check of the dataset's training pairs on the accepted
runtime, without reading the test pair, and all 61 checks passed:

- the file has the datasets' top-level and instance keys in their order,
  with LF line endings and no byte-order mark;
- each training pair's grids equal the originals, in order;
- every mask keeps its full canvas, 3 by 3 for inputs and 6 by 3 for
  outputs, holds only its grid's cells, and matches its x/y fields;
- every implementation passes the input box's `x`, `y`, `width`, `height`,
  and `orientation`, and every input and output snapshot reruns exactly;
- each output mask is its input mask over its row-reversed copy, and each
  output box's height is `2 * (canvas_height - y)`;
- the masks merge back into each grid, and the 242 instance names across
  the 27 JSON datasets are unique.

The worker's own offline checks, which also cover the test pair, all passed:
`document()` includes the dataset as the 15th of the 27 JSON instruction
datasets, in filename order, the rules reproduce every output, the
snapshots rerun with the rules reversed and declared after the instances,
and `bun run typecheck` and `bun run lint` pass.

The worker wrote the dataset, its README paragraph, and the puzzle's entry
in the **ARC puzzle report** workflow's list, its 24th puzzle, which leaves
the job's 170-minute timeout sufficient. The caller made the puzzle the
solver's default in `src/app.ts`, in place of `a9f96cdd`, before the run.

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
confirmed: each pattern is 0 in both states, the vertical line of the third
training pair by its axis and the others because they have no direction. Only
the comment line changed, so the class, the rules, the grids, masks,
positions, instance names, implementations, and snapshots are as before, and
the gestalt overview above still holds.

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
| 6-Nov-2026 12:40 UTC | `bun start 8be77c9e` | Accepted runtime at `4772793` (last runtime change `7096171`), local | 3 | 3 | 2m 3.0s |

The run was `bun start 8be77c9e` from `arc/`, run once by `/arc-purposal`
with a 30-minute limit and `NUCLEOID_EXECUTABLE` unset, after
`bun install --frozen-lockfile` reported no changes. It used the primary
checkout's debug runtime (`nucleoid 1.0.0`), current with `cargo build` at
`4772793`, and the committed solver at `4772793` with only its default puzzle
changed. The model settings were the committed `qwen/qwen3.8-27b`, pinned to
the `deepinfra/bf16` provider. `--only-puzzle-dataset` was off, so the
few-shot context held all 27 instruction datasets, this dataset's annotated
test pair among them, as the ARC instructions intend while the datasets are
prepared as few-shot examples.

The run was no exact match: exit status 3, difference 3, in 2m 3.0s by the
solver's own report. The runner recorded the process from 12:40:11.7 to
12:42:15.2 UTC. Its error log holds only bun's command line and the script's
exit line, and it wrote `logs/8be77c9e.log.json`, which Git ignores. The
runtime's hash was the same before and after the run, and no commit reached
the primary checkout during it. The invocation authorizes one run, so none
was repeated, and nothing was revised from its output. `bun run typecheck`
and `bun run lint` passed with the new default after the run. The puzzle
joined the **ARC puzzle report** workflow with this dataset, so no workflow
run is recorded yet.

None of these runs measured the orientation version of 6-Nov-2026: the user
chose offline verification for the batch, so its difference is unknown, not
zero.

## Open items

- **The output canvas cannot keep a trailing empty row.** This evidence
  comes from the training pairs alone. Training outputs 2 and 3 are 6 by 3,
  and their bottom row is empty. `src/visualizer.ts` crops each predicted
  output instance to its nonzero cells and pads it to the input's 3 rows, or
  down to the object's lowest cell. `Matrix.canvas` in `src/lib/Matrix.ts`
  then returns the merged result for `EXACT` and crops it for `FIT`. Composed
  that way, the second and third training outputs come out 5 rows tall, while
  the first comes out at its full 6. No dataset can close the gap, because an
  output mask holds only cells of its output, and that row has none.
  Suggested, not applied: add an `output_canvas` form whose width and height
  are Nucleoid expressions over `canvas_width` and `canvas_height`, here
  `canvas_width` and `2 * canvas_height`. The solver would evaluate them with
  the same canvas declarations and frame the merged result to exactly that
  size, in `src/app.ts`, `src/lib/Matrix.ts`, the `output_canvas` format in
  `src/analyzer.ts`, and the README's output-canvas description. `EXACT` and
  `FIT` would stay unchanged, and this dataset could adopt the new form in a
  later invocation.
- **Messages are not JSON-only.** Carried over from
  [ARC-NUC-a9f96cdd](ARC-NUC-a9f96cdd.md): `src/lib/llm.ts` sends its system
  messages as JSON, but the user and assistant messages that
  `src/analyzer.ts` and `src/visualizer.ts` build still wrap JSON in text
  labels, while the ARC instructions require every message to parse as one
  JSON document. It remains an open question for the user. This dataset does
  not depend on it.
- **Orientation version unmeasured.** No solver run has measured the
  6-Nov-2026 orientation version, so whether consistent orientation labels
  change this puzzle's result is unknown.

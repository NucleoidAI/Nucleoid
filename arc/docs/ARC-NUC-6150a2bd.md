# ARC-NUC-6150a2bd -- Half-Turn Rotation

Progress record for ARC training puzzle `6150a2bd`: its instruction dataset,
the solver and Nucleoid changes it needed, its runs, and what is still open.

| Item | Status |
| --- | --- |
| Puzzle | [`data/training/6150a2bd.json`](../data/training/6150a2bd.json) |
| Instruction dataset | [`dataset.6150a2bd.json`](../src/dataset/dataset.6150a2bd.json): the puzzle's 2 original training pairs and its test pair, one whole-block instance each (3 instances); the third version, which reads the canvas size from the solver's `canvas_width` and `canvas_height`, uncommitted, replacing the second, which `cc87228` committed, and the first, with 12 region instances, which `ed49ce3` committed |
| Output canvas | `EXACT` |
| Nucleoid enhancements | None requested; the rules use the `@first` anchors of [NUC 53](../../docs/nuc-0053.md) and [NUC 55](../../docs/nuc-0055.md) |
| Solver changes | On 4-Nov-2026, at the user's request, every `nucleoid.run` call in `src/analyzer.ts` and `src/visualizer.ts` declares `canvas_width` and `canvas_height` at the start of its program; two suggested changes await review, not applied |
| Solver runs | 1 GitHub Actions run on 2-Nov-2026 for version 1 and 1 local run on 3-Nov-2026 for version 2, both exit 0; version 3 not run |
| Updated | 5-Nov-2026 |

## Puzzle

Each training input is a 3 by 3 canvas of colored cells and zeros. The output
turns the whole input a half turn, 180 degrees, about the canvas center. The
canvas keeps its size, so the dataset is `EXACT`. In both training inputs the
nonzero cells form one edge-connected block across colors: six cells of four
colors, then four cells of three colors. The first block's bounding box fills
the canvas. The second is 3 cells wide and 2 tall, so the half turn moves it
from the top two rows to the bottom two.

## Gestalt overview

- **Whole grid.** A 3 by 3 canvas of colored and background cells.
- **Figures.** One figure per grid, the block of nonzero cells, connected
  through edges across its colors: six cells of four colors, then four cells
  of three colors.
- **Relations.** The block relates to the canvas center, which it turns
  about, so a block that does not fill the canvas changes sides: the second
  moves from the top two rows to the bottom two.
- **Change.** The whole grid turns a half turn about its center.
- **World model.** One object per pair, the whole block, turned by its
  orientation and moved by its box. Version 1 split each block into its
  single-color regions, finer than the figure, and the user's hint for
  version 2 moved the model to the whole block.

## Dataset

The dataset holds the two original training pairs in source order, followed
by the original test pair at the same level. Each pair is one instance, the
whole block of nonzero cells: `Obj(x, y, orientation, width, height)`, tagged
`l9100`, a single shape. The class has no color, parts list, canvas size, or
cell matrix, and zero cells are background. The rules read the canvas's size
from `canvas_width` and `canvas_height`, which the solver declares at the
start of every program. Unconditional class rules turn the block and move its
bounding box:

```nuc
$Obj.orientation = ($Obj.orientation@first + 180) % 360
$Obj.x = canvas_width - $Obj.x@first - $Obj.width
$Obj.y = canvas_height - $Obj.y@first - $Obj.height
```

Orientation is 0 in every input state and 180 in every output state. The
solver's model redraws each block from its input mask and that change of
orientation, then places the drawing at the output `x` and `y`. The `@first`
anchors read the block's input origin and orientation, so no rule reads the
value it writes. The 3 by 3 blocks keep their origin at 0 and 0, while the
second training block moves down one row. Like the other datasets since
`e1bdb5a`, the declarations carry `#` comments. The
[README](../README.md#instruction-datasets) describes the dataset in full.

On 3-Nov-2026 the second version replaced the first at the user's request,
with the hint "how about consider whole thing is one instance change
orientation to 180 and redraw". The training pairs confirm one instance per
pair, since all nonzero cells form one block, and the 180-degree turn. They
contradict two readings of the hint. The whole 3 by 3 canvas cannot be the
object, because its origin would stay at 0 while the second training block
moves down a row. Redrawing the turned block at its input origin fails for the
same reason. The rules therefore still move `x` and `y`, as version 1's did.
The arc-dataset worker wrote the dataset and its README paragraph. The user
committed both, with the solver's default puzzle, in `cc87228` during the run.

### Versions

| Version | Request | Instances | Rules |
| --- | --- | --- | --- |
| 1 | The puzzle's first dataset, `ed49ce3` | One per 4-connected single-color region, with `color`, `width`, `height`, `canvas_width`, and `canvas_height`: 12, with 4, 3, and 5 per pair | Orientation turned by 180 with `@first`; `x` and `y` moved to the opposite side of the canvas |
| 2 | `/arc-purposal 6150a2bd` with the hint "how about consider whole thing is one instance change orientation to 180 and redraw" | One whole block per pair, without `color`: 3 | The same three rules |
| 3 | The user's request on 4-Nov-2026 that every `nucleoid.run` call declare `canvas_width` and `canvas_height`, so that datasets need not carry them | The same blocks, without `canvas_width` and `canvas_height`: 3 | The same three rules, reading the solver's variables |

Version 3 removes `canvas_width` and `canvas_height` from the class, the three
implementations, and their snapshots, and the `x` and `y` rules read the
solver's variables instead of the properties. Regenerated with the accepted
runtime, built at `6d3acec`, from programs that start with the same canvas
declarations as the solver's `nucleoid.run` calls, every input and output
state equals version 2's without the two properties, so the
turned orientation and moved origins are unchanged. Instance names, masks,
pair grids, and the output canvas are unchanged.

### Gestalt review

On 5-Nov-2026 the dataset was checked against the gestalt requirement, which
`/arc-purposal` and the arc-dataset profile added on 4-Nov-2026, after this
dataset was written: start from the big picture, the figures each grid shows,
and go down to single cells only when that cannot express what the training
pairs show. Version 3 meets it. Each training input's nonzero cells form one
block, and the whole block is the object that turns. Version 1 split each
block into its single-color regions, finer than the figure the training pairs
show, and the user's hint for version 2 moved the model to the whole block.
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
class Obj(x: int, y: int, width: int, height: int, orientation: int):
$Obj.orientation = ($Obj.orientation@first + 180) % 360
$Obj.x = canvas_width - $Obj.x@first - $Obj.width
$Obj.y = canvas_height - $Obj.y@first - $Obj.height
```

No rule changed: the class's existing `width` and `height` moved into the
standard position, and the half turn stays in `orientation`. The migration
worker regenerated every snapshot with the accepted runtime at `491512d`, and
the caller replayed every pair offline against the unchanged grids, masks,
positions and instance names. No solver run has measured this version yet.

## Runs

| Date | Dataset version | Runtime | Exit status | Difference | Elapsed |
| --- | --- | --- | --- | --- | --- |
| 2-Nov-2026 06:45 UTC | 1, per-color regions | `ed49ce3`, GitHub Actions run [41401913542](https://github.com/NucleoidAI/Nucleoid/actions/runs/41401913542) | 0 | 0 | 157s |
| 3-Nov-2026 11:01 UTC | 2, one whole block per pair | Accepted `7709519`, local | 0 | 0 | 1m 16.3s |

The first run is the **ARC puzzle report** workflow's
`bun --no-env-file run start 6150a2bd` on `ubuntu-latest`, with the commit's
debug runtime and a 30-minute limit. Its elapsed time is the workflow's wall
time for the puzzle, from the run's `arc-puzzle-report` artifact.

The second run was `bun start 6150a2bd` from `arc/`, run once by
`/arc-purposal` with a 30-minute limit. It used the primary checkout's debug
runtime (`nucleoid 1.0.0`), rebuilt with `cargo build` at `7709519`, whose
only source change since `4ad55da` is the REPL header in `src/terminal/`. The
dataset worker had captured the snapshots with the build before it. Rerun
with the rebuilt runtime before the solver run, the worker's check script
reproduced every input and output snapshot. The runtime's hash was the same
before and after the run, and the dataset the run read is byte for byte the
file `cc87228` committed. Elapsed is the solver's own report. Version 3 has
not been run; measuring it takes a new `/arc-purposal 6150a2bd` invocation,
which authorizes its one run.

## Open items

- **Every run saw the answer.** `src/lib/llm.ts` sends `document(puzzle_id)`
  as few-shot context, and `src/dataset/dataset.ts` discards `puzzle_id`. Both
  prompts therefore held this puzzle's annotated test pair, expected output
  included, so neither exact match shows that the rule was inferred from the
  training pairs. While overwriting, the dataset worker also had version 1 at
  hand, which held the annotated test pair; it reports inferring the rules and
  the canvas mode from the training pairs. Suggested, not applied: keep the
  running puzzle's test pairs out of `document(puzzle_id)`.
- **Canvas size, resolved by version 3.** In versions 1 and 2 the model had
  to fill in `canvas_width` and `canvas_height` for the test instance,
  although `value()` in `src/visualizer.ts` sends only the cropped mask with
  `input_x` and `input_y`, not the test canvas's size. Since 4-Nov-2026
  every `nucleoid.run` call declares both at the start of its program, from
  the test input in the visualizer, and version 3 reads them from there, so
  the model no longer supplies them. This replaces the earlier suggestion to
  pass the test input's row and column counts to `visualizer.value`, which
  [ARC-NUC-d037b0a7](ARC-NUC-d037b0a7.md) also recorded.
- **Messages are not JSON-only.** `src/lib/llm.ts`, `src/analyzer.ts`, and
  `src/visualizer.ts` wrap JSON in text labels such as `few_shot:`,
  `train_dataset:`, and `input_x:`, while the ARC instructions require every
  message to parse as one JSON document. Suggested, not applied: build each
  message with `JSON.stringify` of one data object.
- **Comments in declarations.** `.claude/rules/arc.instructions.md` permits
  executable code in `declarations` but no prose comments, while version 2
  follows the `#` comments that `e1bdb5a` added to every dataset. Decide which
  wins. Removing the comments needs no new snapshots.
- Review and commit version 3, its README paragraph, the solver change in
  `src/analyzer.ts`, `src/visualizer.ts`, and `src/app.ts`, and this record's
  updates, which remain uncommitted. Since
  3-Nov-2026 only `/nuc-proposal` commits and pushes, so `/arc-purposal`
  leaves its changes for review.

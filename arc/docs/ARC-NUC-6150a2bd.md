# ARC-NUC-6150a2bd -- Half-Turn Rotation

Progress record for ARC training puzzle `6150a2bd`: its instruction dataset,
the solver and Nucleoid changes it needed, its runs, and what is still open.

| Item | Status |
| --- | --- |
| Puzzle | [`data/training/6150a2bd.json`](../data/training/6150a2bd.json) |
| Instruction dataset | [`dataset.6150a2bd.json`](../src/dataset/dataset.6150a2bd.json): the puzzle's 2 original training pairs and its test pair, one whole-block instance each (3 instances); the third version, which reads the canvas size from the solver's `canvas_width` and `canvas_height`, committed in `f771c15` and moved to the standard entries in `521aa3c`, replacing the second, which `cc87228` committed, and the first, with 12 region instances, which `ed49ce3` committed; orientation labelled by shape and the shared orientation comment added on 6-Nov-2026; descriptive `implementation` comments added on 7-Nov-2026; colors written as letters on 8-Nov-2026; respelled on 8-Nov-2026 with the `is` assignments of NUC 93 and bare words, every snapshot unchanged |
| Output canvas | `EXACT` |
| Nucleoid enhancements | None requested; the rules use the `@first` anchors of [NUC 53](../../docs/nuc-0053.md) and [NUC 55](../../docs/nuc-0055.md) |
| Solver changes | On 4-Nov-2026, at the user's request, every `nucleoid.run` call in `src/analyzer.ts` and `src/visualizer.ts` declares `canvas_width` and `canvas_height` at the start of its program; one suggested change awaits review, not applied; since 8-Nov-2026 every respelled dataset needs `is` assignments, which `src/lib/nucleoid.ts` enabled with `--is-assignment` until `d945213` made them the default |
| Solver runs | 1 GitHub Actions run on 2-Nov-2026 for version 1 and 1 local run on 3-Nov-2026 for version 2, both exit 0; version 3 not run; none of the current version; none of the 8-Nov-2026 `is` version |
| Updated | 8-Nov-2026 |

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

One of the three blocks changed: the second training block is a three-color
corner whose full top row and left column meet at the top-left vertex of its
box, so it is 315 in its input, and the half-turn rule,
`($Obj.orientation@first + 180) % 360`, makes it 135 in its output, where the
vertex lies at the bottom right. The other blocks have no direction and stay 0
in their inputs and 180 in their outputs. No rule changed.

The grids, masks, positions, instance names, tags, and every other property
are as before; only the orientation values changed, so the world model keeps
its level.

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
colors as letters ranked by count: 3 as `A`, 5 as `B`, 8 as `C`, 6 as `D`, 7
as `E`, 2 as `F`, 1 as `G`, and 4 as `H`. Mapping the letters back reproduces
the puzzle's pairs cell for cell.

The class and rules hold no color, so only the pair grids and both instance
masks changed; the declarations and implementations are unchanged.

Every snapshot was regenerated with the accepted runtime at `adc282a` and
equals its earlier snapshot byte for byte, so the rules behave as before;
instance names, positions, sizes, orientations, and tags are unchanged. `Matrix.silhouette` now also takes letters, so the
orientation call's few-shot crops read exactly as before. No solver run has
measured this version, so its difference is unknown, not zero.

### Is assignments

On 8-Nov-2026 the user asked to apply [NUC 93](../../docs/nuc-0093.md) and
[NUC 86](../../docs/nuc-0086.md) to every dataset in `src/dataset`. Every
assignment in the declarations and implementations now uses `is`, the opt-in
assignment of NUC 93, and a quoted word that was a whole value, or the whole
right operand of an `is` comparison, lost its quotes:

```nuc
class Obj(x: int, y: int, width: int, height: int, orientation: int):
    this.x is x
    this.y is y
    this.width is width
    this.height is height
    this.orientation is orientation
    this.tag is l9100

$Obj.orientation is ($Obj.orientation@first + 180) % 360

$Obj.x is canvas_width - $Obj.x@first - $Obj.width

$Obj.y is canvas_height - $Obj.y@first - $Obj.height
```

`target is value` builds exactly the syntax tree of `target = value`. A bare
word reads the name of its spelling when one exists and otherwise stores its
own text: in the constructor, `this.x is x` reads the parameter `x`, and no
program of this dataset defines `l9100`, so that word reads as its own text.
Implementations construct their instances with `is` as well, as in
`obj0c2414cf33eb is Obj(x=0, y=0, width=3, height=3, orientation=0)`. Only the
spelling changed: the class, the rules' structure, the grids, masks,
positions, instance names, and snapshots are as before, and the world model
keeps its level, so the gestalt overview above still holds.

The rules now need a runtime at `ad45ba4`, the NUC 93 commit, or later, run
with `--is-assignment`, which `src/lib/nucleoid.ts` now passes to every
program; without the flag, `obj0c2414cf33eb is Obj(...)` reads as three
juxtaposed statements and raises `ReferenceError`. Offline, with the accepted
runtime at `4ac649b` (last runtime change `ad45ba4`), the flag reproduced all
3 input and 3 output snapshots of the 3 pairs, both in a direct replay and
through the solver's own `nucleoid.run`, and the previous version reproduced
them with and without the flag. No NUC was requested, so no proposal strategy
was selected and strategy learning did not apply. No solver run has used this
version.

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

None of these runs measured the orientation version of 6-Nov-2026: the user
chose offline verification for the batch, so its difference is unknown, not
zero.

## Open items

- **Every run saw the answer.** Resolved on 6-Nov-2026: the user decided this
  is intended, because the datasets are being prepared as few-shot examples
  and runs at this stage are not yet held-out tests. The ARC instructions now
  say so, and the suggested change to keep the running puzzle's test pairs out
  of `document(puzzle_id)` is withdrawn. `src/lib/llm.ts` sends
  `document(puzzle_id)` as few-shot context, and `src/dataset/dataset.ts`
  discards `puzzle_id`. Both prompts therefore held this puzzle's annotated
  test pair, expected output included, so neither exact match shows that the
  rule was inferred from the training pairs. While overwriting, the dataset
  worker also had version 1 at hand, which held the annotated test pair; it
  reports inferring the rules and the canvas mode from the training pairs.
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
- **Comments in declarations.** Resolved on 6-Nov-2026: the user decided that
  natural-language `#` comments may reach the model only as part of
  `declarations`, and the ARC instructions now say so. Datasets with and
  without such comments both follow them, so nothing needs to change. On
  7-Nov-2026 the user also allowed them in `implementation`, and each
  implementation in this dataset now opens with comment lines that describe
  its object.
- **Committed.** `f771c15` committed version 3, its README paragraph, the
  solver change in `src/analyzer.ts`, `src/visualizer.ts`, and `src/app.ts`,
  and this record's updates.
- **Orientation version unmeasured.** No solver run has measured the
  6-Nov-2026 orientation version, so whether consistent orientation labels
  change this puzzle's result is unknown.
- **Letter version unmeasured.** No solver run has measured the 8-Nov-2026
  letter version. The solver still sends this puzzle's own grids as numbers,
  while every few-shot dataset now writes its colors as letters.
- **The `is` assignment version is unmeasured.** No solver run has used the
  8-Nov-2026 version with `is` assignments and bare words, so its effect on
  this puzzle's result is unknown. Every run's few-shot context now shows bare
  words beside quoted strings, and a word reads as its text only as a whole
  value: a model that writes one inside a larger value, such as
  `$Obj.fill_color is A if $Obj.symmetric else C`, gets a `ReferenceError`.

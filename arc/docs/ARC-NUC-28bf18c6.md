# ARC-NUC-28bf18c6 -- Repeated Shape Crop

Progress record for ARC training puzzle `28bf18c6`: its instruction dataset,
the solver and Nucleoid changes it needed, its runs, and what is still open.

| Item | Status |
| --- | --- |
| Puzzle | [`data/training/28bf18c6.json`](../data/training/28bf18c6.json) |
| Instruction dataset | [`dataset.28bf18c6.json`](../src/dataset/dataset.28bf18c6.json): the puzzle's 3 original training pairs and its test pair, 4 instances; shared orientation comment added on 6-Nov-2026; descriptive `implementation` comments added on 7-Nov-2026; colors written as letters on 8-Nov-2026; respelled on 8-Nov-2026 with the `is` assignments of NUC 93 and bare words, every snapshot unchanged; colors written as numbers again on 8-Nov-2026 |
| Output canvas | `FIT` |
| Nucleoid enhancements | None; the dataset uses accepted features only |
| Solver changes | `output_canvas`, applied by `Matrix.canvas`; since 8-Nov-2026 every respelled dataset needs `is` assignments, which `src/lib/nucleoid.ts` enabled with `--is-assignment` until `d945213` made them the default |
| Solver runs | 5 GitHub Actions runs, 1-2 Nov 2026; all exit 0; none of the current version; none of the 8-Nov-2026 `is` version |
| Updated | 8-Nov-2026 |

## Puzzle

Each training pair holds one shape on an 8 by 8 canvas. The output repeats
the shape's 3 by 3 crop twice, side by side, as a 3 by 6 grid. The output is
smaller than the input and has no empty border: it exactly fills its own
nonzero bounding box.

## Gestalt overview

- **Whole grid.** An 8 by 8 background canvas holds one small shape of one
  color that fits a 3 by 3 box; the output is a 3 by 6 strip.
- **Figures.** One figure per grid, the whole shape. Its cells touch through
  edges or corners, and no shape is its own left-right mirror.
- **Relations.** The shape relates to nothing else, and the canvas around it
  does not reach the output.
- **Change.** The shape is lifted out of its canvas and set down twice, side
  by side, the two copies filling the output.
- **World model.** One object per pair, the whole shape, which the rules
  repeat to the right and move to the origin; `FIT` drops the rest of the
  canvas.

## Dataset

The single dataset contains the three original training pairs in source
order, followed by the original test pair at the same level, preserving their
8 by 8 inputs, 3 by 6 outputs, cells, colors, and positions. Each input
instance mask retains its full 8 by 8 canvas; each output mask retains the
full 3 by 6 canvas, with zero padding outside the object. Every shape is
left-right asymmetric, so the pairs tell repeating apart from mirroring. One
`Obj(x, y, orientation, color)` class carries unconditional class rules that
hold for every instance:

```nuc
$Obj.tag = 'l4305'
$Obj.repeat_count = 2
$Obj.repeat_direction = 'right'
$Obj.x = 0
$Obj.y = 0
```

`repeat_count` and `repeat_direction` describe the repetition, and `x` and `y`
move the object to the output origin, where the solver places the repeated
crop. The [README](../README.md#instruction-datasets) describes the dataset in
full.

### Gestalt review

On 5-Nov-2026 the dataset was checked against the gestalt requirement, which
`/arc-purposal` and the arc-dataset profile added on 4-Nov-2026, after this
dataset was written: start from the big picture, the figures each grid shows,
and go down to single cells only when that cannot express what the training
pairs show. The dataset meets it. Each training input shows one shape, and
that whole shape is the object the rules repeat. Nothing needs to change.

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
$Obj.tag = 'l4305'
$Obj.repeat_count = 2
$Obj.repeat_direction = 'right'
$Obj.width = $Obj.width@first * $Obj.repeat_count
$Obj.x = 0
$Obj.y = 0
```

One rule, `$Obj.width = $Obj.width@first * $Obj.repeat_count`, gives the
repeated shape its doubled width, while its height stays. The migration worker
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
confirmed: each shape has no direction, so both states are 0. Only the comment
line changed, so the class, the rules, the grids, masks, positions, instance
names, implementations, and snapshots are as before, and the gestalt overview
above still holds.

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
colors as letters ranked by count: 8 as `A`, 2 as `B`, 3 as `C`, and 1 as `D`.
Mapping the letters back reproduces the puzzle's pairs cell for cell.

The pair grids, both instance masks, the implementations' `color` arguments,
and the snapshots use those letters, and `color` is now a `str` parameter. The
rules name no color, so they are unchanged.

Every snapshot was regenerated with the accepted runtime at `adc282a` and
equals its earlier snapshot with its colors replaced by their letters, so the
rules behave as before; instance names, positions, sizes, orientations, and
tags are unchanged. `Matrix.silhouette` now also takes letters, so the
orientation call's few-shot crops read exactly as before. No solver run has
measured this version, so its difference is unknown, not zero.

## Decision: `output_canvas` trims the composed matrix

### Problem

The solver composes its prediction on a canvas that is never smaller than the
test input:

- The result starts as a zero matrix with the test input's dimensions.
- `Matrix.pad` places each predicted output crop at the `x` and `y` of the
  object's output snapshot, treating the test input's dimensions as minimums.
- `Matrix.merge` keeps the largest height and width of its inputs.

That suits puzzles whose output keeps the input canvas, such as `0ca9ddb6` and
the genesis puzzle `3aa6fb7a`. This puzzle instead turns an 8 by 8 input into
a 3 by 6 output, so even a perfect prediction composes as the correct 3 by 6
content in the top-left corner of an 8 by 8 zero canvas. Scoring is exact grid
equality, shape included, and `Matrix.difference` counts every extra cell. On
a training pair, that perfect prediction would still score 46: 6 extra cells
beside the three content rows and 40 in the five extra rows. No run of this
puzzle could reach an exact match.

### Decision

Each dataset declares `output_canvas`, one of a closed set of framing modes.
`Matrix.canvas` applies it once to the merged result, just before comparison:

| Value | Framing |
| --- | --- |
| `EXACT` | Keep the composed canvas: the test input's dimensions, or larger where a placed object extends past them |
| `FIT` | Crop the canvas to its nonzero bounding box with `Matrix.crop` |
| Any other value | Fail explicitly |

The analyzer infers the value from the training pairs in the same response as
the declarations. This dataset and `9af7a82c` are `FIT`; `0ca9ddb6` and the
genesis datasets are `EXACT`.

### Reasoning

- **Framing belongs to the whole output, not to one object.** Only the merged
  canvas shows the extent of every placed object, so the trim runs once, at
  the end.
- **Nothing upstream changes.** Instance masks stay full-size and zero-padded,
  `x` and `y` stay canvas coordinates, and padding and merging keep their
  semantics. Trimming earlier would have switched objects to cropped
  representations, which the ARC instructions rule out without an explicit
  conversion.
- **The frame comes from training pairs, never the test output.** Sizing the
  result from the expected test grid would leak the answer. `FIT` derives the
  size from the prediction itself, and the mode is chosen from the training
  pairs, where every output fills its nonzero bounding box.
- **A closed enum keeps the exchange structured.** The model picks one of two
  identifiers instead of predicting numeric dimensions for each input, and an
  unknown value fails instead of falling back to a default.
- **Cropping cannot be unconditional.** `0ca9ddb6` and genesis outputs keep
  empty rows and columns at their edges, and cropping would break them. The
  framing is therefore a per-puzzle choice, and it lives in the dataset.
- **The world model and the frame split the work.** The rules describe the
  repetition and move the object to the origin; `FIT` removes the input-sized
  remainder. Because `FIT` crops all four sides, the result would not change
  if the object were placed away from the origin.

### Limits

- `FIT` removes every all-zero border row and column, so it cannot produce an
  expected output that keeps an empty edge.
- Neither mode produces an output that is smaller than the test input but not
  tight around its content, or larger than the test input with zero padding
  beyond the placed objects. Such puzzles need another mode, for example
  dimensions computed by the world model.
- `FIT` returns `[]` for an all-zero result.
- The value is checked only when `Matrix.canvas` runs, after every model call
  in the run, so an unknown value fails at the end rather than when the
  analyzer receives it.

### Is assignments

On 8-Nov-2026 the user asked to apply [NUC 93](../../docs/nuc-0093.md) and
[NUC 86](../../docs/nuc-0086.md) to every dataset in `src/dataset`. Every
assignment in the declarations and implementations now uses `is`, the opt-in
assignment of NUC 93, and a quoted word that was a whole value, or the whole
right operand of an `is` comparison, lost its quotes:

```nuc
class Obj(x: int, y: int, width: int, height: int, orientation: int, color: str):
    this.x is x
    this.y is y
    this.width is width
    this.height is height
    this.orientation is orientation
    this.tag is l9100
    this.color is color

$Obj.tag is l4305

$Obj.repeat_count is 2

$Obj.repeat_direction is right

$Obj.width is $Obj.width@first * $Obj.repeat_count

$Obj.x is 0

$Obj.y is 0
```

`target is value` builds exactly the syntax tree of `target = value`. A bare
word reads the name of its spelling when one exists and otherwise stores its
own text: in the constructor, `this.x is x` reads the parameter `x`, and no
program of this dataset defines any of `l9100`, `l4305`, and `right`, so those
words read as their own text. Implementations construct their instances with
`is` as well, as in
`obj9b9fe2a02df1 is Obj(x=1, y=1, width=3, height=3, orientation=0, color='A')`,
and keep their quoted keyword arguments, which are not assignments. Only the
spelling changed: the class, the rules' structure, the grids, masks,
positions, instance names, and snapshots are as before, and the world model
keeps its level, so the gestalt overview above still holds.

The rules now need a runtime at `ad45ba4`, the NUC 93 commit, or later, run
with `--is-assignment`, which `src/lib/nucleoid.ts` now passes to every
program; without the flag, `obj9b9fe2a02df1 is Obj(...)` reads as three
juxtaposed statements and raises `ReferenceError`. Offline, with the accepted
runtime at `4ac649b` (last runtime change `ad45ba4`), the flag reproduced all
4 input and 4 output snapshots of the 4 pairs, both in a direct replay and
through the solver's own `nucleoid.run`, and the previous version reproduced
them with and without the flag. No NUC was requested, so no proposal strategy
was selected and strategy learning did not apply. No solver run has used this
version.

### Number colors

On 8-Nov-2026, at the user's request, the letter colors were reverted, and the
dataset writes every color as its number again, as the puzzle's pairs hold it,
turning `A` back into 8, `B` into 2, `C` into 3, and `D` into 1. The pair
grids, both instance masks, the implementations' `color` arguments, and the
snapshots hold numbers, and `color` is an `int` parameter again. The changed
declarations now read:

```nuc
class Obj(x: int, y: int, width: int, height: int, orientation: int, color: int):
```

The `is` assignments and bare words of the previous section are unchanged.
Offline, with the accepted runtime at `1a41ca7` (last runtime change
`f463d94`), all 4 input and 4 output snapshots of the 4 pairs replay, each
equal to its letter snapshot with every letter read as its number. The pair
grids equal the puzzle's pairs cell for cell, and the grids, masks, and
snapshots are again those of the version before the letters.
`Matrix.silhouette` again takes only the colors 0 to 9, and the orientation
calls' few-shot crops read exactly as they did with letters, because the
counts that rank them are unchanged. No NUC was requested, so no proposal
strategy was selected and strategy learning did not apply. No solver run has
measured this version, so its difference is unknown, not zero.

## Runs

| Date | Run | Commit | Exit status | Difference | Seconds |
| --- | --- | --- | --- | --- | --- |
| 1-Nov-2026 12:00 UTC | [41304414238](https://github.com/NucleoidAI/Nucleoid/actions/runs/41304414238) | `3cca0aa` | 0 | 0 | 56 |
| 1-Nov-2026 14:03 UTC | [41314257591](https://github.com/NucleoidAI/Nucleoid/actions/runs/41314257591) | `3f734a8` | 0 | 0 | 87 |
| 1-Nov-2026 20:20 UTC | [41346341604](https://github.com/NucleoidAI/Nucleoid/actions/runs/41346341604) | `c2fff09` | 0 | 0 | 93 |
| 2-Nov-2026 05:26 UTC | [41394006982](https://github.com/NucleoidAI/Nucleoid/actions/runs/41394006982) | `b90a525` | 0 | 0 | 78 |
| 2-Nov-2026 06:45 UTC | [41401913542](https://github.com/NucleoidAI/Nucleoid/actions/runs/41401913542) | `ed49ce3` | 0 | 0 | 109 |

Each run is the **ARC puzzle report** workflow's
`bun --no-env-file run start 28bf18c6` on `ubuntu-latest`, with the commit's
debug runtime and a 30-minute limit. The exit status is the difference score;
seconds are the workflow's wall time for the puzzle, from each run's
`arc-puzzle-report` artifact. No local `bun start 28bf18c6` run is recorded.

None of these runs measured the orientation version of 6-Nov-2026: the user
chose offline verification for the batch, so its difference is unknown, not
zero.

## Open items

- **Runs from `c2fff09` on saw the answer.** Resolved on 6-Nov-2026: the user
  decided this is intended, because the datasets are being prepared as
  few-shot examples and runs at this stage are not yet held-out tests. The ARC
  instructions now say so, and the suggested change to keep the running
  puzzle's test pairs out of `document(puzzle_id)` is withdrawn. Since that
  commit, `src/dataset/dataset.ts` sends every JSON dataset as few-shot
  context, and this puzzle's dataset includes its annotated test pair with the
  expected output. The exact matches at `c2fff09`, `b90a525`, and `ed49ce3`
  therefore do not show that the rule was inferred from the training pairs;
  the runs at `3cca0aa` and `3f734a8`, which predate it, also matched exactly.
- **Orientation version unmeasured.** No solver run has measured the
  6-Nov-2026 orientation version, so whether consistent orientation labels
  change this puzzle's result is unknown.
- **Letter version withdrawn.** No solver run measured the 8-Nov-2026 letter
  version before the user reverted it the same day. The dataset writes its
  colors as numbers again, as the solver sends this puzzle's own grids, and
  no solver run has measured that version either, so its difference is
  unknown.
- **The `is` assignment version is unmeasured.** No solver run has used the
  8-Nov-2026 version with `is` assignments and bare words, so its effect on
  this puzzle's result is unknown. Every run's few-shot context now shows bare
  words beside quoted strings, and a word reads as its text only as a whole
  value: a model that writes one inside a larger value, such as
  `$Obj.fill_color is A if $Obj.symmetric else C`, gets a `ReferenceError`.

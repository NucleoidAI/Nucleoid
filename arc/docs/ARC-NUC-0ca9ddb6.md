# ARC-NUC-0ca9ddb6 -- Point Markers

Progress record for ARC training puzzle `0ca9ddb6`: its instruction dataset,
the solver and Nucleoid changes it needed, its runs, and what is still open.

| Item | Status |
| --- | --- |
| Puzzle | [`data/training/0ca9ddb6.json`](../data/training/0ca9ddb6.json) |
| Instruction dataset | [`dataset.0ca9ddb6.json`](../src/dataset/dataset.0ca9ddb6.json): the puzzle's 3 training pairs and its test pair, 15 point instances; conditions respelled with `is` on 6-Nov-2026; shared orientation comment added on 6-Nov-2026; descriptive `implementation` comments added on 7-Nov-2026; colors written as letters on 8-Nov-2026; respelled on 8-Nov-2026 with the `is` assignments of NUC 93 and bare words, every snapshot unchanged |
| Output canvas | `EXACT` |
| Nucleoid enhancements | [NUC 53](../../docs/nuc-0053.md) and [NUC 55](../../docs/nuc-0055.md), both Accepted; the conditions also use the accepted `is` of [NUC 86](../../docs/nuc-0086.md), which this puzzle did not request |
| Solver changes | None specific to this puzzle; since 8-Nov-2026 every respelled dataset needs `is` assignments, which `src/lib/nucleoid.ts` enabled with `--is-assignment` until `d945213` made them the default |
| Solver runs | 5 GitHub Actions runs, 1-2 Nov 2026; all exit 0; none of the current version; none of the 8-Nov-2026 `is` version |
| Updated | 8-Nov-2026 |

## Puzzle

Each training pair is a 9 by 9 canvas of single-cell points. A color-1 point
gains color-7 neighbors above, below, left, and right; a color-2 point gains
color-4 neighbors on its four diagonals; color-6 and color-8 points stay as
they are. The output keeps the input's canvas, so the dataset is `EXACT`.

## Gestalt overview

- **Whole grid.** A 9 by 9 background canvas with two to five single-cell
  points of colors 1, 2, 6, and 8 scattered over it.
- **Figures.** Each point is a whole figure on its own: no nonzero cell
  touches another through an edge or a corner.
- **Relations.** None between points. Each point's color alone decides what
  it becomes, and the markers they grow never overlap.
- **Change.** Every color-1 point grows a plus of color-7 cells on its four
  sides, every color-2 point an X of color-4 cells on its four diagonals, and
  color-6 and color-8 points stay as they are, all on the same canvas.
- **World model.** One object per point, the figure seen first. A marked
  point's output is its whole 3 by 3 marker, so the rules move its box one
  cell up and left from where the point started.

## Dataset

The single dataset preserves all three original training pairs in source
order. Every input and output instance mask retains the complete 9 by 9
canvas, with zero padding outside the object; its x/y metadata records the
nonzero bounding-box origin. Its training examples already followed
`/arc-dataset`'s original-pairs policy and needed no replacement.
Only the key ordering of their twenty snapshot fields was normalized to the
native runtime's output; grids, masks, identities, and snapshot values remain
unchanged. The original test pair follows the training pairs at the same
level, adding five point instances annotated by the same rules.

Every point is one instance of `Obj(x, y, orientation, vertex_color)`, tagged
`l7544`. Two class conditionals add the markers and move the object:

```nuc
if $Obj.vertex_color == 1:
    $Obj.tag = 'l1707'
    $Obj.marker_color = 7
    $Obj.x = $Obj.x@first - 1
    $Obj.y = $Obj.y@first - 1
```

The color-2 conditional is the same with tag `l2404` and marker color 4.
Points of other colors match neither conditional and keep their state. The
[README](../README.md#instruction-datasets) describes the dataset in full.

### Gestalt review

On 5-Nov-2026 the dataset was checked against the gestalt requirement, which
`/arc-purposal` and the arc-dataset profile added on 4-Nov-2026, after this
dataset was written: start from the big picture, the figures each grid shows,
and go down to single cells only when that cannot express what the training
pairs show. The dataset meets it. No nonzero cell of a training input touches
another through an edge or a corner, so each point is a whole figure, not a
piece of a larger shape, and the rules give each color-1 and color-2 point its
whole marker cross. Nothing needs to change.

### Standard entries

On 5-Nov-2026, at the user's request, every ARC dataset moved to the standard
object entries `x`, `y`, `width`, `height`, and `orientation`. `width` and
`height` are the column and row counts of the object's input bounding box. An
orientation change alone leaves them unchanged, and a rule changes them only
when the shape itself changes size, reading the input with `@first`. Every
implementation now passes the input box's `width` and `height` after `y`, and
the class and rules read:

```nuc
class Obj(x: int, y: int, width: int, height: int, orientation: int, vertex_color: int):
if $Obj.vertex_color == 1:
    $Obj.tag = 'l1707'
    $Obj.marker_color = 7
    $Obj.x = $Obj.x@first - 1
    $Obj.y = $Obj.y@first - 1
    $Obj.width = $Obj.width@first + 2
    $Obj.height = $Obj.height@first + 2
if $Obj.vertex_color == 2:
    $Obj.tag = 'l2404'
    $Obj.marker_color = 4
    $Obj.x = $Obj.x@first - 1
    $Obj.y = $Obj.y@first - 1
    $Obj.width = $Obj.width@first + 2
    $Obj.height = $Obj.height@first + 2
```

Both marker branches add `$Obj.width = $Obj.width@first + 2` and `$Obj.height
= $Obj.height@first + 2`, because a marked point grows from 1 by 1 to its 3 by
3 marker; color-6 and color-8 points stay 1 by 1. The migration worker
regenerated every snapshot with the accepted runtime at `491512d`, and the
caller replayed every pair offline against the unchanged grids, masks,
positions and instance names. No solver run has measured this version yet.

### Is comparisons

On 6-Nov-2026 the user asked `/arc-purposal` to apply
[NUC 86](../../docs/nuc-0086.md) to the existing datasets where it fits. Both
conditions now read with `is`, NUC 86's spelling of `==`, and the rules read:

```nuc
if $Obj.vertex_color is 1:
    $Obj.tag = 'l1707'
    $Obj.marker_color = 7
    $Obj.x = $Obj.x@first - 1
    $Obj.y = $Obj.y@first - 1
    $Obj.width = $Obj.width@first + 2
    $Obj.height = $Obj.height@first + 2
if $Obj.vertex_color is 2:
    $Obj.tag = 'l2404'
    $Obj.marker_color = 4
    $Obj.x = $Obj.x@first - 1
    $Obj.y = $Obj.y@first - 1
    $Obj.width = $Obj.width@first + 2
    $Obj.height = $Obj.height@first + 2
```

`is` compares exactly as `==` does, with the same loose equality and
precedence, and an `if` header is one of the enclosed positions where NUC 86
reads it as an operator. The right operands are the numbers 1 and 2, not bare
words, so each condition is the same standing rule as before. Only the two
header lines changed. The class, the branch bodies, the grids, masks,
positions, instance names, implementations, and snapshots are as before, and
the world model keeps its level, so the gestalt overview above still holds.
Replaying the rules now needs a runtime at `7096171`, the NUC 86 commit, or
later. No NUC was requested, so no proposal strategy was selected and
strategy learning did not apply.

The arc-dataset worker respelled the headers in an external staging
directory and regenerated all 30 input and output snapshots with the accepted
runtime at `34d0b19`, which includes NUC 86; each is byte-identical to the
previous version's. The caller's own offline check passed all 43 of its checks
on the staged file and again after integration: the file equals the previous
one with only `==` respelled, its pairs are unchanged, no `==` or `!=` is
left in its declarations, every snapshot replays on that runtime, and the
instance names stay unique across all datasets. `document()` loads it with
the other 28 datasets. At the user's choice, no solver run has measured this
version.

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
confirmed: each point is 0, and so is its output, the point with its ring of
markers. Only the comment line changed, so the class, the rules, the grids,
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
colors as letters ranked by count: 4 as `A`, 7 as `B`, 2 as `C`, 1 as `D`, 8
as `E`, and 6 as `F`. Mapping the letters back reproduces the puzzle's pairs
cell for cell.

The pair grids, both instance masks, the implementations, and the snapshots
use those letters, and the class and rules name the same colors by letter.
Only the color literals changed, and the changed declarations now read:

```nuc
class Obj(x: int, y: int, width: int, height: int, orientation: int, vertex_color: str):
if $Obj.vertex_color is 'D':
    $Obj.tag = 'l1707'
    $Obj.marker_color = 'B'
    $Obj.x = $Obj.x@first - 1
    $Obj.y = $Obj.y@first - 1
    $Obj.width = $Obj.width@first + 2
    $Obj.height = $Obj.height@first + 2
if $Obj.vertex_color is 'C':
    $Obj.tag = 'l2404'
    $Obj.marker_color = 'A'
    $Obj.x = $Obj.x@first - 1
    $Obj.y = $Obj.y@first - 1
    $Obj.width = $Obj.width@first + 2
    $Obj.height = $Obj.height@first + 2
```

Every snapshot was regenerated with the accepted runtime at `adc282a` and
equals its earlier snapshot with its colors replaced by their letters, so the
rules behave as before; instance names, positions, sizes, orientations, and
tags are unchanged. `Matrix.silhouette` now also takes letters, so the
orientation call's few-shot crops read exactly as before. No solver run has
measured this version, so its difference is unknown, not zero.

## Decision: locate the delta from the initial position (NUC 53 and NUC 55)

### Why the world model has to move the object

The solver places each predicted output crop at the `x` and `y` of the
object's output snapshot, the state the Nucleoid rules compute. A color-1 or
color-2 point grows into a 3 by 3 pattern centered on itself, so its output
crop starts one column left of and one row above the point:

```text
output_x = input_x - 1
output_y = input_y - 1
```

The first version of the dataset set only `tag` and `marker_color`. Its output
snapshots kept the point's own coordinates, and only the `output_x` and
`output_y` annotations recorded the shift. The world model therefore did not
explain where the pattern goes, and a crop placed at the snapshot's
coordinates lands one cell right of and below its place.

### Why accepted Nucleoid could not express it

The direct rule reads the property it writes:

```nuc
$Obj.x = $Obj.x - 1
```

Nucleoid rejects it with `TypeError: Circular Dependency`, and as a reactive
rule it would subtract again each time it re-evaluated. Without a history, the
original position would have to be copied into an extra field, such as
`this.x0 = x`, only to compute from it: bookkeeping that is not a property of
the object.

### What the NUCs add

| NUC | Contribution |
| --- | --- |
| [53](../../docs/nuc-0053.md) -- Read-Only State Version Access | Every variable and object-property slot keeps its values. `@first`, the same as `@0`, reads a slot's first value: here, the position the constructor stored. |
| [55](../../docs/nuc-0055.md) -- Class-Level State Version Anchors | A class rule or conditional may assign a property from its own fixed anchor, `@first` or `@n`. Each instance anchors independently, the self-anchor is exempt from the class-cycle check, re-evaluation never accumulates, and a missing anchor waits instead of seeding a placeholder. |

Together they let the rules state the delta from the initial position
directly:

```nuc
$Obj.x = $Obj.x@first - 1
$Obj.y = $Obj.y@first - 1
```

All ten output snapshots now agree with their `output_x` and `output_y`
annotations, so the world model itself carries the position change that
placement relies on. The genesis datasets `dataset.curve.json` and
`dataset.line.json` use the same anchors.

### Offline check

On 29-Oct-2026, the local debug runtime (`nucleoid 1.0.0`) confirmed this
without model calls:

- `Obj(x=2, y=3, orientation=0, vertex_color=2)` ends at `x = 1` and `y = 2`,
  with tag `l2404` and marker color 4.
- Turning its guard off and on again leaves `x = 1` and `y = 2`, while
  `x@first` and `y@first` stay 2 and 3.
- `$Obj.x = $Obj.x - 1` fails with `TypeError: Circular Dependency`.

### Is assignments

On 8-Nov-2026 the user asked to apply [NUC 93](../../docs/nuc-0093.md) and
[NUC 86](../../docs/nuc-0086.md) to every dataset in `src/dataset`. Every
assignment in the declarations and implementations now uses `is`, the opt-in
assignment of NUC 93, and a quoted word that was a whole value, or the whole
right operand of an `is` comparison, lost its quotes:

```nuc
class Obj(x: int, y: int, width: int, height: int, orientation: int, vertex_color: str):
    this.x is x
    this.y is y
    this.width is width
    this.height is height
    this.orientation is orientation
    this.tag is l7544
    this.vertex_color is vertex_color

if $Obj.vertex_color is D:
    $Obj.tag is l1707
    $Obj.marker_color is B
    $Obj.x is $Obj.x@first - 1
    $Obj.y is $Obj.y@first - 1
    $Obj.width is $Obj.width@first + 2
    $Obj.height is $Obj.height@first + 2

if $Obj.vertex_color is C:
    $Obj.tag is l2404
    $Obj.marker_color is A
    $Obj.x is $Obj.x@first - 1
    $Obj.y is $Obj.y@first - 1
    $Obj.width is $Obj.width@first + 2
    $Obj.height is $Obj.height@first + 2
```

`target is value` builds exactly the syntax tree of `target = value`. A bare
word reads the name of its spelling when one exists and otherwise stores its
own text: in the constructor, `this.x is x` reads the parameter `x`, and no
program of this dataset defines any of `l7544`, `D`, `l1707`, `B`, `C`,
`l2404`, and `A`, so those words read as their own text. The conditions
`if $Obj.vertex_color is D:` and `if $Obj.vertex_color is C:` now compare with
a bare word by NUC 86's word rule, as the quoted letter did. Implementations
construct their instances with `is` as well, as in
`obj08c8feaa60ea is Obj(x=2, y=3, width=1, height=1, orientation=0, vertex_color='C')`,
and keep their quoted keyword arguments, which are not assignments. Only the
spelling changed: the class, the rules' structure, the grids, masks,
positions, instance names, and snapshots are as before, and the world model
keeps its level, so the gestalt overview above still holds.

The rules now need a runtime at `ad45ba4`, the NUC 93 commit, or later, run
with `--is-assignment`, which `src/lib/nucleoid.ts` now passes to every
program; without the flag, `obj08c8feaa60ea is Obj(...)` reads as three
juxtaposed statements and raises `ReferenceError`. Offline, with the accepted
runtime at `4ac649b` (last runtime change `ad45ba4`), the flag reproduced all
15 input and 15 output snapshots of the 4 pairs, both in a direct replay and
through the solver's own `nucleoid.run`, and the previous version reproduced
them with and without the flag. No NUC was requested, so no proposal strategy
was selected and strategy learning did not apply. No solver run has used this
version.

## Runs

| Date | Run | Commit | Exit status | Difference | Seconds |
| --- | --- | --- | --- | --- | --- |
| 1-Nov-2026 12:00 UTC | [41304414238](https://github.com/NucleoidAI/Nucleoid/actions/runs/41304414238) | `3cca0aa` | 0 | 0 | 280 |
| 1-Nov-2026 14:03 UTC | [41314257591](https://github.com/NucleoidAI/Nucleoid/actions/runs/41314257591) | `3f734a8` | 0 | 0 | 224 |
| 1-Nov-2026 20:20 UTC | [41346341604](https://github.com/NucleoidAI/Nucleoid/actions/runs/41346341604) | `c2fff09` | 0 | 0 | 355 |
| 2-Nov-2026 05:26 UTC | [41394006982](https://github.com/NucleoidAI/Nucleoid/actions/runs/41394006982) | `b90a525` | 0 | 0 | 213 |
| 2-Nov-2026 06:45 UTC | [41401913542](https://github.com/NucleoidAI/Nucleoid/actions/runs/41401913542) | `ed49ce3` | 0 | 0 | 397 |

Each run is the **ARC puzzle report** workflow's
`bun --no-env-file run start 0ca9ddb6` on `ubuntu-latest`, with the commit's
debug runtime and a 30-minute limit. The exit status is the difference score;
seconds are the workflow's wall time for the puzzle, from each run's
`arc-puzzle-report` artifact. No local `bun start 0ca9ddb6` run is recorded.

None of these runs measured the current version: the standard entries of
5-Nov-2026 and the `is` conditions of 6-Nov-2026 came later, and the user
chose offline verification for the respelling. Its difference is unknown, not
zero.

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
- **The `is` version is unmeasured.** No solver run, local or in the
  **ARC Benchmark** workflow, has used the version of 6-Nov-2026. Its snapshots
  are unchanged, but every run's few-shot context now shows `is` in
  conditions. NUC 86 reads `is` as an operator only in enclosed positions, so a
  model that copies it straight into an assignment value, such as
  `$Obj.flag = $Obj.color is 5`, gets `is` read as a name and a
  `ReferenceError`. A dispatched **ARC Benchmark** run would measure the
  effect.
- **Repeated branch bodies.** Noted by the arc-dataset worker and not
  applied, because it is a redesign rather than the respelling the user asked
  for: both branches repeat the four lines that shift the origin and grow the
  point to 3 by 3, which a later model could state once.
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

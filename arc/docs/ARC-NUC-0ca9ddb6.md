# ARC-NUC-0ca9ddb6 -- Point Markers

Progress record for ARC training puzzle `0ca9ddb6`: its instruction dataset,
the solver and Nucleoid changes it needed, its runs, and what is still open.

| Item | Status |
| --- | --- |
| Puzzle | [`data/training/0ca9ddb6.json`](../data/training/0ca9ddb6.json) |
| Instruction dataset | [`dataset.0ca9ddb6.json`](../src/dataset/dataset.0ca9ddb6.json): the puzzle's 3 training pairs and its test pair, 15 point instances |
| Output canvas | `EXACT` |
| Nucleoid enhancements | [NUC 53](../../docs/nuc-0053.md) and [NUC 55](../../docs/nuc-0055.md), both Accepted |
| Solver changes | None specific to this puzzle |
| Solver runs | 5 GitHub Actions runs, 1-2 Nov 2026; all exit 0 |
| Updated | 5-Nov-2026 |

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

## Open items

- **Runs from `c2fff09` on saw the answer.** Since that commit,
  `src/dataset/dataset.ts` sends every JSON dataset as few-shot context, and
  this puzzle's dataset includes its annotated test pair with the expected
  output. The exact matches at `c2fff09`, `b90a525`, and `ed49ce3` therefore
  do not show that the rule was inferred from the training pairs; the runs at
  `3cca0aa` and `3f734a8`, which predate it, also matched exactly. Suggested,
  not applied: keep the running puzzle's test pairs out of
  `document(puzzle_id)`.

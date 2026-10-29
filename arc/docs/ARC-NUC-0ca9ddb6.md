# ARC-NUC-0ca9ddb6 -- Point Markers

Progress record for ARC training puzzle `0ca9ddb6`: its instruction dataset,
the solver and Nucleoid changes it needed, its runs, and what is still open.

| Item | Status |
| --- | --- |
| Puzzle | [`data/training/0ca9ddb6.json`](../data/training/0ca9ddb6.json) |
| Instruction dataset | [`dataset.0ca9ddb6.json`](../src/dataset/dataset.0ca9ddb6.json): the puzzle's 3 training pairs, 10 point instances |
| Output canvas | `EXACT` |
| Nucleoid enhancements | [NUC 53](../../docs/nuc-0053.md) and [NUC 55](../../docs/nuc-0055.md), both Accepted |
| Solver changes | None specific to this puzzle |
| Solver runs | None recorded |
| Updated | 29-Oct-2026 |

## Puzzle

Each training pair is a 9 by 9 canvas of single-cell points. A color-1 point
gains color-7 neighbors above, below, left, and right; a color-2 point gains
color-4 neighbors on its four diagonals; color-6 and color-8 points stay as
they are. The output keeps the input's canvas, so the dataset is `EXACT`.

## Dataset

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
placement relies on. The genesis datasets `dataset.curve.1.json` and
`dataset.line.1.json` use the same anchors; see
[ARC-NUC-3aa6fb7a](ARC-NUC-3aa6fb7a.md).

### Offline check

On 29-Oct-2026, the local debug runtime (`nucleoid 1.0.0`) confirmed this
without model calls:

- `Obj(x=2, y=3, orientation=0, vertex_color=2)` ends at `x = 1` and `y = 2`,
  with tag `l2404` and marker color 4.
- Turning its guard off and on again leaves `x = 1` and `y = 2`, while
  `x@first` and `y@first` stay 2 and 3.
- `$Obj.x = $Obj.x - 1` fails with `TypeError: Circular Dependency`.

## Runs

No `bun start 0ca9ddb6` run is recorded yet; its difference is unknown, not
zero.

## Open items

- The README paragraph for this dataset still says native snapshot
  coordinates stay anchored to the input point; the output snapshots now carry
  the shifted origin.
- The pairs are the puzzle's own training grids. Datasets written by the
  `arc-dataset` agent now synthesize their pairs instead, as `28bf18c6` does;
  replacing these would take an explicit overwrite through `/arc-dataset`.
- Record the first run's date, runtime, exit status, and difference.

# ARC-NUC-3aa6fb7a -- Genesis Corner Markers

Progress record for the genesis ARC training puzzle `3aa6fb7a`: its
instruction datasets, the solver and Nucleoid changes they needed, its runs,
and what is still open.

| Item | Status |
| --- | --- |
| Puzzle | [`data/training/3aa6fb7a.json`](../data/training/3aa6fb7a.json) |
| Instruction datasets | [`dataset.core.1.json`](../src/dataset/dataset.core.1.json), [`dataset.core.2.json`](../src/dataset/dataset.core.2.json), [`dataset.curve.1.json`](../src/dataset/dataset.curve.1.json), and [`dataset.line.1.json`](../src/dataset/dataset.line.1.json), sent in that order |
| Output canvas | `EXACT` in all four |
| Nucleoid enhancements | [NUC 53](../../docs/nuc-0053.md) and [NUC 55](../../docs/nuc-0055.md), used by the curve and line anchors |
| Solver changes | None specific to this puzzle |
| Solver runs | None recorded |
| Updated | 29-Oct-2026 |

This puzzle has no `dataset.3aa6fb7a.json`. As the genesis puzzle, it learns
from the shared core, curve, and line datasets, so this one file tracks all
four.

## Puzzle

Each training pair is a 7 by 7 canvas of color-8 corners, each three cells of
a 2 by 2 square. The output marks every corner's missing cell with color 1.
The marker stays inside the corner's bounding box and the canvas keeps its
size, so the datasets are `EXACT`.

## Datasets

| Dataset | Pairs | Instances | Canvases | Rules |
| --- | ---: | ---: | --- | --- |
| `core.1` | 3 | 9 | 8 by 8 | Color-9 corners take tag `l6624` and a color-4 marker outside the corner, which grows the crop |
| `core.2` | 3 | 9 | 6 by 9, 10 by 7, 9 by 10 | Color-4 corners take tag `l6624` and a color-7 marker inside the corner |
| `curve.1` | 3 | 7 | 6 by 6 | Vertex color 4 takes tag `l6840` and rim color 8, anchored one column left and one row up; vertex color 9 takes tag `l9906` and rim color 2, anchored one column left |
| `line.1` | 3 | 7 | 8 by 8 | Lines extend away from their color-8 endpoint (`l4102` horizontal, `l6012` vertical), anchored one column left or one row up when extending left or up |

The [README](../README.md#analysis) covers their properties, orientations, and
origins.

## Decision: anchor curve and line shifts to the initial position

As in [ARC-NUC-0ca9ddb6](ARC-NUC-0ca9ddb6.md), the solver places each
predicted output crop at the `x` and `y` of the object's output snapshot. The
curve and line rules therefore compute their crop's shifted origin from each
object's first stored position with NUC 53 and NUC 55 anchors, for example:

```nuc
if $Obj.orientation == 0 and $Obj.endpoint_position == 'down':
    $Obj.tag = 'l6012'
    $Obj.extension_direction = 'up'
    $Obj.y = $Obj.y@first - 1
```

Every curve and line output snapshot agrees with its `output_x` and
`output_y` annotations.

## Runs

No `bun start 3aa6fb7a` run is recorded yet; its difference is unknown, not
zero.

## Open items

- `core.1` records its outside marker's shift only in the `output_x` and
  `output_y` annotations: 6 of its 9 output snapshots keep the input origin,
  so those examples disagree with placement at snapshot coordinates. The
  genesis puzzle's own markers stay inside each corner's bounding box, so its
  outputs need no shift.
- The README still says both native snapshots keep `x` and `y` at the input
  bounding box; that now holds only for the core datasets.
- Record the first run's date, runtime, exit status, and difference.

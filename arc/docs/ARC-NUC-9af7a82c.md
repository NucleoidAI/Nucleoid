# ARC-NUC-9af7a82c -- Region Size Histogram

Progress record for ARC training puzzle `9af7a82c`: its instruction dataset,
the solver and Nucleoid changes it needed, its runs, and what is still open.

| Item | Status |
| --- | --- |
| Puzzle | [`data/training/9af7a82c.json`](../data/training/9af7a82c.json) |
| Instruction dataset | [`dataset.9af7a82c.json`](../src/dataset/dataset.9af7a82c.json): 4 synthesized pairs, 15 region instances; not yet committed |
| Output canvas | `FIT` |
| Nucleoid enhancements | None recorded |
| Solver changes | None specific to this puzzle |
| Solver runs | None recorded |
| Updated | 29-Oct-2026 |

## Puzzle

Each training input is a small, fully colored canvas (3 by 3, 3 by 4, or 4 by
3) split into one region per color. The output is a histogram: the region with
the r-th largest cell count becomes a vertical bar of its color in column r,
starting at the top row and as tall as the region's cell count. Outputs can be
taller than their inputs, such as 5 by 3 from 3 by 3. Every bar starts at the
top and the tallest spans every row, so each output fills its nonzero bounding
box and the dataset is `FIT`; see
[ARC-NUC-28bf18c6](ARC-NUC-28bf18c6.md#decision-output_canvas-trims-the-composed-matrix).

## Dataset

The working-tree file has four synthesized pairs (5 by 4 to 8 by 3, 5 by 3 to
8 by 4, 2 by 5 to 7 by 3, and 6 by 3 to 6 by 5) with 3, 4, 3, and 5 regions.
Each region is an `Obj(x, y, orientation, color, cells)`, where `cells` is its
cropped mask, and unconditional class rules derive the bar:

```nuc
$Obj.tag = 'l6953'
$Obj.cell_count = $Obj.cells.flatMap(row => row).filter(cell => cell != 0).length
$Obj.bar_height = $Obj.cell_count
$Obj.x = Obj.filter(other => other.cell_count > $Obj.cell_count).length
$Obj.y = 0
```

`x` is the region's rank: the number of other regions with a larger cell count.

## Runs

No `bun start 9af7a82c` run is recorded yet; its difference is unknown, not
zero.

## Open items

- **The rank needs every region in one program.** The recorded output `x`
  values come out only when all of a pair's instances run together. The solver
  runs each object in its own Nucleoid process, where `Obj` holds just that
  object, so every rank is 0 and every bar lands in column 0. On 29-Oct-2026,
  per-object runs with the local debug runtime turned the first pair's
  recorded ranks 1, 2, and 0 into 0, 0, and 0. Either the world model or the
  per-object execution has to change for the ranks to reach placement.
- **The README describes an earlier version.** Its paragraph lists nineteen
  instances on 2 by 8, 6 by 3, 5 by 8, and 8 by 4 canvases, and a `size_rank`
  constructor property chosen because objects run in separate processes. The
  working-tree file computes `cell_count` from `cells` and ranks by comparing
  instances. Synchronize the two once the approach is settled.
- **The dataset file is not committed.**
- Record the first run's date, runtime, exit status, and difference.

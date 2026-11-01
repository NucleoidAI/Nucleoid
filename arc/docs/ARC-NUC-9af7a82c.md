# ARC-NUC-9af7a82c -- Region Size Histogram

Progress record for ARC training puzzle `9af7a82c`: its instruction dataset,
the solver and Nucleoid changes it needed, its runs, and what is still open.

| Item | Status |
| --- | --- |
| Puzzle | [`data/training/9af7a82c.json`](../data/training/9af7a82c.json) |
| Instruction dataset | [`dataset.9af7a82c.json`](../src/dataset/dataset.9af7a82c.json): 4 synthesized pairs, 18 region instances; full-canvas zero-padded masks |
| Output canvas | `FIT` |
| Nucleoid enhancements | None recorded |
| Solver changes | None specific to this puzzle |
| Solver runs | None recorded |
| Updated | 1-Nov-2026 |

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

The dataset has four synthesized pairs (3 by 6 to 9 by 3, 7 by 3 to 7 by 5,
4 by 7 to 11 by 6, and 6 by 4 to 8 by 4) with 3, 5, 6, and 4 regions.
Each region is an `Obj(x, y, orientation, color, cell_count)`, where
`cell_count` is the number of cells in its full-size, zero-padded input mask.
Unconditional class rules derive the bar:

```nuc
$Obj.tag = 'l6953'
$Obj.bar_height = $Obj.cell_count
$Obj.x = Obj.filter(other => other.cell_count > $Obj.cell_count).length
$Obj.y = 0
```

`x` is the region's rank: the number of other regions with a larger cell count.

Each `input_instance_matrix` matches its pair's complete `input_matrix`
dimensions, and each `output_instance_matrix` matches its pair's complete
`output_matrix` dimensions. Both masks retain the object's cells at their
canvas coordinates and contain `0` everywhere else; neither mask is stored as
a crop. The output bar occupies `bar_height` cells in column `output_x`,
starting at row `output_y`. These origins equal `output_value.x` and
`output_value.y` and describe positions already present within the full mask.
Merging the full output masks directly reconstructs each unchanged
`output_matrix`. Input masks, sampled grids, instance identities, rules,
implementations, and runtime snapshots are unchanged by this padding repair.
No TypeScript change or live solver run accompanies it.

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
- **The dataset file is not committed.**
- Record the first run's date, runtime, exit status, and difference.

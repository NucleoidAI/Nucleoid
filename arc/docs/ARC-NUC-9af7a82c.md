# ARC-NUC-9af7a82c -- Region Size Histogram

Progress record for ARC training puzzle `9af7a82c`: its instruction dataset,
the solver and Nucleoid changes it needed, its runs, and what is still open.

| Item | Status |
| --- | --- |
| Puzzle | [`data/training/9af7a82c.json`](../data/training/9af7a82c.json) |
| Instruction dataset | [`dataset.9af7a82c.json`](../src/dataset/dataset.9af7a82c.json): the puzzle's 4 original training pairs and its test pair, 19 region instances; full-canvas zero-padded masks |
| Output canvas | `FIT` |
| Nucleoid enhancements | None recorded |
| Solver changes | None specific to this puzzle |
| Solver runs | 5 GitHub Actions runs, 1-2 Nov 2026: differences of 10 and 5, then three exact matches |
| Updated | 2-Nov-2026 |

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

The single dataset has the four original training pairs in source order:
3 by 3 to 5 by 3, 3 by 4 to 6 by 4, 4 by 3 to 5 by 4, and 4 by 3 to
6 by 3, with 3, 4, 4, and 3 regions. The original test pair follows them at
the same level: 4 by 4 to 6 by 5, with 5 regions. All source cells, colors,
positions, and canvas dimensions are preserved.
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
Merging the full output masks directly reconstructs each original
`output_matrix`. Native input snapshots execute the constructor and initial
class declaration; native output snapshots execute all declarations and
implementations of the same pair together, followed by `Obj[:]`. Instances
are listed largest first, and each recorded rank sees the complete pair's
population. The original-training replacement changes no TypeScript and
includes no live solver run.

## Runs

| Date | Run | Commit | Exit status | Difference | Seconds |
| --- | --- | --- | --- | --- | --- |
| 1-Nov-2026 12:00 UTC | [41304414238](https://github.com/NucleoidAI/Nucleoid/actions/runs/41304414238) | `3cca0aa` | 10 | 10 | 178 |
| 1-Nov-2026 14:03 UTC | [41314257591](https://github.com/NucleoidAI/Nucleoid/actions/runs/41314257591) | `3f734a8` | 5 | 5 | 279 |
| 1-Nov-2026 20:20 UTC | [41346341604](https://github.com/NucleoidAI/Nucleoid/actions/runs/41346341604) | `c2fff09` | 0 | 0 | 276 |
| 2-Nov-2026 05:26 UTC | [41394006982](https://github.com/NucleoidAI/Nucleoid/actions/runs/41394006982) | `b90a525` | 0 | 0 | 178 |
| 2-Nov-2026 06:45 UTC | [41401913542](https://github.com/NucleoidAI/Nucleoid/actions/runs/41401913542) | `ed49ce3` | 0 | 0 | 304 |

Each run is the **ARC puzzle report** workflow's
`bun --no-env-file run start 9af7a82c` on `ubuntu-latest`, with the commit's
debug runtime and a 30-minute limit. The exit status is the difference score;
seconds are the workflow's wall time for the puzzle, from each run's
`arc-puzzle-report` artifact. Both non-zero exits were mismatches with those
differences, not failed runs. No local `bun start 9af7a82c` run is recorded.

## Open items

- **The rank needs every region in one program.** Isolated native execution
  cannot reproduce population-dependent ranks. On 29-Oct-2026, with the
  earlier examples, per-object runs turned recorded ranks 1, 2, and 0 into
  0, 0, and 0. The current annotations use complete-pair output execution;
  this offline evidence is not a live solver result.
- **Runs from `c2fff09` on saw the answer.** Since that commit,
  `src/dataset/dataset.ts` sends every JSON dataset as few-shot context, and
  this puzzle's dataset includes its annotated test pair with the expected
  output. The runs at `3cca0aa` and `3f734a8`, which predate it, scored
  differences of 10 and 5; the three runs since matched exactly, which does
  not show that the rule was inferred from the training pairs. Suggested, not
  applied: keep the running puzzle's test pairs out of `document(puzzle_id)`.

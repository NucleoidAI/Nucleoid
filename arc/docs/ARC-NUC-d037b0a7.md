# ARC-NUC-d037b0a7 -- Downward Bars

Progress record for ARC training puzzle `d037b0a7`: its instruction dataset,
the solver and Nucleoid changes it needed, its runs, and what is still open.

| Item | Status |
| --- | --- |
| Puzzle | [`data/training/d037b0a7.json`](../data/training/d037b0a7.json) |
| Instruction dataset | [`dataset.d037b0a7.json`](../src/dataset/dataset.d037b0a7.json): the puzzle's 3 original training pairs and its test pair, 11 point instances (3, 3, 2, and 3 per pair); uncommitted |
| Output canvas | `EXACT` |
| Nucleoid enhancements | None requested |
| Solver changes | None; three suggested changes await review, not applied |
| Solver runs | 1 local run on 3-Nov-2026, exit 0 |
| Updated | 3-Nov-2026 |

## Puzzle

Each training input is a 3 by 3 canvas with two or three single-cell points
of different colors, no two in the same column. The output keeps the canvas
and extends every point straight down, in its own color, to the bottom row. A
point on the top row becomes a three-cell bar, a point on the middle row a
two-cell bar, and a point on the bottom row stays a single cell. The bottom
comes from the canvas, not from the other points: in the second and third
training pairs the lowest point is on the middle row, yet the bars reach the
bottom row. The third training output keeps an empty right column, which
`FIT` would crop, so the dataset is `EXACT`.

## Dataset

The dataset holds the three original training pairs in source order, followed
by the original test pair at the same level. Every nonzero input cell is one
instance of `Obj(x, y, orientation, color, canvas_height)`, tagged `l7544`, a
single point, as in `dataset.0ca9ddb6.json`. `canvas_height` is the height of
the object's canvas, a scalar as in `dataset.6150a2bd.json`; no object
carries a cell matrix. Two unconditional class-level rules turn every point
into a bar:

```nuc
$Obj.tag = 'l6953'
$Obj.bar_height = $Obj.canvas_height - $Obj.y
```

`l6953` is a vertical bar, as in `dataset.9af7a82c.json`. `bar_height` is
absent from the input states. `x` and `y` keep the point's position, which is
the bar's top cell, and orientation is 0 in both states. Each bar depends only
on its own point and the canvas, so no rule relates objects to each other.
Like the other ten datasets since `e1bdb5a`, the declarations carry `#`
comments; the native snapshots are the same without them. The
[README](../README.md#instruction-datasets) describes the dataset in full.

The arc-dataset worker fixed the rules from the training pairs before reading
the test pair, and the three training outputs already matched by then. The
same rules reproduce the test pair. `arc/README.md` already held a paragraph
for this dataset from `feef5f8`, although no dataset file was ever committed.
That paragraph already described this model, so the worker kept it and added
only that the rules also set the output tag. It also added `d037b0a7` to the
**ARC puzzle report** workflow's `ARC_PUZZLES` list.

## Runs

| Date | Dataset version | Runtime | Exit status | Difference | Elapsed |
| --- | --- | --- | --- | --- | --- |
| 3-Nov-2026 06:15 UTC | 1, one point per nonzero cell | Accepted `e1bdb5a` | 0 | 0 | 3m 3.8s |

The run was `bun start d037b0a7` from `arc/`, run once by `/arc-purposal` with
a 30-minute limit. It used the primary checkout's debug runtime
(`nucleoid 1.0.0`), the same binary that produced the dataset's snapshots.
`cargo build` could not replace that binary, because an open `nucleoid.exe`
REPL locked it. It had been built after the last runtime commit, `4c57af8`
(NUC 75), and since that build only uncommitted REPL edits in `src/terminal/`
and an uncommitted `rand` dependency had changed, so it ran the accepted
runtime. Its hash was the same before and after the run. Elapsed is the
solver's own report. The puzzle has not been in an **ARC puzzle report**
workflow run.

## Open items

- **The run saw the answer.** `src/lib/llm.ts` sends `document(puzzle_id)` as
  few-shot context, and `src/dataset/dataset.ts` discards `puzzle_id`. The
  prompt therefore held this puzzle's annotated test pair, expected output
  included, and the exact match does not show that the rule was inferred from
  the training pairs. Suggested, not applied: keep the running puzzle's test
  pairs out of `document(puzzle_id)`.
- **The test canvas's height is not in the instance prompt.**
  `value()` in `src/visualizer.ts` sends the test instance's mask cropped by
  `Matrix.crop`, with only `input_x` and `input_y`, and crops the training
  masks as well. Nothing in that message gives the height of the test canvas,
  which the model needs to fill in `canvas_height`. The model can only carry
  the value over from the training implementations, where it is always 3, or
  from the few-shot datasets. Suggested, not applied: pass the test input's
  row and column counts to `visualizer.value`. `dataset.6150a2bd.json`'s
  `canvas_width` and `canvas_height` have the same need.
- **Messages are not JSON-only.** `src/lib/llm.ts`, `src/analyzer.ts`, and
  `src/visualizer.ts` wrap JSON in text labels such as `few_shot:`,
  `train_dataset:`, and `input_x:`, while the ARC instructions require every
  message to parse as one JSON document. Suggested, not applied: build each
  message with `JSON.stringify` of one data object.
- **Comments in declarations.** `.claude/rules/arc.instructions.md` permits
  executable code in `declarations` but no prose comments, while `e1bdb5a`
  added `#` comments to every other dataset and this one follows them. Decide
  which wins. Removing the comments needs no new snapshots.
- **Stale README text.** `arc/README.md` still describes
  `dataset.0d3d703e.json`, `dataset.27a28665.json`, and
  `dataset.44f52bb0.json`, which do not exist. Its GitHub Actions paragraph
  names four puzzles, while the workflow now lists eight.
- Review and commit the dataset, its README paragraph, the workflow entry, the
  solver's default puzzle in `src/app.ts`, and this record, which remain
  uncommitted.

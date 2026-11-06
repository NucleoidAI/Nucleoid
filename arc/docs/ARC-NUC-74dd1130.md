# ARC-NUC-74dd1130 -- Main-Diagonal Mirror

Progress record for ARC training puzzle `74dd1130`: its instruction dataset,
the solver and Nucleoid changes it needed, its runs, and what is still open.

| Item | Status |
| --- | --- |
| Puzzle | [`data/training/74dd1130.json`](../data/training/74dd1130.json) |
| Instruction dataset | [`dataset.74dd1130.json`](../src/dataset/dataset.74dd1130.json): the puzzle's 4 original training pairs and its test pair, 5 mosaic instances, one per pair; committed with this record |
| Output canvas | `EXACT` |
| Nucleoid enhancements | None requested, so no proposal strategy was selected and strategy learning did not apply; the rule is one unconditional class-level rule of accepted Nucleoid |
| Solver changes | None; suggested changes await review, not applied |
| Solver runs | 1 local run on 5-Nov-2026, exit 0, an exact match, with this dataset's test pair in its few-shot context |
| Updated | 6-Nov-2026 |

## Puzzle

Each training input is a 3 by 3 canvas filled by one mosaic: every cell holds
a color, and none holds the background color 0. The four mosaics use three
colors each: 1, 2, and 5; 2, 5, and 6; 5, 8, and 9; and 1, 2, and 6. The
output keeps the canvas and mirrors the whole mosaic across its main
diagonal, the line from the top-left corner to the bottom-right corner, so the
cell in row r and column c moves to row c and column r. Each input row becomes
the output column of the same index, the three cells on the diagonal stay in
place, and every color keeps its cell count. Of the square's eight turns and
mirrors, only this flip maps each training input to its output, and no
training mosaic looks the same under any of the other seven, so each pair
confirms the flip on its own. No turn reproduces any training output, because
a mirror reverses the mosaic's handedness and a turn never does. The output
keeps the input's 3 by 3 canvas, so the dataset is `EXACT`; with no zero cell
to crop, `FIT` would frame the same grids.

`/arc-purposal` chose this puzzle by default: among the 385 training puzzles
other than the genesis puzzle `3aa6fb7a` and those with a dataset, its largest
training matrix has the fewest cells, 9, with 72 cells in all its training
matrices. Three other puzzles, `9565186b`, `a85d4709`, and `ed36ccf7`, tie on
both measures, and `74dd1130` has the lowest ID among them. It was one of the
puzzles tied with [`3c9b0459`](ARC-NUC-3c9b0459.md), the previous default.

## Gestalt overview

- **Whole grid.** A 3 by 3 canvas filled by one mosaic of three colors, with
  no background cell.
- **Figures.** One figure per grid, the whole mosaic.
- **Relations.** The mosaic relates to the canvas's main diagonal, from the
  top-left corner to the bottom-right. No training mosaic looks the same
  under another turn or mirror of the square, so each pair shows the flip on
  its own.
- **Change.** The whole mosaic flips across that diagonal, so its rows
  become its columns.
- **World model.** One object per pair, the whole mosaic, whose flip is its
  `mirror_axis`, 135.

## Dataset

The dataset holds the four original training pairs in source order, followed
at the same level by the puzzle's test pair, which the same rule annotates.
Every mosaic is one instance of `Obj(x, y, orientation)`, tagged `l9100`, a
single shape, as in `dataset.28bf18c6.json` and `dataset.3c9b0459.json`. The
class takes no color, size, cell list, canvas size, or matrix of cells: the
mosaic's cells travel unchanged in its masks. One unconditional class-level
rule states the flip:

```nuc
$Obj.mirror_axis = 135
```

`mirror_axis` is the axis the mosaic is mirrored across, as an angle clockwise
from north, and is absent from every input state. It follows the convention of
the line orientations in `dataset.line.json`, where vertical lines are 0 and
horizontal lines 90, so the main diagonal, which runs from northwest to
southeast, is 135, and the other diagonal would be 45. `orientation` cannot
carry the change, since it is a clockwise turn and no turn reproduces a
training output, so it stays 0 in both states. The rule creates no other
property and leaves `x`, `y`, and the tag unchanged: a mosaic that fills its
square canvas keeps its box at 0 and 0 under the flip. Each output state adds
only `mirror_axis` to its input state. The declarations carry no `#` comments,
as in `dataset.3c9b0459.json`, `dataset.25d8a9c8.json`, and
`dataset.25ff71a9.json`. The [README](../README.md#instruction-datasets)
describes the dataset in full.

The worker started from the big picture, the whole mosaic as the object, and
that formulation explained every training pair, so it never went down to rows,
color regions, or single cells. It weighed two alternatives. A turn plus a
mirror flag would state one change with two properties and a convention for
their order. Rules that swap the box's origin, `$Obj.x = $Obj.y@first` and
`$Obj.y = $Obj.x@first`, reproduce every training output too, but every
mosaic's origin is 0 and 0, so they would only return the input origin, the
reason `dataset.3c9b0459.json` also leaves its position rules out. The worker
fixed the rule from the training pairs before it read the test pair and
reported that the same rule reproduces it. Each training grid holds one mosaic
and relates it to nothing else, so the rule needs no population operation such
as `filter` or `some`. This is the first dataset that states a mirror as a
transformation: `dataset.44f52bb0.json` only tests whether a pattern is mirror
symmetric, and `dataset.28bf18c6.json` repeats its shape rather than mirroring
it.

The caller checked the handoff against the declarative, higher-order, and
gestalt requirements, found that it met them, and sent nothing back. It
confirmed the line-orientation convention in `dataset.line.json`. It also
replayed the four training pairs on the accepted runtime: 98 checks of grids,
keys, the flip, mask shapes and cells, bounding boxes, merges,
implementations, orientations, and input and output snapshots, all passing.
The worker wrote the dataset, its README paragraph, and the puzzle's entry in
the **ARC puzzle report** workflow's list, its 16th puzzle. The caller made the
puzzle the solver's default in `src/app.ts`, in place of `3c9b0459`, before the
run.

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
$Obj.mirror_axis = 135
```

No rule changed: the mirror stays in `mirror_axis`, and every box stays 3 by
3. The migration worker regenerated every snapshot with the accepted runtime
at `491512d`, and the caller replayed every pair offline against the unchanged
grids, masks, positions and instance names. No solver run has measured this
version yet.

## Runs

| Date | Dataset version | Runtime | Exit status | Difference | Elapsed |
| --- | --- | --- | --- | --- | --- |
| 5-Nov-2026 05:46 UTC | 1, one mosaic per object | Accepted `a605465`, local | 0 | 0, exact match | 1m 55.1s |

The run was `bun start 74dd1130` from `arc/`, run once by `/arc-purposal` with
a 30-minute limit and `NUCLEOID_EXECUTABLE` unset, after
`bun install --frozen-lockfile` reported no changes. It used the primary
checkout's debug runtime (`nucleoid 1.0.0`), built with `cargo build` at
`a605465`, and the committed solver at `a605465` with only its default puzzle
changed. The solver reported an exact match, a difference of 0, and an elapsed
time of 1m 55.1s, and wrote `logs/74dd1130.log.json`, which Git ignores.
Unlike the runs of [25d8a9c8](ARC-NUC-25d8a9c8.md),
[25ff71a9](ARC-NUC-25ff71a9.md), and [3c9b0459](ARC-NUC-3c9b0459.md), it
reached the model and scored a prediction. The hashes of the runtime, the
dataset, and `src/app.ts` were the same before and after the run. The
invocation authorizes one run, so none was repeated. The puzzle has not been in
an **ARC puzzle report** workflow run.

The match does not show that the solver inferred the flip from the training
pairs alone: as the first open item explains, the run's few-shot context held
this dataset, its annotated test pair and expected output included.

## Open items

- **A run sees the answer.** Resolved on 6-Nov-2026: the user decided this is
  intended, because the datasets are being prepared as few-shot examples and
  runs at this stage are not yet held-out tests. The ARC instructions now say
  so, and the suggested change to keep the running puzzle's test pairs out of
  `document(puzzle_id)` is withdrawn. `src/lib/llm.ts` sends
  `dataset.document(puzzle_id)` as few-shot context, and
  `src/dataset/dataset.ts` discards `puzzle_id`. This run's prompts therefore
  held this dataset's annotated test pair, expected output included, so its
  exact match does not show that the flip, or `mirror_axis`, a property no
  other dataset uses, was learned from the training pairs.
- **Messages are not JSON-only.** `src/lib/llm.ts`, `src/analyzer.ts`, and
  `src/visualizer.ts` wrap JSON in text labels such as `few_shot:`,
  `control_id:`, `train_dataset:`, and `json_format:`, while the ARC
  instructions require every message to parse as one JSON document.
  Suggested, not applied: build each message with `JSON.stringify` of one
  data object.
- **Lint fails on untouched solver files.** At `a605465`, `bun run lint`
  reports 28 `prettier/prettier` errors, 4 in `src/analyzer.ts` and 24 in
  `src/visualizer.ts`, files this invocation did not change, while
  `bun run typecheck` passes with the new default. Suggested, not applied:
  reformat both files with Prettier, for example `bunx eslint --fix
  src/analyzer.ts src/visualizer.ts`, a formatting-only change.
- **Comments in declarations.** Resolved on 6-Nov-2026: the user decided that
  natural-language `#` comments may reach the model only as part of
  `declarations`, and the ARC instructions now say so. Datasets with and
  without such comments both follow them, so nothing needs to change.
- **Workflow timeout.** Resolved on 6-Nov-2026: at the user's request, the
  arc-dataset profile now asks for the batch budget the workflow needs,
  `ceil(listed puzzles / ARC_CONCURRENCY) * 30 + 20` minutes, within GitHub
  Actions' 360-minute job limit. The workflow's 150-minute timeout covers its
  20 listed puzzles.

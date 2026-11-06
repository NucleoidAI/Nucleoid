# ARC-NUC-25ff71a9 -- One-Row Downward Shift

Progress record for ARC training puzzle `25ff71a9`: its instruction dataset,
the solver and Nucleoid changes it needed, its runs, and what is still open.

| Item | Status |
| --- | --- |
| Puzzle | [`data/training/25ff71a9.json`](../data/training/25ff71a9.json) |
| Instruction dataset | [`dataset.25ff71a9.json`](../src/dataset/dataset.25ff71a9.json): the puzzle's 4 original training pairs and its 2 test pairs, 6 figure instances, one per pair; committed with this record |
| Output canvas | `EXACT` |
| Nucleoid enhancements | None requested, so no proposal strategy was selected; the rule uses the class-level `@first` anchor of [NUC 55](../../docs/nuc-0055.md), built on the state versions of [NUC 53](../../docs/nuc-0053.md) |
| Solver changes | None; suggested changes await review, not applied |
| Solver runs | 1 local run on 4-Nov-2026, exit 1 before any model call (`OPENROUTER_API_KEY is required`); difference unknown |
| Updated | 6-Nov-2026 |

## Puzzle

Each training input is a 3 by 3 canvas holding one figure: three cells of a
single color, joined through their edges, on the background color 0. The
output keeps the canvas and moves the whole figure down one row, with its
shape, color, and columns unchanged. The first two figures are full-width bars
of color 1, on the top row and on the middle row, and they move to the middle
row and the bottom row. The third is an L-shaped tromino of color 1 in the
canvas's top-left 2 by 2 box, and the fourth an L-shaped tromino of color 2 in
its top-right 2 by 2 box; each drops one row within its columns. The step is
one row, not a fall to the bottom edge, since the first bar stops on the
middle row above the empty bottom row, and not the figure's height, since the
two-row trominoes also move one row. Every training output keeps an empty top
row, which `FIT` would crop, so the dataset is `EXACT`.

`/arc-purposal` chose this puzzle by default: among the 387 training puzzles
other than the genesis puzzle `3aa6fb7a` and those with a dataset, its largest
training matrix has the fewest cells, 9, with 72 cells in all its training
matrices. Five other puzzles, `3c9b0459`, `74dd1130`, `9565186b`, `a85d4709`,
and `ed36ccf7`, tie on both measures, and `25ff71a9` has the lowest ID among
them.

## Gestalt overview

- **Whole grid.** A 3 by 3 background canvas holds one figure of three cells
  above an empty bottom row.
- **Figures.** The whole figure, edge-connected and of one color: a
  full-width bar of color 1 in the first two pairs, and an L-shaped tromino
  of color 1 or 2 in the other two.
- **Relations.** The figure relates to nothing but the canvas it moves in.
- **Change.** The whole figure drops one row, keeping its shape, color, and
  columns.
- **World model.** One object per pair, the whole figure, the unit seen
  first. Rolling the canvas's rows down would also fit the training pairs,
  but it is a finer, row-level reading.

## Dataset

The dataset holds the four original training pairs in source order, followed
at the same level by the puzzle's two test pairs, which the same rule
annotates. Every figure is one instance of `Obj(x, y, orientation)`, tagged
`l9100`, a single shape, as in `dataset.28bf18c6.json` and
`dataset.6150a2bd.json`. The class takes no color, size, cell list, canvas
size, or matrix of cells: the figure's cells travel unchanged in its masks,
and the visualizer places each predicted crop at the output position. One
unconditional class-level rule moves every figure:

```nuc
$Obj.y = $Obj.y@first + 1
```

`@first` reads the figure's `y` from before the rules ran, which the rule
needs because it rewrites `y` itself. The rule creates no property and leaves
`x`, `orientation`, and the tag unchanged, so each input and output state
differs only in `y`. The declarations carry no `#` comments, as in
`dataset.25d8a9c8.json`. The [README](../README.md#instruction-datasets)
describes the dataset in full.

The worker started from the big picture, the whole figure as the object, and
that formulation explained every training pair, so it never went down to
single cells. The training pairs rule out a fall to the bottom edge and a step
of the figure's height. Rolling the canvas's rows down with wraparound also
reproduces every training output, because every training input's bottom row
is empty, but it is a finer row-level model, so the worker kept the figure,
the unit seen first, and fixed that choice before reading the test pairs. Each
training grid holds one figure and relates it to nothing else, so the rule
needs no population operation such as `filter` or `some`; the class-level
rule already states the move once for every figure.

The caller checked the handoff against the declarative, higher-order, and
gestalt requirements, found that it met them, and sent nothing back. The
caller also replayed the four training pairs on the accepted runtime, 77
checks of grids, keys, mask shapes and cells, bounding boxes, merges, and
input and output snapshots, all passing. The worker froze the rules from the
training pairs before it read the test pairs and reported that the same rules
reproduce both. It wrote the dataset, its README paragraph, and the puzzle's
entry in the **ARC puzzle report** workflow's list, its 14th puzzle. The
caller made the puzzle the solver's default in `src/app.ts`, in place of
`25d8a9c8`, before the run.

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
$Obj.y = $Obj.y@first + 1
```

No rule changed: the shape only moves down one row. The migration worker
regenerated every snapshot with the accepted runtime at `491512d`, and the
caller replayed every pair offline against the unchanged grids, masks,
positions and instance names. No solver run has measured this version yet.

## Runs

| Date | Dataset version | Runtime | Exit status | Difference | Elapsed |
| --- | --- | --- | --- | --- | --- |
| 4-Nov-2026 20:51 UTC | 1, one figure per object | Accepted `2131caf`, local | 1 | Unknown: `Error: OPENROUTER_API_KEY is required` | 0.5s |

The run was `bun start 25ff71a9` from `arc/`, run once by `/arc-purposal` with
a 30-minute limit and `NUCLEOID_EXECUTABLE` unset, after
`bun install --frozen-lockfile` reported no changes. It used the primary
checkout's debug runtime (`nucleoid 1.0.0`), rebuilt with `cargo build` at
`2131caf`, and the committed solver at `2131caf` with only its default puzzle
changed. It stopped in the analyzer's first model call,
`analyzer.declarations`, where `src/lib/openrouter.ts` raised
`OPENROUTER_API_KEY is required`, before any model request, prediction, or
score. Its difference is therefore unknown, not zero, and the solver wrote no
`logs/25ff71a9.log.json`. The solver reported no elapsed time, so elapsed is
the process's run from 20:51:57.4 to 20:51:57.9 UTC. The hashes of the
runtime, the dataset, and `src/app.ts` were the same before and after the run.
The invocation authorizes one run, so none was repeated. The puzzle has not
been in an **ARC puzzle report** workflow run.

## Open items

- **No model credential.** As in the run of [25d8a9c8](ARC-NUC-25d8a9c8.md),
  the solver process had no `OPENROUTER_API_KEY` value, so it stopped before
  its first model request. Measuring this dataset takes a separately
  authorized run with the credential available to the solver.
- **A run would see the answer.** Resolved on 6-Nov-2026: the user decided
  this is intended, because the datasets are being prepared as few-shot
  examples and runs at this stage are not yet held-out tests. The ARC
  instructions now say so, and the suggested change to keep the running
  puzzle's test pairs out of `document(puzzle_id)` is withdrawn.
  `src/lib/llm.ts` sends `dataset.document(puzzle_id)` as few-shot context,
  and `src/dataset/dataset.ts` discards `puzzle_id`. Any run of this puzzle
  therefore holds this dataset's two annotated test pairs, expected outputs
  included, in its prompts, and an exact match would not show that the rule
  was inferred from the training pairs.
- **Messages are not JSON-only.** `src/lib/llm.ts` and `src/analyzer.ts` wrap
  JSON in text labels such as `few_shot:`, `control_id:`, `train_dataset:`,
  and `json_format:`, while the ARC instructions require every message to
  parse as one JSON document. Suggested, not applied: build each message with
  `JSON.stringify` of one data object.
- **Comments in declarations.** Resolved on 6-Nov-2026: the user decided that
  natural-language `#` comments may reach the model only as part of
  `declarations`, and the ARC instructions now say so. Datasets with and
  without such comments both follow them, so nothing needs to change.
- **Workflow timeout.** Resolved on 6-Nov-2026: at the user's request, the
  arc-dataset profile now asks for the batch budget the workflow needs,
  `ceil(listed puzzles / ARC_CONCURRENCY) * 30 + 20` minutes, within GitHub
  Actions' 360-minute job limit. The workflow's 150-minute timeout covers its
  20 listed puzzles.

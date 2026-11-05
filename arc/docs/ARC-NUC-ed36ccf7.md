# ARC-NUC-ed36ccf7 -- Pattern Quarter Turn

Progress record for ARC training puzzle `ed36ccf7`: its instruction dataset,
the solver and Nucleoid changes it needed, its runs, and what is still open.

| Item | Status |
| --- | --- |
| Puzzle | [`data/training/ed36ccf7.json`](../data/training/ed36ccf7.json) |
| Instruction dataset | [`dataset.ed36ccf7.json`](../src/dataset/dataset.ed36ccf7.json): the puzzle's 4 original training pairs and its test pair, 5 pattern instances, one per pair; committed with this record |
| Output canvas | `EXACT` |
| Nucleoid enhancements | None requested, so no proposal strategy was selected and strategy learning did not apply; the rules use the `@first` anchors of [NUC 55](../../docs/nuc-0055.md) and the accepted `.value` read |
| Solver changes | None; suggested changes await review, not applied |
| Solver runs | 1 local run on 5-Nov-2026, exit 1 before any model call (`OPENROUTER_API_KEY is required`); difference unknown |
| Updated | 5-Nov-2026 |

## Puzzle

Each training input is a 3 by 3 canvas holding one pattern of a single color
on a background of zero cells: 7 cells of color 9, 5 of color 6, 5 of color 9,
and 5 of color 2. Not every pattern is connected. The second is two bars, the
full top row and the left two cells of the bottom row, and the fourth is a
lone top-left cell beside a hook of four cells along the right column and the
bottom row. The output keeps the canvas and turns the whole grid a quarter
turn counterclockwise, so the cell in row r and column c moves to row 2 - c
and column r: the right column becomes the top row, and every color keeps its
cell count. Of the square's eight turns and mirrors, only this turn maps every
training input to its output. The third pattern, an L symmetric across the
main diagonal, also fits a top-bottom mirror, but each of the other three
pairs fits only the quarter turn. Every training pattern's box spans its
3 by 3 canvas before and after the turn. The output keeps the input's canvas,
so the dataset is `EXACT`; every training output's pattern spans its canvas,
so `FIT` would frame the same training grids.

`/arc-purposal` chose this puzzle by default: among the 382 training puzzles
other than the genesis puzzle `3aa6fb7a` and those with a dataset, its largest
training matrix has the fewest cells, 9, with 72 cells in all its training
matrices. `6e02f1e3` and `794b24be` also have a largest matrix of 9 cells, but
90 and 180 cells in all, so no other puzzle tied on both measures. `ed36ccf7`
had tied on both with [`3c9b0459`](ARC-NUC-3c9b0459.md),
[`74dd1130`](ARC-NUC-74dd1130.md), [`9565186b`](ARC-NUC-9565186b.md), and
[`a85d4709`](ARC-NUC-a85d4709.md), which earlier invocations took.

## Gestalt overview

- **Whole grid.** A 3 by 3 background canvas holds one pattern of a single
  color.
- **Figures.** One figure per grid, the whole pattern, whether or not its
  cells touch.
- **Relations.** The pattern relates to the canvas center, which it turns
  about.
- **Change.** The whole grid turns a quarter turn counterclockwise, so the
  right column becomes the top row.
- **World model.** One object per pair, the whole pattern, turned to
  orientation 270 and moved about the canvas center.

## Dataset

The dataset holds the four original training pairs in source order, followed
at the same level by the puzzle's test pair, which the same rules annotate.
Each grid's whole pattern is one instance of `Obj(x, y, orientation, width)`,
tagged `l9100`, a single shape, as in `dataset.28bf18c6.json`,
`dataset.3c9b0459.json`, and `dataset.6150a2bd.json`. `width` is the number of
columns of the pattern's box. The class takes no color, height, parts list,
canvas size, or matrix of cells: the pattern's cells travel unchanged in its
masks, and its turn is its orientation. Three unconditional class-level rules
turn every pattern about the canvas center:

```nuc
$Obj.orientation = ($Obj.orientation@first + 270) % 360
$Obj.x = $Obj.y@first.value
$Obj.y = canvas_width - $Obj.x@first.value - $Obj.width
```

A counterclockwise quarter turn is 270 degrees clockwise, as
`dataset.6150a2bd.json` adds 180 for its half turn. The box's top row becomes
the turned box's left column, and its right column becomes the turned box's
top row, as far below the top edge as the box was left of the right edge; the
box's height is not needed. The rules read `canvas_width` from the solver, as
`dataset.d037b0a7.json` reads `canvas_height`. The `@first` anchors read the
pattern's input position whatever order the rules run in. The rules create no
property and leave `width` and the tag unchanged, so each input state has
orientation 0 and each output state 270, with the same keys. The declarations
carry no `#` comments. The [README](../README.md#instruction-datasets)
describes the dataset in full.

The `x` and `y` rules each read the other coordinate's anchor. Written with
plain `@first` reads, the accepted runtime rejects them with
`TypeError: Circular Dependency`, because [NUC 55](../../docs/nuc-0055.md)
exempts only a rule's own fixed anchor from the class cycle check. The
[value property](../../docs/README.md#the-value-property) reads a value
without recording a dependency, and the language reference documents it for
breaking what would otherwise be a cycle. The worker therefore read both
anchors with `.value` and requested no enhancement: an accepted declarative
formulation works, and the arc-dataset profile takes one that works over
requesting a shorter spelling.

The worker started from the big picture, each grid's whole pattern as one
object whether or not its cells touch, and that formulation explained every
training pair, so it never went down to pieces or single cells. One choice
remained that the training pairs could not settle. Every training pattern's
box spans its canvas, so turning the pattern in place, with `orientation`
alone, as `dataset.3c9b0459.json` states its half turn, also reproduces every
training output. The worker turned it about the canvas center instead, as
`dataset.6150a2bd.json` does, because these patterns lie on a background of
zero cells. Unlike the mosaics of `dataset.3c9b0459.json` and
`dataset.74dd1130.json`, which have no zero cell, their boxes need not span
the canvas, and a turn of the whole grid moves a box that does not. The worker
recorded that choice, with the full declarations, before it read the test
pair, and reported that the same rules reproduce the test pair. It also noted
that ARC training puzzles are public, so it could not rule out having seen
this puzzle's test pair before; the choice rests on the training-only argument
above. Each training grid holds one pattern and relates it to nothing else,
so the rules need no population operation such as `filter` or `some`.

The caller checked the handoff against the declarative, higher-order, and
gestalt requirements and found that it met them: each relation is stated once
over every pattern, no property restates another, and `.value` is an accepted
declarative read, not an imperative workaround. It sent nothing back. It
replayed the four training pairs on the accepted runtime, and all 106 checks
passed. They covered:

- grids, keys, and the quarter turn;
- mask shapes and cells, bounding boxes, and merges;
- implementations, widths, and orientations;
- input and output snapshots, with the rules declared before and after the
  instance.

`document()` includes the dataset as the 20th of the 22 JSON instruction
datasets, in filename order, and its five instance names appear nowhere else
in the repository.

The worker wrote the dataset, its README paragraph, and the puzzle's entry in
the **ARC puzzle report** workflow's list, its 19th puzzle. The caller made
the puzzle the solver's default in `src/app.ts`, in place of `a85d4709`,
before the run.

## Runs

| Date | Dataset version | Runtime | Exit status | Difference | Elapsed |
| --- | --- | --- | --- | --- | --- |
| 5-Nov-2026 07:50 UTC | 1, one pattern per object | Accepted `161e65d`, local | 1 | Unknown: `Error: OPENROUTER_API_KEY is required` | 0.7s |

The run was `bun start ed36ccf7` from `arc/`, run once by `/arc-purposal` with
a 30-minute limit and `NUCLEOID_EXECUTABLE` unset, after
`bun install --frozen-lockfile` reported no changes. It used the primary
checkout's debug runtime (`nucleoid 1.0.0`), built with `cargo build` at
`161e65d`, and the committed solver at `161e65d` with only its default puzzle
changed. It stopped in the analyzer's first model call,
`analyzer.declarations`, where `src/lib/openrouter.ts` raised
`OPENROUTER_API_KEY is required`, before any model request, prediction, or
score. Its difference is therefore unknown, not zero, and the solver wrote no
`logs/ed36ccf7.log.json`. The solver reported no elapsed time, so elapsed is
the process's run from 07:50:24.3 to 07:50:25.0 UTC. The hashes of the
runtime, the dataset, and `src/app.ts` were the same before and after the run.
The invocation authorizes one run, so none was repeated. The puzzle has not
been in an **ARC puzzle report** workflow run.

## Open items

- **No model credential.** As in the runs of
  [25d8a9c8](ARC-NUC-25d8a9c8.md), [25ff71a9](ARC-NUC-25ff71a9.md), and
  [3c9b0459](ARC-NUC-3c9b0459.md), the solver process had no
  `OPENROUTER_API_KEY` value, so it stopped before its first model request.
  The runs of [74dd1130](ARC-NUC-74dd1130.md) and
  [a85d4709](ARC-NUC-a85d4709.md) earlier the same day reached the model.
  Measuring this dataset takes a separately authorized run with the credential
  available to the solver.
- **A run would see the answer.** `src/lib/llm.ts` sends
  `dataset.document(puzzle_id)` as few-shot context, and
  `src/dataset/dataset.ts` discards `puzzle_id`. Any run of this puzzle
  therefore holds this dataset's annotated test pair, expected output
  included, in its prompts, and an exact match would not show that the turn
  was inferred from the training pairs. Suggested, not applied: keep the
  running puzzle's test pairs out of `document(puzzle_id)`.
- **Messages are not JSON-only.** `src/lib/llm.ts`, `src/analyzer.ts`, and
  `src/visualizer.ts` still wrap JSON in text labels such as `few_shot:`,
  `control_id:`, `train_dataset:`, and `json_format:`, while the ARC
  instructions require every message to parse as one JSON document.
  Suggested, not applied: build each message with `JSON.stringify` of one
  data object.
- **Lint fails on untouched solver files.** At `161e65d`, `bun run lint`
  reports 28 `prettier/prettier` errors, all in `src/analyzer.ts` and
  `src/visualizer.ts`, which this invocation did not change. `src/app.ts`
  lints clean with the new default, and `bun run typecheck` passes.
  Suggested, not applied: reformat both files with Prettier, a
  formatting-only change.
- **Anchors of other properties count as dependencies.** The accepted runtime
  treats a rule's read of another property's `@first` anchor as a dependency
  on that property, so two rules that each read the other's anchor form a
  class cycle; NUC 55 exempts only a rule's own anchor. This dataset breaks
  the cycle with `.value`. Whether a separate proposal should exempt the fixed
  anchors of other properties too is open; this invocation requested none,
  because the accepted formulation works.
- **Comments in declarations.** This dataset and six others carry none,
  following `.claude/rules/arc.instructions.md`, while the other 15 datasets
  carry `#` comments. Decide which wins; neither choice changes any snapshot.
- **Workflow timeout wording.** The arc-dataset profile still asks for a job
  timeout of at least 30 minutes per listed puzzle plus 20 minutes, 590
  minutes for the 19 puzzles now listed, beyond GitHub Actions' 360-minute job
  limit. Since `56b093c` the workflow runs the puzzles in batches of
  `ARC_CONCURRENCY`, 5, under a 150-minute job timeout, and the README asks
  for a budget sufficient for all batches. The worker sized it by batch, four
  batches of 30 minutes plus 20, 140 minutes, and left the timeout at 150.
  Suggested, not applied: state the batch budget in the profile.

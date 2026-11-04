# ARC-NUC-25d8a9c8 -- Uniform Row Bars

Progress record for ARC training puzzle `25d8a9c8`: its instruction dataset,
the solver and Nucleoid changes it needed, its runs, and what is still open.

| Item | Status |
| --- | --- |
| Puzzle | [`data/training/25d8a9c8.json`](../data/training/25d8a9c8.json) |
| Instruction dataset | [`dataset.25d8a9c8.json`](../src/dataset/dataset.25d8a9c8.json): the puzzle's 4 original training pairs and its test pair, 15 row instances, three per pair; committed with this record |
| Output canvas | `EXACT` |
| Nucleoid enhancements | None requested, so no proposal strategy was selected; the rule uses the list query `distinct` of [NUC 71](../../docs/nuc-0071.md) and the conditional expression of [NUC 59](../../docs/nuc-0059.md) |
| Solver changes | None; suggested changes await review, not applied |
| Solver runs | 1 local run on 4-Nov-2026, exit 1 before any model call (`OPENROUTER_API_KEY is required`); difference unknown |
| Updated | 4-Nov-2026 |

## Puzzle

Each training input is a 3 by 3 canvas of three full-width rows, with no zero
cell. Some rows hold a single color, and the others mix two colors. Each
output keeps the canvas and every row in place: a row whose three cells share
one color becomes a solid color-5 row, and every other row is cleared to the
background color 0. The single-color training rows are the top row of the
first input, the middle row of the second, the middle and bottom rows of the
third, and the bottom row of the fourth. Their colors, 4, 6, 4 and 9, and 1,
all turn 5, so the rule tests whether a row is uniform, not which color it
has. The output keeps the input's canvas, including the cleared top or bottom
rows that `FIT` would crop, so the dataset is `EXACT`.

`/arc-purposal` chose this puzzle by default: among the training puzzles other
than the genesis puzzle `3aa6fb7a` and those with a dataset, its largest
training matrix has the fewest cells, 9, with 72 cells in all its training
matrices. Six other puzzles tie on both measures, and `25d8a9c8` has the
lowest ID among them.

## Dataset

The dataset holds the four original training pairs in source order, followed
at the same level by the puzzle's test pair, which the same rule annotates.
Every row is one instance of `Obj(x, y, orientation, parts)`, a horizontal bar
tagged `l3058` in both states, as in `dataset.d631b094.json`. `parts` lists
the row's three colors from left to right, such as `[4, 4, 4]` for the top
row of the first training input. The class has no color, color count, canvas
size, or matrix of cells, so no input property states whether the row is
uniform. One unconditional class-level rule introduces `fill_color`:

```nuc
$Obj.fill_color = 5 if $Obj.parts.distinct().length == 1 else 0
```

`distinct` keeps each of the row's colors once, so a single kept color means
the row is one color, and the conditional expression gives such a row color 5
and every other row the background color 0. `fill_color` is absent from the
input states, and the rule leaves `x`, `y`, `orientation`, and the tag
unchanged. A cleared row's output mask is all zero, so its `output_x` and
`output_y` record the row's unchanged origin, the explicit output position an
erased output object keeps in composition. This is the first dataset whose
objects vanish. Unlike the other datasets, the declarations carry no `#`
comments, following the ARC instructions' rule against prose comments in code
fields. The [README](../README.md#instruction-datasets) describes the dataset
in full.

The worker started from the big picture: each grid is three full-width rows,
and every output row is a single color, so the row is the unit in which every
output changes. It weighed two other readings on the training pairs. The whole
grid as one object would need a per-row map inside it and an output made of
separate stripes. Same-colored shapes also fit the training pairs, but they
need size tests, one row tall and as wide as the canvas, and they split mixed
rows into pieces, 17 objects instead of 12 in the training pairs. No training
pair relates one row to another, so the rule needs no population query. The
quantifier `all part in $Obj.parts: part == $Obj.parts[0]` also works, but it
needs a reference cell and is longer than `distinct`.

`parts` gives each row the list of its cells' colors. The records of
[017c7c7b](ARC-NUC-017c7c7b.md), [27a28665](ARC-NUC-27a28665.md), and
[44f52bb0](ARC-NUC-44f52bb0.md) treat such a list as allowed only where the
user's hint asks for one, and this invocation had no hint. The caller checked
the handoff against the declarative, higher-order, and gestalt requirements,
found that it met them, and asked the user about the list before the run; the
user approved it. The caller also replayed the four training pairs on the
accepted runtime, 133 checks of grids, mask shapes and cells, merges, and
input and output snapshots, all passing, and sent nothing back.

The worker froze the rules from the training pairs before it read the test
pair, and the same rules reproduce the test pair. It wrote the dataset, its
README paragraph, and the puzzle's entry in the **ARC puzzle report**
workflow's list, its 13th puzzle. The caller made the puzzle the solver's
default in `src/app.ts`, in place of `0d3d703e`, before the run.

## Runs

| Date | Dataset version | Runtime | Exit status | Difference | Elapsed |
| --- | --- | --- | --- | --- | --- |
| 4-Nov-2026 20:21 UTC | 1, one row per object | Accepted `5b3f3e7`, local | 1 | Unknown: `Error: OPENROUTER_API_KEY is required` | 0.7s |

The run was `bun start 25d8a9c8` from `arc/`, run once by `/arc-purposal` with
a 30-minute limit and `NUCLEOID_EXECUTABLE` unset. It used the primary
checkout's debug runtime (`nucleoid 1.0.0`), rebuilt with `cargo build` at
`5b3f3e7`, and the committed solver at `5b3f3e7` with only its default puzzle
changed. It stopped in the analyzer's first model call, where
`src/lib/openrouter.ts` raised `OPENROUTER_API_KEY is required`, before any
model request, prediction, or score. Its difference is therefore unknown, not
zero, and the solver wrote no `logs/25d8a9c8.log.json`. The solver reported no
elapsed time, so elapsed is the process's run from 20:21:58.6 to 20:21:59.4
UTC. The runtime's hash and the dataset's were the same before and after the
run. The invocation authorizes one run, so none was repeated. The puzzle has
not been in an **ARC puzzle report** workflow run.

## Open items

- **No model credential.** The run's process had no `OPENROUTER_API_KEY` in
  its environment, so the solver stopped before its first model request.
  Measuring this dataset takes a separately authorized run with the
  credential available to the solver.
- **A run would see the answer.** `src/lib/llm.ts` sends `document(puzzle_id)`
  as few-shot context, and `src/dataset/dataset.ts` discards `puzzle_id`. Any
  run of this puzzle therefore holds this dataset's annotated test pair,
  expected output included, in its prompts, and an exact match would not show
  that the rule was inferred from the training pairs. Suggested, not applied:
  keep the running puzzle's test pairs out of `document(puzzle_id)`.
- **Messages are not JSON-only.** `src/lib/llm.ts`, `src/analyzer.ts`, and
  `src/visualizer.ts` wrap JSON in text labels such as `control_id:`,
  `few_shot:`, `train_dataset:`, and `input_x:`, while the ARC instructions
  require every message to parse as one JSON document. Suggested, not applied:
  build each message with `JSON.stringify` of one data object.
- **Comments in declarations.** This dataset carries none, following
  `.claude/rules/arc.instructions.md`, while the other 15 datasets carry `#`
  comments. Decide which wins; neither choice changes any snapshot.
- **Workflow timeout.** The arc-dataset profile asks for a job timeout of at
  least 30 minutes per listed puzzle plus 20 minutes, 410 minutes for the 13
  puzzles now listed, which exceeds GitHub Actions' 360-minute job limit,
  while `e8ef229` set it to 30 minutes because the puzzles run in parallel.
  The worker added this puzzle and left the timeout at 30. Decide which holds
  and update the profile or the workflow.
- **Test-pair facts in the handoff.** The worker's handoff quoted the test
  pair's input and output grids to the caller. The caller's review and its
  question to the user used the training pairs only, and nothing from the
  test pair reached another agent, the rules, or the run.

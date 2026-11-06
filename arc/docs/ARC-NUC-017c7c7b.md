# ARC-NUC-017c7c7b -- Periodic Pattern Continuation

Progress record for ARC training puzzle `017c7c7b`: its instruction dataset,
the solver and Nucleoid changes it needed, its runs, and what is still open.

| Item | Status |
| --- | --- |
| Puzzle | [`data/training/017c7c7b.json`](../data/training/017c7c7b.json) |
| Instruction dataset | [`dataset.017c7c7b.json`](../src/dataset/dataset.017c7c7b.json): the puzzle's 3 original training pairs and its test pair, one whole-pattern instance each (4 instances); version 3, which is version 2 without its `resume_row` rule (version 1 had 36 point instances); committed in `a5a793d` and moved to the standard entries in `521aa3c` |
| Output canvas | `EXACT` |
| Nucleoid enhancements | None requested for any version, so no proposal strategy was selected; versions 2 and 3 use the `@first` anchors of [NUC 53](../../docs/nuc-0053.md) and [NUC 55](../../docs/nuc-0055.md) |
| Solver changes | None; one suggested change awaits review, not applied |
| Solver runs | 2 local runs on 4-Nov-2026: version 1 exit 2, version 2 exit 0; version 3 not run |
| Updated | 6-Nov-2026 |

## Puzzle

Each training input is a 6 by 3 canvas holding one color-1 pattern that
repeats down the canvas: every 4 rows in the first training pair, every 2 in
the second, and every 3 in the third. The output is 9 by 3, one and a half
times the input's height. It continues the same pattern down to the ninth
row, with every cell recolored 2. Appending the input's first three rows
would reproduce the second and third training outputs but not the first,
whose pattern repeats every four rows, so the period must be found. The third
training output keeps an empty right column, which `FIT` would crop, so the
dataset is `EXACT`.

## Gestalt overview

- **Whole grid.** A narrow 6 by 3 canvas holds one color-1 pattern from its
  top row to its bottom row; the output is a taller 9 by 3 canvas.
- **Figures.** One figure per grid, the whole pattern, whose cells all touch
  through edges or corners. It reads as a motif of rows repeated down the
  canvas, a motif 4, 2, and 3 rows tall in the three training pairs.
- **Relations.** The pattern relates only to itself: its rows repeat every
  `period` rows, and nothing else is on the canvas.
- **Change.** The whole pattern grows down to one and a half times its
  height, continuing its own rhythm rather than repeating its top rows, and
  turns color 2 as one.
- **World model.** One object per pair, the whole pattern, measured by its
  `height` and `period`. Version 1's one point per cell was finer than the
  figure, and the user's hint for version 2 moved the model to the whole
  pattern.

## Dataset

The dataset holds the three original training pairs in source order, followed
by the original test pair at the same level. In every input the color-1 cells
form one pattern whose cells touch through edges or corners, only through
corners in the second training pair. Each pair has one instance of
`Obj(x, y, orientation, color, height, period)`, tagged `l9100`, a single
shape, as in `dataset.6150a2bd.json`. `height` is the height of the pattern's
box, 6 rows in every training pair, and `period` is the number of rows after
which the pattern starts over: 4, 2, and 3 in the training pairs. Both are
scalars measured on the pattern's own box; no object carries a cell matrix or
a parts list. Unconditional class-level rules grow the pattern:

```nuc
$Obj.tag = 'l8384'
$Obj.extension_direction = 'down'
$Obj.height = $Obj.height@first * 3 / 2
$Obj.color = 2
```

`l8384` is a new tag for a pattern whose rows repeat every `period` rows down
to its height, so output row `r` is input row `r mod period`, recolored. The
first added row repeats row 2 in the first training pair and row 0 in the
others. `height` grows to one and a half times its first value, 9 rows,
through the self-anchor of NUC 55. `x` and `y` stay at the box's top-left
origin, and orientation is 0 in both states. `extension_direction` is absent
from the input states. Like the other datasets, the declarations carry `#`
comments. The [README](../README.md#instruction-datasets)
describes the dataset in full.

### Version 2 and the hint

On 4-Nov-2026 the second version replaced the first at the user's request,
with the hint "Consider it is single instance and extend 1.5 to the down with
first half". The training pairs confirm one instance per pair, since all of a
pattern's cells touch, and the growth to one and a half times its height. They
contradict "with first half": appending the pattern's top three rows would
reproduce the second and third training outputs but not the first, whose
pattern repeats every four rows, so its added rows repeat its rows 2 to 4.
Version 2's rules therefore resumed the pattern at a derived `resume_row`
rather than at its top row. The hint does not mention the recolor to 2, which
every training pair shows.

Version 1's rules derived the period from the population of its point
instances. With one instance per pair and no approved list of cells, nothing
is left to derive it from, so version 2 carries `period` as an input property,
measured like `height`, as `dataset.9af7a82c.json` carries `cell_count`. The
solver's model must therefore measure the period on the test pattern when it
writes the test instance. Version 2 also replaces version 1's `canvas_height`
with the pattern's own `height`, which the cropped mask the solver sends
shows.

The arc-dataset worker wrote every version and its README paragraph. Each
time it fixed the rules from the training pairs before reading the test pair,
and the same rules reproduce the test pair. No version needed a Nucleoid
enhancement. The first invocation also added `017c7c7b` to the
**ARC puzzle report** workflow's `ARC_PUZZLES` list.

### Version 3

After version 2's run, the user asked whether `resume_row` was needed. It was
not: `height@first % period` restates what `period` and the first height
already determine, no other rule reads it, and the solver's TypeScript reads
only `x` and `y` from an output state, leaving the rest to the model that
draws the output grid. At the user's request, version 3 drops that rule and
its property and keeps everything else of version 2. The worker regenerated
the output snapshots with the same runtime, and only `resume_row` left them.
The user also added to `/arc-purposal` the requirement to always go with the
declarative, higher-order, and most expressive solution. The worker applied it
by changing nothing beyond `resume_row` and suggested further candidates,
listed under the open items. Version 3 has not been run.

While that question was being answered, a repository search by the caller
printed the dataset's test-pair output state. Nothing was taken from it: it
reached no agent, and version 3's change was the user's decision, argued from
the training pairs.

### Versions

| Version | Source | Instances | Rules |
| --- | --- | --- | --- |
| 1 | `/arc-purposal 017c7c7b`, without a hint | One per nonzero input cell, a point tagged `l7544` with `color` and `canvas_height`: 36, with 9, 9, 8, and 10 per pair | `period` derived from the point population; every point repeated down its column, tagged `l2235` |
| 2 | `/arc-purposal 017c7c7b` with the hint "Consider it is single instance and extend 1.5 to the down with first half" | One whole pattern per pair, with `color`, `height`, and `period`: 4 | The four rules above and `$Obj.resume_row = $Obj.height@first % $Obj.period` |
| 3 | The user's request, after version 2's run, to simplify | As in version 2: 4 | The four rules above |

Version 1's rules found the period with the `all` and `any` quantifiers of
[NUC 62](../../docs/nuc-0062.md), whose conditions are evaluated in place and
so can read the filter callback's `start` parameter, which an arrow callback
nested in another cannot:

```nuc
$Obj.period = min of Obj.filter(start => start.y > 0 and all point in Obj: (point.y + start.y >= point.canvas_height or any twin in Obj: twin.x == point.x and twin.y == point.y + start.y) and (point.y < start.y or any twin in Obj: twin.x == point.x and twin.y == point.y - start.y)).map(start => start.y)
$Obj.output_height = $Obj.canvas_height * 3 / 2
$Obj.tag = 'l2235'
$Obj.repeat_direction = 'down'
$Obj.repeat_count = Math.floor(($Obj.output_height - 1 - $Obj.y) / $Obj.period) + 1
$Obj.color = 2
```

### Gestalt review

On 5-Nov-2026 the dataset was checked against the gestalt requirement, which
`/arc-purposal` and the arc-dataset profile added on 4-Nov-2026, after this
dataset was written: start from the big picture, the figures each grid shows,
and go down to single cells only when that cannot express what the training
pairs show. Version 3 meets it. Each training input shows one pattern whose
cells all touch, and that whole pattern is the one object, measured by its
`height` and `period`. Version 1's one point per cell was the pixel-level
start that the requirement now rules out, and the user's hint for version 2
moved the model to the whole pattern. Nothing needs to change.

### Standard entries

On 5-Nov-2026, at the user's request, every ARC dataset moved to the standard
object entries `x`, `y`, `width`, `height`, and `orientation`. `width` and
`height` are the column and row counts of the object's input bounding box. An
orientation change alone leaves them unchanged, and a rule changes them only
when the shape itself changes size, reading the input with `@first`. Every
implementation now passes the input box's `width` and `height` after `y`, and
the class and rules read:

```nuc
class Obj(x: int, y: int, width: int, height: int, orientation: int, color: int, period: int):
$Obj.tag = 'l8384'
$Obj.extension_direction = 'down'
$Obj.height = $Obj.height@first * 3 / 2
$Obj.color = 2
```

No rule changed: the existing `$Obj.height = $Obj.height@first * 3 / 2`
already grows the pattern the standard way, and the class's `height` moved
into the standard position. The migration worker regenerated every snapshot
with the accepted runtime at `491512d`, and the caller replayed every pair
offline against the unchanged grids, masks, positions and instance names. No
solver run has measured this version yet.

## Runs

| Date | Dataset version | Runtime | Exit status | Difference | Elapsed |
| --- | --- | --- | --- | --- | --- |
| 4-Nov-2026 10:18 UTC | 1, one point per nonzero cell | Accepted `a4d4c4a` | 2 | 2 | 7m 43.4s |
| 4-Nov-2026 12:30 UTC | 2, one whole pattern per pair | Accepted `a4d4c4a` | 0 | 0 | 1m 30.5s |

Each run was `bun start 017c7c7b` from `arc/`, run once by its own
`/arc-purposal` invocation with a 30-minute limit and `NUCLEOID_EXECUTABLE`
unset. Both used the primary checkout's debug runtime (`nucleoid 1.0.0`),
which `cargo build` had built at accepted `a4d4c4a` (NUC 78). That binary
produced both versions' snapshots, and its hash and the dataset's were the
same before and after each run. The exit status is the difference score, and
neither run's log records an error. Elapsed is the solver's own report.
Version 3 has not been run: no `/arc-purposal` invocation has run it, so its
difference is unknown, not zero.

A solver log for this puzzle, dated 4-Nov-2026 08:47 UTC, predates both
invocations and the dataset. It is not recorded here: its command, runtime,
and result are unknown, so the difference before this dataset is unknown, not
zero. The puzzle has not been in an **ARC puzzle report** workflow run.

## Open items

- **The runs saw the answer.** Resolved on 6-Nov-2026: the user decided this
  is intended, because the datasets are being prepared as few-shot examples
  and runs at this stage are not yet held-out tests. The ARC instructions now
  say so, and the suggested change to keep the running puzzle's test pairs out
  of `document(puzzle_id)` is withdrawn. `src/lib/llm.ts` sends
  `document(puzzle_id)` as few-shot context, and `src/dataset/dataset.ts`
  discards `puzzle_id`. Both runs' prompts therefore held this puzzle's
  annotated test pair: its expected output and, in version 2, the test
  instance's implementation with its `period`. Version 2's exact match does
  not show that the rules were inferred from the training pairs or that the
  period was measured on the test pattern. Version 3 carries the same test
  implementation, so a run of it would too.
- **The period is an input property.** Since version 2, the rules no longer
  derive the period; the model supplies it for the test instance. Deriving it
  declaratively from one instance per pair would need a list property the
  user approves, such as the pattern's rows; version 1 shows the derivation
  over point instances. The worker notes, untested, that accepted Nucleoid has
  no `range` builtin and that list callbacks receive only the item
  ([NUC 61](../../docs/nuc-0061.md)), so deriving `period` and `height` from a
  rows list would probably need a language feature. Decide whether the
  trade-off stands.
- **`extension_direction`.** The worker found one more candidate under the
  new requirement: with `y` unchanged while `height` grows, the output state
  already implies growth downward, and `l8384` names rows repeating down. Its
  comment is the only explicit statement that the top row stays in place, and
  `dataset.line.json` uses the same property, so the worker suggests keeping
  it. Decide.
- **Messages are not JSON-only.** `src/lib/llm.ts`, `src/analyzer.ts`, and
  `src/visualizer.ts` wrap JSON in text labels such as `few_shot:`,
  `train_dataset:`, and `input_x:`, while the ARC instructions require every
  message to parse as one JSON document. Suggested, not applied: build each
  message with `JSON.stringify` of one data object.
- **Comments in declarations.** Resolved on 6-Nov-2026: the user decided that
  natural-language `#` comments may reach the model only as part of
  `declarations`, and the ARC instructions now say so. Datasets with and
  without such comments both follow them, so nothing needs to change.
- **README text.** `arc/README.md`'s shared sentence that `EXACT` outputs
  keep the input matrix's dimensions does not hold for this dataset, whose
  6 by 3 inputs grow to 9 by 3 outputs; its paragraph explains the growth.
  The README also still describes `dataset.0d3d703e.json` and
  `dataset.27a28665.json`, which do not exist; `dataset.44f52bb0.json`
  arrived with commit `bca5ff6`.
- **Canvas height, resolved for this dataset.** Version 1's `canvas_height`
  needed the test canvas's height, which `value()` in `src/visualizer.ts`
  does not send. Versions 2 and 3 measure `height` on the pattern's own box,
  so this dataset no longer depends on it. The suggestion to pass the test
  input's row and column counts to `visualizer.value` is superseded: since
  4-Nov-2026 every `nucleoid.run` call declares `canvas_width` and
  `canvas_height` at the start of its program, and the `d037b0a7`,
  `6150a2bd`, and `44f52bb0` datasets read them there.
- **Run version 3.** Measuring it takes a new `/arc-purposal 017c7c7b`
  invocation, which authorizes its one run.
- **Committed.** `a5a793d` committed the dataset, its README paragraph, the
  workflow entry, and this record. The solver's default puzzle in
  `src/app.ts` is no longer this puzzle: commit `bca5ff6`, made after both
  runs, set it to `44f52bb0`, and the pull that brought it in kept that later
  value.

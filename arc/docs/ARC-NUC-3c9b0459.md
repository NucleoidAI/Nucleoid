# ARC-NUC-3c9b0459 -- Mosaic Half Turn

Progress record for ARC training puzzle `3c9b0459`: its instruction dataset,
the solver and Nucleoid changes it needed, its runs, and what is still open.

| Item | Status |
| --- | --- |
| Puzzle | [`data/training/3c9b0459.json`](../data/training/3c9b0459.json) |
| Instruction dataset | [`dataset.3c9b0459.json`](../src/dataset/dataset.3c9b0459.json): the puzzle's 4 original training pairs and its test pair, 5 mosaic instances, one per pair; committed with this record; shared orientation comment added on 6-Nov-2026; descriptive `implementation` comments added on 7-Nov-2026; colors written as letters on 8-Nov-2026; respelled on 8-Nov-2026 with the `is` assignments of NUC 93 and bare words, every snapshot unchanged; colors written as numbers again on 8-Nov-2026 |
| Output canvas | `EXACT` |
| Nucleoid enhancements | None requested, so no proposal strategy was selected; the rule uses the class-level `@first` anchor of [NUC 55](../../docs/nuc-0055.md), built on the state versions of [NUC 53](../../docs/nuc-0053.md) |
| Solver changes | None; suggested changes await review, not applied; since 8-Nov-2026 every respelled dataset needs `is` assignments, which `src/lib/nucleoid.ts` enabled with `--is-assignment` until `d945213` made them the default |
| Solver runs | 1 local run on 4-Nov-2026, exit 1 before any model call (`OPENROUTER_API_KEY is required`); difference unknown; none of the current version; none of the 8-Nov-2026 `is` version |
| Updated | 8-Nov-2026 |

## Puzzle

Each training input is a 3 by 3 canvas filled by one mosaic: every cell holds
a color, and none holds the background color 0. The four mosaics use two or
three colors each: 1, 2, and 8; 2, 4, and 9; 5 and 8; and 2, 3, and 9. The
output keeps the canvas and turns the whole mosaic a half turn, 180 degrees,
so the cell in row r and column c moves to row 2 - r and column 2 - c, and
every color keeps its cell count. Of the square's eight turns and mirrors,
only the half turn maps each training input to its output, and no training
mosaic looks the same under any of the other seven, so each pair confirms the
half turn on its own. The output keeps the input's 3 by 3 canvas, so the
dataset is `EXACT`; with no zero cell to crop, `FIT` would frame the same
grids.

`/arc-purposal` chose this puzzle by default: among the 386 training puzzles
other than the genesis puzzle `3aa6fb7a` and those with a dataset, its largest
training matrix has the fewest cells, 9, with 72 cells in all its training
matrices. Four other puzzles, `74dd1130`, `9565186b`, `a85d4709`, and
`ed36ccf7`, tie on both measures, and `3c9b0459` has the lowest ID among them.

## Gestalt overview

- **Whole grid.** A 3 by 3 canvas filled by one mosaic of two or three
  colors, with no background cell.
- **Figures.** One figure per grid, the whole mosaic. Its color regions are
  parts of it, not figures that act on their own.
- **Relations.** The mosaic relates only to the canvas center. No training
  mosaic looks the same under another turn or mirror of the square, so each
  pair shows the half turn on its own.
- **Change.** The whole mosaic turns a half turn about the canvas center, and
  every color keeps its cell count.
- **World model.** One object per pair, the whole mosaic, whose turn is its
  orientation.

## Dataset

The dataset holds the four original training pairs in source order, followed
at the same level by the puzzle's test pair, which the same rule annotates.
Every mosaic is one instance of `Obj(x, y, orientation)`, tagged `l9100`, a
single shape, as in `dataset.28bf18c6.json`, `dataset.6150a2bd.json`, and
`dataset.25ff71a9.json`. The class takes no color, size, cell list, canvas
size, or matrix of cells: the mosaic's cells travel unchanged in its masks,
and its turn is its orientation. One unconditional class-level rule turns
every mosaic:

```nuc
$Obj.orientation = ($Obj.orientation@first + 180) % 360
```

It is the half-turn rule of `dataset.6150a2bd.json`. `@first` reads the
mosaic's orientation from before the rules ran, which the rule needs because
it rewrites `orientation` itself. The rule creates no property and leaves `x`,
`y`, and the tag unchanged: a mosaic that fills its canvas keeps its box at 0
and 0 under a half turn. Each input state has orientation 0 and each output
state 180, and they differ in nothing else. The declarations carry no `#`
comments, as in `dataset.25d8a9c8.json` and `dataset.25ff71a9.json`. The
[README](../README.md#instruction-datasets) describes the dataset in full.

The worker started from the big picture, the whole mosaic as the object, and
that formulation explained every training pair, so it never went down to
single cells. `dataset.6150a2bd.json` turns its shapes with the same rule and
also moves each box about the canvas center through `width` and `height`
properties. That fuller rule set reproduces every training output here too,
but every mosaic's width and height equal the canvas's, so those properties
would only restate the canvas size and its position rules would only return
the origin. The worker kept the half turn alone, a choice fixed from the
training pairs before it read the test pair. Each training grid holds one
mosaic and relates it to nothing else, so the rule needs no population
operation such as `filter` or `some`; the class-level rule already states the
turn once for every mosaic.

The caller checked the handoff against the declarative, higher-order, and
gestalt requirements, found that it met them, and sent nothing back. The
caller also replayed the four training pairs on the accepted runtime, 66
checks of grids, keys, mask shapes and cells, bounding boxes, merges,
orientations, and input and output snapshots, all passing. The worker froze
the rules from the training pairs before it read the test pair and reported
that the same rules reproduce it. It wrote the dataset, its README paragraph,
and the puzzle's entry in the **ARC puzzle report** workflow's list, its 15th
puzzle. The caller made the puzzle the solver's default in `src/app.ts`, in
place of `25ff71a9`, before the run.

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
$Obj.orientation = ($Obj.orientation@first + 180) % 360
```

No rule changed: the half turn stays in `orientation`, so the size stays. The
migration worker regenerated every snapshot with the accepted runtime at
`491512d`, and the caller replayed every pair offline against the unchanged
grids, masks, positions and instance names. No solver run has measured this
version yet.

### Orientation convention

On 6-Nov-2026 the user asked `/arc-purposal` to make `orientation` consistent
across all 29 instruction datasets, with a reference of eight directed 3-cell
lines, each of which points toward its head. Every state now takes the
direction its own shape points, by the convention in the
[README](../README.md#3-extract-instances): a line toward its head or along
its axis, a corner or chevron toward its vertex, and anything else 0, while a
turn adds its angle. Every dataset's first declaration also carries one shared
comment line, directly above its class, that states the convention for the
model:

```nuc
# Orientation is the direction an object points, in degrees clockwise from north (0 up, 90 right, 180 down, 270 left)
```

This dataset's labels already followed the convention, as the labeller
confirmed: each mosaic has no direction, 0, and the half-turn rule adds 180,
so every output is 180. Only the comment line changed, so the class, the
rules, the grids, masks, positions, instance names, implementations, and
snapshots are as before, and the gestalt overview above still holds.

The arc-dataset worker made the change in an external staging directory with
the accepted runtime at `d3413e6` (last runtime change `7096171`), and every
snapshot reruns byte for byte on it. The caller's own offline check passed on
the staged file and again after integration: the pairs, grids, masks,
positions, and instance names are as before, every state's orientation matches
the convention's labeller, every snapshot replays on that runtime, and
`document()` loads the dataset with the other 28. No NUC was requested, so no
proposal strategy was selected and strategy learning did not apply. At the
user's choice, no solver run has measured this version.

### Letter colors

On 8-Nov-2026, at the user's request, every instruction dataset started
writing its colors as letters instead of numbers. `0` stays `0`, and every
other color is a letter of `ABCDEFGHI`, ranked by how often it occurs across
all of the dataset's input and output matrices, training and test pairs
together: `A` for the most used, then `B`, and so on, a tie going to the color
read first, as `Matrix.silhouette` ranks one matrix. This dataset writes its
colors as letters ranked by count: 2 as `A`, 4 as `B`, 8 as `C`, 9 as `D`, 5
as `E`, 6 as `F`, 1 as `G`, 3 as `H`, and 7 as `I`. Mapping the letters back
reproduces the puzzle's pairs cell for cell.

The class and rules hold no color, so only the pair grids and both instance
masks changed; the declarations and implementations are unchanged.

Every snapshot was regenerated with the accepted runtime at `adc282a` and
equals its earlier snapshot byte for byte, so the rules behave as before;
instance names, positions, sizes, orientations, and tags are unchanged. `Matrix.silhouette` now also takes letters, so the
orientation call's few-shot crops read exactly as before. No solver run has
measured this version, so its difference is unknown, not zero.

### Is assignments

On 8-Nov-2026 the user asked to apply [NUC 93](../../docs/nuc-0093.md) and
[NUC 86](../../docs/nuc-0086.md) to every dataset in `src/dataset`. Every
assignment in the declarations and implementations now uses `is`, the opt-in
assignment of NUC 93, and a quoted word that was a whole value, or the whole
right operand of an `is` comparison, lost its quotes:

```nuc
class Obj(x: int, y: int, width: int, height: int, orientation: int):
    this.x is x
    this.y is y
    this.width is width
    this.height is height
    this.orientation is orientation
    this.tag is l9100

$Obj.orientation is ($Obj.orientation@first + 180) % 360
```

`target is value` builds exactly the syntax tree of `target = value`. A bare
word reads the name of its spelling when one exists and otherwise stores its
own text: in the constructor, `this.x is x` reads the parameter `x`, and no
program of this dataset defines `l9100`, so that word reads as its own text.
Implementations construct their instances with `is` as well, as in
`objad2abfd1734f is Obj(x=0, y=0, width=3, height=3, orientation=0)`. Only the
spelling changed: the class, the rules' structure, the grids, masks,
positions, instance names, and snapshots are as before, and the world model
keeps its level, so the gestalt overview above still holds.

The rules now need a runtime at `ad45ba4`, the NUC 93 commit, or later, run
with `--is-assignment`, which `src/lib/nucleoid.ts` now passes to every
program; without the flag, `objad2abfd1734f is Obj(...)` reads as three
juxtaposed statements and raises `ReferenceError`. Offline, with the accepted
runtime at `4ac649b` (last runtime change `ad45ba4`), the flag reproduced all
5 input and 5 output snapshots of the 5 pairs, both in a direct replay and
through the solver's own `nucleoid.run`, and the previous version reproduced
them with and without the flag. No NUC was requested, so no proposal strategy
was selected and strategy learning did not apply. No solver run has used this
version.

### Number colors

On 8-Nov-2026, at the user's request, the letter colors were reverted, and the
dataset writes every color as its number again, as the puzzle's pairs hold it,
turning `A` back into 2, `B` into 4, `C` into 8, `D` into 9, `E` into 5, `F`
into 6, `G` into 1, `H` into 3, and `I` into 7. The pair grids and both
instance masks hold numbers. The declarations and implementations name no
color, so they and the snapshots are unchanged.

The `is` assignments and bare words of the previous section are unchanged.
Offline, with the accepted runtime at `1a41ca7` (last runtime change
`f463d94`), all 5 input and 5 output snapshots of the 5 pairs replay
unchanged. The pair grids equal the puzzle's pairs cell for cell, and the
grids, masks, and snapshots are again those of the version before the letters.
`Matrix.silhouette` again takes only the colors 0 to 9, and the orientation
calls' few-shot crops read exactly as they did with letters, because the
counts that rank them are unchanged. No NUC was requested, so no proposal
strategy was selected and strategy learning did not apply. No solver run has
measured this version, so its difference is unknown, not zero.

## Runs

| Date | Dataset version | Runtime | Exit status | Difference | Elapsed |
| --- | --- | --- | --- | --- | --- |
| 4-Nov-2026 21:35 UTC | 1, one mosaic per object | Accepted `c3aa076`, local | 1 | Unknown: `Error: OPENROUTER_API_KEY is required` | 0.5s |

The run was `bun start 3c9b0459` from `arc/`, run once by `/arc-purposal` with
a 30-minute limit and `NUCLEOID_EXECUTABLE` unset, after
`bun install --frozen-lockfile` reported no changes. It used the primary
checkout's debug runtime (`nucleoid 1.0.0`), built with `cargo build` at
`c3aa076`, and the committed solver at `c3aa076` with only its default puzzle
changed. It stopped in the analyzer's first model call,
`analyzer.declarations`, where `src/lib/openrouter.ts` raised
`OPENROUTER_API_KEY is required`, before any model request, prediction, or
score. Its difference is therefore unknown, not zero, and the solver wrote no
`logs/3c9b0459.log.json`. The solver reported no elapsed time, so elapsed is
the process's run from 21:34:59.9 to 21:35:00.4 UTC. The hashes of the
runtime, the dataset, and `src/app.ts` were the same before and after the run.
The invocation authorizes one run, so none was repeated. The puzzle has not
been in an **ARC puzzle report** workflow run.

None of these runs measured the orientation version of 6-Nov-2026: the user
chose offline verification for the batch, so its difference is unknown, not
zero.

## Open items

- **No model credential.** As in the runs of [25d8a9c8](ARC-NUC-25d8a9c8.md)
  and [25ff71a9](ARC-NUC-25ff71a9.md), the solver process had no
  `OPENROUTER_API_KEY` value, so it stopped before its first model request.
  Measuring this dataset takes a separately authorized run with the
  credential available to the solver.
- **A run would see the answer.** Resolved on 6-Nov-2026: the user decided
  this is intended, because the datasets are being prepared as few-shot
  examples and runs at this stage are not yet held-out tests. The ARC
  instructions now say so, and the suggested change to keep the running
  puzzle's test pairs out of `document(puzzle_id)` is withdrawn.
  `src/lib/llm.ts` sends `dataset.document(puzzle_id)` as few-shot context,
  and `src/dataset/dataset.ts` discards `puzzle_id`. Any run of this puzzle
  therefore holds this dataset's annotated test pair, expected output
  included, in its prompts, and an exact match would not show that the rule
  was inferred from the training pairs.
- **Messages are not JSON-only.** `src/lib/llm.ts`, `src/analyzer.ts`, and
  `src/visualizer.ts` wrap JSON in text labels such as `few_shot:`,
  `control_id:`, `train_dataset:`, and `json_format:`, while the ARC
  instructions require every message to parse as one JSON document.
  Suggested, not applied: build each message with `JSON.stringify` of one
  data object.
- **Lint fails on untouched solver files.** At `c3aa076`, `bun run lint`
  reports 28 `prettier/prettier` errors, 4 in `src/analyzer.ts` and 24 in
  `src/visualizer.ts`, files this invocation did not change, while
  `bun run typecheck` passes. Suggested, not applied: reformat both files
  with Prettier, for example `bunx eslint --fix src/analyzer.ts
  src/visualizer.ts`, a formatting-only change.
- **Comments in declarations.** Resolved on 6-Nov-2026: the user decided that
  natural-language `#` comments may reach the model only as part of
  `declarations`, and the ARC instructions now say so. Datasets with and
  without such comments both follow them, so nothing needs to change. On
  7-Nov-2026 the user also allowed them in `implementation`, and each
  implementation in this dataset now opens with comment lines that describe
  its object.
- **Workflow timeout.** Resolved on 6-Nov-2026: at the user's request, the
  arc-dataset profile now asks for the batch budget the workflow needs,
  `ceil(listed puzzles / ARC_CONCURRENCY) * 30 + 20` minutes, within GitHub
  Actions' 360-minute job limit. The workflow's 150-minute timeout covers its
  20 listed puzzles.
- **Orientation version unmeasured.** No solver run has measured the
  6-Nov-2026 orientation version, so whether consistent orientation labels
  change this puzzle's result is unknown.
- **Letter version withdrawn.** No solver run measured the 8-Nov-2026 letter
  version before the user reverted it the same day. The dataset writes its
  colors as numbers again, as the solver sends this puzzle's own grids, and
  no solver run has measured that version either, so its difference is
  unknown.
- **The `is` assignment version is unmeasured.** No solver run has used the
  8-Nov-2026 version with `is` assignments and bare words, so its effect on
  this puzzle's result is unknown. Every run's few-shot context now shows bare
  words beside quoted strings, and a word reads as its text only as a whole
  value: a model that writes one inside a larger value, such as
  `$Obj.fill_color is A if $Obj.symmetric else C`, gets a `ReferenceError`.

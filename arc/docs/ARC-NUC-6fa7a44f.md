# ARC-NUC-6fa7a44f -- Block Over Its Mirror Image

Progress record for ARC training puzzle `6fa7a44f`: its instruction dataset,
the solver and Nucleoid changes it needed, its runs, and what is still open.

| Item | Status |
| --- | --- |
| Puzzle | [`data/training/6fa7a44f.json`](../data/training/6fa7a44f.json) |
| Instruction dataset | [`dataset.6fa7a44f.json`](../src/dataset/dataset.6fa7a44f.json): the puzzle's 4 original training pairs and its test pair, 5 block instances, 1 in each pair; added in `f7ec134` with SHA-256 letters; colors relettered by count on 8-Nov-2026 in `af795b3`; respelled on 8-Nov-2026 with the `is` assignments of NUC 93 and bare words, every snapshot unchanged; colors written as numbers instead of letters on 8-Nov-2026 |
| Output canvas | `EXACT` |
| Nucleoid enhancements | None requested, so no proposal strategy was selected and strategy learning did not apply; the height rule uses the class-level `@first` anchor of [NUC 55](../../docs/nuc-0055.md), built on the state versions of [NUC 53](../../docs/nuc-0053.md) |
| Solver changes | None applied or suggested; the JSON-only message question carried over from earlier records stays open; since 8-Nov-2026 every respelled dataset needs `is` assignments, which `src/lib/nucleoid.ts` enabled with `--is-assignment` until `d945213` made them the default |
| Solver runs | 1 local run on 8-Nov-2026 of the first, SHA-256 letter version: exit 0, difference 0; none of the current count-ranked version; none of the 8-Nov-2026 `is` version |
| Updated | 8-Nov-2026 |

## Puzzle

Each training input is a 3 by 3 canvas with no zero cell. The whole canvas
is one block of three or four colors: 9, 1, 4, and 2; 4, 8, 7, and 6; 7, 9,
5, and 1; and 2, 6, and 9 in the four training pairs. Each output is a 6 by 3
canvas, twice as tall, that stacks the block over its top-bottom mirror
image. The input reappears unchanged in the top three rows, and its row r
reappears as row 5 - r, so every output reads the same from the bottom up as
from the top down. The block fills its canvas, so its own bottom edge is the
canvas's bottom edge and the line it is mirrored across. The output keeps
the input's 3 columns and its origin and grows only downward, so the dataset
is `EXACT`.

`/arc-purposal` chose this puzzle by default. Among the 373 training puzzles
other than the genesis puzzle `3aa6fb7a` and those with a dataset, its
largest training matrix has the fewest cells, 18, with 108 cells in all its
training matrices. The next two, `dae9d2b5` and `fafffa47`, each have 18 and
135. The puzzle is the top-bottom counterpart of
[ARC-NUC-6d0aefbc](ARC-NUC-6d0aefbc.md), which sets its block beside its
left-right mirror image and tied with this puzzle when it was chosen. It
makes the change of [ARC-NUC-8be77c9e](ARC-NUC-8be77c9e.md), which stacks a
one-color pattern over its mirror image, on a canvas the block fills.

## Gestalt overview

- **Whole grid.** A small 3 by 3 canvas filled edge to edge with three or
  four colors. The output is the same grid on top and its reflection below,
  a 6 by 3 canvas that is symmetric about its horizontal center line.
- **Figures.** The whole block is the figure seen first, one per grid: no
  cell is zero, so the block is the canvas.
- **Relations.** The block fills its canvas, so its bottom edge is the
  canvas's bottom edge, the line it is mirrored across. Each grid holds one
  block, so no relation between objects arises.
- **Change.** The canvas doubles downward, and the block is joined by its
  reflection across its bottom edge, keeping its colors, its columns, and
  its place.
- **World model.** One object per grid, the whole block, the figure seen
  first. Its box doubles in height to hold the block and its reflection, so
  nothing finer was needed. Before it read the test pair, the worker set
  aside four alternatives:
  - a plain copy placed below, which reproduces no training output, because
    no training block is top-bottom symmetric;
  - a half turn placed below, which reproduces only the second training pair,
    whose rows each read the same from right to left;
  - a left-right mirror image placed below;
  - a mirror image placed above.

  Rows or color regions would be finer than the training pairs need.

## Dataset

The single dataset holds the four original training pairs in source order,
followed at the same level by the puzzle's test pair, which the same rules
annotate. Each object is one whole block: an instance of
`Obj(x, y, width, height, orientation)` with only the standard entries, whose
initial tag `l9100` is the single shape. Its declaration is that of
`dataset.6d0aefbc.json`. The class takes no descriptive property: the block's
cells and colors travel in its masks, so there is no color, parts list,
canvas size, or matrix of cells. Two unconditional class-level rules turn
every block into the figure it forms with its mirror image:

```nuc
class Obj(x: int, y: int, width: int, height: int, orientation: int):
$Obj.tag = 'l7511'
$Obj.height = $Obj.height@first * 2
```

`l7511`, the tag of `dataset.8be77c9e.json`, is a shape over its top-bottom
mirror image, symmetric about the horizontal center line of its box. `height`
doubles from the block's input height, read with `@first`, so the figure is
6 rows tall and its center line is the block's bottom edge. `x`, `y`,
`width`, and `orientation` stay as they were. The figure keeps the block's
top-left corner and columns, and the reflection is part of the figure that
`l7511` names, not a turn. So orientation is 0 in both states, as for any
block. The rules create no property and state no mirror axis, because the
tag and the doubled height already determine it. The tag is not redundant:
doubling the height alone does not tell a mirror image from a copy or a half
turn. Each grid holds a single block and shows no relation between objects,
so no population operation appears. Like every dataset since `ac49bb3`, the
declarations and implementations carry no `#` comments, and each instance
has the six keys the datasets have shared since `d0e3eed`. The
[README](../README.md#instruction-datasets) describes the dataset in full.

The training pairs cannot tell doubling the block's own height from the
canvas-edge rule of `dataset.8be77c9e.json`,
`$Obj.height = 2 * (canvas_height - $Obj.y@first)`. Every block fills its
canvas, so both give 6 rows in every pair. The worker took the doubling
rule, which reads only the block, as `dataset.6d0aefbc.json` doubles its
block's width.

The dataset writes its colors as letters ranked by count across all its pair
grids, training and test pairs together: `A` for the most used color, then
`B`, and so on, with a tie going to the color read first. That gives 2 as
`A`, 9 as `B`, 7 as `C`, 1 as `D`, 8 as `E`, 4 as `F`, 5 as `G`, and 6 as
`H`. Mapping the letters back reproduces the puzzle's pairs cell for cell.
The dataset was first committed with letters from a SHA-256 shuffle; see
[Letters ranked by count](#letters-ranked-by-count) below.

For the first version, the caller passed the arc-dataset worker these
requirements:

- the declarative, higher-order, gestalt, and letter requirements;
- the orientation convention and the standard entries;
- the six instance keys and the comment-free format, which the arc-dataset
  profile does not reflect yet.

It checked the handoff against those requirements and found that it met
them, so it sent nothing back. It then recomputed the SHA-256 shuffle then
in force from the dataset's key and ran its own offline check on the accepted
runtime, printing
only each check's name and result. The letter checks covered the whole file;
every other check read the training pairs alone. All 24 checks passed:

- the file parses, with the datasets' top-level, pair, and instance keys in
  their order, LF line endings, no byte-order mark, and no `#` comment;
- every grid and mask cell is `0` or a letter, the letters are the first
  letters of the shuffle, and the training colors take them in increasing
  order;
- the 4 training pairs keep their original grids, in order, once the letters
  are mapped back;
- every mask keeps its full canvas, 3 by 3 for inputs and 6 by 3 for
  outputs, holds only its grid's cells, and matches its implementation's and
  snapshot's box, and the masks merge back into each grid;
- every input and output snapshot reruns exactly, also with the rules
  reversed and declared after the instance;
- the instance names are unique across all 31 JSON datasets.

The worker's own offline checks, which also cover the test pair, all passed.
The rules reproduce every output. `document()` includes the dataset as the
14th of the 31 JSON instruction datasets, in filename order, and with
`--only-puzzle-dataset` it loads the four genesis datasets and this one.

The worker wrote three things:

- the dataset;
- its README paragraph, between the `dataset.6e02f1e3.json` and
  `dataset.74dd1130.json` paragraphs;
- the puzzle's entry at the end of the **ARC Benchmark** workflow's list, its
  28th puzzle.

The caller made the puzzle the solver's default in `src/app.ts`, in place of
`d631b094`, before the run.

While the worker wrote the dataset, another session left uncommitted changes
in the primary checkout: a revision of `dataset.74dd1130.json`, its progress
record, and its README paragraph. Asked before the run, the user chose to run
with that revision in the few-shot context. By the time the run started, the
dataset and its record were back at their committed versions. So the run's
few-shot context held the 30 committed datasets and this one, and every
dataset's hash was the same before and after the run. That session's
uncommitted README paragraph is not sent to the model, and this record's
commit leaves it out.

### Letters ranked by count

Later on 8-Nov-2026 the user reverted the SHA-256 letter rule (`cea73a4` and
`cfc11db`) to letters ranked by count, the rule every other dataset uses
again, and asked for this dataset to be reworked to match. The revert also
restored `Matrix.silhouette` in `src/lib/Matrix.ts` to accept only `0` to `9`
and `A` to `I`. The orientation calls of `analyzer.value` and
`visualizer.value` pass every dataset's crops through it. So the first
version's letters, such as `M`, `J`, `Y`, and `Q`, made those calls throw in
every solver run, whatever its puzzle.

A fresh arc-dataset worker, explicitly authorized to overwrite this dataset,
relettered it from the puzzle's original colors, not from the old letters:

| Color | SHA-256 letter | Ranked letter |
| --- | --- | --- |
| 2 | `B` | `A` |
| 9 | `H` | `B` |
| 7 | `E` | `C` |
| 1 | `M` | `D` |
| 8 | `J` | `E` |
| 4 | `C` | `F` |
| 5 | `Y` | `G` |
| 6 | `Q` | `H` |

Three ties went to the color read first: 9 before 7, 1 before 8, and 4
before 5. Only the letters of the pair grids and both instance masks
changed, 270 cells in all. The declarations, output canvas, instance names,
and implementations are unchanged. Every regenerated snapshot equals its
committed one byte for byte, because the class and rules hold no color.

The worker's checks all passed:

- the letters map back to the original grids;
- the recount gives the ranked letters;
- every crop passes `Matrix.silhouette`, where all ten of the first
  version's crops throw;
- `document()` loads all 31 datasets;
- the instance names are unique.

It also changed the README paragraph's last sentence to the ranked form.

The user committed the worker's output as `af795b3`, "Rank ARC dataset
6fa7a44f colors by frequency", while the caller was still reviewing it. The
caller then reran its own check, adapted to the ranked rule, on the committed
file, and all 24 checks passed. As before, the letter checks covered the
whole file and every other check read the training pairs alone. The gestalt
overview and the world model are unchanged. At the user's choice, no solver
run has measured this version.

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

$Obj.tag is l7511

$Obj.height is $Obj.height@first * 2
```

`target is value` builds exactly the syntax tree of `target = value`. A bare
word reads the name of its spelling when one exists and otherwise stores its
own text: in the constructor, `this.x is x` reads the parameter `x`, and no
program of this dataset defines any of `l9100` and `l7511`, so those words
read as their own text. Implementations construct their instances with `is` as
well, as in
`obj54b166493c46 is Obj(x=0, y=0, width=3, height=3, orientation=0)`. Only the
spelling changed: the class, the rules' structure, the grids, masks,
positions, instance names, and snapshots are as before, and the world model
keeps its level, so the gestalt overview above still holds.

The rules now need a runtime at `ad45ba4`, the NUC 93 commit, or later, run
with `--is-assignment`, which `src/lib/nucleoid.ts` now passes to every
program; without the flag, `obj54b166493c46 is Obj(...)` reads as three
juxtaposed statements and raises `ReferenceError`. Offline, with the accepted
runtime at `4ac649b` (last runtime change `ad45ba4`), the flag reproduced all
5 input and 5 output snapshots of the 5 pairs, both in a direct replay and
through the solver's own `nucleoid.run`, and the previous version reproduced
them with and without the flag. No NUC was requested, so no proposal strategy
was selected and strategy learning did not apply. No solver run has used this
version.

### Number colors

On 8-Nov-2026, at the user's request, the letter colors were reverted, and the
dataset writes every color as its number, as the puzzle's pairs hold it,
turning `A` into 2, `B` into 9, `C` into 7, `D` into 1, `E` into 8, `F` into
4, `G` into 5, and `H` into 6. The pair grids and both instance masks hold
numbers. The declarations and implementations name no color, so they and the
snapshots are unchanged.

The `is` assignments and bare words of the previous section are unchanged.
Offline, with the accepted runtime at `1a41ca7` (last runtime change
`f463d94`), all 5 input and 5 output snapshots of the 5 pairs replay
unchanged. The pair grids equal the puzzle's pairs cell for cell.
`Matrix.silhouette` again takes only the colors 0 to 9, and the orientation
calls' few-shot crops read exactly as they did with letters, because the
counts that rank them are unchanged. No NUC was requested, so no proposal
strategy was selected and strategy learning did not apply. No solver run has
measured this version, so its difference is unknown, not zero.

## Runs

| Date | Command | Dataset version | Runtime | Exit status | Difference | Elapsed |
| --- | --- | --- | --- | --- | --- | --- |
| 8-Nov-2026 13:39 UTC | `bun start 6fa7a44f` | 1, SHA-256 letters | Accepted runtime at `11443aa` (last runtime change `34bbba0`), local | 0 | 0 | 2m 48.7s |

`/arc-purposal` ran `bun start 6fa7a44f` once from `arc/`, with a 30-minute
limit and `NUCLEOID_EXECUTABLE` unset, after `bun install --frozen-lockfile`
reported no changes. The setup was:

- **Runtime:** the primary checkout's debug runtime (`nucleoid 1.0.0`),
  current with `cargo build` at `11443aa`. Its sources last changed in
  `34bbba0`, "NUC 90 Enum Declarations".
- **Solver:** the one committed at `11443aa`, with only its default puzzle
  changed.
- **Model:** the committed `qwen/qwen3.8-27b`, pinned to the `deepinfra/bf16`
  provider with fallbacks and caching off. Each of the run's 17 model
  requests reported 0 cached input tokens.
- **Few-shot context:** `--only-puzzle-dataset` was off, so the calls that
  send few-shot examples held all 31 instruction datasets. This dataset's
  annotated test pair was among them, as the ARC instructions intend while
  the datasets are prepared as few-shot examples.

The run ended in an exact match, exit status 0 and difference 0, in 2m 48.7s
by the solver's own report. The runner recorded the process from 13:39:29.1
to 13:42:18.4 UTC. Its error log holds only bun's command line. The run wrote
`logs/6fa7a44f.log.json`, which Git ignores, as it ended. The runtime's hash
and every instruction dataset's hash were the same before and after the run.
No commit reached the primary checkout or the remote during it. The
invocation authorizes one run, so none was repeated.

`bun run typecheck` and `bun run lint` passed with the new default after the
run. The puzzle joined the **ARC Benchmark** workflow with this dataset, so
no workflow run is recorded yet.

No run has measured the count-ranked version of 8-Nov-2026. The user chose
offline verification for it, so its difference is unknown, not zero.

## Open items

- **Messages are not JSON-only.** Carried over from
  [ARC-NUC-6d0aefbc](ARC-NUC-6d0aefbc.md) and earlier records. The messages
  that `src/lib/llm.ts`, `src/analyzer.ts`, and `src/visualizer.ts` build
  wrap JSON in text labels such as `control_id:`, `few_shot_examples:`,
  `train_dataset:`, and `json_format:`, while the ARC instructions require
  every message to parse as one JSON document. It remains an open question
  for the user. This dataset does not depend on it.
- **The arc-dataset profile is out of date.** It still lists the six
  instance fields that `d0e3eed` removed (`input_x`, `output_x`, `input_y`,
  `output_y`, `input_orientation`, and `output_orientation`). It still asks
  for `#` comment lines in every implementation, which `ac49bb3` removed. It
  still labels symmetric or unrotated shapes 0, where the README's
  convention labels each shape by the direction it points. Parts of
  `arc/README.md` still describe the removed fields and the shared
  orientation comment line. The caller briefed the worker on all of these
  explicitly. Updating the profile and the README is the user's call.
- **Count-ranked version withdrawn.** No solver run measured the relettered
  version before the user reverted the letters on 8-Nov-2026. The dataset
  writes its colors as numbers, as the solver sends this puzzle's own grids,
  and no solver run has measured that version either, so its difference is
  unknown.
- **The `is` assignment version is unmeasured.** No solver run has used the
  8-Nov-2026 version with `is` assignments and bare words, so its effect on
  this puzzle's result is unknown. Every run's few-shot context now shows bare
  words beside quoted strings, and a word reads as its text only as a whole
  value: a model that writes one inside a larger value, such as
  `$Obj.fill_color is A if $Obj.symmetric else C`, gets a `ReferenceError`.

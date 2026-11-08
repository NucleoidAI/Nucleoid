# ARC-NUC-6d0aefbc -- Block Beside Its Mirror Image

Progress record for ARC training puzzle `6d0aefbc`: its instruction dataset,
the solver and Nucleoid changes it needed, its runs, and what is still open.

| Item | Status |
| --- | --- |
| Puzzle | [`data/training/6d0aefbc.json`](../data/training/6d0aefbc.json) |
| Instruction dataset | [`dataset.6d0aefbc.json`](../src/dataset/dataset.6d0aefbc.json): the puzzle's 4 original training pairs and its test pair, 5 block instances, 1 in each pair; committed with this record; colors written as letters on 8-Nov-2026 |
| Output canvas | `EXACT` |
| Nucleoid enhancements | None requested, so no proposal strategy was selected and strategy learning did not apply; the width rule uses the class-level `@first` anchor of [NUC 55](../../docs/nuc-0055.md), built on the state versions of [NUC 53](../../docs/nuc-0053.md) |
| Solver changes | None applied or suggested; the JSON-only message question carried over from earlier records stays open |
| Solver runs | 1 local run on 7-Nov-2026: exit 0, difference 0 |
| Updated | 8-Nov-2026 |

## Puzzle

Each training input is a 3 by 3 canvas with no zero cell, one block of
colors 1, 6, and 8, only 1 and 6 in the fourth pair. Each output is a 3 by 6
canvas, twice as wide, that sets the block beside its left-right mirror
image: the input reappears unchanged in the left three columns, and its
column c reappears as column 5 - c, so every output row reads the same from
right to left as from left to right. The block fills its canvas, so its own
right edge is the canvas's right edge and the line it is mirrored across. The
output keeps the input's 3 rows and its origin and grows only to the right,
so the dataset is `EXACT`.

`/arc-purposal` chose this puzzle by default: among the 374 training puzzles
other than the genesis puzzle `3aa6fb7a` and those with a dataset, its
largest training matrix has the fewest cells, 18, with 108 cells in all its
training matrices. `6fa7a44f` tied on both measurements and came after it by
ID; the next, `dae9d2b5`, has 18 and 135. The puzzle makes the change of
[ARC-NUC-c9e6f938](ARC-NUC-c9e6f938.md), which mirrors a one-color pattern
across the same edge, on a canvas the block fills, and it is the left-right
counterpart of [ARC-NUC-8be77c9e](ARC-NUC-8be77c9e.md), which stacks its grid
over its top-bottom mirror image.

## Gestalt overview

- **Whole grid.** A small 3 by 3 canvas filled edge to edge with colors 1,
  6, and 8; the output is the same grid on the left and its reflection on the
  right, a 3 by 6 canvas that is symmetric about its vertical center line.
- **Figures.** The whole block is the figure seen first, one per grid: no
  cell is zero, so the block is the canvas.
- **Relations.** The block fills its canvas, so its right edge is the
  canvas's right edge, the line it is mirrored across. Each grid holds one
  block, so no relation between objects arises.
- **Change.** The canvas doubles to the right, and the block is joined by its
  reflection across its right edge, keeping its colors, its rows, and its
  place.
- **World model.** One object per grid, the whole block, the figure seen
  first. Its box doubles in width to hold the block and its reflection, so
  nothing finer was needed. Before it read the test pair, the worker set
  aside a plain copy placed to the right, a half turn or a top-bottom mirror
  image placed to the right, and a mirror image placed on the left, none of
  which reproduces any training output, because no training block is
  symmetric in those ways. Color regions or single cells would be finer than
  the training pairs need.

## Dataset

The single dataset holds the four original training pairs in source order,
followed at the same level by the puzzle's test pair, which the same rules
annotate. Each object is one whole block: an instance of
`Obj(x, y, width, height, orientation)` with only the standard entries, whose
initial tag `l9100` is the single shape; its declaration, comments included,
is that of `dataset.6150a2bd.json`. The class takes no descriptive property:
the block's cells and colors travel in its masks, so there is no color, parts
list, canvas size, or matrix of cells. Two unconditional class-level rules
turn every block into the figure it forms with its mirror image:

```nuc
class Obj(x: int, y: int, width: int, height: int, orientation: int):
$Obj.tag = 'l6919'
$Obj.width = $Obj.width@first * 2
```

`l6919`, the tag of `dataset.c9e6f938.json`, is a shape beside its
left-right mirror image, symmetric about the vertical center line of its box.
`width` doubles from the block's input width, read with `@first`, so the
figure is 6 columns wide and its center line is the block's right edge. `x`,
`y`, `height`, and `orientation` stay as they were: the figure keeps the
block's top-left corner and rows, and the reflection is part of the figure
that `l6919` names, not a turn, so orientation is 0 in both states, as for
any block. The rules create no property and state no mirror axis, because the
tag and the doubled width already determine it. Each grid holds a single
block and shows no relation between objects, so no population operation
appears. Each rule carries `#` comment lines that describe it, and each
implementation opens with comment lines that name the block's colors in
reading order and its box, such as `# The whole block of colors 6, 1 and 8,`
and `# 3 by 3 at x 0, y 0` for the first training input. The
[README](../README.md#instruction-datasets) describes the dataset in full.

The training pairs cannot tell the canvas-edge rule of
`dataset.c9e6f938.json`, `$Obj.width = 2 * (canvas_width - $Obj.x@first)`,
from doubling the block's own width: every block fills its canvas, so both
give 6 columns in every pair. The worker took the doubling rule, which reads
only the block.

The caller passed the arc-dataset worker the declarative, higher-order, and
gestalt requirements, the orientation convention with its shared comment
line, the standard entries, and the six instance keys that every dataset has
had since `d0e3eed`, which the arc-dataset profile does not list yet. It
checked the handoff against those requirements and found that it met them,
so it sent nothing back. It reran the worker's offline check on the accepted
runtime, printing only each check's name and result, and all 146 checks
passed:

- the file parses, with the datasets' top-level, pair, and instance keys in
  their order, LF line endings, and the datasets' layout;
- the 4 training pairs and the test pair keep their original grids, in
  order;
- every mask keeps its full canvas, 3 by 3 for inputs and 3 by 6 for
  outputs, holds only its grid's cells, and matches its snapshot's box, and
  the masks merge back into each grid;
- each model state reproduces its original output, the input beside its
  left-right mirror image;
- every input and output snapshot reruns exactly, also with the rules
  reversed and declared after the instance;
- the instance names are unique across all 30 JSON datasets.

After the run, the pull before this record's commit merged `64bc0c9`,
"Split long Rust files under a 500-line limit", which by its own account
moves the runtime's items verbatim into child modules, and the same 146
checks pass on a scratch build of the merged runtime as well.

The worker's other offline checks passed as well: `document()` includes the
dataset as the 12th of the 30 JSON instruction datasets, in filename order,
and with `--only-puzzle-dataset` it loads the four genesis datasets and this
one. The worker wrote the dataset, its README paragraph, between the
`dataset.6150a2bd.json` and `dataset.6e02f1e3.json` paragraphs, and the
puzzle's entry at the end of the **ARC Benchmark** workflow's list, its 27th
puzzle. The caller made the puzzle the solver's default in `src/app.ts`, in
place of the genesis puzzle `3aa6fb7a`, before the run.

### Letter colors

On 8-Nov-2026, at the user's request, every instruction dataset started
writing its colors as letters instead of numbers, so that the pattern reaches
the model without the colors. `0` stays `0`, and every other color takes a
letter from `A` to `Z` out of the dataset's own random set: the 26 letters,
ordered by the SHA-256 digest of `<key>:<letter>` with the puzzle ID as the
key, go to the dataset's colors in increasing order. Each file therefore has
its own letters, a color keeps one letter in every pair, and the letters carry
no meaning beyond which cells share a color. This dataset writes its colors in
its own random letters: 1 as `P`, 6 as `H`, and 8 as `J`. Mapping the letters
back reproduces the puzzle's pairs cell for cell. A first version the same day
ranked the letters by count, `A` for the most used color, which gave every
letter a meaning; the random set replaced it.

The class and rules hold no color, so only the pair grids and both instance
masks changed; the declarations and implementations are unchanged.

Every snapshot was regenerated and replays byte for byte with the accepted
runtime at `1931f77`, whose last runtime change is `34bbba0`, and equals its
earlier snapshot byte for byte, so the rules behave as before; instance names,
positions, sizes, orientations, and tags are unchanged. `Matrix.silhouette`
takes any letter from `A` to `Z` and still ranks each crop by its own counts,
the most used color as `A`, so the orientation call's crops read exactly as
before. The solver still sends the running puzzle as numbers. No solver run
has measured this version, so its difference is unknown, not zero.

## Runs

| Date | Command | Runtime | Exit status | Difference | Elapsed |
| --- | --- | --- | --- | --- | --- |
| 7-Nov-2026 14:12 UTC | `bun start 6d0aefbc` | Accepted runtime at `363e5a5` (last runtime change `33a66ec`), local | 0 | 0 | 4m 7.1s |

The run was `bun start 6d0aefbc` from `arc/`, run once by `/arc-purposal`
with a 30-minute limit and `NUCLEOID_EXECUTABLE` unset, after
`bun install --frozen-lockfile` reported no changes. It used the primary
checkout's debug runtime (`nucleoid 1.0.0`), built with `cargo build` at the
start of the invocation, whose runtime sources last changed in `33a66ec`,
"NUC 88 Derivation Path Selections". The solver was the one committed at
`363e5a5`, "Remove input instances from visualizer prompts", with only its
default puzzle changed. The user committed `363e5a5` while the dataset was
being written, on top of `4909f30`, which amended the invocation's starting
revision `6880522`; neither changed the runtime, and no solver change was
uncommitted when the run started. The model settings were the committed
`qwen/qwen3.8-27b`, pinned to the `deepinfra/bf16` provider with caching
off, and each of the run's 21 model requests reported 0 cached input tokens.
`--only-puzzle-dataset` was off, so the calls that send few-shot examples
held all 30 instruction datasets, this dataset's annotated test pair among
them, as the ARC instructions intend while the datasets are prepared as
few-shot examples.

The run ended in an exact match, exit status 0 and difference 0, in 4m 7.1s
by the solver's own report; the runner recorded the process from 14:12:02.4
to 14:16:10.2 UTC. Its error log holds only bun's command line. The
runtime's hash was the same before and after the run, and no commit reached
the primary checkout during it. The invocation authorizes one run, so none
was repeated.

At 14:13:17 UTC, 74 seconds into the run, an edit outside the invocation
added `few_shot: () => ""` to three calls in `src/visualizer.ts`. The solver
had loaded its modules when it started, so the run used the committed file;
the user committed the edit after the run as `9f73238`, "Disable few-shot
examples in visualizer calls", which gives those three calls an empty
few-shot projection in later runs. A solver run started outside the
invocation rewrote `logs/6d0aefbc.log.json`, which Git ignores, at 14:16:46
UTC, 36 seconds after this run ended, so that file is not this run's log;
this run's own output stays in the invocation's session workspace. The
puzzle joined the **ARC Benchmark** workflow with this dataset, so no
workflow run is recorded yet.

## Open items

- **Messages are not JSON-only.** Carried over from
  [ARC-NUC-c9e6f938](ARC-NUC-c9e6f938.md) and
  [ARC-NUC-c8f0f002](ARC-NUC-c8f0f002.md): the messages that
  `src/lib/llm.ts`, `src/analyzer.ts`, and `src/visualizer.ts` build wrap
  JSON in text labels such as `few_shot_examples:`, `declarations:`, and
  `json_format:`, while the ARC instructions require every message to parse
  as one JSON document. It remains an open question for the user. This
  dataset does not depend on it.
- **Removed instance fields are still documented.** `d0e3eed` removed
  `input_x`, `output_x`, `input_y`, `output_y`, `input_orientation`, and
  `output_orientation` from every dataset, but the arc-dataset profile still
  lists them among an instance's keys, and parts of `arc/README.md` still
  describe them as dataset fields: the sentence after the analysis table
  that the dataset files keep every property, and the "3. Extract
  instances" text, which says the genesis datasets expose them on each
  instance. This dataset has the six keys the current datasets share.
  Updating the profile and the README is the user's call.
- **Letter version unmeasured.** No solver run has measured the 8-Nov-2026
  letter version. The solver still sends this puzzle's own grids as numbers,
  while every few-shot dataset now writes its colors as letters.

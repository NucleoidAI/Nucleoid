# ARC-NUC-c9e6f938 -- Grid Beside Its Mirror Image

Progress record for ARC training puzzle `c9e6f938`: its instruction dataset,
the solver and Nucleoid changes it needed, its runs, and what is still open.

| Item | Status |
| --- | --- |
| Puzzle | [`data/training/c9e6f938.json`](../data/training/c9e6f938.json) |
| Instruction dataset | [`dataset.c9e6f938.json`](../src/dataset/dataset.c9e6f938.json): the puzzle's 3 original training pairs and its test pair, 4 pattern instances, 1 in each pair; committed with this record; orientation labelled by shape and the shared orientation comment added on 6-Nov-2026; descriptive `implementation` comments added on 7-Nov-2026; colors written as letters on 8-Nov-2026; respelled on 8-Nov-2026 with the `is` assignments of NUC 93 and bare words, every snapshot unchanged |
| Output canvas | `EXACT` |
| Nucleoid enhancements | None requested, so no proposal strategy was selected and strategy learning did not apply; the rules use the class-level `@first` anchor of [NUC 55](../../docs/nuc-0055.md), built on the state versions of [NUC 53](../../docs/nuc-0053.md) |
| Solver changes | None applied; one suggested and not applied, the output canvas framed on the world model's canvas size that [ARC-NUC-8be77c9e](ARC-NUC-8be77c9e.md) also suggests; the JSON-only message question carried over from earlier records stays open; since 8-Nov-2026 `src/lib/nucleoid.ts` passes `--is-assignment`, which every respelled dataset needs |
| Solver runs | 1 local run on 6-Nov-2026: exit 0, difference 0; none of the current version; none of the 8-Nov-2026 `is` version |
| Updated | 8-Nov-2026 |

## Puzzle

Each training input is a 3 by 3 canvas holding one pattern of color-7 cells
on a background of zero cells: 4, 2, and 1 cells in the three training
pairs. Each output is a 3 by 6 canvas, twice as wide, that sets the input
beside its left-right mirror image: the input reappears unchanged in the left
three columns, and its column c reappears as column 5 - c, so every output
row reads the same from right to left as from left to right. The mirror line
is the canvas's right edge, not the pattern's: the third training pattern is
a single cell in the left column of the middle row, so four empty columns
separate it from its mirror image in the right column, while the first and
second patterns reach the right edge and meet theirs. The output keeps the
input's 3 rows and grows only to the right, so the dataset is `EXACT`.

`/arc-purposal` chose this puzzle by default: among the 376 training puzzles
other than the genesis puzzle `3aa6fb7a` and those with a dataset, its
largest training matrix has the fewest cells, 18, with 81 cells in all its
training matrices. `8be77c9e` tied on both measurements and came first by
ID, and it already had its dataset; the next, `c8f0f002`, has 18 and 90. The
puzzle is the left-right counterpart of
[ARC-NUC-8be77c9e](ARC-NUC-8be77c9e.md), which stacks its grid over its
top-bottom mirror image.

## Gestalt overview

- **Whole grid.** A small 3 by 3 canvas with one color-7 pattern; the output
  is the same grid on the left and its reflection on the right, a 3 by 6
  canvas that is symmetric about its vertical center line.
- **Figures.** The whole color-7 pattern is the figure seen first, one per
  grid, including where its cells touch only at a corner, as the first
  training pattern's top cell does.
- **Relations.** The pattern sits against the canvas's edges: against the
  right edge in the first and second pairs, and against the left edge, two
  empty columns from the right one, in the third. The canvas's right edge is
  the line the pattern is mirrored across.
- **Change.** The canvas doubles to the right, and the whole grid is joined
  by its reflection across its right edge, so each pattern gains its mirror
  image to its right, keeping its color, its rows, and its place.
- **World model.** One object per grid, the whole pattern, the figure seen
  first. Its box grows right to hold the pattern and its reflection, so
  nothing finer was needed. Before it read the test pair, the worker set
  aside a plain copy placed to the right, which reproduces no training
  output; a half turn placed to the right, which reproduces only the second
  and third, whose inputs are top-bottom symmetric; mirroring across the
  pattern's own right edge, which fails the third; and a top-bottom mirror
  or a mirror placed to the left, which reproduce none.

## Dataset

The single dataset holds the three original training pairs in source order,
followed at the same level by the puzzle's test pair, which the same rules
annotate. Each object is one whole pattern: an instance of
`Obj(x, y, width, height, orientation)` with only the standard entries, whose
initial tag `l9100` is the single shape of `dataset.28bf18c6.json` and
`dataset.8be77c9e.json`. The class takes no descriptive property: the
pattern's cells travel in its masks, and the rules read the canvas's width
from the solver's `canvas_width`, so there is no color, parts list, canvas
size, or matrix of cells. Two unconditional class-level rules turn every
pattern into the figure it forms with its reflection:

```nuc
class Obj(x: int, y: int, width: int, height: int, orientation: int):
$Obj.tag = 'l6919'
$Obj.width = 2 * (canvas_width - $Obj.x@first)
```

`l6919`, a new tag, is a shape beside its left-right mirror image, symmetric
about the vertical center line of its box, the left-right counterpart of the
`l7511` of `dataset.8be77c9e.json`. `width` becomes twice the distance from
the pattern's left column to the canvas's right edge, so the mirror line lies
at the box's center: 4 columns for the first and second training patterns,
which start in the second column, and 6 for the third, which starts in the
left column. The rule reads `canvas_width` as `dataset.8be77c9e.json` reads
`canvas_height`, anchored with `@first` to read the pattern's input column,
although no rule moves `x`. `x`, `y`, `height`, and `orientation` stay as
they were: the reflection keeps the pattern's rows, the figure starts at the
pattern's left column, and the reflection is part of the figure `l6919`
names, not a turn. The rules create no property and state no `mirror_axis`,
the property of `dataset.74dd1130.json`, because the tag and the growing
`width` already determine the axis. Each grid holds a single pattern and
shows no relation between objects, so no population operation appears. The
declarations carry no `#` comments. The
[README](../README.md#instruction-datasets) describes the dataset in full.

The caller checked the handoff against the declarative, higher-order, and
gestalt requirements and found that it met them, so it sent nothing back. It
reran the worker's offline check on the accepted runtime, printing only each
check's name and result, and all 105 checks passed:

- the file parses, with the datasets' top-level and instance keys in their
  order, LF line endings, no byte-order mark, and no `#` comments;
- the 3 training pairs and the test pair keep their original grids, in
  order;
- every mask keeps its full canvas, 3 by 3 for inputs and 3 by 6 for
  outputs, holds only its grid's cells, and matches its x/y fields, and the
  masks merge back into each grid;
- each output mask is its input mask beside its left-right mirror image, and
  each output box's width is `2 * (canvas_width - x)`;
- every input and output snapshot reruns exactly, also with the rules
  reversed and declared after the instances;
- the 246 instance names across the 28 JSON datasets are unique.

The worker's other offline checks passed as well: `document()` includes the
dataset as the 20th of the 28 JSON instruction datasets, in filename order,
and `bun run typecheck` and `bun run lint` pass. Its offline copy of the
solver's composition found the canvas gap under Open items.

The worker wrote the dataset, its README paragraph, and the puzzle's entry
in the **ARC Benchmark** workflow's list, its 25th puzzle, which leaves the
job's 170-minute timeout sufficient. The caller had asked for the paragraph
at the end of the README's dataset paragraphs, which put it out of their
filename order, so the caller moved it unchanged to follow the
`dataset.a9f96cdd.json` paragraph. The caller made the puzzle the solver's
default in `src/app.ts`, in place of `8be77c9e`, before the run.

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

One of the four patterns changed: the second training pair's two-cell row is a
horizontal line without a head, 90, and so is its four-cell figure with its
mirror image. The other patterns and figures stay 0. The mirror image is part
of the figure, not a turn, so no rule changed.

The grids, masks, positions, instance names, tags, and every other property
are as before; only the orientation values changed, so the world model keeps
its level.

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
only color, 7, as `A`. Mapping the letters back reproduces the puzzle's pairs
cell for cell.

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

$Obj.tag is l6919

$Obj.width is 2 * (canvas_width - $Obj.x@first)
```

`target is value` builds exactly the syntax tree of `target = value`. A bare
word reads the name of its spelling when one exists and otherwise stores its
own text: in the constructor, `this.x is x` reads the parameter `x`, and no
program of this dataset defines any of `l9100` and `l6919`, so those words
read as their own text. Implementations construct their instances with `is` as
well, as in
`obj8e3cc5cb798f is Obj(x=1, y=0, width=2, height=3, orientation=0)`. Only the
spelling changed: the class, the rules' structure, the grids, masks,
positions, instance names, and snapshots are as before, and the world model
keeps its level, so the gestalt overview above still holds.

The rules now need a runtime at `ad45ba4`, the NUC 93 commit, or later, run
with `--is-assignment`, which `src/lib/nucleoid.ts` now passes to every
program; without the flag, `obj8e3cc5cb798f is Obj(...)` reads as three
juxtaposed statements and raises `ReferenceError`. Offline, with the accepted
runtime at `4ac649b` (last runtime change `ad45ba4`), the flag reproduced all
4 input and 4 output snapshots of the 4 pairs, both in a direct replay and
through the solver's own `nucleoid.run`, and the previous version reproduced
them with and without the flag. No NUC was requested, so no proposal strategy
was selected and strategy learning did not apply. No solver run has used this
version.

## Runs

| Date | Command | Runtime | Exit status | Difference | Elapsed |
| --- | --- | --- | --- | --- | --- |
| 6-Nov-2026 13:00 UTC | `bun start c9e6f938` | Accepted runtime at `b3cbd26` (last runtime change `7096171`), local | 0 | 0 | 2m 1.2s |

The run was `bun start c9e6f938` from `arc/`, run once by `/arc-purposal`
with a 30-minute limit and `NUCLEOID_EXECUTABLE` unset, after
`bun install --frozen-lockfile` reported no changes. It used the primary
checkout's debug runtime (`nucleoid 1.0.0`), current with `cargo build` at
`b3cbd26`. The solver was the one committed at `b3cbd26` with its default
puzzle changed, plus another session's change to `src/lib/openrouter.ts`,
`src/lib/milestone.ts`, and `src/lib/terminal.ts`, uncommitted when the run
started. That session committed it 19 seconds after the run ended, then
rewrote its unpushed commits; the change is now `f45e54b`, "Disable
OpenRouter caching and alert on cache use", whose three solver files hold
the same contents the run used. The files were last written 23 seconds
before the run started. The change sends `X-OpenRouter-Cache: false`, fails a
request whose response reports a cache hit, and reports cached input
tokens; it leaves the model, sampling, provider, and prompts as they were.
The model settings were the committed `qwen/qwen3.8-27b`, pinned to the
`deepinfra/bf16` provider. `--only-puzzle-dataset` was off, so the few-shot
context held all 28 instruction datasets, this dataset's annotated test pair
among them, as the ARC instructions intend while the datasets are prepared
as few-shot examples.

The run ended in an exact match, exit status 0 and difference 0, in 2m 1.2s
by the solver's own report; the runner recorded the process from 13:00:45.7
to 13:02:47.3 UTC. Its error log holds only bun's command line, and it wrote
`logs/c9e6f938.log.json`, which Git ignores. The runtime's hash was the same
before and after the run, and no commit reached the primary checkout during
it. The invocation authorizes one run, so none was repeated.
`bun run typecheck` and `bun run lint` passed with the new default after the
run. The puzzle joined the **ARC Benchmark** workflow with this dataset, so
no workflow run is recorded yet.

None of these runs measured the orientation version of 6-Nov-2026: the user
chose offline verification for the batch, so its difference is unknown, not
zero.

## Open items

- **The output canvas cannot keep a trailing empty column.** This evidence
  comes from the training pairs alone. Training outputs 1 and 2 are 3 by 6,
  and their right column is empty, because their patterns start in the
  second column. `src/visualizer.ts` crops each predicted output instance to
  its nonzero cells, and `Matrix.pad` places it on the input's 3 columns or
  as far right as its rightmost cell; `Matrix.canvas` in `src/lib/Matrix.ts`
  then returns the merged result for `EXACT` and crops it for `FIT`. Composed
  that way, the first and second training outputs come out 5 columns wide,
  while the third comes out at its full 6. No dataset can close the gap,
  because an output mask holds only cells of its output, and that column has
  none. It is the column counterpart of the trailing empty row in
  [ARC-NUC-8be77c9e](ARC-NUC-8be77c9e.md), and one change would close both.
  Suggested, not applied: add an `output_canvas` form whose width and height
  are Nucleoid expressions over `canvas_width` and `canvas_height`, here
  `2 * canvas_width` and `canvas_height`. The solver would evaluate them with
  the same canvas declarations and frame the merged result to exactly that
  size, in `src/app.ts`, `src/lib/Matrix.ts`, the `output_canvas` format in
  `src/analyzer.ts`, and the README's output-canvas description. `EXACT` and
  `FIT` would stay unchanged, and this dataset could adopt the new form in a
  later invocation.
- **Messages are not JSON-only.** Carried over from
  [ARC-NUC-8be77c9e](ARC-NUC-8be77c9e.md): `src/lib/llm.ts` sends its system
  messages as JSON, but the user and assistant messages that
  `src/analyzer.ts` and `src/visualizer.ts` build still wrap JSON in text
  labels, while the ARC instructions require every message to parse as one
  JSON document. It remains an open question for the user. This dataset does
  not depend on it.
- **Orientation version unmeasured.** No solver run has measured the
  6-Nov-2026 orientation version, so whether consistent orientation labels
  change this puzzle's result is unknown.
- **Letter version unmeasured.** No solver run has measured the 8-Nov-2026
  letter version. The solver still sends this puzzle's own grids as numbers,
  while every few-shot dataset now writes its colors as letters.
- **The `is` assignment version is unmeasured.** No solver run has used the
  8-Nov-2026 version with `is` assignments and bare words, so its effect on
  this puzzle's result is unknown. Every run's few-shot context now shows bare
  words beside quoted strings, and a word reads as its text only as a whole
  value: a model that writes one inside a larger value, such as
  `$Obj.fill_color is A if $Obj.symmetric else C`, gets a `ReferenceError`.

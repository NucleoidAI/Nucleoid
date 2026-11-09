# ARC-NUC-e9afcf9a -- Bars Woven Into a Checkerboard

Progress record for ARC training puzzle `e9afcf9a`: its instruction dataset,
the solver and Nucleoid changes it needed, its runs, and what is still open.

| Item | Status |
| --- | --- |
| Puzzle | [`data/training/e9afcf9a.json`](../data/training/e9afcf9a.json) |
| Instruction dataset | [`dataset.e9afcf9a.json`](../src/dataset/dataset.e9afcf9a.json): the puzzle's 2 original training pairs and its test pair, 6 bar instances, 2 in each pair; committed with this record; orientation labelled by shape and the shared orientation comment added on 6-Nov-2026; descriptive `implementation` comments added on 7-Nov-2026; colors written as letters on 8-Nov-2026; respelled on 8-Nov-2026 with the `is` assignments of NUC 93 and bare words, every snapshot unchanged |
| Output canvas | `EXACT` |
| Nucleoid enhancements | None requested, so no proposal strategy was selected and strategy learning did not apply; the rules use the class-level `@first` anchor of [NUC 55](../../docs/nuc-0055.md), built on the state versions of [NUC 53](../../docs/nuc-0053.md) |
| Solver changes | None; one earlier suggested change still awaits review, not applied; since 8-Nov-2026 every respelled dataset needs `is` assignments, which `src/lib/nucleoid.ts` enabled with `--is-assignment` until `d945213` made them the default |
| Solver runs | 1 local run on 6-Nov-2026: exit 0, difference 0; none of the current version; none of the 8-Nov-2026 `is` version |
| Updated | 8-Nov-2026 |

## Puzzle

Each training input is a 2 by 6 canvas holding two horizontal bars, one on
top of the other, each a whole row of one color: color 3 over color 9 in the
first pair and color 4 over color 8 in the second, with no zero cell. The
output keeps the canvas and weaves the two colors into a checkerboard. In the
even columns, counting the left column as 0, each bar keeps its cells in its
own row; in the odd columns its cells trade places with the other bar's. The
top row therefore alternates the top bar's color and the bottom bar's,
starting with the top bar's color, and the bottom row alternates them the
other way round. Each color keeps its six cells. The output keeps the input's
2 by 6 canvas, so the dataset is `EXACT`; with no zero cell in any output,
`FIT` would frame the same grids.

`/arc-purposal` chose this puzzle by default: among the 379 training puzzles
other than the genesis puzzle `3aa6fb7a` and those with a dataset, its largest
training matrix has the fewest cells, 12, with 48 cells in all its training
matrices. The next, `a9f96cdd`, has 15 and 120.

## Gestalt overview

- **Whole grid.** A 2 by 6 canvas split into two solid horizontal stripes of
  different colors; the output is the same canvas as a two-color
  checkerboard.
- **Figures.** The two bars, each a full-width row of one color, are what is
  seen first. The output reads as one checkerboard of the same two colors.
- **Relations.** One bar lies on top of the other, and together they fill
  the canvas. Each bar's row decides its share of the checkerboard, and the
  top bar keeps the top-left cell.
- **Change.** The bars weave into each other: each keeps its own row in every
  other column, starting from the left, and trades the cells between them
  with the other bar, so the colors alternate along every row and down every
  column.
- **World model.** One object per bar, the figure seen first. Each bar
  becomes one color of the checkerboard, the cells whose row plus column has
  the parity of its own row, so nothing finer was needed. The worker set
  aside the whole grid as one mosaic, because the two bars, not the grid, are
  seen first, and six columns with every other one flipped, which is finer
  than the training pairs need.

## Dataset

The single dataset holds the two original training pairs in source order,
followed at the same level by the puzzle's test pair, which the same rules
annotate. Each object is one bar: an instance of
`Obj(x, y, width, height, orientation)` with only the standard entries,
tagged `l3058`, the horizontal bar of `dataset.d631b094.json`,
`dataset.25d8a9c8.json`, and `dataset.a85d4709.json`. Every bar is 6 wide and
1 tall. The class takes no descriptive property: the bar's color travels in
its masks, and its share of the checkerboard follows from its row, so there
is no color, list, canvas size, or matrix of cells. Four unconditional
class-level rules turn every bar into its share of the checkerboard:

```nuc
class Obj(x: int, y: int, width: int, height: int, orientation: int):
$Obj.tag = 'l5973'
$Obj.parity = $Obj.y@first % 2
$Obj.height = canvas_height
$Obj.y = 0
```

`l5973`, a new tag, is one color of a checkerboard: the cells of its box
whose row plus column has the parity `parity`. `parity` is the parity of the
bar's input row, 0 for the top bar and 1 for the bottom one, anchored with
`@first` because the last rule moves `y`. `height` reads the solver's
`canvas_height`, as `dataset.d037b0a7.json` does, so each share spans the
canvas from its top row, while `x`, `width`, and `orientation` stay as they
were. The rules create only `parity`, which input states lack. Orientation
stays 0: the two shares of a pair are mirror images, but no turn carries one
onto the other, so `parity`, not `orientation`, tells them apart. The two
shares interlock because the bars lie in rows of different parity, not
because a rule compares them, so no population operation appears. The
declarations carry no `#` comments. The
[README](../README.md#instruction-datasets) describes the dataset in full.

The two bars fill the canvas in every training pair, so a checkerboard over
only the rows the bars cover fits the pairs as well. The worker tried that
reading with the population queries
`$Obj.height = sum of Obj.map(other => other.height@first)` and
`$Obj.y = min of Obj.map(other => other.y@first)`, and the accepted runtime
rejected the program with `TypeError: Circular Dependency` at its second
instance. The canvas rules say the same with less, so the dataset takes them
and requests no enhancement; the open items keep the error.

The caller checked the handoff against the declarative, higher-order, and
gestalt requirements and the user's ARC conventions, found that it met them,
and sent nothing back. It reran the worker's offline check on the accepted
runtime, and all 17 checks passed:

- the file parses and keeps the other datasets' layout byte for byte;
- `document()` includes it as the 21st of the 25 JSON instruction datasets,
  in filename order;
- the pairs and their grids equal the originals, in order;
- the 233 instance names across all datasets are unique;
- the rules reproduce every pair's output;
- every mask keeps its full 2 by 6 canvas, and the masks merge back into
  each grid;
- every input and output snapshot reruns byte for byte.

The worker wrote the dataset, its README paragraph, and the puzzle's entry
in the **ARC puzzle report** workflow's list, its 22nd puzzle, which leaves
the job's 170-minute timeout sufficient. The caller made the puzzle the
solver's default in `src/app.ts`, in place of `794b24be`, before the run.

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

Each input bar is a straight row of one color without a head, so all six
inputs are now 90. Each output is the bar's share of the checkerboard, a
two-row pattern that points no way, 0, so one new unconditional rule,
`$Obj.orientation = 0`, sets it, and the output states are as before. The
rules now read:

```nuc
class Obj(x: int, y: int, width: int, height: int, orientation: int):
$Obj.tag = 'l5973'
$Obj.orientation = 0
$Obj.parity = $Obj.y@first % 2
$Obj.height = canvas_height
$Obj.y = 0
```

The grids, masks, positions, instance names, tags, and every other property
are as before; only the orientation values and the new rule changed, so the
world model keeps its level.

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
colors as letters ranked by count: 3 as `A`, 9 as `B`, 4 as `C`, 8 as `D`, 6
as `E`, and 2 as `F`. Mapping the letters back reproduces the puzzle's pairs
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
    this.tag is l3058

$Obj.tag is l5973

$Obj.orientation is 0

$Obj.parity is $Obj.y@first % 2

$Obj.height is canvas_height

$Obj.y is 0
```

`target is value` builds exactly the syntax tree of `target = value`. A bare
word reads the name of its spelling when one exists and otherwise stores its
own text: in the constructor, `this.x is x` reads the parameter `x`, and no
program of this dataset defines any of `l3058` and `l5973`, so those words
read as their own text. Implementations construct their instances with `is` as
well, as in
`objf75bfec5b742 is Obj(x=0, y=0, width=6, height=1, orientation=90)`. Only
the spelling changed: the class, the rules' structure, the grids, masks,
positions, instance names, and snapshots are as before, and the world model
keeps its level, so the gestalt overview above still holds.

The rules now need a runtime at `ad45ba4`, the NUC 93 commit, or later, run
with `--is-assignment`, which `src/lib/nucleoid.ts` now passes to every
program; without the flag, `objf75bfec5b742 is Obj(...)` reads as three
juxtaposed statements and raises `ReferenceError`. Offline, with the accepted
runtime at `4ac649b` (last runtime change `ad45ba4`), the flag reproduced all
6 input and 6 output snapshots of the 3 pairs, both in a direct replay and
through the solver's own `nucleoid.run`, and the previous version reproduced
them with and without the flag. No NUC was requested, so no proposal strategy
was selected and strategy learning did not apply. No solver run has used this
version.

## Runs

| Date | Command | Runtime | Exit status | Difference | Elapsed |
| --- | --- | --- | --- | --- | --- |
| 6-Nov-2026 11:40 UTC | `bun start e9afcf9a` | Accepted runtime at `00d91d0` (last runtime change `3592d4b`), local | 0 | 0 | 4m 11.4s |

The run was `bun start e9afcf9a` from `arc/`, run once by `/arc-purposal`
with a 30-minute limit and `NUCLEOID_EXECUTABLE` unset, after
`bun install --frozen-lockfile` reported no changes. It used the primary
checkout's debug runtime (`nucleoid 1.0.0`), current with `cargo build` at
`00d91d0`, and the committed solver at `00d91d0` with only its default puzzle
changed. The model settings were the committed `qwen/qwen3.8-27b`, pinned to
the `deepinfra/bf16` provider. `--only-puzzle-dataset` was off, so the
few-shot context held all 25 instruction datasets, this dataset's annotated
test pair among them, as the ARC instructions intend while the datasets are
prepared as few-shot examples.

The run ended in an exact match, exit status 0 and difference 0, in 4m 11.4s
by the solver's own report; the runner recorded the process from 11:40:03.0
to 11:44:14.9 UTC. Its error log holds only bun's command line, and it wrote
`logs/e9afcf9a.log.json`, which Git ignores. The runtime's hash was the same
before and after the run, and no commit reached the primary checkout during
it. [NUC 86](../../docs/nuc-0086.md), `7096171`, which another session
committed four seconds before the run started, arrived only with the pull
before this record's commit. On a runtime built at `7096171` in a scratch
target directory, the worker's 17 offline checks still pass, every snapshot
included. The invocation authorizes one run, so none was repeated. The puzzle
joined the **ARC puzzle report** workflow with this dataset, so no workflow
run is recorded yet.

None of these runs measured the orientation version of 6-Nov-2026: the user
chose offline verification for the batch, so its difference is unknown, not
zero.

## Open items

- **Same-property population anchors are circular.** On the accepted runtime
  at `00d91d0`, and still at `7096171`, a class rule that sets a property from
  the whole population's `@first` values of that same property, such as
  `$Obj.height = sum of Obj.map(other => other.height@first)` or
  `$Obj.y = min of Obj.map(other => other.y@first)`, raises
  `TypeError: Circular Dependency` when the second instance is created,
  although every value it reads is anchored to the input. The same query
  feeding a different property, such as
  `$Obj.parity = sum of Obj.map(other => other.height@first)`, runs. This
  dataset does not need it. A later puzzle whose rules do can request a NUC
  through its own `/arc-dataset` handoff.
- **Messages are not JSON-only.** Carried over from
  [ARC-NUC-794b24be](ARC-NUC-794b24be.md): since `f58e26a`, `src/lib/llm.ts`
  sends its system messages as JSON, but the user and assistant messages that
  `src/analyzer.ts` and `src/visualizer.ts` build still wrap JSON in text
  labels such as `train_dataset:`, `declarations:`, and `json_format:`, while
  the ARC instructions require every message to parse as one JSON document.
  Suggested, not applied: build each of those messages with `JSON.stringify`
  of one data object. This dataset does not depend on it.
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

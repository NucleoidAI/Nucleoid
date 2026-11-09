# ARC-NUC-a9f96cdd -- Point Turned Into Four Colored Corners

Progress record for ARC training puzzle `a9f96cdd`: its instruction dataset,
the solver and Nucleoid changes it needed, its runs, and what is still open.

| Item | Status |
| --- | --- |
| Puzzle | [`data/training/a9f96cdd.json`](../data/training/a9f96cdd.json) |
| Instruction dataset | [`dataset.a9f96cdd.json`](../src/dataset/dataset.a9f96cdd.json): the puzzle's 4 original training pairs and its test pair, 5 point instances, 1 in each pair; committed with this record; shared orientation comment added on 6-Nov-2026; descriptive `implementation` comments added on 7-Nov-2026; colors written as letters on 8-Nov-2026; respelled on 8-Nov-2026 with the `is` assignments of NUC 93 and bare words, every snapshot unchanged; colors written as numbers again on 8-Nov-2026 |
| Output canvas | `EXACT` |
| Nucleoid enhancements | None requested, so no proposal strategy was selected and strategy learning did not apply; the rules use the class-level `@first` anchor of [NUC 55](../../docs/nuc-0055.md), built on the state versions of [NUC 53](../../docs/nuc-0053.md), and the `min of` and `max of` noun queries of [NUC 75](../../docs/nuc-0075.md) over the extremes of [NUC 69](../../docs/nuc-0069.md) |
| Solver changes | None; the JSON-only message question carried over from earlier records stays open; since 8-Nov-2026 every respelled dataset needs `is` assignments, which `src/lib/nucleoid.ts` enabled with `--is-assignment` until `d945213` made them the default |
| Solver runs | 1 local run on 6-Nov-2026: exit 1, a failed run, `OPENROUTER_API_KEY is required` before the first model request; difference unknown; none of the current version; none of the 8-Nov-2026 `is` version |
| Updated | 8-Nov-2026 |

## Puzzle

Each training input is a 3 by 5 canvas of zero cells holding a single point
of color 2. The output keeps the canvas, turns the point's own cell to 0, and
colors its four diagonal neighbors, the corners of the 3 by 3 box centered on
it, each in a color of its own: 3 at the top left, 6 at the top right, 8 at
the bottom left, and 7 at the bottom right. Corners that would fall outside
the canvas are cut off. All four remain in the first and fourth training
pairs, whose points lie inside the canvas; only the top-left corner remains in
the second, whose point sits in the canvas's bottom-right corner; and only the
two bottom corners remain in the third, whose point sits on the top row. The
output keeps the input's 3 by 5 canvas, so the dataset is `EXACT`; `FIT` would
crop each output to its corners, such as the single cell of the second
training output.

`/arc-purposal` chose this puzzle by default: among the 378 training puzzles
other than the genesis puzzle `3aa6fb7a` and those with a dataset, its largest
training matrix has the fewest cells, 15, with 120 cells in all its training
matrices. The next, `8be77c9e` and `c9e6f938`, have 18 and 81.

## Gestalt overview

- **Whole grid.** A 3 by 5 canvas of zero cells with one point of color 2;
  the output is the same canvas with the point gone and up to four colored
  cells on its diagonals.
- **Figures.** The single point is what is seen first. The output reads as
  one figure, an X of four colored corners around an empty center, or the
  part of it that fits on the canvas.
- **Relations.** The corners sit one step from the point on each of its four
  diagonals, each corner's color fixed by its place: 3 and 6 above, 8 and 7
  below. The canvas edges cut off the corners beyond them.
- **Change.** The point opens into the ring of its four diagonal corners, its
  own cell turns 0, and the canvas clips the ring.
- **World model.** One object per point, the figure seen first, which
  becomes the ring of its remaining corners. No corner is an object of its
  own, because none exists in the input and each one's position and color
  follow from the point and the canvas, so nothing finer was needed. The
  worker set aside clamping the 3 by 3 box to the canvas: the third training
  point lies on the top row, so the clamped box would start on that row,
  which holds no corner, while the remaining corners lie only in the row
  below.

## Dataset

The single dataset holds the four original training pairs in source order,
followed at the same level by the puzzle's test pair, which the same rules
annotate. Each object is one point: an instance of
`Obj(x, y, width, height, orientation)` with only the standard entries,
1 wide and 1 tall, tagged `l7544`, the single point of
`dataset.0ca9ddb6.json`, `dataset.d037b0a7.json`, and `dataset.a85d4709.json`.
The class takes no descriptive property: every point has color 2, which the
rules never read. Six unconditional class-level rules turn every point into
its figure:

```nuc
class Obj(x: int, y: int, width: int, height: int, orientation: int):
$Obj.tag = 'l2898'
$Obj.corner_colors = [[3, 6], [8, 7]]
$Obj.x = min of [$Obj.x@first - 1, $Obj.x@first + 1].filter(column => column >= 0 and column < canvas_width)
$Obj.y = min of [$Obj.y@first - 1, $Obj.y@first + 1].filter(row => row >= 0 and row < canvas_height)
$Obj.width = max of [$Obj.x@first - 1, $Obj.x@first + 1].filter(column => column >= 0 and column < canvas_width) - $Obj.x + 1
$Obj.height = max of [$Obj.y@first - 1, $Obj.y@first + 1].filter(row => row >= 0 and row < canvas_height) - $Obj.y + 1
```

`l2898`, a new tag, is the four corners of a 3 by 3 box around its empty
center, in the colors of `corner_colors`, unlike `l2404`, the diagonal cross
of `dataset.0ca9ddb6.json` and `dataset.27a28665.json`, which keeps its
center. `corner_colors` lists the corners' colors as they lie around the
point, the top row's left and right corners first and then the bottom row's,
and is the only property the rules create. The other four rules move the box
onto the remaining corners: the higher-order `filter` keeps the two corner
columns, one left and one right of the point, and the two corner rows, one
above and one below it, that lie on the canvas, reading the solver's
`canvas_width` and `canvas_height`, and `min of` and `max of` take the ends of
what it keeps. `x` and `y` become the leftmost column and topmost row kept,
and `width` and `height` run from there to the rightmost and bottommost,
anchored to the point's input column and row with `@first` because the rules
move `x` and `y`. Each figure depends only on its own point and the canvas,
and every pair holds one point, so no population operation appears.
Orientation stays 0: the point is symmetric and the figure is not turned, so
`corner_colors`, not `orientation`, records where each color goes. The rules
give the same states in reverse order and when they are declared after the
instances. The declarations carry no `#` comments. The
[README](../README.md#instruction-datasets) describes the dataset in full.

Each rule of an axis writes the filtered list out rather than storing it,
because the stored form is circular in the accepted runtime: the worker's
program with `$Obj.columns` holding the filtered columns and
`$Obj.x = min of $Obj.columns` raises `TypeError: Circular Dependency` at that
rule, and the caller reproduced the error at `cd7bd18`. The arc-dataset
profile takes a declarative formulation that works over a shorter spelling
that does not, so the dataset requests no enhancement; the open items keep
the error.

The caller checked the handoff against the declarative, higher-order, and
gestalt requirements and the user's ARC conventions, found that it met them,
and sent nothing back. It reran the worker's offline check on the accepted
runtime, and all 21 checks passed:

- the file parses and keeps the other datasets' layout and key order;
- the pairs and their grids equal the originals, in order;
- every mask keeps its full 3 by 5 canvas, and the masks merge back into
  each grid;
- the 238 instance names across all datasets are unique;
- every input and output snapshot reruns byte for byte, also with the rules
  declared after the instances and in reverse order;
- the rules reproduce every pair's output, and the solver's own crop, place,
  and merge steps with `EXACT` reproduce each output grid;
- `document()` includes the file among the 26 JSON instruction datasets, in
  filename order, and `document('a9f96cdd', true)` keeps the four genesis
  datasets and this one.

The worker wrote the dataset, its README paragraph, and the puzzle's entry
in the **ARC puzzle report** workflow's list, its 23rd puzzle, which leaves
the job's 170-minute timeout sufficient. The caller made the puzzle the
solver's default in `src/app.ts`, in place of `e9afcf9a`, before the run.

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
confirmed: each point is 0, and so is its output, the cells around it. Only
the comment line changed, so the class, the rules, the grids, masks,
positions, instance names, implementations, and snapshots are as before, and
the gestalt overview above still holds.

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
colors as letters ranked by count: 2 as `A`, 3 as `B`, 8 as `C`, 7 as `D`, and
6 as `E`. Mapping the letters back reproduces the puzzle's pairs cell for
cell.

The pair grids, both instance masks, the implementations, and the snapshots
use those letters, and the class and rules name the same colors by letter.
Only the color literals changed, and the changed declarations now read:

```nuc
$Obj.corner_colors = [['B', 'E'], ['C', 'D']]
```

Every snapshot was regenerated with the accepted runtime at `adc282a` and
equals its earlier snapshot with its colors replaced by their letters, so the
rules behave as before; instance names, positions, sizes, orientations, and
tags are unchanged. `Matrix.silhouette` now also takes letters, so the
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
    this.tag is l7544

$Obj.tag is l2898

$Obj.corner_colors is [['B', 'E'], ['C', 'D']]

$Obj.x is min of [$Obj.x@first - 1, $Obj.x@first + 1].filter(column => column >= 0 and column < canvas_width)

$Obj.y is min of [$Obj.y@first - 1, $Obj.y@first + 1].filter(row => row >= 0 and row < canvas_height)

$Obj.width is max of [$Obj.x@first - 1, $Obj.x@first + 1].filter(column => column >= 0 and column < canvas_width) - $Obj.x + 1

$Obj.height is max of [$Obj.y@first - 1, $Obj.y@first + 1].filter(row => row >= 0 and row < canvas_height) - $Obj.y + 1
```

`target is value` builds exactly the syntax tree of `target = value`. A bare
word reads the name of its spelling when one exists and otherwise stores its
own text: in the constructor, `this.x is x` reads the parameter `x`, and no
program of this dataset defines any of `l7544` and `l2898`, so those words
read as their own text. A string inside a larger value keeps its quotes, as in
`$Obj.corner_colors is [['B', 'E'], ['C', 'D']]`, because a word must be the
whole value. Implementations construct their instances with `is` as well, as
in `objed6a3fa35475 is Obj(x=1, y=1, width=1, height=1, orientation=0)`. Only
the spelling changed: the class, the rules' structure, the grids, masks,
positions, instance names, and snapshots are as before, and the world model
keeps its level, so the gestalt overview above still holds.

The rules now need a runtime at `ad45ba4`, the NUC 93 commit, or later, run
with `--is-assignment`, which `src/lib/nucleoid.ts` now passes to every
program; without the flag, `objed6a3fa35475 is Obj(...)` reads as three
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
turning `A` back into 2, `B` into 3, `C` into 8, `D` into 7, and `E` into 6.
The pair grids, both instance masks, and the snapshots hold numbers. The
changed declarations now read:

```nuc
$Obj.corner_colors is [[3, 6], [8, 7]]
```

The `is` assignments and bare words of the previous section are unchanged.
Offline, with the accepted runtime at `1a41ca7` (last runtime change
`f463d94`), all 5 input and 5 output snapshots of the 5 pairs replay, each
equal to its letter snapshot with every letter read as its number. The pair
grids equal the puzzle's pairs cell for cell, and the grids, masks, and
snapshots are again those of the version before the letters.
`Matrix.silhouette` again takes only the colors 0 to 9, and the orientation
calls' few-shot crops read exactly as they did with letters, because the
counts that rank them are unchanged. No NUC was requested, so no proposal
strategy was selected and strategy learning did not apply. No solver run has
measured this version, so its difference is unknown, not zero.

## Runs

| Date | Command | Runtime | Exit status | Difference | Elapsed |
| --- | --- | --- | --- | --- | --- |
| 6-Nov-2026 12:22 UTC | `bun start a9f96cdd` | Accepted runtime at `cd7bd18` (last runtime change `7096171`), local | 1, a failed run | Unknown: `Error: OPENROUTER_API_KEY is required` before the first model request | 0.5s |

The run was `bun start a9f96cdd` from `arc/`, run once by `/arc-purposal`
with a 30-minute limit and `NUCLEOID_EXECUTABLE` unset, after
`bun install --frozen-lockfile` reported no changes and `bun run typecheck`
and `bun run lint` passed with the new default. It used the primary
checkout's debug runtime (`nucleoid 1.0.0`), current with `cargo build` at
`cd7bd18`, and the committed solver at `cd7bd18` with only its default puzzle
changed.

The solver stopped in its analysis stage, at the analyzer's first model
request, with `Error: OPENROUTER_API_KEY is required` from
`src/lib/openrouter.ts`, and exited with status 1 after 0.5 seconds by the
runner's clock, from 12:22:25.9 to 12:22:26.4 UTC. No OpenRouter key reached
the process: there is no `.env` in `arc/`, and the key was not set in the
invoking session's environment; the caller checked only whether they exist,
never their contents. The run therefore made no model request and produced no
prediction, so its difference from the expected output is unknown, not zero,
and it wrote no `logs/a9f96cdd.log.json`. The runtime's hash was the same
before and after the run, and no commit reached the primary checkout during
it. The invocation authorizes one run, so none was repeated. The puzzle
joined the **ARC puzzle report** workflow with this dataset, so no workflow
run is recorded yet.

None of these runs measured the orientation version of 6-Nov-2026: the user
chose offline verification for the batch, so its difference is unknown, not
zero.

## Open items

- **No scored run yet.** The only run failed before its first model request
  because no OpenRouter key was available locally. Measuring the puzzle needs
  `OPENROUTER_API_KEY` in `arc/.env` or the environment and a separately
  authorized `bun start a9f96cdd`, or a run of the **ARC puzzle report**
  workflow, which now lists the puzzle.
- **Stored filtered lists are circular.** On the accepted runtime at
  `cd7bd18`, a class rule that stores a list derived only from `@first`
  values, `$Obj.columns = [$Obj.x@first - 1, $Obj.x@first + 1].filter(column => column >= 0 and column < canvas_width)`,
  and a rule that reads it into the anchored property,
  `$Obj.x = min of $Obj.columns`, raise `TypeError: Circular Dependency` at
  the second rule, although every value the list reads is anchored to the
  input. This dataset writes the filter out in each rule instead. It joins
  the same-property population anchors that
  [ARC-NUC-e9afcf9a](ARC-NUC-e9afcf9a.md) records as circular. A later puzzle
  whose rules need the stored form can request a NUC through its own
  `/arc-dataset` handoff.
- **Block locals take the first instance's value.** On the same runtime,
  `class Box(side: int)` with `this.side = side`, the class-level block
  `{ doubled = $Box.side * 2; $Box.double = doubled }`, `a = Box(side=1)`, and
  `b = Box(side=5)` give `double` 2 for both instances, where `b`'s
  `$Box.side * 2` is 10. The specification's block cases, such as the one in
  which "retail is a local variable that is any purchase's price times 1.15",
  state a per-instance meaning but exercise one instance each. This dataset
  uses no block. The worker found it and the caller reproduced it; it is
  reported here for a Nucleoid fix or NUC, not applied.
- **Messages are not JSON-only.** Carried over from
  [ARC-NUC-e9afcf9a](ARC-NUC-e9afcf9a.md): since `f58e26a`, `src/lib/llm.ts`
  sends its system messages as JSON, but the user and assistant messages that
  `src/analyzer.ts` and `src/visualizer.ts` build still wrap JSON in text
  labels such as `train_dataset:`, `declarations:`, and `json_format:`, while
  the ARC instructions require every message to parse as one JSON document.
  Open question for the user: change those messages, or give the rule an
  exception for them. This dataset does not depend on it.
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

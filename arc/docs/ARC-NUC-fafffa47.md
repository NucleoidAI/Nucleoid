# ARC-NUC-fafffa47 -- Shared Panel Holes

Progress record for ARC training puzzle `fafffa47`: its instruction dataset,
the solver and Nucleoid changes it needed, its runs, and what is still open.

| Item | Status |
| --- | --- |
| Puzzle | [`data/training/fafffa47.json`](../data/training/fafffa47.json) |
| Instruction dataset | [`dataset.fafffa47.json`](../src/dataset/dataset.fafffa47.json): 5 original training pairs followed by 1 test pair, 12 whole panel-figure instances, written on 9-Nov-2026 |
| Output canvas | `FIT` |
| Nucleoid enhancements | None requested; accepted Nucleoid expresses the model. A quote-free NUC for other datasets' quotes was drafted and then withdrawn before evaluation, once the accepted [NUC 95](../../docs/nuc-0095.md) made those quotes expressible |
| Solver changes | Suggested, not applied: an output canvas mode that frames one panel, and class declarations that stay in `declarations[0]` |
| Solver runs | 1 local run on 9-Nov-2026: exit 1, a failed run, `ReferenceError: Obj is not defined` in the analysis stage; difference unknown |
| Updated | 9-Nov-2026 |

## Puzzle

Every training input is a 6 by 3 canvas, 6 rows of 3 columns, holding two
3 by 3 panels stacked vertically: a color-9 figure in the top panel and a
color-1 figure in the bottom panel. Each output is one 3 by 3 panel that lays
the two panels over each other at matching panel coordinates. A position
takes color 2 when both panels leave it empty, and background otherwise. The
panels' colors do not matter, only which positions each figure covers.

This puzzle was selected without an explicit ID. Among the 371 non-genesis
training puzzles without a canonical dataset, its largest training input or
output has 18 cells, the fewest, and its training matrices total 135 cells.
Selection used the training pairs alone.

## Gestalt overview

- **Whole grid.** Each input is two stacked silhouette panels, and each
  output is a single panel the size of one of them: the two become one
  registered overlay.
- **Figures.** The whole figure of each panel is an object, including any
  disconnected pieces. The cells it leaves open in its panel are its holes.
- **Relations.** The panels share one coordinate frame, each read from its
  own top-left corner. Only the positions each figure leaves open relate the
  two; colors and the figures' own boxes play no part.
- **Change.** The panels are laid over each other and only the positions
  that both leave open are painted color 2; everything a figure covers
  becomes background. It is the complement of the union that
  [ARC-NUC-dae9d2b5](ARC-NUC-dae9d2b5.md) composes.
- **World model.** Two whole panel figures per pair, each carrying its
  panel's open cells as `holes`, so the objects are the figures seen first.
  The overlay of `dae9d2b5`, which composes the figures' own masks, falls
  short: merging masks only adds nonzero cells, so no composition of the
  figures' masks gives the cells neither covers. Listing those cells from
  the figures alone would need a rule to enumerate a panel's cells, and
  accepted Nucleoid has no integer range for it. The holes, the figure read
  as a stencil, keep the model at the figure level.

## Dataset

The canonical dataset contains the five original training pairs in source
order followed by the original test pair at the same level. Every pair
retains its original patterns, positions, dimensions, and zero cells, and
both instance masks keep their complete canvases. Colors stay numbers, as the
puzzle holds them.

Each `Obj` takes the standard entries and `holes`, the `[column, row]` cells
of its panel that the figure leaves empty, counted from the panel's top-left
corner. One rule keeps the holes that every panel shares, with the
quantifier `all` of [NUC 62](../../docs/nuc-0062.md) over the class
population and the deep-equality `includes` of
[NUC 61](../../docs/nuc-0061.md); the others paint them color 2 and move
the box to their bounding box with the list extremes of
[NUC 69](../../docs/nuc-0069.md):

```nuc
# Orientation is the direction an object points, in degrees clockwise from north (0 up, 90 right, 180 down, 270 left)
# Each object is the whole figure of one panel, the top or bottom half of the canvas, including any disconnected pieces.
# holes are the [column, row] cells of its panel that the figure leaves empty, counted from the top-left corner of the panel.
class Obj(x: int, y: int, width: int, height: int, orientation: int, holes: list):
    this.x is x
    this.y is y
    this.width is width
    this.height is height
    this.orientation is orientation
    this.tag is l9100
    this.holes is holes

$Obj.tag is l8372

# Overlay the panels and keep the holes that every panel shares.
$Obj.points is $Obj.holes.filter(hole => all figure in Obj: figure.holes.includes(hole))

$Obj.fill_color is 2

$Obj.x is $Obj.points.min(point => point[0])

$Obj.y is $Obj.points.min(point => point[1])

$Obj.width is $Obj.points.max(point => point[0]) - $Obj.x + 1

$Obj.height is $Obj.points.max(point => point[1]) - $Obj.y + 1
```

Both figures of a pair reach the same output state apart from their `id` and
`holes`, and their identical output masks compose to the output. `l9100`
names a single shape, as in `dataset.28bf18c6.json`, and the new tag `l8372`
names the space every panel leaves open. Every assignment uses `is`, and the
tags are bare words, so the dataset holds no quoted string. The
[README paragraph](../README.md#instruction-datasets) describes the
annotations and native snapshots in full.

`FIT` crops the composed canvas to the shared holes, which span the whole
panel in only one training pair; `EXACT` keeps the two-panel input canvas.
Neither frames one panel, so the solver needs another output canvas mode for
this puzzle, as **Open items** describes. On the training pairs, every output
crop sits at its correct panel position, and only the final framing differs.

The worker froze the model from the training pairs before extracting the test
pair and applied it without revision. Its offline checks passed for pair order
and grids, full-canvas masks and recomposition, unique instance names, native
input and whole-world output snapshots, and the loader, which includes the
file among all 33 JSON instruction datasets in filename order. The caller
independently replayed every training pair's snapshots on the accepted
runtime at `4b10586`, and again at `67fc69d` after NUC 95, and compared its
grids with the puzzle's training pairs; these are offline checks, not a
live-model result. The puzzle ID was added once to the
reporting list in [`arc.yml`](../../.github/workflows/arc.yml); no workflow
was dispatched. No hint was given and no NUC was requested for the dataset,
so it selected no proposal strategy.

Every form the dataset uses was already in the
[dataset grammar](../nucleoid.ebnf), which still derives every program in
`src/dataset` and needs each of its forms, so the grammar did not change.

## Quote-free NUC

After the run, the invocation listed every quote in the datasets at `HEAD`
and in this dataset, 15 in all, none of them in this dataset, and checked
each bare spelling on the accepted runtime at `4b10586`. The tag of
`dataset.dae9d2b5.json`, `this.tag = 'l9100'`, reads the same as
`this.tag is l9100`, and the four object keys of the `fill_color` rule of
`dataset.27a28665.json` read the same bare; replaying those datasets with
the respellings reproduced every snapshot. The ten keyword arguments of
`dataset.27a28665.json`, such as `tag='l4262'`, raised
`ReferenceError: l4262 is not defined` bare, and no accepted, Draft or
Rejected NUC wrote them without quotes.

The invocation therefore prepared a quote-free NUC, drafted as NUC 95
*Keyword Argument Words*, a Syntax enhancement in single mode, in an
isolated candidate: a keyword argument whose value is one identifier reads
as a word under the word rule of [NUC 93](../../docs/nuc-0093.md). The user
authorized an empty-history genesis policy for its strategy trial, and the
selector chose `reuse_first`, the cold-start first arm; the proposer reused
the word marking of NUC 93 and [NUC 94](../../docs/nuc-0094.md).

Before the candidate was frozen, the pull ahead of its Rust improvement
pass brought in `67fc69d`, which accepted another session's
[NUC 95 Item Words](../../docs/nuc-0095.md): inside an `is` value, the
`name=value` arguments of bare-name calls are words. Every keyword-argument
quote here sits in such a call, as in
`obj3fc7d6382a47 is Obj(x=0, y=0, width=3, height=3, orientation=0, tag='l4262')`,
and on the accepted runtime at `67fc69d` the bare spelling reproduces every
snapshot of `dataset.27a28665.json`. No quote was left without a bare
spelling, so the draft was withdrawn before evaluation and never integrated,
and its number belongs to Item Words. Its strategy trial was never
evaluated, so learning was unscored: no reward and no checkpoint, and the
genesis remains the retained head.

## Runs

| Date | Command | Runtime | Exit status | Difference | Elapsed |
| --- | --- | --- | --- | --- | --- |
| 9-Nov-2026 07:27 UTC | `bun start fafffa47` | Accepted runtime at `4b10586` (last runtime change `f463d94`), local | 1, a failed run | Unknown: `Nucleoid execution failed: ReferenceError: Obj is not defined (line 11, column 1)` in the analysis stage | 62.6s |

The run was `bun start fafffa47` from `arc/`, run once by `/arc-purposal`
with a 30-minute limit and `NUCLEOID_EXECUTABLE` unset, after
`bun install --frozen-lockfile` reported no changes. It used the primary
checkout's debug runtime (`nucleoid 1.0.0`), built with `cargo build` at
`4b10586`, and the committed solver at `4b10586` with only its default puzzle
changed from `74dd1130` to `fafffa47`. `bun run typecheck` and `bun run lint`
pass with that default.

The solver stopped in its analysis stage, at the first training instance's
input state: `analyzer.value` runs `declarations[0]` alone with the
instance's implementation, and the declarations the model returned held the
dataset's three leading comment lines as their own first element, so the
program declared no class and its constructor line raised
`ReferenceError: Obj is not defined`. The process exited with status 1 after
62.6 seconds by the runner's clock, before any prediction, so its difference
from the expected output is unknown, not zero, and it wrote no
`logs/fafffa47.log.json`. No commit reached the primary checkout during the
run. The invocation authorizes one run, so none was repeated.

## Open items

- **No scored run yet.** The only run failed before any prediction. A later,
  separately authorized run measures the puzzle once the solver changes
  below exist.
- **Output canvas (suggested solver change).** Each output is one panel, the
  input's width and half its height, but `FIT` crops to the painted cells
  and `EXACT` keeps the input canvas. A third `output_canvas` mode would crop
  or pad the composed canvas from the top-left to a size the declarations
  derive, such as `canvas_width` by `canvas_height / 2`: in
  `src/lib/Matrix.ts` (`canvas`), `src/app.ts`, `src/visualizer.ts`
  (`state`), `src/analyzer.ts`, and the README. It rests on the training
  pairs alone.
- **Class declarations outside `declarations[0]` (suggested solver change).**
  `analyzer.value` (`src/analyzer.ts`) and `visualizer.value`
  (`src/visualizer.ts`) build input states from `declarations[0]` alone, so
  a model response that splits leading comments from the class leaves the
  class undeclared, as in this run. Validating the `declarations` response
  before running it, for example by requiring `declarations[0]` to declare
  every class and answering otherwise with a JSON error code within the retry
  budget, would keep that error visible and recoverable.
- **Instance fields and comments.** The dataset keeps the six position and
  orientation fields and the `#` comments that the arc-dataset profile and
  the README require, as `dataset.dae9d2b5.json` does. The other datasets
  dropped the fields in `d0e3eeda` (*Refine ARC few-shot prompts*) and their
  comments in `ac49bb38` (*Expose object dimensions to ARC visualizer*). No
  solver call receives the six fields, so they change no prompt, but the
  comments in `declarations` and `implementation` reach the model. Which
  format the datasets keep is a decision for the user.
- **README canvas list.** The README sentence that lists the `FIT` datasets
  names neither `dataset.dae9d2b5.json` nor `dataset.fafffa47.json`.
- **Quotes.** This dataset holds none. The 15 quotes of other datasets all
  have bare spellings on the accepted runtime at `67fc69d` and wait for a
  later invocation to respell them: `this.tag = 'l9100'` in
  `dataset.dae9d2b5.json` as `this.tag is l9100`, and in
  `dataset.27a28665.json` the four `fill_color` keys, such as `'l4262': 1`
  as `l4262: 1`, and the ten keyword arguments, such as `tag='l4262'` as
  `tag=l4262`.

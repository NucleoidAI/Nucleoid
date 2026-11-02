# nuc-arc

## Development

The application keeps `.ts` source files and runs directly on Node.js using
`tsx`, without generating JavaScript files. Primitive values carry inline
`string`, `number`, or `boolean` annotations on parameters, including callback
parameters, on local variables and loop counters, and on the return types of
synchronous functions. Everything else stays inferred: arrays, objects,
destructured bindings, callback results, async return types, and payload
fields whose type is checked at runtime. Runtime guards check values and
shapes, such as non-negative offsets and rectangular rows, without repeating
type checks like `typeof`, `Array.isArray`, or `Number.isSafeInteger`. Sources
declare no interfaces, type aliases, or generics, and the TypeScript check uses
non-strict inference.
Use Bun to install dependencies from the committed lockfile:

```sh
bun install
bun run typecheck
bun run lint
```

Do not add or retain automated ARC tests, including outside this directory.
Use type checking, linting, builds, and manual/offline evaluation instead.
Keep puzzle datasets, instruction examples, and training/held-out scoring intact.

OpenRouter is the only LLM provider. Set `OPENROUTER_API_KEY` in `.env`.
Optionally set `OPENROUTER_MODEL` (default: `qwen/qwen3.8-27b`) to a model that
supports JSON output. A request's `model` argument overrides that setting.
The legacy `LLM` selector is no longer needed.

Sampling defaults to `temperature: 0`, `top_k: 1`, `top_p: 1`, and `seed: 42`,
with frequency and presence penalties set to `0`. These settings favor greedy,
repeatable output. Parameter support and reproducibility depend on the model and
provider; a fixed seed does not guarantee identical responses. Repeatability also
requires identical prompts: `llm.generate` currently prepends a random
`control_id`.
See [OpenRouter's sampling parameters](https://openrouter.ai/docs/api/reference/parameters).

Requests send `X-OpenRouter-Cache: false` to disable OpenRouter response caching,
including caching enabled by a preset. This does not disable upstream providers'
automatic prompt/KV caching. See OpenRouter's
[response caching](https://openrouter.ai/docs/guides/features/response-caching)
and [prompt caching](https://openrouter.ai/docs/guides/best-practices/prompt-caching)
documentation for the distinction.

Run the ARC example with `bun run start`, which loads the default `puzzle_id`
in `src\app.ts`. [`/arc-purposal`](#arc-proposal-agent) sets that default to
each puzzle it runs. Pass a puzzle ID as the first argument, for example
`bun run start 0d3d703e`, to load `data\training\0d3d703e.json`. This makes
live OpenRouter calls; type checking, linting, and offline inspection do not
require model calls.

### Genesis puzzle

`3aa6fb7a` is the genesis puzzle, the training puzzle the solver was originally
built and [benchmarked](#benchmark) on. Instead of its own
`dataset.3aa6fb7a.json`, it learns from the four shared genesis datasets that
every run receives, described under
[Instruction datasets](#instruction-datasets).

### GitHub Actions reporting

The **ARC puzzle report** workflow in `.github\workflows\arc.yml` runs
manually from the repository's Actions tab. Configure the repository Actions
secret `OPENROUTER_API_KEY` before dispatching it; do not upload an environment
file. The workflow disables environment-file loading, builds the checked-out
Nucleoid runtime, installs ARC dependencies from the frozen Bun lockfile,
and runs `3aa6fb7a`, `28bf18c6`, `9af7a82c`, and
`0ca9ddb6` once each, with a thirty-minute limit per puzzle. These are live,
paid OpenRouter solver benchmarks, not automated ARC tests.

The run summary contains a table with each puzzle's result, exit code, and
elapsed seconds. `PASS` means exact grid equality, including shape; `FAIL`
means a nonzero exit from a mismatch, execution error, or timeout. Missing
exit records, such as after a setup failure, report `NOT RUN` instead of
inventing a score. Puzzle failures do not prevent later puzzles from running,
but any failure makes the workflow fail.
The `arc-puzzle-report` artifact retains solver logs and exit records for seven
days. Logs may contain expected held-out grids and must never feed back into
datasets, prompts, or NUC candidates.

Whenever `/arc-dataset` introduces a new dataset, it also adds that puzzle ID
exactly once to the workflow's `jobs.report.env.ARC_PUZZLES` list and keeps the
job timeout budget sufficient. Dataset and agent-configuration changes alone
do not dispatch the workflow. If `/arc-purposal` withdraws a candidate dataset,
it also removes only the reporting entry that invocation added.

## Terminal presentation

The terminal uses light and dark grey styling with a small bright green `nuc` wordmark
from [Nucleoid's avatar](https://avatars.githubusercontent.com/u/87045767?s=96&v=4).
The header title is **Nucleoid - Logic Language for World Models 🌱🐋🌍**.
It displays the runtime version reported by the Nucleoid executable's `--version`
option, using the same `NUCLEOID_EXECUTABLE` selection as program execution.
Two blank rows pad the header content above and below.
The header content is centered as one group, with its left inset set to
50% of the unused horizontal space. Its first two rows pair the unboxed
wordmark on the left with the title and `Inspired by Nature` on the right.
After a blank line, the workflow and runtime version share the same text
column. Narrow terminals use smaller artwork and stack the header when both
columns cannot fit. Artwork is embedded, with
no image downloads during a run. Square corners, straight grey borders, light
grey headings, and dark grey secondary text carry the theme throughout the run.
Warnings and errors retain their semantic colors, and ARC color IDs `0` through
`9` keep their standard grid palette.
Declaration/implementation code and JSON/object-state content use bright white;
panel frames, titles, and matrix colors retain their existing styling.
Panels expand up to 120 terminal columns, shrinking to fit narrower windows.

Immediately after the header, a puzzle preview shows the training input/output
pairs side by side, with the test input below. Narrow windows wrap complete
pairs onto additional rows; oversized pairs use the existing column-split matrix
panels instead of dropping cells. The opening preview never receives or displays
the expected test output. The final Expected/Result comparison remains unchanged.

Numbered analysis/visualization stages, boxed declarations and object states,
and color-coded ARC grids with row/column coordinates keep the run readable.
Wide grids split into column panels rather than dropping cells. The final
comparison keeps **Expected** before **Result**, followed by an
exact-match badge in green or a mismatch badge in red, the unchanged difference
score, and elapsed time.
Model token usage stays visible underneath the active step, and failures go to
standard error with a boxed stack trace.

Colors follow the terminal automatically. Set `NO_COLOR=1` for uncolored output;
redirected output is uncolored unless `FORCE_COLOR=1` is set. Styling uses
Node.js's built-in utilities. `string-width` measures terminal display cells,
and `wrap-ansi` wraps long output without breaking colors, so both vertical
borders stay aligned even with wide Unicode text. Formatting does not change
prompts, programs, matrices, provider settings, or scoring.

Milestone functions in `src\milestone.ts` mark each point in a run that shows
output, such as a stage, a step about to start, or a step's result, and are the
only callers of the `src\lib\terminal.ts` renderer. The analyzer, visualizer,
OpenRouter client, and entry point report progress only through them.

`milestone.state` displays the result of each batched state pass as a table:
one per training pair at the end of the analysis, titled
`Training pair N · State`, and one for the test objects, titled
`Test · State`, before their outputs are composed. Each row is an object that
`Obj[:]` returns after the declarative rules run, with `id` first and then one
column per property in the order the runtime reports them. Cells keep the
formatting of the boxed object states, so strings stay quoted, and a property
that an object lacks shows a grey `—`. Tables span the panel width. Wider
tables continue in panels titled `(continued)` that repeat `id`, and a cell
too wide for its panel wraps instead of being cut.

## Instruction datasets

The synchronous `document()` function in `src\dataset\dataset.ts` reads every
regular JSON file directly in that directory on each call, independently of
the working directory. Files are sorted by filename; extension matching is
case-insensitive, and `.env` files are excluded. The optional legacy puzzle-ID
argument remains accepted but does not filter the files. Every run receives
all instruction datasets, including the shared genesis examples. Each file
is validated and minified by removing whitespace between JSON tokens; string
contents, numbers, and key order stay exactly as written. The return value is
one JSON array, with a separate entry per file. New JSON files need no
additional imports or registration in the loader. An unreadable directory or
file, or malformed JSON, fails explicitly; JSON errors name the file.

Each non-genesis puzzle has exactly one `dataset.<puzzle_id>.json`, containing
all its original `train` pairs from `data\training\<puzzle_id>.json` in source
order, followed by all its original `test` pairs in source order. Test pairs
are additional entries at the same level as the training pairs, in the same
`dataset` array with the same structure and no marker. Input and output grids
preserve every original cell, color, position, zero border, and canvas
dimension. `/arc-dataset` infers one rule set from the training pairs and
annotates every pair with it, rather than synthesizing varieties or creating
numbered copies. `document()` includes a puzzle's own dataset in that puzzle's
runs, so a run of a puzzle that has a dataset receives its expected test
output in the few-shot context.

The genesis datasets are `dataset.core.json`, `dataset.extension.json`,
`dataset.curve.json`, and `dataset.line.json`. They are the shared examples of
the genesis puzzle `3aa6fb7a`, which has no `dataset.3aa6fb7a.json`, and every
run receives them with the other datasets. Their pairs are synthesized
varieties of corners, curves, and lines rather than original training or test
pairs, so they contain no test pair. Each ends with one synthesized pair that
holds a single object on the genesis puzzle's 7 by 7 canvas, annotated with
that dataset's own declarations: a color-9 corner at orientation 135 with its
outside color-4 marker (`dataset.core.json`), a color-4 corner at orientation
45 with its inside color-7 marker (`dataset.extension.json`), a color-9 point
with its color-2 rim below (`dataset.curve.json`), and a vertical line that
extends down from its top endpoint (`dataset.line.json`).

Instance `implementation` snippets use named constructor arguments whose names
match the parameters in their class declarations.

Every `input_instance_matrix` has exactly its pair's `input_matrix` row and
column counts, and every `output_instance_matrix` has exactly its pair's
`output_matrix` row and column counts, even when the two canvases differ.
Both masks are rectangular, with the object's cells at their canvas coordinates
and every other cell padded with `0`. The x/y metadata records the nonzero
bounding-box origin within the full mask, not an offset to apply again. This
storage framing is separate from `output_canvas`'s final composition/cropping mode.

Each dataset also sets `output_canvas`, how its output matrices are framed.
`EXACT` outputs keep the input matrix's dimensions, while `FIT` outputs are
cropped to the nonzero bounding box of their content. `dataset.27a28665.json`,
`dataset.28bf18c6.json`, `dataset.44f52bb0.json`, `dataset.9af7a82c.json`, and
`dataset.d631b094.json` are `FIT`; the other datasets are `EXACT`.

`dataset.0ca9ddb6.json` annotates the three training pairs from
`data\training\0ca9ddb6.json`, followed by its test pair, with fifteen point
instances in total (2, 5, 3, and 5 per pair).
Color-1 points gain color-7 orthogonal neighbors; color-2 points gain color-4
diagonal neighbors. Color-6 and color-8 points remain unchanged.
The declarative rules introduce `marker_color`, which is absent before the
rules run. Tags identify the geometry: `l1707` is an orthogonal cross,
`l2404` is a diagonal cross, and `l7544` remains a single point.
Every instance mask retains the complete 9 by 9 canvas. The x/y metadata
records each mask's nonzero bounding-box origin. Native output snapshots carry
the shifted pattern origin, derived from the original point coordinates with
`@first` anchors. Orientation is 0 in both states because
the patterns are symmetric. The test pair follows the training pairs at the
same level and is annotated by the same rules.
`document()` includes it in the shared few-shot context for every run.

`dataset.0d3d703e.json` annotates the four original training pairs from
`data\training\0d3d703e.json`, in order, followed by its test pair, fifteen
bar instances in total, three per pair. Every input and output retains its
original 3 by 3 canvas, colors, and positions. Each input holds three vertical
bars of distinct colors, one per column; each object is one bar, 3 cells tall
and 1 cell wide, listed from left to right. Each output keeps every bar in
place and swaps its color with its partner in a fixed pairing of the colors 1
and 5, 2 and 6, 3 and 4, and 8 and 9. The training pairs show every pairing
in both directions, and the test pair turns 8, 1, and 3 into 9, 5, and 4. Colors
0 and 7 never appear, and the rules leave them outside the pairing rather than
assume an identity. The class takes `color` as its only descriptive scalar,
with no canvas size or matrix of cells. One unconditional class-level rule
rewrites `color` from the input color, anchored with `@first`:
`[[1, 5], [2, 6], [3, 4], [8, 9]].find(pair => pair.includes($Obj.color@first)).find(partner => partner != $Obj.color@first)`
finds the pair that holds the input color and takes its other member, so each
pairing is stated once and serves both directions. Each bar recolors
independently of the other bars. The rules create no other properties,
so input and output states share the same keys, and leave `x` and `y`
unchanged because the bar stays in place. Native output snapshots are
captured with all of the same pair's implementations together, followed by
`Obj[:]`. The tag `l6953` is a vertical bar, as in `dataset.9af7a82c.json`
and `dataset.d037b0a7.json`, in both states. Input and output masks retain the
full 3 by 3 canvas, padded with `0` outside the bar, and merging a pair's masks
reproduces its input and output. The `output_x`/`output_y` fields record the
bar's nonzero bounding-box origin, equal to `output_value.x`/`output_value.y`.
`output_canvas` is `EXACT` because each output keeps its input's canvas.
Orientation is 0 in both states because the bars are vertical and not
rotated. The test pair follows the training pairs at the same level and is
annotated by the same rules.

`dataset.27a28665.json` annotates the seven original training pairs from
`data\training\27a28665.json`, in order, followed by its three test pairs,
fifty point instances in total, five per pair. Every input retains its
original 3 by 3 canvas, colors, positions, and zero cells. Each input holds one
five-cell pattern of a single color whose 3 by 3 bounding box spans the
canvas; each object is one of its cells, a single 1 by 1 point of that color,
listed in reading order. Each output is the original 1 by 1 grid, a code color
that depends only on the pattern's geometry, not on its color or cell count:
color 5 alone yields the codes 1, 2, and 6. The rules identify the geometry by
how many corners of the pattern's bounding box it fills: none, the orthogonal
cross, gives 6; one, the pattern with an empty center, gives 1; two, a 2 by 2
block with a diagonal tail, gives 3; and all four, the diagonal cross, give 2.
The first and sixth training pairs and the second test pair give 1, the second
and third training pairs and the third test pair give 2, the fourth and fifth
training pairs give 3, and the seventh training pair and the first test pair
give 6. The class takes `color` as its only descriptive scalar, with no canvas
size or matrix of cells. Unconditional class-level rules introduce three
properties, absent before the rules run, from the pair's whole population:
`at_corner` tests whether the point's input column is
`Obj.min(point => point.x@first)` or `Obj.max(point => point.x@first)` and its
input row is the population's smallest or largest `y@first`, a corner of the
pattern's bounding box; `corner_count` is
`Obj.filter(point => point.at_corner).length`; and `fill_color` is
`6 if $Obj.corner_count == 0 else 1 if $Obj.corner_count == 1 else 3 if $Obj.corner_count == 2 else 2`.
The population rules re-evaluate whenever an instance is added, so every point
of a pair records the same `corner_count` and `fill_color`. Other features,
such as the number of points without an orthogonal neighbor, also separate the
four training patterns; the rules count bounding-box corners, which needs
neither the canvas size nor the pattern's position. The rules also move `x`
and `y` to the output origin 0, and `@first` anchors keep the corner test on
the input coordinates. Native output snapshots are captured with all of the
same pair's implementations together, followed by `Obj[:]`. The tag `l7544` is
a single point, as in `dataset.0ca9ddb6.json` and `dataset.44f52bb0.json`, in
both states, because each point's output is the single output cell. Input
masks retain each pair's full-size, zero-padded 3 by 3 input canvas; every
point's output mask is the complete 1 by 1 output, and merging them reproduces
it. The `output_x`/`output_y` fields record the cell's nonzero bounding-box
origin, equal to `output_value.x`/`output_value.y`. `output_canvas` is `FIT`
because each output is cropped to its single cell instead of keeping the
3 by 3 input canvas. Orientation is 0 in both states because the points are
symmetric. The test pairs follow the training pairs at the same level and are
annotated by the same rules.

`dataset.28bf18c6.json` annotates the three original training pairs from
`data\training\28bf18c6.json`, followed by its test pair, one shape instance
per pair. Every input retains
its original 8 by 8 canvas and 3 by 3 shape, including its color and position.
Each shape is left-right asymmetric, so the pairs distinguish repeating from
mirroring.
Each output places two copies of the shape's crop side by side on a new canvas
twice as wide as the crop: the original 3 by 6 output. Unconditional
class-level rules hold for every instance, whatever its `color`: they introduce
`repeat_count` (2, the total number of copies) and `repeat_direction`
(`right`), which are absent before the rules run, and move `x` and `y` to the
output origin 0. Tags identify the geometry: `l9100` is a single shape and
`l4305` is the shape repeated side by side. Input masks retain each pair's
input canvas and output masks its output canvas. Orientation is 0 in both
states because the shapes are not rotated. The test pair follows the training
pairs at the same level and is annotated by the same rules.

`dataset.44f52bb0.json` annotates the six original training pairs from
`data\training\44f52bb0.json`, in order, followed by its two test pairs,
thirty-two point instances in total (5, 3, 6, 2, 4, 3, 7, and 2 per pair).
Every input retains its original 3 by 3 canvas, colors, positions, and zero
cells. Each object is one nonzero input cell, a single 1 by 1 point of color
2, listed in reading order. Each output is the original 1 by 1 grid: a color-1
cell when the input is left-right mirror symmetric, meaning every point has a
point of its color in the same row at the mirrored column of the canvas, and a
color-7 cell otherwise. The first, third, and fourth training pairs and the
first test pair are symmetric. The training pairs alone also fit a top-bottom
mirror; the rules test the left-right one. The class takes `color` and
`canvas_width`, the width of the object's canvas, as scalars rather than a
matrix of cells. Unconditional class-level rules introduce three properties,
absent before the rules run, from the pair's whole population: `has_mirror`
uses `Obj.some(other => ...)` to look for a point of the same color in the
same input row at input column `$Obj.canvas_width - 1 - $Obj.x@first`;
`symmetric` is `Obj.every(point => point.has_mirror)`; and `fill_color` is
`1 if $Obj.symmetric else 7`. Both population rules re-evaluate whenever an
instance is added, so every point of a pair records the same `symmetric` and
`fill_color`. The rules also move `x` and `y` to the output origin 0, and
`@first` anchors keep the mirror test on the input coordinates. Native output
snapshots are captured with all of the same pair's implementations together,
followed by `Obj[:]`. The tag `l7544` is a single point, as in
`dataset.0ca9ddb6.json`, in both states, because each point's output is the
single output cell. Input masks retain each pair's full-size, zero-padded
3 by 3 input canvas; every point's output mask is the complete 1 by 1 output,
and merging them reproduces it. The `output_x`/`output_y` fields record the
cell's nonzero bounding-box origin, equal to `output_value.x`/`output_value.y`.
`output_canvas` is `FIT` because each output is cropped to its single cell
instead of keeping the 3 by 3 input canvas. Orientation is 0 in both states
because the points are symmetric. The test pairs follow the training pairs at
the same level and are annotated by the same rules.

`dataset.5582e5ca.json` annotates the three original training pairs from
`data\training\5582e5ca.json`, in order, followed by its test pair, four block
instances in total, one per pair. Every input and output retains its original
3 by 3 canvas, colors, and positions, including the zero cell at the bottom
right of the first training input and of the test input. Each input holds one
multicolored block whose nonzero cells are edge-connected across colors and
whose 3 by 3 bounding box spans the canvas; each object is that whole block,
8 or 9 cells, and zero cells are background. Each output fills the block's
whole 3 by 3 footprint, zero cell included, with the color that has the most
cells in the block: three cells in every pair, a unique maximum, giving the
colors 4, 9, 6, and 8. The numerically largest color, 8, 9, 9, and 9, differs
from the fill in the first and third training pairs and the test pair, so the
rules maximize a count, not a color. The class takes `parts`, a flat list of
the block's nonzero cell colors in reading order, such as
`[4, 4, 8, 6, 4, 3, 6, 3]` for the first training input, as its only
descriptive property, with no count, histogram, fill color, canvas size, or
matrix of cells, so no input property names a count or the fill color.
Unconditional class-level rules introduce three properties, absent before the
rules run: `sorted_parts` is `$Obj.parts.slice().sort()`, a sorted copy that
groups equal colors and leaves `parts` in reading order; `part_counts` is
`$Obj.parts.map(part => $Obj.sorted_parts.lastIndexOf(part) - $Obj.sorted_parts.indexOf(part) + 1)`,
how many parts share each part's color, the length of that color's run in the
sorted copy; and `fill_color` is
`$Obj.parts[$Obj.part_counts.indexOf($Obj.part_counts.max())]`, the color of
the first part whose count is the largest. On a tie, which no pair has, the
tied color whose first cell comes first in reading order would win. Accepted
Nucleoid callbacks do not capture an enclosing callback's parameter, so the
direct count `$Obj.parts.map(part => $Obj.parts.count(other => other == part))`
raises `ReferenceError("part is not defined")`; the sorted copy states the same
count with callbacks that read only their own parameter and `$Obj`. The rules
leave `x` and `y` unchanged because the fill starts at the block's origin, 0
and 0. Native output snapshots are captured with the pair's implementation
followed by `Obj[:]`. Tags identify the geometry: `l9100` is a single shape,
as in `dataset.28bf18c6.json`, and `l5236` is a solid rectangle that fills the
whole canvas. Input masks retain each pair's full-size 3 by 3 input canvas,
with the zero cell padded `0`, and output masks the complete 3 by 3 output;
merging them reproduces each pair's input and output. The
`output_x`/`output_y` fields record the fill's nonzero bounding-box origin,
equal to `output_value.x`/`output_value.y`. `output_canvas` is `EXACT` because
each output keeps its input's canvas. Orientation is 0 in both states because
the block is not rotated and the fill is symmetric. The test pair follows the
training pairs at the same level and is annotated by the same rules.

`dataset.6150a2bd.json` annotates the two original training pairs from
`data\training\6150a2bd.json`, in order, followed by its test pair, twelve
object instances in total (4, 3, and 5 per pair). Every input and output
retains its original 3 by 3 canvas, colors, positions, and zero cells. Each
object is one 4-connected region of a single color, listed in the reading
order of its first cell: a three-cell corner in a 2 by 2 box, a horizontal and
a vertical two-cell bar, and single cells. Each output turns its input a half
turn about the canvas center, so every object keeps its color and cell count
while its shape turns 180 degrees and its bounding box moves to the opposite
side of the canvas. The class takes `color`, the bounding box's `width` and
`height`, and the `canvas_width` and `canvas_height` of the object's canvas as
scalars. Unconditional class-level rules read each object's input origin with
`@first` anchors: `$Obj.canvas_width - $Obj.x@first - $Obj.width` gives `x`,
`$Obj.canvas_height - $Obj.y@first - $Obj.height` gives `y`, and
`($Obj.orientation@first + 180) % 360` gives `orientation`. The rules create
no other properties, so input and output states share the same keys. The
`output_x`/`output_y` fields record the turned object's bounding-box origin,
equal to `output_value.x`/`output_value.y`. The tag `l9100` is a single shape,
as in `dataset.28bf18c6.json`, in both states. Orientation is 0 in every input
state, where no object is turned, and 180 in every output state, because the
rules turn every object a half turn, including single cells and bars whose
turned shape looks the same. Input and output masks retain the full 3 by 3
canvas. `output_canvas` is `EXACT` because each output keeps its input's
canvas, including the empty top row of the second training output. The test
pair follows the training pairs at the same level and is annotated by the same
rules.

`dataset.9af7a82c.json` annotates the four original training pairs from
`data\training\9af7a82c.json`, followed by its test pair, nineteen region
instances in total (3, 4, 4, 3, and 5 per pair). The fully colored input
canvases retain their original sizes: 3 by 3, 3 by 4, 4 by 3, 4 by 3, and
4 by 4, rows by columns. Each color is one region,
with its original cells and position. Cell counts range from 1 to 6 and are
distinct within each pair. Each output is a histogram: the region
with the r-th largest cell count becomes a vertical bar of its color in
column r, starting at the top row and as tall as the region's cell count,
giving the original 5 by 3, 6 by 4, 5 by 4, 6 by 3, and 6 by 5 outputs. The
class takes
`color` and `cell_count`, the region's number of cells, as scalars rather than a
matrix of cells. Unconditional class-level rules introduce `bar_height` (the
cell count), which is absent before the rules run, move `y` to the output
origin 0, and derive `x`, the bar's column, from the pair's whole population:
`Obj.filter(other => other.cell_count > $Obj.cell_count).length` counts the
larger regions and re-evaluates whenever an instance is added. Each pair lists
its instances largest first. Native output snapshots are captured with all of
the same pair's implementations together, followed by `Obj[:]`, so every
recorded `x` counts all larger regions of its pair. Tags
identify the geometry: `l9100` is a single shape, as in
`dataset.28bf18c6.json`, and `l6953` is a vertical bar. Input masks retain
each pair's full-size, zero-padded input canvas; output masks retain each
pair's full-size output canvas padded with `0` outside the bar. The
`output_x`/`output_y` fields record the bar's nonzero bounding-box world origin,
equal to `output_value.x`/`output_value.y`, not offsets to apply again.
`output_canvas` is `FIT` because each histogram fills its nonzero bounding box,
and two outputs are wider than their inputs. Orientation is 0 in both states
because the regions are not rotated and the bars are vertical. The test pair
follows the training pairs at the same level and is annotated by the same
rules.

`dataset.d037b0a7.json` annotates the three original training pairs from
`data\training\d037b0a7.json`, in order, followed by its test pair, eleven
point instances in total (3, 3, 2, and 3 per pair). Every input and output
retains its original 3 by 3 canvas, colors, positions, and zero cells. Each
object is one nonzero input cell, a single 1 by 1 point of its own color,
listed in reading order; no two points of a pair share a column, so their bars
never overlap. Each output extends every point downward into a vertical bar of
its color, from the point's row to the bottom row of the canvas, so each bar is
1 to 3 cells tall and a point on the bottom row stays a single cell. The class
takes `color` and `canvas_height`, the height of the object's canvas, as
scalars rather than a matrix of cells. Unconditional class-level rules
introduce `bar_height`, which is absent before the rules run, as
`$Obj.canvas_height - $Obj.y`, and leave `x` and `y` unchanged because the
point is the bar's top cell. The bottom edge comes from the canvas, not the
population: the second training pair's lowest point lies on the middle row,
yet its bars reach the bottom row. Native output snapshots are captured with
all of the same pair's implementations together, followed by `Obj[:]`. Tags
identify the geometry: `l7544` is a single point, as in
`dataset.0ca9ddb6.json`, and `l6953` is a vertical bar, as in
`dataset.9af7a82c.json`. Input masks retain each pair's full-size,
zero-padded input canvas; output masks retain each pair's full-size output
canvas padded with `0` outside the bar. The `output_x`/`output_y` fields
record the bar's nonzero bounding-box origin, equal to
`output_value.x`/`output_value.y`. `output_canvas` is `EXACT` because each
output keeps its input's canvas, including the empty right column of the third
training output. Orientation is 0 in both states because the points are
symmetric and the bars are vertical. The test pair follows the training pairs
at the same level and is annotated by the same rules.

`dataset.d631b094.json` annotates the four original training pairs from
`data\training\d631b094.json`, in order, followed by its test pair, five
region instances in total, one per pair. Every input retains its original
3 by 3 canvas, colors, positions, and zero cells. Each input holds a single
nonzero color, and all cells of that color form one region, whether or not
they touch: 2, 3, 1, 4, and 5 cells per pair. Each output is a horizontal bar
of the region's color, one row high and as wide as the region's cell count,
giving the original 1 by 2, 1 by 3, 1 by 1, 1 by 4, and 1 by 5 outputs. The
class takes `color` and `cell_count`, the region's number of cells, as
scalars rather than a matrix of cells. Unconditional class-level rules
introduce `bar_width` (the cell count), which is absent before the rules run,
and move `x` and `y` to the output origin 0. Tags identify the geometry:
`l9100` is a single shape, as in `dataset.28bf18c6.json`, and `l3058` is a
horizontal bar. Input masks retain each pair's full-size, zero-padded 3 by 3
input canvas; output masks retain each pair's full-size output canvas. The
`output_x`/`output_y` fields record the bar's nonzero bounding-box origin,
equal to `output_value.x`/`output_value.y`. `output_canvas` is `FIT` because
each output is cropped to the bar. Orientation is 0 in both states because
the regions are not rotated and the bar is a distinct horizontal geometry.
The test pair follows the training pairs at the same level and is annotated
by the same rules.

## ARC proposal agent

Run `/arc-purposal [<puzzle-id>]` to take one training puzzle through the
[arc-purposal](../.claude/agents/arc-purposal.agent.md) workflow in its
[skill](../.claude/skills/arc-purposal/SKILL.md):

1. The [arc-dataset](../.claude/agents/arc-dataset.agent.md) agent writes the
   puzzle's instruction dataset, using any dataset in `src\dataset` as a
   reference. It prefers declarative, higher-order rules over every object. If
   accepted Nucleoid cannot express the rules the training pairs need that way,
   it writes nothing and requests one Nucleoid enhancement instead,
   whose idea may come from the [language reference](../docs/README.md) or
   the documentation it links to.
2. A requested enhancement goes through the usual NUC proposal and independent
   evaluation stages as a single-proposal request. The dataset is then written
   with the frozen candidate's runtime.
3. `bun start <puzzle-id>` runs once, with that candidate's runtime when there
   is one, while the workflow waits up to 30 minutes for it. Exit status `0` is
   an exact match; a run still going at 30 minutes is killed and does not pass.
4. The requested NUC follows the automatic acceptance rule. A compile error, a
   failing test or a backward-compatibility break rejects it. Otherwise an
   exact match or an evaluator acceptance makes it Accepted, and any other
   result makes it Automatic-Accepted, with the policy results recorded in the
   NUC.
5. The workflow itself never changes the solver's TypeScript, apart from
   making the puzzle the default `puzzle_id` in `src\app.ts` before the run.
   When the dataset, the candidate, or the run shows that the solver needs a
   change, the final report suggests it for review.
6. Finally, the workflow creates or updates the puzzle's progress record,
   `docs\ARC-NUC-<puzzle_id>.md`, in the format of the existing records such as
   [ARC-NUC-0ca9ddb6](docs/ARC-NUC-0ca9ddb6.md): the dataset, any NUC and its
   outcome, every run's exit status and difference, and the open items. The
   record keeps earlier runs and stays uncommitted for review with the
   dataset.

Without a puzzle ID, the workflow chooses one from `data\training`. It skips
the [genesis puzzle](#genesis-puzzle) and every puzzle that already has a
`dataset.<puzzle_id>.json`, then takes the puzzle whose largest training
matrix has the fewest cells, breaking ties by the fewest total training cells
and then by the lowest ID. Only training pairs are measured.

Append `--hint <text>` to suggest the puzzle's objects, their roles, or its
transformation. Only the arc-dataset agent receives the hint, and it keeps only
what the training pairs confirm.

The run makes paid OpenRouter calls and shows the expected test grid, so it
runs once, is never repeated to obtain a pass, and its output never feeds back
into the dataset or the candidate. The
[ARC instructions](../.claude/rules/arc.instructions.md) require an
object-oriented Nucleoid world model, training-only rule inference, and
exact-grid evaluation without test-answer leakage.

ARC LLM exchanges follow **JSON data in, JSON data out** for every message,
including injected examples and retry feedback. Natural-language
prompts and prose hidden in JSON values are forbidden; executable Nucleoid code
travels in explicit JSON string fields. Analyzer and visualizer requests,
injected instruction examples, and control IDs each serialize one complete
JSON document. Responses are validated before symbolic execution or composition.

## JSONL Review Tools

Run `bun run tools` and open **http://127.0.0.1:3000**. This builds the browser
TypeScript and starts a local server; no API keys or additional dependencies are
needed. The existing ARC example remains available through `bun run start`.
Set `PORT` to use a different port.

Choose or drop a `.jsonl` or `.ndjson` file containing one valid JSON value per
line. Blank lines are ignored; invalid JSON reports its source line and leaves
the current dataset intact. Every imported record starts in **Unsupervised**.
Search or page through the list and select a record to see its formatted,
readable JSON. Ordinary arrays containing only primitive values stay on one
line, so ARC matrices display one row per line. String values in `code`,
`declarations`, `implementation`, and `output_code` appear as indented, multiline
code, including nested properties and arrays of snippets. Code arrays expand
one entry per block in the readable view. Nucleoid's `'use declarative'` and
`'use imperative'` directives also identify code strings in other properties.
This is display-only: code is never executed, and the saved strings are
unchanged. If a code snippet cannot be formatted, the view shows the original
string with a formatting error.
Click **Edit JSON** to edit valid JSON, or **Readable view** to preview edits
before saving.

Review or edit a record, then click **Mark supervised** (or press
**Ctrl/Cmd + Enter**). Its saved JSON moves to the **Supervised** list, and the
next unsupervised record is selected. Labeling is review status only: no label
or other metadata is added to your JSON. Supervised records can be edited and
saved or moved back to Unsupervised.

Use **Download supervised** and **Download unsupervised** to export the lists
separately as `<filename>.supervised.jsonl` and
`<filename>.unsupervised.jsonl`. Exports retain source order, preserve duplicate
records, and contain compact JSON with one record per line. An empty list
downloads as an empty file. Save or reset pending edits before downloading.
Formatting preserves number precision, duplicate object keys, and string
escapes.

Files are processed entirely in the browser, not uploaded or stored on the
server. Review progress lasts for the current page session, so download both
lists before closing or reloading. The app warns before discarding unsaved
edits or unexported review changes. Reimporting an export treats its records as
unsupervised again.

## ARC Benchmark

The ARC benchmark provides a dataset and evaluation benchmark designed to test the reasoning abilities of AI models. The Abstraction and Reasoning Corpus (ARC) benchmark, introduced by François Chollet, tests an AI's ability to demonstrate general intelligence rather than task-specific performance. Unlike traditional benchmarks that focus on training models on large, fixed datasets, ARC assesses how well an AI system can generalize, reason abstractly, and solve novel problems using limited information—skills akin to human cognitive abilities. It challenges AI to learn and adapt without task-specific training, emphasizing flexibility and the capacity to understand and apply abstract relationships. Designed with problems that a human can solve using common sense and basic reasoning, ARC remains difficult for AI, highlighting the gap between current models and true human-like cognitive capabilities. This benchmark is seen as a significant step toward developing Artificial General Intelligence (AGI), as it requires AI systems to exhibit foundational skills like inductive reasoning, analogy-making, and adaptability, pushing AI research toward more human-like problem-solving abilities.

https://arcprize.org

![ARC Puzzle Example](https://github.com/user-attachments/assets/41701a8e-5639-4f35-96ae-f4815dbd59cc)

## Nucleoid Approach

Nucleoid aka `nuc` approaches Neuro-Symbolic AI with introducing an intermediate language. Briefly, Nucleoid is a declarative, logic-based, contextual runtime that tracks each statement in declarative syntax and dynamically creates relationships between both logic and data statements in the knowledge graph to used in decision-making and problem-solving process.

> **Essential intelligence** is integration of pattern, data and logic.

This concept is also introduced in *Thinking, Fast and Slow* by Daniel Kahneman, where System 1 operates through pattern recognition, while System 2 applies logical reasoning. Data acts as the bridge, enabling collaboration between these systems to yield insights based on both probabilistic and deterministic information. However, the real challenge lies in enabling effective collaboration between the two systems so they can understand and support one another.

We've found that using an intermediate, ubiquitous language is highly effective beyond NLP and CoT methods, as it allows both pattern recognition and logical reasoning to be represented through 5GL, facilitating seamless bidirectional communication between the systems.

### Process Flow

There are 2 sections in our approach: analysis and visualization. In analysis phase, the AI system aims generalize patterns and identifies instances in order to use in actual test, and in visualization, the extracted abstraction is applied to given test input.

> :zap: Solver proposals must exchange JSON data with the LLM, carrying `nuc`
> code in explicit string fields instead of natural-language prompts.

![ARC_Flow](https://github.com/user-attachments/assets/0b5132cb-8269-461d-b27b-2b84ec1dd640)

#### Analysis

##### 1. Find patterns and Represent in `nuc` lang

The LLM finds pattern for given training input and output data and creates declarations in `nuc` lang.
It also infers the `output_canvas` from the training pairs.

```json
{
  "declarations": [
    "class Obj(x: int, y: int, orientation: int, vertex_color: int, vertical_segment_color: int, horizontal_segment_color: int):\n    this.x = x\n    this.y = y\n    this.orientation = orientation\n    this.tag = 'l6747'\n    this.vertex_color = vertex_color\n    this.vertical_segment_color = vertical_segment_color\n    this.horizontal_segment_color = horizontal_segment_color\n    this.marker_color = 0",
    "if $Obj.vertex_color == 9 and $Obj.vertical_segment_color == 9 and $Obj.horizontal_segment_color == 9:\n    $Obj.tag = 'l6624'\n    $Obj.marker_color = 1"
  ],
  "output_canvas": "EXACT"
}
```

##### 2. Initialize Nucleoid session

Nucleoid session is initialized with the declarations in order to use for identified instances later on.

##### 3. Extract instances

Based on declarations in `nuc` lang, the LLM extracts instances.
`input_matrix` and `output_matrix` remain the complete world canvases.
Each instance matrix keeps the full canvas of its pair's input or output
matrix: the object's cells stay at their canvas coordinates and every other
cell is `0`, as in the [instruction datasets](#instruction-datasets). The core,
extension, curve, and line instruction datasets expose
`input_x`, `output_x`, `input_y`, `output_y`, `input_orientation`, and
`output_orientation` on each instance. The x/y fields give the zero-based
column/row of the object's nonzero bounding-box origin within its mask; the
orientation fields give each state's numeric angle. The current
transformations preserve orientation, so their input and output angles match.
Native snapshots use `x`, `y`, and `orientation`; `tag` replaces `shape`
with shortened identifiers such as `l6747`.
Matrices already follow the canvas axes; composition does not rotate them.

Angles run clockwise from north: `N = 0`, `NE = 45`, `SE = 135`, `S = 180`,
`SW = 225`, and `NW = 315`. Line orientations describe axes:
horizontal is 90 and vertical is 0; endpoint and extension directions remain
separate properties. Curve orientation gives the rim direction: 0 is up and
180 is down.

| Dataset | Descriptive properties |
| --- | --- |
| Core and extension | `vertex_color`, `vertical_segment_color`, `horizontal_segment_color`, `marker_color` |
| Line | `endpoint_position` (side of the color-8 endpoint), `extension_direction` |
| Curve | `vertex_color`, `rim_color` |

Color properties are categorical grid colors, not segment lengths or coordinates.

Every instance has `input_value` before the class rule and `output_value`
after it. Obtain the input snapshot using only the class declaration; the
complete declaration set immediately applies the rules to new instances.
Line `extension_direction` and curve `rim_color` are created
by those rules and are absent from their input snapshots rather than filled
with artificial defaults.

Both native snapshots keep `x` and `y` anchored to the input bounding box;
`output_x` and `output_y` record the output bounding box's origin instead.
In `dataset.core.json`, adding the exterior marker grows the output to
3 by 3: subtract one from the input bounding box's x origin for
`input_orientation` 225 or 315, and from its y origin for `input_orientation`
45 or 315.
The other origins are unchanged.
For lines, subtract one from x when `extension_direction` is `left`, or from
y when it is `up`; extensions to the right or down keep the input origin.
For curves, subtract one from x for either orientation, and from y only when
`orientation` is 0. These offsets are already recorded in `output_x`
and `output_y`; do not apply them a second time during placement.
The inner markers in `dataset.extension.json` do not change the
bounding-box origin.

```json
{
  "instances": [
    {
      "instance_name": "obj0",
      "input_x": 5,
      "output_x": 5,
      "input_y": 1,
      "output_y": 1,
      "input_orientation": 135,
      "output_orientation": 135,
      "input_instance_matrix": [
        [0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 9, 0],
        [0, 0, 0, 0, 0, 9, 9, 0],
        [0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0]
      ],
      "output_instance_matrix": [
        [0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 9, 0],
        [0, 0, 0, 0, 0, 9, 9, 0],
        [0, 0, 0, 0, 0, 0, 0, 1],
        [0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0]
      ]
    }
  ]
}
```

##### 4. Create instances in Nucleoid session

The LLM also represents identified instances in `nuc` lang. This step is crucial because this completes **knowledge packet**, which contains visual as well as logical representation of the instances.

```json
{
  "instances": [
    {
      "instance_name": "obj0",
      "input_x": 5,
      "output_x": 5,
      "input_y": 1,
      "output_y": 1,
      "input_orientation": 135,
      "output_orientation": 135,
      "input_instance_matrix": [
        [0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 9, 0],
        [0, 0, 0, 0, 0, 9, 9, 0],
        [0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0]
      ],
      "output_instance_matrix": [
        [0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 9, 0],
        [0, 0, 0, 0, 0, 9, 9, 0],
        [0, 0, 0, 0, 0, 0, 0, 1],
        [0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0]
      ],
      "implementation": "obj0 = Obj(x=5, y=1, orientation=135, vertex_color=9, vertical_segment_color=9, horizontal_segment_color=9)",
      "input_value": {
        "id": "obj0",
        "x": 5,
        "y": 1,
        "orientation": 135,
        "tag": "l6747",
        "vertex_color": 9,
        "vertical_segment_color": 9,
        "horizontal_segment_color": 9,
        "marker_color": 0
      },
      "output_value": {
        "id": "obj0",
        "x": 5,
        "y": 1,
        "orientation": 135,
        "tag": "l6624",
        "vertex_color": 9,
        "vertical_segment_color": 9,
        "horizontal_segment_color": 9,
        "marker_color": 1
      }
    }
  ]
}
```

#### Visualization

##### 5. Extract instances from Test Input

This step is slight different then from extracting instances in analysis because now the LLM has to use instances list from previous step as a reference.
It returns cropped input objects with instance-level `input_x`, `input_y`,
and numeric `input_orientation`, matching the coordinates and `orientation`
in `input_value`,
plus `output_dimensions: { "width": ..., "height": ... }` inferred from
training pairs and the supplied input only. Expected test grids and their
dimensions are never part of prediction requests.

##### 6. Initialize Nucleoid session for test input

In order not to mix up instances with training dataset, new Nucleoid session required, but the same declarations is applied.

##### 7. Create test instances in Nucleoid session

For identified instances, the LLM represents instances in `nuc` lang, this is particularly similar to the step in analysis phase.

##### 8. Visualize output instance

For each input instance in test, the LLM generates output instance since it has all information needed from **knowledge packets**.

Finally, `Matrix.pad` places each output matrix at its output position before
`Matrix.merge` combines the masks. Padding expands the reference canvas when
needed to retain the entire placed matrix, and merging preserves the largest
height and width across its inputs. Zero is transparent; later nonzero cells
win on overlap, as before. An erased output object uses `[[0]]` with an explicit
output position.
`Matrix.canvas` then frames the merged result with the inferred
`output_canvas`: `EXACT` keeps the composed canvas, which starts at the test
input's dimensions, and `FIT` crops it to its nonzero bounding box with
`Matrix.crop`. Any other value fails explicitly.

`Matrix.crop(matrix)` returns a new matrix containing the smallest rectangle
around all nonzero cells. All-zero borders are removed from all four sides:
top, bottom, left, and right. Internal zeros and empty rows or columns between
occupied cells are preserved. Empty and all-zero matrices return `[]`. The input
is not modified. Ragged matrices are rejected explicitly.
This standalone utility does not change the full-size instance masks or return
position metadata. Use `Matrix.locate` to retain the original position with
the crop.

`Matrix.locate(matrix)` returns `{ matrix, x, y }`, pairing the same tight crop
with its zero-based top-left position in the original matrix: `x` is the column
and `y` is the row. Empty and all-zero matrices return
`{ matrix: [], x: 0, y: 0 }`. It uses the same validation as `Matrix.crop` and
does not modify the input. For a located result,
`Matrix.pad(result.matrix, rows, cols, result.x, result.y)` restores the crop
to its original position on a canvas at least as large as the supplied dimensions.

`Matrix.pad(matrix, rows, cols, left, top)` treats the requested dimensions as
minimums, placing the input at the required zero-based column and row offsets
and filling the remaining cells with zeros. Each dimension grows as needed to
include the entire input and its placement offset instead of rejecting an
undersized reference. It does not center or modify the input. Empty matrices
can be padded to an all-zero canvas. Ragged matrices and negative dimensions or
offsets are still rejected.

The same prediction path replays every training pair before predicting the
held-out input. A training mismatch stops the run. Held-out predictions are
frozen before expected outputs are accessed for exact-grid scoring, including
shape. Each object executes with the learned declarations in a fresh Nucleoid
process, so no training instance state carries into prediction.

The run prints `Difference: N` after the result and expected matrices. Each
different, missing, or extra cell adds one to the score: `0` means an exact
match, and higher scores are worse. An extra empty row counts as one shape
mismatch.

`bun run start` exits with the same score, so a caller can check the exit
status: `0` means an exact match. Exit statuses keep only eight bits, so scores
above 255 exit with `255` instead of wrapping around to `0`. A failed run
prints its error to standard error and exits with `1`.

## Benchmark

![Benchmark](https://github.com/user-attachments/assets/e9d30330-6d4f-461b-9b9e-c8d1b9eccaf5)

This benchmark demonstrates how LLMs respond to different prompting methods for ARC subtasks. Our observations indicate that using natural language prompts often yields inconsistent and unpredictable results, particularly in chain-of-thought (CoT) reasoning. However, switching to 5GL such as `nuc` lang, significantly increases accuracy, with LLM responses approaching deterministic behavior. Notably, `nuc` lang achieves this performance without extensive training requirements. This suggests that structured, high-level programming languages may be more effective for certain types of task prompting in LLMs compared to conventional natural language approaches.

---

**Nucleoid**

![3aa6fb7a_nuc](https://github.com/user-attachments/assets/98c5147e-d1b9-4a09-8c38-0c803d99ea55)


**ChatGPT o-1**

![3aa6fb7a_chatgpt-o1](https://github.com/user-attachments/assets/84febfe8-4c6d-4390-91f0-0bdca35edab0)

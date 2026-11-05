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
and runs every puzzle in `ARC_PUZZLES` once in parallel, with a thirty-minute
limit per puzzle. These are live,
paid OpenRouter solver benchmarks, not automated ARC tests.

The run summary contains a table with each puzzle's result, exit code, and
elapsed seconds. `PASS` means exact grid equality, including shape; `FAIL`
means a nonzero exit from a mismatch, execution error, or timeout. Missing
exit records, such as after a setup failure, report `NOT RUN` instead of
inventing a score. Puzzle failures do not cancel the other running puzzles,
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
pairs, so they contain no test pair. Each ends with two synthesized pairs on
the genesis puzzle's 7 by 7 canvas, annotated with that dataset's own
declarations. The first holds a single object: a color-9 corner at
orientation 135 with its outside color-4 marker (`dataset.core.json`), a
color-4 corner at orientation 45 with its inside color-7 marker
(`dataset.extension.json`), a color-9 point with its color-2 rim below
(`dataset.curve.json`), and a vertical line that extends down from its top
endpoint (`dataset.line.json`). The last holds two objects: color-9 corners
at orientations 45 and 315 with their outside color-4 markers, color-4
corners at orientations 225 and 135 with their inside color-7 markers, a
color-9 point with its color-2 rim below and a color-4 point with its color-8
rim above, and a vertical line that extends up from its bottom endpoint with
a horizontal line that extends left from its right endpoint.

Instance `implementation` snippets use named constructor arguments whose names
match the parameters in their class declarations. The canvas size is not one
of them: the solver declares `canvas_width` and `canvas_height` at the start of
every program it runs, as
[Initialize Nucleoid session](#2-initialize-nucleoid-session) describes, so
rules read it from those variables. Native snapshots are captured the same
way, with the pair's `input_matrix` size declared first.

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

`dataset.017c7c7b.json` annotates the three original training pairs from
`data\training\017c7c7b.json`, in order, followed by its test pair, four
pattern instances in total, one per pair. Every input retains its original
6 by 3 canvas, colors, positions, and zero cells, and every output its
original 9 by 3 canvas. Each input holds one color-1 pattern whose cells touch
through edges or corners, only through corners in the second training pair;
each object is that whole pattern, 9, 9, 8, and 10 cells, and zero cells are
background. Each pattern's box starts at the canvas's top-left corner and is
6 rows tall, as tall as its canvas, and 3, 3, 2, and 3 columns wide. Its rows
repeat down the box every 4, 2, and 3 rows in the training pairs and every 3
rows in the test pair. Each output grows the pattern down to 9 rows, one and a
half times its height, continuing it in color 2 while its top row stays in
place. Appending the pattern's first half, its top three rows, would
reproduce the second and third training outputs and the test output, whose
periods divide the height, but not the first training output: its pattern
repeats every four rows, so the rows added below repeat its rows 2 to 4, and
its row 7 is 0, 2, 2 rather than 2, 2, 0. The class takes `color`, the box's
`height`, and `period`, the number of rows after which the pattern starts
over, as scalars, with no parts list, canvas size, or matrix of cells.
`period` is the smallest number of rows such that every row of the box equals
the row that many rows below it, wherever that row is still inside the box;
like `height`, it is measured on the pattern's own box rather than the canvas.
Unconditional class-level rules set the output tag and introduce one
property, `extension_direction`, which is absent before the rules run and is
`down`, as in `dataset.line.json`.
The rules also rewrite `height` to `$Obj.height@first * 3 / 2`, 9 rows, and
`color` to 2, and leave `x` and `y` unchanged because the pattern keeps its
top-left origin. The training pairs alone also fit a fixed 9-row output; the
rules scale the pattern's height instead. Native output snapshots are captured
with the pair's implementation followed by `Obj[:]`. Tags identify the
geometry: `l9100` is a single shape, as in `dataset.28bf18c6.json` and
`dataset.6150a2bd.json`, and `l8384` is a pattern whose rows repeat every
`period` rows down to its height. Input masks retain each pair's full-size
6 by 3 input canvas, with zero cells padded `0`, and output masks the complete
9 by 3 output; merging them reproduces each pair's input and output. The
`output_x`/`output_y` fields record the grown pattern's nonzero bounding-box
origin, equal to `output_value.x`/`output_value.y`. `output_canvas` is
`EXACT`: the composed canvas starts at the input's 6 by 3 dimensions and grows
only to retain the pattern placed down to row 8, giving the 9 by 3 outputs,
while `FIT` would crop the empty right column of the third training output.
Orientation is 0 in both states because the pattern is not turned. The test
pair follows the training pairs at the same level and is annotated by the
same rules.

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
stripe instances in total, three per pair. Every input and output retains its
original 3 by 3 canvas, colors, and positions. Each input is three vertical
stripes side by side, each one column wide and three rows tall and each of its
own color, so no cell is zero; each object is one stripe, 3 cells of one
color, listed from left to right. Each output keeps every stripe in place and
swaps its color for its partner: the colors form four pairs, 1 and 5, 2 and 6,
3 and 4, and 8 and 9, whose two colors trade places. The training pairs show
every pair in both directions: 1 turns 5 in the first and 5 turns 1 in the
third, 2 turns 6 in the first, second, and fourth and 6 turns 2 in the third,
3 turns 4 in the first and second and 4 turns 3 in the fourth, and 8 turns 9
in the second and third and 9 turns 8 in the fourth; colors 0 and 7 do not
appear. Colors are categorical, and no arithmetic relates the partners, whose
differences are 4, 4, 1, and 1, so the rule consults the pairs as data. The
class takes `color` as its only descriptive scalar, with no partner color,
canvas size, or matrix of cells. One class-level rule rewrites `color` as
`[[1, 5], [2, 6], [3, 4], [8, 9]].find(pair => pair.includes($Obj.color@first)).find(partner => partner != $Obj.color@first)`:
the list states each pair once, the first `find` selects the pair that
includes the stripe's input color, and the second `find` takes the other color
of that pair. The `@first` anchors read the input color because the rule
rewrites `color` itself; a dictionary from every color to its partner would
state each pair twice. The two callbacks are chained rather than nested, so
each reads only its own parameter and the stripe. The rules create no
property and leave `x`, `y`, `orientation`, and the tag unchanged, so input
and output states share the same keys. Native output snapshots are captured
with all of the same pair's implementations together, followed by `Obj[:]`.
The tag `l6953` is a vertical bar, as in `dataset.d037b0a7.json` and
`dataset.9af7a82c.json`, in both states. Input masks retain each pair's
full-size 3 by 3 input canvas and output masks the complete 3 by 3 output,
each padded with `0` outside its stripe; merging them reproduces each pair's
input and output. The `output_x`/`output_y` fields record the stripe's
nonzero bounding-box origin, equal to `output_value.x`/`output_value.y`.
`output_canvas` is `EXACT` because each output keeps its input's canvas.
Orientation is 0 in both states because the stripes are vertical and not
turned. The test pair follows the training pairs at the same level and is
annotated by the same rules.

`dataset.25d8a9c8.json` annotates the four original training pairs from
`data\training\25d8a9c8.json`, in order, followed by its test pair, fifteen
row instances in total, three per pair. Every input and output retains its
original 3 by 3 canvas, colors, and positions, including the zero cells of
the outputs. Each input is three full-width rows stacked from top to bottom,
with no zero cell; each object is one row, 3 cells wide and 1 cell tall,
listed from top to bottom. Some rows hold a single color, and the others mix
two colors. Each output keeps every row in place: a row whose cells all share
one color becomes a solid color-5 bar, and every other row is cleared to the
background color 0, so seven rows become bars and eight are cleared. The
single-color rows are the top row of the first training input, the middle row
of the second, the middle and bottom rows of the third, the bottom row of the
fourth, and the top and bottom rows of the test input. Their colors, 4, 6, 4
and 9, and 1 in the training pairs, all turn 5, so the rule tests whether a
row is uniform, not which color it has. The training pairs alone also fit
reading each input as single-colored shapes and turning gray only the shapes
that are one row tall and as wide as the canvas; the rules judge whole rows
instead, the unit in which every output changes, since each output row is a
single color, a choice fixed from the training pairs before the test pair was
read. The class takes `parts`, the colors of the row's cells from left to
right, as a list, such as `[4, 4, 4]` for the top row of the first training
input, as its only descriptive property, with no color, color count, canvas
size, or matrix of cells, so no input property states whether the row is
uniform. One unconditional class-level rule introduces `fill_color`, which is
absent before the rules run, as
`5 if $Obj.parts.distinct().length == 1 else 0`: the list query `distinct` of
[NUC 71](../docs/nuc-0071.md) keeps each of the row's colors once, so exactly
one kept color means the row is a single color, and the conditional
expression of [NUC 59](../docs/nuc-0059.md) gives such a row color 5 and
every other row the background color 0. The rules create no other property
and leave `x`, `y`, `orientation`, and the tag unchanged, so each output
state adds only `fill_color` to its input state's keys. Native snapshots
start with the pair's `canvas_width` and `canvas_height`, and output
snapshots are captured with all of the same pair's implementations together,
followed by `Obj[:]`. The tag `l3058` is a horizontal bar, as in
`dataset.d631b094.json`, in both states. Input masks retain each pair's
full-size 3 by 3 input canvas and output masks the complete 3 by 3 output,
each padded with `0` outside its row, so a cleared row's output mask is all
zero; merging them reproduces each pair's input and output. The
`output_x`/`output_y` fields record the bar's nonzero bounding-box origin,
equal to `output_value.x`/`output_value.y`. A cleared row has no nonzero
output cell, so its fields record its explicit output position instead, the
row's unchanged origin, as an erased output object keeps one in
[composition](#8-visualize-output-instance). `output_canvas` is `EXACT`
because each output keeps its input's canvas, including the cleared top or
bottom rows that `FIT` would crop from every training output. Orientation is
0 in both states because the rows are not turned and the bar is a distinct
horizontal geometry. The declarations hold executable code only, without the
`#` comments of the other datasets. The test pair follows the training pairs
at the same level and is annotated by the same rules.

`dataset.25ff71a9.json` annotates the four original training pairs from
`data\training\25ff71a9.json`, in order, followed by its two test pairs, six
figure instances in total, one per pair. Every training input and output
retains its original 3 by 3 canvas, colors, positions, and zero cells, and the
test pairs keep their original grids as well. Each training input holds one
figure, 3 cells of a single color that touch through edges; each object is
that whole figure, and zero cells are background. The first two figures are
full-width horizontal bars of color 1, 1 by 3, on the top row and on the
middle row; the third and fourth are L-shaped trominoes in a 2 by 2 box, of
color 1 in the canvas's top-left corner and of color 2 in its top-right
corner. Each output moves the whole figure down one row and keeps its shape,
color, and columns, so nothing else changes. The step is one row, not a fall
to the bottom edge: the first bar stops on the middle row above the empty
bottom row, and the 2-row-tall trominoes also move one row rather than their
height. Rolling the canvas's rows down with wraparound would also reproduce
every training output, because every training input's bottom row is empty;
the rules move the whole figure instead, the unit seen first, a choice fixed
from the training pairs before the test pairs were read. The class takes no
descriptive property, with no color, size, cell list, canvas size, or matrix
of cells, because the rule needs none: the figure's cells travel unchanged in
its masks. One unconditional class-level rule rewrites `y` as
`$Obj.y@first + 1`, the row below the figure's input origin, anchored with
`@first` because the rule rewrites `y` itself. Each training grid holds a
single figure and shows no relation between objects, so no population
operation appears; the class-level rule moves every figure on its own. The
rules create no property and leave `x`, `orientation`, and the tag unchanged,
so input and output states share the same keys and differ only in `y`. Native
snapshots start with the pair's `canvas_width` and `canvas_height`, and output
snapshots are captured with the pair's implementation followed by `Obj[:]`.
The tag `l9100` is a single shape, as in `dataset.28bf18c6.json` and
`dataset.6150a2bd.json`, in both states. Input masks retain each pair's
full-size input canvas and output masks the complete output, each padded with
`0` outside its figure; merging them reproduces each pair's input and output.
The `output_x`/`output_y` fields record the moved figure's nonzero
bounding-box origin, equal to `output_value.x`/`output_value.y`.
`output_canvas` is `EXACT` because each training output keeps its input's
3 by 3 canvas, including the empty top row that `FIT` would crop from every
one of them. Orientation is 0 in both states of the training pairs because no
training figure is turned. The declarations hold executable code only,
without `#` comments. The test pairs follow the training pairs at the same
level and are annotated by the same rules.

`dataset.27a28665.json` annotates the seven original training pairs from
`data\training\27a28665.json`, in order, followed by its three test pairs,
ten block instances in total, one per pair. Every training input retains its
original 3 by 3 canvas, colors, positions, and zero cells, and every training
output its original 1 by 1 grid; the test pairs keep their original grids as
well. Each training input holds one block of a single color, 5 cells that
touch through edges or corners, whose 3 by 3 bounding box spans the canvas;
each object is that whole block, and zero cells are background. The training
blocks take four shapes: a hollow diamond, the four edge cells around an empty
center, with its top-left corner filled, in colors 5 and 4 (the first and
sixth pairs); a diagonal cross, the center and the four corners, in colors 8
and 5 (the second and third); a square with a diagonal tail, a 2 by 2 square
in the top-right corner with one cell touching its bottom-left corner, in
colors 1 and 8 (the fourth and fifth); and an orthogonal cross, the center and
its four edge neighbors, in color 5 (the seventh). Each output is a single
cell whose color is the code of the block's shape, whatever the block's color:
1 for the hollow diamond, 2 for the diagonal cross, 3 for the square with a
tail, and 6 for the orthogonal cross. Three shapes appear in two colors with
the same code, so the color is ignored. No measure of the shape yields the
code: every block has 5 cells; both crosses keep every mirror and quarter-turn
symmetry of the canvas yet give 2 and 6; the hollow diamond and the square
with a tail each mirror only across one diagonal yet give 1 and 3; and their
3, 5, 2, and 1 edge-connected pieces do not order the codes. Each block is
therefore one instance whose tag names its shape, and the rules look the code
up by that tag. The class takes `tag` as its only descriptive property, such
as `tag='l4262'` for the first training input, with no color, cell count,
parts list, canvas size, or matrix of cells. Its constructor stores the tag it
is given instead of a fixed initial tag, so the input state names the shape
and no input property names the code. One class serves every shape because the
solver reads output states with `Obj[:]`, which lists no instance of a
subclass. Tags identify the geometry: `l4262` is the hollow diamond with a
filled corner and `l3743` the square with a diagonal tail, while `l2404` is a
diagonal cross and `l1707` an orthogonal cross, as in `dataset.0ca9ddb6.json`.
Unconditional class-level rules set the output tag `l7544`, a single point, as
in `dataset.44f52bb0.json`, and introduce one property, `fill_color`, which is
absent before the rules run, as
`{'l4262': 1, 'l2404': 2, 'l3743': 3, 'l1707': 6}[$Obj.tag@first]`: the object
literal states each shape's code once, and object bracket access reads the
code of the block's input tag, anchored with `@first` because the other rule
turns the tag into `l7544`. The rules create no other properties, so each
output state adds only `fill_color` to its input state's keys, and they leave
`x` and `y` unchanged because the point sits at the block's origin, 0 and 0 in
every training pair. Native snapshots start with the pair's `canvas_width` and
`canvas_height`, and output snapshots are captured with the pair's
implementation followed by `Obj[:]`. Input masks retain each pair's full-size
input canvas, and output masks the complete output; merging them reproduces
each pair's input and output. The `output_x`/`output_y` fields record the
cell's nonzero bounding-box origin, equal to `output_value.x`/`output_value.y`.
`output_canvas` is `FIT` because each training output is cropped to its single
cell instead of keeping the 3 by 3 input canvas. Orientation is 0 in both
states of the training pairs because no block is turned and the point is
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

`dataset.3c9b0459.json` annotates the four original training pairs from
`data\training\3c9b0459.json`, in order, followed by its test pair, five
mosaic instances in total, one per pair. Every input and output retains its
original 3 by 3 canvas, colors, and positions, and no grid has a zero cell.
Each input is one mosaic of 9 cells that fills its canvas, in two or three
colors: 1, 2, and 8; 2, 4, and 9; 5 and 8; and 2, 3, and 9 in the training
pairs. Each object is that whole mosaic, the figure seen first, whose 3 by 3
box spans the canvas from its origin, 0 and 0. Each output turns the whole
mosaic a half turn, 180 degrees, so the cell in row r and column c moves to
row 2 - r and column 2 - c, and every color keeps its cell count. The half
turn is the only one of the square's eight turns and mirrors, the identity
included, that reproduces any training pair: no training mosaic looks the
same after a quarter turn, a half turn, a mirror across either center line,
or a flip across either diagonal. The class takes no descriptive property,
with no color, parts list, width, height, canvas size, or matrix of cells,
because the rule needs none: the mosaic's cells travel in its masks, and its
turn is its orientation. One unconditional class-level rule rewrites
`orientation` as `($Obj.orientation@first + 180) % 360`, the half-turn rule
of `dataset.6150a2bd.json`, anchored with `@first` because the rule rewrites
`orientation` itself. The rules leave `x` and `y` unchanged: a mosaic that
fills its canvas keeps its box in place under a half turn, whether about its
own center or the canvas's. The two further rules of `dataset.6150a2bd.json`,
which move the box about the canvas center as
`canvas_width - $Obj.x@first - $Obj.width` and
`canvas_height - $Obj.y@first - $Obj.height`, would reproduce every training
output as well, but every mosaic's `width` and `height` equal the canvas's,
so those properties would only restate the canvas size and those rules would
only return the input origin; the rules state the half turn alone, a choice
fixed from the training pairs before the test pair was read. Each training
grid holds a single mosaic and shows no relation between objects, so no
population operation appears. The rules create no property and leave `x`,
`y`, and the tag unchanged, so input and output states share the same keys
and differ only in `orientation`. Native snapshots start with the pair's
`canvas_width` and `canvas_height`, and output snapshots are captured with
the pair's implementation followed by `Obj[:]`. The tag `l9100` is a single
shape, as in `dataset.28bf18c6.json` and `dataset.6150a2bd.json`, in both
states. Input masks retain each pair's full-size 3 by 3 input canvas and
output masks the complete 3 by 3 output; with no zero cell, each mask equals
its pair's grid, and merging them reproduces each pair's input and output.
The `output_x`/`output_y` fields record the turned mosaic's nonzero
bounding-box origin, 0 and 0, equal to `output_value.x`/`output_value.y`.
`output_canvas` is `EXACT` because each output keeps its input's 3 by 3
canvas; with no zero cell to crop, `FIT` would frame the same grids.
Orientation is 0 in every input state, where the mosaic is not turned, and
180 in every output state. The declarations hold executable code only,
without `#` comments. The test pair follows the training pairs at the same
level and is annotated by the same rules.

`dataset.44f52bb0.json` annotates the six original training pairs from
`data\training\44f52bb0.json`, in order, followed by its two test pairs,
eight pattern instances in total, one per pair. Every input retains its
original 3 by 3 canvas, colors, positions, and zero cells, and every output
its original 1 by 1 grid. Each input holds one pattern of color-2 cells,
whether or not they touch; each object is that whole pattern, 5, 3, 6, 2, 4,
and 3 cells in the training pairs and 7 and 2 in the test pairs, and zero
cells are background. Each pattern's box starts at the canvas's left column,
and at its top row except in the fourth training pair and the second test
pair, where it starts one row down; the boxes are 3 by 3, 3 by 2, 3 by 3,
1 by 3, 2 by 3, 2 by 2, 3 by 3, and 2 by 1, rows by columns. Each output is a
single cell: color 1 when the pattern is left-right mirror symmetric, the
same after a flip across the vertical center line of the canvas, and color 7
otherwise; the first, third, and fourth training patterns are symmetric. No
cell count separates the outputs: 5, 6, and 2 cells give 1, while 3, 4, and 3
cells give 7. Symmetry about the pattern itself does not decide either: the
fifth training input, a Z shape, is a half turn of itself, and the sixth, an
L shape, mirrors itself across a diagonal of its own 2 by 2 box, yet both
give 7. The third and fourth training pairs rule out the canvas diagonals.
The training pairs alone still fit a top-bottom mirror, a half turn about the
canvas center, and their combinations with the left-right mirror, and,
because every symmetric training pattern spans the canvas's width, a
left-right flip of the pattern's own box. The rules flip across the canvas's
center line, by the convention that mirror symmetry is bilateral symmetry
across the grid's vertical axis, a choice fixed from the training pairs
before the test pairs were read. The class takes `points`, the
`[column, row]` canvas position of each of the pattern's cells in reading
order, as a list, such as `[[0, 0], [2, 0], [1, 1], [0, 2], [2, 2]]` for the
first training input. It takes no color, cell count, canvas size, symmetry
flag, output color, or matrix of cells, so no input property states the
symmetry or the answer; the rules read the canvas's width from the solver's
`canvas_width`. Unconditional class-level rules set the output tag and
introduce two properties, absent before the rules run. `symmetric` is
`all point in $Obj.points: $Obj.points.includes([canvas_width - 1 - point[0], point[1]])`:
the mirror image of every point, in the same row and as far from the right
edge as the point is from the left edge, is also one of the points, which
holds exactly when the pattern equals its mirror image. The rule uses the
quantifier `all` of [NUC 62](../docs/nuc-0062.md), which states the claim of
`every` directly, and `includes`, which matches the mirrored `[column, row]`
list against the points with the deep equality of
[NUC 61](../docs/nuc-0061.md). `fill_color` is `1 if $Obj.symmetric else 7`.
The rules also move `x` and `y` to the output origin 0, which changes `y` in
the fourth training pair and the second test pair. Native output snapshots
are captured with the pair's implementation followed by `Obj[:]`. Tags
identify the geometry: `l9100` is a single shape, as in
`dataset.28bf18c6.json` and `dataset.017c7c7b.json`, and `l7544` is a single
point, as in `dataset.0ca9ddb6.json` and `dataset.d037b0a7.json`. Input masks
retain each pair's full-size, zero-padded 3 by 3 input canvas, and output
masks the complete 1 by 1 output; merging them reproduces each pair's input
and output. The `output_x`/`output_y` fields record the cell's nonzero
bounding-box origin, equal to `output_value.x`/`output_value.y`.
`output_canvas` is `FIT` because each output is cropped to its single cell
instead of keeping the 3 by 3 input canvas. Orientation is 0 in both states
because the pattern is not turned and the point is symmetric. The test pairs
follow the training pairs at the same level and are annotated by the same
rules.

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
Unconditional class-level rules set the output tag and introduce one property,
`fill_color`, which is absent before the rules run, as `mode of $Obj.parts`.
That is the noun form of `$Obj.parts.mode()`: the accepted list query `mode`
from [NUC 75](../docs/nuc-0075.md) returns the most frequent item of `parts`,
the color with the most cells, matching items as `distinct` does. On a tie,
which no pair has, `mode` returns the tied color that occurs first in `parts`,
the one whose first cell comes first in reading order. The rules create no
other properties, so each output state adds only
`fill_color` to its input state's keys, and they leave `x` and `y` unchanged
because the fill starts at the block's origin, 0 and 0. Native output
snapshots are captured with the pair's implementation followed by `Obj[:]`.
Tags identify the geometry: `l9100` is a single shape, as in
`dataset.28bf18c6.json`, and `l5236` is a solid rectangle that fills the whole
canvas. Input masks retain each pair's full-size 3 by 3 input canvas, with the
zero cell padded `0`, and output masks the complete 3 by 3 output; merging
them reproduces each pair's input and output. The `output_x`/`output_y` fields
record the fill's nonzero bounding-box origin, equal to
`output_value.x`/`output_value.y`. `output_canvas` is `EXACT` because each
output keeps its input's canvas. Orientation is 0 in both states because the
block is not rotated and the fill is symmetric. The test pair follows the
training pairs at the same level and is annotated by the same rules.

`dataset.6150a2bd.json` annotates the two original training pairs from
`data\training\6150a2bd.json`, in order, followed by its test pair, three
block instances in total, one per pair. Every input and output retains its
original 3 by 3 canvas, colors, positions, and zero cells. Each object is the
whole block of its grid's nonzero cells, edge-connected across colors: 6, 4,
and 6 cells of four, three, and five colors, and zero cells are background.
Each block's bounding box is 3 cells wide and 3, 2, and 3 cells tall. Each
output turns its input block a half turn about the canvas center, so the block
keeps its colors and cell count while it turns 180 degrees and its bounding
box moves to the opposite side of the canvas. The 3 by 3 boxes span the canvas
and keep their origin, 0 and 0, while the second training block's box moves
down one row, from the top two rows to the bottom two. Redrawing the turned
block at its input origin, a turn about its own center, would not reproduce
that output. The class takes the bounding box's `width` and `height` as
scalars, with no color, parts list, canvas size, or matrix of cells; the rules
read the canvas's size from the solver's `canvas_width` and `canvas_height`.
Unconditional class-level rules read the block's input state with `@first`
anchors: `($Obj.orientation@first + 180) % 360` gives `orientation`,
`canvas_width - $Obj.x@first - $Obj.width` gives `x`, and
`canvas_height - $Obj.y@first - $Obj.height` gives `y`. The rules create
no other properties, so input and output states share the same keys. Native
output snapshots are captured with the pair's implementation followed by
`Obj[:]`. The tag `l9100` is a single shape, as in `dataset.28bf18c6.json` and
`dataset.5582e5ca.json`, in both states. Input masks retain each pair's
full-size 3 by 3 input canvas, with its zero cells padded `0`, and output
masks the complete 3 by 3 output; merging them reproduces each pair's input
and output. The `output_x`/`output_y` fields record the turned block's nonzero
bounding-box origin, equal to `output_value.x`/`output_value.y`.
`output_canvas` is `EXACT` because each output keeps its input's canvas,
including the empty top row of the second training output. Orientation is 0
in every input state, where the block is not turned, and 180 in every output
state. The test pair follows the training pairs at the same level and is
annotated by the same rules.

`dataset.74dd1130.json` annotates the four original training pairs from
`data\training\74dd1130.json`, in order, followed by its test pair, five
mosaic instances in total, one per pair. Every input and output retains its
original 3 by 3 canvas, colors, and positions, and no grid has a zero cell.
Each input is one mosaic of 9 cells that fills its canvas, in three colors:
1, 2, and 5; 2, 5, and 6; 5, 8, and 9; and 1, 2, and 6 in the training pairs.
Each object is that whole mosaic, the figure seen first, whose 3 by 3 box
spans the canvas from its origin, 0 and 0. Each output mirrors the whole
mosaic across its main diagonal, the line from its top-left corner to its
bottom-right corner, so the cell in row r and column c moves to row c and
column r: each input row becomes the output column of the same index, read
top to bottom as the row reads left to right, the three cells on the diagonal
stay in place, and every color keeps its cell count. The flip across the main
diagonal is the only one of the square's eight turns and mirrors, the
identity included, that reproduces any training pair: no training mosaic
looks the same after a quarter turn, a half turn, a mirror across either
center line, or a flip across either diagonal, and the flip across the other
diagonal, from the top-right corner to the bottom-left, reproduces none. No
turn reproduces a training output, so `orientation`, a clockwise angle from
north, cannot state the change: a mirror reverses the mosaic's handedness,
which no turn does. The class takes no descriptive property, with no color,
parts list, width, height, canvas size, or matrix of cells, because the rule
needs none: the mosaic's cells travel in its masks, and its flip is its
mirror axis. One unconditional class-level rule introduces `mirror_axis`,
which is absent before the rules run, as `135`: the axis the mosaic is
mirrored across, as an angle clockwise from north by the convention of line
orientations, where vertical is 0 and horizontal is 90, so the main diagonal,
which runs from northwest to southeast, is 135 and the other diagonal 45. The
rules leave `x` and `y` unchanged: a mosaic that fills its square canvas keeps
its box in place under a flip across its main diagonal, whether its own or the
canvas's. Two further rules, `$Obj.x = $Obj.y@first` and
`$Obj.y = $Obj.x@first`, which swap the box's origin as a flip across the
canvas's diagonal does, would reproduce every training output as well, but
every mosaic's origin is 0 and 0, so those rules would only return the input
origin; the rules state the flip alone, a choice fixed from the training pairs
before the test pair was read. Each training grid holds a single mosaic and
shows no relation between objects, so no population operation appears. The
rules create no other property and leave `x`, `y`, `orientation`, and the tag
unchanged, so each output state adds only `mirror_axis` to its input state's
keys. Native snapshots start with the pair's `canvas_width` and
`canvas_height`, and output snapshots are captured with the pair's
implementation followed by `Obj[:]`. The tag `l9100` is a single shape, as in
`dataset.28bf18c6.json` and `dataset.3c9b0459.json`, in both states. Input
masks retain each pair's full-size 3 by 3 input canvas and output masks the
complete 3 by 3 output; with no zero cell, each mask equals its pair's grid,
and merging them reproduces each pair's input and output. The
`output_x`/`output_y` fields record the mirrored mosaic's nonzero bounding-box
origin, 0 and 0, equal to `output_value.x`/`output_value.y`. `output_canvas`
is `EXACT` because each output keeps its input's 3 by 3 canvas; with no zero
cell to crop, `FIT` would frame the same grids. Orientation is 0 in both
states because the mosaic is mirrored, not turned, and `mirror_axis` states
the flip. The declarations hold executable code only, without `#` comments.
The test pair follows the training pairs at the same level and is annotated
by the same rules.

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
takes `color` as its only descriptive scalar, with no canvas size or matrix of
cells. Unconditional class-level rules set the output tag and introduce
`bar_height`, which is absent before the rules run, as
`canvas_height - $Obj.y`, reading the canvas's height from the solver's
`canvas_height`, and leave `x` and `y` unchanged because the point is the
bar's top cell. The bottom edge comes from the canvas, not the
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

Run `/arc-purposal [<puzzle-id>] [--strategy-policy <absolute-json-path>]`
to take one training puzzle through the
[arc-purposal](../.claude/agents/arc-purposal.agent.md) workflow in its
[skill](../.claude/skills/arc-purposal/SKILL.md):

1. The [arc-dataset](../.claude/agents/arc-dataset.agent.md) agent writes the
   puzzle's instruction dataset, using any dataset in `src\dataset` as a
   reference. It designs with gestalt perception: it starts from the big
   picture, each grid's figures and their relations, writes declarative
   statements at that level, and goes down to single cells only when the big
   picture cannot express what the training pairs show. It prefers
   declarative, higher-order rules over every object. If
   accepted Nucleoid cannot express the rules the training pairs need that way,
   it writes nothing and requests one Nucleoid enhancement instead,
   whose idea may come from the [language reference](../docs/README.md) or
   the documentation it links to.
2. A requested enhancement goes through the usual NUC proposal and independent
   evaluation stages as a single-proposal request, using the existing UCB1
   proposal-strategy learner. The dataset is then written with the frozen
   candidate's runtime.
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
6. The workflow then creates or updates the puzzle's progress record,
   `docs\ARC-NUC-<puzzle_id>.md`, in the format of the existing records such as
   [ARC-NUC-0ca9ddb6](docs/ARC-NUC-0ca9ddb6.md): the dataset, any NUC and its
   outcome, every run's exit status and difference, and the open items. The
   record keeps earlier runs.
7. Finally, the workflow commits its own changes in one commit and pushes it:
   the dataset, its README paragraph, its reporting-list entry, the default
   `puzzle_id`, and the progress record, as far as it changed and kept them,
   never another session's changes. A requested NUC's outcome has its own
   `NUC <number> <Title>` commit, pushed when the outcome is decided.

Without a puzzle ID, the workflow chooses one from `data\training`. It skips
the [genesis puzzle](#genesis-puzzle) and every puzzle that already has a
`dataset.<puzzle_id>.json`, then takes the puzzle whose largest training
matrix has the fewest cells, breaking ties by the fewest total training cells
and then by the lowest ID. Only training pairs are measured.

Every requested NUC requires a retained strategy-policy checkpoint, or an
explicit `--strategy-policy <absolute-json-path>` outside the repository.
For an initial trial, supply the empty-history policy from the
[existing NUC bandit contract](../CONTRIBUTING.md#reward-driven-proposal-strategies).
There is no automatic initialization/reset; missing or invalid state pauses
before drafting or a solver run. A dataset-only invocation needs no policy.
Selection happens before drafting, and the independent evaluator updates
from verified NUC usefulness/cost evidence before the ARC run. The session
learning result retains the latest checkpoint for the next authorized NUC;
unscored trials keep the prior head. ARC answers, scores and acceptance never
train this learner, which does not train LLM weights or authorize extra runs.

Append `--hint <text>` to suggest the puzzle's objects, their roles, or its
transformation, placing `--strategy-policy` before it when supplied. Only the
arc-dataset agent receives the hint, and it keeps only what the training pairs
confirm.

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

Every `nucleoid.run` call in `src\analyzer.ts` and `src\visualizer.ts` starts
its program with two declarations of the canvas's size, ahead of the
declarations and implementations:

```nuc
canvas_width = 3

canvas_height = 3
```

Their values are the column and row counts of the training pair's
`input_matrix`, or of the test input during visualization. Declarations read
the canvas size from these variables, as
`$Obj.bar_height = canvas_height - $Obj.y` in `dataset.d037b0a7.json` does, so
no class takes it as a parameter and no implementation passes it. They are
variables, not object properties, so the input and output states and `Obj[:]`
do not include them. A class parameter of the same name still shadows them
inside its constructor.

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
Its `canvas_width` and `canvas_height` are the test input's column and row
counts.

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

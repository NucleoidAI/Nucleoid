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
`src\lib\openrouter.ts` fixes the model to `qwen/qwen3.8-27b`, served only by
the `deepinfra/bf16` provider without fallbacks, with reasoning disabled and
JSON output requested through `response_format: { type: "json_object" }`. No
setting or request argument changes the model. The legacy `LLM` selector is no
longer needed.

Sampling defaults to `temperature: 0`, `top_k: 1`, `top_p: 1`, and `seed: 42`,
with frequency and presence penalties set to `0`. These settings favor greedy,
repeatable output. Parameter support and reproducibility depend on the model and
provider; a fixed seed does not guarantee identical responses. Repeatability also
requires identical prompts: `llm.generate` currently prepends a random
`control_id`.
See [OpenRouter's sampling parameters](https://openrouter.ai/docs/api/reference/parameters).

Requests disable OpenRouter response caching with `X-OpenRouter-Cache: false`,
which applies even when a preset enables caching, and `src\lib\openrouter.ts`
rejects any response whose `X-OpenRouter-Cache-Status` is `HIT`. Neither
OpenRouter nor DeepInfra offers a switch for the provider's own automatic
prompt/KV caching, which reuses a cached prefix only where a prompt matches it
exactly from its first token. `llm.generate` therefore keeps the random
`control_id` as the first message of every request, so another request's
cached prefix can cover at most the few tokens before that identifier. Each
response is checked for `usage.prompt_tokens_details.cached_tokens`: the model
usage line shows the reported count as `(N cached)`, and the terminal prints a
yellow `⚠` alert to standard error when a response reports cached tokens or
reports no count at all, so an unverified response is never taken as uncached.
See OpenRouter's
[response caching](https://openrouter.ai/docs/guides/features/response-caching)
and [prompt caching](https://openrouter.ai/docs/guides/best-practices/prompt-caching)
documentation for the distinction, and DeepInfra's
[prompt caching](https://docs.deepinfra.com/chat/prompt-caching).

Run the ARC example with `bun run start`, which loads the default `puzzle_id`
in `src\app.ts`. [`/arc-purposal`](#arc-proposal-agent) sets that default to
each puzzle it runs. Pass a puzzle ID, for example `bun run start 0d3d703e`,
to load `data\training\0d3d703e.json`. This makes live OpenRouter calls; type
checking, linting, and offline inspection do not require model calls.

Add the `--only-puzzle-dataset` flag, before or after the puzzle ID, to limit
the few-shot context to the genesis datasets and the puzzle's own dataset, for
example `bun run start 0d3d703e --only-puzzle-dataset`. The
[genesis puzzle](#genesis-puzzle), which has no dataset of its own, then
receives only the genesis datasets. The flag is off by default, so a run
otherwise receives all [instruction datasets](#instruction-datasets), and the
run log records it as `only_puzzle_dataset`. When the flag is on, the
[terminal header](#terminal-presentation) also shows `--only-puzzle-dataset`
below the puzzle ID; a run without it does not mention the flag.
`server.ts` reads its arguments with Node.js's built-in `util.parseArgs`,
without an additional library; an unknown option, or a value such as
`--only-puzzle-dataset=true`, fails the run before any model call.

### Genesis puzzle

`3aa6fb7a` is the genesis puzzle, the training puzzle the solver was originally
built and [benchmarked](#benchmark) on. Instead of its own
`dataset.3aa6fb7a.json`, it learns from the four shared genesis datasets that
every run receives, described under
[Instruction datasets](#instruction-datasets).

### GitHub Actions reporting

The **ARC Benchmark** workflow in `.github\workflows\arc.yml` runs
manually from the repository's Actions tab. Configure the repository Actions
secret `OPENROUTER_API_KEY` before dispatching it; do not upload an environment
file. The workflow disables environment-file loading. Its `build` job checks
the secret and the puzzle list, then builds the checked-out Nucleoid runtime
once and shares it with one `puzzle` job per entry of `ARC_PUZZLES`, up to 256,
GitHub Actions' matrix limit. Each puzzle job installs ARC dependencies from
the frozen Bun lockfile and runs its puzzle once under its own thirty-minute
limit, within a 35-minute job timeout, with at most `ARC_CONCURRENCY` puzzle
jobs (default `5`) running at a time. Change the workflow-level
`env.ARC_CONCURRENCY` to adjust that limit; no timeout depends on how many
puzzles are listed. These are live, paid OpenRouter solver benchmarks, not
automated ARC tests.

After every puzzle job ends, the `report` job publishes a run summary with a
table of each puzzle's result, exit code, and elapsed seconds. `PASS` means
exact grid equality, including shape; `FAIL`
means a nonzero exit from a mismatch, execution error, or timeout. Missing
exit records, such as after a setup failure, report `NOT RUN` instead of
inventing a score. A summary table at the bottom counts the `PASS`, `FAIL`,
and `NOT RUN` puzzles and their total. Puzzle failures do not cancel the other
puzzle jobs, but any failure makes the workflow fail.
The `arc-benchmark` artifact retains solver logs and exit records for seven
days; the per-puzzle `arc-benchmark-<puzzle_id>` artifacts it is merged from
expire after one day. Logs may contain expected held-out grids and must never
feed back into datasets, prompts, or NUC candidates.

Whenever `/arc-dataset` introduces a new dataset, it also adds that puzzle ID
exactly once to the workflow-level `env.ARC_PUZZLES` list. Dataset and
agent-configuration changes alone do not dispatch the workflow. If `/arc-purposal` withdraws a candidate dataset,
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
After a blank line, the workflow, the runtime version, and a grey
`Puzzle <puzzle-id>` line naming the running puzzle share the same text
column. When the run uses `--only-puzzle-dataset`, a grey line naming that
flag follows the puzzle line; otherwise the header does not mention the flag.
Narrow terminals use smaller artwork and stack the header when both
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
Milestone steps and panel titles include stage-local subnumbers, such as
`1.1 Inferring declarations`, restarting at each stage and run. Repeated
milestones retain the same second number and add a third occurrence number,
such as `1.3.1` and `1.3.2` for training-object segmentation. Continued grid
panels keep the same full number. The declarations header includes the
output canvas instead of adding a separate milestone. Segmented-object panels
use `Input` and `Output` without object-number labels. Model token usage remains
attached to its active milestone. Instance-specific steps, crop and
implementation headers, and input- and output-state headers include the
instance name. In the visualization, each test object's synthesis shows its
`Input crop` right before its `Implementation`, as in
`2.6.1 Input crop · <instance>` and
`2.7.1 Implementation · <instance>`. Its composition shows the same
`Input crop` again before its `Output crop`, under its own number, such as
`2.12.1`, rather than continuing the synthesis count, and an `Output state`
panel right after the `Output crop`: the object's row of the
`Visualization · State` table, before its `Placed output`.
The visualization's output-crop header also shows the object's output
orientation as an arrow emoji and its degrees, such as
`2.13.1 Output crop · <instance> · ➡️ 90°`, and every input- and
output-state header, in both stages, shows the orientation of the state it
displays the same way, such as `2.9.1 Input state · <instance> · ⬆️ 0°`. An
orientation that is not a multiple of 45 degrees shows `🧭` instead of an
arrow, and an object without a numeric `orientation` adds nothing to the
header.
Grid headers and the preview's matrix labels omit the grid's dimensions; its
row and column coordinates show its size.
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

Milestone functions in `src\lib\milestone.ts` mark each point in a run that shows
output, such as a stage, a step about to start, or a step's result, and are the
only callers of the `src\lib\terminal.ts` renderer. The analyzer, visualizer,
OpenRouter client, and entry point report progress only through them.

`milestone.state` displays exactly one `Analysis · State` table containing
objects from all training pairs, in pair order, after the analysis state passes
finish. One `Visualization · State` table shows the test objects before their
outputs are composed. Each row is an object that `Obj[:]` returns after the
declarative rules run, with `id` first and then one column per property in the
order first reported by the runtime. Strings stay quoted, and a property that
an object lacks shows a grey `—`. Numeric `orientation` values include the same
direction emoji and degrees used by state headers. Present `time` values are
prefixed with `⏱️`.

State tables are compact and never split into continued panels. Cells omit
horizontal padding, and column widths share the available space so all
properties stay in one table. Headers and values wrap without truncation.
Small tables use only the width their content and title need. If even the
minimum character widths and column separators cannot fit a narrow terminal,
the single table extends beyond that width rather than hiding columns.

## Instruction datasets

The synchronous `document()` function in `src\dataset\dataset.ts` reads every
regular JSON file directly in that directory on each call, independently of
the working directory. Files are sorted by filename; extension matching is
case-insensitive, and `.env` files are excluded. By default, every run
receives all instruction datasets, including the shared genesis examples, and
the puzzle-ID argument does not filter the files. With its second argument,
`only_puzzle_dataset`, set to `true`, as the
[`--only-puzzle-dataset`](#development) flag does, `document()` keeps the
genesis datasets and the running puzzle's own `dataset.<puzzle_id>.json`, and
drops every other puzzle's dataset. Puzzle datasets are recognized by their
eight-digit hexadecimal ARC IDs and matched case-insensitively; every other
file counts as a genesis dataset. The genesis puzzle has no dataset of its
own, so it then receives only the genesis datasets. Each file
is minified by removing whitespace between JSON tokens; string contents,
numbers, and key order stay exactly as written. The return value is one JSON
array, with a separate entry per file. New JSON files need no additional
imports or registration in the loader. An unreadable directory or file fails
explicitly. `document()` does not parse the files, so it passes malformed JSON
through. `llm.generate` parses that array for its `few_shot_examples`
message, so a malformed dataset fails the run at its first model call, before
any request, with a JSON syntax error that does not name the file.

Each model call can pass `llm.generate` a `few_shot` function that maps every
instruction dataset to only the properties of that call's own prompt; a call
without one receives every instruction dataset with all its properties. A call
that sends a `train_dataset` builds it and its few-shot examples with the same
projection, so both carry the same properties in the same order. Every solver
call passes its own:

| Call | Few-shot properties of each instruction dataset |
| --- | --- |
| `analyzer.declarations` | `declarations`, `output_canvas`, and each pair's `input_matrix` and `output_matrix` |
| `analyzer.instances`, `visualizer.instances` | `declarations`, and each pair's `input_matrix`, `output_matrix`, and `instances` with `input_instance_matrix` and `output_instance_matrix` |
| `analyzer.value` | `declarations`, and each pair's `instances` with `instance_name`, `input_instance_matrix`, and `implementation` |
| `visualizer.value` | `declarations`, and the instances of every pair in one list, with `implementation`, `input_value`, and `input_instance_matrix` |
| `visualizer.output_instance_matrix` | The instances of every pair in one list, with `input_value`, `input_instance_matrix`, `output_value`, and `output_instance_matrix` |

`analyzer.value` sends no `train_dataset`, so its examples carry the
properties of the `instances` it sends, the instances of the same pair
constructed before it. The calls that crop their own instance matrices,
`analyzer.value`, `visualizer.value`, and `visualizer.output_instance_matrix`,
also crop their examples' instance matrices to the nonzero bounding box with
`Matrix.crop`, or to an empty list for an all-zero mask. The orientation calls
of `analyzer.value` and `visualizer.value` then write each nonzero color of
those crops as a letter by its count, `A` for the most frequent, through
`Matrix.silhouette`, in their examples and their own prompts alike. None of
these calls receives `input_x`, `output_x`, `input_y`, `output_y`,
`input_orientation`, or `output_orientation`. The dataset files keep every
property and their full-canvas matrices.

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
output in the few-shot context. That is intended: the datasets are being
prepared as few-shot examples, and these runs are not yet held-out tests.

The genesis datasets are `dataset.core.json`, `dataset.extension.json`,
`dataset.curve.json`, and `dataset.line.json`. They are the shared examples of
the genesis puzzle `3aa6fb7a`, which has no `dataset.3aa6fb7a.json`, and every
run receives them, with or without `--only-puzzle-dataset`. Their pairs are
synthesized varieties of corners, curves, and lines rather than original
training or test pairs, so they contain no test pair. Each ends with two
synthesized pairs on
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
Each genesis class takes the standard entries `x`, `y`, `width`, `height`,
and `orientation` first, then its descriptive properties in their existing
order, and every implementation passes the input shape's bounding-box
`width` and `height`. A rule that grows a shape computes its new size from
its first values in the branch that sets its output tag: a core corner grows
from 2 by 2 to 3 by 3 with its outside marker, and its origin moves one
column left when it points west, at orientation 225 or 315, and one row up
when it points north, at orientation 45 or 315; a curve point grows from
1 by 1 to 3 by 2 with its rim; and a line of 3 or 4 cells grows by one cell,
one column wider when horizontal and one row taller when vertical. An
extension corner keeps its 3 by 3 size, because its inside marker fills a
cell of its own box. Every native output snapshot therefore carries its
shape's output bounding box, so `output_value.x` and `output_value.y` equal
`output_x` and `output_y`. A line points toward its color-8 endpoint: its
orientation is 270, 90, 0, or 180 when the endpoint is on the left, on the
right, at the top, or at the bottom, and it keeps that orientation as it grows
away from the endpoint. A curve point, a single cell, is at orientation 0, and
its rim makes a V that points toward the point, its vertex: the color-4 rule
sets `$Obj.orientation = 180`, because its color-8 arms go up and the V points
down, while a color-9 point keeps 0, because its color-2 arms go down and the
V points up. The genesis rules match categorical colors and orientations with
`is`, the spelling of `==` that [NUC 86](../docs/nuc-0086.md) adds, as in the
line header `if $Obj.orientation is 270:`, and the line rules assign quoted
strings, as in `$Obj.extension_direction = 'right'`.

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
its row 7 is 0, 2, 2 rather than 2, 2, 0. The class takes the standard entries
`x`, `y`, `width`, `height`, and `orientation` first, in that order, where
`width` and `height` are the column and row counts of the pattern's box, the
nonzero bounding box of its input mask, and then `color` and `period`, the
number of rows after which the pattern starts over, as scalars, with no parts
list, canvas size, or matrix of cells. Each implementation passes `width` and
`height` right after `y`, such as `width=3, height=6` for the first training
input.
`period` is the smallest number of rows such that every row of the box equals
the row that many rows below it, wherever that row is still inside the box;
like `height`, it is measured on the pattern's own box rather than the canvas.
Unconditional class-level rules set the output tag and introduce one
property, `extension_direction`, which is absent before the rules run and is
`down`, as in `dataset.line.json`.
The rules also rewrite `height` to `$Obj.height@first * 3 / 2`, 9 rows, the
pattern's one change in size, and `color` to 2. They leave `width` unchanged,
because the pattern grows down only and each output's box is as wide as its
input's, 3, 3, 2, and 3 columns, and they leave `x` and `y` unchanged because
the pattern keeps its top-left origin. The training pairs alone also fit a
fixed 9-row output; the rules scale the pattern's height instead. Native
output snapshots are captured with the pair's implementation followed by
`Obj[:]`. Tags identify the geometry: `l9100` is a single shape, as in
`dataset.28bf18c6.json` and `dataset.6150a2bd.json`, and `l8384` is a pattern
whose rows repeat every `period` rows down to its height. Input masks retain
each pair's full-size 6 by 3 input canvas, with zero cells padded `0`, and
output masks the complete 9 by 3 output; merging them reproduces each pair's
input and output. The `output_x`/`output_y` fields record the grown pattern's
nonzero bounding-box origin, equal to `output_value.x`/`output_value.y`, and
each output state's `width` and `height` are the grown pattern's bounding box.
`output_canvas` is `EXACT`: the composed canvas starts at the input's 6 by 3
dimensions and grows only to retain the pattern placed down to row 8, giving
the 9 by 3 outputs, while `FIT` would crop the empty right column of the third
training output. Orientation is 0 in both states because the pattern is not
turned. The test pair follows the training pairs at the same level and is
annotated by the same rules.

`dataset.0ca9ddb6.json` annotates the three training pairs from
`data\training\0ca9ddb6.json`, followed by its test pair, with fifteen point
instances in total (2, 5, 3, and 5 per pair).
Color-1 points gain color-7 orthogonal neighbors; color-2 points gain color-4
diagonal neighbors. Color-6 and color-8 points remain unchanged.
The class takes the standard entries `x`, `y`, `width`, `height`, and
`orientation`, then `vertex_color`; every point is 1 by 1, so each
implementation passes `width=1, height=1`.
The declarative rules introduce `marker_color`, which is absent before the
rules run, and, in the same branches that shift the origin, grow each cross
to 3 by 3 with `$Obj.width = $Obj.width@first + 2` and
`$Obj.height = $Obj.height@first + 2`; color-6 and color-8 points keep their
1 by 1 size. The two branches are guarded by `if $Obj.vertex_color is 1:`
and `if $Obj.vertex_color is 2:`, with `is`, the spelling of `==` that
[NUC 86](../docs/nuc-0086.md) adds. Tags identify the geometry: `l1707` is an
orthogonal cross, `l2404` is a diagonal cross, and `l7544` remains a single
point.
Every instance mask retains the complete 9 by 9 canvas. The x/y metadata
records each mask's nonzero bounding-box origin. Native output snapshots carry
the shifted pattern origin, derived from the original point coordinates with
`@first` anchors, and the pattern's size, equal to its output mask's
bounding box. Orientation is 0 in both states because
the patterns are symmetric. The test pair follows the training pairs at the
same level and is annotated by the same rules.
`document()` includes it in the shared few-shot context for every run by
default.

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
differences are 4, 4, 1, and 1, so the rule consults the pairs as data.
The class takes the standard entries `x`, `y`, `width`, `height`, and
`orientation` first, then `color` as its only descriptive scalar,
with no partner color, canvas size, or matrix of cells. `width` and
`height`, the column and row counts of the stripe's bounding box, are
1 and 3 for every stripe. One class-level rule rewrites `color` as
`[[1, 5], [2, 6], [3, 4], [8, 9]].find(pair => pair.includes($Obj.color@first)).find(partner => partner is not $Obj.color@first)`:
the list states each pair once, the first `find` selects the pair that
includes the stripe's input color, and the second `find` takes the other color
of that pair: the comparison `is not` of [NUC 86](../docs/nuc-0086.md), the
spelling of `!=`, holds only for the color that differs from the input color.
The `@first` anchors read the input color because the rule
rewrites `color` itself; a dictionary from every color to its partner would
state each pair twice. The two callbacks are chained rather than nested, so
each reads only its own parameter and the stripe. The rules create no
property and leave `x`, `y`, `width`, `height`, `orientation`, and the tag
unchanged, since a stripe keeps its size when it changes color, so input
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
read. The class takes the standard entries `x`, `y`, `width`, `height`,
and `orientation` first; `width` and `height`, the column and row
counts of the row's bounding box, are 3 and 1 for every row. It then
takes `parts`, the colors of the row's cells from left to right, as
a list, such as `[4, 4, 4]` for the top row of the first training
input, as its only descriptive property, with no color, color count, canvas
size, or matrix of cells, so no input property states whether the row is
uniform. One unconditional class-level rule introduces `fill_color`, which is
absent before the rules run, as
`5 if $Obj.parts.distinct().length is 1 else 0`, whose condition uses the
comparison `is` of [NUC 86](../docs/nuc-0086.md), the spelling of `==`: the
list query `distinct` of
[NUC 71](../docs/nuc-0071.md) keeps each of the row's colors once, so exactly
one kept color means the row is a single color, and the conditional
expression of [NUC 59](../docs/nuc-0059.md) gives such a row color 5 and
every other row the background color 0. The rules create no other property
and leave `x`, `y`, `width`, `height`, `orientation`, and the tag unchanged,
since a row keeps its size whether it becomes a bar or is cleared, so each
output state adds only `fill_color` to its input state's keys. Native
snapshots start with the pair's `canvas_width` and `canvas_height`, and
output snapshots are captured with all of the same pair's implementations
together, followed by `Obj[:]`. The tag `l3058` is a horizontal bar, as in
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
the direction each row points as a straight line of 3 cells. A row of exactly
two colors in which one end cell alone has its color has that cell as its head
and points toward it: 270 for the left end cells of `[2, 3, 3]`, `[7, 3, 3]`,
and `[3, 7, 7]`, and 90 for the right end cell of `[2, 2, 4]`. Every other
row, of one color or with its lone color in the middle cell, has no head and
takes the horizontal axis, 90. The rules leave `orientation` unchanged: each
bar keeps 90, and each cleared row, which has no output cell, keeps its input
orientation, 270 for three of the cleared rows and 90 for the other five. The
declarations hold executable code and one `#` comment, the orientation line
that every dataset places above its class. The test pair follows the training
pairs at the same level and is annotated by the same rules.

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
from the training pairs before the test pairs were read. The class takes
only the standard entries `x`, `y`, `width`, `height`, and `orientation`,
and no descriptive property, with no color, cell list, canvas size, or
matrix of cells, because the rule needs none: the figure's cells travel
unchanged in its masks. `width` and `height`, the column and row counts
of the figure's bounding box, are 3 and 1 for the bars and 2 and 2
for the trominoes. One unconditional class-level rule rewrites `y` as
`$Obj.y@first + 1`, the row below the figure's input origin, anchored with
`@first` because the rule rewrites `y` itself. Each training grid holds a
single figure and shows no relation between objects, so no population
operation appears; the class-level rule moves every figure on its own. The
rules create no property and leave `x`, `width`, `height`, `orientation`,
and the tag unchanged, since the figure moves without changing its size, so
input and output states share the same keys and differ only in `y`. Native
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
one of them. Orientation is the direction each figure points, the same in
both states because the figure moves without turning: 90 for the bars,
horizontal lines without a head; 135 and 45 for the trominoes, corners that
point toward their vertex, the bottom-right cell of the third figure's box and
the top-right cell of the fourth's; and 0 for the test pairs' vertical 2-cell
bar and single cell. The declarations hold executable code and one `#` comment,
the orientation line that every dataset places above its class. The test pairs
follow the training pairs at the same level and are annotated by the same
rules.

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
up by that tag. The class takes the standard entries `x`, `y`, `width`,
`height`, and `orientation`, where `width` and `height` are the column and row
counts of the block's bounding box, 3 and 3 in every pair, and `tag` as its
only descriptive property, such as `tag='l4262'` for the first training input,
with no color, cell count, parts list, canvas size, or matrix of cells. Its
constructor stores the tag it
is given instead of a fixed initial tag, so the input state names the shape
and no input property names the code. One class serves every shape because the
solver reads output states with `Obj[:]`, which lists no instance of a
subclass. Tags identify the geometry: `l4262` is the hollow diamond with a
filled corner and `l3743` the square with a diagonal tail, while `l2404` is a
diagonal cross and `l1707` an orthogonal cross, as in `dataset.0ca9ddb6.json`.
Unconditional class-level rules set the output tag `l7544`, a single point, as
in `dataset.44f52bb0.json`, shrink the block to that point's size, `width` 1
and `height` 1, and introduce one property, `fill_color`, which is absent
before the rules run, as
`{'l4262': 1, 'l2404': 2, 'l3743': 3, 'l1707': 6}[$Obj.tag@first]`: the object
literal states each shape's code once, and object bracket access reads the
code of the block's input tag, anchored with `@first` because the tag rule
turns the tag into `l7544`. The rules create no other properties, so each
output state adds only `fill_color` to its input state's keys, and they leave
`x` and `y` unchanged because the point sits at the block's origin, 0 and 0 in
every training pair. Native snapshots start with the pair's `canvas_width` and
`canvas_height`, and output snapshots are captured with the pair's
implementation followed by `Obj[:]`. Input masks retain each pair's full-size
input canvas, and output masks the complete output; merging them reproduces
each pair's input and output. The `output_x`/`output_y` fields record the
cell's nonzero bounding-box origin, equal to `output_value.x`/`output_value.y`,
and `output_value.width`/`output_value.height` equal its 1 by 1 size.
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
twice as wide as the crop: the original 3 by 6 output. The class takes the
standard entries `x`, `y`, `width`, `height`, and `orientation` first, in that
order, where `width` and `height` are the column and row counts of the shape's
box, the nonzero bounding box of its input mask, 3 and 3 in every pair, and
then `color`; each implementation passes `width=3, height=3` right after `y`.
Unconditional class-level rules hold for every instance, whatever its `color`:
they introduce `repeat_count` (2, the total number of copies) and
`repeat_direction` (`right`), which are absent before the rules run, rewrite
`width` as `$Obj.width@first * $Obj.repeat_count`, 6 columns, because the
copies side by side make the repeated shape that many times as wide as the
input shape while its `height` stays 3, and move `x` and `y` to the output
origin 0. Each output state's `width` and `height` are therefore the repeated
shape's bounding box, the whole 3 by 6 output. Tags identify the geometry:
`l9100` is a single shape and `l4305` is the shape repeated side by side.
Input masks retain each pair's input canvas and output masks its output
canvas. Orientation is 0 in both states because the shapes are not rotated.
The test pair follows the training pairs at the same level and is annotated
by the same rules.

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
or a flip across either diagonal. The class takes only the standard
entries `x`, `y`, `width`, `height`, and `orientation`, and no
descriptive property, with no color, parts list, canvas size, or
matrix of cells, because the rule needs none: the mosaic's cells
travel in its masks, and its turn is its orientation. `width` and
`height`, the column and row counts of the mosaic's bounding box, are
3 and 3 for every mosaic. One unconditional class-level rule rewrites
`orientation` as `($Obj.orientation@first + 180) % 360`, the half-turn rule
of `dataset.6150a2bd.json`, anchored with `@first` because the rule rewrites
`orientation` itself. The rules leave `x` and `y` unchanged: a mosaic that
fills its canvas keeps its box in place under a half turn, whether about its
own center or the canvas's. The two further rules of `dataset.6150a2bd.json`,
which move the box about the canvas center as
`canvas_width - $Obj.x@first - $Obj.width` and
`canvas_height - $Obj.y@first - $Obj.height`, would reproduce every training
output as well, but every mosaic's `width` and `height` equal the
canvas's, so those rules would only return the input origin; the
rules state the half turn alone, a choice fixed from the training
pairs before the test pair was read. Each training grid holds a
single mosaic and shows no relation between objects, so no population
operation appears. The rules create no property and leave `x`, `y`, `width`,
`height`, and the tag unchanged, since a turn changes the orientation alone,
not the mosaic's size, so input and output states share the same keys
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
180 in every output state. The declarations hold executable code and one `#`
comment, the orientation line that every dataset places above its class. The
test pair follows the training pairs at the same level and is annotated by the
same rules.

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
before the test pairs were read. The class takes the standard entries `x`,
`y`, `width`, `height`, and `orientation`, where `width` and `height` are the
column and row counts of the pattern's box, followed by `points`, the
`[column, row]` canvas position of each of the pattern's cells in reading
order, as a list, such as `[[0, 0], [2, 0], [1, 1], [0, 2], [2, 2]]` for the
first training input. It takes no color, cell count, canvas size, symmetry
flag, output color, or matrix of cells, so no input property states the
symmetry or the answer; the rules read the canvas's width from the solver's
`canvas_width`, not the pattern's own `width`. Unconditional class-level rules
set the output tag, shrink the pattern to the output point's size, `width` 1
and `height` 1, and introduce two properties, absent before the rules run.
`symmetric` is
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
bounding-box origin, equal to `output_value.x`/`output_value.y`, and
`output_value.width`/`output_value.height` equal its 1 by 1 size.
`output_canvas` is `FIT` because each output is cropped to its single cell
instead of keeping the 3 by 3 input canvas. Orientation follows each state's
own shape. In the inputs it is 45 for the sixth training pattern, an L-shaped
corner whose full top row and full right column meet at the top-right vertex
of its 2 by 2 box, toward which it points; 0 for the second test pattern, a
vertical line of one color with no head, by its axis; and 0 for the other six
patterns, which are neither lines, corners, nor chevrons. Every output is a
single point, whose orientation is 0, so one more unconditional class-level
rule sets `orientation` to 0, replacing the corner's 45 in the sixth training
pair. The test pairs follow the training pairs at the same level and are
annotated by the same rules.

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
rules maximize a count, not a color. The class takes the standard entries
`x`, `y`, `width`, `height`, and `orientation` first; `width` and `height`,
the column and row counts of the block's bounding box, are 3 and 3 for
every block, with or without a zero cell. It then takes `parts`, a
flat list of the block's nonzero cell colors in reading order, such as
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
because the fill starts at the block's origin, 0 and 0, and `width` and
`height` unchanged because the fill covers the block's 3 by 3 bounding box.
Native output snapshots are captured with the pair's implementation followed
by `Obj[:]`. Tags identify the geometry: `l9100` is a single shape, as in
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
that output. The class takes only the standard entries `x`, `y`, `width`,
`height`, and `orientation`, in that order, where `width` and `height` are the
column and row counts of the block's bounding box, the nonzero bounding box of
its input mask, with no color, parts list, canvas size, or matrix of cells;
each implementation passes them right after `y`, such as `width=3, height=2`
for the second training input, and the rules read the canvas's size from the
solver's `canvas_width` and `canvas_height`.
Unconditional class-level rules read the block's input state with `@first`
anchors: `($Obj.orientation@first + 180) % 360` gives `orientation`,
`canvas_width - $Obj.x@first - $Obj.width` gives `x`, and
`canvas_height - $Obj.y@first - $Obj.height` gives `y`. The rules leave
`width` and `height` unchanged, because a half turn keeps the box's size while
`orientation` carries the turn, and create no other properties, so input and
output states share the same keys. Native output snapshots are captured with
the pair's implementation followed by `Obj[:]`. The tag `l9100` is a single
shape, as in `dataset.28bf18c6.json` and `dataset.5582e5ca.json`, in both
states. Input masks retain each pair's full-size 3 by 3 input canvas, with its
zero cells padded `0`, and output masks the complete 3 by 3 output; merging
them reproduces each pair's input and output. The `output_x`/`output_y` fields
record the turned block's nonzero bounding-box origin, equal to
`output_value.x`/`output_value.y`. `output_canvas` is `EXACT` because each
output keeps its input's canvas, including the empty top row of the second
training output. Orientation is each block's direction, and the half turn
adds 180 to it. The second training block is a corner of three colors, whose
full top row and left column meet at the top-left vertex of its box, so it
points 315 in its input state and 135 in its output state, where its bottom
row and right column meet at the bottom-right vertex. The first training
block and the test block are neither lines, corners, nor chevrons, so they
are 0 in their input states and 180 in their output states. The test pair
follows the training
pairs at the same level and is annotated by the same rules.

`dataset.6e02f1e3.json` annotates the five original training pairs from
`data\training\6e02f1e3.json`, in order, followed by its test pair, six
mosaic instances in total, one per pair. Every input and output retains its
original 3 by 3 canvas, colors, positions, and zero cells; no input has a zero
cell. Each input is one mosaic of 9 cells that fills its canvas, in one, two,
or three colors: 2 and 3; 3, 4, and 2; 4; 3; and 4 and 3 in the training
pairs, and 4, 2, and 3 in the test pair. Each object is that whole mosaic, the
figure seen first, whose 3 by 3 box spans the canvas from its origin, 0 and 0.
Each output replaces the mosaic with one straight line of three color-5 cells
across the canvas, on a background of zero cells, and the number of colors in
the mosaic decides the line: one color gives the top row, two colors the main
diagonal, from the top-left corner to the bottom-right, and three colors the
other diagonal, from the top-right corner to the bottom-left. The third and
fourth training mosaics have one color, the first and fifth two, and the
second three, as does the test mosaic. Neither the colors nor the top row
decides the line: color 3 appears in the first, second, fourth, and fifth
training mosaics, which give all three lines, and every training mosaic's top
row is a single color. The whole mosaic changes as one: all its cells give way
to one line that follows none of its color regions, so each grid holds one
object, as in `dataset.3c9b0459.json` and `dataset.74dd1130.json`, rather
than the color regions of `dataset.9565186b.json`. The training pairs alone
also fit counting edge-connected pieces, because every training color forms
one piece; the rules count colors instead, a choice fixed from the training
pairs before the test pair was read, and in the test mosaic colors 2 and 3
each split into three single cells. Every canvas is 3 by 3, so the training
pairs alone also fit a line of three cells on a canvas of any size; the line
instead crosses the whole canvas, from edge to edge or corner to corner, and
its tag names that geometry, so the rules state no length: every line keeps
the mosaic's `width`, which spans the canvas. The class takes the standard
entries `x`, `y`, `width`, `height`, and `orientation` first, in that order,
where `width` and `height` are the column and row counts of the mosaic's box,
the nonzero bounding box of its input mask, 3 and 3 in every pair, and then
`parts`, a flat list of the mosaic's cell colors in reading order, as in
`dataset.5582e5ca.json`, such as `[2, 2, 2, 3, 2, 3, 3, 3, 3]` for the first
training input, as its only descriptive property, with no color list, color
count, line direction, canvas size, or matrix of cells, so no input property
names the count or the line. Each implementation passes `width=3, height=3`
right after `y`. Three unconditional class-level rules and one conditional
rule turn the mosaic into the line. `$Obj.tag = 'l6074'` names the line, and
`$Obj.fill_color = 5` introduces `fill_color`, which is absent before the
rules run, because the line is gray whatever the mosaic's colors.
`orientation` is `{1: 90, 2: 135, 3: 45}[$Obj.parts.distinct().length]`: the
list query `distinct` of [NUC 71](../docs/nuc-0071.md) keeps each of the
mosaic's colors once, so its length is the number of colors, and the object
literal, keyed by that number, states each count's axis once, by the shared
convention for straight lines without a head, where vertical is 0 and
horizontal is 90, so the main diagonal, which runs from northwest to southeast,
is 135 and the other diagonal 45, as `dataset.74dd1130.json` names its mirror
axis. No
arithmetic relates the counts to the axes, so the rule looks the axis up, as
`dataset.27a28665.json` looks up its code by tag, and it reads no
orientation, so it rewrites `orientation` without an `@first` anchor. The
conditional rule `if $Obj.orientation is 90:` uses `is`, the spelling of `==`
that [NUC 86](../docs/nuc-0086.md) adds, and sets `height` to 1, guarding on a
horizontal line: the line along the top row is
one row tall, the one change in the shape's size, while a diagonal crosses the
mosaic's whole 3 by 3 box and keeps its input `height`. The rule reads the
orientation that the lookup gives and writes `height` only for the horizontal
line, so it needs no `@first` anchor, and no rule writes `width`. Each
training grid holds a single mosaic and shows no relation between objects, so
no population operation appears; the count is a list query over the mosaic's
own cells. The rules leave `x` and `y` unchanged, because every line's box
starts at the canvas's origin, 0 and 0, where the mosaic's box starts, also
for the other diagonal, whose cells run from the top-right corner. They create
no other property, so each output state adds only `fill_color` to its input
state's keys, its tag turns `l6074`, its orientation turns 90, 135, or 45, and
its `height` turns 1 for the top-row line, so its `width` and `height` are the
line's bounding box: 3 and 1 for the top row and 3 and 3 for a diagonal.
Native snapshots start with the pair's `canvas_width` and `canvas_height`, and
output snapshots are captured with the pair's implementation followed by
`Obj[:]`. Tags identify the geometry: `l9100` is a single shape, as in
`dataset.28bf18c6.json` and `dataset.3c9b0459.json`, and `l6074` is a straight
line of cells across the whole canvas, along the axis its orientation gives.
Input masks retain each pair's full-size 3 by 3 input canvas and output masks
the complete 3 by 3 output, padded with `0` outside the line; with one object
per grid, each mask equals its pair's grid, and merging them reproduces each
pair's input and output. The `output_x`/`output_y` fields record the line's
nonzero bounding-box origin, 0 and 0, equal to
`output_value.x`/`output_value.y`.
`output_canvas` is `EXACT` because each output keeps its input's 3 by 3
canvas, including the two empty rows below the top-row line that `FIT` would
crop from the third and fourth training outputs. Orientation is 0 in every
input state, where the mosaic is not turned, and the line's axis, 90, 135, or
45, in every output state. The declarations hold executable code and one `#`
comment, the orientation line that every dataset places above its class. The
test pair follows the training pairs at the same level and is annotated by the
same rules.

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
which no turn does. The class takes only the standard entries `x`, `y`,
`width`, `height`, and `orientation`, in that order, where `width` and
`height` are the column and row counts of the mosaic's box, the nonzero
bounding box of its input mask, 3 and 3 in every pair, so each implementation
passes `width=3, height=3` right after `y`. It takes no descriptive property,
with no color, parts list, canvas size, or matrix of cells, because the rule
needs none: the mosaic's cells travel in its masks, and its flip is its
mirror axis. One unconditional class-level rule introduces `mirror_axis`,
which is absent before the rules run, as `135`: the axis the mosaic is
mirrored across, as an angle clockwise from north by the shared convention for
straight lines without a head, where vertical is 0 and horizontal is 90, so the
main diagonal, which runs from northwest to southeast, is 135 and the other
diagonal 45. The
rules leave `x` and `y` unchanged: a mosaic that fills its square canvas keeps
its box in place under a flip across its main diagonal, whether its own or the
canvas's. Two further rules, `$Obj.x = $Obj.y@first` and
`$Obj.y = $Obj.x@first`, which swap the box's origin as a flip across the
canvas's diagonal does, would reproduce every training output as well, but
every mosaic's origin is 0 and 0, so those rules would only return the input
origin; the rules state the flip alone, a choice fixed from the training pairs
before the test pair was read. Each training grid holds a single mosaic and
shows no relation between objects, so no population operation appears. The
rules create no other property and leave `x`, `y`, `width`, `height`,
`orientation`, and the tag unchanged, because the flip keeps the square box's
size and `mirror_axis` carries it, so each output state adds only
`mirror_axis` to its input state's keys. Native snapshots start with the
pair's `canvas_width` and `canvas_height`, and output snapshots are captured
with the pair's implementation followed by `Obj[:]`. The tag `l9100` is a
single shape, as in `dataset.28bf18c6.json` and `dataset.3c9b0459.json`, in
both states. Input masks retain each pair's full-size 3 by 3 input canvas and
output masks the complete 3 by 3 output; with no zero cell, each mask equals
its pair's grid, and merging them reproduces each pair's input and output. The
`output_x`/`output_y` fields record the mirrored mosaic's nonzero bounding-box
origin, 0 and 0, equal to `output_value.x`/`output_value.y`. `output_canvas`
is `EXACT` because each output keeps its input's 3 by 3 canvas; with no zero
cell to crop, `FIT` would frame the same grids. Orientation is 0 in both
states because the mosaic is mirrored, not turned, and `mirror_axis` states
the flip. The declarations hold executable code and one `#` comment, the
orientation line that every dataset places above its class. The test pair
follows the training pairs at the same level and is annotated by the same
rules.

`dataset.794b24be.json` annotates the ten original training pairs from
`data\training\794b24be.json`, in order, followed by its two test pairs,
twelve pattern instances in total, one per pair. Every input and output
retains its original 3 by 3 canvas, colors, positions, and zero cells. Each
input holds a few blue cells of color 1 on a background of zero cells: 1, 2,
2, 2, 1, 3, 3, 4, 4, and 4 cells in the training pairs and 2 and 4 in the
test pairs. Each object is that whole pattern, all the input's color-1 cells,
whether or not they touch, as in `dataset.d631b094.json`: the eighth training
input's four cells lie in three pieces, the ninth's form one zigzag, and the
tenth's are a corner cell and an L-shaped piece. Each output replaces the
pattern with a red tally of color 2, as many cells as the pattern has, in the
canvas's top-left corner: a bar along the top row, filled from the left, and,
once the bar spans the canvas, a stem down from the bar's middle cell. One
cell gives a point, two and three cells a bar of that length, and four cells
a T, the full top row with the middle cell of the second row. Only the number
of cells decides: the second, third, and fourth training patterns, two cells
that touch at a corner in the second and fourth and lie in opposite corners
of the canvas in the third, all give the same bar, and the three four-cell
training patterns all give the same T. The figure seen first is the group of
blue cells, whose count is all that the output keeps, so the whole pattern
changes as one into the whole tally. The training pairs alone also fit one
object per blue cell, each filling the tally's next cell in reading order;
the rules count the whole pattern instead, because no pair ties a blue cell
to a particular red cell. Because every canvas is 3 columns wide and no
pattern has more than four cells, the training pairs alone also fit a bar of
at most three cells on a canvas of any width, and any placement of a fifth
cell; the rules let the bar span the canvas's width, read from the solver's
`canvas_width` as in `dataset.a85d4709.json`, and give every cell beyond the
bar to the stem, choices fixed from the training pairs before the test pairs
were read. The class takes the standard entries `x`, `y`, `width`, `height`,
and `orientation` first, in that order, where `width` and `height` are the
column and row counts of the pattern's box, the nonzero bounding box of its
input mask, from 1 by 1 to 3 by 3, and then `cell_count`, the pattern's
number of cells, the scalar of `dataset.d631b094.json` and
`dataset.9565186b.json`, as its only descriptive property, with no color,
parts list, canvas size, or matrix of cells: every pattern is color 1, which
the rules never read. Each implementation passes
`width` and `height` right after `y`, such as
`x=0, y=1, width=1, height=1, orientation=0, cell_count=1` for the first
training input. Seven unconditional class-level rules turn the pattern into the
tally. `$Obj.tag = 'l5714'` names the tally, and `$Obj.fill_color = 2`
introduces `fill_color`, which is absent before the rules run, because the
tally is red whatever the pattern. `width` is
`min of [$Obj.cell_count, canvas_width]`, the noun form of the list query
`min` from [NUC 75](../docs/nuc-0075.md): the bar holds the first cells, up to
the canvas's width. `height` is `1 + $Obj.cell_count - $Obj.width`: the bar's
one row and one row for each cell beyond the bar, which the stem holds. That
rule reads the bar's `width` as the width rule sets it, not the pattern's
input width, so it takes no `@first` anchor, and no rule rewrites
`cell_count`. `orientation` is
`90 if $Obj.height is 1 and $Obj.width > 1 else 0`, the conditional
expression of [NUC 59](../docs/nuc-0059.md) with the comparison `is` of
[NUC 86](../docs/nuc-0086.md), the spelling of `==`: a tally one row high
and more than one cell wide is a bar, which lies along its row at 90, and
every other tally, a single cell or a T, is 0. It reads the tally's `width`
and `height` as their rules set them, never the pattern's input
orientation, which the tally does not keep. The rules give the same states
in reverse order and when they are declared after the instances.
`$Obj.x = 0` and `$Obj.y = 0` move the
tally to the canvas's top-left corner, wherever the pattern lies, which
changes `x` in the fourth and fifth training pairs and the first test pair,
and `y` in the first training pair. Each grid holds a single pattern and
shows no relation between objects, so no population operation appears; the
count is the pattern's own scalar. The rules create no other property, so
each output state adds only `fill_color` to its input state's keys, its tag
turns `l5714`, and its `width` and `height` are the tally's box: 1 and 1,
2 and 1, 3 and 1, or 3 and 2. Native snapshots start with the pair's
`canvas_width` and `canvas_height`, and output snapshots are captured with the
pair's implementation followed by `Obj[:]`. Tags identify the geometry:
`l9100` is a single shape, as in `dataset.28bf18c6.json` and
`dataset.d631b094.json`, and `l5714` is a tally, a bar along the top row
filled from the left up to the canvas's width, with the remaining cells as a
stem down from the bar's middle cell, a T once it has a stem. Input masks
retain each pair's full-size 3 by 3 input canvas and output masks the
complete 3 by 3 output, padded with `0` outside the pattern and the tally;
with one object per grid, each mask equals its pair's grid, and merging them
reproduces each pair's input and output. The `output_x`/`output_y` fields
record the tally's nonzero bounding-box origin, 0 and 0, equal to
`output_value.x`/`output_value.y`. `output_canvas` is `EXACT` because each
output keeps its input's 3 by 3 canvas, including the empty bottom row that
`FIT` would crop from every output. Each state's orientation follows its own
shape. In the input, a single cell is a point, 0, in the first and fifth
training pairs; two cells that touch at a corner form a diagonal line, which
rises to the right, at 45, in the second training pair and falls to the
right, at 135, in the fourth; the seventh training pattern is a corner of
three cells, which points toward its bottom-right vertex, at 135; and every
other pattern, whose cells lie apart or form no line or corner, is 0. In the
output, the tally is a point, 0, for one cell, a horizontal bar, 90, for two
or three cells, and 0 as a T, which is neither a line nor a
corner. The declarations hold executable code and one `#` comment, the
orientation line that every dataset places above its class. The test pairs
follow the training pairs at the same level and are annotated by the same
rules.

`dataset.8be77c9e.json` annotates the three original training pairs from
`data\training\8be77c9e.json`, in order, followed by its test pair, four
pattern instances in total, one per pair. Every input retains its original
3 by 3 canvas and every output its original 6 by 3 canvas, 6 rows of 3
columns, with their colors, positions, and zero cells. Each input holds one
pattern of color-1 cells on a background of zero cells: 5, 4, and 2 cells in
the training pairs and 2 in the test pair. Each object is that whole pattern,
the figure seen first, whether or not its cells touch: the second training
pattern's top-right cell touches the rest only at a corner, and the test
pattern's two cells do not touch at all; the same rules would mirror each
separate piece as well. Each output stacks the whole input grid over its
top-bottom mirror image on a canvas twice as tall: the input reappears
unchanged in the top three rows, and its row r reappears as row 5 - r, so the
output reads the same from the bottom up as from the top down. Each pattern is
therefore joined by its mirror image across the canvas's bottom edge, the line
between rows 2 and 3, and keeps its color, its columns, and its place. The
mirror line is the canvas's edge, not the pattern's own: the first training
pattern lies in the top two rows above an empty bottom row, so its mirror
image lies in the bottom two rows of the output, two empty rows below it,
while mirroring across the pattern's own bottom edge, which doubles its
height, reproduces only the second and third training pairs, whose patterns
reach the canvas's bottom edge. Repeating the grid below without mirroring it
reproduces no training output, and neither does a half turn of the grid
placed below, because no training grid is left-right symmetric. These choices
were fixed from the training pairs before the test pair was read. The class
takes only the standard entries `x`, `y`, `width`, `height`, and
`orientation`, in that order, where `width` and `height` are the column and
row counts of the pattern's box, the nonzero bounding box of its input mask:
3 and 2 for the first and second training patterns and the test pattern, and
1 and 2 for the third training pattern. It takes no descriptive property, with
no color, parts list, canvas size, or matrix of cells, because the rules need
none: the pattern's cells travel in its masks, and the rules read the canvas's
height from the solver's `canvas_height`. Each implementation passes `width`
and `height` right after `y`, such as
`x=0, y=0, width=3, height=2, orientation=0` for the first training input.
Two unconditional class-level rules turn every pattern into the figure it
forms with its mirror image. `$Obj.tag = 'l7511'` names that geometry, and
`$Obj.height = 2 * (canvas_height - $Obj.y@first)` grows the box down to the
mirror image of its top row: twice the distance from the pattern's top row to
the canvas's bottom edge, so the mirror line lies at the box's center. That
gives 6 rows for the first training pattern, which starts on the top row, and
4 for the others, which start one row down. The rule reads the canvas's height
as `dataset.d037b0a7.json` grows its bars with `canvas_height - $Obj.y@first`,
anchored with `@first` to read the pattern's input row, as there, although no
rule moves `y`. The rules leave `x`, `y`, `width`, and `orientation`
unchanged, because the mirror image keeps the pattern's columns and the figure
starts on the pattern's top row. They state no `mirror_axis`, the property of
`dataset.74dd1130.json`: the tag already names a mirror image below the shape,
and only `height` grows, so an axis would restate them. Each grid holds a
single pattern and shows no relation between objects, so no population
operation appears. The rules give the same states in reverse order and when
they are declared after the instances. They create no property, so input and
output states share the same keys and differ only in the tag and `height`.
Native snapshots start with the pair's `canvas_width` and `canvas_height`, and
output snapshots are captured with the pair's implementation followed by
`Obj[:]`. Tags identify the geometry: `l9100` is a single shape, as in
`dataset.28bf18c6.json` and `dataset.25ff71a9.json`, and `l7511` is a shape
over its top-bottom mirror image, symmetric about the horizontal center line
of its box. Input masks retain each pair's full-size 3 by 3 input canvas and
output masks the complete 6 by 3 output, with zero cells padded `0`; with one
pattern per grid, each mask equals its pair's grid, and merging them
reproduces each pair's input and output. The `output_x`/`output_y` fields
record the figure's nonzero bounding-box origin, 0 and 0 in the first
training pair, 2 and 1 in the third, and 0 and 1 in the second and the test
pair, equal to `output_value.x`/`output_value.y`, and
`output_value.width`/`output_value.height` equal its size. `output_canvas` is
`EXACT`, which keeps the input's 3 columns and grows the composed canvas down
to retain the figure, as in `dataset.017c7c7b.json`, while `FIT` would crop
the empty top row from the second and third training outputs and the two
empty left columns from the third. The composed canvas grows only to the
figure's lowest cell, though, so it reaches the outputs' 6 rows only when the
pattern starts on the top row, as in the first training pair. The second and
third training patterns, like the test pattern, start one row down, so their
outputs end in an empty row that neither mode frames: `EXACT` composes them
5 rows tall. Framing each output on the world model's canvas, `canvas_width`
columns by twice `canvas_height` rows, needs a solver change. Orientation is 0
in both states because the pattern is not turned: its mirror image is part of
the figure that `l7511` names, not a turn. The declarations hold executable
code and one `#` comment, the orientation line that every dataset places above
its class. The test pair follows the training pairs at the same level and is
annotated by the same rules.

`dataset.9565186b.json` annotates the four original training pairs from
`data\training\9565186b.json`, in order, followed by its test pair, sixteen
region instances in total (3, 4, 2, and 4 per training pair and 3 in the test
pair). Every input and output retains its original 3 by 3 canvas, colors, and
positions, and no grid has a zero cell. Each input is a mosaic of two to four
colors, and each object is one color region, all the cells of one color,
whether or not they touch, as in `dataset.d631b094.json`: 5, 1, and 3 cells of
colors 2, 1, and 8 in the first training pair; 4, 2, 1, and 2 cells of colors
1, 8, 3, and 2 in the second; 7 and 2 cells of colors 2 and 8 in the third;
and 2, 2, 3, and 2 cells of colors 3, 8, 4, and 1 in the fourth, listed in
reading order of each color's first cell. In the training pairs every color
forms one edge-connected piece except color 8 in the fourth, two single cells
in the top-right and bottom-left corners. Each output keeps the region with
the most cells in its color and turns every other region color 5, so 9 of the
13 training regions turn gray. The whole mosaic, the figure seen first in
`dataset.3c9b0459.json` and `dataset.74dd1130.json`, does not change as one:
its cells keep or lose their color by the region they belong to, which no
scalar property of a single mosaic object can state, so the objects are the
regions the mosaic is made of. The count decides, not the color or a share of
the canvas: the largest color, 8, never stays, the smallest stays only in the
second and third training pairs, and the kept region covers 5, 4, 7, and 3 of
the 9 cells, more than half only in the first and third. The training pairs
alone also fit reading each edge-connected piece as an object and keeping the
largest piece, because the only split color, the fourth pair's two single
color-8 cells, turns gray either way; the rules count each color as one region
instead, the unit in which every output changes, since all cells of a color
share one output color, a choice fixed from the training pairs before the
test pair was read. The class takes the standard entries `x`, `y`, `width`,
`height`, and `orientation` first, then `color` and `cell_count`, the region's
number of cells, as scalars, as in `dataset.9af7a82c.json`, with no parts
list, histogram, rank, output color, canvas size, or matrix of cells, so no
input property names the kept color. `width` and `height` are the column
and row counts of the region's bounding box, from a single cell to the
whole 3 by 3 canvas. They do not replace `cell_count`, because a region's
box does not count its cells: the fourth training pair's two color-8 corner
cells span a 3 by 3 box, yet that region turns gray, while the 3 cells of
color 4, a single row, keep their color. One conditional class-level rule,
`if any other in Obj: other.cell_count > $Obj.cell_count:` with the body
`$Obj.color = 5`, turns gray every region that another region of its pair
outnumbers: the quantifier `any` of [NUC 62](../docs/nuc-0062.md) states the
claim of `Obj.some(...)` directly, its binder ranges over the pair's whole
population, and `$Obj.cell_count` reads the region the rule is evaluated for.
The condition reads no color, so the rule rewrites `color` without an
`@first` anchor, and a tie for the most cells, which no pair has, would keep
every tied region's color. The rules create no property and leave `x`, `y`,
`width`, `height`, `orientation`, and the tag unchanged, since a region keeps
its cells when it turns gray, so input and output states share the same
keys, and a gray region's output state differs from its input state only
in `color`. Native snapshots start with the pair's `canvas_width` and
`canvas_height`, and output snapshots are captured with all of the same pair's
implementations together, followed by `Obj[:]`, so every region is compared
with its whole pair. The tag `l9100` is a single shape, as in
`dataset.9af7a82c.json` and `dataset.d631b094.json`, in both states. Input
masks retain each pair's full-size 3 by 3 input canvas and output masks the
complete 3 by 3 output, each padded with `0` outside its region; merging them
reproduces each pair's input and output. The `output_x`/`output_y` fields
record the region's nonzero bounding-box origin, unchanged from its input and
equal to `output_value.x`/`output_value.y`. `output_canvas` is `EXACT` because
each output keeps its input's 3 by 3 canvas; with no zero cell to crop, `FIT`
would frame the same grids. Orientation is the direction each region points,
the same in both states because a region keeps its cells when it turns gray:
315 and 135 for the first training pair's color-2 and color-8 regions,
corners that point toward the top-left and bottom-right vertices of their
boxes; 90 for the five horizontal rows of 2 or 3 cells, lines without a head;
and 0 for the vertical lines, the single cells, the T and C shapes, and the
regions split into separate cells. The declarations hold executable code and
one `#` comment, the orientation line that every dataset places above its
class. The test pair follows the training pairs at the same level and is
annotated by the same rules.

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
class takes the standard entries `x`, `y`, `width`, `height`, and
`orientation`, where `width` and `height` are the column and row counts of the
region's bounding box, 1 to 4 each, followed by `color` and `cell_count`, the
region's number of cells, as scalars rather than a matrix of cells.
Unconditional class-level rules resize every region into its bar, with `width`
1 and `height` the cell count, move `y` to the output origin 0, and derive `x`,
the bar's column, from the pair's whole population:
`Obj.filter(other => other.height > $Obj.height).length` counts the taller
bars, those of the larger regions, and re-evaluates whenever an instance is
added. The bar's size is the object's own `width` and `height`, so the rules
create no property, and input and output states share the same keys. Each pair
lists its instances largest first. Native output snapshots are captured with
all of the same pair's implementations together, followed by `Obj[:]`, so
every recorded `x` counts all larger regions of its pair. Tags
identify the geometry: `l9100` is a single shape, as in
`dataset.28bf18c6.json`, and `l6953` is a vertical bar. Input masks retain
each pair's full-size, zero-padded input canvas; output masks retain each
pair's full-size output canvas padded with `0` outside the bar. The
`output_x`/`output_y` fields record the bar's nonzero bounding-box world origin,
equal to `output_value.x`/`output_value.y`, not offsets to apply again, and
`output_value.width`/`output_value.height` equal that bounding box's size.
`output_canvas` is `FIT` because each histogram fills its nonzero bounding box,
and two outputs are wider than their inputs. Each state's orientation follows
its own shape. Seven input regions are corners, a full edge row and a full edge
column of their box, which point toward the box corner where the two meet: 45 at
the top right, 135 at the bottom right, 225 at the bottom left, and 315 at the
top left. Three regions are horizontal bars of two or three cells, at 90; the
two vertical bars and the four single cells are at 0, and so are the three
remaining regions, a notched block and two zigzags, which have no direction.
Every output is a vertical bar, at 0 by its vertical axis, or a single cell,
also 0, so one unconditional class-level rule, `$Obj.orientation = 0`, stands
every bar upright, whatever its region's orientation. The test pair
follows the training pairs at the same level and is annotated by the same
rules.

`dataset.a85d4709.json` annotates the four original training pairs from
`data\training\a85d4709.json`, in order, followed by its test pair, fifteen
marker instances in total, three per pair. Every input and output retains its
original 3 by 3 canvas, colors, positions, and zero cells. Each input holds
three gray cells of color 5, one in each row, on a background of zero cells.
In the training pairs they form one figure from the top row to the bottom row
whose cells touch through edges or corners: a diagonal line from the top-right
corner to the bottom-left corner, a vertical line down the right column, and,
in the third and fourth pairs, the same chevron pointing right, in the left
two columns and in the right two. Each output paints every row as a solid
horizontal bar, 1 cell tall and as wide as the canvas, whose color is decided
by the column of that row's gray cell: 2 for the left column, 4 for the middle
column, and 3 for the right column, so no output cell is zero. The whole gray
figure, the figure seen first, does not change as one: each of its rows takes
its own color, which no scalar property of a single figure object can state.
Its shape does not decide the colors either: the third and fourth training
figures are the same chevron one column apart, yet their rows turn 2, 4, and
2 and 4, 3, and 4. Each column instead gives the same color in every training
pair where it holds a gray cell, whichever figure that cell belongs to. Each
object is therefore one gray cell, the marker of its row, a single 1 by 1
point, listed from top to bottom, because the row is the unit in which every
output changes: each output row is a single color. The training pairs alone
also fit reading the vertical line of the second training pair as one
edge-connected object that widens into a solid 3 by 3 block, and reading the
colors by the canvas's edges, 2 at the left edge, 3 at the right edge, and 4
between them, because every canvas is 3 columns wide; the rules take one
marker per row in every pair and look the color up by column instead, choices
fixed from the training pairs before the test pair was read. The class takes
only the standard entries `x`, `y`, `width`, `height`, and `orientation`,
where `width` and `height` are the column and row counts of the marker's box,
1 and 1, and no descriptive property, with no color, column list, canvas size,
or matrix of cells: every marker is gray, which the rules never read, and the
marker's column is its `x`. Unconditional class-level rules set the output
tag and orientation, introduce one property, `fill_color`, absent before the
rules run, widen the marker into its bar, and move `x` to the left edge, 0,
while `y` keeps the marker's row. `fill_color` is
`[2, 4, 3][$Obj.x@first]`: the list holds one color per column of the
3-column canvases, from left to right, and indexing it with the marker's input
column picks its row's color, anchored with `@first` because another rule
moves `x`. `width` becomes `canvas_width`, reading the canvas's width from the
solver's `canvas_width`, as `dataset.d037b0a7.json` reads `canvas_height`,
while `height` stays the marker's 1 because the bar is one row tall. The
bar's ends come from the canvas, not the population: in the second training
pair every marker lies in the right column, yet every bar spans the whole row.
Each bar depends only on its own marker, and the training pairs show no
relation between markers, so no population operation appears. The rules create
no other property and leave `y` and `height` unchanged, so each output state
adds only `fill_color` to its input state's keys, its tag turns `l3058`, its
`width` turns 3, its `x` turns 0, and its `orientation` 90. Native snapshots
start with the pair's `canvas_width` and `canvas_height`, and output snapshots
are captured with all of the same pair's implementations together, followed by
`Obj[:]`. Tags
identify the geometry: `l7544` is a single point, as in
`dataset.0ca9ddb6.json` and `dataset.d037b0a7.json`, and `l3058` is a
horizontal bar, as in `dataset.d631b094.json` and `dataset.25d8a9c8.json`.
Input masks retain each pair's full-size 3 by 3 input canvas, padded with `0`
outside the marker, and output masks the complete 3 by 3 output, padded with
`0` outside the bar; merging them reproduces each pair's input and output. The
`output_x`/`output_y` fields record the bar's nonzero bounding-box origin, the
left end of its marker's row, equal to `output_value.x`/`output_value.y`, and
`output_value.width`/`output_value.height`, 3 and 1, equal the bar's size.
`output_canvas` is `EXACT` because each output keeps its input's 3 by 3
canvas; with no zero cell in any output, `FIT` would frame the same grids.
Orientation is 0 in the input state because the marker is a single point,
and 90 in the output state, where `$Obj.orientation = 90` sets it, because
the bar is a horizontal line without a head, which points along its
horizontal axis. The declarations hold executable code and one `#` comment, the
orientation line that every dataset places above its class. The test pair
follows the training pairs at the same level and is annotated by the same
rules.

`dataset.a9f96cdd.json` annotates the four original training pairs from
`data\training\a9f96cdd.json`, in order, followed by its test pair, five
point instances in total, one per pair. Every input and output retains its
original 3 by 5 canvas, 3 rows of 5 columns, with its colors, positions, and
zero cells. Each input holds a single color-2 point on a background of zero
cells: in the second column of the middle row, at the right end of the bottom
row, in the middle of the top row, and in the fourth column of the middle row
in the training pairs, and at the right end of the middle row in the test
pair. Each output replaces the point with its four diagonal neighbors, the
corners of the 3 by 3 box centered on it, each in a color of its own: 3 at the
top left, 6 at the top right, 8 at the bottom left, and 7 at the bottom right,
while the point's own cell turns 0. Corners that would fall outside the canvas
are cut off: all four remain in the first and fourth training pairs, only the
top-left corner in the second, whose point sits in the canvas's bottom-right
corner, the two bottom corners in the third, whose point sits on the top row,
and the two left corners in the test pair, whose point sits on the right edge.
Each object is therefore the point, the figure seen first, which becomes the
ring of its remaining corners: every corner's position and color follow from
the point and the canvas alone, and no corner exists in the input, so no
corner is an object of its own. The figure's box is the bounding box of the
corners that remain, not the 3 by 3 box clamped to the canvas: the third
training point lies on the top row, so the clamped box would start at that
row, which holds no corner, while its corners lie only in the row below. The
class takes only the standard entries `x`, `y`, `width`, `height`, and
`orientation`, in that order, where `width` and `height` are the column and
row counts of the point's box, 1 and 1, and no descriptive property, with no
color, canvas size, or matrix of cells: every point has color 2, which the
rules never read. Each implementation passes `width=1, height=1` right after
`y`, such as `x=1, y=1, width=1, height=1, orientation=0` for the first
training input. Six unconditional class-level rules turn every point into its
figure. `$Obj.tag = 'l2898'` names the figure, and
`$Obj.corner_colors = [[3, 6], [8, 7]]` introduces `corner_colors`, which is
absent before the rules run: the corners' colors as they lie around the
point, the top row's left and right corners first and then the bottom row's.
The other four rules move the box onto the remaining corners. `x` becomes
`min of [$Obj.x@first - 1, $Obj.x@first + 1].filter(column => column >= 0 and column < canvas_width)`,
the leftmost of the two corner columns, one left and one right of the point,
that lie on the canvas, reading the canvas's width from the solver's
`canvas_width`, as `dataset.a85d4709.json` does; `y` becomes the topmost of
the two corner rows, one above and one below the point, that lie on the
canvas, filtered the same way with `canvas_height`. `width` becomes the
`max of` the same filtered columns minus the new `x`, plus one, and `height`
the `max of` the same filtered rows minus the new `y`, plus one, so the box
ends at the rightmost corner column and the bottommost corner row on the
canvas. The `@first` anchors read the point's input column and row, because
the rules move `x` and `y`. The higher-order `filter` keeps the corners'
columns and rows that lie on the canvas, and `min of` and `max of` take the
ends of what it keeps. The filtered list is written out in each rule that
uses it rather than stored, because a property such as `$Obj.columns`, read
by `$Obj.x = min of $Obj.columns` while it reads `$Obj.x@first`, raises
`TypeError("Circular Dependency")` in the accepted runtime. Each figure
depends only on its own point and the canvas, and every pair holds one point,
so no population operation appears. The training pairs alone also fit
clipping at fixed rows and columns, because every canvas is 3 by 5; the rules
read `canvas_width` and `canvas_height` instead, a choice fixed from the
training pairs before the test pair was read. The rules create no other
property, so each output state adds only `corner_colors` to its input state's
keys, its tag turns `l2898`, and its `x`, `y`, `width`, and `height` become
the remaining corners' box: origin 0 and 0, 3 columns by 3 rows, in the first
training pair; 3 and 1, 1 column by 1 row, in the second; 1 and 1, 3 columns
by 1 row, in the third; 2 and 0, 3 columns by 3 rows, in the fourth; and 3
and 0, 1 column by 3 rows, in the test pair. The rules give the same states
in reverse order and when they are declared after the instances. Native
snapshots start with the pair's `canvas_width` and `canvas_height`, and
output snapshots are captured with all of the same pair's implementations
together, followed by `Obj[:]`. Tags identify the geometry: `l7544` is a
single point, as in `dataset.0ca9ddb6.json`, `dataset.d037b0a7.json`, and
`dataset.a85d4709.json`, and `l2898` is the four corners of a 3 by 3 box
around its empty center, in the colors of `corner_colors`, unlike `l2404`,
the diagonal cross of `dataset.0ca9ddb6.json` and `dataset.27a28665.json`,
which keeps its center. Input masks retain each pair's full-size 3 by 5 input
canvas, padded with `0` outside the point, and output masks the complete
3 by 5 output, padded with `0` outside the remaining corners; merging them
reproduces each pair's input and output. The `output_x`/`output_y` fields
record the figure's nonzero bounding-box origin, equal to
`output_value.x`/`output_value.y`, and
`output_value.width`/`output_value.height` equal its size, so the solver
places each crop without a negative offset or a grown canvas.
`output_canvas` is `EXACT` because each output keeps its input's 3 by 5
canvas, while `FIT` would crop every output to its corners, such as the
single cell of the second training output. Orientation is 0 in both states
because the point is symmetric and the figure is not turned: its corner
colors keep their places on the canvas in every pair, so `corner_colors`, not
`orientation`, records them. The declarations hold executable code and one `#`
comment, the orientation line that every dataset places above its class. The
test pair follows the training pairs at the same level and is annotated by the
same rules.

`dataset.c8f0f002.json` annotates the three original training pairs from
`data\training\c8f0f002.json`, in order, followed by its test pair, twelve
region instances in total, three per pair. Every input and output retains
its original canvas, colors, and positions: 3 by 6, 3 by 4, and 3 by 5 in
the training pairs and 3 by 5 in the test pair, rows by columns, and no grid
has a zero cell. Each input is a mosaic of the same three colors, 1, 7, and
8, that fills its canvas. Each output turns every color-7 cell gray, color
5, and keeps every color-1 and color-8 cell, so 3 of the 9 training regions
turn gray, one per pair. The whole mosaic, the figure seen first in
`dataset.3c9b0459.json` and `dataset.74dd1130.json`, does not change as one:
its color-7 cells turn gray while its other cells keep their colors, which a
single mosaic object could state only cell by cell, through a list of its
cells' colors. The objects are therefore the color regions the mosaic is made
of, as in `dataset.9565186b.json`: each object is all the cells of one color,
whether or not they touch, listed in reading order of each color's first
cell: 6, 5, and 7 cells of colors 1, 8, and 7 in the first training pair; 6,
5, and 1 cells of colors 7, 1, and 8 in the second; 6, 5, and 4 cells of
colors 1, 8, and 7 in the third; and 4, 8, and 3 cells of colors 1, 7, and 8
in the test pair. The color decides, not the count: color 7 has the most
cells in the first and second training pairs, 7 of 18 and 6 of 12, and the
fewest in the third, 4 of 15. Every training color-7 region lies in two or
three edge-connected pieces, among them a single cell in the bottom-left
corner of the first training input, and every piece turns gray. The training
pairs alone also fit reading each edge-connected piece as an object, 6, 7,
and 8 pieces in the training pairs, because the rule reads only a piece's
color, and, because every training grid holds the same three colors, turning
gray every region that is neither color 1 nor color 8; the rules take each
color as one region, the unit in which every output changes, since all cells
of a color share one output color, and name the one color that changes,
choices fixed from the training pairs before the test pair was read. The
class takes the standard entries `x`, `y`, `width`, `height`, and
`orientation` first, then `color` as its only descriptive scalar, as in
`dataset.0d3d703e.json`, with no cell count, parts list, output color, canvas
size, or matrix of cells, so no input property names the output color.
`width` and `height` are the column and row counts of the region's bounding
box, the nonzero bounding box of its input mask, from the single color-8 cell
of the second training pair, 1 by 1, to 5 columns by 3 rows, and each
implementation passes them right after `y`, such as
`x=1, y=0, width=5, height=3, orientation=0, color=8` for the color-8 region
of the first training input. One conditional class-level rule,
`if $Obj.color@first is 7:` with the body `$Obj.color = 5`, turns every
color-7 region gray. Its condition uses `is`, the spelling of `==` from
[NUC 86](../docs/nuc-0086.md), which an `if` header reads as an operator.
The condition reads the region's input color with an
`@first` anchor because the body rewrites `color` itself, as
`dataset.0d3d703e.json` reads `$Obj.color@first`, and a region of any other
color keeps its color because a false condition leaves its target untouched,
so the rule states only the change; the conditional expression
`5 if $Obj.color@first is 7 else $Obj.color@first` would give the same states
but restate every other region's color. Each region's change depends only on
its own color, and no relation between regions decides it, so no population
operation appears. The rule gives the same states when it is declared after
the instances. The rules create no property and leave `x`, `y`, `width`,
`height`, `orientation`, and the tag unchanged, since a region keeps its cells
when it turns gray, so input and output states share the same keys, and a gray
region's output state differs from its input state only in `color`. Native
snapshots start with the pair's `canvas_width` and `canvas_height`, and output
snapshots are captured with all of the same pair's implementations together,
followed by `Obj[:]`. The tag `l9100` is a single shape, as in
`dataset.28bf18c6.json`, `dataset.9565186b.json`, and `dataset.d631b094.json`,
in both states. Input masks retain each pair's full-size input canvas and
output masks the complete output, each padded with `0` outside its region;
merging them reproduces each pair's input and output. The
`output_x`/`output_y` fields record the region's nonzero bounding-box origin,
unchanged from its input and equal to `output_value.x`/`output_value.y`.
`output_canvas` is `EXACT` because each output keeps its input's canvas; with
no zero cell to crop, `FIT` would frame the same grids. Orientation is 0 in
both states because no region is turned. The declarations hold executable code
and one `#` comment, the orientation line that every dataset places above its
class. The test pair follows the training pairs at the same level and is
annotated by the same rules.

`dataset.c9e6f938.json` annotates the three original training pairs from
`data\training\c9e6f938.json`, in order, followed by its test pair, four
pattern instances in total, one per pair. Every input retains its original
3 by 3 canvas and every output its original 3 by 6 canvas, 3 rows of 6
columns, with their colors, positions, and zero cells. Each input holds one
pattern of color-7 cells on a background of zero cells: 4, 2, and 1 cells in
the training pairs and 4 in the test pair. Each object is that whole pattern,
the figure seen first, whether or not its cells touch: the first training
pattern's top cell touches the rest only at a corner, and so does the test
pattern's bottom-right cell; the same rules would mirror each separate piece
as well. Each output sets the whole input grid beside its left-right mirror
image on a canvas twice as wide: the input reappears unchanged in the left
three columns, and its column c reappears as column 5 - c, so every output
row reads the same from right to left as from left to right. Each pattern is
therefore joined by its mirror image across the canvas's right edge, the line
between columns 2 and 3, and keeps its color, its rows, and its place. The
mirror line is the canvas's edge, not the pattern's own: the third training
pattern is a single cell in the left column of the middle row, so its mirror
image lies in the right column of the output, four empty columns away, while
mirroring across the pattern's own right edge, which doubles its width,
reproduces only the first and second training pairs, whose patterns reach the
canvas's right edge. Repeating the grid to the right without mirroring it
reproduces no training output, and a half turn of the grid placed to the
right reproduces only the second and third, whose inputs are top-bottom
symmetric. These choices were fixed from the training pairs before the test
pair was read; the puzzle is the left-right counterpart of
`dataset.8be77c9e.json`, which stacks its grid over its top-bottom mirror
image. The class takes only the standard entries `x`, `y`, `width`,
`height`, and `orientation`, in that order, where `width` and `height` are
the column and row counts of the pattern's box, the nonzero bounding box of
its input mask: 2 and 3 for the first training pattern, 2 and 1 for the
second, 1 and 1 for the third, and 3 and 3 for the test pattern. It takes no
descriptive property, with no color, parts list, canvas size, or matrix of
cells, because the rules need none: the pattern's cells travel in its masks,
and the rules read the canvas's width from the solver's `canvas_width`. Each
implementation passes `width` and `height` right after `y`, such as
`x=1, y=0, width=2, height=3, orientation=0` for the first training input.
Two unconditional class-level rules turn every pattern into the figure it
forms with its mirror image. `$Obj.tag = 'l6919'` names that geometry, and
`$Obj.width = 2 * (canvas_width - $Obj.x@first)` grows the box right to the
mirror image of its left column: twice the distance from the pattern's left
column to the canvas's right edge, so the mirror line lies at the box's
center. That gives 4 columns for the first and second training patterns,
which start in the second column, and 6 for the third training pattern and
the test pattern, which start in the left column. The rule reads the canvas's
width as `dataset.8be77c9e.json` reads its height, anchored with `@first` to
read the pattern's input column, as there, although no rule moves `x`. The
rules leave `x`, `y`, `height`, and `orientation` unchanged, because the
mirror image keeps the pattern's rows and the figure starts at the pattern's
left column. They state no `mirror_axis`, the property of
`dataset.74dd1130.json`: the tag already names a mirror image to the right of
the shape, and only `width` grows, so an axis would restate them. Each grid
holds a single pattern and shows no relation between objects, so no
population operation appears. The rules give the same states in reverse
order and when they are declared after the instances. They create no
property, so input and output states share the same keys and differ only in
the tag and `width`. Native snapshots start with the pair's `canvas_width`
and `canvas_height`, and output snapshots are captured with the pair's
implementation followed by `Obj[:]`. Tags identify the geometry: `l9100` is a
single shape, as in `dataset.28bf18c6.json` and `dataset.8be77c9e.json`, and
`l6919` is a shape beside its left-right mirror image, symmetric about the
vertical center line of its box, the left-right counterpart of the `l7511` of
`dataset.8be77c9e.json`. Input masks retain each pair's full-size 3 by 3
input canvas and output masks the complete 3 by 6 output, with zero cells
padded `0`; with one pattern per grid, each mask equals its pair's grid, and
merging them reproduces each pair's input and output. The
`output_x`/`output_y` fields record the figure's nonzero bounding-box origin,
1 and 0 in the first training pair, 1 and 1 in the second, 0 and 1 in the
third, and 0 and 0 in the test pair, equal to
`output_value.x`/`output_value.y`, and
`output_value.width`/`output_value.height` equal its size, 4 and 3, 4 and 1,
6 and 1, and 6 and 3. `output_canvas` is `EXACT`, which keeps the input's
3 rows and grows the composed canvas right to retain the figure, as
`dataset.8be77c9e.json` grows its canvas down, while `FIT` would crop the
empty left and right columns from the first and second training outputs and
the empty top and bottom rows from the second and third. The composed canvas
grows only to the figure's rightmost cell, though, so it reaches the outputs'
6 columns only when the pattern starts in the left column, as in the third
training pair and the test pair. The first and second training patterns start
one column right, so their outputs end in an empty column that neither mode
frames: `EXACT` composes them 5 columns wide. Framing each output on the world
model's canvas, twice `canvas_width` columns by `canvas_height` rows, needs a
solver change. Orientation is the same in both states because the pattern is
not turned: its mirror image is part of the figure that `l6919` names, not a
turn. It is 90 in the second training pair, whose 2-cell row and 4-cell
figure are horizontal lines without a head, and 0 in the other pairs, where
the third training pattern is a single cell and the other patterns and
figures are neither lines, corners, nor chevrons.
The declarations hold executable code and one `#` comment, the orientation line
that every dataset places above its class. The test pair follows the training
pairs at the same level and is annotated by the same rules.

`dataset.d037b0a7.json` annotates the three original training pairs from
`data\training\d037b0a7.json`, in order, followed by its test pair, eleven
point instances in total (3, 3, 2, and 3 per pair). Every input and output
retains its original 3 by 3 canvas, colors, positions, and zero cells. Each
object is one nonzero input cell, a single 1 by 1 point of its own color,
listed in reading order; no two points of a pair share a column, so their bars
never overlap. Each output extends every point downward into a vertical bar of
its color, from the point's row to the bottom row of the canvas, so each bar is
1 to 3 cells tall and a point on the bottom row stays a single cell. The class
takes the standard entries `x`, `y`, `width`, `height`, and `orientation`,
where `width` and `height` are the column and row counts of the point's box,
1 and 1, and `color` as its only descriptive scalar, with no canvas size or
matrix of cells. Unconditional class-level rules set the output tag and grow
the point's `height` into the bar's, `canvas_height - $Obj.y@first`, reading
the canvas's height from the solver's `canvas_height`, and leave `x`, `y`, and
`width` unchanged because the point is the bar's top cell and the bar is one
column wide. The `@first` anchor reads the point's input row, so the bar grows
from the point's input state even though no rule moves `y`; the rules create
no property, so input and output states share the same keys. The bottom edge
comes from the canvas, not the
population: the second training pair's lowest point lies on the middle row,
yet its bars reach the bottom row. Native output snapshots are captured with
all of the same pair's implementations together, followed by `Obj[:]`. Tags
identify the geometry: `l7544` is a single point, as in
`dataset.0ca9ddb6.json`, and `l6953` is a vertical bar, as in
`dataset.9af7a82c.json`. Input masks retain each pair's full-size,
zero-padded input canvas; output masks retain each pair's full-size output
canvas padded with `0` outside the bar. The `output_x`/`output_y` fields
record the bar's nonzero bounding-box origin, equal to
`output_value.x`/`output_value.y`, and
`output_value.width`/`output_value.height` equal the bar's size.
`output_canvas` is `EXACT` because each output keeps its input's canvas,
including the empty right column of the third training output. Orientation is
0 in both states because the points are symmetric and the bars are vertical.
The test pair follows the training pairs at the same level and is annotated by
the same rules.

`dataset.d631b094.json` annotates the four original training pairs from
`data\training\d631b094.json`, in order, followed by its test pair, five
region instances in total, one per pair. Every input retains its original
3 by 3 canvas, colors, positions, and zero cells. Each input holds a single
nonzero color, and all cells of that color form one region, whether or not
they touch: 2, 3, 1, 4, and 5 cells per pair. Each output is a horizontal bar
of the region's color, one row high and as wide as the region's cell count,
giving the original 1 by 2, 1 by 3, 1 by 1, 1 by 4, and 1 by 5 outputs. The
class takes the standard entries `x`, `y`, `width`, `height`, and
`orientation`, where `width` and `height` are the column and row counts of the
region's bounding box, 1 to 3 each, followed by `color` and `cell_count`, the
region's number of cells, as scalars rather than a matrix of cells.
Unconditional class-level rules resize every region into its bar, with `width`
the cell count and `height` 1, and move `x` and `y` to the output origin 0.
The bar's size is the object's own `width` and `height`, so the rules create
no property, and input and output states share the same keys. Tags identify
the geometry:
`l9100` is a single shape, as in `dataset.28bf18c6.json`, and `l3058` is a
horizontal bar. Input masks retain each pair's full-size, zero-padded 3 by 3
input canvas; output masks retain each pair's full-size output canvas. The
`output_x`/`output_y` fields record the bar's nonzero bounding-box origin,
equal to `output_value.x`/`output_value.y`, and
`output_value.width`/`output_value.height` equal the bar's size.
`output_canvas` is `FIT` because each output is cropped to the bar.
Orientation follows each state's own shape, in degrees clockwise from north.
In the inputs, the first training region, two cells on a falling diagonal
from top left to bottom right, is a line without a head, 135; the second, a
three-cell chevron with its vertex on the left and both arms going right,
points left, 270; the single cell of the third is a point, 0; and the fourth
training region and the test region, neither a line, a corner, nor a chevron,
are 0. Another unconditional class-level rule,
`$Obj.orientation = 90 if $Obj.width > 1 else 0`, a conditional expression of
[NUC 59](../docs/nuc-0059.md), sets each bar's orientation from the bar's own
`width`, which the width rule makes the cell count: the bars of 2, 3, 4, and
5 cells are horizontal lines, 90, and the one-cell bar of the third training
pair is a point, 0. The rule reads no input orientation, so it rewrites
`orientation` without an `@first` anchor. The test pair follows the training
pairs at the same level and is annotated by the same rules.

`dataset.e9afcf9a.json` annotates the two original training pairs from
`data\training\e9afcf9a.json`, in order, followed by its test pair, six bar
instances in total, two per pair. Every input and output retains its original
2 by 6 canvas, 2 rows of 6 columns, with its colors and positions, and no grid
has a zero cell. Each input is two horizontal bars, one on top of the other,
each 6 cells of one color, one row tall and as wide as the canvas: color 3
over color 9 and 4 over 8 in the training pairs, and 6 over 2 in the test
pair. Each object is one bar, the figure seen first, listed from top to
bottom. Each output weaves the two bars into a checkerboard of their two
colors that fills the canvas: each bar keeps its cells in its own row in every
other column, starting with the left column, and moves its cells in the
columns between them into the other bar's row, so the two colors alternate
along every row and down every column, and the top-left cell keeps the top
bar's color. Each bar therefore becomes one color of the checkerboard, the
cells whose row plus column has the parity of the bar's own row, even for the
top bar and odd for the bottom bar, and keeps its color and its 6 cells. The
training pairs alone also fit reading the whole grid as one two-color mosaic
that becomes a checkerboard, and as six columns, every other one turned upside
down; the rules take the two bars instead, the figures seen first, each of
which keeps its color and cell count while its cells move, and the columns
would be finer objects than the bars, choices fixed from the training pairs
before the test pair was read. The class takes only the standard entries `x`,
`y`, `width`, `height`, and `orientation`, in that order, where `width` and
`height` are the column and row counts of the bar's box, the nonzero bounding
box of its input mask, 6 and 1 for every bar. It takes no descriptive
property, with no color, parts list, canvas size, or matrix of cells, because
the rules need none: the bar's color travels in its masks, and its share of
the checkerboard follows from its row. Each implementation passes
`width=6, height=1` right after `y`, such as
`x=0, y=1, width=6, height=1, orientation=90` for the bottom bar of the first
training input. Five unconditional class-level rules turn every bar into its
share of the checkerboard. `$Obj.tag = 'l5973'` names that geometry,
`$Obj.orientation = 0` marks it as a pattern that points no way, and
`$Obj.parity = $Obj.y@first % 2` introduces `parity`, which is absent before
the rules run: the parity of the bar's input row, 0 for the top bar and 1 for
the bottom bar, and so of row plus column on every cell the bar fills,
anchored with `@first` because another rule moves `y`.
`$Obj.height = canvas_height` grows the bar from one row to the canvas's two,
reading the canvas's height from the solver's `canvas_height`, as
`dataset.a85d4709.json` reads `canvas_width`, and `$Obj.y = 0` moves its box
to the top row, which changes `y` for every bottom bar. The rules leave `x`
and `width` unchanged, because the checkerboard spans the bars' six columns.
Each bar's share depends only on its own row and
the canvas, so no population operation appears: the two shares interlock
because the bars lie in rows of different parity, not because a rule compares
them. The two bars fill the canvas in every pair, so the training pairs alone
also fit a checkerboard over only the rows the bars cover; the rules cover the
canvas instead, a choice fixed from the training pairs before the test pair
was read. The rules give the same states in reverse order and when they are
declared after the instances. They create no other property, so each output
state adds only `parity` to its input state's keys, its tag turns `l5973`, its
`orientation` becomes 0, its `height` turns 2, and its `y` turns 0. Native
snapshots start with the pair's
`canvas_width` and `canvas_height`, and output snapshots are captured with all
of the same pair's implementations together, followed by `Obj[:]`. Tags
identify the geometry: `l3058` is a horizontal bar, as in
`dataset.d631b094.json`, `dataset.25d8a9c8.json`, and
`dataset.a85d4709.json`, and `l5973` is one color of a checkerboard, the cells
of its box whose row plus column, counted from the canvas's top-left corner,
has the parity `parity`, so they alternate along each row and down each
column. Input masks retain each pair's full-size 2 by 6 input canvas and
output masks the complete 2 by 6 output, each padded with `0` outside its bar
or its share of the checkerboard; merging them reproduces each pair's input
and output. The `output_x`/`output_y` fields record the share's nonzero
bounding-box origin, 0 and 0, equal to `output_value.x`/`output_value.y`, and
`output_value.width`/`output_value.height`, 6 and 2, equal its size, the whole
canvas. `output_canvas` is `EXACT` because each output keeps its input's
2 by 6 canvas; with no zero cell in any output, `FIT` would frame the same
grids. Orientation is 90 in every input state, because each bar is a straight
horizontal line without a head, and 0 in every output state, because each
share of the checkerboard is a two-row pattern that points no way. The two
shares of a pair are mirror images of each other, but no turn carries one onto
the other, since a half turn leaves each of them unchanged and a quarter turn
stands it upright, so `parity`, not `orientation`, tells them apart. The
declarations hold executable code and one `#` comment, the orientation line
that every dataset places above its class. The test pair follows the training
pairs at the same level and is annotated by the same rules.

`dataset.ed36ccf7.json` annotates the four original training pairs from
`data\training\ed36ccf7.json`, in order, followed by its test pair, five
pattern instances in total, one per pair. Every input and output retains its
original 3 by 3 canvas, colors, positions, and zero cells. Each input holds one
pattern of a single color on a background of zero cells: 7 cells of color 9,
5 of color 6, 5 of color 9, and 5 of color 2 in the training pairs, and 3 of
color 5 in the test pair. Each object is that whole pattern, the figure seen
first, whether or not its cells touch: the second training pattern is two
bars, the fourth a lone corner cell and a hook of four cells, and the test
pattern a single cell that touches a two-cell bar only at a corner. Each output
turns the whole grid a quarter turn counterclockwise, so the cell in row r and
column c moves to row 2 - c and column r: the right column becomes the top row,
the top row becomes the left column, read from the bottom up, and every color
keeps its cell count. The counterclockwise quarter turn is the only one of the
square's eight turns and mirrors, the identity included, that reproduces every
training pair: the third training pattern, an L symmetric across its main
diagonal, also fits a top-bottom mirror, but the other three pairs fit only
the quarter turn. The pattern turns as one, so each grid holds one object; the
same rules would turn each separate piece as well. The class takes only the
standard entries `x`, `y`, `width`, `height`, and `orientation`, in that
order, with no color, parts list, canvas size, or matrix of cells: the
pattern's cells travel in its masks, its turn is its orientation, and the
rules read the canvas's width from the solver's `canvas_width`. `width` and
`height` are the column and row counts of the pattern's box, the nonzero
bounding box of its input mask, 3 and 3 for every training pattern and 3 and
2 for the test pattern, and each implementation passes them right after `y`,
such as `width=3, height=3` for the first training input. Every training
pattern's box spans its canvas, 3 by 3 from the origin, 0 and 0, before and
after the turn, so the training pairs alone also fit turning the pattern in
place, with `orientation` alone, as `dataset.3c9b0459.json` states its half
turn. The rules turn it about the canvas center instead, as
`dataset.6150a2bd.json` turns its block a half turn, a choice fixed from the
training pairs before the test pair was read: unlike the mosaics of
`dataset.3c9b0459.json` and `dataset.74dd1130.json`, which have no zero cell
and always fill their canvases, these patterns lie on a background, so their
boxes need not span the canvas, and a turn of the whole grid carries a box
that does not span it to a new place. Three unconditional class-level rules
read the pattern's input state with `@first` anchors.
`($Obj.orientation@first + 270) % 360` gives `orientation`, because a
counterclockwise quarter turn is 270 degrees clockwise, as
`dataset.6150a2bd.json` adds 180 for its half turn. `$Obj.y@first` gives `x`,
because the box's top row becomes the turned box's left column.
`canvas_width - $Obj.x@first - $Obj.width@first` gives `y`, because the box's
right column becomes the turned box's top row, as far below the top edge as
the box was left of the right edge; the box's height is not needed. The rules
leave `width` and `height` unchanged, because a quarter turn changes the
pattern's orientation, not its size: they remain the column and row counts of
the pattern's own box, and the 270 degrees that the turn adds to `orientation`
say that the turned box is `height` columns wide and `width` rows tall. The
test pattern's box
starts one row down and is 3 columns wide and 2 rows tall; the turn carries it
to a box 2 columns wide and 3 rows tall that starts one column right, which
the turn in place, keeping the input origin, would not reproduce, while its
output state keeps `width` 3 and `height` 2. The `x` and `y` rules each read
the other coordinate's first value, a class cycle made only of fixed anchors.
[NUC 55](../docs/nuc-0055.md) exempts only a rule's own fixed anchor from the
circular-dependency check, so the runtime rejected such a cycle with
`TypeError("Circular Dependency")` until [NUC 85](../docs/nuc-0085.md), Fixed
Cross-Property Class Anchors, accepted every class cycle whose reads are all
fixed anchors of the same instance. Those anchors stay tracked, so the rules
read plain `@first` anchors and need no `.value` snapshot, and they give the
same states whether they are declared before or after the instances. Each
training grid holds a single pattern and shows no relation between objects,
so no population operation appears. The rules create no property, so input
and output states share the same keys; they differ only in `orientation` in
the training pairs, and also in `x` and `y` in the test pair. Native
snapshots start with the pair's `canvas_width` and `canvas_height`, and output
snapshots are captured with the pair's implementation followed by `Obj[:]`.
The tag `l9100` is a single shape, as in `dataset.28bf18c6.json`,
`dataset.3c9b0459.json`, and `dataset.6150a2bd.json`, in both states. Input
masks retain each pair's full-size 3 by 3 input canvas and output masks the
complete 3 by 3 output, with zero cells padded `0`; with one pattern per grid,
each mask equals its pair's grid, and merging them reproduces each pair's
input and output. The `output_x`/`output_y` fields record the turned
pattern's nonzero bounding-box origin, 0 and 0 in the training pairs and 1
and 0 in the test pair, equal to `output_value.x`/`output_value.y`.
`output_canvas` is `EXACT` because each output keeps its input's 3 by 3
canvas, as a turn of the whole square grid does; every training output's
pattern spans its canvas, so `FIT` would frame the same training grids, but
it would crop the empty left column of the test output. Orientation is each
pattern's direction, and the quarter turn adds 270 to it. The third training
pattern is a corner, whose full bottom row and right column meet at the
bottom-right vertex of its box, so it points 135 in its input state and 45 in
its output state, where its top row and right column meet at the top-right
vertex. Every other pattern is neither a line, a corner, nor a chevron, so it
is 0 in its input state and 270 in its output state. The declarations hold
executable code and one `#` comment, the orientation line that every dataset
places above its class. The test pair follows the training pairs at the same
level and is annotated by the same rules.

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
   [ARC-NUC-0ca9ddb6](docs/ARC-NUC-0ca9ddb6.md): a gestalt overview of the
   puzzle zoomed out to its figures, their relations, and its whole change,
   the dataset, any NUC and its outcome, every run's exit status and
   difference, and the open items. Every invocation brings the gestalt
   overview up to date, and the record keeps earlier runs.
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
    "class Obj(x: int, y: int, width: int, height: int, orientation: int, vertex_color: int, vertical_segment_color: int, horizontal_segment_color: int):\n    this.x = x\n    this.y = y\n    this.width = width\n    this.height = height\n    this.orientation = orientation\n    this.tag = 'l6747'\n    this.vertex_color = vertex_color\n    this.vertical_segment_color = vertical_segment_color\n    this.horizontal_segment_color = horizontal_segment_color",
    "if $Obj.vertex_color == 9 and $Obj.vertical_segment_color == 9 and $Obj.horizontal_segment_color == 9:\n    $Obj.tag = 'l6624'\n    $Obj.marker_color = 1\n    $Obj.x = $Obj.x@first - 1 if $Obj.orientation > 180 else $Obj.x@first\n    $Obj.y = $Obj.y@first - 1 if $Obj.orientation < 90 or $Obj.orientation > 270 else $Obj.y@first\n    $Obj.width = $Obj.width@first + 1\n    $Obj.height = $Obj.height@first + 1"
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
`$Obj.height = canvas_height - $Obj.y@first` in `dataset.d037b0a7.json` does, so
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
orientation fields give each state's numeric angle. Core, extension, and
line keep each object's orientation, while a curve point's rim turns it
into a V, as the convention below describes.
Native snapshots use `x`, `y`, `width`, `height`, and `orientation`; `tag`
replaces `shape` with shortened identifiers such as `l6747`.
Matrices already follow the canvas axes; composition does not rotate them.

Angles run clockwise from north: `N = 0`, `NE = 45`, `E = 90`, `SE = 135`,
`S = 180`, `SW = 225`, `W = 270`, and `NW = 315`. Every instruction dataset
gives each input and output state the direction its own shape points, by the
first of these rules that applies:

- A straight line of two or more cells points toward its head, an end cell
  with a color of its own, so a row of `x` cells that ends in a `y` cell on
  the right is 90. A line without a head takes its axis: vertical is 0,
  horizontal 90, the diagonal that rises to the right 45, and the one that
  falls to the right 135.
- A corner, a full edge row and a full edge column of its box, points toward
  the box corner where they meet: 45 at the top right, 135 at the bottom
  right, 225 at the bottom left, and 315 at the top left. A one-color corner
  keeps that direction with a marker of another color inside its box or just
  beyond its vertex.
- A chevron, a vertex cell with two equal diagonal arms on one side, points
  toward its vertex, so a V whose arms go up is 180.
- A point, a block, a pattern, or a group of scattered cells is 0.

An object that vanishes keeps its orientation, and a rule that turns an
object adds its clockwise turn, as in `($Obj.orientation@first + 180) % 360`.
Wherever an output's shape points another way, a class-level rule sets its
orientation, as `$Obj.orientation = 90` does for the rows that
`dataset.a85d4709.json` paints. A line in `dataset.line.json` points toward its
color-8 endpoint, so it needs no separate endpoint property. The first
declaration of every dataset states the convention for the model in one
shared comment line, directly above its class:

```nuc
# Orientation is the direction an object points, in degrees clockwise from north (0 up, 90 right, 180 down, 270 left)
```

| Dataset | Descriptive properties |
| --- | --- |
| Core and extension | `vertex_color`, `vertical_segment_color`, `horizontal_segment_color`, `marker_color` |
| Line | `extension_direction` |
| Curve | `vertex_color`, `rim_color` |

Color properties are categorical grid colors, not segment lengths or coordinates.

Every instance has `input_value` before the class rule and `output_value`
after it. Obtain the input snapshot using only the class declaration; the
complete declaration set immediately applies the rules to new instances.
Line `extension_direction` and curve `rim_color` are created
by those rules and are absent from their input snapshots rather than filled
with artificial defaults.

The input snapshot's `x` and `y` give the input bounding box's origin, and its
`width` and `height` give that box's column and row counts. The rules move the
output snapshot's `x` and `y` to the output bounding box's origin, recorded in
`output_x` and `output_y`, and set its `width` and `height` to that box's size.
In `dataset.core.json`, adding the exterior marker grows the corner from
2 by 2 to 3 by 3: the rule subtracts one from the x origin for orientation
225 or 315, and from the y origin for orientation 45 or 315.
The other origins are unchanged.
For lines, the rules subtract one from x when `extension_direction` is `left`,
or from y when it is `up`, and add one to `width` for a horizontal line or to
`height` for a vertical one; extensions to the right or down keep the input
origin. For curves, the rules subtract one from x for either vertex color, and
from y only for a color-4 point, whose rim grows above it, and grow the point
from 1 by 1 to 3 by 2.
These offsets are already applied in the output snapshot and recorded in
`output_x` and `output_y`; do not apply them a second time during placement.
The inner markers in `dataset.extension.json` change neither the
bounding-box origin nor the corner's 3 by 3 size.

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
      "implementation": "obj0 = Obj(x=5, y=1, width=2, height=2, orientation=135, vertex_color=9, vertical_segment_color=9, horizontal_segment_color=9)",
      "input_value": {
        "height": 2,
        "horizontal_segment_color": 9,
        "id": "obj0",
        "orientation": 135,
        "tag": "l6747",
        "vertex_color": 9,
        "vertical_segment_color": 9,
        "width": 2,
        "x": 5,
        "y": 1
      },
      "output_value": {
        "height": 3,
        "horizontal_segment_color": 9,
        "id": "obj0",
        "marker_color": 1,
        "orientation": 135,
        "tag": "l6624",
        "vertex_color": 9,
        "vertical_segment_color": 9,
        "width": 3,
        "x": 5,
        "y": 1
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

`Matrix.silhouette(matrix)` returns a new matrix of the same size in which each
nonzero color is a letter ranked by how often it occurs in that matrix: `A` for
the most frequent color, `B` for the next, and so on, with a tie going to the
color that comes first in reading order. Every `0` stays `0`. It rejects any
cell that is not an integer from 0 to 9, and it does not crop or modify the
input.

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

# nuc-arc

## Development

The application keeps `.ts` source files and runs directly on Node.js using
`tsx`, without generating JavaScript files. Sources use JavaScript syntax without
explicit TypeScript types; the TypeScript check uses non-strict inference.
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

Run the ARC example with `bun run start`. This makes live OpenRouter calls;
type checking, linting, and offline inspection do not require model calls.

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

## Instruction datasets

The synchronous `document()` function reads every JSON file in `src\dataset`,
alongside `instruct_dataset.ts`, on each call, independently of the working directory.
Files are sorted by name and serialized as one
`{"datasets": [...]}` document, with a separate array entry per file.
New files, including `dataset.core.2.json`, need no additional imports.
Non-JSON files and subdirectories are ignored. An empty dataset directory,
unreadable files, or malformed JSON fail explicitly; JSON errors name the file.

Instance `implementation` snippets use named constructor arguments whose names
match the parameters in their class declarations.

`dataset.cross.1.json` annotates only the three training pairs from
`src\data\training\0ca9ddb6.json`, with ten point instances in total.
Color-1 points gain color-7 orthogonal neighbors; color-2 points gain color-4
diagonal neighbors. Color-6 and color-8 points remain unchanged.
The declarative rules introduce `marker_color`, which is absent before the
rules run. Tags identify the geometry: `l1707` is an orthogonal cross,
`l2404` is a diagonal cross, and `l7544` remains a single point.
Every instance mask retains the complete 9 by 9 canvas. The x/y metadata
records each mask's nonzero bounding-box origin; native snapshot coordinates
remain anchored to the input point. Orientation is 0 in both states because
the patterns are symmetric. Held-out inputs and outputs are not included.
The dataset is discovered automatically by `document()`.

## ARC proposal agent

Use [arc-purposal](../.github/agents/arc-purposal.agent.md) and its
[skill](../.github/skills/arc-purposal/SKILL.md) to propose and prototype focused
solver improvements. The [ARC instructions](../.github/instructions/arc.instructions.md)
require an object-oriented Nucleoid world model, training-only rule inference,
and exact-grid evaluation without test-answer leakage.

The agent's contract is **JSON data in, JSON data out** for every ARC LLM
message, including injected examples and retry feedback. Natural-language
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

```json
{
  "declarations": [
    "class Obj(x: int, y: int, orientation: int, vertex_color: int, vertical_segment_color: int, horizontal_segment_color: int):\n    this.x = x\n    this.y = y\n    this.orientation = orientation\n    this.tag = 'l6747'\n    this.vertex_color = vertex_color\n    this.vertical_segment_color = vertical_segment_color\n    this.horizontal_segment_color = horizontal_segment_color\n    this.marker_color = 0",
    "if $Obj.vertex_color == 9 and $Obj.vertical_segment_color == 9 and $Obj.horizontal_segment_color == 9:\n    $Obj.tag = 'l6624'\n    $Obj.marker_color = 1"
  ]
}
```

##### 2. Initialize Nucleoid session

Nucleoid session is initialized with the declarations in order to use for identified instances later on.

##### 3. Extract instances

Based on declarations in `nuc` lang, the LLM extracts instances.
`input_matrix` and `output_matrix` remain the complete world canvases.
Each instance matrix is cropped to its own nonzero bounding box, retaining
internal zeros. The core, line, and curve instruction datasets expose
`input_x`, `output_x`, `input_y`, `output_y`, `input_orientation`, and
`output_orientation` on each instance. The x/y fields give the zero-based
column/row of the corresponding crop's origin; the orientation fields give
each state's numeric angle. The current transformations preserve orientation,
so their input and output angles match.
Native snapshots use `x`, `y`, and `orientation`; `tag` replaces `shape`
with shortened identifiers such as `l6747`.
Matrices already follow the canvas axes; composition does not rotate them.

Angles run clockwise from north: `N = 0`, `NE = 45`, `SE = 135`, `S = 180`,
`SW = 225`, and `NW = 315`. Line orientations describe axes:
horizontal is 90 and vertical is 0; endpoint and extension directions remain
separate properties.

| Dataset | Descriptive properties |
| --- | --- |
| Core | `vertex_color`, `vertical_segment_color`, `horizontal_segment_color`, `marker_color` |
| Line | `endpoint_position` (side of the color-8 endpoint), `extension_direction` |
| Curve | `vertex_color`, `rim_color`, `rim_direction` |

Color properties are categorical grid colors, not segment lengths or coordinates.

Every instance has `input_value` before the class rule and `output_value`
after it. Obtain the input snapshot using only the class declaration; the
complete declaration set immediately applies the rules to new instances.
Line `extension_direction` and curve `rim_color`/`rim_direction` are created
by those rules and are absent from their input snapshots rather than filled
with artificial defaults.

Both native snapshots keep `x` and `y` anchored to the input bounding box;
`output_x` and `output_y` record the output crop's origin instead.
In `dataset.core.1.json`, adding the exterior marker grows the output to
3 by 3: subtract one from the input crop's x origin for `input_orientation`
225 or 315, and from its y origin for `input_orientation` 45 or 315.
The other origins are unchanged.
For lines, subtract one from x when `extension_direction` is `left`, or from
y when it is `up`; extensions to the right or down keep the input origin.
For curves, subtract one from x for either direction, and from y only when
`rim_direction` is `up`. These offsets are already recorded in `output_x`
and `output_y`; do not apply them a second time during placement.
The inner markers in
`dataset.core.2.json`, `dataset.core.3.json`, and `dataset.core.4.json` do not
change the crop origin.
The third core dataset uses 4 by 4 color-2 corners with color-6 markers one
cell diagonally inward from each vertex, across all four orientations.
The fourth uses 3 by 3 color-2 corners with a color-7 marker one cell
diagonally inward from each vertex. Input and output masks retain the same
origin and dimensions; the layouts keep the resized objects separate.

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
        [0, 9],
        [9, 9]
      ],
      "output_instance_matrix": [
        [0, 9, 0],
        [9, 9, 0],
        [0, 0, 1]
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
        [0, 9],
        [9, 9]
      ],
      "output_instance_matrix": [
        [0, 9, 0],
        [9, 9, 0],
        [0, 0, 1]
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

Finally, `Matrix.place` expands each cropped output at its output crop origin
onto the inferred canvas before `Matrix.merge` combines the masks. Zero is
transparent; later nonzero cells win on overlap, as before. Invalid colors,
ragged or padded object matrices, missing coordinates, incomplete segmentation,
and out-of-bounds placements are rejected explicitly. An erased output object
uses `[[0]]` with an explicit output position.

`Matrix.crop(matrix)` returns a new matrix containing the smallest rectangle
around all nonzero cells. All-zero borders are removed from all four sides:
top, bottom, left, and right. Internal zeros and empty rows or columns between
occupied cells are preserved. Empty and all-zero matrices return `[]`. The input
is not modified. Ragged matrices and non-finite or nonnumeric cells are rejected
explicitly.
This standalone utility does not change the full-size instance masks or return
position metadata. Use `Matrix.locate` to retain the original position with
the crop.

`Matrix.locate(matrix)` returns `{ matrix, x, y }`, pairing the same tight crop
with its zero-based top-left position in the original matrix: `x` is the column
and `y` is the row. Empty and all-zero matrices return
`{ matrix: [], x: 0, y: 0 }`. It uses the same validation as `Matrix.crop` and
does not modify the input. For a located result,
`Matrix.pad(result.matrix, rows, cols, result.x, result.y)` restores the crop
to its original position on a canvas with the supplied dimensions.

`Matrix.pad(matrix, rows, cols, left, top)` returns a new matrix of the requested
dimensions, placing the input at the required zero-based column and row offsets
and filling the remaining cells with zeros. It does not center or modify the
input. Empty matrices can be padded to an all-zero canvas. Invalid matrices,
dimensions or offsets that are not non-negative safe integers, and placements
that do not fit inside the requested dimensions are rejected.

The same prediction path replays every training pair before predicting the
held-out input. A training mismatch stops the run. Held-out predictions are
frozen before expected outputs are accessed for exact-grid scoring, including
shape. Each object executes with the learned declarations in a fresh Nucleoid
process, so no training instance state carries into prediction.

The run prints `Difference: N` after the result and expected matrices. Each
different, missing, or extra cell adds one to the score: `0` means an exact
match, and higher scores are worse. An extra empty row counts as one shape
mismatch.

## Benchmark

![Benchmark](https://github.com/user-attachments/assets/e9d30330-6d4f-461b-9b9e-c8d1b9eccaf5)

This benchmark demonstrates how LLMs respond to different prompting methods for ARC subtasks. Our observations indicate that using natural language prompts often yields inconsistent and unpredictable results, particularly in chain-of-thought (CoT) reasoning. However, switching to 5GL such as `nuc` lang, significantly increases accuracy, with LLM responses approaching deterministic behavior. Notably, `nuc` lang achieves this performance without extensive training requirements. This suggests that structured, high-level programming languages may be more effective for certain types of task prompting in LLMs compared to conventional natural language approaches.

---

**Nucleoid**

![3aa6fb7a_nuc](https://github.com/user-attachments/assets/98c5147e-d1b9-4a09-8c38-0c803d99ea55)


**ChatGPT o-1**

![3aa6fb7a_chatgpt-o1](https://github.com/user-attachments/assets/84febfe8-4c6d-4390-91f0-0bdca35edab0)

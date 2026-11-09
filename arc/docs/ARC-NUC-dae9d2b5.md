# ARC-NUC-dae9d2b5 -- Panel Union

Progress record for ARC training puzzle `dae9d2b5`: its instruction dataset,
the solver and Nucleoid changes it needed, its runs, and what is still open.

| Item | Status |
| --- | --- |
| Puzzle | [`data/training/dae9d2b5.json`](../data/training/dae9d2b5.json) |
| Instruction dataset | [`dataset.dae9d2b5.json`](../src/dataset/dataset.dae9d2b5.json): 5 original training pairs followed by 2 test pairs, 14 whole panel-figure instances, written on 8-Nov-2026; colors written as numbers instead of letters on 8-Nov-2026 |
| Output canvas | `FIT` |
| Nucleoid enhancements | None; accepted Nucleoid expresses the model |
| Solver changes | JSON-only message construction suggested in `src/lib/llm.ts`, `src/analyzer.ts`, and `src/visualizer.ts`; not applied |
| Solver runs | None; message-format and process-environment prerequisites blocked the run; difference unknown |
| Updated | 8-Nov-2026 |

## Puzzle

Every training input is a 3 by 6 canvas containing two equal-width panels:
a color-4 figure on the left and a color-3 figure on the right. Each output
is a 3 by 3 overlay of the panels at their matching local coordinates.
A position is occupied when either or both figures occupy it, and every
occupied output position has color 6. Empty positions remain background.

This puzzle was selected without an explicit ID. Among non-genesis training
puzzles without a canonical dataset, its largest training input or output
has 18 cells and its training matrices total 135 cells. It wins the remaining
tie by puzzle ID. Selection used the training pairs alone.

## Gestalt overview

- **Whole grid.** Two side-by-side silhouette panels become one registered
  overlay, rather than a resized version of either individual figure.
- **Figures.** The whole occupied figure of each panel is an object,
  including any disconnected pieces. Its empty space is background.
- **Relations.** The panels share a coordinate frame after the right panel
  moves left by half the input canvas width. Registration follows the
  panels, not their figures' tight bounding boxes.
- **Change.** Preserve both complete silhouettes, align them, and recolor
  their occupied positions uniformly. Identically colored overlaps form
  their union without order-dependent conflict.
- **World model.** Two whole panel figures per pair, each retaining its
  shape mask. Translation and recoloring express the transformation at the
  figure level; connected components, individual-cell objects, cell lists,
  and population bookkeeping are unnecessary.

## Dataset

The canonical dataset contains the five original training pairs in source
order followed by the two original test pairs at the same level. Every pair
retains its original patterns, positions, dimensions, and zero padding.
Both instance masks keep their respective complete canvases. The dataset's
global color ranking writes color 6 as `A` (40 occurrences), color 3 as `B`
(26), and color 4 as `C` (24); zero remains `0`. These counts come from the
dataset worker's annotation handoff, not from solver predictions.

Each `Obj` takes only the standard spatial entries. Its tag `l9100`, reused
from existing single-shape datasets, describes the preserved silhouette.
The class-level rules register the right figure and introduce only
`fill_color`, which is absent from input states:

```nuc
class Obj(x: int, y: int, width: int, height: int, orientation: int):
    this.x = x
    this.y = y
    this.width = width
    this.height = height
    this.orientation = orientation
    this.tag = 'l9100'

if $Obj.x@first >= canvas_width / 2:
    $Obj.x = $Obj.x@first - canvas_width / 2

$Obj.fill_color = 'A'
```

The input origin anchored by `@first` prevents cumulative translation.
The figures keep `y`, their own width and height, orientation, and tag.
The two training corners point northwest at 315 degrees; the other
training figures take 0, and no figure turns. No property of another object
determines registration or color, so a population query would restate work
already supplied by the canvas and same-color mask composition. `FIT`
frames the union rather than retaining the larger two-panel input canvas.
The [README paragraph](../README.md#instruction-datasets) describes the
annotations and native snapshots in full.

The worker froze the model from training before extracting the test pairs
and applied it without revision. Its manual/offline checks passed for the
original pair order and patterns, global letter ranking, full-canvas masks,
recomposition and overlap agreement, native input and whole-world output
snapshots, and rendering with the existing `Matrix` operations. The caller
confirmed the frozen dataset blob and declarations and independently
compared the training patterns with the original training-only extraction.
These are offline annotation checks, not a live-model result.

The unchanged loader includes the file among all 32 JSON instruction
datasets. Its puzzle ID was added once to the existing reporting list in
[`arc.yml`](../../.github/workflows/arc.yml), preserving the previous IDs,
their order, settings, and timeouts. No workflow was dispatched.
No hint or NUC was supplied or needed, so strategy learning is
`not_applicable`; no strategy trial or checkpoint was created.

### Number colors

On 8-Nov-2026, at the user's request, the letter colors were reverted, and the
dataset writes every color as its number, as the puzzle's pairs hold it,
turning `A` into 6, `B` into 3, and `C` into 4. The pair grids, both instance
masks, the implementations' `#` comments, and the snapshots hold numbers. The
changed declarations now read:

```nuc
# Recolor every occupied figure cell; identically colored overlaps compose as a union.
$Obj.fill_color = 6
```

Every other spelling is unchanged. Offline, with the accepted runtime at
`1a41ca7` (last runtime change `f463d94`), all 14 input and 14 output
snapshots of the 7 pairs replay, each equal to its letter snapshot with every
letter read as its number. The pair grids equal the puzzle's pairs cell for
cell. `Matrix.silhouette` again takes only the colors 0 to 9, and the
orientation calls' few-shot crops read exactly as they did with letters,
because the counts that rank them are unchanged. No NUC was requested, so no
proposal strategy was selected and strategy learning did not apply. No solver
run has measured this version, so its difference is unknown, not zero.

## Runs

### 8-Nov-2026 -- Run not started

The planned command was `bun start dae9d2b5`, using accepted Nucleoid at
`84fa81e94f64ba6aac9069e72b9126d3655bb734`. No live solver process or paid
model call was started. Exit status, elapsed time, and difference are
unknown, not zero; there is also no measured difference before this dataset.

Offline source inspection found that the shared LLM wrapper sends
`few_shot:` and `control_id:` labels outside JSON documents
(`src/lib/llm.ts`, lines 17-38), and its grammar branch sends raw reference
text. Analyzer and visualizer requests likewise concatenate unquoted labels
and pseudo-JSON schema placeholders (`src/analyzer.ts`, lines 20-30, and
`src/visualizer.ts`, lines 20-38). Such calls do not meet the JSON-only input
requirement. These findings concern message construction, not an expected
test grid or prediction difference.

Separately, `OPENROUTER_API_KEY` was absent from the permitted process
environment. The server imports `dotenv/config`; a future invocation must
avoid environment-file loading rather than inspect `.env` files for a key.
The solver implementation and its previous default puzzle, `74dd1130`,
remain unchanged. No NUC outcome exists.

## Open items

- Before a separately authorized run, construct every analyzer, visualizer,
  and shared LLM message as a data object serialized with `JSON.stringify`.
  Encode operations, response schemas, grammar context, control IDs,
  few-shot examples, and diagnostics as structured JSON; validate each
  message independently. Preserve the intentionally shared instruction
  datasets and existing solver behavior. This change is suggested only.
- Supply the API key independently through the process environment, and
  prevent both Bun and `dotenv/config` from loading `.env` files. No
  credential value was inspected.
- The dataset has no live-model score. Neither an exact match nor a
  regression has been measured, and this invocation will not rerun it.

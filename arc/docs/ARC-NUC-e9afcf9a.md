# ARC-NUC-e9afcf9a -- Bars Woven Into a Checkerboard

Progress record for ARC training puzzle `e9afcf9a`: its instruction dataset,
the solver and Nucleoid changes it needed, its runs, and what is still open.

| Item | Status |
| --- | --- |
| Puzzle | [`data/training/e9afcf9a.json`](../data/training/e9afcf9a.json) |
| Instruction dataset | [`dataset.e9afcf9a.json`](../src/dataset/dataset.e9afcf9a.json): the puzzle's 2 original training pairs and its test pair, 6 bar instances, 2 in each pair; committed with this record |
| Output canvas | `EXACT` |
| Nucleoid enhancements | None requested, so no proposal strategy was selected and strategy learning did not apply; the rules use the class-level `@first` anchor of [NUC 55](../../docs/nuc-0055.md), built on the state versions of [NUC 53](../../docs/nuc-0053.md) |
| Solver changes | None; one earlier suggested change still awaits review, not applied |
| Solver runs | 1 local run on 6-Nov-2026: exit 0, difference 0 |
| Updated | 6-Nov-2026 |

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

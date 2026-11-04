# ARC-NUC-44f52bb0 -- Mirror Symmetry Check

Progress record for ARC training puzzle `44f52bb0`: its instruction dataset,
the solver and Nucleoid changes it needed, its runs, and what is still open.

| Item | Status |
| --- | --- |
| Puzzle | [`data/training/44f52bb0.json`](../data/training/44f52bb0.json) |
| Instruction dataset | [`dataset.44f52bb0.json`](../src/dataset/dataset.44f52bb0.json): the puzzle's 6 original training pairs and its 2 test pairs, 32 point instances; uncommitted, awaiting review |
| Output canvas | `FIT` |
| Nucleoid enhancements | None requested; the rules use the `@first` reads of [NUC 53](../../docs/nuc-0053.md) and the accepted population queries `some` and `every` |
| Solver changes | None; one suggested change awaits review, not applied |
| Solver runs | 1 local run on 4-Nov-2026, exit 1 before any model call: `OPENROUTER_API_KEY is required`; no difference score |
| Updated | 4-Nov-2026 |

## Puzzle

Each training input is a 3 by 3 canvas of color-2 points, and each output is
a single 1 by 1 cell. The cell is color 1 when the input is left-right mirror
symmetric across the canvas, every point having a point of its color in the
same row at the mirrored column, and color 7 otherwise. Three of the six
training inputs are symmetric. The number of points does not decide the
output, and two inputs that are symmetric only about their own shape, a Z
that is a half turn of itself and an L that mirrors itself across its own
diagonal, give 7, so symmetry is judged in the canvas frame. The output
canvas shrinks from 3 by 3 to 1 by 1, so the dataset is `FIT`.

## Dataset

The single dataset preserves the six original training pairs in source
order, followed at the same level by the puzzle's two test pairs, which the
same rules annotate. Every input mask keeps the full 3 by 3 canvas with zero
padding, and every output mask is the complete 1 by 1 output. Each point is
one instance of `Obj(x, y, orientation, color, canvas_width)`, tagged `l7544`,
the single point of `0ca9ddb6` and `d037b0a7`. Unconditional class-level
rules decide one verdict for the whole population of a pair and move each
point to the output origin:

```nuc
$Obj.has_mirror = Obj.some(other => other.color == $Obj.color and other.y@first == $Obj.y@first and other.x@first == $Obj.canvas_width - 1 - $Obj.x@first)
$Obj.symmetric = Obj.every(point => point.has_mirror)
$Obj.fill_color = 1 if $Obj.symmetric else 7
$Obj.x = 0
$Obj.y = 0
```

The `@first` reads keep the mirror test on the input coordinates after the
last two rules move the point. The training pairs cannot separate the
left-right mirror from a top-bottom mirror or a half turn about the canvas
center; the rules test the left-right axis, by the convention that mirror
symmetry is bilateral symmetry across the vertical axis. `/arc-dataset` chose
and froze the rules from the training pairs before it read the test pairs.
The [README](../README.md#instruction-datasets) describes the dataset in full.

## Runs

| Date | Dataset version | Runtime | Exit status | Difference | Elapsed |
| --- | --- | --- | --- | --- | --- |
| 4-Nov-2026 09:28 UTC | 1 | Accepted `a4d4c4a`, local | 1 | Unknown: `Error: OPENROUTER_API_KEY is required` | 0.6s |

The run was `bun start 44f52bb0` from `arc/`, run once by `/arc-purposal`
with a 30-minute limit and `NUCLEOID_EXECUTABLE` unset, using the primary
checkout's debug runtime (`nucleoid 1.0.0`) rebuilt with `cargo build` at
`a4d4c4a`. It stopped in the analyzer's first model call, where
`src/lib/openrouter.ts` raised `OPENROUTER_API_KEY is required`, before any
model request, prediction or score, so its difference is unknown, not zero.
The invocation authorizes one run, so it was not repeated.

## Open items

- **No model credential in the local run.** The local `bun start` process had
  no `OPENROUTER_API_KEY` in its environment. A scored run needs one, such as
  the **ARC puzzle report** workflow's repository secret; that workflow's
  `ARC_PUZZLES` list now includes this puzzle, uncommitted. Another local run
  needs a separately authorized invocation.
- **A run would see the answer.** `src/dataset/dataset.ts` sends every JSON
  dataset as few-shot context, and this dataset includes its annotated test
  pairs with their expected outputs, so an exact match would not show that
  the rule was inferred from the training pairs. Suggested, not applied: keep
  the running puzzle's test pairs out of `document(puzzle_id)`.
- **README provenance.** Since `feef5f81`, `arc/README.md` had described a
  `dataset.44f52bb0.json` that was never committed, including facts about its
  test pairs. `/arc-dataset` updated that paragraph in place for this
  dataset. It read the old paragraph during its required README review,
  before the caller warned it, and reports that it fixed the rules from the
  training pairs; its property names match the old paragraph's.
- **Orphaned README entries.** `arc/README.md` still describes `0d3d703e` and
  `27a28665`, which have no datasets, and lists `27a28665` among the `FIT`
  datasets; they are outside this record.
- **Review.** The dataset, its README paragraph, the workflow entry, the
  solver's default puzzle (`6150a2bd` to `44f52bb0` in `src/app.ts`) and this
  record are uncommitted for review.

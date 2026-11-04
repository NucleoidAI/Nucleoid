# ARC-NUC-44f52bb0 -- Mirror Symmetry Check

Progress record for ARC training puzzle `44f52bb0`: its instruction dataset,
the solver and Nucleoid changes it needed, its runs, and what is still open.

| Item | Status |
| --- | --- |
| Puzzle | [`data/training/44f52bb0.json`](../data/training/44f52bb0.json) |
| Instruction dataset | [`dataset.44f52bb0.json`](../src/dataset/dataset.44f52bb0.json): the puzzle's 6 original training pairs and its 2 test pairs, one whole-pattern instance each (8 instances); version 3, which derives `symmetric` by a rule, replacing version 2's input `symmetric` and version 1's 32 point instances; uncommitted |
| Output canvas | `FIT` |
| Nucleoid enhancements | None requested for any version, so no proposal strategy was selected; version 3's rule uses the quantifiers of [NUC 62](../../docs/nuc-0062.md) and the deep-equality `includes` of [NUC 61](../../docs/nuc-0061.md) |
| Solver changes | None; three suggested changes await review, not applied |
| Solver runs | 3 local runs on 4-Nov-2026: version 1 exit 1 before any model call (`OPENROUTER_API_KEY is required`); version 2 exit 0; version 3 exit 0 |
| Updated | 4-Nov-2026 |

## Puzzle

Each training input is a 3 by 3 canvas holding one pattern of color-2 cells,
and each output is a single 1 by 1 cell. The cell is color 1 when the pattern
is left-right mirror symmetric, every cell having a cell of its color in the
same row at the mirrored column, and color 7 otherwise. Three of the six
training inputs are symmetric. The number of cells does not decide the
output, and two inputs that are symmetric only in another sense, a Z that is
a half turn of its own shape and an L that mirrors itself across its own
diagonal, give 7. The training pairs do not separate a flip across the
canvas's center line from a flip across the pattern's own box, because every
symmetric training pattern spans the canvas width. Across the canvas, they do
not separate the left-right mirror from a top-bottom mirror or a half turn
either. The output canvas shrinks from 3 by 3 to 1 by 1, so the dataset is
`FIT`.

## Dataset

The dataset holds the six original training pairs in source order, followed
at the same level by the puzzle's two test pairs, which the same rules
annotate. Every input mask keeps the full 3 by 3 canvas with zero padding,
and every output mask is the complete 1 by 1 output. Each pair has one
instance of `Obj(x, y, orientation, canvas_width, points)`, the whole
pattern, tagged `l9100`, a single shape, as in `dataset.017c7c7b.json`.
`canvas_width` is the input canvas's width, and `points` lists the
`[column, row]` of each of the pattern's cells in reading order: only its
colored cells, not a cell matrix. Unconditional class-level rules decide the
pattern's symmetry and turn it into one point at the output origin:

```nuc
$Obj.tag = 'l7544'
$Obj.symmetric = all point in $Obj.points: $Obj.points.includes([$Obj.canvas_width - 1 - point[0], point[1]])
$Obj.fill_color = 1 if $Obj.symmetric else 7
$Obj.x = 0
$Obj.y = 0
```

A pattern is symmetric when the mirror image of each of its points across the
canvas's vertical center line, in the same row and as far from the right edge
as the point is from the left, is one of its points too. The `all` quantifier
of NUC 62 states that once over the pattern's points, and `includes` compares
the mirrored `[column, row]` by deep equality (NUC 61). `l7544` is the single
point of `0ca9ddb6` and `d037b0a7`, used here only in the output state.
`symmetric` and `fill_color` are absent from the input states. Every training
pattern touches the left column, so `$Obj.x = 0` changes nothing in these
pairs and keeps the output position for patterns that do not; `$Obj.y = 0`
moves the fourth training pair's pattern up from row 1. The declarations
carry `#` comments, as the other datasets do. The
[README](../README.md#instruction-datasets) describes the dataset in full.

### Version 3 and the hint

On 4-Nov-2026 the third version replaced the second at the user's request,
with the hint "Also declaratively formulise $Obj.symmetric". It keeps version
2's single whole-pattern instance and `1 if $Obj.symmetric else 7`, and
derives `symmetric` by a class-level rule instead of carrying it as an input.
The caller read the hint as the user's request for the list of cells that the
derivation needs, as `dataset.5582e5ca.json`'s `parts` was requested; the
rule keeps version 2's meaning of `symmetric`, a flip across the canvas's
center line. The training pairs confirm the hint and contradict nothing.

The arc-dataset worker froze the rules from the training pairs before it read
the test pairs, and the same rules reproduce both. It ran each alternative on
training patterns with the accepted runtime before choosing the rule above:

| Alternative | Result | Why not chosen |
| --- | --- | --- |
| Nested `all`/`any` quantifiers comparing coordinates | Works | Longer; compares coordinate by coordinate |
| `every` with `includes` | Works | The same claim; the quantifier states it more directly |
| Build the mirror image with `map`, then test it | Works | One step more |
| `every` with a nested `some` callback | `ReferenceError: p is not defined` | An inner callback cannot read the outer parameter |
| A flat list of reading-order indices | Works | Positions encoded as numbers and decoded with `%` arithmetic |
| A flip of the pattern's own box, without `canvas_width` | Works | Changes version 2's meaning of `symmetric`; longer |
| A list of `{column, row}` records | Works | The same rule with more verbose data |

`/arc-purposal` requires the declarative, higher-order, and most expressive
solution, so the caller checked the rules against it before the run:
`symmetric` is one quantified statement over the pattern's points,
`fill_color` reads the derived property rather than the points, and no rule
restates another, so they were not sent back. The caller also replayed the
six training snapshots on the runtime, and each matched.

### Version 2 and the hint

Version 2 replaced the first version earlier on 4-Nov-2026, at the user's
request with the hint "Consider whole thing as a single instance, 1 if
$Obj.symmetric else 7". The training pairs confirmed both parts and
contradicted nothing. With one instance per pair and no list of cells,
nothing was left to derive `symmetric` from, so version 2 carried it as an
input property: effectively the deciding answer, which the solver's model had
to judge on each test pattern, allowed because the hint overrode the user's
rule against precomputed answers. Its worker found that a `points` list would
let accepted quantifiers derive it, which version 3 does.

### Versions

| Version | Source | Instances | Rules |
| --- | --- | --- | --- |
| 1 | `/arc-purposal 44f52bb0` in an earlier session, committed in `bca5ff6` | One per colored input cell, a point tagged `l7544` with `color` and `canvas_width`: 32 | `has_mirror` and `symmetric` derived over the point population, `fill_color`, and every point moved to the origin |
| 2 | `/arc-purposal 44f52bb0` with the hint "Consider whole thing as a single instance, 1 if $Obj.symmetric else 7" | One whole pattern per pair, with an input `symmetric`: 8 | `$Obj.tag = 'l7544'`, `fill_color`, `x` and `y`: the rules above without `symmetric` |
| 3 | `/arc-purposal 44f52bb0` with the hint "Also declaratively formulise $Obj.symmetric" | One whole pattern per pair, with `canvas_width` and `points`: 8 | The five rules above |

Version 1's rules, whose `@first` reads of [NUC 53](../../docs/nuc-0053.md)
kept the mirror test on the input coordinates after the last two rules moved
the point:

```nuc
$Obj.has_mirror = Obj.some(other => other.color == $Obj.color and other.y@first == $Obj.y@first and other.x@first == $Obj.canvas_width - 1 - $Obj.x@first)
$Obj.symmetric = Obj.every(point => point.has_mirror)
$Obj.fill_color = 1 if $Obj.symmetric else 7
$Obj.x = 0
$Obj.y = 0
```

## Runs

| Date | Dataset version | Runtime | Exit status | Difference | Elapsed |
| --- | --- | --- | --- | --- | --- |
| 4-Nov-2026 09:28 UTC | 1 | Accepted `a4d4c4a`, local | 1 | Unknown: `Error: OPENROUTER_API_KEY is required` | 0.6s |
| 4-Nov-2026 15:17 UTC | 2 | Accepted `a4d4c4a`, local | 0 | 0 | 2m 53.9s |
| 4-Nov-2026 15:40 UTC | 3 | Accepted `628b0ad`, local | 0 | 0 | 1m 46.4s |

Each run was `bun start 44f52bb0` from `arc/`, run once by its own
`/arc-purposal` invocation with a 30-minute limit and `NUCLEOID_EXECUTABLE`
unset, using the primary checkout's debug runtime (`nucleoid 1.0.0`). The
first stopped in the analyzer's first model call, where
`src/lib/openrouter.ts` raised `OPENROUTER_API_KEY is required`, before any
model request, prediction or score, so its difference is unknown, not zero.
The second ran at `657f0de` on the binary built at `a4d4c4a`, since no Rust
source had changed. Before the third, `cargo build` rebuilt the runtime at
`628b0ad`, whose changes since `a4d4c4a` are unit-test code only. In the
second and third runs, the runtime's hash and the dataset's were the same
before and after the run. Each exit status is the difference score, neither
scored run's log records an error, and elapsed is the solver's own report.
Each invocation authorizes one run, so none was repeated.

## Open items

- **The runs saw the answer.** `src/lib/llm.ts` sends `document(puzzle_id)` as
  few-shot context, and `src/dataset/dataset.ts` discards `puzzle_id`. The
  second and third runs' prompts therefore held this dataset's two annotated
  test pairs: their expected outputs and each test instance's implementation,
  with `symmetric` in version 2 and `canvas_width` and `points` in version 3.
  Neither exact match shows that the rules were inferred from the training
  pairs or that the model read the test patterns itself. Suggested, not
  applied: keep the running puzzle's test pairs out of `document(puzzle_id)`.
- **`points` is a list of pairs.** It holds only the pattern's colored cells,
  so it is not a grid or mask, but it is a list of `[column, row]` pairs
  rather than a flat list of single values. A flat list of reading-order
  indices also works, with `%` arithmetic to decode positions. Confirm that
  the list of pairs is acceptable.
- **Canvas size.** `canvas_width` is an input the solver's model must supply
  for the test instance, and `symmetric` is defined across the canvas's
  center line, but `value()` in `src/visualizer.ts` sends the model only the
  cropped pattern and its `input_x` and `input_y`, not the canvas size.
  Suggested, not applied, as in the `d037b0a7` and `6150a2bd` records: pass
  the test input's row and column counts to `visualizer.value`.
- **Messages are not JSON-only.** `src/lib/llm.ts`, `src/analyzer.ts`, and
  `src/visualizer.ts` wrap JSON in text labels such as `few_shot:` and
  `input_x:`, while the ARC instructions require every message to parse as one
  JSON document. Suggested, not applied: build each message with
  `JSON.stringify` of one data object.
- **Comments in declarations.** `.claude/rules/arc.instructions.md` permits
  executable code in `declarations` but no prose comments. Versions 2 and 3
  carry `#` comments, as the other datasets do, while version 1 had none.
  Decide which wins; removing the comments needs no new snapshots.
- **Test-pair facts.** Since `feef5f81`, `arc/README.md` had described a
  `dataset.44f52bb0.json` that was never committed, including facts about its
  test pairs. For version 1, `/arc-dataset` read that paragraph during its
  required README review, before the caller warned it, and updated it in
  place. For versions 2 and 3, the worker's README review read the previous
  version's paragraph, which describes the test pairs, before it froze the
  rules from the training pairs. Version 2's handoff also described one test
  input's shape and verdict; version 3's reported its test checks as pass or
  fail only. The caller read no test entry, and nothing from these facts
  changed a dataset or reached another agent or a run.
- **Stale references elsewhere.** The `dataset.27a28665.json` paragraph in
  `arc/README.md` says `l7544` is used as in `dataset.44f52bb0.json` "in both
  states"; since version 2, this dataset uses it only in the output state.
  The README also still describes `0d3d703e` and `27a28665`, which have no
  datasets, and lists `27a28665` among the `FIT` datasets, and
  [`ARC-NUC-d037b0a7.md`](ARC-NUC-d037b0a7.md) still says this dataset does
  not exist. They are outside this dataset's paragraph and were left as they
  are.
- **The first run had no model credential.** That process had no
  `OPENROUTER_API_KEY` in its environment; the later runs' processes reached
  the model.
- **Review.** Version 1, its README paragraph, the workflow entry, the
  solver's default puzzle and this record's first version were committed in
  `bca5ff6`. Version 2 was never committed. Version 3, its README paragraph
  and this update are uncommitted for review.

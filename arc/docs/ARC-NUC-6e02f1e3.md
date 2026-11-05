# ARC-NUC-6e02f1e3 -- Color Diversity Lines

Progress record for ARC training puzzle `6e02f1e3`: its instruction dataset,
the solver and Nucleoid changes it needed, its runs, and what is still open.

| Item | Status |
| --- | --- |
| Puzzle | [`data/training/6e02f1e3.json`](../data/training/6e02f1e3.json) |
| Instruction dataset | Not published: workflow timeout policy conflict. The prospective dataset's 5 training pairs and 1 test pair, with 6 whole-mosaic instances, were verified offline. |
| Output canvas | `EXACT`, inferred from the training pairs |
| Nucleoid enhancements | None: accepted distinct-value and class-level rules express the model |
| Solver changes | JSON-only messages and held-out context exclusion suggested, not applied |
| Solver runs | None in this invocation; live difference unknown |
| Updated | 5-Nov-2026 |

## Puzzle

Each training input is a completely filled 3 by 3 mosaic. The number of
distinct categorical colors determines the direction of one gray line:
one color produces a horizontal bar across the top, two produce a
northwest-to-southeast diagonal, and three produce a
southwest-to-northeast diagonal. The gray foreground is color 5 and the
output background is color 0. The original 3 by 3 canvas stays intact,
including the two empty rows beneath a horizontal bar, so the model uses
`EXACT`, not `FIT`.

The default selection chose this puzzle from 381 eligible puzzles: its
largest training input or output matrix has 9 cells, and all training
matrices total 90 cells. The accepted starting revision was `f4cf6803`;
the working tree was clean and the pulls received no commits.

## Dataset

The dataset worker started from the entire mosaic as one pattern, not from
cells or separate color regions. Its row-major color list, `parts`, is a
descriptor of that whole object. One class-level expression counts distinct
colors and derives the resulting line's orientation. No finer-object
fallback, imperative steps, or per-pair rules were needed.

The frozen declaration set is:

```nuc
class Obj(x: int, y: int, orientation: int, parts: list):
    this.x = x
    this.y = y
    this.orientation = orientation
    this.tag = 'l9100'
    this.parts = parts

$Obj.fill_color = 5
$Obj.orientation = [90, 135, 45][$Obj.parts.distinct().length - 1]

if $Obj.orientation == 90:
    $Obj.tag = 'l3058'
    $Obj.bar_width = canvas_width

if $Obj.orientation == 45 or $Obj.orientation == 135:
    $Obj.tag = 'l8831'
    $Obj.line_length = canvas_width
```

The input pattern tag `l9100` and horizontal-bar tag `l3058` reuse existing
geometries. The proposed unused tag `l8831` describes an unmarked,
one-cell-thick diagonal whose raster-cell length is `canvas_width`.
Positions remain at the origins inferred from the training pairs; the
rules create the fill color and the applicable width or length.

The worker froze the model from training before evaluating the held-out
pair. Manual/offline verification passed for all six original pairs:
full-canvas masks, recomposition, identities, native snapshot replay, and
the unchanged rule set. Twelve native programs were executed and replayed,
for 24 successful Nucleoid executions. These checks are not a live-model
solver result or an automated ARC test.

Publication was blocked, so no canonical `dataset.6e02f1e3.json`, README
paragraph, or reporting-list entry was created. The model and verification
evidence remain in the invocation's external session workspace. The
[instruction dataset documentation](../README.md#instruction-datasets)
describes the retained dataset conventions.

## Publication Blocker

The dataset profile's literal minimum is thirty minutes per listed puzzle
plus twenty minutes for setup and reporting. Adding this puzzle to the
nineteen existing entries would require 620 minutes, exceeding the
GitHub-hosted job limit of 360 minutes.

The actual workflow already uses `ARC_CONCURRENCY = 5`: twenty puzzles
need four thirty-minute batches plus twenty minutes, or 140 minutes.
Its existing 150-minute timeout therefore covers the batched execution,
but does not satisfy the profile's literal serial budget. The worker
returned `blocked` instead of overriding the policy or publishing a partial
dataset. Neither the policy nor the workflow was changed.

No NUC was requested, so proposal-strategy learning is `not_applicable`;
there was no strategy selection, trial, rubric, or checkpoint.

## Runs

On 5-Nov-2026, no live solver run was started. The authorized
`bun start 6e02f1e3` command was not executed because the dataset handoff
was blocked. Its live exit status, difference, and elapsed time are
unknown, not zero. The solver's previous default, `ed36ccf7`, remains
unchanged.

The primary `cargo build` and ARC type checking succeeded. The existing
ARC lint command failed with 28 `prettier/prettier` errors in unchanged
`src/analyzer.ts` and `src/visualizer.ts`; no solver source was edited.

## Open Items

- Resolve the serial-versus-batched timeout requirement in
  `.claude/agents/arc-dataset.agent.md` before publishing the dataset.
- Make every analyzer, visualizer, and shared-wrapper message a complete
  JSON document. `src/analyzer.ts`, `src/visualizer.ts`, and
  `src/lib/llm.ts` currently concatenate labels and pseudo-JSON rather
  than serializing structured operation and schema objects.
- Exclude the active puzzle's held-out records from model context without
  changing canonical datasets or inspecting held-out grids. In
  `src/dataset/dataset.ts`, `document(puzzle_id)` ignores its argument;
  `src/lib/llm.ts` prepends every instruction file, which would include
  this puzzle's expected held-out outputs after publication.
- Supply `OPENROUTER_API_KEY` through the process environment before a
  separately authorized run. Only its absence was checked; no
  environment file or credential content was read.
- Resolve the pre-existing solver formatting errors in a separate change.
  Solver changes above are suggestions only and were not applied.

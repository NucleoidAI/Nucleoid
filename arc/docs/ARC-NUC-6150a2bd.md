# ARC-NUC-6150a2bd -- Half-Turn Rotation

Progress record for ARC training puzzle `6150a2bd`: its instruction dataset,
the solver and Nucleoid changes it needed, its runs, and what is still open.

| Item | Status |
| --- | --- |
| Puzzle | [`data/training/6150a2bd.json`](../data/training/6150a2bd.json) |
| Instruction dataset | [`dataset.6150a2bd.json`](../src/dataset/dataset.6150a2bd.json): the puzzle's 2 original training pairs and its test pair, 12 region instances |
| Output canvas | `EXACT` |
| Nucleoid enhancements | None requested; the rules use the `@first` anchors of [NUC 53](../../docs/nuc-0053.md) and [NUC 55](../../docs/nuc-0055.md) |
| Solver changes | None specific to this puzzle |
| Solver runs | 1 GitHub Actions run, 2-Nov-2026: exit 0 |
| Updated | 2-Nov-2026 |

## Puzzle

Each training input is a 3 by 3 canvas of colored cells and zeros. The output
turns the whole input a half turn, 180 degrees, about the canvas center. The
canvas keeps its size, so the dataset is `EXACT`.

## Dataset

The single dataset holds the two original training pairs in source order,
followed by the original test pair at the same level, with 4, 3, and 5
instances. Each object is one 4-connected region of a single color, an
`Obj(x, y, orientation, color, width, height, canvas_width, canvas_height)`
tagged `l9100`. Unconditional class rules move each region to the opposite
side of the canvas and turn it:

```nuc
$Obj.x = $Obj.canvas_width - $Obj.x@first - $Obj.width
$Obj.y = $Obj.canvas_height - $Obj.y@first - $Obj.height
$Obj.orientation = ($Obj.orientation@first + 180) % 360
```

The `@first` anchors read each region's input origin and orientation, so no
rule reads the value it writes. Every region keeps its color and cell count.
The dataset was committed in `ed49ce3`; the
[README](../README.md#instruction-datasets) describes it in full.

## Runs

| Date | Run | Commit | Exit status | Difference | Seconds |
| --- | --- | --- | --- | --- | --- |
| 2-Nov-2026 06:45 UTC | [41401913542](https://github.com/NucleoidAI/Nucleoid/actions/runs/41401913542) | `ed49ce3` | 0 | 0 | 157 |

The run is the **ARC puzzle report** workflow's
`bun --no-env-file run start 6150a2bd` on `ubuntu-latest`, with the commit's
debug runtime and a 30-minute limit. Seconds are the workflow's wall time for
the puzzle, from the run's `arc-puzzle-report` artifact. No local
`bun start 6150a2bd` run is recorded.

## Open items

- **The run saw the answer.** Since `c2fff09`, `src/dataset/dataset.ts` sends
  every JSON dataset as few-shot context, and this puzzle's dataset includes
  its annotated test pair with the expected output. This exact match therefore
  does not show that the rule was inferred from the training pairs. Suggested,
  not applied: keep the running puzzle's test pairs out of
  `document(puzzle_id)`.

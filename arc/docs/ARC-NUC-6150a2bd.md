# ARC-NUC-6150a2bd -- Half-Turn Rotation

Progress record for ARC training puzzle `6150a2bd`: its instruction dataset,
the solver and Nucleoid changes it needed, its runs, and what is still open.

| Item | Status |
| --- | --- |
| Puzzle | [`data/training/6150a2bd.json`](../data/training/6150a2bd.json) |
| Instruction dataset | [`dataset.6150a2bd.json`](../src/dataset/dataset.6150a2bd.json): the puzzle's 2 original training pairs and its test pair, one whole-block instance each (3 instances); the second version, which `cc87228` committed, replacing the first, with 12 region instances, which `ed49ce3` committed |
| Output canvas | `EXACT` |
| Nucleoid enhancements | None requested; the rules use the `@first` anchors of [NUC 53](../../docs/nuc-0053.md) and [NUC 55](../../docs/nuc-0055.md) |
| Solver changes | None; three suggested changes await review, not applied |
| Solver runs | 1 GitHub Actions run on 2-Nov-2026 for version 1 and 1 local run on 3-Nov-2026 for version 2, both exit 0 |
| Updated | 3-Nov-2026 |

## Puzzle

Each training input is a 3 by 3 canvas of colored cells and zeros. The output
turns the whole input a half turn, 180 degrees, about the canvas center. The
canvas keeps its size, so the dataset is `EXACT`. In both training inputs the
nonzero cells form one edge-connected block across colors: six cells of four
colors, then four cells of three colors. The first block's bounding box fills
the canvas. The second is 3 cells wide and 2 tall, so the half turn moves it
from the top two rows to the bottom two.

## Dataset

The dataset holds the two original training pairs in source order, followed
by the original test pair at the same level. Each pair is one instance, the
whole block of nonzero cells: `Obj(x, y, orientation, width, height,
canvas_width, canvas_height)`, tagged `l9100`, a single shape. The class has
no color, parts list, or cell matrix, and zero cells are background.
Unconditional class rules turn the block and move its bounding box:

```nuc
$Obj.orientation = ($Obj.orientation@first + 180) % 360
$Obj.x = $Obj.canvas_width - $Obj.x@first - $Obj.width
$Obj.y = $Obj.canvas_height - $Obj.y@first - $Obj.height
```

Orientation is 0 in every input state and 180 in every output state. The
solver's model redraws each block from its input mask and that change of
orientation, then places the drawing at the output `x` and `y`. The `@first`
anchors read the block's input origin and orientation, so no rule reads the
value it writes. The 3 by 3 blocks keep their origin at 0 and 0, while the
second training block moves down one row. Like the other datasets since
`e1bdb5a`, the declarations carry `#` comments. The
[README](../README.md#instruction-datasets) describes the dataset in full.

On 3-Nov-2026 the second version replaced the first at the user's request,
with the hint "how about consider whole thing is one instance change
orientation to 180 and redraw". The training pairs confirm one instance per
pair, since all nonzero cells form one block, and the 180-degree turn. They
contradict two readings of the hint. The whole 3 by 3 canvas cannot be the
object, because its origin would stay at 0 while the second training block
moves down a row. Redrawing the turned block at its input origin fails for the
same reason. The rules therefore still move `x` and `y`, as version 1's did.
The arc-dataset worker wrote the dataset and its README paragraph. The user
committed both, with the solver's default puzzle, in `cc87228` during the run.

### Versions

| Version | Request | Instances | Rules |
| --- | --- | --- | --- |
| 1 | The puzzle's first dataset, `ed49ce3` | One per 4-connected single-color region, with `color`, `width`, `height`, `canvas_width`, and `canvas_height`: 12, with 4, 3, and 5 per pair | Orientation turned by 180 with `@first`; `x` and `y` moved to the opposite side of the canvas |
| 2 | `/arc-purposal 6150a2bd` with the hint "how about consider whole thing is one instance change orientation to 180 and redraw" | One whole block per pair, without `color`: 3 | The same three rules |

## Runs

| Date | Dataset version | Runtime | Exit status | Difference | Elapsed |
| --- | --- | --- | --- | --- | --- |
| 2-Nov-2026 06:45 UTC | 1, per-color regions | `ed49ce3`, GitHub Actions run [41401913542](https://github.com/NucleoidAI/Nucleoid/actions/runs/41401913542) | 0 | 0 | 157s |
| 3-Nov-2026 11:01 UTC | 2, one whole block per pair | Accepted `7709519`, local | 0 | 0 | 1m 16.3s |

The first run is the **ARC puzzle report** workflow's
`bun --no-env-file run start 6150a2bd` on `ubuntu-latest`, with the commit's
debug runtime and a 30-minute limit. Its elapsed time is the workflow's wall
time for the puzzle, from the run's `arc-puzzle-report` artifact.

The second run was `bun start 6150a2bd` from `arc/`, run once by
`/arc-purposal` with a 30-minute limit. It used the primary checkout's debug
runtime (`nucleoid 1.0.0`), rebuilt with `cargo build` at `7709519`, whose
only source change since `4ad55da` is the REPL header in `src/terminal/`. The
dataset worker had captured the snapshots with the build before it. Rerun
with the rebuilt runtime before the solver run, the worker's check script
reproduced every input and output snapshot. The runtime's hash was the same
before and after the run, and the dataset the run read is byte for byte the
file `cc87228` committed. Elapsed is the solver's own report.

## Open items

- **Every run saw the answer.** `src/lib/llm.ts` sends `document(puzzle_id)`
  as few-shot context, and `src/dataset/dataset.ts` discards `puzzle_id`. Both
  prompts therefore held this puzzle's annotated test pair, expected output
  included, so neither exact match shows that the rule was inferred from the
  training pairs. While overwriting, the dataset worker also had version 1 at
  hand, which held the annotated test pair; it reports inferring the rules and
  the canvas mode from the training pairs. Suggested, not applied: keep the
  running puzzle's test pairs out of `document(puzzle_id)`.
- **The test canvas's size is not in the instance prompt.** `value()` in
  `src/visualizer.ts` sends the test instance's mask cropped by
  `Matrix.crop`, with only `input_x` and `input_y`, and crops the training
  masks as well. Nothing in that message gives the test canvas's width and
  height, which the model needs for `canvas_width` and `canvas_height`. It can
  only carry 3 over from the training implementations or the few-shot
  datasets. Suggested, not applied: pass the test input's row and column
  counts to `visualizer.value`. [ARC-NUC-d037b0a7](ARC-NUC-d037b0a7.md)
  records the same need.
- **Messages are not JSON-only.** `src/lib/llm.ts`, `src/analyzer.ts`, and
  `src/visualizer.ts` wrap JSON in text labels such as `few_shot:`,
  `train_dataset:`, and `input_x:`, while the ARC instructions require every
  message to parse as one JSON document. Suggested, not applied: build each
  message with `JSON.stringify` of one data object.
- **Comments in declarations.** `.claude/rules/arc.instructions.md` permits
  executable code in `declarations` but no prose comments, while version 2
  follows the `#` comments that `e1bdb5a` added to every dataset. Decide which
  wins. Removing the comments needs no new snapshots.
- Review and commit this record's update, which remains uncommitted. Since
  3-Nov-2026 only `/nuc-proposal` commits and pushes, so `/arc-purposal`
  leaves its changes for review.

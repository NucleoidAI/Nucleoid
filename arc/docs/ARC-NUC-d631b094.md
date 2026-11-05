# ARC-NUC-d631b094 -- Cell Count Bar

Progress record for ARC training puzzle `d631b094`: its instruction dataset,
the solver and Nucleoid changes it needed, its runs, and what is still open.

| Item | Status |
| --- | --- |
| Puzzle | [`data/training/d631b094.json`](../data/training/d631b094.json) |
| Instruction dataset | [`dataset.d631b094.json`](../src/dataset/dataset.d631b094.json): the puzzle's 4 original training pairs and its test pair, one region instance each (5 instances) |
| Output canvas | `FIT` |
| Nucleoid enhancements | None; the dataset uses accepted features only |
| Solver changes | None specific to this puzzle |
| Solver runs | 1 local run, 2-Nov-2026: exit 0 |
| Updated | 5-Nov-2026 |

## Puzzle

Each training input is a 3 by 3 canvas with cells of a single nonzero color.
The output is one row of that color, as long as the number of its cells:
1 by 2, 1 by 3, 1 by 1, and 1 by 4 in the training pairs. Each output is
smaller than its input and fills its own bounding box, so the dataset is
`FIT`; see
[ARC-NUC-28bf18c6](ARC-NUC-28bf18c6.md#decision-output_canvas-trims-the-composed-matrix).

## Dataset

The single dataset holds the four original training pairs in source order,
followed by the original test pair at the same level. All cells of the one
color form one region, whether or not they touch, and each region is an
`Obj(x, y, orientation, color, cell_count)` tagged `l9100`, where
`cell_count` is its number of cells. Unconditional class rules turn it into a
bar at the origin:

```nuc
$Obj.tag = 'l3058'
$Obj.bar_width = $Obj.cell_count
$Obj.x = 0
$Obj.y = 0
```

`l3058` is a horizontal bar. The dataset was committed in `b9d6551`; the
[README](../README.md#instruction-datasets) describes it in full.

### Gestalt review

On 5-Nov-2026 the dataset was checked against the gestalt requirement, which
`/arc-purposal` and the arc-dataset profile added on 4-Nov-2026, after this
dataset was written: start from the big picture, the figures each grid shows,
and go down to single cells only when that cannot express what the training
pairs show. The dataset meets it. All cells of the one color read as one
group, whether or not they touch, and that whole group is the object whose
size sets the bar. Nothing needs to change.

## Runs

| Date | Command | Runtime | Exit status | Difference | Elapsed |
| --- | --- | --- | --- | --- | --- |
| 2-Nov-2026 | `bun start d631b094` | Primary checkout's debug runtime at `a3e15ce` | 0 | 0 | 1m 7.5s |

An `/arc-purposal` session ran it once, without a NUC, and kept its result
record in that session's workspace. The puzzle joined the **ARC puzzle
report** workflow after that workflow's last run, so no workflow run is
recorded.

## Open items

- **The run saw the answer.** `src/dataset/dataset.ts` sends every JSON
  dataset as few-shot context, and this puzzle's dataset includes its
  annotated test pair with the expected output. This exact match therefore
  does not show that the rule was inferred from the training pairs.
  Suggested, not applied: keep the running puzzle's test pairs out of
  `document(puzzle_id)`.

# ARC-NUC-28bf18c6 -- Repeated Shape Crop

Progress record for ARC training puzzle `28bf18c6`: its instruction dataset,
the solver and Nucleoid changes it needed, its runs, and what is still open.

| Item | Status |
| --- | --- |
| Puzzle | [`data/training/28bf18c6.json`](../data/training/28bf18c6.json) |
| Instruction dataset | [`dataset.28bf18c6.json`](../src/dataset/dataset.28bf18c6.json): 3 synthesized pairs, 3 instances |
| Output canvas | `FIT` |
| Nucleoid enhancements | None; the dataset uses accepted features only |
| Solver changes | `output_canvas`, applied by `Matrix.canvas` |
| Solver runs | None recorded |
| Updated | 29-Oct-2026 |

## Puzzle

Each training pair holds one shape on an 8 by 8 canvas. The output repeats
the shape's 3 by 3 crop twice, side by side, as a 3 by 6 grid. The output is
smaller than the input and has no empty border: it exactly fills its own
nonzero bounding box.

## Dataset

Three synthesized pairs follow the rule with freshly sampled canvases, shapes,
colors, and positions: 7 by 10 to 2 by 8, 6 by 5 to 4 by 4, and 11 by 11 to
2 by 6 (rows by columns). Every shape is left-right asymmetric, so the pairs
tell repeating apart from mirroring. One `Obj(x, y, orientation, color)` class
carries unconditional class rules that hold for every instance:

```nuc
$Obj.tag = 'l4305'
$Obj.repeat_count = 2
$Obj.repeat_direction = 'right'
$Obj.x = 0
$Obj.y = 0
```

`repeat_count` and `repeat_direction` describe the repetition, and `x` and `y`
move the object to the output origin, where the solver places the repeated
crop. The [README](../README.md#instruction-datasets) describes the dataset in
full.

## Decision: `output_canvas` trims the composed matrix

### Problem

The solver composes its prediction on a canvas that is never smaller than the
test input:

- The result starts as a zero matrix with the test input's dimensions.
- `Matrix.pad` places each predicted output crop at the `x` and `y` of the
  object's output snapshot, treating the test input's dimensions as minimums.
- `Matrix.merge` keeps the largest height and width of its inputs.

That suits puzzles whose output keeps the input canvas, such as `0ca9ddb6` and
the genesis puzzle `3aa6fb7a`. This puzzle instead turns an 8 by 8 input into
a 3 by 6 output, so even a perfect prediction composes as the correct 3 by 6
content in the top-left corner of an 8 by 8 zero canvas. Scoring is exact grid
equality, shape included, and `Matrix.difference` counts every extra cell. On
a training pair, that perfect prediction would still score 46: 6 extra cells
beside the three content rows and 40 in the five extra rows. No run of this
puzzle could reach an exact match.

### Decision

Each dataset declares `output_canvas`, one of a closed set of framing modes.
`Matrix.canvas` applies it once to the merged result, just before comparison:

| Value | Framing |
| --- | --- |
| `EXACT` | Keep the composed canvas: the test input's dimensions, or larger where a placed object extends past them |
| `FIT` | Crop the canvas to its nonzero bounding box with `Matrix.crop` |
| Any other value | Fail explicitly |

The analyzer infers the value from the training pairs in the same response as
the declarations. This dataset and `9af7a82c` are `FIT`; `0ca9ddb6` and the
genesis datasets are `EXACT`.

### Reasoning

- **Framing belongs to the whole output, not to one object.** Only the merged
  canvas shows the extent of every placed object, so the trim runs once, at
  the end.
- **Nothing upstream changes.** Instance masks stay full-size and zero-padded,
  `x` and `y` stay canvas coordinates, and padding and merging keep their
  semantics. Trimming earlier would have switched objects to cropped
  representations, which the ARC instructions rule out without an explicit
  conversion.
- **The frame comes from training pairs, never the test output.** Sizing the
  result from the expected test grid would leak the answer. `FIT` derives the
  size from the prediction itself, and the mode is chosen from the training
  pairs, where every output fills its nonzero bounding box.
- **A closed enum keeps the exchange structured.** The model picks one of two
  identifiers instead of predicting numeric dimensions for each input, and an
  unknown value fails instead of falling back to a default.
- **Cropping cannot be unconditional.** `0ca9ddb6` and genesis outputs keep
  empty rows and columns at their edges, and cropping would break them. The
  framing is therefore a per-puzzle choice, and it lives in the dataset.
- **The world model and the frame split the work.** The rules describe the
  repetition and move the object to the origin; `FIT` removes the input-sized
  remainder. Because `FIT` crops all four sides, the result would not change
  if the object were placed away from the origin.

### Limits

- `FIT` removes every all-zero border row and column, so it cannot produce an
  expected output that keeps an empty edge.
- Neither mode produces an output that is smaller than the test input but not
  tight around its content, or larger than the test input with zero padding
  beyond the placed objects. Such puzzles need another mode, for example
  dimensions computed by the world model.
- `FIT` returns `[]` for an all-zero result.
- The value is checked only when `Matrix.canvas` runs, after every model call
  in the run, so an unknown value fails at the end rather than when the
  analyzer receives it.

## Runs

No `bun start 28bf18c6` run is recorded yet; its difference is unknown, not
zero.

## Open items

- Record the first run's date, runtime, exit status, and difference.

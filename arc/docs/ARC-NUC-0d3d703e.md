# ARC-NUC-0d3d703e -- Paired Color Swap

Progress record for ARC training puzzle `0d3d703e`: its instruction dataset,
the solver and Nucleoid changes it needed, its runs, and what is still open.

| Item | Status |
| --- | --- |
| Puzzle | [`data/training/0d3d703e.json`](../data/training/0d3d703e.json) |
| Instruction dataset | [`dataset.0d3d703e.json`](../src/dataset/dataset.0d3d703e.json): the puzzle's 4 original training pairs and its test pair, 15 stripe instances, three per pair; committed in `ebd026c` |
| Output canvas | `EXACT` |
| Nucleoid enhancements | None requested, so no proposal strategy was selected; the rule uses the `@first` anchors of [NUC 53](../../docs/nuc-0053.md) and [NUC 55](../../docs/nuc-0055.md) and the list `includes` that [NUC 61](../../docs/nuc-0061.md) specifies |
| Solver changes | None; suggested changes await review, not applied |
| Solver runs | 1 local run on 4-Nov-2026, exit 0 |
| Updated | 6-Nov-2026 |

## Puzzle

Each training input is a 3 by 3 canvas of three vertical stripes, each one
column wide and three rows tall and each of its own color, so no cell is zero.
The output keeps every stripe in place and replaces its color with a partner
color. The colors form four pairs, 1 and 5, 2 and 6, 3 and 4, and 8 and 9,
whose two colors trade places, and the training pairs show every pair in both
directions: 3, 1, 2 become 4, 5, 6; 2, 3, 8 become 6, 4, 9; 5, 8, 6 become
1, 9, 2; and 9, 4, 2 become 8, 3, 6. Colors 0 and 7 do not appear in the
training pairs. The partners
differ by 4, 4, 1, and 1, so no arithmetic relates them, and colors are
categorical anyway. The output keeps the input's canvas, so the dataset is
`EXACT`.

`/arc-purposal` chose this puzzle by default: among the training puzzles other
than the genesis puzzle `3aa6fb7a` and those with a dataset, its largest
training matrix has the fewest cells, 9, with 72 cells in all its training
matrices. Seven other puzzles tie on both measures, and `0d3d703e` has the
lowest ID among them.

## Gestalt overview

- **Whole grid.** A 3 by 3 canvas of three solid vertical stripes, each one
  column of its own color, with no background cell.
- **Figures.** The three stripes, seen as columns rather than cells.
- **Relations.** No stripe depends on its neighbors. Each stripe's color has
  one partner in four fixed pairs: 1 and 5, 2 and 6, 3 and 4, and 8 and 9.
- **Change.** The grid keeps every stripe in place and gives each stripe its
  partner color.
- **World model.** One object per stripe, the figures seen first. The whole
  grid as one object holding a list of its stripe colors would be one level
  coarser, but accepted Nucleoid cannot run that form, and the stripe model
  is the simpler one.

## Dataset

The dataset holds the four original training pairs in source order, followed
at the same level by the puzzle's test pair, which the same rule annotates.
Every stripe is one instance of `Obj(x, y, orientation, color)`, a vertical
bar tagged `l6953` in both states, as in `dataset.d037b0a7.json` and
`dataset.9af7a82c.json`. One unconditional class-level rule gives every stripe
the other color of its pair:

```nuc
$Obj.color = [[1, 5], [2, 6], [3, 4], [8, 9]].find(pair => pair.includes($Obj.color@first)).find(partner => partner != $Obj.color@first)
```

The list states each pair once: the first `find` selects the pair that
includes the stripe's input color, and the second takes that pair's other
color. A dictionary from every color to its partner also runs, but it states
each pair twice. The two callbacks are chained rather than nested, so each
reads only its own parameter and the stripe, and the `@first` anchors read the
input color because the rule rewrites `color` itself. The rule creates no
property and leaves `x`, `y`, `orientation`, and the tag unchanged, so input
and output states share the same keys. The declarations carry `#` comments,
as the other datasets do. The [README](../README.md#instruction-datasets)
describes the dataset in full.

The worker started from the big picture: each grid is three stripes side by
side, the figures seen first, and the whole grid changes only their colors. It
also tried one level coarser, the whole grid as one object holding a list of
its stripe colors. Accepted Nucleoid cannot run that form, because the lookup
nested inside `map` raises `ReferenceError: color is not defined`: a nested
callback cannot read its enclosing callback's parameter. The stripe model is
also the simpler one, so no finer model was needed and no enhancement was
requested. The worker fixed the rule from the training pairs and confirmed it
reproduces all four training outputs before reading the test pair; the test
pair was then reproduced without any change. The caller checked the handoff
against the declarative, higher-order, and gestalt requirements and sent
nothing back. No hint was given.

The arc-dataset worker wrote the dataset, its README paragraph, and the
puzzle's entry in the **ARC puzzle report** workflow's list, and the caller
made the puzzle the solver's default in `src/app.ts` before the run. All four
changes stayed uncommitted, as `/arc-purposal` requires, until `ebd026c`, made
outside the invocation during the run, committed and pushed them. Its dataset
is byte-identical to the one the run used. The README paragraph replaced a
stale paragraph from an earlier session that described a
`dataset.0d3d703e.json` never committed; the worker was told not to read it
and replaced it by script.

### Standard entries

On 5-Nov-2026, at the user's request, every ARC dataset moved to the standard
object entries `x`, `y`, `width`, `height`, and `orientation`. `width` and
`height` are the column and row counts of the object's input bounding box. An
orientation change alone leaves them unchanged, and a rule changes them only
when the shape itself changes size, reading the input with `@first`. Every
implementation now passes the input box's `width` and `height` after `y`, and
the class and rules read:

```nuc
class Obj(x: int, y: int, width: int, height: int, orientation: int, color: int):
$Obj.color = [[1, 5], [2, 6], [3, 4], [8, 9]].find(pair => pair.includes($Obj.color@first)).find(partner => partner != $Obj.color@first)
```

No rule changed: the color swap leaves every stripe 1 column by 3 rows. The
migration worker regenerated every snapshot with the accepted runtime at
`491512d`, and the caller replayed every pair offline against the unchanged
grids, masks, positions and instance names. No solver run has measured this
version yet.

## Runs

| Date | Dataset version | Runtime | Exit status | Difference | Elapsed |
| --- | --- | --- | --- | --- | --- |
| 4-Nov-2026 19:05 UTC | 1, one stripe per object | Accepted `88c781d`, local | 0 | 0 | 4m 42.7s |

The run was `bun start 0d3d703e` from `arc/`, run once by `/arc-purposal` with
a 30-minute limit and `NUCLEOID_EXECUTABLE` unset. It used the primary
checkout's debug runtime (`nucleoid 1.0.0`), up to date with `cargo build` at
`88c781d`. The runtime's hash and the dataset's were the same before and after
the run. The solver was the committed one at `88c781d`, with only its default
puzzle changed from `27a28665` to this puzzle. Elapsed is the solver's own
report; the process ran from 19:05:52 to 19:10:35 UTC.

When the invocation started, the primary checkout held an uncommitted edit,
not made by the invocation, that ended `src/dataset/dataset.ts`'s few-shot
document in `//]`, which is not valid JSON. Asked before the run, the user
chose to restore the committed line. The file was already back at its
committed version when the caller went to restore it, so the caller changed
nothing there. The few-shot document parsed, with all 15 JSON datasets,
before the run. The puzzle has not been in an **ARC puzzle report** workflow
run.

## Open items

- **The run saw the answer.** Resolved on 6-Nov-2026: the user decided this is
  intended, because the datasets are being prepared as few-shot examples and
  runs at this stage are not yet held-out tests. The ARC instructions now say
  so, and the suggested change to keep the running puzzle's test pairs out of
  `document(puzzle_id)` is withdrawn. `src/lib/llm.ts` sends
  `document(puzzle_id)` as few-shot context, and `src/dataset/dataset.ts`
  discards `puzzle_id`. The prompts therefore held this dataset's annotated
  test pair, expected output included, and the exact match does not show that
  the rule was inferred from the training pairs.
- **Messages are not JSON-only.** `src/lib/llm.ts`, `src/analyzer.ts`, and
  `src/visualizer.ts` wrap JSON in text labels such as `few_shot:`, while the
  ARC instructions require every message to parse as one JSON document.
  Suggested, not applied: build each message with `JSON.stringify` of one data
  object.
- **Comments in declarations.** Resolved on 6-Nov-2026: the user decided that
  natural-language `#` comments may reach the model only as part of
  `declarations`, and the ARC instructions now say so. Datasets with and
  without such comments both follow them, so nothing needs to change.
- **Workflow timeout.** Resolved on 6-Nov-2026: at the user's request, the
  arc-dataset profile now asks for the batch budget the workflow needs,
  `ceil(listed puzzles / ARC_CONCURRENCY) * 30 + 20` minutes, within GitHub
  Actions' 360-minute job limit. The workflow's 150-minute timeout covers its
  20 listed puzzles.
- **Stale item in another record.** [ARC-NUC-27a28665](ARC-NUC-27a28665.md)
  still lists the stale README paragraph about `dataset.0d3d703e.json` as
  open; `ebd026c` replaced that paragraph, so the item can be closed there.
- **Committed.** `a526fec` committed this record with the dataset.

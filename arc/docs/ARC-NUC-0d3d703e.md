# ARC-NUC-0d3d703e -- Paired Color Swap

Progress record for ARC training puzzle `0d3d703e`: its instruction dataset,
the solver and Nucleoid changes it needed, its runs, and what is still open.

| Item | Status |
| --- | --- |
| Puzzle | [`data/training/0d3d703e.json`](../data/training/0d3d703e.json) |
| Instruction dataset | [`dataset.0d3d703e.json`](../src/dataset/dataset.0d3d703e.json): the puzzle's 4 original training pairs and its test pair, 15 stripe instances, three per pair; committed in `ebd026c`; partner test respelled with `is not` on 6-Nov-2026; shared orientation comment added on 6-Nov-2026; descriptive `implementation` comments added on 7-Nov-2026; colors written as letters on 8-Nov-2026 |
| Output canvas | `EXACT` |
| Nucleoid enhancements | None requested, so no proposal strategy was selected; the rule uses the `@first` anchors of [NUC 53](../../docs/nuc-0053.md) and [NUC 55](../../docs/nuc-0055.md), the list `includes` that [NUC 61](../../docs/nuc-0061.md) specifies, and the accepted `is not` of [NUC 86](../../docs/nuc-0086.md) |
| Solver changes | None; suggested changes await review, not applied |
| Solver runs | 1 local run on 4-Nov-2026, exit 0; none of the current version |
| Updated | 8-Nov-2026 |

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

### Is comparisons

On 6-Nov-2026 the user asked `/arc-purposal` to apply
[NUC 86](../../docs/nuc-0086.md) to the existing datasets where it fits. The
second `find` now takes the partner with `is not`, NUC 86's spelling of `!=`:

```nuc
$Obj.color = [[1, 5], [2, 6], [3, 4], [8, 9]].find(pair => pair.includes($Obj.color@first)).find(partner => partner is not $Obj.color@first)
```

`is not` compares exactly as `!=` does, with the same loose equality and
precedence. The callback is a call argument, one of the enclosed positions
where NUC 86 reads `is` as an operator, and its right operand,
`$Obj.color@first`, is an anchored property rather than a bare word, so it
reads as it did with `!=`. Only that comparison changed. The class, the pair
list, the grids, masks, positions, instance names, implementations, and
snapshots are as before, and the world model keeps its level, so the gestalt
overview above still holds. Replaying the rule now needs a runtime at
`7096171`, the NUC 86 commit, or later. No NUC was requested, so no proposal
strategy was selected and strategy learning did not apply.

The arc-dataset worker respelled the comparison in an external staging
directory and regenerated all 30 input and output snapshots with the accepted
runtime at `34d0b19`, which includes NUC 86; each is byte-identical to the
previous version's. The caller's own offline check passed all 44 of its checks
on the staged file and again after integration: the file equals the previous
one with only `!=` respelled, its pairs are unchanged, no `==` or `!=` is
left in its declarations, every snapshot replays on that runtime, and the
instance names stay unique across all datasets. `document()` loads it with
the other 28 datasets. At the user's choice, no solver run has measured this
version.

### Orientation convention

On 6-Nov-2026 the user asked `/arc-purposal` to make `orientation` consistent
across all 29 instruction datasets, with a reference of eight directed 3-cell
lines, each of which points toward its head. Every state now takes the
direction its own shape points, by the convention in the
[README](../README.md#3-extract-instances): a line toward its head or along
its axis, a corner or chevron toward its vertex, and anything else 0, while a
turn adds its angle. Every dataset's first declaration also carries one shared
comment line, directly above its class, that states the convention for the
model:

```nuc
# Orientation is the direction an object points, in degrees clockwise from north (0 up, 90 right, 180 down, 270 left)
```

This dataset's labels already followed the convention, as the labeller
confirmed: each vertical stripe is a line without a head along the vertical
axis, 0 in both states. Only the comment line changed, so the class, the
rules, the grids, masks, positions, instance names, implementations, and
snapshots are as before, and the gestalt overview above still holds.

The arc-dataset worker made the change in an external staging directory with
the accepted runtime at `d3413e6` (last runtime change `7096171`), and every
snapshot reruns byte for byte on it. The caller's own offline check passed on
the staged file and again after integration: the pairs, grids, masks,
positions, and instance names are as before, every state's orientation matches
the convention's labeller, every snapshot replays on that runtime, and
`document()` loads the dataset with the other 28. No NUC was requested, so no
proposal strategy was selected and strategy learning did not apply. At the
user's choice, no solver run has measured this version.

### Letter colors

On 8-Nov-2026, at the user's request, every instruction dataset started
writing its colors as letters instead of numbers. `0` stays `0`, and every
other color is a letter of `ABCDEFGHI`, ranked by how often it occurs across
all of the dataset's input and output matrices, training and test pairs
together: `A` for the most used, then `B`, and so on, a tie going to the color
read first, as `Matrix.silhouette` ranks one matrix. This dataset writes its
colors as letters ranked by count: 3 as `A`, 2 as `B`, 4 as `C`, 6 as `D`, 8
as `E`, 9 as `F`, 1 as `G`, and 5 as `H`. Mapping the letters back reproduces
the puzzle's pairs cell for cell.

The pair grids, both instance masks, the implementations, and the snapshots
use those letters, and the class and rules name the same colors by letter.
Only the color literals changed, and the changed declarations now read:

```nuc
class Obj(x: int, y: int, width: int, height: int, orientation: int, color: str):
$Obj.color = [['G', 'H'], ['B', 'D'], ['A', 'C'], ['E', 'F']].find(pair => pair.includes($Obj.color@first)).find(partner => partner is not $Obj.color@first)
```

Every snapshot was regenerated with the accepted runtime at `adc282a` and
equals its earlier snapshot with its colors replaced by their letters, so the
rules behave as before; instance names, positions, sizes, orientations, and
tags are unchanged. `Matrix.silhouette` now also takes letters, so the
orientation call's few-shot crops read exactly as before. No solver run has
measured this version, so its difference is unknown, not zero.

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

The run used the first version of the dataset. No solver run has measured the
version with standard entries or the version with `is not` of 6-Nov-2026,
which the user chose to verify offline, so their difference is unknown, not
zero.

None of these runs measured the orientation version of 6-Nov-2026: the user
chose offline verification for the batch, so its difference is unknown, not
zero.

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
  without such comments both follow them, so nothing needs to change. On
  7-Nov-2026 the user also allowed them in `implementation`, and each
  implementation in this dataset now opens with comment lines that describe
  its object.
- **Workflow timeout.** Resolved on 6-Nov-2026: at the user's request, the
  arc-dataset profile now asks for the batch budget the workflow needs,
  `ceil(listed puzzles / ARC_CONCURRENCY) * 30 + 20` minutes, within GitHub
  Actions' 360-minute job limit. The workflow's 150-minute timeout covers its
  20 listed puzzles.
- **Stale item in another record.** [ARC-NUC-27a28665](ARC-NUC-27a28665.md)
  still lists the stale README paragraph about `dataset.0d3d703e.json` as
  open; `ebd026c` replaced that paragraph, so the item can be closed there.
- **Committed.** `a526fec` committed this record with the dataset.
- **The `is not` version is unmeasured.** No solver run, local or in the
  **ARC Benchmark** workflow, has used the version of 6-Nov-2026. Its
  snapshots are unchanged, but every run's few-shot context now shows `is`
  and `is not` in conditions and callbacks. NUC 86 reads them as operators
  only in enclosed positions, so a model that copies `is` straight into an
  assignment value, such as `$Obj.flag = $Obj.color is 5`, gets `is` read as
  a name and a `ReferenceError`. A dispatched **ARC Benchmark** run would
  measure the effect.
- **Orientation version unmeasured.** No solver run has measured the
  6-Nov-2026 orientation version, so whether consistent orientation labels
  change this puzzle's result is unknown.
- **Letter version unmeasured.** No solver run has measured the 8-Nov-2026
  letter version. The solver still sends this puzzle's own grids as numbers,
  while every few-shot dataset now writes its colors as letters.

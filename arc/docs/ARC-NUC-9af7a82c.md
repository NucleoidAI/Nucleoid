# ARC-NUC-9af7a82c -- Region Size Histogram

Progress record for ARC training puzzle `9af7a82c`: its instruction dataset,
the solver and Nucleoid changes it needed, its runs, and what is still open.

| Item | Status |
| --- | --- |
| Puzzle | [`data/training/9af7a82c.json`](../data/training/9af7a82c.json) |
| Instruction dataset | [`dataset.9af7a82c.json`](../src/dataset/dataset.9af7a82c.json): the puzzle's 4 original training pairs and its test pair, 19 region instances; full-canvas zero-padded masks; orientation labelled by shape and the shared orientation comment added on 6-Nov-2026; descriptive `implementation` comments added on 7-Nov-2026; colors written as letters on 8-Nov-2026; respelled on 8-Nov-2026 with the `is` assignments of NUC 93 and bare words, every snapshot unchanged; colors written as numbers again on 8-Nov-2026 |
| Output canvas | `FIT` |
| Nucleoid enhancements | None recorded |
| Solver changes | None specific to this puzzle; since 8-Nov-2026 every respelled dataset needs `is` assignments, which `src/lib/nucleoid.ts` enabled with `--is-assignment` until `d945213` made them the default |
| Solver runs | 5 GitHub Actions runs, 1-2 Nov 2026: differences of 10 and 5, then three exact matches; none of the current version; none of the 8-Nov-2026 `is` version |
| Updated | 8-Nov-2026 |

## Puzzle

Each training input is a small, fully colored canvas (3 by 3, 3 by 4, or 4 by
3) split into one region per color. The output is a histogram: the region with
the r-th largest cell count becomes a vertical bar of its color in column r,
starting at the top row and as tall as the region's cell count. Outputs can be
taller than their inputs, such as 5 by 3 from 3 by 3. Every bar starts at the
top and the tallest spans every row, so each output fills its nonzero bounding
box and the dataset is `FIT`; see
[ARC-NUC-28bf18c6](ARC-NUC-28bf18c6.md#decision-output_canvas-trims-the-composed-matrix).

## Gestalt overview

- **Whole grid.** A small, fully colored canvas, 3 by 3, 3 by 4, or 4 by 3;
  the output is a histogram on the background, as tall as the largest region
  and as wide as the number of regions.
- **Figures.** The single-color regions, three or four per grid; each color
  forms one region.
- **Relations.** The regions rank by size, their cell counts.
- **Change.** The regions turn into a histogram: each becomes a vertical bar
  of its color, from the top row and as tall as its cell count, placed from
  left to right, largest first.
- **World model.** One object per region, the figures seen first; the rank,
  the relation between them, is stated over the whole population.

## Dataset

The single dataset has the four original training pairs in source order:
3 by 3 to 5 by 3, 3 by 4 to 6 by 4, 4 by 3 to 5 by 4, and 4 by 3 to
6 by 3, with 3, 4, 4, and 3 regions. The original test pair follows them at
the same level: 4 by 4 to 6 by 5, with 5 regions. All source cells, colors,
positions, and canvas dimensions are preserved.
Each region is an `Obj(x, y, orientation, color, cell_count)`, where
`cell_count` is the number of cells in its full-size, zero-padded input mask.
Unconditional class rules derive the bar:

```nuc
$Obj.tag = 'l6953'
$Obj.bar_height = $Obj.cell_count
$Obj.x = Obj.filter(other => other.bar_height > $Obj.bar_height).length
$Obj.y = 0
```

`x` is the bar's rank: the number of other bars taller than it. At the user's
request, the rank compares `bar_height`, the output bar's own property, instead
of `cell_count`, the input measurement it is derived from, so the rule states
the histogram's order in the histogram's own terms. Because `bar_height`
equals `cell_count`, both rules give the same ranks and every snapshot is
unchanged. On 2-Nov-2026 the accepted runtime at `feef5f8` replayed the
output snapshots of all five pairs offline with each rule: with the instances
created in source, reversed, and rotated order, and with the rank rule declared
before `bar_height`, every `x`, `y`, `bar_height`, and `tag` matched its
recorded value. Ranking by a derived property therefore needs no NUC. This
offline check is not a solver run.

Each `input_instance_matrix` matches its pair's complete `input_matrix`
dimensions, and each `output_instance_matrix` matches its pair's complete
`output_matrix` dimensions. Both masks retain the object's cells at their
canvas coordinates and contain `0` everywhere else; neither mask is stored as
a crop. The output bar occupies `bar_height` cells in column `output_x`,
starting at row `output_y`. These origins equal `output_value.x` and
`output_value.y` and describe positions already present within the full mask.
Merging the full output masks directly reconstructs each original
`output_matrix`. Native input snapshots execute the constructor and initial
class declaration; native output snapshots execute all declarations and
implementations of the same pair together, followed by `Obj[:]`. Instances
are listed largest first, and each recorded rank sees the complete pair's
population. The original-training replacement changes no TypeScript and
includes no live solver run.

### Gestalt review

On 5-Nov-2026 the dataset was checked against the gestalt requirement, which
`/arc-purposal` and the arc-dataset profile added on 4-Nov-2026, after this
dataset was written: start from the big picture, the figures each grid shows,
and go down to single cells only when that cannot express what the training
pairs show. The dataset meets it. The figures each input shows are its
single-color regions, each region is one object, and the relation between
them, their order by size, places the bars. Nothing needs to change.

### Standard entries

On 5-Nov-2026, at the user's request, every ARC dataset moved to the standard
object entries `x`, `y`, `width`, `height`, and `orientation`. `width` and
`height` are the column and row counts of the object's input bounding box. An
orientation change alone leaves them unchanged, and a rule changes them only
when the shape itself changes size, reading the input with `@first`. Every
implementation now passes the input box's `width` and `height` after `y`, and
the class and rules read:

```nuc
class Obj(x: int, y: int, width: int, height: int, orientation: int, color: int, cell_count: int):
$Obj.tag = 'l6953'
$Obj.width = 1
$Obj.height = $Obj.cell_count
$Obj.x = Obj.filter(other => other.height > $Obj.height).length
$Obj.y = 0
```

`bar_height` became the standard `height`: `$Obj.height = $Obj.cell_count` and
`$Obj.width = 1` size each bar, and the ranking now reads `Obj.filter(other =>
other.height > $Obj.height).length`. The migration worker regenerated every
snapshot with the accepted runtime at `491512d`, and the caller replayed every
pair offline against the unchanged grids, masks, positions and instance names.
No solver run has measured this version yet.

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

Ten of the nineteen input regions changed: seven corners point toward their
vertices, at 45, 135, 225, or 315, and three horizontal bars are 90. Every
output is a vertical bar or a single cell, 0, so one new unconditional rule,
`$Obj.orientation = 0`, stands every bar upright, whatever its region's
orientation, and the output states are as before. The rules now read:

```nuc
class Obj(x: int, y: int, width: int, height: int, orientation: int, color: int, cell_count: int):
$Obj.tag = 'l6953'
$Obj.width = 1
$Obj.orientation = 0
$Obj.height = $Obj.cell_count
$Obj.x = Obj.filter(other => other.height > $Obj.height).length
$Obj.y = 0
```

The grids, masks, positions, instance names, tags, and every other property
are as before; only the orientation values and the new rule changed, so the
world model keeps its level.

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
colors as letters ranked by count: 1 as `A`, 2 as `B`, 8 as `C`, 3 as `D`, and
4 as `E`. Mapping the letters back reproduces the puzzle's pairs cell for
cell.

The pair grids, both instance masks, the implementations' `color` arguments,
and the snapshots use those letters, and `color` is now a `str` parameter. The
rules name no color, so they are unchanged.

Every snapshot was regenerated with the accepted runtime at `adc282a` and
equals its earlier snapshot with its colors replaced by their letters, so the
rules behave as before; instance names, positions, sizes, orientations, and
tags are unchanged. `Matrix.silhouette` now also takes letters, so the
orientation call's few-shot crops read exactly as before. No solver run has
measured this version, so its difference is unknown, not zero.

### Is assignments

On 8-Nov-2026 the user asked to apply [NUC 93](../../docs/nuc-0093.md) and
[NUC 86](../../docs/nuc-0086.md) to every dataset in `src/dataset`. Every
assignment in the declarations and implementations now uses `is`, the opt-in
assignment of NUC 93, and a quoted word that was a whole value, or the whole
right operand of an `is` comparison, lost its quotes:

```nuc
class Obj(x: int, y: int, width: int, height: int, orientation: int, color: str, cell_count: int):
    this.x is x
    this.y is y
    this.width is width
    this.height is height
    this.orientation is orientation
    this.tag is l9100
    this.color is color
    this.cell_count is cell_count

$Obj.tag is l6953

$Obj.width is 1

$Obj.orientation is 0

$Obj.height is $Obj.cell_count

$Obj.x is Obj.filter(other => other.height > $Obj.height).length

$Obj.y is 0
```

`target is value` builds exactly the syntax tree of `target = value`. A bare
word reads the name of its spelling when one exists and otherwise stores its
own text: in the constructor, `this.x is x` reads the parameter `x`, and no
program of this dataset defines any of `l9100` and `l6953`, so those words
read as their own text. Implementations construct their instances with `is` as
well, as in
`obj5fb8dee4c604 is Obj(x=0, y=0, width=3, height=3, orientation=135, color='A', cell_count=5)`,
and keep their quoted keyword arguments, which are not assignments. Only the
spelling changed: the class, the rules' structure, the grids, masks,
positions, instance names, and snapshots are as before, and the world model
keeps its level, so the gestalt overview above still holds.

The rules now need a runtime at `ad45ba4`, the NUC 93 commit, or later, run
with `--is-assignment`, which `src/lib/nucleoid.ts` now passes to every
program; without the flag, `obj5fb8dee4c604 is Obj(...)` reads as three
juxtaposed statements and raises `ReferenceError`. Offline, with the accepted
runtime at `4ac649b` (last runtime change `ad45ba4`), the flag reproduced all
19 input and 19 output snapshots of the 5 pairs, both in a direct replay and
through the solver's own `nucleoid.run`, and the previous version reproduced
them with and without the flag. No NUC was requested, so no proposal strategy
was selected and strategy learning did not apply. No solver run has used this
version.

### Number colors

On 8-Nov-2026, at the user's request, the letter colors were reverted, and the
dataset writes every color as its number again, as the puzzle's pairs hold it,
turning `A` back into 1, `B` into 2, `C` into 8, `D` into 3, and `E` into 4.
The pair grids, both instance masks, the implementations' `color` arguments,
and the snapshots hold numbers, and `color` is an `int` parameter again. The
changed declarations now read:

```nuc
class Obj(x: int, y: int, width: int, height: int, orientation: int, color: int, cell_count: int):
```

The `is` assignments and bare words of the previous section are unchanged.
Offline, with the accepted runtime at `1a41ca7` (last runtime change
`f463d94`), all 19 input and 19 output snapshots of the 5 pairs replay, each
equal to its letter snapshot with every letter read as its number. The pair
grids equal the puzzle's pairs cell for cell, and the grids, masks, and
snapshots are again those of the version before the letters.
`Matrix.silhouette` again takes only the colors 0 to 9, and the orientation
calls' few-shot crops read exactly as they did with letters, because the
counts that rank them are unchanged. No NUC was requested, so no proposal
strategy was selected and strategy learning did not apply. No solver run has
measured this version, so its difference is unknown, not zero.

## Runs

| Date | Run | Commit | Exit status | Difference | Seconds |
| --- | --- | --- | --- | --- | --- |
| 1-Nov-2026 12:00 UTC | [41304414238](https://github.com/NucleoidAI/Nucleoid/actions/runs/41304414238) | `3cca0aa` | 10 | 10 | 178 |
| 1-Nov-2026 14:03 UTC | [41314257591](https://github.com/NucleoidAI/Nucleoid/actions/runs/41314257591) | `3f734a8` | 5 | 5 | 279 |
| 1-Nov-2026 20:20 UTC | [41346341604](https://github.com/NucleoidAI/Nucleoid/actions/runs/41346341604) | `c2fff09` | 0 | 0 | 276 |
| 2-Nov-2026 05:26 UTC | [41394006982](https://github.com/NucleoidAI/Nucleoid/actions/runs/41394006982) | `b90a525` | 0 | 0 | 178 |
| 2-Nov-2026 06:45 UTC | [41401913542](https://github.com/NucleoidAI/Nucleoid/actions/runs/41401913542) | `ed49ce3` | 0 | 0 | 304 |

Each run is the **ARC puzzle report** workflow's
`bun --no-env-file run start 9af7a82c` on `ubuntu-latest`, with the commit's
debug runtime and a 30-minute limit. The exit status is the difference score;
seconds are the workflow's wall time for the puzzle, from each run's
`arc-puzzle-report` artifact. Both non-zero exits were mismatches with those
differences, not failed runs. No local `bun start 9af7a82c` run is recorded.
Every recorded run used the earlier `cell_count` rank rule. The `bar_height`
rank, committed in `0bf1b0f` after these runs, has no run yet, so its
difference is unknown, not zero.

None of these runs measured the orientation version of 6-Nov-2026: the user
chose offline verification for the batch, so its difference is unknown, not
zero.

## Open items

- **The rank needs every region in one program.** Isolated native execution
  cannot reproduce population-dependent ranks. On 29-Oct-2026, with the
  earlier examples, per-object runs turned recorded ranks 1, 2, and 0 into
  0, 0, and 0. The current annotations use complete-pair output execution;
  this offline evidence is not a live solver result.
- **Runs from `c2fff09` on saw the answer.** Resolved on 6-Nov-2026: the user
  decided this is intended, because the datasets are being prepared as
  few-shot examples and runs at this stage are not yet held-out tests. The ARC
  instructions now say so, and the suggested change to keep the running
  puzzle's test pairs out of `document(puzzle_id)` is withdrawn. Since that
  commit, `src/dataset/dataset.ts` sends every JSON dataset as few-shot
  context, and this puzzle's dataset includes its annotated test pair with the
  expected output. The runs at `3cca0aa` and `3f734a8`, which predate it,
  scored differences of 10 and 5; the three runs since matched exactly, which
  does not show that the rule was inferred from the training pairs.
- **Orientation version unmeasured.** No solver run has measured the
  6-Nov-2026 orientation version, so whether consistent orientation labels
  change this puzzle's result is unknown.
- **Letter version withdrawn.** No solver run measured the 8-Nov-2026 letter
  version before the user reverted it the same day. The dataset writes its
  colors as numbers again, as the solver sends this puzzle's own grids, and
  no solver run has measured that version either, so its difference is
  unknown.
- **The `is` assignment version is unmeasured.** No solver run has used the
  8-Nov-2026 version with `is` assignments and bare words, so its effect on
  this puzzle's result is unknown. Every run's few-shot context now shows bare
  words beside quoted strings, and a word reads as its text only as a whole
  value: a model that writes one inside a larger value, such as
  `$Obj.fill_color is A if $Obj.symmetric else C`, gets a `ReferenceError`.

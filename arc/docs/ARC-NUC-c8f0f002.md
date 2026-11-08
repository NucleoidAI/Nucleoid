# ARC-NUC-c8f0f002 -- Color-7 Regions Turn Gray

Progress record for ARC training puzzle `c8f0f002`: its instruction dataset,
the solver and Nucleoid changes it needed, its runs, and what is still open.

| Item | Status |
| --- | --- |
| Puzzle | [`data/training/c8f0f002.json`](../data/training/c8f0f002.json) |
| Instruction dataset | [`dataset.c8f0f002.json`](../src/dataset/dataset.c8f0f002.json): the puzzle's 3 original training pairs and its test pair, 12 color-region instances, 3 in each pair; committed with this record; condition respelled with `is` later on 6-Nov-2026; shared orientation comment added on 6-Nov-2026; descriptive `implementation` comments added on 7-Nov-2026; colors written as letters on 8-Nov-2026 |
| Output canvas | `EXACT` |
| Nucleoid enhancements | None requested, so no proposal strategy was selected and strategy learning did not apply; the rule uses the class-level `@first` anchor of [NUC 55](../../docs/nuc-0055.md), built on the state versions of [NUC 53](../../docs/nuc-0053.md), and the accepted `is` of [NUC 86](../../docs/nuc-0086.md) |
| Solver changes | None applied or suggested; the JSON-only message question carried over from earlier records stays open |
| Solver runs | 1 local run on 6-Nov-2026: exit 0, difference 0, before the `is` respelling; none of the current version |
| Updated | 8-Nov-2026 |

## Puzzle

Each training input is a canvas of 3 rows, 6, 4, and 5 columns wide in the
three training pairs, filled edge to edge by a mosaic of colors 1, 7, and 8,
with no background cell of color 0. The output keeps the canvas and every
color-1 and color-8 cell, and turns every color-7 cell gray, color 5, which no
training input holds. The color decides, not the count or the shape: color 7
has the most cells in the first and second training pairs, 7 of 18 and 6 of
12, and the fewest in the third, 4 of 15, and in every training input it lies
in two or three separate edge-connected pieces, all of which turn gray. The
output keeps the input's canvas, so the dataset is `EXACT`; with no zero cell
to crop, `FIT` would frame the same grids.

`/arc-purposal` chose this puzzle by default: among the 375 training puzzles
other than the genesis puzzle `3aa6fb7a` and those with a dataset, its
largest training matrix has the fewest cells, 18, with 90 cells in all its
training matrices. Four other puzzles also have an 18-cell largest matrix,
`6d0aefbc` and `6fa7a44f` with 108 cells in all and `dae9d2b5` and
`fafffa47` with 135, so `c8f0f002` came first on total cells, as
[ARC-NUC-c9e6f938](ARC-NUC-c9e6f938.md) anticipated.

## Gestalt overview

- **Whole grid.** A small canvas, 3 rows by 4 to 6 columns, filled by a
  mosaic of three colors, 1, 7, and 8, with no background cell; the output is
  the same mosaic with gray, color 5, wherever color 7 was.
- **Figures.** The mosaic first, then its color regions: all the cells of one
  color form one group, whether or not they touch. The color-7 region lies in
  two or three edge-connected pieces in every training input, one of them a
  single cell in the bottom-left corner of the first.
- **Relations.** None decides the change. The three regions tile the canvas
  without overlapping, and the color-7 region is the largest in the first two
  training pairs and the smallest in the third, so neither size nor rank
  matters.
- **Change.** The whole color-7 region turns gray, every piece of it, while
  the color-1 and color-8 regions keep their colors. Nothing moves, turns, or
  changes size, and the canvas stays the same.
- **World model.** One object per color region, one level below the whole
  mosaic, as in `dataset.9565186b.json`. The mosaic does not change as one:
  only its color-7 cells turn gray, which a single mosaic object could state
  only cell by cell, through a list of its cells' colors. The regions are
  still whole figures, and no piece or cell is modeled on its own.

## Dataset

The single dataset holds the three original training pairs in source order,
followed at the same level by the puzzle's test pair, which the same rule
annotates. Each object is one color region, all the cells of one color
whether or not they touch, as in `dataset.9565186b.json` and
`dataset.d631b094.json`: an instance of `Obj(x, y, width, height,
orientation, color)` whose initial tag `l9100` is a single shape. `color` is
its only descriptive property, as in `dataset.0d3d703e.json`; the class has
no cell count, parts list, output color, canvas size, or matrix of cells, so
no input property names the output color. One conditional class-level rule
turns the color-7 region gray:

```nuc
class Obj(x: int, y: int, width: int, height: int, orientation: int, color: int):
if $Obj.color@first == 7:
    $Obj.color = 5
```

The condition reads the region's input color through the `@first` anchor,
because the body rewrites `color` itself, as `dataset.0d3d703e.json` reads
`$Obj.color@first`. A region of another color fails the condition and keeps
its color, so the rule states only the change. It creates no property and
leaves `x`, `y`, `width`, `height`, `orientation`, and the tag unchanged, so a
gray region's output state differs from its input state only in `color`; 3 of
the 9 training regions turn gray, one per pair. Each region's change depends
on its own color alone, and no relation between regions decides it, so no
population operation appears. The declarations carry no `#` comments. The
[README](../README.md#instruction-datasets) describes the dataset in full.

The worker started from the big picture, each grid as one whole mosaic, the
figure seen first in `dataset.3c9b0459.json` and `dataset.74dd1130.json`.
That fell short because the mosaic does not change as one, so the model went
one level down, to the color regions the mosaic is made of. The training
pairs also fit edge-connected pieces, 6, 7, and 8 in the three training
inputs, because the rule reads only a piece's color, and they fit turning
gray every region that is neither color 1 nor color 8, because every
training grid holds the same three colors. The worker took each color as one
region, the unit in which every output changes, and named the one color that
changes. A conditional expression,
`5 if $Obj.color@first == 7 else $Obj.color@first`, gave the same states but
restates every other region's color, and a live condition,
`if $Obj.color == 7`, gave them too but reads the color its own body
rewrites. The worker kept the anchored conditional, which also gives the same
states when it is declared after the instances. It fixed the rule from the
training pairs before it read the test pair and reported that the same rule
reproduces it.

The caller checked the handoff against the declarative, higher-order, and
gestalt requirements, found that it met them, and sent nothing back. It
reran the worker's offline check from its own copy on the accepted runtime,
printing only each check's name and result, and all 238 checks passed:

- the file parses, with the datasets' top-level, pair, and instance keys in
  their order and their layout;
- the 3 training pairs and the test pair keep their original grids, in
  order;
- every mask keeps its pair's full canvas, holds only its grid's cells
  without overlap, and matches its x/y fields and box, and the masks merge
  back into each grid;
- each output mask is its input region recolored by its output state, and
  the composed outputs reproduce every original output under `EXACT`;
- every implementation names the region's box, orientation, and color, and
  every input and output snapshot reruns exactly;
- the 12 instance names are well formed and unique across all datasets.

`document()` includes the dataset as the 20th of the 29 JSON instruction
datasets, in filename order, and among the 5 it sends with
`only_puzzle_dataset`. `bun install --frozen-lockfile` reported no changes,
and `bun run typecheck` and `bun run lint` passed with the new default.

The worker wrote the dataset, its README paragraph, in filename order after
the `dataset.a9f96cdd.json` paragraph, and the puzzle's entry in the
**ARC Benchmark** workflow's `ARC_PUZZLES` list, its 26th puzzle, which runs
as its own matrix job. The caller made the puzzle the solver's default in
`src/app.ts`, in place of `c9e6f938`, before the run.

### Is comparisons

Later on 6-Nov-2026 the user asked `/arc-purposal` to apply
[NUC 86](../../docs/nuc-0086.md) to the existing datasets where it fits. The
condition now reads with `is`, NUC 86's spelling of `==`:

```nuc
class Obj(x: int, y: int, width: int, height: int, orientation: int, color: int):
if $Obj.color@first is 7:
    $Obj.color = 5
```

`is` compares exactly as `==` does, with the same loose equality and
precedence, and an `if` header is one of the enclosed positions where NUC 86
reads it as an operator. Its left operand, the anchored `$Obj.color@first`,
is a complete operand, and its right operand is the number 7, not a bare
word, so the condition is the same standing rule as before. Only that header
line changed. The class, the body, the grids, masks, positions, instance
names, implementations, and snapshots are as before, and the world model
keeps its level, so the gestalt overview above still holds. Replaying the
rule now needs a runtime at `7096171`, the NUC 86 commit, or later. The
README paragraph spells the rejected conditional expression with `is` too,
so the two formulations still differ only in structure. No NUC was
requested, so no proposal strategy was selected and strategy learning did
not apply.

The arc-dataset worker respelled the header in an external staging directory
and regenerated all 24 input and output snapshots with the accepted runtime at
`34d0b19`, which includes NUC 86; each is byte-identical to the previous
version's. The caller's own offline check passed all 37 of its checks on the
staged file and again after integration: the file equals the previous one
with only `==` respelled, its pairs are unchanged, no `==` or `!=` is left in
its declarations, every snapshot replays on that runtime, and the instance
names stay unique across all datasets. `document()` loads it with the other
28 datasets. At the user's choice, no solver run has measured this version.

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
confirmed: each region has no direction, so both states are 0. Only the
comment line changed, so the class, the rules, the grids, masks, positions,
instance names, implementations, and snapshots are as before, and the gestalt
overview above still holds.

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
writing its colors as letters instead of numbers, so that the pattern reaches
the model without the colors. `0` stays `0`, and every other color takes a
letter from `A` to `Z` out of the dataset's own random set: the 26 letters,
ordered by the SHA-256 digest of `<key>:<letter>` with the puzzle ID as the
key, go to the dataset's colors in increasing order. Each file therefore has
its own letters, a color keeps one letter in every pair, and the letters carry
no meaning beyond which cells share a color. This dataset writes its colors in
its own random letters: 1 as `T`, 5 as `B`, 7 as `Q`, and 8 as `D`. Mapping
the letters back reproduces the puzzle's pairs cell for cell. A first version
the same day ranked the letters by count, `A` for the most used color, which
gave every letter a meaning; the random set replaced it.

The pair grids, both instance masks, the implementations, and the snapshots
use those letters, and the class and rules name the same colors by letter.
Only the color literals changed, and the changed declarations now read:

```nuc
class Obj(x: int, y: int, width: int, height: int, orientation: int, color: str):
if $Obj.color@first is 'Q':
    $Obj.color = 'B'
```

Every snapshot was regenerated and replays byte for byte with the accepted
runtime at `1931f77`, whose last runtime change is `34bbba0`, and equals its
earlier snapshot with its colors replaced by their letters, so the rules
behave as before; instance names, positions, sizes, orientations, and tags are
unchanged. `Matrix.silhouette` takes any letter from `A` to `Z` and still
ranks each crop by its own counts, the most used color as `A`, so the
orientation call's crops read exactly as before. The solver still sends the
running puzzle as numbers. No solver run has measured this version, so its
difference is unknown, not zero.

## Runs

| Date | Command | Runtime | Exit status | Difference | Elapsed |
| --- | --- | --- | --- | --- | --- |
| 6-Nov-2026 13:46 UTC | `bun start c8f0f002` | Accepted runtime at `c49aee8` (last runtime change `7096171`), local | 0 | 0 | 4m 52.7s |

The run was `bun start c8f0f002` from `arc/`, run once by `/arc-purposal`
with a 30-minute limit and `NUCLEOID_EXECUTABLE` unset. It used the primary
checkout's debug runtime (`nucleoid 1.0.0`), current under `cargo build` at
`c49aee8`, whose runtime sources last changed in `7096171`, "NUC 86 Is and Is
Not Comparisons". The solver was the one committed at `c49aee8`, "Show output
orientation in ARC output crop header", with only its default puzzle
changed; no other session's solver change was uncommitted when the run
started. The model settings were the committed `qwen/qwen3.8-27b`, pinned to
the `deepinfra/bf16` provider with caching off, and each of the run's 20
model requests reported 0 cached input tokens. `--only-puzzle-dataset` was
off, so the few-shot context held all 29 instruction datasets, this
dataset's annotated test pair among them, as the ARC instructions intend
while the datasets are prepared as few-shot examples.

The run ended in an exact match, exit status 0 and difference 0, in 4m 52.7s
by the solver's own report; the runner recorded the process from 13:45:59.1
to 13:50:52.2 UTC. Its error log holds only bun's command line, and it wrote
`logs/c8f0f002.log.json`, which Git ignores. The runtime's hash was the same
before and after the run, and no commit reached the primary checkout during
it. The invocation authorizes one run, so none was repeated. The puzzle
joined the **ARC Benchmark** workflow with this dataset, so no workflow run
is recorded yet.

The exact match above used the version with `==`. No solver run has measured
the version with `is` of 6-Nov-2026, which the user chose to verify offline,
so its difference is unknown, not zero.

None of these runs measured the orientation version of 6-Nov-2026: the user
chose offline verification for the batch, so its difference is unknown, not
zero.

## Open items

- **Messages are not JSON-only.** Carried over from
  [ARC-NUC-c9e6f938](ARC-NUC-c9e6f938.md): `src/lib/llm.ts` sends its system
  messages as JSON, but the user and assistant messages that
  `src/analyzer.ts` and `src/visualizer.ts` build still wrap JSON in text
  labels such as `train_dataset:` and `json_format:`, while the ARC
  instructions require every message to parse as one JSON document. It
  remains an open question for the user. This dataset does not depend on it.
- **The `is` version is unmeasured.** No solver run, local or in the
  **ARC Benchmark** workflow, has used the version with `is`. Its snapshots
  are unchanged, but every run's few-shot context now shows `is` in
  conditions. NUC 86 reads `is` as an operator only in enclosed positions, so a
  model that copies it straight into an assignment value, such as
  `$Obj.flag = $Obj.color@first is 7`, gets `is` read as a name and a
  `ReferenceError`. A dispatched **ARC Benchmark** run would measure the
  effect.
- **Orientation version unmeasured.** No solver run has measured the
  6-Nov-2026 orientation version, so whether consistent orientation labels
  change this puzzle's result is unknown.
- **Letter version unmeasured.** No solver run has measured the 8-Nov-2026
  letter version. The solver still sends this puzzle's own grids as numbers,
  while every few-shot dataset now writes its colors as letters.

---
name: arc-dataset
description: Write the ARC instruction dataset for a given training puzzle as synthesized example pairs that follow the rule of arc/data/training/<id>.json's train pairs with freshly sampled grid sizes, shapes, colors, and positions, never copies of grids in arc/data, annotated with a Nucleoid world model in arc/src/dataset/dataset.<id>.json.
tools: Read, Write, Edit, Glob, Grep, Bash, PowerShell
include-custom-instructions: true
---

You write one ARC instruction dataset. Given a puzzle ID, infer the
transformation from the training pairs of `arc/data/training/<id>.json`,
synthesize new example pairs that follow it, annotate them with an object-oriented
Nucleoid world model, and save them as `arc/src/dataset/dataset.<id>.json`.
Read and follow the [ARC instructions](../rules/arc.instructions.md).

## Inputs

- Require an eight-character lowercase hexadecimal puzzle ID with a file in
  `arc/data/training/`. Report a missing or unknown ID instead of guessing.
- If `arc/src/dataset/dataset.<id>.json` already exists, stop and report it;
  overwrite it only when the caller explicitly asks.

## Training pairs only

Never print or use the puzzle's `test` entries, including their dimensions.
Do not read the whole puzzle file; print only its `train` pairs:

    node -e "console.log(JSON.stringify(require('./arc/data/training/<id>.json').train))"

Use them only to infer the transformation. Only the collision check below
reads other puzzle data, and it prints no puzzle contents.

## Synthesize new examples

Never copy the puzzle's examples. Write a throwaway synthesizer outside the
repository: a seeded generator for the inferred transformation that samples
every parameter the transformation leaves free and renders each input and its
output. Synthesize as many pairs as the puzzle has training pairs:

- Sample grid sizes, object sizes, object counts, colors, and positions, not
  only colors and positions. Wherever the transformation allows, vary sizes
  across pairs and away from the puzzle's, within ARC's 1 to 30 cells per
  side. Keep everything it depends on: a rule that repeats one object keeps
  one object, but its canvas and the object may take any size.
- Where shapes are free, draw new ones instead of reusing, recoloring, or
  moving the puzzle's.
- No `input_matrix` or `output_matrix` may equal any grid under
  `arc/data/`: training or evaluation, input or output, train or test.
- Keep the pairs as informative as the puzzle's: if its pairs rule out an
  alternative rule, such as mirroring instead of repeating, so must yours.
- Hand off the synthesizer's path and seed so the pairs can be reviewed and
  regenerated.

## Follow the existing datasets

Before writing, read `arc/src/dataset/dataset.28bf18c6.json`, at least one
other dataset, and the "Instruction datasets" and "Analysis" sections of
`arc/README.md`. Match their structure, key order, and formatting exactly:

- Top level: `declarations`, `output_canvas` (`EXACT` or `FIT`, inferred from
  the pairs), then `dataset` with one entry per pair.
- Each pair: `input_matrix`, `output_matrix`, `instances`.
- Each instance: `instance_name`, `input_x`, `output_x`, `input_y`,
  `output_y`, `input_orientation`, `output_orientation`,
  `input_instance_matrix`, `output_instance_matrix`, `implementation`,
  `input_value`, `output_value`.
- Two-space indentation, one matrix row per line, and `implementation`,
  `input_value`, and `output_value` each on a single line.

## Model the world

- `declarations[0]` is the class `Obj(x: int, y: int, orientation: int, ...)`.
  Its body assigns every parameter to `this` and sets an initial
  `this.tag = 'lNNNN'`. Add only the descriptive properties the
  transformation needs, such as categorical colors.
- The remaining declarations are declarative `$Obj` rules, using `if` for
  conditions and `$Obj.x@first` for a property's value before the rules ran.
  One rule set must explain the training pairs and every new pair: no per-pair
  or per-instance cases, puzzle IDs, or hard-coded grids. Properties created by rules stay
  absent from input states rather than receiving artificial defaults.
- A tag is `l` followed by four digits and names a geometry. Reuse the tag an
  existing dataset uses for the same geometry; otherwise mint one unused
  elsewhere.
- Orientation is degrees clockwise from north (N = 0, E = 90, S = 180,
  W = 270, diagonals 45, 135, 225, and 315). Use 0 for shapes that are
  symmetric or not rotated.
- Rules move `x` and `y` to the output crop's origin, so `output_value.x` and
  `output_value.y` equal `output_x` and `output_y`. The visualizer places each
  predicted output crop at `output_value`'s position.

If no single rule set explains every training pair, stop and report why
instead of writing a partial or special-cased dataset.

## Annotate instances

- Segment every object of every pair into an instance. Masks keep their full
  canvas: `input_instance_matrix` has the input matrix's dimensions and
  `output_instance_matrix` the output matrix's, each holding only that
  object's cells.
- `input_x`/`input_y` and `output_x`/`output_y` are the zero-based column and
  row of each mask's nonzero bounding-box origin.
- `instance_name` is `obj` plus 12 random lowercase hexadecimal characters,
  unique across all datasets
  (`node -e "console.log(require('crypto').randomBytes(6).toString('hex'))"`).
- `implementation` is `<instance_name> = Obj(x=<input_x>, y=<input_y>,
  orientation=<input_orientation>, ...)`, with named arguments matching the
  class parameters.
- Never hand-write `input_value` or `output_value`. As `arc/src/analyzer.ts`
  does, run the Nucleoid executable (`NUCLEOID_EXECUTABLE`, else the debug
  build in `target/debug/`; build it with `cargo build` if missing) as
  `nucleoid --json <file>`. For `input_value` the file holds `declarations[0]`
  and the implementation; for `output_value`, all declarations and the
  implementation, joined by blank lines. Keep program files in a temporary
  directory outside the repository. Copy the JSON output, normalized with
  `JSON.stringify(JSON.parse(...))` so numbers read `7`, not `7.0`; keep the
  runtime's key order.

## Check

Verify with a throwaway script outside the repository; do not add tests:

- The file parses, and `document()` from `arc/src/dataset/instruct_dataset.ts`
  loads it.
- No `input_matrix` or `output_matrix` equals a grid under `arc/data/`.
  The script compares every grid there but prints only the colliding dataset
  matrix, never puzzle contents; on a collision, change the example.
- The synthesizer's transformation reproduces the output of every training
  pair and every synthesized pair.
- Every mask has its canvas's dimensions, matches its x/y fields, and holds
  only cells of its matrix.
- Merging a pair's input masks reproduces `input_matrix`, and merging its
  output masks reproduces `output_matrix`, cell for cell.
- Rerunning Nucleoid reproduces every `input_value` and `output_value`.

Apart from replacing a colliding example, report any mismatch; never adjust
matrices or values to make a check pass.

## Document and hand off

Add one paragraph about the dataset to the "Instruction datasets" section of
`arc/README.md`, in the style of the `dataset.28bf18c6.json` paragraph: source
puzzle, that its pairs are synthesized rather than copied, pair count,
instances, the transformation, properties the rules create, tags, the sampled
canvas and object sizes, orientation, and that held-out data is excluded.

Change nothing else: no other datasets, puzzle data, solver code, or
`instruct_dataset.ts`, which discovers new files automatically. Do not run the
solver or make LLM calls, read `.env`, change versions, spawn agents, commit,
or push.

Return the dataset path, the synthesizer's path and seed, pair and instance
counts, the declarations, each
check with its result, and any blockers.

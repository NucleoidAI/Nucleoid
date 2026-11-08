---
name: arc-dataset
description: Annotate one ARC puzzle's original train and test pairs in a single instruction dataset, or return the Nucleoid enhancement request it requires.
argument-hint: <puzzle-id> [caller inputs] [--hint <text>]
context: fork
agent: arc-dataset
---

Write the ARC instruction dataset for puzzle $ARGUMENTS. The puzzle ID comes
first; inputs after it, such as a candidate Nucleoid executable or a final
`--hint <text>`, are defined in the
[arc-dataset agent profile](../../agents/arc-dataset.agent.md). Follow the
profile; if it is not already your system prompt, delegate this request to the
`arc-dataset` agent.

## One original-pairs dataset

Create exactly one canonical `arc\src\dataset\dataset.<puzzle-id>.json`.
Annotate every original `train` pair from
`arc\data\training\<puzzle-id>.json`, in its original order, then every
original `test` pair from the same file, in its original order. Test pairs are
additional entries at the same level as the training pairs: the same `dataset`
array and entry structure, with no separate section or marker. Copy the input
and output grids' exact pattern, positions, zero padding, and canvas
dimensions, but write each color as a letter: `0` stays `0`, and the other
colors take the letters `ABCDEFGHI` by how often they occur across all of the
dataset's input and output matrices, `A` for the most used, never as their
numbers. The masks and the world model use the same letters, as the profile's
[letter rule](../../agents/arc-dataset.agent.md#write-colors-as-letters)
defines. Do not synthesize, resample, recolor beyond those letters, or
duplicate examples into numbered variants.

Infer one transformation, Nucleoid declaration set, and output-canvas mode
from the training pairs, then apply them unchanged to every pair, train and
test. If they do not reproduce a test pair, stop and report it instead of
revising the rules from that pair, special-casing it, or leaving it out.
Object identifiers remain unique across datasets. Generation produces one file
per puzzle; the loader aggregates every JSON instruction dataset in
`arc\src\dataset` in filename order for all runs.

The genesis puzzle `3aa6fb7a` is an exception. Its four shared genesis
datasets, `dataset.core.json`, `dataset.extension.json`, `dataset.curve.json`,
and `dataset.line.json`, hold synthesized examples instead of original pairs;
preserve them, and do not add the puzzle's test pair to them. Do not
regenerate or change genesis while updating other puzzles. Do not delete or
overwrite existing datasets without explicit authorization; a padding-only
audit does not replace their examples. When explicitly asked to add test pairs
to an existing dataset, append them after its training pairs with its existing
declarations and output-canvas mode, leaving every existing pair unchanged.

## Full-canvas instance masks

Every instance must store rectangular, zero-padded masks, never cropped masks:

- `input_instance_matrix` must have exactly the same row and column counts as
  its pair's `input_matrix`.
- `output_instance_matrix` must have exactly the same row and column counts as
  its pair's `output_matrix`, even when the input and output canvases differ.
- Keep each object's cells at their canvas coordinates and fill every other
  cell with `0`. The x/y metadata records the nonzero bounding-box origin within
  the full mask; it is not an offset to apply to that mask again.

Check both mask shapes, source-cell consistency, and full-canvas recomposition
before handoff. In an explicitly requested existing-dataset audit, inspect
every requested dataset and repair unambiguous canvas/padding defects using
the recorded world positions. Preserve pair grids, object identities,
declarations, implementations, and runtime snapshots; report ambiguous
placement or other mismatches instead of changing answers or inventing cells.

Whenever a new dataset is successfully introduced, the agent also registers
its puzzle ID exactly once in the workflow-level `env.ARC_PUZZLES` in
`.github/workflows/arc.yml` for the GitHub Actions pass/fail table. Preserve
existing entries; each listed puzzle runs as its own job with its own
thirty-minute limit, so the list needs no timeout budget, as the profile
specifies. Updating the list does not authorize a solver run or workflow
dispatch.

A `nucleoid_enhancement_required` handoff writes no dataset and returns a NUC
request to the caller. Under `/arc-purposal`, the
[arc-purposal skill](../arc-purposal/SKILL.md) handles it. Otherwise the caller
may run `/nuc-proposal` with the request as a single-proposal request, decided
under the automatic acceptance rule, and run `/arc-dataset` again once the
feature is accepted.

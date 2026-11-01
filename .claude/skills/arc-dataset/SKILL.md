---
name: arc-dataset
description: Annotate one ARC puzzle's original training pairs in a single instruction dataset, or return the Nucleoid enhancement request it requires.
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

## One original-training dataset

Create exactly one canonical `arc\src\dataset\dataset.<puzzle-id>.json`.
Annotate every original `train` pair from
`arc\data\training\<puzzle-id>.json`, in its original order. Copy the input and
output grids exactly, including their colors, positions, zero padding, and
canvas dimensions. Do not synthesize, resample, recolor, or duplicate examples
into numbered variants. Never read or include `test` entries or their dimensions.

Use one inferred transformation, Nucleoid declaration set, and output-canvas
mode for all the training pairs. Object identifiers remain unique across
datasets. The loader reads the puzzle's single canonical file, not numbered
variants or other puzzles' examples.

The genesis puzzle `3aa6fb7a` is an exception: preserve its four shared core,
curve, and line datasets and their loading order. Do not regenerate or change
genesis while updating other puzzles. Do not delete or overwrite existing
datasets without explicit authorization; a padding-only audit does not replace
their examples.

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
its puzzle ID exactly once in `jobs.report.env.ARC_PUZZLES` in
`.github/workflows/arc.yml` for the GitHub Actions pass/fail table. Preserve
existing entries and keep the job's timeout budget sufficient for the list,
as the profile specifies. Updating the list does not authorize a solver run
or workflow dispatch.

A `nucleoid_enhancement_required` handoff writes no dataset and returns a NUC
request to the caller. Under `/arc-purposal`, the
[arc-purposal skill](../arc-purposal/SKILL.md) handles it. Otherwise the caller
may run `/nuc-proposal` with the request as a single-proposal request, which
the independent evaluator alone decides, and run `/arc-dataset` again once the
feature is accepted.

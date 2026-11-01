---
name: arc-dataset
description: Write the ARC instruction dataset for one training puzzle ID by running the arc-dataset agent, or return the Nucleoid enhancement request the dataset requires.
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

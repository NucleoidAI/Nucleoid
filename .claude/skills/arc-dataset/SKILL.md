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

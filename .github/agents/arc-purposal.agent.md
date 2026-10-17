---
name: arc-purposal
description: Propose and prototype ARC solver improvements using an object-oriented Nucleoid world model and JSON-only LLM input and output.
---

You propose and prototype one bounded improvement to the `arc/` project.
Use an object-oriented world model to infer reusable transformations from
training examples and solve unseen test inputs, rather than hard-coding answers.

Follow `.github/copilot-instructions.md`, the
[ARC instructions](../instructions/arc.instructions.md), and the
[arc-purposal skill](../skills/arc-purposal/SKILL.md).

## Mandatory LLM contract

**Never use natural language inside ARC LLM prompts. Always use JSON data in
and JSON data out.** Every message content must be a complete JSON document,
including injected examples, metadata, and retry feedback. Do not hide prose
instructions or reasoning inside JSON strings. Executable Nucleoid source is
allowed only as explicitly identified JSON code fields.

## Scope and handoff

- Inspect the existing pipeline, record the baseline, and implement the
  requested ARC change with typed objects, properties, relations, and rules.
- Preserve test integrity: infer from training pairs and test inputs only;
  reserve test outputs, including their dimensions, for scoring.
- Validate every LLM response before executing code or composing grids.
- Run applicable offline checks and report exact per-task results. Mocked
  responses do not establish live ARC-solving accuracy.
- Preserve caller changes and project versions. Do not spawn agents, commit,
  push, merge, or start another proposal cycle.
- Hand language/runtime changes requiring a NUC back to the caller for the
  existing independent proposal process; do not bypass it.

Return one JSON object without Markdown fences or surrounding prose, containing
`status`, `changed_paths`, `world_model`, `checks`, `cases`, and `blockers` as
defined by the skill. Use structured identifiers and error codes, not narrative
reasoning. Stop at this handoff; the caller owns integration and required
post-change agents.

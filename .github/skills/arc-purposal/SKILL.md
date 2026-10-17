---
name: arc-purposal
description: Propose and prototype improvements to the ARC project through an object-oriented world model, JSON-only LLM exchanges, and honest exact-grid evaluation.
---

# Preparing an ARC solver proposal

Use this skill for ARC solver proposals, object-oriented world modeling, and
JSON-only ARC prompt changes. Follow `.github/copilot-instructions.md`, the
[ARC instructions](../../instructions/arc.instructions.md), and the
[agent profile](../../agents/arc-purposal.agent.md).

When delegated as `arc-purposal`, perform the work below directly. Do not spawn
another agent. Work on one bounded change and stop at its JSON handoff.

## Establish scope and baseline

Read the caller's requested change, allowed paths, target task IDs or fixtures,
and any live-model call budget. Inspect the working-tree status and preserve
pre-existing changes. If target tasks are unspecified, use the existing example
task and affected offline fixtures; report that limited scope explicitly.
Missing fixtures or required execution inputs block their evaluation, rather
than justify invented scores. Default to no paid or network LLM calls.

Read `arc/README.md`, `arc/package.json`, and the relevant pipeline:

- `arc/src/app.ts` and `arc/src/types.ts` for orchestration and data contracts.
- `arc/src/lib/analyzer.ts` and `arc/src/lib/visualizer.ts` for inference.
- `arc/src/lib/llm.ts`, `arc/src/lib/openrouter.ts`, and
  `arc/src/instruct_dataset/` for every model-facing message source.
- `arc/src/lib/nucleoid.ts` and `arc/src/lib/Matrix.ts` for symbolic execution
  and grid composition/scoring.

Record the revision, task IDs, execution mode, and actual baseline results in
the session workspace. Keep training and evaluation splits identifiable.
Never inspect or feed held-out outputs into model selection; expose them to
the scorer only after freezing predictions. Unknown results remain `null`.

## Build an object-oriented solution

Choose one concrete failure or missing capability supported by the inspected
code and training examples. Model the world with:

- Grid dimensions, categorical colors, background, and coordinate conventions.
- Object classes and stable per-world instance identities.
- Properties such as occupied cells, position, shape, color, and orientation.
- Relations such as containment, adjacency, alignment, symmetry, and ordering.
- Executable transformation rules and deterministic output composition.

Use only properties and relations needed for the change. Infer them from
training data rather than puzzle filenames or expected test answers. Preserve
full-size instance masks at current interfaces, and make any internal coordinate
conversion explicit. Derive output shape from the learned transformation and
test input, never the expected test grid.

Use Nucleoid's accepted class/instance and declarative semantics to evaluate
symbolic rules. Replay a hypothesis against all selected training pairs before
applying it to test inputs, without carrying training instance state into a
test world. Keep TypeScript types concrete and validate external JSON at runtime;
a generic type argument or cast is not validation.

## Enforce JSON-only model exchanges

**Never use natural language in an ARC LLM prompt: JSON data in, JSON data out.**
This includes shared wrappers and example injection, not just the immediate
analyzer or visualizer call. Keep every role's message content independently
valid JSON. Serialize objects rather than assembling mixed text templates.

Represent the operation, input data, and required response shape structurally.
For example, this is a valid request payload, not a new mandatory wire schema:

```json
{
  "operation": "infer_world_model",
  "data": {
    "train": [
      { "input": [[8]], "output": [[8]] }
    ]
  },
  "response_schema": {
    "type": "object",
    "required": ["declarations"],
    "properties": {
      "declarations": {
        "type": "array",
        "items": { "type": "string" }
      }
    },
    "additionalProperties": false
  }
}
```

Use symbolic operation names, enum constraints, real JSON schemas, and
data/code examples. No prose-valued `instructions`, `description`, `reasoning`,
or similar escape hatches. Carry executable Nucleoid source only inside
explicit code fields; do not add natural-language comments or wrap code in
Markdown. Encode control IDs and validation feedback as JSON data too.

Validate responses before use: required fields, permitted operations, object
references, rectangular grids, integer colors in `0..9`, and dimensions
consistent with the model. Surface malformed data or runtime errors explicitly.
Do not execute invalid responses, scrape JSON from surrounding text, silently
substitute empty objects/grids, or return the input as a success fallback.

## Check the candidate and hand off

Add focused offline tests for the affected behavior. Capture complete outgoing
messages after wrapper/example injection and parse every content as JSON.
Check structural request/response contracts, invalid-response rejection,
object-to-grid round trips, applicable spatial rules, and train/test isolation.
Use deliberately different expected-grid dimensions to detect answer-shape
leakage when testing prediction independence. Preserve existing assertions and
fixtures. Mocked model tests verify contracts, not live solving ability.

From `arc`, run `bun run typecheck` and `bun run lint`. The current offline
matrix tests run with `node --import tsx --test src\lib\Matrix.test.ts`; include
new or affected test files in the same runner. Inspect the manifest before
assuming a `test` script exists. Live-model evaluation requires the caller's
authorization and budget; do not expose `.env` or credentials.

Measure each selected task separately against the unchanged scorer. Report
training replay and held-out scoring separately, exact matches and shape
mismatches, and any regressions. Claim a task solved only for a measured exact
match; do not promise arbitrary ARC generalization or call a mocked answer a
live pass. Update affected ARC documentation and canonical JSONL records only
when the change requires it, preserving stable IDs and annotations.

Return exactly one JSON object, without fences or surrounding prose:

- `status`: `ready_for_review` or `blocked`; never an invented acceptance verdict.
- `changed_paths`: repository-relative paths for this task's changes.
- `world_model`: structured class, property, relation, and operation identifiers.
- `checks`: command, exit code, and execution mode for each completed check.
- `cases`: task ID, split, execution mode, before/after difference, and exact-match
  booleans; use `null` for unmeasured values.
- `blockers`: structured error codes and affected paths or task IDs.

Keep supporting baseline/result artifacts in the session workspace and include
their paths in the JSON when present. Do not commit, push, merge, change project
versions, start a NUC, or launch another proposal. The caller owns integration,
any required NUC workflow, and post-change improvement agents.

---
applyTo: "**"
---

# ARC project and solver proposals

These rules apply when working on `arc/`, its LLM exchanges, or its associated
ARC records. Use the `arc-purposal` agent and matching skill when delegating an
ARC proposal. Agent-configuration changes alone do not launch a solver run.

## No automated ARC tests

Do not add tests for the ARC project. Remove existing ARC-related automated
tests wherever they live, including shared Rust suites outside `arc/`.
Do not replace them with new test files, embedded automated cases, test-only
fixtures, test runners, or test-framework configuration. Preserve non-ARC tests.

Use type checking, linting, builds, and manual/offline inspection to verify ARC
changes. Keep puzzle datasets, instruction examples, and training/held-out
evaluation intact; these are solver data and benchmarking, not automated tests.

## JSON data in, JSON data out

- Never use natural-language prompts for ARC LLM calls. Each message's content
  must independently parse as one complete JSON document, regardless of role.
  Construct data objects and serialize them with `JSON.stringify`; do not
  concatenate labels, raw source, Markdown, or pseudo-JSON around them.
- Cover the entire prompt path: analyzer, visualizer, shared LLM wrapper,
  instruction datasets, few-shot examples, control IDs, and retry feedback.
  Provider JSON output mode does not enforce JSON input.
- Encode operations, schemas, constraints, examples, and diagnostics as
  structured data with identifiers and enum values. No natural-language
  instructions, explanations, chain-of-thought, or prose hidden in string
  values. JSON keys, identifiers, and executable Nucleoid code fields such as
  `declarations` and `input_code` are permitted; prose comments in code are not.
- Require exactly one JSON response matching the operation's schema. Validate
  its structure and values before code execution or grid composition.
  Reject surrounding prose, Markdown fences, malformed JSON, invalid grids,
  and unknown operations explicitly; never strip text to manufacture success.
- Keep errors visible. Any model-facing retry uses JSON error codes and field
  paths, stays within the caller's budget, and never supplies test answers.
  Human-facing repository documentation and terminal diagnostics may use prose.

## Object-oriented world model

Represent grids as worlds containing identifiable object instances with
properties, spatial relations, and transformation rules. Use concrete
TypeScript types and executable Nucleoid classes, instances, and declarative
rules where appropriate; JSON is the exchange format, not a substitute for
symbolic execution. Avoid unstructured textual reasoning as the world model.

Infer segmentation, background, object roles, and transformations from training
pairs. Treat colors `0` through `9` as categorical values. Preserve positions
and the current full-size, zero-padded instance matrices at existing boundaries.
If a representation changes, supply explicit conversion and verify it through
manual/offline checks instead of silently switching to cropped objects. Define
output dimensions, overlap handling, and composition through the inferred model.

## Honest evaluation and bounded changes

Never use test outputs or their dimensions to infer rules, initialize result
grids, choose candidates, or construct prompts. Access expected test grids only
after predictions are frozen, for scoring. Do not hard-code puzzle IDs,
expected grids, or task-specific answers into solver logic.

Preserve puzzle datasets, expected outputs, instruction examples, and scoring.
Replay every selected training pair, then score held-out predictions by exact
grid equality, including shape. Record actual per-task before/after differences;
an unmeasured result is unknown, not zero. Distinguish manual/offline evaluation
from live-model results and report failures or regressions without altering
expected grids or scoring.

Keep changes focused on the ARC project and synchronize affected documentation
and canonical ARC records when their behavior changes. Follow the existing NUC
workflow for language changes and the human-owned version policy. Do not read
or expose credentials, make paid LLM calls without authorization, or alter
provider/model settings merely to manufacture a passing result.

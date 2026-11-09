---
applyTo: "**"
---

# ARC project and solver proposals

These rules apply when working on `arc/`, its LLM exchanges, or its associated
ARC records. Run an ARC proposal with `/arc-purposal [<puzzle-id>]` (the
`arc-purposal` skill and agent), which selects a default puzzle when the ID is
omitted. The `arc-dataset` agent writes the puzzle's
instruction dataset, any Nucleoid enhancement that dataset requires goes
through the NUC stages, and its outcome follows the automatic acceptance rule,
with one solver run whose exact match makes it Accepted. Whenever a quote is
used in `arc/src/dataset/`, it also triggers `/nuc-proposal` to propose a
version without it, under the
[skill's quote step](../skills/arc-purposal/SKILL.md#propose-quote-free-spellings);
that NUC is never run against the puzzle and follows the automatic acceptance
rule directly. Its bare words are always on, and the user accepts in advance
its one compatibility break: an undefined bare word that raised
`ReferenceError` reads as its own text. `arc/nucleoid.ebnf` is the minimal
grammar of the syntax that
the code fields in `arc/src/dataset/` use, and it stays minimal: whenever a
dataset introduces new syntax, `/arc-purposal` adds that syntax to it under the
[skill's grammar step](../skills/arc-purposal/SKILL.md#keep-the-dataset-grammar),
and a form leaves it once no dataset uses it. Afterwards it
documents the puzzle's progress in `arc/docs/ARC-NUC-<puzzle-id>.md`. When it
is done, it commits and pushes its own changes under the
[git synchronization policy](git.instructions.md).
`/arc-purposal` never changes the TypeScript implementation in `arc/`, except
that it sets the default `puzzle_id` in `arc/src/app.ts` to the puzzle it runs;
that default only selects the puzzle for a bare `bun start` and is not solver
logic. It orchestrates its approach between `/arc-dataset` and `/nuc-proposal`
and, when the solver would need a change, suggests that change at the end of
the process.
Agent-configuration changes alone do not launch a solver run.

Every ARC-requested NUC uses the existing proposal-strategy UCB1 bandit under
the [arc-purposal skill](../skills/arc-purposal/SKILL.md#learn-the-nuc-proposal-strategy).
Require a retained policy head or explicit `--strategy-policy <absolute-json-path>`
before drafting; never initialize/reset history automatically. Select before
dispatch, preserve the pinned rubric and attribution through both NUC agents,
and persist the independent evaluator's checkpoint or legitimate unscored skip
before the ARC run. Missing/invalid learning state pauses the NUC; a
dataset-only invocation needs no policy. ARC results and eventual acceptance
are never learning rewards and do not change the existing NUC reward formula.

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
  `declarations` and `implementation` are permitted. Natural-language `#`
  comments reach the model only as part of those two code fields: in
  `declarations` they describe the world model's classes and rules, and in
  `implementation` they describe the object it constructs. No other field
  carries them.
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

Always prefer declarative, higher-order logic. State each transformation once
as rules over every object and the relations between objects: class-level
rules and conditions on each class of the world model (for example `$Obj`),
properties derived from other rules, and higher-order operations such as
`filter`, `map`, `reduce`, `some`, `every`, `find`, and reasoning selections. Avoid imperative loops, per-instance or
per-case steps, and manual bookkeeping. When accepted Nucleoid cannot express
such a formulation, request the enhancement it needs, which `/arc-purposal`
carries through `/nuc-proposal`, rather than settling for an imperative or
special-cased workaround.

Infer segmentation, background, object roles, and transformations from training
pairs. Treat colors `0` through `9` as categorical values. Instruction datasets
write them as letters: `0` stays `0`, and every other color is a letter of
`ABCDEFGHI` ranked by its count across the dataset's input and output matrices,
`A` for the most used, under the
[arc-dataset profile](../agents/arc-dataset.agent.md#write-colors-as-letters).
Preserve positions
and the current full-size, zero-padded instance matrices at existing boundaries.
If a representation changes, supply explicit conversion and verify it through
manual/offline checks instead of silently switching to cropped objects. Define
output dimensions, overlap handling, and composition through the inferred model.

## Honest evaluation and bounded changes

Never use test outputs or their dimensions to infer rules, initialize result
grids, choose candidates, or construct prompts. Access expected test grids only
after predictions are frozen, for scoring. Do not hard-code puzzle IDs,
expected grids, or task-specific answers into solver logic or language features.
The solver's output shows the expected grid, so never feed it back into
datasets, NUC candidates, or prompts. Deciding an `/arc-purposal` NUC from its
one scored run is scoring, not candidate selection: never rerun it or compare
alternatives against the expected grid.

The shared few-shot context is the one exception. `document()` sends every
instruction dataset to every run, including the running puzzle's own annotated
test pairs. That is intended: the datasets are being prepared as few-shot
examples, and runs at this stage are not yet held-out tests. Do not report it
as a leak or suggest excluding those pairs. Every other rule here still holds,
and rules are still inferred from the training pairs alone.

Preserve puzzle datasets, expected outputs, instruction examples, and scoring.
Replay every selected training pair, then score held-out predictions by exact
grid equality, including shape. Record actual per-task before/after differences;
an unmeasured result is unknown, not zero. Distinguish manual/offline evaluation
from live-model results and report failures or regressions without altering
expected grids or scoring.

Keep changes focused on the ARC project and synchronize affected documentation
and canonical ARC records when their behavior changes. Follow the existing NUC
workflow for language changes and the human-owned version policy. A NUC that
`/arc-purposal` requests follows the
[automatic acceptance rule](proposals.instructions.md#automatic-acceptance-caller-only):
when its candidate meets every hard criterion (no compile errors, every test
passing, backward compatibility), an exact match makes it Accepted regardless
of its evaluation, and otherwise it is Accepted or Automatic-Accepted; only a
failed hard criterion rejects it, under the
[ARC-requested proposal rule](proposals.instructions.md#arc-requested-proposals-caller-only).
Do not read or expose credentials, make paid LLM calls without authorization,
or alter provider/model settings merely to manufacture a passing result. An
`/arc-purposal` invocation authorizes its one `bun start <puzzle-id>` run and
nothing more.

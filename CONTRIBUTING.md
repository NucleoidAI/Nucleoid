# Contributing

Thanks to declarative programming, we have a brand-new approach to data and logic. As we are still discovering what we can do with this powerful programming model, please join us with any types of contribution!

## Working on the Rust runtime

The Rust crate is at the repository root.

```console
$ cargo test                  # the whole suite
$ cargo clippy --all-targets -- -D warnings
$ cargo fmt
$ cargo run -- program.nuc    # run a file
$ cargo run                   # or type statements at a prompt
```

### What lives where

| path | what it is |
| --- | --- |
| `nucleoid.spec.md` | what the language does. Authoritative — when anything disagrees with it, it wins |
| `dataset/nucleoid.spec.jsonl` | generated specification export, rendered from `nucleoid.spec.md` |
| `dataset/nucleoid.synth.*.jsonl` | canonical synthesized fine-tuning records; not normative runtime tests |
| `arc/src/instruct_dataset/` | ARC instruction examples, maintained within the ARC project |
| `dataset/` | JSONL files only; no Markdown mirrors or local dataset card |
| `docs/` | the language reference, as NUC documents in the style of a PEP |
| `docs/README.md` | prose reference; `tests/reference.md` is its executable form |
| `src/` | the runtime |
| `tests/` | executable specification, documentation, dataset, and runtime checks |

The ARC project does not keep automated tests, including ARC-related cases in
shared Rust suites. Follow the [ARC testing policy](.github/instructions/arc.instructions.md#no-automated-arc-tests):
use type checking, linting, builds, and manual/offline evaluation instead.
Preserve non-ARC tests, puzzle datasets, and instruction examples.

### Adding a behaviour

Cases are written in Nucleoid, not in Rust. Add one to `nucleoid.spec.md` —
title it with a comment, and end it with `assert(...)` or a `# return:` line:

```text
# Nucleoid keeps a total in step with what it is made of

# subtotal is 10
subtotal = 10

# total is subtotal plus tax
total = subtotal * 1.2

assert(total, 12)
```

`build.rs` turns every case into a test of its own, so there is nothing else to
edit — `cargo test` will show it by name:

```console
$ cargo test --test nucleoid_spec keeps_a_total
test nucleoid::keeps_a_total_in_step_with_what_it_is_made_of ... ok
```

Two rules the suites enforce, both of which fail the build rather than pass
quietly:

- **Every case needs a title of its own.** Tests find their case by title, so a
  repeated title would leave the second case unreachable.
- **Assertions have to actually run.** A case whose `assert` sits in a `catch`
  that never fires proves nothing, so the harness compares how many assertions
  ran against how many the source contains.

### Keeping things in step

`nucleoid.spec.md`, the training records in `dataset/`, `docs/` and the crate
describe the same language. Propagate affected behavior changes to the relevant
records and references in the same change. `cargo test` executes normative
documents, verifies the specification export, and validates standalone training
records. Synthesized programs are not normative runtime tests.

Edit `nucleoid.spec.md`, not its generated JSONL export. Regenerate that export
and its Rust tests from the repository root when the specification changes:

```console
$ UPDATE_DATASET=1 cargo test --test dataset
$ UPDATE_SPEC_TESTS=1 cargo build
```

Maintain synthesized and ARC cases directly in their existing JSONL files,
without Markdown source copies. `UPDATE_DATASET` updates only the normative
specification export; it does not rewrite these training records. Each record
has a stable `id`, a `description`, NUC `code`, and a nullable JSON-encoded
`return`. ARC records also retain explicit grids, object annotations, provenance,
and normalization notes.

Keep `dataset/` JSONL-only. Do not recreate `docs/dataset/` or move duplicate
Markdown datasets or dataset cards somewhere else.

### Evaluating a language proposal

Every added or updated `docs/nuc-XXXX.md` is a proposal revision, including
process and documentation-only NUCs. During a Copilot session, the repository
instructions require a fresh `nuc-proposal` agent followed by an independent
fresh `nuc-evaluate` agent per revision. The proposer drafts and prototypes in
an isolated candidate, preserving behavior, case identities, canonical
assertions, return expectations, and executed assertion coverage. It hands off
the proposal's Git blob revision without deciding acceptance. The evaluator
checks the frozen candidate without rewriting it, measuring tokens only when
the execution-mode policy requires comparison.

The procedures live in the [proposal skill](.github/skills/nuc-proposal/SKILL.md)
and [evaluation skill](.github/skills/nuc-evaluate/SKILL.md), with separate
profiles in `.github/agents/`. The caller starts both stages; neither agent
delegates to the other. See [NUC 1](docs/nuc-0001.md#proposal-evaluation).

Every NUC has one `Category` header from the extensible
[proposal category registry](docs/nuc-0001.md#proposal-categories), separate
from its `Type` and `Status`. Whenever a category is added, selected, or updated,
ask the user for confirmation before proceeding if its meaning, eligibility,
or proposal assignments are uncertain. Every category addition requires
reviewing all existing proposals and updating those that fit the new category
more accurately, with synchronized category and numerical indexes. Unspecified
enhancements use an equal random draw across the currently eligible categories;
Documentation is not part of that draw.

The **token non-regression gate** requires each compared case to use the same
or fewer lexer tokens than its accepted baseline. `/nuc-proposal`
defaults to continuous mode for open-ended requests. A request for one specific
NUC, concrete feature, or particular revision uses single-proposal mode: skip
token comparison, independently evaluate behavior/compatibility/development,
record the outcome, then stop without starting another proposal.

Only continuous cycles use this matrix:

| Category | Token comparison |
| --- | --- |
| Syntax enhancement | Required |
| Runtime features | Required for existing comparable features; skipped for brand-new features or genuinely uncomparable existing features with explicit reason and evidence |
| Utility enhancement | Required |
| Reasoning enhancement | Required |
| Documentation | Skipped |

All modes retain immutable accepted-corpus baselines, pre-existing case
integrity, assertion execution/coverage, explicit return checks, backward
syntax compatibility, development checks, and independent evaluation.
An invalid baseline, bad input, or behavior/compatibility failure never becomes
a successful skip. Missing comparisons still block a required token gate;
missing artifacts do not establish that a feature is uncomparable.

When required, the gate measures each case from the specification and executable
reference, README, and examples. It uses Nucleoid lexer tokens, including
`Newline`, `Indent`, and `Dedent`, excluding `Eof`; it does not measure LLM
tokens, generated Rust, or synthesized training records. Each paired case must
use the same or fewer tokens. Any increase rejects the proposal, regardless of
savings in other cases.

Record the accepted baseline before editing the runtime or executable corpus.
The recorder reads documents from the pinned Git revision and refuses to use
a runtime that differs from it. Use a separate baseline worktree if candidate
runtime edits already exist. `HEAD` is suitable only when it is the accepted
revision, not a commit containing an unevaluated proposal.

Example PowerShell commands, using fresh artifact paths outside the repository:

```powershell
$env:NUC_PROPOSAL_BASE_REF = 'HEAD'
$env:NUC_PROPOSAL_BASELINE = Join-Path $env:TEMP 'nucleoid-nuc-baseline.json'
cargo test --test proposals record_baseline -- --ignored --nocapture
```

After the proposal handoff, the fresh evaluator runs from the frozen candidate
worktree with the same baseline path:

```powershell
$env:NUC_PROPOSAL = 'docs\nuc-0001.md'
$env:NUC_PROPOSAL_MODE = 'single'
$env:NUC_PROPOSAL_REPORT = Join-Path $env:TEMP 'nucleoid-nuc-0001-report.json'
cargo test --test proposals evaluate_proposal -- --ignored --nocapture
```

`NUC_PROPOSAL_MODE` accepts only `continuous` or `single`; omitted means
`continuous` for existing open-ended invocations. Pass `single` explicitly for
one-shot requests and preserve the caller's mode throughout evaluation.
The evaluator reads the actual fenced `Category` header and checks it against
NUC 1's registry; there is no category override or unchecked ignore-failures flag.

For a Runtime features proposal, also set `NUC_PROPOSAL_RUNTIME` to exactly one
of `new_feature`, `existing_comparable`, or `existing_uncomparable`, even in
single mode. The last requires both `NUC_PROPOSAL_RUNTIME_REASON` and
`NUC_PROPOSAL_RUNTIME_EVIDENCE`: nonblank explanations and supporting evidence
for why the existing feature genuinely cannot be compared in accepted syntax.
The independent evaluator verifies that classification and evidence.
Runtime inputs are invalid for other categories; reason/evidence are invalid
for the other runtime branches. Unset inapplicable variables rather than
leaving stale skip inputs.

Baseline and report files are created once and never overwritten. Use fresh
paths for later revisions; keep actual evaluation artifacts in the Copilot
session workspace. A nonzero exit means rejected or blocked, not accepted.
The report records the proposal's Git blob revision, actual mode/category/runtime
applicability and rationale, behavior checks, and `token_check.status`.
For `ran`, it includes the metric and each paired case's `before`, `after`,
and token change. For `skipped`, it includes the skip reason, not measured cases
or zero placeholder counts; this is not a passing token gate.
The evaluator does not fix failures or regenerate exports. A repaired,
substantively revised, or rebased candidate needs a fresh agent pair and report.

When token comparison is required, prepare an additional baseline JSON array
for new cases in accepted syntax **before** candidate runtime edits.
Each entry names the case's
destination document (`spec`, `reference`, `readme`, or `examples`) and exact
title, and contains its accepted-syntax source and optional expected return.
Each new baseline must contain assertions or an expected return:

```json
[
  {
    "document": "spec",
    "title": "Nucleoid keeps a proposed total",
    "source": "total = (1 + 2)\nassert(total, 3)",
    "expected": null
  }
]
```

Set `NUC_PROPOSAL_EXTRA_BASELINES` to this file before `record_baseline`.
The accepted runtime executes these programs when recording them. The
candidate must contain matching cases and unchanged canonical assertions and
return expectations. Supply the original extra-baseline sources through the
same variable during evaluation, so unchanged legacy programs can be replayed.
Missing required comparisons block acceptance rather than receiving invented
before-counts.

When comparison is legitimately skipped, genuinely new cases need no
accepted-syntax counterpart, but must execute their assertions or explicit
expected returns with full assertion coverage. Every pre-existing case remains
protected, and every pinned accepted program and supplied extra-baseline program
is replayed under the candidate. Baseline JSON must be valid and match its
pinned corpus; skipping tokens never excuses missing accepted-corpus inputs.

The proposer regenerates affected exports before handoff; the evaluator checks
their consistency without setting either update variable. After independent
acceptance, the caller integrates the unchanged candidate. Leave rejected
candidates out of the accepted runtime;
do not discard someone else's edits. Outcome status/index updates are part of
the evaluation, not triggers for recursively creating more agents. These
instructions apply to Copilot sessions; there is no background file watcher
or GitHub Actions proposal trigger.

#### Advisory usefulness/cost scoring

The existing Rust `evaluate_proposal` entry point also reports advisory
experience scores. It does not implement a learner, adaptive proposal/category
selection, model training, paid calls, or a language/runtime feature. Scores
never change deterministic verdicts, independent acceptance requirements, or
the single/continuous token policy.

Before drafting, the **caller**, not the proposer, may create and freeze a
usefulness rubric outside the repository. Its exact JSON schema is:

```json
{
  "criterion_id": "caller-defined-observable-benefit",
  "proposal_path": "docs\\nuc-0001.md",
  "baseline_revision": "<full accepted commit ID>",
  "criterion": "<independently testable benefit requested before drafting>"
}
```

Pin the raw rubric's full Git blob before drafting, preserve it in both
handoffs, and keep the original criterion immutable. For example:

```powershell
$env:NUC_PROPOSAL_RUBRIC = 'C:\session\nuc-rubric.json'
$env:NUC_PROPOSAL_RUBRIC_REVISION = (git --no-pager hash-object --no-filters -- $env:NUC_PROPOSAL_RUBRIC).Trim()
```

After independently checking the frozen candidate, the **evaluator** may
create a new session assessment, never edit the candidate or use the proposer's
self-assessment. This template describes the schema, not actual evidence;
replace placeholders only with independently verified inputs/results:

```json
{
  "criterion_id": "caller-defined-observable-benefit",
  "rubric_revision": "<caller-frozen full rubric Git blob ID>",
  "proposal_path": "docs\\nuc-0001.md",
  "baseline_revision": "<full accepted commit ID>",
  "proposal_revision": "<exact handed-off proposal Git blob ID>",
  "assessor": {
    "role": "independent_evaluator",
    "id": "<independent evaluator/session identity>"
  },
  "checks": {
    "behavior_integrity": {
      "status": "unresolved",
      "reason": "<missing independent behavior/integrity result>"
    },
    "syntax_compatibility": {
      "status": "unresolved",
      "reason": "<missing pinned-source and extra-baseline replay result>"
    },
    "development_suites": {
      "status": "unresolved",
      "reason": "<missing affected-suite commands and results>"
    },
    "formatting": {
      "status": "unresolved",
      "reason": "<missing cargo fmt --check result>"
    },
    "clippy": {
      "status": "unresolved",
      "reason": "<missing warning-denied Clippy result>"
    },
    "export_consistency": {
      "status": "unresolved",
      "reason": "<missing no-regeneration export/dataset consistency result>"
    }
  },
  "usefulness": {
    "status": "unknown",
    "reason": "<criterion not yet independently assessed>"
  }
}
```

- Every check group is required: `passed` and `failed` each require a nonblank
  `evidence` field instead of `reason`; `unresolved` requires a nonblank `reason`.
  Record actual commands/results, not a missing or assumed check. The core
  gate does not run external development suites, formatting or Clippy.
- Usefulness is `demonstrated` or `no_demonstrated_benefit` with nonblank
  `evidence`, or `unknown` with a nonblank `reason`. Judge the frozen criterion
  through independent observations; acceptance, changed lines, extra
  assertions/cases, token skipping and proposer claims do not demonstrate benefit.
- Only `independent_evaluator` is a valid assessor role. Its nonblank ID and
  evidence record provenance, not authentication; the evaluation agent remains
  responsible for genuine independence and truthful evidence.
- The rubric pin, criterion ID, accepted baseline, proposal path and actual
  handed-off proposal blob must match. Either path separator is accepted in
  proposal identities. Input blobs are checked for stability during evaluation.
  Unknown/missing fields, invalid statuses, contradictory status/fields, blank
  evidence, stale bindings and integrity/legacy success claims contradicted by
  gate failures are explicit errors, never successful unscored fallbacks.
- Both inputs must be absolute external `.json` paths. Add the independently
  created assessment to the normal evaluation command's environment:

```powershell
$env:NUC_PROPOSAL_ASSESSMENT = 'C:\session\nuc-assessment.json'
cargo test --test proposals evaluate_proposal -- --ignored --nocapture
```

- The rubric path and original pin are required together; an assessment also
  requires that pinned rubric. Keep all scoring artifacts create-once in the
  session workspace, never overwrite them or put them in the repository.
- Leave all three optional variables unset for legacy evaluation without
  scoring evidence; empty/stale variables are not absence. The gate runs
  normally and explicitly reports unscored usefulness, not zero. A pinned rubric
  without an assessment is also valid and unscored.
- `advisory_score.metric` is `nuc-advisory-usefulness-evaluator-time-v1`.
  `usefulness` reports demonstrated/no-benefit/unscored separately from reward;
  `usefulness_basis` preserves rubric/assessment paths, blobs, criterion,
  assessor and check evidence, while `independent_check_eligibility` is explicit.
- A blocked core result is unscored. A genuine core rejection has base/final
  reward -1 regardless of usefulness; with a passed core, a known independent
  failure likewise earns -1, while unresolved checks remain unscored.
- With a passed core and **all** independent checks passed, demonstrated
  usefulness earns base +1, adjustment `-0.1 * t / (t + 1)` and final reward
  `1 - 0.1 * t / (t + 1)` in `[0.9, 1]`. An independently assessed valid
  no-benefit change stays neutral 0 without a cost adjustment. Missing/unknown
  usefulness stays unscored, with a reason and no fabricated numeric reward.
- `t` is the actual `Instant`-measured `evaluation_time.elapsed_seconds`:
  deterministic evaluator input validation, source loading, legacy/candidate
  checks, and blob stability verification. The report names this scope and
  formula and records the raw seconds even for reported neutral/rejected/
  blocked/unknown states. No report/duration is fabricated on an early error.
- This nondeterministic measurement excludes compilation, external suites/
  Clippy, proposal/model time, score calculation and report writing. It is not
  language runtime performance, total end-to-end proposal cost or API/model
  spending; there is no token-savings bonus or token-skip cost credit.
- The existing top-level `decision` remains the deterministic core gate's
  verdict, **not** the complete independent decision. External failures still
  require Rejected, and unresolved prerequisites still require Blocked under the
  existing process, irrespective of an advisory score. Return both evidence and
  scoring limitations honestly.

## Declarative Runtime Environment

Nucleoid records declarative statements in a dependency graph and re-evaluates
them when their inputs change. Programs describe relationships to maintain
rather than the sequence of updates needed to maintain them.

Learn more at [nucleoid.com/docs/runtime](https://nucleoid.com/docs/runtime/)

## Join our [Thinkers Club](https://github.com/NucleoidJS/Nucleoid/discussions/categories/thinkers-club)

If you have an opinion, you are already a philosopher. We are working on brand-new approach to data and logic. Come join us in [discussions](https://github.com/NucleoidJS/Nucleoid/discussions/categories/thinkers-club).

[![Nobel](https://cdn.nucleoid.com/media/nobel.png)](https://github.com/NucleoidJS/Nucleoid/discussions/categories/thinkers-club)

### Pinned Discussions

[![Discussion 25](https://cdn.nucleoid.com/media/discussion-25x500.png)](https://github.com/NucleoidJS/Nucleoid/discussions/25)
[![Discussion 26](https://cdn.nucleoid.com/media/discussion-26x500.png)](https://github.com/NucleoidJS/Nucleoid/discussions/26)
[![Discussion 28](https://cdn.nucleoid.com/media/discussion-28x500.png)](https://github.com/NucleoidJS/Nucleoid/discussions/28)

## Code of Conduct

Please read our [Code of Conduct](https://github.com/NucleoidJS/Nucleoid/blob/main/CODE_OF_CONDUCT.md)

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
checks the frozen candidate and measures tokens without rewriting it.

The procedures live in the [proposal skill](.github/skills/nuc-proposal/SKILL.md)
and [evaluation skill](.github/skills/nuc-evaluate/SKILL.md), with separate
profiles in `.github/agents/`. The caller starts both stages; neither agent
delegates to the other. See [NUC 1](docs/nuc-0001.md#proposal-evaluation).

The gate measures each case from the specification and executable reference,
README, and examples. It uses Nucleoid lexer tokens, including `Newline`,
`Indent`, and `Dedent`, excluding `Eof`; it does not measure LLM tokens,
generated Rust, or synthesized training records. Each case must use the same
or fewer tokens. Any increase rejects the proposal, regardless of savings in
other cases. Missing baselines block acceptance.

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
$env:NUC_PROPOSAL = 'docs\nuc-0012.md'
$env:NUC_PROPOSAL_REPORT = Join-Path $env:TEMP 'nucleoid-nuc-0012-report.json'
cargo test --test proposals evaluate_proposal -- --ignored --nocapture
```

Baseline and report files are created once and never overwritten. Use fresh
paths for later revisions; keep actual evaluation artifacts in the Copilot
session workspace. A nonzero exit means rejected or blocked, not accepted.
The report records the proposal's Git blob revision and every paired case's
`before`, `after`, and token change.
Process-only proposals can pass with all cases unchanged.
The evaluator does not fix failures or regenerate exports. A repaired,
substantively revised, or rebased candidate needs a fresh agent pair and report.

For new cases, prepare an additional baseline JSON array in the accepted
syntax **before** candidate runtime edits. Each entry names the case's
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
return expectations. If the old language cannot express an equivalent case,
the proposal is blocked rather than assigned an invented before-count.

The proposer regenerates affected exports before handoff; the evaluator checks
their consistency without setting either update variable. After independent
acceptance, the caller integrates the unchanged candidate. Leave rejected
candidates out of the accepted runtime;
do not discard someone else's edits. Outcome status/index updates are part of
the evaluation, not triggers for recursively creating more agents. These
instructions apply to Copilot sessions; there is no background file watcher
or GitHub Actions proposal trigger.

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

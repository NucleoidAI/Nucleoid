---
name: nuc-proposal
description: Draft or revise a Nucleoid NUC and prototype its changes. Use for proposal preparation, not acceptance evaluation.
argument-hint: "[request] [--hint <text>]"
---

# Preparing a NUC proposal

When coordinating a NUC revision, launch a fresh `nuc-proposal` agent using
[its profile](../../agents/nuc-proposal.agent.md) and this skill. If named
custom agents are unavailable, give both documents to a fresh general-purpose
agent. Inside that agent, execute the steps below; do not spawn another agent.

Follow `.claude/CLAUDE.md` and `docs/nuc-0001.md`.
Work on exactly one proposal revision in the caller's isolated candidate.

## Allocate the NUC first (caller only)

Every `/nuc-proposal` task runs in this order, and every continuous cycle
repeats it:

1. **Pull.** As always before starting a task, pull the primary checkout with
   `git pull --no-rebase --autostash` under the
   [git synchronization policy](../../rules/git.instructions.md#pull-first),
   so the task runs against the most current remote state. Report the commits
   it brought in.
2. **Pick up a stale Draft first.** In continuous mode, look for an allocated
   Draft that has gone 24 hours without an allocation or claim commit. When
   one exists, the cycle claims and completes it under the
   [stale-Draft rule](#pick-up-stale-drafts-first-caller-only) instead of
   starting a new NUC, and continues at step 6 with that claim pinned.
3. **Draft first.** Once the
   [execution mode](#choose-execution-mode-caller-only), the confirmed
   [category](#classify-every-proposal) and the requested change are chosen,
   number the new NUC as the highest NUC recorded in `docs/` plus one. In the
   primary checkout, write `docs/nuc-XXXX.md` with its complete header,
   including `Status: Draft` and the confirmed `Category`, and its proposal
   details: the Abstract and Motivation of the requested change, followed by
   the remaining required sections for the proposer to complete. Add its Draft
   rows to both indexes in `docs/nuc-0000.md`, and change nothing else.
4. **Check before submitting.** Right before the allocation commit, pull
   again and compare the draft with every NUC that `main` now records:
   - **Same number:** when another NUC already uses the drafted number,
     claim the next one, the highest number recorded plus one, and renumber
     the draft's file name, title line, header and both index rows.
   - **Same concept:** when a NUC of any status already proposes the same
     concept, the same feature or change under any title, skip this
     proposal. Remove the uncommitted draft and its own index rows, allocate
     nothing, and start again at step 1 with a new selection under the
     [selection rule](#select-an-unspecified-enhancement). A request that
     names that concept stops instead and reports the existing NUC.
5. **Allocate.** Commit those two paths alone, with the subject
   `Draft NUC <number> <Title>` and the category and requested change in the
   body, and push right away. The pushed commit allocates the proposal and its
   number, so concurrent sessions number their NUCs after it.
6. **Work on the details as usual.** Pin the `HEAD` just pushed, which holds
   the allocation or the claim, as the accepted revision, record its baseline,
   and create the candidate worktree from it. Then
   [select the strategy](#select-and-consume-a-proposal-strategy) and run the
   fresh proposer, which completes the draft and prototypes the change, the
   Rust improvement pass and the fresh evaluator, as the rest of this skill
   describes.
7. **Commit and push the final version.** Record the outcome in a new commit,
   `NUC <number> <Title>`, that holds the final proposal along with its code
   changes when it is accepted, and push it right after, under the
   [outcome commit rule](#name-outcome-commits-caller-only).

- Pull again right before each commit and each push. When a pull brings in
  the drafted number, keep the other session's NUC and renumber the draft to
  main's next number, in the merge if the allocation is already committed.
  When a push is rejected because the remote moved, pull, resolve, and push
  again until it succeeds.
- Stage the two paths explicitly, only the allocation's own hunks where
  another session's uncommitted edits share them, and verify that the commit
  holds nothing else. It changes no runtime, executable document, test,
  dataset, export or grammar, so its language is still the accepted language.
- Verify the inputs that do not depend on the pinned revision, such as a
  configured strategy policy, before drafting, so a missing one blocks the
  task before anything is committed.
- The allocation is not an outcome and needs no evaluation of its own; the
  one proposal/evaluation pair covers the final version. The draft records
  only the caller's own inputs, so the strategy selection and any usefulness
  rubric, which bind the pinned revision, still precede the proposer's
  drafting.
- The final commit builds on the allocation; never amend or rewrite the
  pushed allocation. For Accepted or Automatic-Accepted, it integrates the
  unchanged frozen candidate with the final proposal and its status/index
  metadata. For Rejected, it holds only the final rejection documentation
  under the [rejection rule](#record-rejected-outcomes-caller-only), and the
  candidate's code stays isolated.
- A revision of an existing NUC, including a revisited Rejected NUC, already
  holds its number: skip the draft and allocation steps, and leave its text
  and status on `main` unchanged until the outcome commit. A Blocked or paused
  NUC keeps its pushed Draft and number, and a later task claims and resumes
  it under that number, under the
  [stale-Draft rule](#pick-up-stale-drafts-first-caller-only), instead of
  allocating another.

## Pick up stale Drafts first (caller only)

In continuous mode, every cycle, the first one included, completes an
abandoned NUC before it starts a new one. A single-proposal request still does
only what it asks.

1. **Find.** After the cycle's pull, a NUC is a stale Draft when its header on
   `main` still reads `Status: Draft`, it has a pushed
   `Draft NUC <number> <Title>` allocation commit, and the latest of that
   commit and any `Resume NUC <number> <Title>` claim commit is more than
   24 hours old by committer date, which
   `git log -1 --format=%cI -E --grep='^(Draft|Resume) NUC <number> '` shows.
   A Draft without an allocation commit, such as the reference NUCs 2–7 and
   9–11, is never picked up. When the request names a category, only Drafts
   of that category count.
2. **Pick the lowest-numbered stale Draft,** one per cycle. The cycle skips
   the [selection](#select-an-unspecified-enhancement) and steps 3–5 of the
   [allocation order](#allocate-the-nuc-first-caller-only). Its confirmed
   inputs are the Draft's number, `Title` and `Category`, and the requested
   change that its Abstract, Motivation and allocation commit body record. An
   uncertain category still goes to the user under the
   [classification rule](#classify-every-proposal).
3. **Claim it.** Pull again right before the claim, and look again when the
   NUC is no longer a stale Draft. Otherwise make the claim with
   `git commit --allow-empty --only`, which leaves anything staged out of it:
   the subject `Resume NUC <number> <Title>`, and in the body the allocation
   commit and how long the Draft waited. Verify that the commit holds no file
   and push it right away. The pushed claim restarts the Draft's 24 hours, so
   other sessions leave it alone, as the allocation keeps them off its number.
   A claim whose push is rejected for another session's claim or outcome of
   that NUC never holds it: push the merge as usual, leave the NUC to that
   session and look again.
4. **Complete it like a new NUC.** Pin the pushed claim and continue at step 6
   of the allocation order with a fresh baseline, candidate worktree, strategy
   trial, proposer and evaluator; the proposer completes the Draft under its
   number and category. Never reuse another session's candidate worktree,
   agents, baselines, receipts or reports, and leave them as they are.
5. **Decide under the automatic acceptance rule directly.** The cycle stays in
   continuous mode, so the
   [NUC 1 token policy](../../../docs/nuc-0001.md#proposal-evaluation)
   follows the Draft's registered `Category`. The
   [ARC-requested](../../rules/proposals.instructions.md#arc-requested-proposals-caller-only)
   and
   [user-requested](../../rules/proposals.instructions.md#user-requested-proposals-caller-only)
   bases cover only the invocation that requested a NUC, so they never carry
   over to a pickup.

Only a cycle that finds no stale Draft selects and allocates a new NUC. A
picked-up NUC that is Blocked pauses the loop like any other and keeps its
Draft, which a later cycle picks up again once its claim is 24 hours old.

Every task that works on an allocated NUC, in either mode, keeps it claimed:

- **Renew.** At each pull, when the latest allocation or claim of the NUC it
  is still working on is more than 12 hours old, half the 24 hours, the task
  pushes another `Resume NUC <number> <Title>` claim, so a long agent run
  cannot let the NUC go stale. A task that resumes a Blocked or paused NUC,
  including one the user names, claims it the same way before it starts.
- **Yield.** Another session's claim takes over a NUC the task holds only when
  it came more than 24 hours after the task's own latest allocation or held
  claim; a sooner one lost a race, and its session has left the NUC. When a
  pull shows a takeover, or the NUC's outcome already on `main`, the task
  records nothing for that NUC, leaves its candidate isolated, reports it and,
  in continuous mode, moves on to the next cycle.
- **Record nothing else.** A claim changes no file and records no outcome, so
  it needs no evaluation of its own; the outcome commit still carries the
  final version.

## Choose execution mode (caller only)

`/nuc-proposal` defaults to continuous mode for open-ended proposal requests.
If the user requests one specific NUC by number or path, one concrete feature,
or a particular revision, use single-proposal mode instead. Complete that
proposal's preparation, independent evaluation, and outcome recording, then
stop; do not select or start another proposal unless the user explicitly
requests continuation.

Preserve this mode through evaluation, including coordination through
`/nuc-evaluate`, and pass `NUC_PROPOSAL_MODE=single` explicitly for one-shot
requests. An omitted evaluator mode remains `continuous` for existing
open-ended invocations.

Every outcome follows the
[automatic acceptance rule](../../rules/proposals.instructions.md#automatic-acceptance-caller-only).
A candidate that has no compile errors (build, warning-denied Clippy and
formatting), passes the full `cargo test` with every existing test unweakened,
and is backward compatible with all accepted syntax and behavior is accepted
whatever the policy checks found: `Accepted` when every policy check passes,
`Automatic-Accepted` when one fails (for example a token increase or an
evaluator rejection), with the policy results recorded in the NUC's
documentation. Only a failed hard criterion rejects a NUC.

A NUC that `/arc-purposal` requests is a single-proposal request whose outcome
waits for the puzzle's solver run; when the hard criteria pass, an exact match
makes it Accepted regardless of the evaluator's decision. Follow the
[ARC-requested proposal rule](../../rules/proposals.instructions.md#arc-requested-proposals-caller-only).

A NUC the user requests directly is Accepted when its frozen candidate meets
every hard criterion, whatever the evaluator decides; a failed hard criterion
rejects it. Follow the
[user-requested proposal rule](../../rules/proposals.instructions.md#user-requested-proposals-caller-only).

The **token non-regression gate** requires each compared case to use the same
or fewer lexer tokens than its accepted baseline. Single-proposal
requests always skip token comparison. All modes retain valid immutable
accepted-corpus inputs, behavior/integrity, syntax compatibility, development
checks, independent evaluation, and outcome recording.

Only continuous cycles apply the
[NUC 1 token policy](../../../docs/nuc-0001.md#proposal-evaluation):

- Syntax enhancement, Utility enhancement, and Reasoning enhancement: run.
- Runtime features: run for an existing comparable feature; do not run at all
  for a brand-new feature; explicitly skip a genuinely uncomparable existing
  feature with a reason and supporting evidence.
- Language compatibility: do not run; additive cross-language aliases and
  equivalent familiar source forms may add lexer tokens or new token kinds.
- Documentation: do not run.

Language compatibility is a categorical skip, not advisory token scoring,
a zero-case comparison, or a passing token gate. Single mode takes precedence
over every category's continuous policy. Every category/mode retains valid
immutable baselines, accepted-source replay, behavior/integrity, compatibility,
the full `cargo test`, build, warning-denied Clippy, and formatting checks.

Supply explicit runtime applicability for Runtime features even in single mode.
The independent evaluator verifies any uncomparable rationale and evidence;
missing comparison artifacts are not evidence of impossibility. No skip may
hide invalid inputs/baselines, behavior or syntax failures, or a required token
increase.

## Classify every proposal

Give every new and existing NUC exactly one `Category` header from the
[proposal category registry](../../../docs/nuc-0001.md#proposal-categories),
including Draft, Accepted, Rejected, Active, and Superseded documents.
Category describes the proposal's purpose; it does not replace `Type` or
change its status.

Language compatibility covers proposals primarily intended to add optional
cross-language aliases or equivalent familiar forms while preserving every
accepted form and meaning. The language-superset aspiration is a goal, not
existing full compatibility; familiar prior art alone does not reclassify
other grammar, runtime, utility, or reasoning proposals.

Use the user's explicit classification when supplied. Whenever a category is
added, selected, or updated, the coordinating caller must ask the user for
confirmation if anything is uncertain, including its meaning, automatic-selection
eligibility, or which proposals belong to it. Ask before proceeding, not after
guessing. Workers return the question to the caller. This clarification also
applies during continuous cycles.

The user may add categories in the future. Every category addition requires
reviewing all existing NUCs, across every status, and reclassifying proposals
that fit the new category more accurately. Keep unaffected classifications,
`Type`, `Status`, and historical outcomes unchanged. Synchronize the registry,
affected proposal headers, and both category and numerical indexes through the
normal proposal/evaluation process. Ask for confirmation on uncertain moves;
do not invent categories or retain stale classifications without review.

## Select an unspecified enhancement

When asked to choose a language enhancement without a specific feature or
category, the coordinating caller makes one unbiased random draw with equal
probability across the registry's automatically selectable enhancement
categories. The current five enhancement categories each have a 20% chance;
`Documentation` is not part of this automatic language-enhancement draw.

Read the current registry before drawing; do not hard-code its size or retain
the former 50/50 syntax/runtime split. For example, populate `$categories`
from the eligible registry entries and use
`Get-Random -Minimum 0 -Maximum $categories.Count` to select an index.
Choose one bounded enhancement in that category; do not redraw to favor a
preferred category or substitute deterministic alternation for randomness.

Select before recording baselines or delegating preparation. Supply the
selected category, requested change, and a brief selection rationale to the
proposal agent, which must not redraw. Include the category and rationale in
its handoff and write the same category in the proposal header. Explicit
feature/category requests take precedence, and process or documentation-only
revisions are not randomized.

Selection does not bypass shared behavior, integrity, compatibility, or
development safeguards. Token requirements follow execution mode and the
confirmed category/applicability policy, not a new random draw.

## Pass the hint

`/nuc-proposal [request] [--hint <text>]` takes an optional hint: everything
after `--hint` is the user's guidance for the proposal, such as a syntax idea,
prior art, or an area to explore. The caller passes it unchanged to each
proposal agent it launches, in every continuous cycle of the invocation.

A hint is not a request. It never makes an open-ended request specific or
changes the execution mode, category draw, runtime applicability, or selected
strategy; to request a feature or category, state it as the request. Within
the confirmed request and category, the hint may guide which bounded
enhancement the caller chooses and how the proposer drafts it. Set aside any
part that conflicts with them or with a rule here, and report it. A hint never
relaxes a baseline, check, gate, or acceptance rule, and following it is not
an acceptance criterion.

## Confirm the inputs

Require the proposal path, confirmed category and requested change, execution
mode and runtime applicability with supporting rationale when applicable,
accepted Git revision, immutable baseline JSON artifact, and candidate worktree.
For a new NUC, the accepted revision is the pushed
[allocation](#allocate-the-nuc-first-caller-only), or the pushed claim of a
[picked-up Draft](#pick-up-stale-drafts-first-caller-only), and the candidate
starts from its Draft. Confirm that the baseline belongs to that accepted revision
before making runtime or executable document edits. Missing inputs stop
preparation, not justify assumptions.

For advisory usefulness scoring, the caller freezes a create-once session
rubric and its full Git blob pin **before the proposer drafts** under
[NUC 1](../../../docs/nuc-0001.md#advisory-usefulness-and-evaluation-cost).
Require the supplied rubric/pin when scoring is requested; preserve its
criterion, proposal path, and accepted-baseline binding through the handoff.
Do not alter the rubric or assess your own usefulness/independent checks.
Missing optional usefulness evidence is legitimately unscored, not zero;
acceptance checks and token policy are unchanged. Scoring alone reports
experience; optional category-conditioned UCB1 learning consumes verified
rewards under the separate protocol below, never trains LLM weights.

## Select and consume a proposal strategy

Follow [NUC 1's strategy policy](../../../docs/nuc-0001.md#reward-driven-proposal-strategies)
and the [callable commands and schemas](../../../CONTRIBUTING.md#reward-driven-proposal-strategies).
Category selection is still caller-owned and uniformly random when unspecified;
explicit requested categories/features and the single-proposal stop rule are
unchanged. A strategy selects an approach within that category, not another
feature or category.

For configured learning, the **caller must actually run**
`cargo test --test proposals select_proposal_strategy -- --ignored --exact --nocapture`
**after the [allocation](#allocate-the-nuc-first-caller-only), or the claim of
a [picked-up Draft](#pick-up-stale-drafts-first-caller-only), and before the
proposer drafts the details or is dispatched**. Use the latest verified
single-owner checkpoint (or an explicit frozen empty-history genesis), not an
older head or fabricated history. Fix `NUC_PROPOSAL`, accepted
`NUC_PROPOSAL_BASELINE`, preserved `NUC_PROPOSAL_MODE`/runtime inputs,
`NUC_STRATEGY_CATEGORY`, unique `NUC_STRATEGY_TRIAL_ID`, pinned
`NUC_STRATEGY_POLICY`/`NUC_STRATEGY_POLICY_REVISION` and fresh
`NUC_STRATEGY_SELECTION`; unset any old selection pin/checkpoint output.
After success, freeze `NUC_STRATEGY_SELECTION_REVISION` using
`git hash-object --no-filters` before the proposer drafts. Do not select again
in the worker, backfill attribution, or silently reset missing/invalid state.

Inside the proposal agent, require both policy/receipt paths and raw full blob
pins when learning is configured. Verify them and their algorithm, parent,
trial ID, proposal path, accepted baseline, confirmed category and mode against
the caller's inputs. Partial, stale, changed, or mismatched inputs stop work;
return the blocker instead of dropping learning. With no learning configuration,
record `not_configured`/unattributed and proceed with the ordinary safeguards.
Use no invented reward or self-assessed benefit.

**Execute the selected approach**, in stable registry order:

- `reuse_first`: start from accepted primitives/helpers and implement the
  requested feature with a minimal additive extension.
- `evidence_first`: start from a concrete unsupported/failing use case or
  counterexample for the requested feature, then make the minimal preserving
  fix/extension.

Both preserve all baseline, behavior/integrity, compatibility, development,
independent-evaluation and applicable token requirements. Neither changes the
feature scope, category, rewards or acceptance criteria. Include concrete
preparation actions showing strategy consumption in the handoff; merely logging
a label is insufficient. Never run `learn_proposal_strategy` on your own trial,
write its real assessment/report/checkpoint, or populate an independent score.
Both single and continuous genuinely attributed trials may later learn, but
only the independent evaluator may update from this revision's saved report.

When token comparison is required, new measured cases need behavior-equivalent
accepted-syntax programs with the same document, title, canonical assertions,
and expected return. They must
already have been executed by the accepted runtime and included through
`NUC_PROPOSAL_EXTRA_BASELINES` when recording the baseline. If a case is not
paired, return its required baseline to the caller before proceeding.
For legitimate skipped comparisons, genuinely new cases need no accepted-syntax
counterpart; execute their assertions or explicit expected returns and verify
coverage. Protect every pre-existing case, and supply the original extra-baseline
sources for legacy replay in every mode. Never overwrite a baseline, invent a
counterpart, or derive before-counts from the candidate lexer.

## Preserve syntax compatibility

Always preserve backward compatibility with previously accepted Nucleoid
syntax, for both syntax improvements and runtime features. Existing supported
forms must retain their parsing and runtime semantics. New syntax is additive
and optional; shorter spellings never replace or disable the existing forms.
Do not remove or reinterpret accepted syntax, or introduce keyword or grammar
collisions that break existing programs.

Keep representative legacy forms documented and covered by regression tests.
Run unchanged accepted-syntax counterparts from the pinned Git revision and
the supplied extra-baseline sources through the candidate parser and runtime.
Verify their protected assertions, explicit expected returns and executed
assertion coverage alongside the candidate forms.

Include compatibility-check results in the handoff. Resolve any known syntax
compatibility break before declaring the candidate ready for evaluation;
token savings or new functionality never justify a breaking syntax change.

## Draft and prototype

Complete the caller's allocated Draft of a new NUC, keeping its number and
confirmed category, or revise an existing NUC, using the required format and
sections. Trace implemented claims to `nucleoid.spec.md`; draft claims do not
override the accepted language. Keep dependencies already accepted and do not
combine pending proposals.

Implement the candidate change where needed,
then rewrite applicable cases in `nucleoid.spec.md`, `tests/reference.md`, and
the `nuc` blocks in `README.md`
and `docs/examples.md`. Preserve case identities, assertion order and canonical
assertions, expected returns, and executed assertion coverage. Do not remove
cases, weaken assertions, replace them with tautologies, or bypass branches.
A process-only proposal needs no artificial code or case rewrite.

Keep affected reference and synthesized examples synchronized. Edit source
documents, never generated JSONL or exported Rust tests. When their sources
change, regenerate using `UPDATE_DATASET=1 cargo test --test dataset` and
`UPDATE_SPEC_TESTS=1 cargo build`.

## Update the grammar

Keep `docs/nucleoid.g4`, the ANTLR4 syntax reference taken from the lexer and
parser, in step with every candidate. When a proposal changes what the lexer
or parser accepts, such as a token, keyword, alias, contextual word, rule,
lookahead, lowering, limit or layout rule, update the affected rules and
implementation-contract comments in the same candidate. Keep valid ANTLR4
notation and syntax-tree node comments, and check each changed rule against
the candidate parser. A proposal that leaves the accepted syntax unchanged
leaves the file as it is. Either way, the handoff states the grammar change or
why there is none.

## Check and hand off

Run the affected development tests, `cargo fmt --check`, and
`cargo clippy --all-targets -- -D warnings`. Fix feature-coupled problems in
the candidate, without changing the baseline or caller work.

The caller completes the required Rust improvement pass before the final
frozen handoff. If that pass is pending, return a preparation checkpoint and
await the caller's same-revision follow-up to recheck and finalize; a checkpoint
is not a ready-for-evaluation handoff.

Do not run the ignored `evaluate_proposal` entry point, produce an acceptance
report, or mark the proposal Accepted or Rejected. Those belong to the
independent evaluator.

Return the proposal path and Git blob revision (`git hash-object` on the
proposal), accepted baseline revision and path, execution mode, confirmed
proposal category and selection rationale when applicable, runtime applicability
and supporting rationale, any hint and how it was used or set aside, candidate
worktree, changed files, the `docs/nucleoid.g4` change or why there is none,
development and syntax-compatibility checks, and explicit blockers. State the
expected token policy, without claiming an unevaluated gate passed. Freeze the
candidate after handoff.
Include the caller-frozen rubric path and Git blob pin when supplied, without
claiming a usefulness score; only the independent evaluator may create the
assessment after its checks. Keep scoring artifacts outside the repository.
The caller starts a fresh `nuc-evaluate` agent with this handoff and
a fresh report path; do not launch it yourself. The caller owns integration
and evaluation status/index updates. The proposal agent must not commit, push,
or merge.

For configured learning, also hand off the unchanged policy and selection
paths/raw pins, algorithm/scope, trial identity, selected strategy and how it
was followed. Preserve all four `NUC_STRATEGY_POLICY`,
`NUC_STRATEGY_POLICY_REVISION`, `NUC_STRATEGY_SELECTION` and
`NUC_STRATEGY_SELECTION_REVISION` inputs into evaluation. Unconfigured trials
are explicitly unattributed; do not retrofit old reports. Only new external
session checkpoints may be written after independent evaluation, never old
policy or candidate sources.

## Name outcome commits (caller only)

Give every commit that records a NUC outcome, Accepted, Automatic-Accepted or
Rejected, the subject line `NUC <number> <Title>`, using the proposal's `Title`
header verbatim, as in `NUC 57 Thin Arrows and Python-Style Lambdas`. State the
outcome, its basis, the hard-criteria evidence and, for Automatic-Accepted,
every failed policy check in the body. Follow the
[outcome commit message rule](../../rules/proposals.instructions.md#outcome-commit-messages-caller-only).

This caller is the only flow that commits and pushes NUC allocations, claims
and outcomes. It makes every
[allocation commit](#allocate-the-nuc-first-caller-only),
`Draft NUC <number> <Title>`, and every
[claim commit](#pick-up-stale-drafts-first-caller-only),
`Resume NUC <number> <Title>`, neither of which records an outcome, and every
NUC outcome commit, including those of NUCs that `/arc-purposal` requests and of cycles
coordinated through `/nuc-evaluate`, and pushes each right after, pulling
first, under the
[git synchronization policy](../../rules/git.instructions.md). No worker agent
or `/nuc-evaluate` commits or pushes; `/arc-purposal` commits and pushes only
its own ARC changes, separately, once it is done.

## Record rejected outcomes (caller only)

When the frozen candidate fails a hard criterion of the
[automatic acceptance rule](../../rules/proposals.instructions.md#automatic-acceptance-caller-only),
the coordinating caller must record and commit the rejected proposal
documentation and its index/outcome metadata. An evaluator's `Rejected`
decision alone does not reject a candidate that meets every hard criterion;
record that NUC as Automatic-Accepted instead.
Rejected proposals are not exempt from commits. Follow the
[shared rejection workflow](../../rules/proposals.instructions.md#record-rejected-outcomes-caller-only).

Always include the rejection reason wherever the rejection is recorded: one
short statement of the failed hard criterion, such as the compile error, the
failing test or the compatibility break with its minimal reproducing program.
For a new proposal, state it in the Abstract's
rejection bullet, in both NUC index entries as `Rejected: <reason>`, and in the
rejection commit message; keep the full evidence in the proposal's evaluation
outcome. For a rejected revision of an accepted or active NUC, keep its status
and index entries, and give the reason in its outcome metadata and the commit
message. Report the same reason to the user.

Commit only rejection documentation, never the rejected candidate's language
changes or session baseline/report artifacts. Keep implementation changes in
the isolated candidate; stage explicit proposal/index paths, use a path-limited
commit, and verify its file list. Preserve existing accepted NUC text and
unrelated staged and unstaged work. Proposal and evaluation agents must not
perform this caller-owned commit.

## Continue with the next NUC (caller only)

This section applies only in continuous mode. A single-proposal request stops
after its outcome under the [execution-mode rule](#choose-execution-mode-caller-only).

After a new NUC receives a final `Accepted`, `Automatic-Accepted` or
`Rejected` outcome, the coordinating caller must finish it before starting the
next cycle:

- **Accepted or Automatic-Accepted:** re-check the hard criteria on the
  integrated tree, integrate the unchanged accepted candidate, record
  status/index metadata with the policy results, and complete the authorized
  Git commit.
- **Rejected:** record and commit only rejection documentation and index/outcome
  metadata; leave rejected language changes isolated.

Immediately start another fresh "purpose a NUC" cycle without waiting for a
new user prompt. Repeat sequentially in the active agent session until the
user stops, pauses or redirects the work, the session ends, or a resource limit
is reached. A `Blocked` decision, failed outcome recording, or missing required
input pauses continuation; report the blocker instead of bypassing it.

Continuous cycles do not ask for confirmation before recording an outcome or
continuing, including when coordinated through `/nuc-evaluate`. The exception
is an uncertain proposal category: the caller asks the user under
[the classification rule](#classify-every-proposal) before assigning it.
Workers return that uncertainty to the caller. Use the existing selection
rules and verified inputs. For other missing prerequisites or blockers,
report them and pause without a question; do not invent inputs or bypass
checks or required tool approvals.

Each cycle pulls and first claims the lowest-numbered stale Draft under the
[stale-Draft rule](#pick-up-stale-drafts-first-caller-only). Only a cycle that
finds none chooses one bounded enhancement under the existing selection rule
and allocates its new NUC number with a pushed Draft under the
[allocation rule](#allocate-the-nuc-first-caller-only). Pin that claim or
allocation, the current verified accepted-language revision, not a rejected
candidate;
create fresh immutable baselines, an isolated candidate and report paths.
Launch fresh proposal and evaluation agents, and perform the required Rust
improvement pass when applicable. Never reuse
workers across cycles or combine pending proposals.

Use actual independent behavior/compatibility results and token measurements
when comparison ran as feedback for the next enhancement; report legitimate
skips rather than inventing counts. With configured strategy learning, retain
the independent evaluator's new verified checkpoint as the next head (or retain
the old head when learning legitimately skipped). Actually invoke
`select_proposal_strategy` from that head before dispatching the next proposer;
use fresh trial/receipt paths and never carry a stale selection pin. Preserve
earlier artifacts and all acceptance, compatibility and version-protection
rules. This is bandit strategy learning plus iterative feedback, not LLM-weight
training or full multi-step RL. Sparse/noisy rewards do not establish improved
proposal quality.

Only the caller continues the loop; proposal and evaluation workers still stop
at their single-proposal handoffs. Starting a distinct next NUC is not recursive
reevaluation of status/index metadata. Instruction-file edits alone do not
complete a NUC or start this loop, and no filesystem watcher or background
service is implied.

---
name: arc-debug
description: Investigate one finished ARC solver run log, arc/logs/<puzzle-id>.log.json, for abnormalities and report them back without changing any file.
argument-hint: <puzzle-id> [focus]
context: fork
agent: arc-debug
---

Investigate the ARC solver run log for puzzle $ARGUMENTS. The puzzle ID comes
first. Any text after it names a suspected problem to examine first. Follow
the [arc-debug agent profile](../../agents/arc-debug.agent.md). If that profile
is not already your system prompt, delegate this request to the `arc-debug`
agent.

The investigation is read-only, apart from the fast-forward pull of the
primary checkout that every agent makes before it starts, under the
[git synchronization policy](../../rules/git.instructions.md#pull-first):

- Read `arc/logs/<puzzle-id>.log.json` and the solver sources that explain it.
- Never edit, stage or commit a repository file, including the log, and
  never push.
- Never run the solver, an LLM call, or anything that reads `.env` or
  credentials.
- Never quote the expected test grid's values. Label findings that depend on
  it as scoring-only, and base suggested fixes on the training pairs only.

Report the profile's summary, the abnormalities ranked by severity with their
evidence and likely causes, the passed and skipped checks, and the suggested
next steps.

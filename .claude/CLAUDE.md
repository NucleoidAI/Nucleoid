# Nucleoid

Nucleoid is a next-gen Logic Programming Language focused on LLMs.

## Repository instructions

Read and follow every instruction file below before acting. This applies to
coordinating agents and delegated workers; this shared entry point is an index,
not a replacement for the policies it links to.

| File | Policy |
| --- | --- |
| [project.instructions.md](rules/project.instructions.md) | Language goals, reference documents, and runtime architecture |
| [arc.instructions.md](rules/arc.instructions.md) | ARC puzzle proposals, JSON-only LLM exchanges, and no automated ARC tests |
| [rust.instructions.md](rules/rust.instructions.md) | Concrete runtime types, dependencies, and Rust community conventions |
| [documents.instructions.md](rules/documents.instructions.md) | Authoritative specification, executable documents, and dataset synchronization |
| [versions.instructions.md](rules/versions.instructions.md) | Human-owned version numbers; no AI version changes |
| [proposals.instructions.md](rules/proposals.instructions.md) | Independent NUC proposal preparation, evaluation, token acceptance, ARC- and user-requested acceptance, and `NUC <number> <Title>` outcome commit messages |
| [rust-improvement.instructions.md](rules/rust-improvement.instructions.md) | Focused Rust improvements and the required post-change agent |

All instruction files use `applyTo: "**"` to preserve their repository-wide scope.

## Shared agent configuration

Claude Code and GitHub Copilot load the same files; keep one copy of each.

- This index: Claude Code reads it directly, and
  `.github/copilot-instructions.md` imports it for Copilot.
- Instruction files: `.claude/rules/`. Omit a `paths` field, which would limit
  a rule to matching files in Claude Code.
- Custom agents: `.claude/agents/`. Name tools as Claude Code does (`Read`,
  `Edit`, `Write`, `Grep`, `Glob`, `Bash`, `PowerShell`); Copilot accepts
  these names as aliases. Set `include-custom-instructions: true` so Copilot
  agents, like Claude Code subagents, receive these instructions.
- Skills: `.claude/skills/`. Fields such as `context: fork` and `agent` are
  Claude Code features, so write skill bodies that also work as plain
  instructions in Copilot.

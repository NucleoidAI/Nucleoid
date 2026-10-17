# Nucleoid

Nucleoid is a next-gen Logic Programming Language focused on LLMs.

## Repository instructions

Read and follow every instruction file below before acting. This applies to
coordinating agents and delegated workers; this shared entry point is an index,
not a replacement for the policies it links to.

| File | Policy |
| --- | --- |
| [project.instructions.md](instructions/project.instructions.md) | Language goals, reference documents, and runtime architecture |
| [arc.instructions.md](instructions/arc.instructions.md) | ARC solver proposals, JSON-only LLM exchanges, and no automated ARC tests |
| [rust.instructions.md](instructions/rust.instructions.md) | Concrete runtime types, dependencies, and Rust community conventions |
| [documents.instructions.md](instructions/documents.instructions.md) | Authoritative specification, executable documents, and dataset synchronization |
| [versions.instructions.md](instructions/versions.instructions.md) | Human-owned version numbers; no AI version changes |
| [proposals.instructions.md](instructions/proposals.instructions.md) | Independent NUC proposal preparation, evaluation, and token acceptance |
| [rust-improvement.instructions.md](instructions/rust-improvement.instructions.md) | Focused Rust improvements and the required post-change agent |

All instruction files use `applyTo: "**"` to preserve their repository-wide scope.

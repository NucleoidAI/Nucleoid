---
applyTo: "**"
---

# Human-owned version numbers

- No AI agent, including coordinating agents and delegated workers, may change the project's version number.
- This prohibition covers `Cargo.toml`'s `package.version`, Nucleoid's own package entry in `Cargo.lock`, and any other project release/version metadata, including changes made indirectly by tools or generated outputs.
- Version changes are human-owned. Report required semantic-versioning changes, including those for breaking public API changes, without editing or automatically bumping a version.

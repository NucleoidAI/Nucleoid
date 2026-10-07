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
| `dataset/nucleoid.spec.synth.*.md` | fine-tuning sources derived from the specification; not runtime tests |
| `dataset/*.jsonl` | rendered specification and fine-tuning data for publication |
| `docs/` | the language reference, as NUC documents in the style of a PEP |
| `docs/README.md` | prose reference; `tests/reference.md` is its executable form |
| `src/` | the runtime |
| `tests/` | executable specification, documentation, dataset, and runtime checks |

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

`nucleoid.spec.md`, the synthesized sources in `dataset/`, `docs/` and the crate
describe the same language, and a change to one belongs in the same change as
the others. `cargo test` checks the normative documents and verifies that the
fine-tuning dataset still matches its source documents; synthesized programs
are not executed as tests.

Edit the Markdown sources, not the generated JSONL. Regenerate the dataset and,
when the specification changes, its exported tests from the repository root:

```console
$ UPDATE_DATASET=1 cargo test --test dataset
$ UPDATE_SPEC_TESTS=1 cargo build
```

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

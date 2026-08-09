# Contributing

Thanks to declarative programming, we have a brand-new approach to data and logic. As we are still discovering what we can do with this powerful programming model, please join us with any types of contribution!

## Working on the Rust runtime

This repository holds the Rust implementation. The crate is at the root, and
`ref/` is the archived JavaScript implementation kept for reference.

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
| `synth/` | further cases, derived from the specification |
| `docs/` | the language reference, as NUC documents in the style of a PEP |
| `docs/README.md` | prose reference; `tests/reference.md` is its executable form |
| `src/` | the runtime |
| `tests/` | the suites that run the documents above |
| `ref/` | the archived JavaScript implementation — **frozen**, read it but never change it |

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
$ cargo test --test spec keeps_a_total
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

`nucleoid.spec.md`, `synth/`, `docs/` and the crate describe the same language,
and a change to one belongs in the same change as the others. `cargo test` is
what checks they still agree.

## Declarative Runtime Environment

Nucleoid is a declarative runtime environment that applies declarative programming at the runtime as rerendering JavaScript statements and creating the graph, so as a result, the declarative runtime system isolates a behavior definition of a program from its technical instructions and executes declarative statements, which represent logical intention without carrying any technical detail.

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

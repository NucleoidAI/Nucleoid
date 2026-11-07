---
applyTo: "**"
---

# Power of 10 for the Rust crate

Gerard J. Holzmann's "The Power of 10: Rules for Developing Safety-Critical
Code" (NASA/JPL, 2006) applies to the crate's production code: the library
(`src/lib.rs` and its modules) and the binary (`src/main.rs`, its
`src/interactive.rs` and `src/continuation.rs` modules, and `src/terminal/`).
Integration tests, examples and build scripts keep the repository's
warning-free default lint set.

Each rule is enforced by a lint, a runtime limit, or `tests/power_of_ten.rs`.
Review covers what tools cannot prove. Lints are declared `warn` in the crate
roots and denied in CI and the NUC hard criteria through
`cargo clippy --all-targets -- -D warnings`.

## 1. Simple control flow

- Production code does not use panics as control flow. These lints are
  enforced: `clippy::panic`, `unwrap_used`, `expect_used`, `unreachable`,
  `todo`, `unimplemented`, `exit`, `indexing_slicing`, `string_slice` and
  `panic_in_result_fn`. Return a typed `Error`, or use `get`, iterators,
  slice patterns or fixed-size destructuring such as ratatui's `areas`.
- Recursion over syntax follows the grammar and is always bounded. Every
  recursive path passes a depth guard (`MAX_NESTING`, `MAX_CHAIN`,
  `MAX_DEPTH`) that returns an error before the host stack can overflow. Reach
  a new recursive function only through such a guard, or write it as a
  worklist, as propagation is.
- Known gap: values built at run time are not depth-bounded yet. A list nested
  about 25,000 deep overflows the release binary's stack when it is rendered,
  compared, cloned or dropped. Closing the gap changes language behaviour, so
  it needs a NUC.

## 2. Bounded loops

- Prefer `for` over finite collections and ranges.
- Every `loop` and `while` has a bound a reader can state. Each iteration
  consumes input (tokens or characters), or walks a worklist that each item
  enters once, or counts against a limit: `MAX_PROPAGATION`, the 100,000
  iterations of the `while` statement, or the class count that caps
  `State::lineage`.
- `clippy::infinite_loop` is enforced. The REPL and terminal event loops are
  the only intentionally unbounded loops; they end on user input.

## 3. Dynamic memory (adapted)

- An interpreter allocates the values its programs create, so this rule is
  adapted. Input-driven growth is bounded by the limits above and by fallible
  reservation (`try_reserve_exact` for repeated and replaced strings).
- Avoid needless allocation: borrow instead of cloning, and use
  `clone_from` and `write!` where pedantic and `clippy::format_push_string`
  ask for them.

## 4. Functions fit on a page

- `clippy::too_many_lines` with `too-many-lines-threshold = 60` in
  `clippy.toml`.
- Split long dispatchers into named helpers by category. Keep each `match`
  over a closed enum exhaustive rather than hiding variants behind `_`.
- Source files stay small too: every production file under `src/` has at
  most 500 lines, which `tests/power_of_ten.rs` checks. A unit-test module
  lives out of line in its module's `tests.rs` (`#[cfg(test)] mod tests;`),
  which the limit exempts, as the crate roots exempt unit tests from the
  rule 4 lint.
- Split a file that outgrows the limit into child modules by
  responsibility, as `parser.rs` groups its `impl Parser` methods into
  `parser/statement.rs`, `parser/expression.rs` and their siblings. Child
  modules stay private, the parent re-exports what was public so public
  paths do not change, and a helper widens only to `pub(super)` when a
  sibling needs it.

## 5. Assertions

- Check invariants that must hold with side-effect-free `debug_assert!`
  (`clippy::debug_assert_with_mut_call`). Examples are balanced depth
  counters, stack and queue bookkeeping, savepoint marks and index
  invariants. They run on every `cargo test` and compile out of release
  builds.
- When public input can reach a violation, recover explicitly by returning a
  typed error, as the AST wrappers do for a node of the wrong kind.
- Do not add assertions that cannot fail (`clippy::assertions_on_constants`).

## 6. Smallest scope

- Declare data where it is used.
- Do not rebind a name to an unrelated value (`clippy::shadow_unrelated`).
- Keep items private or `pub(crate)` unless they are API (`unreachable_pub`).

## 7. Checked results and parameters

- Mark value-returning functions `#[must_use]` (`clippy::must_use_candidate`,
  `return_self_not_must_use`).
- Never discard a `#[must_use]` value with `let _ =`
  (`clippy::let_underscore_must_use`) or `.ok()` (`clippy::unused_result_ok`).
  A deliberate discard names its type, `let _: T = ...`
  (`clippy::let_underscore_untyped`).
- Validate parameters with typed errors. Keep the numeric conversions of the
  language's number model in `src/number.rs`, the only place that casts
  between `f64` and integers.

## 8. Limited metaprogramming

- No `macro_rules!` in `src/`. Use the derive macros of crates already in
  `Cargo.toml`.
- Conditional compilation is limited to `cfg(test)` and the `cfg!(windows)`
  key binding in the terminal.

## 9. Restricted indirection

- `unsafe_code = "forbid"` in `Cargo.toml`.
- No trait objects (`dyn`), function pointers or `Rc`/`RefCell`/`Cell` in
  `src/`. Dispatch through enums and `match`; pass behaviour as a statically
  dispatched generic `impl Fn` when needed.

## 10. Zero warnings, analysed daily

- The production crates enable `clippy::pedantic`, the restriction lints
  above and strict rustc lints (`missing_debug_implementations`,
  `rust_2018_idioms`, `unused_qualifications`, `trivial_casts`,
  `trivial_numeric_casts`, `single_use_lifetimes`, `unused_lifetimes`,
  `unused_import_braces`, `unreachable_pub`).
- CI denies warnings from Clippy on all targets and from rustdoc
  (`cargo doc --no-deps --lib --document-private-items`), and runs daily as
  well as on every push and pull request.
- Local exceptions are `#[expect(...)]` attributes, never `#[allow]`, and only
  for deliberate cases: the conversions in `src/number.rs` and the CLI flag
  struct. The only crate-wide allowance is `clippy::missing_errors_doc` on the
  library.

## Maintenance

- Keep the lint lists in `src/lib.rs` and `src/main.rs` identical;
  `tests/power_of_ten.rs` checks them, the 60-line threshold, the 500-line
  file limit, and rules 8 and 9.
- Unit tests relax only the rule 1, 4 and 6 lints, through
  `cfg_attr(test, allow(...))` in the crate roots.
- NUC candidates and every other change meet these rules like any other
  code: they are part of the warning-denied Clippy and full `cargo test` hard
  criteria.

---
applyTo: "**"
---

# Rust guidelines

The Rust crate (`Cargo.toml`, `src/`, `tests/`, `examples/`, `build.rs`)
always follows the published principles below. They apply to every Rust
change by every agent: features, fixes, refactors, NUC candidates and the
[rust-improver](../agents/rust-improver.agent.md) pass.

| Source | Governs |
| --- | --- |
| [The Power of 10](https://spinroot.com/gerard/pdf/Power_of_Ten.pdf) | Control flow, bounds, assertions, checked results and zero warnings, adapted in [power-of-ten.instructions.md](power-of-ten.instructions.md) |
| [Rust API Guidelines](https://rust-lang.github.io/api-guidelines) | The public API: naming, traits, type safety, dependability and future proofing |
| [Google Rust Style Guide](https://google.github.io/styleguide/rustguide.html) | Code organization, imports, ownership, signatures, traits and implementation style |
| [Microsoft Pragmatic Rust Guidelines](https://microsoft.github.io/rust-guidelines/) | Universal, library, correctness, performance and AI-facing design |
| [Clippy](https://clippy.rust-lang.org) | The lints that enforce much of the above |

## How they apply

- Every applicable guideline is required, not advisory. Deviate only where a
  higher-ranked rule requires it, as recorded under
  [Deliberate divergences](#deliberate-divergences).
- The sections below digest each source for this crate. The source stays the
  full text, and an item the digest leaves out still applies.
- Precedence, highest first:
  1. The user's explicit instructions and the other repository instruction
     files.
  2. The [Power of 10 adaptation](power-of-ten.instructions.md).
  3. The Rust API Guidelines, which Google and Microsoft both build on.
  4. The Microsoft and Google guidelines. Google's product-specific rules
     (`[chrome]`, `[Fuchsia]`, per-codebase formatters and test frameworks)
     do not apply.
- Library API rules bind everything reachable from `src/lib.rs`; the binary
  (`src/main.rs`, `src/interactive.rs`, `src/continuation.rs`,
  `src/terminal/`) follows the rest.
- Apply the guidelines to new and touched code. Bring older code in line
  incrementally under the
  [Leave the world better](rust-improvement.instructions.md) policy, never
  through unrelated rewrites. Report, rather than make, a conformance fix
  that would break the public API; versions stay
  [human-owned](versions.instructions.md).
- Lints enforce what they can and review covers the rest. These guidelines add
  no NUC hard criterion beyond the warning-denied Clippy build that the
  [automatic acceptance rule](proposals.instructions.md#automatic-acceptance-caller-only)
  already requires.

## Rust API Guidelines

- Naming: RFC 430 casing (C-CASE); `as_`, `to_` and `into_` conversions with
  the matching `self` (C-CONV); getters without `get_` (C-GETTER); `iter`,
  `iter_mut` and `into_iter`, with iterator types named after them (C-ITER,
  C-ITER-TY); a consistent word order (C-WORD-ORDER); feature names without
  placeholder words (C-FEATURE).
- Interoperability: eagerly implement the common traits that fit (`Clone`,
  `Copy`, `PartialEq`, `Eq`, `Hash`, `PartialOrd`, `Ord`, `Debug`, `Display`,
  `Default`) (C-COMMON-TRAITS); convert through `From`, `TryFrom`, `AsRef`,
  `AsMut` and `FromStr` (C-CONV-TRAITS); collections implement
  `FromIterator` and `Extend` (C-COLLECT); types are `Send` and `Sync` where
  possible (C-SEND-SYNC, M-TYPES-SEND); errors are meaningful and implement
  `std::error::Error` (C-GOOD-ERR); generic readers and writers are taken by
  value (C-RW-VALUE).
- Predictability: constructors are static inherent functions (C-CTOR);
  functions with a clear receiver are methods (C-METHOD); no out-parameters
  (C-NO-OUT); operator overloads keep their usual meaning (C-OVERLOAD); only
  smart pointers implement `Deref`, and they add no inherent methods
  (C-DEREF, C-SMART-PTR); conversions live on the most specific type
  (C-CONV-SPECIFIC).
- Flexibility: expose intermediate results (C-INTERMEDIATE); let the caller
  decide where data is copied (C-CALLER-CONTROL); minimize assumptions with
  statically dispatched generics (C-GENERIC).
- Type safety: newtypes for distinct values (C-NEWTYPE); enums and dedicated
  types rather than `bool` or `Option` arguments (C-CUSTOM-TYPE); bit-flag
  types for flag sets (C-BITFLAG); builders for complex values (C-BUILDER).
- Dependability: validate arguments with typed errors (C-VALIDATE);
  destructors never fail or block (C-DTOR-FAIL, C-DTOR-BLOCK).
- Debuggability: every public type has a non-empty `Debug` (C-DEBUG,
  C-DEBUG-NONEMPTY), which `missing_debug_implementations` enforces.
- Future proofing: private struct fields (C-STRUCT-PRIVATE); newtypes hide
  their representation (C-NEWTYPE-HIDE); sealed traits where downstream
  implementations must not exist (C-SEALED); no derived bounds repeated on
  data structures (C-STRUCT-BOUNDS); implementation details stay private or
  `pub(crate)`, out of rustdoc (C-HIDDEN).
- Necessities: public dependencies are stable (C-STABLE); the crate and every
  dependency carry permissive licenses compatible with Apache-2.0
  (C-PERMISSIVE); `Cargo.toml` keeps complete metadata (C-METADATA). Release
  notes (C-RELNOTES) are human-owned like versions: report the changes that
  need them.

## Google Rust Style Guide

- Formatting: default `cargo fmt`; `#[rustfmt::skip]` only for manual
  alignment or generated code.
- Organization: name files and leaf modules after their primary item, and
  keep new implementations out of non-leaf modules. A `use` imports from one
  module. No glob imports, except `use super::*` in test modules and
  `use Enum::*` in enum-focused code, and no preludes (M-NO-PRELUDE). Scope a
  trait import to the function that needs it. Call top-level functions
  through their module (`mem::take`) and use type names unqualified unless
  ambiguous (`fmt::Result`, `io::Error`).
- Ownership: unique ownership over `Rc` and `Arc`; borrow rather than clone.
- Signatures: accept `&str`, `&[T]`, `&T` and `Option<&T>`, never `&String`,
  `&Vec<T>`, `&Box<T>` or `&Option<T>`. Return an iterator rather than a
  `Vec`, and accept `impl IntoIterator`, when the caller or callee only
  iterates. Use `impl Into<T>` sparingly. Return `Option` for plain absence
  and the typed `Result` for recoverable failure.
- Types: a newtype rather than a type alias when the meaning differs; named
  fields rather than tuples, except single-field newtypes.
- Traits: implement the standard trait rather than a look-alike method
  (`should_implement_trait`) and derive whenever possible. Define a trait
  only for generic use or as an extension trait named `...Ext`. Pair an
  important `From` with an explicit constructor. A type with `new()` and an
  options struct implement `Default` (`new_without_default`).
- Naming: descriptive lifetime names beyond one or two short input/output
  lifetimes; constructors `new`, `new_with_*` and `try_new`; accessors `foo`,
  `foo_mut`, `set_foo`, `take_foo`, `replace_foo` and `with_foo`; no
  abbreviations a newcomer would not know.
- API surface: `len` comes with `is_empty`; `const fn` only when it will stay
  `const`; `#[must_use]` where ignoring a result is a misuse, as Power of 10
  rule 7 already requires. New public enums, and new public structs with
  public fields, that may grow are `#[non_exhaustive]`; adding it to an
  existing type breaks the API.
- Implementation: annotate the types a reader needs, especially around
  `.into()`, `.parse()` and `collect()`. Pattern-match rather than check and
  unwrap. Prefer a plain `for` loop to a long or stateful combinator chain or
  `for_each`. Build strings with `format!` or `write!`. Never call
  `catch_unwind` (M-PANIC-CONTINUATION), run code before `main`, or use
  unstable features.

## Microsoft Pragmatic Rust Guidelines

- Universal: lint overrides are item-scoped `#[expect]`, never `#[allow]`
  (M-LINT-OVERRIDE-EXPECT). Public types are `Debug`, and those meant to be
  read are also `Display` (M-PUBLIC-DEBUG, M-PUBLIC-DISPLAY). Names are short
  and free of weasel words such as `Manager`, `Service` or `Factory`
  (M-SHORT-NAMES, M-WEASEL-WORDS). Associated functions construct instances;
  other receiver-less computation is a free function (M-REGULAR-FN). Magic
  values are named constants (M-DOCUMENTED-MAGIC).
- Library interface: concrete types first, then generics (M-DI-HIERARCHY),
  with no visibly nested type parameters (M-SIMPLE-ABSTRACTIONS) and no smart
  pointers or wrappers in public signatures (M-AVOID-WRAPPERS). Accept
  `impl AsRef<T>`, `impl RangeBounds<T>` and `Read`/`Write` where feasible
  (M-IMPL-ASREF, M-IMPL-RANGEBOUNDS, M-IMPL-IO). Prefer std types to leaked
  third-party types, and re-export no foreign items (M-DONT-LEAK-TYPES,
  M-FOREIGN-REEXPORTS). Give each item one public path, re-export items
  individually and mark crate re-exports `#[doc(inline)]`
  (M-SINGLE-ITEM-PATH, M-NO-GLOB-REEXPORTS, M-DOC-INLINE).
- Library behavior: core logic is inherent and trait impls forward to it
  (M-ESSENTIAL-FN-INHERENT). Convert errors with `From` and `?`, using
  `map_err` only to add context (M-FROM-ERROR). Modules are balanced and
  grouped by use case (M-BALANCED-MODULES), and shared parameters keep one
  order (M-PARAMETER-CONSISTENCY). Collections implement the iterator traits
  with a truthful `size_hint` (M-COLLECTION-TRAITS). Four or more
  initialization permutations get a builder with infallible setters and a
  validating `build()` (M-INIT-BUILDER, M-BUILD-RESULT), and long parameter
  lists group into semantic helper types (M-INIT-CASCADED).
- Resilience: use the strongest fitting std type early, such as `Path` for
  paths (M-STRONG-TYPES); newtypes guard their invariants through fallible
  constructors (M-STRONG-TYPES-GUARD); no statics whose shared state affects
  correctness (M-AVOID-STATICS). Clock, entropy and I/O access that the
  library adds can be replaced by the caller (M-MOCKABLE-SYSCALLS). The
  library never prints; console output belongs to the binary
  (M-LOG-NOT-PRINT). Public-API tests live in `tests/` (M-INTEGRATION-TESTS),
  and test utilities stay behind `cfg(test)` or a `test-util` feature
  (M-TEST-UTIL).
- Building: the crate builds and tests on Tier 1 platforms with `cargo` alone
  (M-OOBE), features are additive (M-FEATURES-ADDITIVE), and it stays on
  edition 2024 with a conservatively raised `rust-version`
  (M-LATEST-EDITION, M-MSRV).
- Correctness: all code is sound (M-UNSOUND), and a panic means "stop the
  program", never an error channel (M-PANIC-IS-STOP). Detected bugs follow
  the [divergence](#deliberate-divergences) below.
- Performance: identify and measure hot paths before optimizing them
  (M-HOTPATH); `examples/scaling.rs` measures propagation's linear cost.
  Reuse allocations, size collections up front and shrink long-lived ones
  (M-MEM-REUSE, M-INITIAL-CAPACITY, M-SHRINK-TO-FIT, M-BOX-DST), and avoid
  needless indirection (M-AVOID-INDIRECTION). A faster hasher needs a
  measurement and keys the running program cannot choose (M-FAST-HASHER).
- AI: design idiomatic, strongly typed, testable APIs (M-DESIGN-FOR-AI).
  Write Rust-shaped code, not a transliteration of JavaScript or another
  language (M-RUST-SHAPED). Tests check behavior and properties, not restated
  constants or mirrored implementation logic (M-TAUTOLOGICAL-TESTS).
  Documentation describes the end state, not the design process
  (M-NO-META-DESIGN-DOCUMENTATION).

## Clippy

- The default groups (`correctness`, `suspicious`, `style`, `complexity`,
  `perf`) are never allowed crate-wide, and `clippy::pedantic` is enabled in
  both crate roots.
- `restriction` lints are chosen one at a time, never as a group, which
  Clippy says "should, emphatically, not be enabled as a whole". The chosen
  set is the one the Power of 10 adaptation lists.
- Fix what a lint reports rather than silencing it, including new lints that
  a newer stable toolchain brings. A deliberate exception is an item-scoped
  `#[expect(...)]`; the only crate-wide allowance is
  `clippy::missing_errors_doc` on the library.
- Thresholds live in `clippy.toml`, and Clippy's `msrv` follows
  `rust-version`.
- The gate is `cargo clippy --all-targets -- -D warnings`, together with
  `cargo fmt --check` and warning-free rustdoc.

## Comments and documentation

- Rust source gains no comments: no `//` narration and no new rustdoc. Names,
  types, named constants and tests carry the intent, and behavior is
  documented in `nucleoid.spec.md`, `docs/`, `README.md` and
  `CONTRIBUTING.md`. `Cargo.toml` keeps its note on why each dependency was
  chosen.
- Existing comments and rustdoc stay, including the crate documentation in
  `src/lib.rs` and its runnable example (C-CRATE-DOC). A change that makes
  one wrong corrects or removes it. Examples use `?` rather than `unwrap`
  (C-QUESTION-MARK), links resolve (C-LINK), and rustdoc stays warning-free.

## Deliberate divergences

Each follows from a higher-ranked rule or awaits the user's decision.

- Detected bugs: production code has no panic paths (Power of 10 rule 1).
  M-PANIC-ON-BUG and Google's advice to panic on bugs become `debug_assert!`
  invariants, which stop tests and debug builds (rule 5), and typed errors
  wherever public input can reach the violation.
- Errors: the library keeps its `Error` type with a public `ErrorKind`, under
  the typed-error rule in [rust.instructions.md](rust.instructions.md),
  instead of M-ERRORS-CANONICAL-STRUCTS' backtrace-carrying structs with
  `is_xxx()` methods. The binary uses the same typed errors, not `anyhow`
  (M-APP-ERROR).
- Dispatch: no `dyn`, function pointers, `Rc`, `RefCell` or `Cell` in `src/`
  (rule 9). This settles M-DI-HIERARCHY's `dyn` tier, Google's code-size
  exception for `dyn` and C-OBJECT.
- Macros: no `macro_rules!` in `src/` (rule 8), so the macro guidelines of
  all three sources do not arise.
- Documentation: [Comments and documentation](#comments-and-documentation)
  replaces C-EXAMPLE, C-FAILURE, M-CANONICAL-DOCS, M-MODULE-DOCS,
  M-FIRST-DOC-SENTENCE, Google's doc-comment rules and the `reason` text of
  M-LINT-OVERRIDE-EXPECT for new code.
- Formatting: default rustfmt, not Google's suggested options such as
  `max_width = 100`.
- Awaiting the user's decision, because they change dependencies, deployment
  or the public API: mimalloc (M-MIMALLOC-APPS), `target-cpu`
  (M-TARGET-CPU), a workspace of smaller crates (M-SMALLER-CRATES,
  M-CARGO-WORKSPACE, M-CRATES-IN-WORKSPACE, M-CRATES-FLAT-FOLDER), and Serde
  implementations (C-SERDE).
- Not applicable while the crate forbids `unsafe`, runs synchronously and has
  no FFI, logging or services: the unsafe, FFI, `-sys`, `async`, throughput,
  logging and service guidelines, and C-NUM-FMT. They apply as written once
  that changes.

## Known gaps

- M-STATIC-VERIFICATION asks for more than CI enforces today: the rustc lints
  `ambiguous_negative_literals`, `redundant_imports` and
  `redundant_lifetimes`; the `clippy::cargo` group; Microsoft's individual
  restriction lints beyond `unused_result_ok`, apart from those that concern
  `unsafe` or need the `reason` text above; and cargo-audit, cargo-hack,
  cargo-udeps and Miri.
- The `Date` built-in reads the system clock directly
  (`src/builtins/date.rs`), which M-MOCKABLE-SYSCALLS would make replaceable.
- Closing a gap is a change of its own: add each lint to both crate roots,
  fix every hit, and keep `tests/power_of_ten.rs` passing. Do not fold it
  into unrelated work.

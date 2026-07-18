# Nucleoid (Rust)

A Rust port of the Nucleoid core runtime (the `src/` directory at the repo
root) — a sibling to the existing `cpp/` and `python/` ports. **This is a
working prototype, not a full parity port**, and it has not been compiled
yet (no Rust toolchain was available when it was written — see
[Status](#status) below).

## What this is

Nucleoid lets you write plain JS-looking statements that persist across
calls and react to each other, e.g.:

```
let a = 1;
let b = a + 1;   // 2
a = 10;
b;               // 11 — recomputed automatically
```

Rather than reimplementing a JS parser/interpreter from scratch, this port
embeds a real JS engine ([QuickJS](https://bellard.org/quickjs/) via the
[`rquickjs`](https://crates.io/crates/rquickjs) crate) and layers Nucleoid's
distinguishing feature — the reactive dependency graph — on top of it in
Rust. Top-level `let`/`const`/`var`/`function`/`class` declarations persist
naturally across separate `eval` calls within one JS realm, which is what
gives statements their session-like behavior; all JS control flow
(`if`/`for`/`function`/`class`) works for free because it really is JS
running in a real engine, not a hand-rolled walker.

## Architecture

| Module | Mirrors | Role |
|---|---|---|
| `engine.rs` | `state.js` | Persistent QuickJS context; eval, read, restore, event draining |
| `statement.rs` | `statement.js`, `stack.js` | Splits source into statements; classifies `name = expr` assignments |
| `graph.rs` | `graph.js`, `Expression.js` | Dependency graph + topological recompute on change |
| `transaction.rs` | `transaction.js` | Snapshot/restore state on error |
| `datastore.rs` | `cache.js` | In-memory history of processed statements |
| `event.rs` | `event.js` | `event(name, data)` records surfaced per statement |
| `runtime.rs` | `runtime.js` | Orchestrates the above into `process()` |
| `http.rs`, `main.rs` | `express.js`, `routes/terminal.js`, `bin.js` | `POST /` to run a statement, `GET /logs`, `GET /graph` |

## Known limitations vs. the JS implementation

This was scoped as a working prototype, not full parity. Deliberately out:

- **Dependency detection is text-based, not AST-based.** A variable's
  dependencies are found by regex-scanning its expression source for known
  identifiers (`statement::referenced_vars`), not by walking a real AST like
  `src/Expression.js` does. It won't see through helper function calls, and
  it can't track dependencies on object/array *properties* — only whole
  variables.
- **Only simple assignments are tracked reactively**: `let/const/var name
  = expr;` or `name = expr;`. Destructuring, compound assignment (`+=`),
  and member assignment (`obj.prop = x`) execute fine as JS but aren't
  wired into the dependency graph.
- **Statement splitting is brace/string-aware, not a real parser.** It
  handles sequential statements and `if`/`for`/`function`/`class` blocks,
  but will mis-split top-level destructuring patterns like `const {a, b} =
  obj;` (rare in Nucleoid usage, but worth knowing).
- **Rollback restores tracked variable values, not arbitrary mutations.**
  Like the original, it won't undo a mutation of a nested object/array's
  contents — only reassignment of a whole tracked variable.
- **No clustering, OpenAPI generation, config-file/`~/.nuc` loading, or
  native/extensions plugin loading** (`cluster.js`, `lib/openapi.js`,
  `config.js`, the plugin-loading half of `process.js`). The HTTP surface is
  a minimal stand-in for `routes/terminal.js`.
- Reassigning a `const`-declared variable during reactive recompute throws
  a JS `TypeError`, which surfaces as a normal error result — declare
  reactive/derived variables with `let`.

## Status

No Rust toolchain (`cargo`/`rustc`) was installed in the environment this
was written in, so **none of this has been compiled or run**. The `rquickjs`
API calls were cross-checked against the published docs for `rquickjs`
0.12.0, but some drift is possible. First step:

```sh
cd rust
cargo build
cargo test
```

Fix forward from whatever `cargo build` reports — the most likely friction
points are exact `rquickjs` method names/signatures in `src/engine.rs`
(exception introspection in particular) and the `axum` extractor/handler
setup in `src/http.rs`. A C compiler (clang/gcc) is required to build
`rquickjs`'s bundled QuickJS sources; this is normally already present on
macOS (Xcode Command Line Tools) and Linux.

## Running

```sh
cargo run
# in another terminal:
curl -X POST http://127.0.0.1:8448/ --data "let a = 1; let b = a + 1; a = 10; b;"
curl http://127.0.0.1:8448/logs
curl http://127.0.0.1:8448/graph
```

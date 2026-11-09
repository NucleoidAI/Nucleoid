# Nucleoid Language Reference

Nucleoid is a logic programming language for LLMs.

A Nucleoid program is a set of statements that remain true. An assignment is not
an instruction that runs once and finishes; it is a relationship the runtime
records and maintains. Writing `c = a + b` states that `c` is the sum of `a` and
`b`, and that stays true as `a` and `b` change.

This document is the consolidated reference, assembled from the NUC documents.
See [NUC 0](nuc-0000.md) for the index and [NUC 1](nuc-0001.md) for the
conventions. `nucleoid.spec.md` is normative; where this document and the
specification disagree, the specification wins.

Each NUC is a language proposal. New or updated proposals get a fresh
`nuc-proposal` agent to draft, prototype, and rewrite equivalent executable
cases, followed by an independent fresh `nuc-evaluate` agent to check the
frozen candidate and compare lexer tokens per case without rewriting it.
A proposal is accepted when it has no compile errors, passes every test and
stays backward compatible; it is `Automatic-Accepted` when a policy check such
as the per-case token comparison fails, and its NUC records that result. Draft
or rejected proposals do not change this implemented reference. See
[NUC 1](nuc-0001.md#proposal-evaluation) and the
[automatic acceptance rule](../.claude/rules/proposals.instructions.md#automatic-acceptance-caller-only).

The entries below are abbreviated to the behaviour they state.
[examples.md](examples.md) covers the same ground as complete programs, each one
runnable as written.

## Contents

1. [Statements and State](#1-statements-and-state)
2. [Variables](#2-variables)
3. [Expressions](#3-expressions)
4. [Types and Instances](#4-types-and-instances)
5. [Properties](#5-properties)
6. [Class-Level Rules](#6-class-level-rules)
7. [Blocks and Scope](#7-blocks-and-scope)
8. [Control Flow](#8-control-flow)
9. [Functions](#9-functions)
10. [Transactions](#10-transactions)
11. [Built-in Objects](#11-built-in-objects)
12. [Reasoning Operations](#12-reasoning-operations)
13. [Error Reference](#13-error-reference)
14. [Syntax Summary](#14-syntax-summary)

## Comments

- Hash (`#`) line comments, `//` line comments through LF/EOF and non-nesting
  `/* ... */` block comments through their first closing delimiter are always
  available. [NUC 76](nuc-0076.md) introduced the C-style forms behind a
  `--c-style-comments` flag; the flag and Rust's `SyntaxOptions` were
  removed, and every program, interpolation, lexer, parser and `check` call
  reads them.
- Comment openers win over ambiguous regex delimiters, so `//` is a comment,
  not an empty regex, and `1 // 2 /` is `1`; `/(?:)/` supplies
  empty-matching behavior with pattern text `(?:)`, not an empty string. No
  `RegExp` constructor is added. Ordinary division, `/=`, nonambiguous
  regexes, quoted strings and template text retain their readings.
  Interpolation braces inside literals/comments are protected, and
  interpolation errors refer to original source positions. Outer template
  quoting is unchanged.
- Comments separate tokens, never combine them. Block LFs preserve layout
  except inside parentheses/brackets, where normal suppression applies.
  Comment text and internal whitespace contribute no indentation; only
  actual external whitespace does. Tabs keep width four; diagnostic columns
  still count original characters, including one per tab, with CRLF preserved.
- An unclosed block raises exact `SyntaxError: Unterminated block comment
  (line L, column C)` at its opening slash, before program execution.
- File execution and both interactive interfaces read comments. Plain input
  buffers open blocks through blank lines and command-looking text; EOF
  reports a remaining unclosed block once. Continuation and Terminal Ctrl-J
  use lexical headers, not punctuation inside comments, so `x = 1 # note:`
  runs at once. Existing REPL exit and error-recovery behavior remains
  unchanged.

## Terminal workspace

With terminal stdin and stdout, `nucleoid` opens a full-screen REPL with
reserved top tabs, a bottom multiline editor, autocomplete, result history,
and mouse support. Text, JSON and Table are alternate views of the same
captured result; switching views never reruns a program or changes its state.
Table rows union the keys of object-array results, preserve missing cells
separately from JSON nulls, and keep nested values in individual cells.
Serialization errors, runtime errors and assertion failures remain visible.

Enter inserts a newline without changing pasted indentation, including on
empty lines; F5 submits the complete editor. Ctrl-Enter also submits on
non-Windows terminals that report it distinctly. Native Windows input maps
pasted LF to Ctrl-Enter, so that key inserts a newline on Windows rather than
submitting a partial pasted program. Ctrl-J inserts an automatically indented
newline when typing by hand. Tab completes or indents;
Ctrl-Space opens suggestions. Ctrl-P / Ctrl-N navigates input history.
Use F1-F3 for tabs, F6 or the view buttons for result formats, Ctrl-Up /
Ctrl-Down for earlier results, and F4 to focus the editor, results or
diagnostics. PageUp / PageDown and the mouse wheel scroll; focused-result
Left / Right (or Shift-wheel) moves across text or table columns.
Click/drag positions/selects editor text. `.help` documents all controls;
`.json`, `.table` and `.graph` accept optional `on` / `off` arguments.

Use `--plain` for the previous line-oriented REPL. Non-terminal stdin or stdout
automatically uses that interface, preserving piped input, blank-line/EOF
submission, JSON output, graph output and command recovery. Running a source
file is unchanged. The terminal workspace changes no language syntax, runtime
semantics or generated specification/dataset records.

---

## 1. Statements and State

- A statement is run against the state and changes it.
- An assignment records a relationship, not just a result.
- An expression statement returns its value.
- Statements are applied in the order received; when two assign the same target, the last one holds.
- One statement per line, or several on a line separated by `;`.
- Every statement runs in a transaction.

```
i = 1
```

```
k = 1
k
```

- Returns `1`.

### Colon-led assertion statements (Accepted NUC 45)

- Accepted [NUC 45](nuc-0045.md) adds
  `assert: actual, expected` as an optional statement spelling of the existing
  two-argument `assert(actual, expected)` helper.
- Both lower to the same call expression statement. Arguments run once,
  left to right, with the same deep equality, failure collection, counters
  and transaction behavior. A check is not a standing model rule, graph
  observer, or replacement for the preceding program result.
- Ordinary calls remain supported in every existing context and retain
  their existing argument-count behavior. The colon-led form requires two
  expressions and one separating comma, with no extra/trailing comma,
  truthiness mode, generic bare call, new keyword, or layout continuation.
- Typed declarations win first: `assert: int` remains a declaration with
  `ReferenceError("Missing definition")`. Bindings, parameter annotations,
  fields and keys named `assert` remain unchanged. Direct identifier calls
  named `assert` invoke the helper even when the name is bound.
- The accepted [specification contract](../nucleoid.spec.md#colon-led-assertion-statements)
  and five appended entries cover values, reasoning exclusion, scopes,
  returns, evaluation order, caught errors and rollback.
  The executable reference counterpart is
  `Nucleoid preserves assert names and declarations beside compact checks`.
- Independent evaluation accepted all 603 paired cases: eight use fewer
  lexer tokens, 595 are unchanged, and none increased. All 2,736 protected
  assertions executed and all 48 explicit expected returns matched.

```
assert(record.amount, 3)
assert: record.amount, 3
```

Full detail: [NUC 2](nuc-0002.md).

---

## 2. Variables

### Assignment

- A variable is created by assigning to it; no keyword is required.
- An assignment evaluates to the assigned value.
- A declaration with a type but no value is rejected.

```
a: int
```

- Raises `ReferenceError("Missing definition")`.

### Dependency

- Naming a variable on the right-hand side establishes a dependency.
- Dependents re-evaluate when a dependency changes.
- Dependencies are transitive.
- Queued upstream dependencies settle before their dependents, even after an upstream declaration is replaced.
- Re-evaluation preserves declaration priority; only an explicit re-declaration changes it.

```
a = 1
b = 2
c = a + b
```

- `c` is `3`; after `a = 2`, `c` is `4`.

### Re-assignment

- Assigning again discards the previous dependencies and records the new ones.

```
c = a + 3
c = b + 3
```

- `c` now follows `b`, not `a`.

### Self-reference

- A variable's own occurrence on the right is read as its current value.
- This is not a cycle.

```
radius = 10
radius += 10
```

- `radius` is `20`.

### Read-only state versions

- [NUC 53](nuc-0053.md) defines automatic, read-only variable/property history.
  The [root specification](../nucleoid.spec.md#read-only-state-versions)
  provides postfix `@0`,
  `@1`, ... and contextual aliases `@first`, `@last`, `@previous`.
- Zero is oldest and indexes remain stable for a slot's full lifetime.
  Record only actual changes, including reactive updates; equal writes and
  reads add nothing. Retain all entries across runs until deletion/clear.
  Transactions restore histories with their state writes.
- Resolve the innermost local or underlying object identity/property slot,
  not a textual alias. Local histories end with their existing scope frame.
  Missing/out-of-range entries defer to Null and remain reactive.
- Results are ordinary Values: list values are copies, objects retain identity
  and can expose later mutations. A real property of a returned object remains
  writable. Direct selected-slot writes/deletes and historical-list write-backs
  raise `TypeError("State versions are read-only")`.
- No negative selectors, dot-history aliases, globally reserved selector words,
  deep snapshots, wrappers, hidden fields or JSON changes are added.
  Ordinary `.first`, `.last`, `.previous`, `.value`, negative list indexing
  and manual history-emulation syntax remain supported.
- The executable reference entry is `State versions expose zero based variable
  history`; six specification cases and the README/examples entries retain
  their supplied accepted counterparts and all 69 assertions.
- Four self-contained training records in `dataset/nucleoid.synth.33.jsonl`
  bring the canonical synthesized corpus from 1,125 to 1,129 records, preserving
  every historical record and leaving pending split 32 excluded.

### Compound arithmetic assignment

- Accepted [NUC 13](nuc-0013.md) defines `+=`, `-=`, `*=`, `/=`, and `%=`.
- `target op= expression` lowers to `target = target op (expression)`.
- Compound syntax has the existing assignment precedence and right associativity; it is not a new imperative update path.
- A target is a name or a plain dotted path rooted in a name, `$Class`, `this`, or `self`; class and receiver roots require a property.
- Parentheses may group a path, but calls, indexes, slices, and assignment expressions inside a target are unsupported.
- Self-reference reads the current value while other right-hand references remain tracked; `.value` still takes a snapshot.
- Local parameters, property receivers, expression return values, graph keys, and transaction rollback are unchanged.
- Class-level self-reference remains `TypeError("Circular Dependency")`; compound syntax does not make it a one-time update.
- These forms passed independent per-case evaluation; existing assignment syntax remains valid.
- [NUC 64](nuc-0064.md) adds `**=` to this family: `target **= expression`
  lowers to `target = target ** (expression)` under the same target, bound,
  precedence, relationship and rollback rules, and diagnostics name it `'**='`.

```
amount = 10
step = 2
amount += step
```

- `amount` is `12`; after `step = 3`, it is `15`.
- `amount += step.value` makes it `18`; a later `step = 5` leaves it `18`.
- The executable counterpart is `Nucleoid preserves compound assignment relationships` in `tests/reference.md`.

### Logical compound assignment

- [Accepted NUC 26](nuc-0026.md) defines optional `&&=` and `||=` as one token
  each; every paired case passed independent evaluation without an increase.
- `target &&= rhs` lowers exactly to `target = target && (rhs)`;
  `target ||= rhs` lowers exactly to `target = target || (rhs)`.
- The full right operand keeps assignment precedence and right associativity;
  `a ||= b &&= 9` means `a = a || (b = b && 9)`.
- A standing assignment always installs the expanded relationship, even when
  logical evaluation skips the right operand. It is not a JavaScript-style
  conditional write.
- Logical operators keep operand values and short-circuiting. Existing
  null/deferred handling and pre-evaluation `.value` freezing are unchanged;
  snapshot reads may still occur in a skipped logical operand.
- The same plain-path validation permits at most 128 dotted links across
  grouped paths, rejects computed targets before cloning, and shares the
  existing expression/statement nesting budget.
- Local parameters, `this`/`self`, property self-reference, class-level cycle
  errors, expression returns, dependencies, rollback, graph keys and canonical
  expanded rendering retain the existing assignment behavior.
- All arithmetic compound forms remain valid. The expanded symbolic and word
  forms also remain supported, for example `enabled = enabled || fallback`,
  `enabled = enabled or fallback`, `ready = ready && allowed`, and
  `ready = ready and allowed`.
- The executable counterpart is
  `Nucleoid preserves logical compound assignment relationships` in
  `tests/reference.md`.
- New variants in the exhaustive public Rust `Token` enum are source-breaking
  for downstream exhaustive matches. A human must decide the API semver action;
  no version or dependency change is part of this revision.

### The value property

- `.value` reads the current value and records no dependency.
- Its snapshot remains fixed inside lists, object literals, templates, and slices when other dependencies update.
- It breaks what would otherwise be a cycle.
- The value of a `null` property is `0`.

```
goldenRatio = 1.618
altitude = 10
width = goldenRatio.value * altitude
```

- `width` is `16.18`; unchanged by `goldenRatio = 1.62`; becomes `161.8` after `altitude = 100`.

### Value assignment

- [NUC 66](nuc-0066.md) adds `target := expression`, an action taken once
  beside the standing declarations that `=` makes, as `:=` assigns in Go and
  Pascal. `:=` is one token.
- The expression is evaluated once in the current scope and the value is
  assigned exactly as `target = literal` would assign it. The value is what
  `target = expression` would store at that moment, including `null` for a
  null read.
- Nothing is recorded as a dependency: not the reads of the expression, the
  functions, methods and callbacks it calls or the path to the target, and
  not for an enclosing `if`, block or declaration. Later changes to those
  inputs leave the target alone.
- At the top level the target becomes a stated fact that replaces any
  standing relationship it had; its dependents still update, and `why` shows
  `target = literal` with state `stated`. In a `for` body, an `if` branch or a
  block it is carried out without being filed, as `=` is there, so a standing
  `if` repeats it whenever its branch runs.
- Targets follow the compound assignment rules: names and dotted paths
  rooted in a name, `$Class`, `this` or `self`. Other targets raise
  `SyntaxError("Invalid value assignment target")`. Outside a class-level
  rule, a class-level target raises
  `TypeError("Cannot use ':=' on a class-level property")`; inside one,
  `$Class.property := expression` sets that instance's property.
- `:=` is a statement only. `` `${x:=2}` `` still renders `x`, and every other
  colon keeps its reading.

```
stock = 10
qty = 3
low = stock < 8
stock := stock - qty
qty = 5
```

- `stock` is `7` and `low` is `true`; `qty = 5` does not take `stock` again.
  With `stock = stock - qty`, the same change would make `stock` `2`.
- The executable counterpart is
  `Nucleoid separates one-shot actions from standing laws` in
  `tests/reference.md`; the root contract is
  [Value assignment](../nucleoid.spec.md#value-assignment).

### Is assignments

- `target is value` is a spelling of `target = value` for variables and
  properties. [NUC 93](nuc-0093.md) introduced it behind an
  `--is-assignment` flag; the flag and Rust's `SyntaxOptions` were removed,
  so `a is good` assigns rather than reading as the three statements `a`,
  `is` and `good`.
- The statement form is a name or a dotted path that `:=` accepts, then
  `is` on the same line, then a token that can begin an expression. After
  `is`, a `:`, `=`, `:=`, `+=` or other operator, a keyword such as `if`, or
  the end of the line keeps the accepted reading, so `a is: T`, `a is = 5`
  and `a is` are unchanged.
- The value is parsed and evaluated exactly as the value of `=`: `a is 5`,
  `o.x is "good"`, `this.total is total * 2`, `$Order.state is "open"`,
  `a is b = c` and `a is suppose:` with a body build the trees of their `=`
  spellings and render as them.
- Quotes are optional around a word. A value of exactly one bare identifier
  reads the local, variable, class, function or built-in of that name when
  one exists, and otherwise stores its own text: `a is good` stores `"good"`
  until something defines `good`, then follows it, and returns to `"good"`
  when it is deleted. `a is "good"` and `a is 'good'` are ordinary strings,
  and `a is (good)` requires `good` to be defined.
- It applies wherever a statement can start, including blocks, bodies and
  constructors, but never inside an expression, so `is` comparisons in
  conditions and parentheses work beside it: `flag is (value is EXACT)`.
- A word assignment renders as `a is good` in graph keys and in `why` and
  `affects`; every other one renders as its `=` spelling. Malformed values
  report the errors of their `=` spellings.

```
status is OPEN
label is "shipped"
count is 2 + 3
OPEN = "open"
```

- `status` is `"OPEN"` until `OPEN = "open"`, then `"open"`; `label` is
  `"shipped"` and `count` is `5`. Grammar, the statements whose NUC 86
  readings changed and edge cases are in the authoritative
  [is assignments](../nucleoid.spec.md#is-assignments) section.

Full detail: [NUC 93](nuc-0093.md).

### Destructuring declarations

- [NUC 67](nuc-0067.md) adds destructuring declarations: object patterns
  `{x, y} = point`, renamed targets `{key: name} = source`, which may follow
  each other without commas as object properties do, and Python-style
  unpacking `a, b = pair`.
- Each target is exactly one ordinary declaration, in pattern order:
  `{x, y} = point` is `x = point.x` followed by `y = point.y`, and
  `a, b = pair` is `a = pair[0]` followed by `b = pair[1]`. Every name stays
  live, `why` explains it as that declaration, and a missing key or position
  reads `null`.
- The value is evaluated once per target, so a call or instantiation in it
  runs once per target: `{x, y} = Point(1, 2)` creates two points. Name such
  a value first, as in `p = Point(1, 2)` and `{x, y} = p`.
- A destructuring is a whole statement on one line: it starts a line, or
  follows `;`, a block's `{` or a suite's `:`, and its value ends the line or
  is followed by `;`, `}`, `else` or `catch`. Keys and targets are names, one
  trailing comma is allowed, and a pattern has at most 128 targets.
- `[a, b] = pair` keeps its accepted meaning, an assignment to a list that
  raises `SyntaxError("Cannot assign to [a,b]")` when it runs. Nested
  patterns, defaults, rest targets, tuple values such as `a, b = 1, 2`, and
  `let`, `var` and `const` prefixes are not destructuring and keep their
  syntax errors.

```
point = {x: 2, y: 3}
{x, y: up} = point
pair = [4, 6]
low, high = pair
point.y = 10
pair = [1, 9]
```

- `x` is `2`, `up` is `10`, `low` is `1` and `high` is `9`.
- The executable counterpart is
  `Nucleoid destructures records and pairs into live names` in
  `tests/reference.md`; the root contract is
  [Destructuring declarations](../nucleoid.spec.md#destructuring-declarations).

### Enum declarations

```
enum Size: SMALL, MEDIUM, LARGE
enum Fit: TIGHT = -1, REGULAR = 0, LOOSE = 1
shirt = {size: Size.MEDIUM, fit: Fit.REGULAR}
assert(shirt, {size: "MEDIUM", fit: 0})
assert([Size.LARGE, Fit.TIGHT, Size.HUGE], ["LARGE", -1, null])
enum Fit: TIGHT = -1, REGULAR = 1, LOOSE = 2
shirt.fit
```

- This is the complete `Nucleoid declares enums of named members` program in
  `tests/reference.md`, without its title and expected-return metadata; it
  returns `1`.
- [NUC 90](nuc-0090.md) adds enum declarations: `enum NAME:` followed by two
  or more comma-separated members on the same line, or by an indented block
  with one member per line. A member is a name, optionally followed by `=`
  and a scalar literal: a finite number with an optional sign, a string,
  `true`, `false` or `null`. A member without a value holds its own name as a
  string.
- `enum NAME: ...` declares `NAME` exactly as `NAME = {...}` with the same
  members would, wherever a statement may start outside a class body. The
  declaration reads nothing. Declaring the name again replaces it and
  re-evaluates its readers, as declaring `Fit` again above moves `shirt.fit`
  to `1`, and rollback restores the earlier enum.
- The value is an ordinary object with the members in declaration order.
  Member reads, missing members, `typeof`, assertions, JSON, entry loops and
  spreads treat it as they treat an object literal. It is read-only:
  assigning, adding or deleting a member raises
  `TypeError("Enums are read-only")` and rolls back its transaction. A copy
  that a `suppose` block's value carries out stays read-only, and a spread
  builds an ordinary object.
- `enum Status: PENDING`, a single name after the colon, keeps its accepted
  reading, the name `enum` followed by a typed declaration, so a single
  member uses the block form. `enum` stays an ordinary name everywhere else,
  and class bodies keep every `enum` spelling as before. Canonical source
  writes `enum Fit:TIGHT=-1,REGULAR=1,LOOSE=2`. See the authoritative
  [enum declarations](../nucleoid.spec.md#enum-declarations) section.

### Null and undefined

- Any `null` dependency yields `null`.
- Reading something undefined yields `null` and re-evaluates when it arrives.
- `value ?? fallback` supplies a default for a null or undefined value; see
  [Nullish coalescing](#nullish-coalescing).

### Deletion

- `delete` removes a variable and returns whether anything was removed.
- Dependents of a deleted variable become `null`.
- Using a deleted variable raises a reference error.
- Temporary undefined reads end when the deletion's propagation finishes, including deletions performed by a standing rule.

```
base = 10
tax = base * 0.1
total = base + tax

delete base
```

- `tax` and `total` are `null`; using `base` raises `ReferenceError("base is not defined")`.

Full detail: [NUC 2](nuc-0002.md).

---

## 3. Expressions

### Literals

- Numbers, including negatives and fractions.
- Strings in single quotes, double quotes, or backticks.
- Template literals interpolate with `${...}` and track their dependencies.
- `true`, `false`, `null`.
- Lists in brackets; objects in braces, with quoted or bare keys.
- Regular expressions between slashes.

```
host = "localhost"
port = 8080
url = `http://${host}:${port}`
```

- After `port = 9090`, `url` is `"http://localhost:9090"`.

### Object-property shorthand (candidate)

- In this isolated [Draft NUC 17](nuc-0017.md) candidate, an ordinary identifier
  field `amount` in an object-literal expression means exactly `amount: amount`.
- Explicit fields may mix with shorthand, nest in objects and lists, and keep
  the existing comma, trailing-comma, newline, and semicolon rules.
- The identifier must be followed by a comma or closing brace, looking through
  only the existing newline/semicolon separators; this shorthand qualification
  is unchanged by the NUC 44 explicit-property separator rule below.
- Quoted and numeric keys need explicit values; keywords, `$Class`, dotted
  paths, and computed keys do not become bare shorthand keys.
- Canonical source remains quoted and expanded, such as `{"amount":amount}`.
- Values use the same scope, parameter/receiver bindings, object aliases,
  dependencies, duplicate-key replacement, snapshots, and rollback.
- A missing value name inside an object raises
  `ReferenceError("missing is not defined")`, not a silent `null` field.
- `.value` snapshots still need explicit fields, such as `amount: amount.value`.
- Statement-leading `{ amount }` stays a block; named `def wrap(amount) =>
  { amount }` returns an object, while anonymous `amount => { amount }` stays
  a block and needs `amount => ({ amount })` to return an object.
- This does not add closure capture, indentation inside unparenthesized braces,
  colon continuation, new key grammar, or destructuring.

```
amount = 4
unit = "C"
reading = { amount, unit }

assert(reading.amount, 4)
assert(reading.unit, "C")

amount = 6

assert(reading.amount, 6)
```

- The executable counterpart is `Object property shorthand` in
  `tests/reference.md`; the five paired specification entries in NUC 17 cover
  the runtime behavior.
- These isolated prototype forms await independent evaluation and integration;
  they are not a change to the accepted reference.

### Comma-free explicit properties

- [NUC 44](nuc-0044.md) makes an explicit-property comma
  optional only after parsing the preceding value's entire ordinary expression.
  `{kind: kind lane: lane(urgent) active: true}` has the same complete ordered
  AST as `{kind: kind, lane: lane(urgent), active: true}`.
- Keep an existing comma first. Without one, after only the original
  newline/semicolon skipping, inspect the next two unconsumed tokens:
  an existing string, ordinary identifier or number key immediately followed
  by `:` continues the same object. The guard consumes nothing and never
  speculatively parses, stops early or reinterprets a value.
- Operators, assignments, postfixes, function/arrow bodies and nested objects
  retain priority. Lists, calls, parameters, statement blocks and key grammar
  do not change. No new brace indentation or colon continuation is added.
- Shorthand still needs its own comma-or-closing-brace lookahead.
  `{amount, unit: "C" active: true}` is valid, but `{amount unit: "C"}` and
  `{amount: 1 unit}` are not. Keep commas around shorthand where its original
  rule needs them; trailing commas and mixed explicit separators remain valid.
- Duplicate keys retain every ordered value evaluation before replacement.
  Numeric keys use the same formatting; canonical source remains quoted and
  comma-separated. Actual dependencies, aliases, scopes, receivers, `.value`
  snapshots, whole caught errors, rollback and resource limits are unchanged.

```
def lane(urgent):
    if urgent: return "fast"
    return "normal"
kind = "invoice"
urgent = false
summary = {kind: kind lane: lane(urgent) active: true}
assert(summary.kind, "invoice")
assert(summary.lane, "normal")
urgent = true
assert(summary.lane, "fast")
kind = "alert"
assert(summary.kind, "alert")
```

- This is the exact `Nucleoid builds compact routing records` executable
  counterpart in `tests/reference.md`, paired with its accepted comma form.
  The five appended [specification entries](../nucleoid.spec.md#comma-free-explicit-object-properties)
  trace live construction, keys/replacement, nesting, snapshots and rollback.
- A failed guard keeps the original closing-brace diagnostic.
  Exact `record = { amount: 1 next }` retains
  `SyntaxError("Expected '}' but found identifier 'next'")` at line 1, column 22.
  `record = { amount other }` retains
  `SyntaxError("Expected ':' but found identifier 'other'")` at line 1, column 19.
- `record = { amount\n: amount }` still raises
  `SyntaxError("Expected ':' but found end of line")` at line 1, column 12;
  unparenthesized `record = {\n    amount: amount,\n}` still raises
  `SyntaxError("Expected a key but found an indented block")` at line 2, column 5.
  Existing computed-key writes, exporter splitting and receiver-frame limits
  are not repaired.

### Object spread entries

```
defaults = {theme: "light", size: 12}
prefs = {size: 14}
settings = {...defaults, ...prefs}
assert(settings.theme, "light")
assert(settings.size, 14)
prefs.theme = "dark"
defaults.font = "mono"
assert(settings.theme, "dark")
assert(settings.font, "mono")
delete prefs.size
settings.size
```

- This is the complete `Nucleoid derives records with spread entries`
  program in `tests/reference.md`, without its title and expected-return
  metadata; it returns `12`.
- [NUC 81](nuc-0081.md) adds spread entries to object literals:
  `...SOURCE` copies the entries of the object `SOURCE` holds, in insertion
  order. They are the entries an entry loop visits: an instance's `id` first,
  `null` values included, undefined slots left out. A copy is shallow, so a
  nested object or list is the same value in both records.
- Entries apply from left to right, and each source is evaluated once. A
  later entry for a key the object already holds replaces its value and keeps
  the key's first position, so `{...point, x: 5}` lists `x` first and
  `{x: 9, ...point}` takes `x` from `point`.
- A spread is three adjacent dots wherever a key may begin. A keyed entry may
  follow it without a comma, as in `{...base qty: 1}`; a spread after a value
  or another spread needs one. Statement-leading braces and anonymous arrow
  bodies in braces stay blocks, so `({...a})` and `lambda p: {...p}` write
  objects. `[...xs]`, `f(...xs)` and `{x, ...rest} = p` keep their syntax
  errors.
- A `null` source or an undefined property copies nothing. Reading a `null`
  source is an ordinary null read, so the declaration is `null` until its
  source holds an object, while copied `null` values never settle it. Any
  other value raises `TypeError("<source> is not an object")`.
- A declaration that spreads stays live. It depends on every entry it copies,
  except one a later keyed entry replaces, and observes its source's set of
  entries, so a changed, deleted or added entry and a rebound source
  re-evaluate it. `why` lists the dependencies of the explicit record that
  names the same entries, `--graph` shows the observation as
  `observes {...<object id>}`, and rollback restores entries, their order and
  observations together.
- While a property declaration evaluates its value, a spread of the object
  that owns the property leaves that property out, so
  `$Order.view = {...$Order, paid: false}` never nests an older view. Two
  class-level rules that spread the same receiver read each other and raise
  `TypeError("Circular Dependency")` when an instance applies both.
- The authoritative contract is the
  [object spread entries](../nucleoid.spec.md#object-spread-entries) section.

### Operators

- Arithmetic: `+`, `-`, `*`, `/`, `%`, and exponentiation `**`; see below.
- Compound assignment: `+=`, `-=`, `*=`, `**=`, `/=`, `%=`; each operator is one token.
- Logical compound assignment: `&&=`, `||=`; adjacent characters form
  one token. `&& =`, `|| =`, `and=`, and `or=` do not form compound operators.
- Comparison: `==`, `!=`, `<`, `>`, `<=`, `>=`.
- Comparison words: `is` and `is not`, spellings of `==` and `!=` with a bare
  word rule; see [is and is not comparisons](#is-and-is-not-comparisons).
- Logical: `and` / `&&`, `or` / `||`, `not` / `!`.
- Conditional: `condition ? consequent : alternate` or
  `consequent if condition else alternate`; see below.
- Nullish coalescing: `left ?? right`; see below.
- Noun queries: `sum of xs`, `max of xs`, `min of xs` and `mode of xs`, prefix
  operators that mean exactly `xs.sum()` and so on; see
  [noun list queries with `of`](#noun-list-queries-with-of).
- Parentheses override precedence.
- `+` concatenates when an operand is a string.

```
assert(a + b * c, 14)
assert((a + b) * c, 20)
```

- A comparison may be assigned directly: `alarm = reading > threshold`.

### Conditional expressions

- `condition ? consequent : alternate` and Python's
  `consequent if condition else alternate` are one expression. The condition
  is evaluated once with the truthiness of `if`, then only the selected branch.
- A declaration that uses one stays live: it re-evaluates when the condition or
  the selected branch's inputs change and switches branches when the condition
  changes. The unselected branch has no effects, errors or dependencies.
- A falsy consequent such as `0` is still selected, unlike in `c && 0 || 5`.
- Both spellings share one right-associative level between assignment and `|>`:
  `x = c ? a : b` assigns the result, `a || b ? c : d` tests `a || b`, and
  `a ? b : c ? d : e` groups to the right.

```
stock = 3
status = stock > 0 ? "available" : "sold out"
badge = "low" if stock < 5 else "plenty"
assert(status, "available")
assert(badge, "low")
stock = 0
assert(status, "sold out")
assert(badge, "low")
stock = 8
assert(status, "available")
assert(badge, "plenty")
status
```

- This is the complete `Nucleoid chooses values with conditional expressions`
  program in `tests/reference.md`, without its title and expected-return
  metadata; it returns `"available"`.
- The existing assignment rules apply: a stored declaration whose condition read
  a null binding settles to `null`, as `x = flag || 2` does, and `.value` is
  frozen in every branch when the declaration is stored. Class-level rules
  validate names and cycles in both branches when they are declared.
- Statement `if` rules are unchanged, including an `if` written after an
  expression on the same line: the Python spelling applies only when `else`
  directly follows its condition.
- `lambda` stays a name at the top level of a `?` consequent and of a Python
  condition. The Python spelling is not recognized at the top level of a
  template interpolation, whose trailing tokens are ignored; write
  `` `${c ? a : b}` `` or `` `${(a if c else b)}` `` there.
- Both spellings render canonically as `(condition?consequent:alternate)`.
  Grammar, errors and edge cases are in the authoritative
  [conditional expressions](../nucleoid.spec.md#conditional-expressions)
  section.

Full detail: [NUC 59](nuc-0059.md).

### Nullish coalescing

- `left ?? right` yields `left` unless it is null, and otherwise evaluates and
  yields `right`. `left` is null exactly when a declaration of it alone would
  store `null`: its value is null or undefined, or evaluating it read a null.
- Falsy values such as `0`, `""` and `false` are kept, unlike with `||`, and a
  null read inside `left` no longer settles the declaration to `null`.
- `right` is evaluated only when `left` is null; otherwise it has no effects,
  errors or dependencies. A declaration that uses `??` stays live and switches
  between the value and the default as `left` changes.
- One right-associative level sits between conditional expressions and `|>`,
  below `||`: `a || b ?? c` is `(a || b) ?? c`, `a ?? b == c` is
  `a ?? (b == c)`, and `a ?? b ? c : d` tests `a ?? b`. Write `(x ?? 0) > 5`
  to compare a default.

```
stock = null
count = stock ?? 0
label = stock ?? "unknown"
assert(count, 0)
assert(label, "unknown")
stock = 0
assert(count, 0)
assert(label, 0)
stock = 4
assert(count, 4)
assert(label, 4)
count
```

- This is the complete `Nucleoid supplies defaults with nullish coalescing`
  program in `tests/reference.md`, without its title and expected-return
  metadata; it returns `4`.
- An undefined name is still an error: `missing ?? 5` raises
  `ReferenceError("missing is not defined")`. Class-level rules validate both
  operands when they are declared, and an undefined read that `??` replaced
  with its default does not make a class-level `if` stand aside.
- `??` is one token and renders canonically as `(left??right)`. `??=`, `?.`
  and `?:` are not added. Grammar, errors and edge cases are in the
  authoritative [nullish coalescing](../nucleoid.spec.md#nullish-coalescing)
  section.

Full detail: [NUC 60](nuc-0060.md).

### Is and is not comparisons

- `left is right` and `left is not right` are spellings of `==` and `!=`,
  with their loose equality and precedence, for numbers, strings, booleans,
  null and property reads: `count is 3`, `ready is true`,
  `order.status is "SHIPPED"` and `a.x is not b.x`.
- A quoted right operand is a string in either quote. A right operand that is
  one bare word, such as `EXACT`, reads the local, variable, class, function or
  built-in of that name when one exists, and otherwise compares with its own
  text, `"EXACT"`. Keywords, `$Class` references, paths, calls and
  parenthesized operands are ordinary operands, so `value is (EXACT)` still
  needs `EXACT` to be defined.
- A comparison with a word depends on that name even while nothing defines
  it: defining, changing or deleting the variable re-evaluates every
  declaration, `if` rule and class-level rule that uses it.
- `is` is an operator in conditions, parentheses, call arguments, lists,
  object values, subscripts, `?` consequents, Python-style conditions and the
  first operand of `assert:`, and in callbacks written there. Where a
  statement can follow, such as directly
  after `flag =` or `return` or in a named arrow body, `is` stays an ordinary
  name and starts the next statement, as before; write
  `flag = (value is EXACT)`. At the start of a statement, `value is EXACT`
  is an [is assignment](#is-assignments).

```
value = "EXACT"
if value is "EXACT":
    double = "yes"
if value is 'EXACT':
    single = "yes"
if value is EXACT:
    bare = "yes"
assert([double, single, bare], ["yes", "yes", "yes"])
changed = (value is not EXACT)
assert(changed, false)
EXACT = "OTHER"
assert(changed, true)
assert: value is EXACT, false
assert(value is "EXACT", true)
changed
```

- This is the complete `Nucleoid compares values with is and is not` program
  in `tests/reference.md`, without its title and expected-return metadata; it
  returns `true`.
- A word comparison renders as `(value is EXACT)`, so it never shares a graph
  key with `value == EXACT`; every other `is` renders as `==` or `!=`.
  Malformed forms report the errors of their `==` spellings. Grammar, the
  exact set of positions and edge cases are in the authoritative
  [is and is not comparisons](../nucleoid.spec.md#is-and-is-not-comparisons)
  section.

Full detail: [NUC 86](nuc-0086.md).

### Exponentiation

- `base ** exponent` raises a number to a power and yields exactly the value
  of `Math.pow(base, exponent)`, including its coercions, NaN, infinities and
  signed zeros. It does not read the name `Math`, so a variable named `Math`
  changes `Math.pow` calls but never `**`.
- `**` binds tighter than `*`, `/` and `%` and groups to the right:
  `2 ** 3 ** 2` is `512` and `3 * x ** 2` is `3 * (x ** 2)`.
- As in Python, a prefix operator written before the base applies to the
  whole power: `-x ** 2` is `-(x ** 2)`, and `not`, `!`, `+`, `typeof`,
  `delete`, noun queries such as `sum of` ([NUC 75](nuc-0075.md)) and prefix
  stages such as `why` behave the same way. Write
  `(-x) ** 2` to raise a negative base. The exponent may carry its own sign:
  `2 ** -1` is `0.5`.
- `target **= exponent` is the compound spelling of
  `target = target ** (exponent)`, with every rule of compound arithmetic
  assignment.
- A declaration that uses `**` stays live, settles to `null` after a null
  read and freezes `.value` operands when it is stored, as other arithmetic
  does.

```
side = 3
volume = side ** 3
tower = 2 ** side ** 2
assert(volume, 27)
assert(tower, 512)
side = 2
assert(volume, 8)
assert(tower, 16)
rate = 0.5
growth = 16
growth **= rate
assert(growth, 4)
rate = 3
assert(growth, 64)
volume
```

- This is the complete
  `Nucleoid raises values to live powers with exponentiation` program in
  `tests/reference.md`, without its title and expected-return metadata; it
  returns `8`.
- `**` and `**=` are one token each and render canonically as
  `base**exponent`, parenthesizing an operand where needed so the text reads
  back as the same expression, such as `(-x)**2`. Each `**` takes one level
  of the expression nesting budget. Grammar, errors and edge cases are in the
  authoritative [exponentiation](../nucleoid.spec.md#exponentiation) section.

Full detail: [NUC 64](nuc-0064.md).

### Indexing, slicing, and length

- Lists and instance lists index with brackets; the index may be a variable.
- Object bracket access has the same null, undefined, and dependency behavior as a dotted property read.
- A range is written `[start:end]`; either bound may be omitted, and a negative bound counts from the end.
- A range on a class's instance list, such as `Task[-2:]` or `Task[:]`, returns its instances in population order with the same bound rules, and depends on the population ([NUC 61](nuc-0061.md)).
- Both slice bounds are expressions whose references are checked when a class-level rule is declared.
- `.length` is a dependency on the collection.

```
meeting.time[-17:]
```

- The last 17 characters.

Full detail: [NUC 10](nuc-0010.md).

---

## 4. Types and Instances

### Declaring a type

- `class Entity` declares an empty type without a placeholder body.
- The explicit `class Entity:` form with an indented `pass` remains valid.
- A bare header ends at a statement boundary and does not adopt the next statement as a constructor.
- An indented body still requires a colon. Inheritance keeps its colon grammar and may also be spelled with `extends`, as [Subtypes](#subtypes) shows.
- `$Name` is the type; `Name` is the list of its instances.
- `typeof $Name` is `Class`; `typeof Name` is `List`.
- Every declared type is added to the global `Class` list.
- Assignments reading `Class.length` update when a type is declared; re-declaring a type does not increase the count.

```
class Entity
```

### Constructors

- Parameters may be declared on the type, with optional annotations.
- A bodyless header may keep the same parameter list, such as `class Shell(code: int)`; parameters alone do not create or assign properties.
- A parameter written `this.name` or `self.name` also assigns the property it names, so `class Shape(this.type: str)` declares the first form below; see [Initializing constructor parameters](#initializing-constructor-parameters).
- An equivalent form declares attributes plus an `init` function.
- A missing argument leaves the attribute `null`.

```
class Shape(type: str):
    this.type = type
```

```
class Shape:
    type: str

    def init(type: str):
        this.type = type
```

### Default named constructors

- Ordinary typed and untyped class headers support positional arguments,
  names in any order, and positional arguments followed by names. There is no
  star marker or named-only class mode.
- Labels do not read or assign caller variables. Values run once in source
  order, then bind by declared name; omitted parameters without literal
  defaults remain `null`.
- The values remain constructor snapshots, not new reactive relationships.
  Binding validation precedes all RHS evaluation, including `.value` reads.
- `class Point` can adopt an ordinary `def init(x, y):` body and parameter
  list. Named binding adds no automatic fields or defaults; scalar defaults
  have their own [bounded contract](#literal-parameter-defaults).
- Named shared class-rule construction also sees the declaration caller's
  parameters; legacy positional shared construction keeps its existing scope.
- Inheritance uses the nearest nonempty constructor body's signature; a child
  can forward positional, named or mixed values with `super(...)`.
- This revision intentionally removes star headers and changes bare
  constructor `name=value` from caller assignment to label binding.
  `Legacy((b=2), (a=1))` explicitly preserves positional assignment effects.
  Compound and member assignments remain positional. Functions, methods,
  built-ins and direct `init` calls keep assignment-expression behavior.
  Class aliases gain no construction route.
- Duplicate bindings (including positional plus named), unknown labels, and
  positional arguments after labels raise explicit `TypeError`s before
  value effects or instance creation. Star headers raise the ordinary
  parameter `SyntaxError`; existing transactions still restore partial
  construction and graph/class state on failure.
- This is the exact paired `Nucleoid constructs an object with named parameters`
  example in `tests/reference.md`.

```
class Point(x, y):
    this.x = x
    this.y = y
point = Point(y=4, x=3)
assert(point, {"id": "point", "x": 3, "y": 4})
```

- Exact binding/error rules are in the authoritative
  [default named constructors](../nucleoid.spec.md#default-named-constructors)
  section; [NUC 47](nuc-0047.md) records this isolated candidate revision.

### Initializing constructor parameters

- A constructor parameter written `this.name` or `self.name` declares the
  parameter `name` and assigns it to the new instance's property `name`, as
  Dart's initializing formals and TypeScript's parameter properties do
  ([NUC 87](nuc-0087.md)).
- `class Point(this.x, this.y)` declares exactly the class that
  `class Point(x, y):` with the indented `this.x = x` and `this.y = y`
  declares: the same parameters, constructor statements, binding, `why`
  rules, graph keys and canonical rendering.
- The prefix applies to the parenthesized list of a class header in every
  header form, bodyless, with a colon body, with a colon parent or with
  `extends`, and to the parenthesized list of a `def init` written in a
  class's indented body. Annotations, literal defaults, labels and a final
  rest parameter keep their meanings.
- The assignments run first, in parameter order, before every written
  statement, including a leading `super(...)`, which may assign the same
  property again. A header with a prefixed parameter therefore has a
  nonempty constructor even without a body, and a bodyless subclass header
  with one is its own nearest nonempty constructor.
- Parameters without the prefix still create no properties. Every other
  parameter list, every bare signature and every other spelling after
  `this` keep their accepted errors.
- This is the exact
  `Nucleoid initializes properties from constructor parameters` example
  in `tests/reference.md`.

```
class Point(this.x, this.y)
class Box(this.width, self.height):
    this.area = width * height
p = Point(3, 4)
b = Box(2, 5)
assert(p, {"id": "p", "x": 3, "y": 4})
assert([b.width, b.height, b.area, Box.length], [2, 5, 10, 1])
b.area
```

- Exact rules are in the authoritative
  [initializing constructor parameters](../nucleoid.spec.md#initializing-constructor-parameters)
  section.

### Instance keywords

- `this` and `self` are interchangeable keywords for the current instance in constructors, methods, and class-level rules.
- Either spelling reads and writes the same properties; both may be mixed in one body without declaring a receiver parameter.
- Neither keyword may be rebound. Using either without a current instance raises `ReferenceError("this is not defined")`; rebinding either raises `TypeError("Cannot assign to 'this'")`.

### Instances

- Creating an instance registers it in the type's list and returns it.
- The variable name becomes the instance `id`.
- An instance created without a variable name gets a generated `id`.
- Instances are addressed by index `Name[0]`, by id `Name["id"]`, or by query `Name.find(...)`.
- `Name[1:]`, `Name.at(-1)`, `Name.list()` and `Name.all()` read them by range, by position from either end, and all at once, in population order ([NUC 61](nuc-0061.md)).

### Subtypes

- A subtype names its supertype after the colon; `super(...)` invokes its constructor.
- `super(...)` advances from the constructor currently executing; subtypes without constructors use the nearest ancestor constructor.
- Class-level rules on the supertype apply to subtype instances.
- Direct or indirect inheritance cycles are rejected when the type is declared or re-declared.

```
class Truck: Vehicle
    def init make, payload:
        super(make)
        this.payload = payload
```

- `extends` is an optional spelling of the parent, as JavaScript, TypeScript, Java, PHP, Dart, Scala and Groovy write it ([NUC 78](nuc-0078.md)). It follows the class name and the optional parameter list: `class Truck extends Vehicle:` and `class Truck(make, payload) extends Vehicle:` take the same indented or same-line body as the colon spelling.
- Without a colon, `class Van extends Vehicle` is a bodyless subtype, exactly `class Van: Vehicle` with an indented `pass`. Like `class Entity`, it ends at a statement boundary and does not adopt the next statement as a constructor; an indented body still requires the colon.
- Every spelling declares the same class: parent, constructors, `super`, populations, class-level rules, graph keys and canonical rendering are unchanged, and canonical source keeps the colon spelling.
- `extends` is not a keyword. Only `extends` directly after a class name or its parameter list starts the clause; variables, properties, parameters, functions, methods and classes named `extends` keep their meanings.

```
class Truck extends Vehicle:
    def init make, payload:
        super(make)
        this.payload = payload
class Van extends Vehicle
```

### Bare zero-argument super calls

- Bare `super` is the primary expression `super()`; it always passes zero arguments and returns `null` on success.
- The parent's constructor runs on the same receiver, without adding an instance to the parent's population.
- In a constructor, resolution advances from the constructor currently executing; in a method, it falls back to the actual receiver's class.
- A receiver with no parent makes the call a no-op returning `null`; missing parent parameters remain `null`.
- Without an instance receiver, either spelling raises `ReferenceError("super is not defined")`.
- An immediate `(` still introduces explicit arguments, including when whitespace precedes it; `super(arguments...)` is unchanged.
- A logical newline or semicolon is not skipped to find arguments; `super` followed by `(7)` on the next logical line is a zero-argument call and then a separate expression.
- Line breaks already suppressed inside `(` or `[` retain their existing meaning; `(super` followed by `(7))` inside the same parentheses is still the explicit call `super(7)`.
- Existing postfix and operator grammar applies; bare `super` is also usable in assignments, returns, and expression bodies.
- The existing adjacent-statement grammar is unchanged: `super amount` parses as a zero-argument call followed by the expression `amount`, not an argument-bearing call.

```
class Parent:
    def init():
        this.ready = true

class Child: Parent
    def init():
        super
        this.code = 7

child = Child()

assert(child.ready, true)
assert(child.code, 7)
```

### Redefining and deleting

- Re-declaring a type preserves the class list and existing instances.
- New attributes are `null` on existing instances.
- `delete` removes an instance and returns whether anything was removed.
- Deletion is refused while the instance still has properties.
- A variable name is reusable after its instance is deleted.

Full detail: [NUC 3](nuc-0003.md); accepted compact headers: [NUC 14](nuc-0014.md); candidate bare calls: [NUC 16](nuc-0016.md); extends clauses: [NUC 78](nuc-0078.md); initializing constructor parameters: [NUC 87](nuc-0087.md).

---

## 5. Properties

- A property is created by assigning to it, and behaves like a variable assignment.
- A property may be assigned from one that does not exist yet; the result is `null` until it does.
- Assigning again replaces the relationship; self-reference reads the current value.

```
order1.upc = "04061" + order1.barcode
```

- `null` until `order1.barcode` is set, then `"0406194067"`.

### Chains through references

- A property may read through another instance, and the dependency follows the reference.
- Replacing the referenced instance re-points the dependency.
- Writing through a reference records the rule on the referenced instance.

```
user1.plan = basic
user1.bill = user1.plan.rate * 12
```

- `120`; after `user1.plan = pro`, `360`; after `pro.rate = 40`, `480`.

### Null, value, and deletion

- Reading through a missing link yields `null`; writing through one raises.
- `value` may not be used as a property name.
- Deleting a property makes its dependents `null` and leaves others untouched.

Full detail: [NUC 4](nuc-0004.md).

---

## 6. Class-Level Rules

- `$Name.property = ...` states a rule for every instance of the type.
- Rules apply to instances created before and after them.
- Rules may chain, read variables, and read through references into other types.

```
class Human(name: str):
    this.name = name

$Human.mortal = true
```

- Every `Human` has `mortal` `true`.

### Conditionals, order, and overriding

- `if`, `else if`, and `else` may be written over `$Name`; each instance is evaluated independently.
- Instances whose inputs are missing are left untouched.
- Rules on the same type apply in the order received; when several assign the same property, the last matching one wins.
- Assigning the property on one instance overrides the rule for that instance only.
- A rule on a subtype overrides the supertype's.
- Parent rule changes also apply to existing subtype instances, while subtype property rules retain precedence over inherited assignments.
- Rules validate references and dependency cycles before instances exist, including expressions nested inside collections, templates, and slices.

```
if $Grade.score > 89:
    $Grade.letter = "A"
else if $Grade.score > 79:
    $Grade.letter = "B"
else:
    $Grade.letter = "C"
```

### Aggregation

- A rule may aggregate another type's list.

```
$Category.revenue = Product.filter(p => p.category == $Category).reduce((sum, p) => sum + p.price, 0)
```

- With [NUC 68](nuc-0068.md), `Product.filter(p => p.category == $Category).sum(p => p.price)`
  states the same total.

Full detail: [NUC 5](nuc-0005.md).

### Fixed per-instance state anchors

- Draft NUC 55 extends NUC53 histories to class self-anchors. In the isolated
  prototype, `$Counter.x = $Counter.x@first - step` uses each receiver's first
  recorded `x`, even when its current value changed before declaration.
  `@0` is identical to `@first`; numeric positions are also fixed anchors.
- Re-evaluation recomputes from that same entry, not from the last result.
  Missing fixed self-entries defer the assignment without writing a placeholder.
  A real recorded Null retains normal Null semantics.
- Conditional anchors observe their property slots through deletion/recreation;
  non-self numeric/newest/previous history readers remain live. Ordinary
  instance overrides and class order/subtype behavior are unchanged.
- Moving `@last`/`@previous` self-reads in an imperative conditional body keep
  their accepted effects and history order; guard reactivation can therefore
  change the result again. Unlike fixed anchors, missing moving entries retain
  ordinary deferred-Null assignment behavior. Direct moving self-rules still
  raise `TypeError("Circular Dependency")`, as do genuine graph cycles.
- `.value` keeps its explicit snapshot meaning. Version destinations remain
  read-only, and histories/rules/observers roll back together.
- The executable reference case is `Class state anchors preserve independent
  first recorded values`. The root contract is **Class-level state version
  anchors** in `nucleoid.spec.md`; the complete orientation example includes
  `this.orientation = orientation`.
- Four new self-contained `dataset\nucleoid.synth.35.jsonl` records illustrate
  the appended normative contract, bringing the synthesized corpus from 1,133 to
  1,137 records without changing historical records.

```
class Counter(seed):
    this.x = seed
step = 1
first = Counter(10)
$Counter.x = $Counter.x@first - step
second = Counter(30)
assert([first.x, second.x], [9, 29])
step = 2
assert([first.x, second.x], [8, 28])
assert([first.x@0, second.x@first], [10, 30])
```

- With [NUC 85](nuc-0085.md), a rule may also read another property of its
  own receiver at a fixed position, as in `$Point.x = $Point.y@first`. A class
  cycle whose every read is such a fixed anchor, or a fixed self-anchor,
  settles and is not a circular dependency; one live or moving read on the
  cycle still raises `TypeError("Circular Dependency")`.
- Fixed anchors stay tracked: deleting the slot they read makes them read
  null, and recreating it makes them follow its new first entry. Only reads
  written directly on `$Class` count; helpers, `this` and other objects keep
  their existing checks. A missing entry of another property reads null.
- The executable reference case is `Class state anchors exchange properties
  through fixed cross anchors`. The root contract is **Fixed cross-property
  class anchors** in `nucleoid.spec.md`.

```
class Point(x, y):
    this.x = x
    this.y = y
$Point.x = $Point.y@first
$Point.y = $Point.x@first
p = Point(1, 5)
assert([p.x, p.y], [5, 1])
```

---

## 7. Blocks and Scope

- A block is written with braces and introduces a scope.
- A name first assigned inside a block is local to it and does not survive it.
- Dependencies carried through a local do survive.
- Blocks nest; an inner block sees enclosing locals.
- A local shadows an outer name; lookup takes the lowest enclosing binding.

```
h = 1

{
    value = h * 2
    j = value * 2
}
```

- `j` is `4`; after `h = 2`, `j` is `8`.

### Instances, guards, and return

- Instances may be created in a block and assigned outward.
- A local may hold an instance from the state; properties assigned through it are recorded on that instance.
- `return` yields a value; the first `return` reached wins.
- A block may return a local, a property of a local, or an inline object or list.

```
{
    device = Device.find(d => d.code == "A0")
    if not device:
        throw "INVALID_DEVICE"
    return device
}
```

Full detail: [NUC 6](nuc-0006.md).

---

## 8. Control Flow

### Conditionals

- A conditional is a standing rule, re-evaluated when its condition or its branches change.
- To choose a value inside an expression instead of running a branch, use a
  [conditional expression](#conditional-expressions). Separate `if` rules with
  the same condition text replace each other; conditional expressions belong to
  the declarations that contain them.
- A false condition leaves the target untouched rather than clearing it.
- Re-declaring a conditional replaces it.
- Conditionals apply in the order received; when several match, the last one holds.
- Re-evaluating one conditional does not move it after the others.
- A conditional can run again when an input changes later in the same cascade. An unchanged write does not restart feedback.
- Accepted [NUC 34](nuc-0034.md) makes the colon after a final `else` optional
  only before a genuine indented suite. The unchanged `else:` spelling is
  equivalent; final `else`/`elif` remains unconditional.
- The [conditional-suite contract](../nucleoid.spec.md#colon-free-indented-conditional-suites)
  also allows the colon to be omitted after an initial `if condition`, explicit
  `else if condition`/`elif if condition`, or an accepted direct alternative.
  Only a genuine indented body qualifies; every colon-bearing spelling remains
  supported. [NUC 49](nuc-0049.md) describes this source-only addition.
- Require immediate newline/indent after `else` and a real indent after only
  normal newline/semicolon layout. Blank/comment lines create no indent;
  same-line statements, semicolons and braces cannot qualify for omission.
- Reuse the whole existing body and nested/dangling-else association. Inline,
  semicolon-separated and braced alternatives retain their required colon.
  Unrelated headers, canonical rendering, live rules, scopes, returns, throws,
  rollback and limits remain unchanged.

```
if toy1.color == "RED":
    toy1.shape = "CIRCLE"
```

- `null` until `toy1.color` is `"RED"`.

The executable reference's `Nucleoid preserves live property relationships in
indented else suites` counterpart covers both branches and later input changes:

```
class Signal
signal = Signal()
signal.enabled = false
base = 4
if signal.enabled:
    signal.output = base
else
    signal.output = -base
assert(signal.output, -4)
base = 6
assert(signal.output, -6)
signal.enabled = true
assert(signal.output, 6)
base = 8
assert(signal.output, 8)
```

- `if false: pass\nelse` raises
  `SyntaxError("Expected ':' but found end of input")` at line 2, column 5.
  A final newline, comments-only body or unindented `pass` instead raises
  `SyntaxError("Expected ':' but found end of line")` at line 2, column 1.
- `if false: pass\nelse;\n    pass` raises
  `SyntaxError("Expected ':' but found ';'")` at line 2, column 5.
  Same-line `else pass` and `else { pass }` retain their missing-colon errors.
- The authoritative [colon-free indented else suites](../nucleoid.spec.md#colon-free-indented-else-suites)
  section records layout, exact inherited diagnostics and executable traces.
- The CLI buffers initial colon-bearing and lexical colon-free `if` headers,
  including lexical comments and balanced condition delimiters. Explicit and
  direct alternatives autoindent only in that same submission; blank-line
  commit/EOF still uses the real parser and exposes missing-body or other errors.
- [NUC 43](nuc-0043.md) also permits `else condition:` or
  `elif condition:` in an existing chain, only with a complete balanced
  condition header and a genuine indented suite. Both lower exactly to
  the nested conditional of `else if condition:`.
- `elif` is still the lexical alias for `else`. Explicit `elif if` works
  just like `else if`; unconditional `elif:` and colon-free indented final
  `elif` keep their accepted behavior. No new keyword or standalone
  alternative statement is introduced.
- Direct conditions cannot use inline/braced bodies, `:;` or header separators.
  The colon is optional only at genuine indentation. A non-consuming balanced
  token scan ignores nested colons and checks the existing real-indent predicate.
  Failed qualification
  leaves the final-else parser and original-source diagnostics untouched;
  qualified conditions and bodies report ordinary parsing errors.
- The document/JSONL exporter retains complete `else` and `elif` keywords
  with their top-level chain for spaced, tabbed and lexer-supported compact
  headers, including balanced multiline conditions and comments. Identifier
  prefixes such as `elsewhere`, `else1` and `elif_name` remain independent
  statements. The parser alone validates the grammar; canonical AST rendering
  remains unchanged. Generated specification coverage includes a direct
  top-level `else` class rule.
- First-match order, short-circuit reads, reactive dependency replacement,
  per-instance rules, scopes, returns, frozen functions and rollback remain
  unchanged. Inherited queued-method receiver failures are not repaired.
  The five new specification entries and
  `Nucleoid routes records through direct alternatives` in the executable
  reference cover the [normative contract](../nucleoid.spec.md#direct-conditional-alternatives).

### Truthiness

- A condition need not be a comparison; a bare value is tested for presence.
- `null`, an undefined property, and `0` are false; a non-empty value is true.
- An instance is true, so a query result can be tested directly.
- `not` inverts the test.
- Conditional expressions use the same test; see
  [Conditional expressions](#conditional-expressions).
- `??` tests whether a value is null rather than whether it is true, so `0`
  and `""` are kept; see [Nullish coalescing](#nullish-coalescing).
- `all` and `any` quantifiers test each member's condition the same way; see
  [Quantifiers](#quantifiers).
- A `while` loop tests its condition the same way before every iteration; see
  [while](#while).

```
if $Device.profile:
    $Device.active = true
```

- Only devices that have a profile become active.

### for ... of / in

- `for <name> of <source>` and `for <name> in <source>` iterate a class
  population, a list of registered typed instances, or a String's Unicode
  scalars under the [NUC 51 candidate](nuc-0051.md).
- The body runs once, over the values present at the time; it does not re-run later.
- Nested conditions and blocks are part of that one-shot body, not independent standing rules.
- Primitive/null elements, unregistered objects, plain `Object()` instances and
  literal objects in lists are skipped; order and duplicates are retained.
- Loops may nest, producing the cross product.
- Explicit sources keep arbitrary expressions and inline/braced/indented suites.
- A String is evaluated once and fully snapshotted before the first body:
  each Unicode scalar becomes one String value, in source order. Empty
  Strings execute no bodies. No byte/UTF-16/grapheme splitting, normalization,
  trimming, coercion or automatic callable invocation is added.
- String bodies use the same temporary binding, imperative mode, enclosing
  `this`/`self`, early return and transactional rollback/recovery as other
  For bodies. Source mutation cannot extend a snapshot; no implicit return
  or standing loop declaration is introduced.
- String entries in Lists are still skipped, like all other primitive
  entries. Traversing `text.split("")` as a List is not direct String
  iteration. Accepted split/map expressions remain supported but their map
  results are not a For statement's normal `null` result.

```
for question of Question:
    Summary(question)
```

- Accepted [NUC 31](nuc-0031.md) specifies `for NAME:` with a genuine indented
  body as exactly `for it of NAME:` with that same body; `NAME` is one ordinary
  identifier source, not the iteration variable.
- Require the immediate identifier and colon, an immediate newline or indent,
  then a real indent after only existing newline/semicolon layout. Do not skip
  header separators; `:;`, inline/braced/missing-indent shorthand and
  property/index/call/literal/class-reference/parenthesized shorthand sources
  are not added.
- Source lookup precedes binding local `it`, so `for it:` reads the outer
  source named `it`. Nested/global bindings and receivers are restored on
  completion, return and error.
- The snapshot, imperative body, return propagation, graph dependencies,
  transactions and rollback are those of the explicit form. The body is
  never re-run merely because an input or class population later changes.
- A named noniterable source, including `broken = null`, raises
  `TypeError("broken is not iterable")`; callable sources are not invoked.
- Nonqualifying `for values:` headers retain
  `SyntaxError("Expected 'of' in for statement")` at line 1, column 11;
  existing explicit-loop and body diagnostics remain unchanged.

```
for Task:
    it.done = true
```

- [NUC 72](nuc-0072.md) adds two-name loops over an object's entries:
  `for key, value of source:` and `for key, value in source:`. A header is a
  two-name header exactly when `for` is followed by a name, a comma, a name and
  `of` or `in`; the source is any expression and the body any suite the
  explicit loop accepts.
- The source is evaluated once and must be an object; every other value raises
  `TypeError("<source> is not an object")`, and an undefined name keeps
  `ReferenceError("<name> is not defined")`.
- The loop visits the object's own entries in insertion order: literal keys as
  written, later properties when first assigned, deletions removed and
  re-added properties last. An instance lists `id` first, then its stored
  fields. `null` values are visited; undefined literal slots are not entries.
- Keys and values are snapshotted before the first body. Added entries are not
  visited, deleted ones still are, and the value binding keeps its snapshotted
  value while `source[key]` reads the current one.
- Each iteration binds the key, a string, and the value in its own scope and
  runs the body imperatively. Returns, rollback, receivers and nesting are
  those of the explicit loop. The snapshot records no dependency of its own,
  so a loop in a standing branch or block does not rerun when an entry changes.
- Rollback restores entry order: after a failed `try`, a failed statement or a
  `suppose` block, every object lists its entries exactly as before.
- Graph keys render `for key,value of source`; single-name loops, including
  `TypeError("<name> is not iterable")` for objects and the
  `Expected 'of' in for statement` diagnostics, are unchanged.

```
stock = {rice: 4, beans: 0, oil: 2}
low = []
for item, count of stock:
    if count < 2: low.push(item)
```

Normative detail: [implicit iterator indented loops](../nucleoid.spec.md#implicit-iterator-indented-loops),
[string iteration](../nucleoid.spec.md#string-iteration) and
[object entry loops with ordered rollback](../nucleoid.spec.md#object-entry-loops-with-ordered-rollback),
alongside the unchanged explicit-loop entries in the root specification.

### while

- `while <condition>:` repeats its body while the condition holds, under
  [NUC 63](nuc-0063.md). The body is an indented block, or statements on the
  same line separated by `;`.
- The condition is tested before every iteration with the truthiness of
  `if`, so a condition that is false at the start runs no iteration.
- Each iteration's assignments, and the propagation they cause, are visible
  to the rest of the body and to the next test.
- The loop runs once, imperatively, like `for`: nested conditions and blocks
  are part of that one-shot body, nothing is filed in the graph, and the loop
  does not run again when its inputs change later.
- The body runs in the enclosing scope, like an `if` branch, with no scope
  frame per iteration. That is where it differs from a `for` body, whose
  frame per iteration shadows a name that the enclosing function or loop body
  already assigned. Parameters, enclosing loop variables, `this`/`self` and
  `return` behave as in `for` bodies.
- Each run of a loop stops after at most 100,000 iterations: a condition that
  still holds then raises `TypeError("Loop did not terminate")` and the
  transaction rolls back. Each run of a nested loop has its own budget, so
  every run terminates, but nested loops multiply and the bound is not a cap
  on total work.
- `while` is not reserved. `while NAME: TYPE`, `while assert: ...`, a
  condition whose last token is a bare `lambda`, and colon-free or braced
  headers keep their accepted readings as separate statements. A condition
  that is the name `suppose`, or an assignment whose value is that name, as
  in `while suppose: ...`, keeps its accepted reading too: `while` followed
  by a `suppose` block.

```
savings = Account(100)
years = 0
while savings.balance < 150:
    savings.balance = savings.balance * 1.1
    years = years + 1
```

- `years` is `5`; changing `savings.balance` later does not run the loop again.

Normative detail: [while loops](../nucleoid.spec.md#while-loops).

---

## 9. Functions

- `def` defines a function in the state; a function with no `return` yields `null`.
- `def name(params) => expression` is exactly a named block containing `return expression`; methods and parameter annotations use the same form.
- [NUC 57](nuc-0057.md) accepts `->` in place of `=>` in every named and anonymous arrow, and Python-style `lambda` expressions; see [Thin arrows and Python-style lambdas](#thin-arrows-and-python-style-lambdas).
- [NUC 89](nuc-0089.md) accepts `function name(params) { body }` as an optional spelling of `def name(params):` with the same statements; see [Named function declarations](#named-function-declarations).
- Accepted [NUC 32](nuc-0032.md) optionally places that expression in a genuinely indented body after `=>`; it preserves the same sole-return block and every accepted parameter spelling.
- [NUC 18](nuc-0018.md) specifies `def name amount => expression` as exactly `def name(amount) => expression` for one untyped ordinary identifier parameter, including methods and `init`.
- [NUC 19](nuc-0019.md) specifies `def name amount:` followed by an indented suite as exactly `def name(amount):` with that same suite.
- [NUC 20](nuc-0020.md) specifies `def name amount: int => expression` as exactly `def name(amount: int) => expression` for one ordinary identifier parameter and one existing ordinary-identifier annotation, including methods and `init`.
- [NUC 24](nuc-0024.md) describes `def name amount: int:` followed by a real indented suite as exactly `def name(amount: int):` with that same suite, including methods and `init`.
- [Accepted NUC 27](nuc-0027.md) adds optional `def name left, right => expression`, exactly equivalent to `def name(left, right) => expression`, including methods and `init`.
- [NUC 28](nuc-0028.md) adds optional `def name left, right:` with a real indented body, exactly equivalent to `def name(left, right):` with that same body, including methods and `init`.
- [NUC 29](nuc-0029.md) adds optional typed/mixed multiple-parameter named arrows, for example `def name left: int, right: int => expression`, exactly equivalent to `def name(left: int, right: int) => expression`, including methods and `init`.
- [NUC 30](nuc-0030.md) separately adds optional typed/mixed multiple-parameter indented blocks, for example `def name left: int, right:` with exactly the body of `def name(left: int, right):`, including methods and `init`.
- The legacy comma-bearing multiple-parameter arrow guard requires the complete immediate sequence `Identifier (Colon Identifier)? (Comma Identifier (Colon Identifier)?)+ Arrow` on the same logical header line. At least one annotation identifies NUC29's new typed/mixed forms; entirely untyped comma lists remain NUC27 behavior. The linear non-consuming guard allocates nothing and skips no separators, indentation or comment-induced logical breaks.
- [NUC 37](nuc-0037.md) adds an additional, optional untyped bare-arrow guard: after unchanged `Def Identifier`, the entire prefix must be `Identifier Identifier Identifier* Arrow`. Require at least two ordinary names with no annotations or commas, consuming nothing, allocating no parameter names and skipping no logical layout. This does not replace or relax NUC27's comma grammar or any typed/mixed/block guard.
- The multiple-parameter block guard shares the complete-list scan: NUC28's untyped lists and NUC30's separate typed/mixed lists may end in the suite `Colon`. Its complete immediate grammar is `Identifier (Colon Identifier)? (Comma Identifier (Colon Identifier)?)+ Colon`. Require immediate `Newline` or `Indent` and then a real `Indent` after only normal newline/semicolon layout; `:;` never qualifies. The final colon and all layout/body tokens are left for the existing suite parser.
- A final untyped parameter's following colon belongs to the suite, not the parameter's annotation parser. A final typed parameter has a separate annotation colon/name and suite colon. Failed complete lookahead consumes nothing; existing malformed parenthesized annotations keep their diagnostics.
- The single-parameter typed-arrow guard requires immediate `Identifier`, `Colon`, `Identifier`, `Arrow` tokens after the required definition name. The parameter name and `type_name: Some(annotation)` metadata are unchanged; the annotation is not a resolved variable or a new runtime type constraint.
- The typed-block guard requires immediate `Identifier`, `Colon`, `Identifier`, `Colon` tokens. The next token is `Newline` or `Indent`, and the first other token after only newline/semicolon layout is `Indent`; the guard consumes nothing before the existing parameter and suite parsers.
- For the single-parameter untyped forms, a bare identifier must be immediately followed by `=>`, or by `:` whose body starts with a logical newline/indent and supplies a real `Indent` after normal newline/semicolon layout. The colon guard only looks ahead; the existing suite parser consumes the body.
- No logical newline, semicolon or comment-induced logical break is skipped within a bare header. Blank/comment lines before an indented body retain the existing layout rules.
- All parenthesized definitions remain available, including `def add(left, right) => left + right`, `def add(left: int, right: int) => left + right` and typed/untyped `def add(left, right):` alternatives with the same indented body. Nullary, multiline-parameter, inline and brace-suite definitions keep their parentheses, as do calls; typed single/multiple-parameter indented blocks have the optional spellings below.
- Bare inline/braced or missing-indent blocks, trailing bare commas, bare defaults, destructured or bare variadic parameters, dotted/union/generic/nullable/keyword/dollar/literal annotations, anonymous colon blocks, return annotations and call-parenthesis omissions are not added. Parenthesized scalar defaults and final rest parameters are specified separately below.
- NUC18/20 single-parameter arrows and NUC19/24's exact untyped/typed indented-colon guards are unchanged; named return-block lowering, parameter metadata/order, argument binding, runtime behavior and canonical rendering remain those of the parenthesized counterparts.
- Braces after a named arrow are object literals, not statement blocks. Parenthesized expressions may span physical lines, starting either on the header line or in the optional NUC32 real-indented body.
- A call in an assignment depends on the function and its arguments.
- Redefining a function updates existing dependents.
- A named function and a variable binding under the same name replace one another; reads and calls both use the latest binding.
- Named function references stay live when aliased, returned, or passed as callbacks.
- Static Math method reads are not callable references; `Math.clamp` retains
  that limitation, as specified by [NUC 21](nuc-0021.md).
- Accepted [NUC 33](nuc-0033.md) keeps `Math.smoothstep` subject to the same
  unsupported method-read limitation.
- Accepted [NUC 35](nuc-0035.md) keeps `Math.smootherstep` subject to that same
  limitation; it adds only a static call, not a callable method value.
- Calls through variables, list items, and object properties depend on the callable's storage location.
- A null callable, an unassigned callable property, or a callable deleted during propagation leaves its dependent result null until rebound. Other non-callable values raise `TypeError`.
- `.value` freezes the function definition, not the state read when its body executes.
- Functions may recurse and may have multiple `return` statements.

### Literal parameter defaults

- Explicitly parenthesized user formals allow `name = literal` and
  `name: Type = literal` in named block/arrow functions, function expressions,
  parenthesized arrows, methods, class headers and adopted `init` methods.
- Only absent positions/constructor keys use defaults. Explicit Null,
  Undefined, false, zero, empty Strings and other supplied values retain their
  existing evaluation/value semantics; missing nondefaulted slots still bind
  Null. Required slots may follow defaults, annotations remain non-enforcing,
  and fixed-only signatures still evaluate and ignore extra arguments.
- Allowed data are finite numeric literals with one optional sign (including
  exact `-0`), quoted Strings, Booleans and null with its existing aliases.
  Arrays, objects, regexes, templates, references, calls, grouped/compound
  expressions and nonfinite numeric defaults are rejected, not evaluated.
- An unsupported/missing literal raises
  `SyntaxError("Expected a scalar literal parameter default")`; a nonfinite
  numeric token raises `SyntaxError("Parameter default must be a finite number")`.
  A scalar followed by anything other than `,` or `)` raises
  `SyntaxError("Expected ',' or ')' after parameter default")`.
- The effective literal initializes the fresh Scope slot at version zero.
  Functions, aliases, callbacks, methods and function Values share binding;
  positional/named/mixed/inherited constructors and `super` use the selected
  signature without changing strict labels, evaluation order, single
  evaluation, receiver capture or parent/class-resolution timing.
- Live redefinition and rollback retain their existing State/graph boundary.
  Frozen/historical function Values retain their own definition/defaults.
  Locals remain nontransactional and noncapturing; constructor/invocation
  frame isolation, snapshots, namespace-history restrictions and serializers
  are unchanged.
- Bare/comma-free signatures gain no default syntax. Function argument
  assignments are still positional expressions, not named arguments.
  Automatic named constructors without formal slots and native built-ins
  are unchanged. See the authoritative
  [literal parameter defaults](../nucleoid.spec.md#literal-parameter-defaults)
  contract and `Literal parameter defaults preserve supplied names` in
  `tests/reference.md`.

### Final rest parameters

- [NUC 74](nuc-0074.md) adds an optional final `...name` or
  `...name: Type` to explicitly parenthesized user-defined signatures:
  named functions, function expressions, arrows, methods, class headers
  and adopted `init` methods. Bare signatures and lambdas gain no rest form.
- Fixed slots retain their defaults, omissions, annotations and behavior.
  The rest slot starts at version zero with a fresh list of surplus
  positional values, in source order, or `[]`. A list argument remains one
  element; no call-site spread, deep copy, closure or new callback argument
  is introduced. Receivers never enter the list.
- Require one final rest slot, without a default and with a name distinct
  from fixed formals. A trailing comma is allowed. Other fixed-name
  duplication retains its accepted behavior.
- Constructor labels still address fixed slots only; named-only calls bind
  an empty rest list, and mixed calls keep the ordinary fixed-prefix binding.
  A named rest argument raises
  `TypeError("Rest parameter '<name>' cannot be a named constructor argument for <class>")`
  before any values run. Other label errors and inherited/`super` selection,
  `init` adoption, named pinning and delayed positional selection are unchanged.
- Arguments still run once, left to right, per invocation. Fresh local
  lists shadow state without becoming graph nodes or captures; contained
  objects retain identity. Live callers, frozen/historical Function Values,
  `.value` settlement, nontransactional locals and State/graph rollback keep
  their existing boundaries. Fixed-only signatures and their canonical
  rendering remain unchanged.
- Invalid rest signatures raise `SyntaxError("Rest parameter must be last")`,
  `SyntaxError("Rest parameter cannot have a default")` or
  `SyntaxError("Rest parameter '<name>' duplicates a fixed parameter")`.
  See the authoritative [rest contract](../nucleoid.spec.md#final-rest-parameters)
  and `Final rest parameters collect ordered surplus arguments` in
  `tests/reference.md`.

```
def collect(head = 1, ...tail) => [head, tail]
assert(collect(), [1, []])
assert(collect(2, 3, 4), [2, [3, 4]])
```

```
def generate number => number * 10

random = 10
number = generate(random)
```

- `100`; after `random = 20`, `200`.

```
def double amount => amount * 2
```

- `double(3)` is `6`; `double(5)` is `10`, with the same named return-block AST and canonical rendering as `def double(amount) => amount * 2`.

### Bare multiple parameter named arrows

```
scale = 1
def area width height => width * height * scale
width = 3
height = 4
result = area(width, height)
assert(result, 12)
scale = 2
assert(result, 24)
width = 5
assert(result, 40)
```

- This is the executable reference's exact paired `Nucleoid preserves bare multiple parameter arrow relationships` example using NUC37's optional comma-free header; `def area width, height => width * height * scale` and `def area(width, height) => width * height * scale` remain valid and behavior-equivalent.
- Reuse the existing named return block and expression-body parser, with no new lexer/public AST/runtime types, capture, parameter-count bound or canonical spelling.
- `def f left, right, => left`, missing types such as `def f left, right: => left`, multiple-parameter blocks without real indentation, bare inline/braced suites and logical header breaks retain `SyntaxError("Expected '(' but found identifier 'left'")` at line 1, column 7; unsupported prefixes are not consumed. NUC30 separately adds real-indented typed/mixed blocks without changing this arrow guard.
- `def f left, right =>` gives `SyntaxError("Unexpected end of input")` at line 1, column 21; a final newline without a real body gives `SyntaxError("Unexpected end of line")` at line 2, column 1. Named-arrow braces remain objects; the existing inline body beginning with `(` may span physical lines. NUC32 separately permits the real-indented body below.
- Full grammar and error positions are in the authoritative [bare multiple-parameter named arrows](../nucleoid.spec.md#bare-multiple-parameter-named-arrows) section.

### Comma-free bare named arrows

```
scale = 1
def area width height => width * height * scale
width = 3
height = 4
result = area(width, height)
assert(result, 12)
scale = 2
assert(result, 24)
width = 5
assert(result, 40)
```

- This is the exact executable `Nucleoid preserves comma-free named arrow relationships` counterpart in `tests/reference.md`, paired with the unchanged accepted comma-bearing program.
- Consume only checked untyped names and reuse the entire named-arrow body path: ordered untyped parameters and the same named sole-return block, canonical source, graph and runtime. NUC32's indented expression, leading objects and header-line parentheses remain available.
- Methods, `init`, nesting/redefinition, binding/arity, receivers/super/adoption, noncapture, locals, aliases/snapshots, reactive dependencies, transactions/rollback and resource limits are unchanged; no new parameter cap, lexer, public AST or API is added.
- This optional arrow form does not add mixed comma/whitespace separators, comma-free typed/mixed lists, parenthesized or anonymous comma omission, class/call changes, defaults/rest/destructuring, return annotations or bare nullary definitions. NUC37 itself adds no colon suites; the separate NUC40 section below adds their wholly untyped real-indented counterpart. Existing comma/typed/parenthesized/block forms remain supported.
- Unsupported prefixes such as `def f left right, third => left`, `def f left right: int => left` and `def f left right: return left` fall through at the original `left`, retaining `SyntaxError("Expected '(' but found identifier 'left'")` at line 1, column 7.
- Exact `def f left right =>` gives `SyntaxError("Unexpected end of input")` at line 1, column 20; a final newline without a real body gives `SyntaxError("Unexpected end of line")` at line 2, column 1. Body grammar and diagnostics remain inherited; only physical columns shortened by an omitted comma move.
- Full grammar, exact errors and executable traces are in the authoritative [comma-free bare named arrows](../nucleoid.spec.md#comma-free-bare-named-arrows) section. The implementation was independently accepted with no per-case token increase.

### Bare typed single parameter named arrows

```
def increment amount: int => amount + 1
def text value: str => value

assert(increment(4), 5)
assert(text("ok"), "ok")
```

- This is the executable reference's paired typed-arrow example; `def increment(amount: int) => amount + 1` and `def text(value: str) => value` remain valid and behavior-equivalent.

### Bare typed multiple parameter named arrows

```
scale = 2
def area width: number, height: number => width * height * scale
result = area(3, 4)
assert(result, 24)
scale = 3
assert(result, 36)
```

- This is the executable reference's exact paired `Nucleoid preserves bare typed multiple parameter arrow relationships` example; `def area(width: number, height: number) => width * height * scale` remains valid and exactly equivalent.
- Any annotation positions may mix with untyped identifiers in a list of at least two parameters; `def scaled left: number, right, factor: number => (left + right) * factor` keeps each existing `Some(annotation)`/`None` in order. Only ordinary `Colon Identifier` annotations qualify.
- The complete-list guard consumes nothing before the existing parameter parser and named return-block lowering. It adds no type checking, capture, parameter cap, public AST/runtime/generator/API or resource-bound change.
- `def f left: int, right, => left`, missing annotations, logical breaks and bare inline/braced blocks retain `SyntaxError("Expected '(' but found identifier 'left'")` at line 1, column 7. NUC29 itself adds no blocks; NUC30 separately extends the shared scanner to typed/mixed blocks with real indentation. NUC28 untyped blocks and all single guards remain exact.
- `def f left: int, right =>` gives `SyntaxError("Unexpected end of input")` at line 1, column 26; a final newline without a real body gives `SyntaxError("Unexpected end of line")` at line 2, column 1. Leading named-arrow braces remain objects; a parenthesis beginning on the header line may continue over physical lines. NUC32 separately permits a real-indented expression without changing this header guard.
- Arity/shadowing, null callable deferral, non-callable errors, receivers/init/super, reactivity, live aliases, `.value` snapshots, redefinition, graph dependencies, rollback and inherited noncapturing errors are unchanged.
- Full grammar, exact errors and positions are in the authoritative [bare typed multiple-parameter named arrows](../nucleoid.spec.md#bare-typed-multiple-parameter-named-arrows) section.

### Indented named arrow expressions

```
scale = 2
def area width: number, height: number =>
    width * height * scale
result = area(3, 4)
assert(result, 24)
scale = 3
assert(result, 36)
```

- This is the exact executable `Nucleoid preserves indented named arrow relationships` counterpart in `tests/reference.md`.
- It is exactly the accepted colon spelling `def area width: number, height: number:` with the sole body statement `return width * height * scale`; both that form and inline `=> width * height * scale` remain valid.
- Preserve parenthesized nullary/single/multiple/annotated lists and every accepted bare single/multiple spelling, including methods, nested definitions and `init`. Bare-nullary `def f =>` and `def f:` remain invalid.
- Non-consumingly require immediate newline/indent and then a real indent after only newline/semicolon layout. A same-line semicolon after `=>` cannot qualify, even before a later indent. Failed qualification preserves the original inline error and position.
- Skip normal body layout, parse one unchanged expression, skip trailing newline/semicolon layout and require its closing dedent. Semicolon-only layout lines and trailing semicolons are permitted; `value; other` or another body statement is rejected, not silently dropped.
- This is not an arbitrary statement suite or an implicit last-statement return. Colon suites and anonymous arrow/function bodies retain all their existing grammar; anonymous arrows gain no indented-body path.
- Blank/comment lines, CRLF/tabs and parenthesized multiline expressions use existing layout. Named braces remain object expressions; anonymous braced arrows remain statement blocks.
- The native REPL buffers a lexical named header ending in its only actual arrow, including a trailing comment. Blank-line commit and stdin EOF execute the complete input; structured errors allow subsequent recovery. Arrow text in literals/comments, inline arrows and direct/anonymous expressions keep their old interaction.
- Exact `def f x =>\nx` still gives `SyntaxError("Unexpected end of line")` at line 2, column 1. Exact `def f x =>\n    x\n    x + 1` instead gives `SyntaxError("Expected the end of a block but found identifier 'x'")` at line 3, column 5.
- Binding, annotations, receivers/init/super/adoption, null deferral, non-callable errors, noncapture, live references/snapshots, state propagation, graph/reasoning, transactions and resource bounds remain unchanged.
- Full layout, exact errors, legacy alternatives and executable traces are in the authoritative [indented named arrow expressions](../nucleoid.spec.md#indented-named-arrow-expressions) section.

### Bare single parameter indented block definitions

```
def double amount:
    offset = 1
    return amount * 2 + offset

assert(double(3), 7)
assert(double(5), 11)
```

- The same suite under `def double(amount):` remains valid; literal locals, multiple statements and explicit returns use the existing block behavior.
- A body without `return` yields `null`, never an implicit last-expression return.
- `def f amount:` without an indented body, `def f amount: int:` at EOF without a body, and bare inline/brace suites retain `SyntaxError("Expected '(' but found identifier 'amount'")` at line 1, column 7.

### Bare typed single parameter indented block definitions

```
def double amount: int:
    offset = 1
    return amount * 2 + offset

assert(double(3), 7)
assert(double(5), 11)
```

- This is the executable reference's exact paired example; the same suite under `def double(amount: int):` remains valid with the same complete AST and canonical rendering.
- The ordinary annotation remains `Some("int")` metadata, not a lookup, dependency or new runtime type constraint; missing/null/extra/string arguments retain their existing binding.
- Require the immediate header prefix and a real indent. Blank/comment lines and normal semicolons after the initial body newline are allowed; header-line breaks/semicolons, inline/braced bare suites and comments-only bodies do not qualify.
- The NUC19 untyped suite colon still belongs to the suite, never to an annotation. NUC18/20 arrows, explicit typed alternatives, class/anonymous lists, receivers/init/super, no implicit return, live references/snapshots, transactions and resource limits are unchanged.
- Inherited parameter-dependent local and nested noncapturing failures remain paired, not repaired by this spelling.

### Bare multiple parameter indented blocks

```
scale = 2
def area width, height:
    return width * height * scale
result = area(3, 4)
assert(result, 24)
scale = 3
assert(result, 36)
```

- This legacy colon example remains equivalent to `def area(width, height):` with the same body. The executable `Nucleoid preserves bare multiple parameter block relationships` counterpart now uses NUC32's equivalent indented arrow for this sole return; the colon syntax is still supported.
- Two or more untyped ordinary identifiers retain their order and `type_name: None`; the complete named block AST and canonical rendering are identical to the parenthesized form. There is no parameter-count cap.
- The body uses the existing suite grammar, including multiple statements, nested definitions, branches and existing inner inline/braced statements. Explicit returns, bare returns and no return keep their existing semantics; the latter two return `null`.
- `def f left, right:` without a real indent, malformed annotations, trailing bare commas, inline/braced bare suites, `:;` and logical header breaks keep `SyntaxError("Expected '(' but found identifier 'left'")` at line 1, column 7. NUC30 separately adds typed/mixed multiple indented headers, not inline/braced bare suites.
- After the qualifying header, four-space-indented `return left +)` keeps `SyntaxError("Unexpected ')'")` at line 2, column 19; `return @` keeps `SyntaxError("Unexpected character '@'")` at line 2, column 12.
- Full grammar, exact errors and positions are in the authoritative [bare multiple-parameter indented block definitions](../nucleoid.spec.md#bare-multiple-parameter-indented-block-definitions) section. Receiver/init/super behavior, shadowing, arity, live references, definition snapshots, redefinition, dependencies, graph keys, transactions and resource limits are unchanged.

### Comma-free bare indented blocks

```nuc
scale = 1
def area width height:
    return width * height * scale
width = 3
height = 4
result = area(width, height)
assert(result, 12)
scale = 2
assert(result, 24)
width = 5
assert(result, 40)
```

- [NUC 40](nuc-0040.md) adds this exact executable `Nucleoid preserves comma-free block relationships` counterpart in `tests/reference.md`. `def area width, height:` and `def area(width, height):` with the same body remain valid and equivalent; the legacy comma-bearing control above is retained.
- Require the complete `Identifier Identifier Identifier* Colon` prefix after `def` and the ordinary name, with at least two wholly untyped identifiers and every parameter comma omitted or none. Share the identifier-run scan with the arrow guard without changing its arrow-only meaning; allocate/consume/skip nothing on failure.
- Keep the required final colon for unchanged `suite()`. Its next token must be `Newline` or `Indent`, then only accepted newline/semicolon separators may precede a genuine `Indent`. Same-line `:;`, missing colon/indent and bare inline/braced suites do not qualify.
- Ordered untyped parameters, full named block AST/metadata/canonical source, explicit/bare/no returns, receiver/init/super/adoption behavior, argument binding/shadowing/noncapture, aliases/snapshots and all live graph/rollback behavior are unchanged. This does not change inherited method lookup.
- Typed/mixed lists keep commas. Mixed omission, parenthesized/anonymous/call/class comma omission, defaults/rest/destructuring, return annotations and bare nullary headers are not added; all accepted forms, including NUC37 arrows, remain available.
- `def f left right:` with no actual indented body, `def f left right: return left`, mixed separators/types and logical header breaks retain `SyntaxError("Expected '(' but found identifier 'left'")` at line 1, column 7. Body errors retain their physical positions.
- The native REPL buffers colon-suite headers with trailing comments until blank-line commit or EOF, with matching results and graph output for comma-free and comma-separated headers.
- Full grammar, exact errors and normative traces are in the authoritative [comma-free bare indented blocks](../nucleoid.spec.md#comma-free-bare-indented-blocks) section.

### Bare typed multiple parameter indented blocks

```
scale = 2
def area width: number, height: number:
    return width * height * scale
result = area(3, 4)
assert(result, 24)
scale = 3
assert(result, 36)
```

- This legacy colon example remains exactly equivalent to `def area(width: number, height: number):` with the same body. The executable `Nucleoid preserves bare typed multiple parameter block relationships` counterpart now uses NUC32's equivalent indented arrow for this sole return; the colon syntax is still supported.
- Two or more ordinary identifier parameters may annotate any position using only the existing `Colon Identifier` metadata. At least one annotation describes NUC30's new forms; entirely untyped lists retain NUC28 behavior. Arbitrary undeclared ordinary annotation names need no lookup, dependency or enforcement.
- The complete non-consuming allocation-free scan skips no logical header breaks. The suite colon needs immediate `Newline` or `Indent`, then a real `Indent` after only normal newline/semicolon layout; `:;` never qualifies. Blank/comment lines remain normal suite layout.
- Consume names and optional annotations in order, leaving the final suite colon and the entire body/layout for the unchanged suite parser. A final untyped parameter's colon is not an annotation; a final typed parameter needs both its annotation colon/name and suite colon.
- Preserve the full named block AST, metadata and canonical rendering, including local assignments, multiple statements, branches, nested definitions and ordinary inner inline/braced statements. Explicit, bare and absent returns retain their meanings; bare/absent returns yield `null`, never an implicit final-expression return.
- `def f left: int, right:` without a real indent, inline/braced bare suites, trailing commas, `:;`, logical header breaks and unsupported type/parameter syntax keep `SyntaxError("Expected '(' but found identifier 'left'")` at line 1, column 7. `def f(left: int, right:):` keeps `SyntaxError("Expected a type but found ')'")` at line 1, column 25.
- After a qualifying header, four-space-indented `return left +)` keeps `SyntaxError("Unexpected ')'")` at line 2, column 19; `return @` keeps `SyntaxError("Unexpected character '@'")` at line 2, column 12. Other lexical/body/runtime errors and positions retain their existing rules.
- Keep NUC24 single typed blocks, NUC28 untyped blocks, NUC29 arrows, every parenthesized alternative and calls. Arity/unchecked arguments, shadowing, null callable deferral/non-callable errors, noncapturing scopes, receivers/init/super/adoption/identity, reactive inputs, aliases/snapshots, redefinition, graph dependencies/reasoning, transactions/rollback and existing bounds remain unchanged.
- Full grammar and exact errors are in the authoritative [bare typed multiple-parameter indented block definitions](../nucleoid.spec.md#bare-typed-multiple-parameter-indented-block-definitions) section.

### Lambdas

- Three equivalent forms; parameters shadow outer names, including outer functions when the parameter is null or non-callable.

```
list.find(function(element) { return element == 3 })
list.find(element => { return element == 2 })
list.find(element => element == 1)
```

- Calls over class lists depend on the population and the properties they read.

### Thin arrows and Python-style lambdas

```
factor = 2
items = [1, 2, 3]
fat = items.map(item => item * factor)
thin = items.map(item -> item * factor)
python = items.map(lambda item: item * factor)
assert(thin, fat)
assert(python, fat)
factor = 10
assert(thin, [10, 20, 30])
assert(python, [10, 20, 30])
python
```

- This is the complete `Nucleoid accepts thin arrows and lambdas as arrow spellings` program in `tests/reference.md`, without its title and expected-return metadata; it returns `[10,20,30]`.
- [NUC 57](nuc-0057.md) makes adjacent `->` an exact alternative to `=>` wherever an arrow is accepted: anonymous single and parenthesized parameters, typed and literal-default parameters, braced statement bodies, and every named header (parenthesized, bare single/typed/multiple/typed-mixed/comma-free, NUC32's indented body, methods and `init`). Both spellings build the same AST and canonical source; `=>` stays canonical.
- `- >` with whitespace stays two tokens, and arrow text in strings, template text outside interpolations, regular expressions and comments is unchanged; an interpolation is an expression and accepts either arrow. Diagnostics name the arrow as written, for example `SyntaxError("Expected '(' but found '->'")` for `def f -> 1`.
- Return-annotation lookalikes are ordinary arrows: `def f() -> int` returns the value of `int`, and `def f() -> int:` with a body still fails at its colon with `SyntaxError("Unexpected ':'")`. Return annotations are not added.
- `lambda: expression`, `lambda name: expression` and `lambda first, second: expression` are exactly `() => (expression)`, `(name) => (expression)` and `(first, second) => (expression)`. Parameters are plain names; the body is one expression, where a leading brace is an object literal. Use a parenthesized arrow for annotations, defaults or statement bodies.
- `lambda` is not reserved. It stays an ordinary name at the top level of `if`/`else`/`elif`/`for` headers, subscript starts (the index or slice start inside `[...]`) and template interpolations; as a single parameter inside an object-literal value (NUC44's `{first: lambda next: 2}`); and as a single parameter whose colon starts a statement, because statement loops accept one directly after an expression: `name: Type` before a statement boundary remains a typed declaration, and `x = lambda assert: x, 1` remains an assignment followed by NUC45's `assert:` statement. Parenthesize the lambda or pass it as an argument there; a single parameter named `assert` never forms a lambda.
- Full grammar, exclusions and errors are in the authoritative [thin arrows and Python-style lambdas](../nucleoid.spec.md#thin-arrows-and-python-style-lambdas) section.

### Property selector arguments

```
class Item(name, price, stock):
    this.name = name
    this.price = price
    this.stock = stock
names = Item.map(.name)
dearest = Item.max(.price)
stocked = Item.count(.stock)
Item("pen", 3, 10)
ink = Item("ink", 9, 0)
assert(names, ["pen", "ink"])
assert(dearest, 9)
assert(stocked, 1)
ink.price = 2
ink.stock = 4
Item("cap", 5, 0)
assert(dearest, 5)
assert(stocked, 2)
names
```

- This is the complete `Nucleoid reads properties with selector arguments` program in `tests/reference.md`, without its title and expected-return metadata; it returns `["pen","ink","cap"]`.
- [NUC 79](nuc-0079.md) lets a call argument written `.name`, or a dotted path such as `.to.city`, stand for the one-parameter function that reads that path: `Item.max(.price)` is `Item.max(item -> item.price)`. A selector is a whole argument of a parenthesized argument list, the `.NAME` pairs directly followed by `,` or `)`, in any call, method call, constructor call, `new` call or `super(...)`.
- Names are those member access accepts, keywords included in their canonical spelling, so `.self` reads `this`. The selector reads only its argument, exactly as the lambda would: missing steps read undefined, a declaration that reads through `null` settles to `null`, `.value` stays the snapshot read, a missing argument is `null` and extra arguments are ignored.
- Selectors stay live and roll back like the lambda they abbreviate. They render as written, so `why` explains `dearest = Item.max(.price)`; anonymous lambdas keep rendering as `function()`.
- Anything else that starts with `.` keeps its error: `f(.a + 1)`, `x = .a`, `[.a]` and `assert: .a, 1` still raise `SyntaxError("Unexpected '.'")`. A selector names at most 127 properties. See the authoritative [property selector arguments](../nucleoid.spec.md#property-selector-arguments) section.

### Named function declarations

```
function scale(amount, factor = 2) {
    return amount * factor
}
class Box:
    function init(size) { this.size = size }
    function grown() { return scale(this.size) }
box = Box(3)
assert(scale(4), 8)
assert(box.grown(), 6)
function scale(amount, factor = 3) { return amount * factor }
assert(box.grown(), 9)
box.grown()
```

- This is the complete `Nucleoid accepts function as the named declaration spelling` program in `tests/reference.md`, without its title and expected-return metadata; it returns `9`.
- [NUC 89](nuc-0089.md) accepts the named function declaration of JavaScript, TypeScript and PHP: `function`, a name, a parenthesized parameter list and a braced body. `function scale(amount, factor = 2) { ... }` declares exactly the function that `def scale(amount, factor = 2):` declares with the same statements, and canonical source keeps writing `def`.
- The parameter list is that of a parenthesized `def`: annotations, literal defaults, a final rest parameter and, for `init` in a class body, NUC 87's `this.name` parameters. The body is the braced block of an anonymous `function (params) { ... }` literal; its `{` follows the parameters on the same line, and `{}` returns `null`.
- The declaration is a statement wherever a statement may start, including after another statement on the same line and inside function bodies. In an indented class body it declares a method, and `function init(...)` becomes the constructor, as `def init(...)` does.
- Declarations are not hoisted: a call before the declaration runs raises `ReferenceError("<name> is not defined")`, as for `def`. Redefinition, `.value`, scopes, `why`, `affects`, graph keys and transactions are also those of `def`.
- In an expression, `function` must still be followed by `(`: `g = function f(a) { return a }` raises `SyntaxError("Expected '(' but found identifier 'f'")`. After the parameters, any token other than `{` raises `SyntaxError("Expected '{' but found <token>")`, so arrows, colon suites and return annotations are not added. The line-oriented REPL buffers a declaration until its braces close. See the authoritative [named function declarations](../nucleoid.spec.md#named-function-declarations) section.

### List comprehensions

```
prices = [12, 30, 8]
limit = 10
picked = [price * 2 for price in prices if price > limit]
assert(picked, prices.filter(price => price > limit).map(price => price * 2))
assert(picked, [24, 60])
limit = 5
picked
```

- This is the complete `Nucleoid reads list comprehensions as map and filter` program in `tests/reference.md`, without its title and expected-return metadata; it returns `[24,60,16]`.
- [NUC 92](nuc-0092.md) accepts Python's list comprehensions. A list literal whose whole content is `EXPR for NAME in SOURCE`, optionally followed by one `if COND`, is exactly `SOURCE.map(NAME => EXPR)` or `SOURCE.filter(NAME => COND).map(NAME => EXPR)`, and `for NAME of SOURCE` is the same. Evaluation, liveness, `why`, `affects`, transactions and canonical source are those of the arrow spelling, which renders as `SOURCE.map(function())`.
- SOURCE is a list or a class's population, evaluated once. NAME is one plain identifier, local to the comprehension. EXPR is a full expression, including another comprehension; SOURCE ends before an `if` at its top level, so a Python-style conditional there needs parentheses; COND runs to `]`. Line breaks inside the brackets carry no layout.
- There is one `for` and at most one `if`, with no other items: a second clause or a trailing comma raises `SyntaxError("Expected ']' but found <token>")`. A NAME that is not an identifier raises `SyntaxError("Expected a name but found <token>")`, and a missing `in` or `of` raises `SyntaxError("Expected 'in' or 'of' but found <token>")`, so tuple targets are not added. Index brackets, `(x for x in xs)`, `f(x for x in xs)` and dict and set comprehensions keep their accepted errors.
- The lambda limits are inherited: the lambdas read variables and their own name but not an enclosing function's or lambda's parameters, so a nested comprehension reads only its own names, `this` inside a method's comprehension raises `ReferenceError("this is not defined")`, and `[c for c in "abc"]` raises `TypeError("abc.map is not a function")`.
- Every input the accepted parser accepts keeps its parse: a source is parsed with comprehensions only after the accepted parse fails. See the authoritative [list comprehensions](../nucleoid.spec.md#list-comprehensions) section.

Full detail: [NUC 8](nuc-0008.md), [NUC 12](nuc-0012.md), [NUC 18](nuc-0018.md), [NUC 19](nuc-0019.md), [NUC 20](nuc-0020.md), [NUC 24](nuc-0024.md), [NUC 27](nuc-0027.md), [NUC 28](nuc-0028.md), [NUC 29](nuc-0029.md), [NUC 30](nuc-0030.md), [NUC 32](nuc-0032.md), [NUC 57](nuc-0057.md), [NUC 79](nuc-0079.md), [NUC 89](nuc-0089.md), and [NUC 92](nuc-0092.md).

---

## 10. Transactions

- A statement and every rule it triggers form one transaction.
- If anything throws, the whole transaction is rolled back and the state is exactly as before.
- `throw` raises a value: a string, a number, or a resolved reference.
- Blocks, including nested blocks, are single transactions.
- An instance rejected at creation is not registered and its name is not bound.
- Instances created earlier in a failing loop are rolled back too.
- A class-level rule that throws for any existing instance is not installed at all.
- Propagation that does not settle raises `TypeError("Propagation did not settle")` and rolls back the transaction.
- A caught failure restores the failing body's pending propagation and deletion tracking before the catch body runs. Work queued before the body is preserved, including across nested catches.
- A `suppose` block runs its statements on a savepoint that always rolls back, whether it ends normally, by a failure or by `return`; see [Hypotheticals](#hypotheticals).
- Rolling back restores the order of every object's entries as well as their values: a deleted property returns where it stood, an added one is removed, and an instance keeps `id` first, at every nested savepoint ([NUC 72](nuc-0072.md)). The order of graph nodes and edges that `model |> why` reflects is unchanged.

```
a = 5

if a > 5:
    throw 'INVALID_VALUE'
```

- Assigning `a = 6` throws; afterwards `a` is `5`.

### Optional colons on indented exception suites (Accepted NUC 46)

- Accepted [NUC 46](nuc-0046.md) allows `try` and a complete existing `catch`
  header to omit the suite colon independently, only before a genuine indent.
  Colon-bearing indented, inline, semicolon-separated and braced forms remain
  valid. The whole `Stmt::Try`, default `error` binding and runtime are shared.
- This is the paired executable reference entry
  `Nucleoid preserves catch bindings and mixed exception suite forms`,
  without its title comment. The inline pair keeps both colons; the two
  indented pairs demonstrate omission in each direction.

```
try: throw "INLINE"
catch issue: assert(issue, "INLINE")
try
    throw "PAREN"
catch (problem):
    assert(problem, "PAREN")
try:
    throw "DEFAULT"
catch
    assert(error, "DEFAULT")
```

- Complete parameter parsing precedes qualification. Default, ordinary-name
  and parenthesized-name headers retain their accepted meanings; no filters,
  catch expressions, multi-catch, `finally` or new scope/throw behavior.
- Existing lexer layout and `has_indented_body` decide whether a real body
  follows, before any layout is consumed. Blank/comment-only lines or a
  header-line semicolon do not suffice. Other header grammars are unchanged.
- The native REPL buffers a complete lexical bare `try` header, including a
  trailing comment, through its catch until blank-line commit or EOF. A
  standalone `catch`, literal/comment text, identifier prefix or malformed
  header does not start that buffer; committed errors are still reported.
- Savepoints restore failed-body state and pending work before recovery.
  Dependencies, rules, receivers, returns and reasoning remain the accepted
  behavior. Already implemented exceptions are baseline, not a pending NUC
  dependency. Exact grammar and errors are in the
  [root contract](../nucleoid.spec.md#colon-free-indented-exception-suites).
- Independent evaluation accepted all 611 paired cases: eight use fewer lexer
  tokens, 603 are unchanged, and none increased. All 2,763 protected assertions
  executed and all 50 explicit expected returns matched.

Full detail: [NUC 9](nuc-0009.md), [NUC 46](nuc-0046.md).

---

## 11. Built-in Objects

- Built-in calls participate in the dependency graph like any other expression.
- Mutating a list re-evaluates its dependents.
- `push`, `pop`, `shift` and, under [NUC 58](nuc-0058.md), `sort` change a
  writable list receiver in place; see
  [In-place list sorting](#in-place-list-sorting).
- `Math.max` and `Math.min` propagate `NaN`, including when an argument cannot be coerced to a number.
- Accepted [NUC 56](nuc-0056.md) adds only static `Math.exp2(value)`, computing
  exactly the accepted `Math.pow(2, value)` host powf operation after intrinsic
  first-input conversion and the existing supplied-Undefined gate. The
  [base-two contract](../nucleoid.spec.md#reactive-base-two-exponentiation)
  defines numeric equivalence, ignored extras and inherited reactive behavior.
- The [NUC 50 candidate](nuc-0050.md) adds only static
  `Math.toRadians(value)`: intrinsically convert the first input, compute
  `factor = PI / 180.0`, then multiply by that factor, never multiply by PI
  before division. The [degree-conversion contract](../nucleoid.spec.md#reactive-degree-to-radian-conversion)
  defines numeric bits, omission, ignored extras and inherited call behavior.
- [NUC 21](nuc-0021.md) specifies static `Math.clamp(value, lower, upper)`;
  its proposal header records its status.
- Numeric clamping is exactly the ordered `Math.min(Math.max(value, lower), upper)`
  after existing numeric coercion, not range validation: reversed bounds yield
  the upper bound, equal bounds and infinities retain that composition's result,
  and signed zeros follow the same ordered min/max folds.
- Numbers, literal `null`, booleans, trimmed numeric strings, empty/singleton
  lists and dates retain their existing conversions; nonnumeric conversions
  and numeric `NaN` in any of the first three positions yield numeric `NaN`.
- Each omitted first/second/third argument supplies numeric `NaN`. Extra values
  are ignored numerically, but every argument expression is evaluated left to
  right and its reads remain dependencies.
- Any supplied `Undefined` value, including an extra, defers the Math call;
  a dependent assignment stores `null` until the missing property is defined.
  Existing live-null assignment handling is also unchanged; literal `null`
  still coerces numerically to zero.
- Strict unknown identifiers still throw `ReferenceError("<name> is not defined")`
  before dispatch. Existing transaction rollback and recovery remain intact.
- Static namespace shadowing and ordinary `.Math` members are unchanged;
  `Math.clamp` read as a value stays unsupported/undefined, not a callable reference.
- This capability adds no bounds error, type enforcement or grammar change;
  other Math functions/constants and their exact errors are unchanged.
- [NUC 22](nuc-0022.md) specifies static `Math.lerp(start, end, weight)`;
  its proposal header records its status.
- Interpolation intrinsically converts the first three values numerically, then
  computes `delta = end - start`, `scaled = delta * weight`, `start + scaled`,
  using exactly those ordinary `f64` operations in order. It does not call a
  shadowable `Number()` function or concatenate strings.
- Weights below zero or above one extrapolate: there is no weight clamping,
  endpoint validation/reordering, midpoint/endpoint shortcut, fused operation,
  higher precision or alternative weighted formula.
- Equal/reversed endpoints, overflow, cancellation, infinities, numeric `NaN`
  in any position and signed zeros follow that ordered arithmetic. In particular,
  `0 + Infinity * 0` and `Infinity - Infinity` yield numeric `NaN`, not endpoints.
- The existing numeric conversions above apply, including literal `null` as zero,
  booleans, trimmed strings, empty/singleton lists and millisecond dates.
  Nonnumeric conversion yields numeric `NaN`, not a new type error.
- Every omitted first/second/third interpolation input supplies numeric `NaN`.
  Extras are ignored numerically, but all supplied expressions run exactly once
  left to right and all reads remain dependencies.
- The unchanged any-`Undefined` Math gate includes extras. Live-null and
  deferred-read assignment handling, class/property recovery, strict argument
  errors and transactional rollback take precedence as before.
- Static `Math` shadowing and ordinary `.Math` members retain their existing
  resolution. `Math.lerp` read as a value is unsupported/undefined, not a
  first-class callable or alias; no `Number` lookup dependency is introduced.
- `Math.clamp`, `min`, `max`, `round`, other Math members, syntax and public
  representations are unchanged. Unknown static methods and `Math.random()`
  retain their exact inherited errors and existing pre-dispatch undefined gate.
- [NUC 23](nuc-0023.md) specifies static `Math.saturate(value)` as exactly
  ordered `Math.clamp(value, 0, 1)`,
  through the existing maximum fold followed by the existing minimum fold.
- Convert the first value intrinsically with `Value::to_number`, using numeric
  `NaN` when omitted; do not call or track shadowable `Number` or `Math.clamp`.
- The existing numeric conversions, numeric `NaN`, infinities, subnormal
  values and signed-zero bits retain that composition's behavior, without
  host `f64::clamp`, input validation or endpoint shortcuts.
- Every supplied expression runs once left to right, including numerically
  ignored extras; their reads, effects and the any-`Undefined` gate remain.
- Literal-null conversion does not override live-null/deferred assignments.
  Class rules, deletion/recovery, strict errors and rollback remain unchanged.
- Static `Math` shadowing and ordinary `.Math` members retain their lookup.
  `Math.saturate` read as a value remains unsupported/undefined, not callable.
- All other Math members, accepted syntax and public representations stay
  unchanged.
- Accepted [NUC 33](nuc-0033.md) defines only static
  `Math.smoothstep(lower, upper, value)`. Bounds precede the value; this is
  deliberately not `Math.clamp`'s value/lower/upper order.
- After the existing any-supplied-`Undefined` gate, intrinsically convert lower,
  upper and value, in order, with `Value::to_number`; each omission is numeric
  `NaN`. Do not convert unused extras or invoke/track shadowable `Number` or
  `Math.saturate`.
- Compute `numerator = value - lower`, `denominator = upper - lower`,
  `normalize = numerator / denominator`, then clamp with the existing maximum
  fold followed by minimum against numeric zero and one.
- With that `t`, compute separate `square = t * t`, `scaled = 2.0 * t`,
  `falloff = 3.0 - scaled`, then `square * falloff`. Do not fuse, reorder,
  shortcut endpoints, clamp the polynomial afterward or validate ranges.
- Equal/reversed bounds, infinities, overflow, subnormals and signed zeros
  follow the accepted normalization and cubic composition. Expected numeric
  `NaN` uses classification; non-NaN bits, including zero bits, match the host
  composition rather than a portable sign promise.
- Literal `null` converts to zero intrinsically without overriding live-null
  or parameter/deferred assignment handling. Other nonnumeric conversions
  yield numeric `NaN`, not new type errors.
- Every provided expression, including extras, runs exactly once left to right;
  effects, reads and dependencies remain. A supplied `Undefined`, even an extra,
  defers before conversion; strict later unknown identifiers still throw.
- Namespace/local shadowing, ordinary `.Math` members, wrappers, methods,
  initializers, class rules, property deletion/recovery and rollback are
  unchanged. No first-class `Math.smoothstep` alias or new lookup is added.
- The intrinsic reuses the private numeric clamp without calling public Math
  methods. All other Math behavior, accepted syntax, APIs and inherited
  scope/resource boundaries remain unchanged.

| Object | Members |
| --- | --- |
| `Object` | `Object()` -- a plain object, not an instance of any type |
| `Boolean` | `Boolean(value)` |
| `Number` | `Number.MAX_INTEGER` |
| `Math` | `pow`, `sqrt`, `floor`, `round`, `max`, `min`, `clamp` ([NUC 21](nuc-0021.md)), `lerp` ([NUC 22](nuc-0022.md)), `saturate` ([NUC 23](nuc-0023.md)), `smoothstep(lower, upper, value)` (accepted [NUC 33](nuc-0033.md)), `smootherstep(lower, upper, value)` (accepted [NUC 35](nuc-0035.md)), `remap(start, end, targetStart, targetEnd, value)` (accepted [NUC 39](nuc-0039.md)) |
| `String` | `String.fromCharCode`; instance `charAt`, `length`, `lower`, `replace`, `replaceAll` (accepted [NUC 42](nuc-0042.md)), `repeat`, `split` |
| `Date` | `Date()`, `Date(string)`, `Date(number)`, `Date.now()`, `Date.parse()`; instance `getTime`, `getYear`, `toDateString` |
| `List` | `push`, `pop`, `filter`, `map`, `flatMap` (accepted [NUC 41](nuc-0041.md)), `reduce`, `find`, `some`, `every`, `slice`, `sort` (in place, [NUC 58](nuc-0058.md)), `join`, `length`, and the queries `at`, `includes`, `indexOf`, `lastIndexOf`, `findIndex`, `findLast`, `findLastIndex` and `concat` ([NUC 61](nuc-0061.md)), `sum` ([NUC 68](nuc-0068.md)), `max` and `min` ([NUC 69](nuc-0069.md)), `distinct` ([NUC 71](nuc-0071.md)), `count(predicate)` ([NUC 73](nuc-0073.md)), `mode` ([NUC 75](nuc-0075.md)), `maxBy` and `minBy` ([NUC 77](nuc-0077.md)), `average` ([NUC 80](nuc-0080.md)), and `median` ([NUC 84](nuc-0084.md)); `sum`, `max`, `min` and `mode` also read as `sum of list` ([NUC 75](nuc-0075.md) revision 2) |
| `Class` | the global list of declared types; `Class.length` |

```
prices = [10, 20, 30]
rate = 2
doubled = prices.map(p => p * rate)
```

- `doubled[2]` is `60`; after `rate = 3`, `90`; after `prices.push(40)`, `doubled[3]` is `120`.

- Class lists support the same functions and depend on the population. Ranges such as `Task[1:]` slice them, and `Task.list()` or `Task.all()` returns one as a list.

```
open = Task.filter(t => not t.done).length
```

- `all t in Task: t.done` and `any t in Task: t.done` state the claims of `every` and `some` directly, and they nest; see [Quantifiers](#quantifiers).

- Regular expressions support `.test(...)` and may be used in any condition.

### Reactive literal string replacement

- Accepted [NUC 42](nuc-0042.md) adds the string-instance method
  `replaceAll(search, replacement)`, without changing accepted syntax.
- Replace literal, non-overlapping matches left to right; replacement strings
  have no regex, dollar or backslash expansion. Return a new string.
- Empty search visits Unicode scalar boundaries, including start/end; an empty
  receiver has one boundary. Existing indexing and empty `split` use the same
  scalar model, not UTF-16 units.
- Snapshot the receiver before arguments; evaluate every supplied expression
  once left to right, including extras. Then coerce the first two values as
  `replace` does; omissions spell `"undefined"`.
- Default/literal values do not imply binding reads. Actual null/undefined
  reads keep inherited deferred assignment behavior, dependencies, skipped
  arguments on nullish receivers and recovery. No simulated reads are added.
- Global/property/class rules, ordinary scopes/noncapture, frozen/live functions,
  `.value`, transactional rollback and source positions remain unchanged.
- Checked output sizing or failed output reservation raises
  `TypeError("Replaced string is too large")`, without an arbitrary cap.
- Existing `replace` stays first-only; `split`, `join` and custom functions,
  properties or instance methods named `replaceAll` retain their old behavior.
  Native method properties are not first-class. Static `String.replaceAll`
  still raises `TypeError("String.replaceAll is not a function")` after arguments.

```
labels = ["first name", "last name", "first name"]
normalized = labels.map(label => label.replaceAll(" ", "_"))
assert(normalized, ["first_name", "last_name", "first_name"])
labels.push("display name")
assert(normalized, ["first_name", "last_name", "first_name", "display_name"])
labels = ["given name", "last name"]
assert(normalized, ["given_name", "last_name"])
```

- This is the paired executable entry `Nucleoid normalizes live text records`
  in `tests/reference.md`. The normative
  [root section](../nucleoid.spec.md#reactive-literal-string-replacement) and
  its five appended entries define results, Unicode, reactivity, class rules
  and failure rollback; native regressions cover the call/coercion boundaries.

### Numeric clamping keeps bounds live

```
reading = -3
floor = 0
ceiling = 10

bounded = Math.clamp(reading, floor, ceiling)

assert(bounded, 0)

reading = 12

assert(bounded, 10)

ceiling = 8

assert(bounded, 8)
```

- This is the sealed executable reference example in `tests/reference.md`.
- All three inputs remain live through the ordinary builtin call path.
- The `Reactive numeric clamping` section and its five paired entries in
  [`nucleoid.spec.md`](../nucleoid.spec.md#reactive-numeric-clamping) define the
  behavior; [NUC 21](nuc-0021.md) explains the deliberately ordered contract.

Full detail: [NUC 10](nuc-0010.md).

### Numeric interpolation keeps endpoints and weights live

```
start = 0
end = 20
weight = 0.25

position = Math.lerp(start, end, weight)

assert(position, 5)

end = 40

assert(position, 10)

weight = 0.5

assert(position, 20)
```

- This is the sealed executable reference example in `tests/reference.md`.
- The start, end and weight remain live through the ordinary builtin call path.
- The `Reactive numeric interpolation` section and its five paired entries in
  [`nucleoid.spec.md`](../nucleoid.spec.md#reactive-numeric-interpolation) define
  the behavior; [NUC 22](nuc-0022.md) explains the deliberately ordered contract.

### Unit-interval saturation keeps live fractions bounded

```
reading = -0.2
fraction = Math.saturate(reading)

assert(fraction, 0)

reading = 0.6

assert(fraction, 0.6)

reading = 1.4

assert(fraction, 1)
```

- This is the sealed executable reference example in `tests/reference.md`,
  omitting only its title comment.
- The input stays live; the fixed numeric zero/one bounds are not dependencies.
- The authoritative [saturation section](../nucleoid.spec.md#reactive-unit-interval-saturation)
  and its five executable entries define the behavior.
- [NUC 23](nuc-0023.md) explains the ordered unit-interval contract.

### Cubic smoothstep keeps all three inputs live

```
lower = 0
upper = 8
value = 2
unit = Math.saturate((value - lower) / (upper - lower))
eased = Math.smoothstep(lower, upper, value)
assert(unit, 0.25)
assert(eased, 0.15625)
value = 6
assert(eased, 0.84375)
upper = 16
assert(eased, 0.31640625)
```

- This is the sealed executable reference entry in `tests/reference.md`,
  omitting only its title comment.
- The `unit` normalization remains an independent asserted control.
- The authoritative [smoothstep section](../nucleoid.spec.md#reactive-cubic-smoothstep)
  and its five executable entries define this accepted behavior.
- Accepted [NUC 33](nuc-0033.md) records the lower/upper/value order, exact
  maximum-then-minimum normalization and separate ordered cubic operations.

### Quintic smootherstep keeps all three inputs live

```
lower = 0
upper = 8
value = 2
unit = Math.saturate((value - lower) / (upper - lower))
eased = Math.smootherstep(lower, upper, value)
assert(unit, 0.25)
assert(eased, 0.103515625)
value = 6
assert(eased, 0.896484375)
upper = 16
assert(eased, 0.27520751953125)
```

- This is the sealed executable reference entry in `tests/reference.md`,
  omitting only its title comment; `unit` remains an independent live control.
- Accepted [NUC 35](nuc-0035.md) adds only static
  `Math.smootherstep(lower, upper, value)` beside unchanged cubic smoothstep.
  The authoritative [quintic section](../nucleoid.spec.md#reactive-quintic-smootherstep)
  and its five entries specify the accepted contract.
- After the existing any-supplied-`Undefined` gate, convert lower, upper and
  value intrinsically with `Value::to_number`, in order; each omission is
  numeric NaN. Do not invoke or track shadowable `Number` or `Math.saturate`.
- Compute separate `numerator = value - lower`, `denominator = upper - lower`,
  `normalize = numerator / denominator`, then use the unchanged maximum-fold
  followed by minimum-fold unit clamp to obtain `t`.
- Compute separate `square = t * t`, `cube = square * t`, `scaled = 6.0 * t`,
  `offset = scaled - 15.0`, `weighted = t * offset`,
  `polynomial = weighted + 10.0`, then `cube * polynomial`, in that order.
- Do not expand/reassociate the polynomial, fuse operations, use `powf` or
  host clamp, validate/reorder bounds, shortcut endpoints or clamp the result.
  Rounded polynomial results can overshoot `[0, 1]`.
- Equal/reversed/infinite/overflow bounds, NaNs, subnormals and zero bits
  follow the exact accepted composition. Compare expected NaNs by
  classification and all other numeric bits, including zeros, against host
  controls; no portable NaN-payload or zero-sign promise is added.
- Literal null converts numerically to zero without overriding live-null
  assignment or parameter deferral. Other values keep existing conversion;
  nonnumeric conversions yield numeric NaN rather than new type errors.
- Every supplied expression runs once left to right, including extras.
  Extra values are unconverted but keep reads, effects, strict errors and the
  supplied-Undefined gate, including deferred property deletion/recovery.
- State/local Math shadowing, ordinary members, unsupported method reads,
  typed wrappers, call frames, receivers, class rules and transactional
  rollback remain unchanged. Immediate intrinsic dependencies differ from
  the emulation's unit node, while controls and transitive behavior agree.
- The mathematical quintic has smoother endpoint transitions than the cubic;
  no exact floating-point derivative claim is made. All sibling Math methods,
  accepted syntax, public APIs and inherited scope/resource limits remain.

---

## Reactive fractional part

### Floor fractional part relationships stay live

```
class Signal
signal = Signal()
signal.sample = 1.75
whole = Math.floor(signal.sample)
fraction = Math.fract(signal.sample)
assert(whole, 1)
assert(fraction, 0.75)
signal.sample = -1.5
assert(whole, -2)
assert(fraction, 0.5)
signal.sample = 4.125
assert(fraction, 0.125)
```

- This is the sealed `tests/reference.md` entry, omitting only its title
  comment; `whole = Math.floor(signal.sample)` stays independent.
- Accepted [NUC 36](nuc-0036.md) adds only unshadowed static
  `Math.fract(value)`. The authoritative
  [fractional-part section](../nucleoid.spec.md#reactive-fractional-part)
  and its five entries define the accepted contract.
- After Math's unchanged any-supplied-`Undefined` gate, convert the first value
  intrinsically with `Value::to_number`; omission supplies numeric NaN.
- Compute separate ordinary `floor = first.floor()` and `first - floor`, in
  that order. The accepted `value - Math.floor(value)` remains available.
  Do not use host `f64::fract`, truncation, remainder, sign correction,
  absolute value, conversion skips, normalization or result clamping.
- Negative fractions subtract floor: `-1.25` yields `0.75`. Tiny negative
  input `-0.00000000000000000001` can round to exactly one; the result has no
  strict `[0, 1)` guarantee. Infinities yield numeric NaN by subtraction.
- Expected NaNs use classification; every other result, including zero, matches
  independent accepted arithmetic/floor controls by bits, without a portable
  NaN-payload or zero-sign promise.
- Literal null converts to zero without overriding live assignment or
  parameter-null deferral. Numbers, booleans, strings, lists and dates retain
  existing conversion; nonnumeric values produce numeric NaN, not new errors.
- Every supplied expression runs once left to right, including ignored extras.
  Extra values are not converted, but retain effects, reads, dependencies,
  strict errors and the supplied-Undefined gate, deletion and recovery.
- Intrinsic conversion and host floor never invoke or track shadowable
  `Number` or `Math.floor`. State/local Math shadowing and ordinary `.Math`
  members keep lookup; reading `Math.fract` as a value remains unsupported/
  Undefined, not a first-class callable.
- Wrappers, frames, receivers, scopes, class rules, property deletion,
  graph transactions, rollback, error positions and recovery are unchanged.
  Unknown methods and `Math.random()` retain their exact inherited errors.
- No sibling Math arm/helper/control, accepted syntax, parser, lexer, AST,
  generator, CLI implementation, API/representation, dependency, version,
  MSRV or resource limit changes. Inherited scope limitations remain.

---

## Reactive degree to radian conversion

### Degree to radian conversion keeps angles live

```
degrees = 90
radians = Math.toRadians(degrees)
assert(radians, Math.PI / 2)
degrees = 180
assert(radians, Math.PI)
degrees = -90
assert(radians, -Math.PI / 2)
```

- This is the complete `tests/reference.md` entry without its title comment.
  The [NUC 50 candidate](nuc-0050.md) and the authoritative
  [root section](../nucleoid.spec.md#reactive-degree-to-radian-conversion)
  specify only the unshadowed static signature `Math.toRadians(value)`.
- After the existing any-supplied-Undefined gate, intrinsically convert the
  first input with `Value::to_number`; omission supplies numeric NaN.
  Compute the ordinary `f64` division `intrinsic PI / 180.0` before
  multiplying the first value by that factor. The accepted
  `value * (Math.PI / 180)` remains available.
- Finite maxima avoid intermediate overflow; signed zeros, subnormal rounding,
  infinities and every non-NaN result match independent accepted arithmetic
  by bits. NaNs compare by classification, without a payload guarantee.
  There is no range normalization, clamping or higher-precision calculation.
- Literal null converts to zero without overriding live assignment or
  parameter-null deferral. Numbers, booleans, strings, lists and dates keep
  existing conversion; nonnumeric values yield numeric NaN, not new errors.
- Every supplied expression evaluates once left to right per invocation.
  Ignored extras are not converted, but retain reads, effects, dependencies,
  errors and the supplied-Undefined gate, including deletion and recovery.
- The factor and conversion never resolve or track shadowable `Math.PI` or
  `Number`. State/local Math shadowing and ordinary `.Math` members retain
  lookup. Reading `Math.toRadians` as a value remains unsupported/Undefined,
  not a callable reference.
- Wrappers, frames, receivers, scopes, independent class rules, error
  positions, transactions and rollback use unchanged machinery. Strict
  unknown arguments retain `ReferenceError("<name> is not defined")`, even
  after Undefined; `Math.random()` retains
  `TypeError("Math.random is not supported")`. Numeric `Math = 7` retains
  `TypeError("number.toRadians is not a function")`.
- Add no aliases, `Math.toDegrees`, syntax, public types/APIs, dependencies
  or configurability. All sibling helpers and accepted syntax remain unchanged.
  Inherited scope limitations remain; new call ASTs and graph rule-source
  strings intentionally differ from their arithmetic counterparts.

---

## Reactive base two exponentiation

### Nucleoid names reactive base two powers

```
def scale power => Math.exp2(power)
power = 3
ratio = scale(power)
assert(ratio, 8)
power = -2
assert(ratio, 0.25)
power = 0
assert(ratio, 1)
ratio
```

- This is the complete new `tests/reference.md` program, without its title
  and expected-return metadata; it returns `1`. Accepted [NUC 56](nuc-0056.md)
  and the [root contract](../nucleoid.spec.md#reactive-base-two-exponentiation)
  specify only unshadowed static `Math.exp2(value)`.
- After Math's existing any-supplied-Undefined gate, intrinsically convert the
  first argument with `Value::to_number` and compute `2.0_f64.powf(first)`.
  Omission supplies numeric NaN. Ordinary `Math.pow(2, value)` stays supported.
- Match every non-NaN result by bits and NaNs by classification against the
  accepted pow operation, including fractional and extreme exponents,
  infinities, subnormals and positive-zero underflow. Both zero exponents
  yield one. Do not substitute native `f64::exp2`, exponent/log algebra,
  shortcuts or rescaled/fused math; no accuracy or speed claim is made.
- Literal null converts to zero without changing live assignment or
  parameter-null deferral. Numbers, booleans, strings, lists, dates and
  nonnumeric values retain their intrinsic conversions and NaN results.
- Every supplied expression evaluates once left to right. Ignored extras
  are not numerically converted, but retain dependencies, effects, strict
  errors and the supplied-Undefined gate, including deletion and recovery.
- Conversion and the fixed base do not consult shadowable Number or Math
  constants. State/local Math bindings and ordinary `.Math` members retain
  lookup; `Math.exp2` value reads remain unsupported/Undefined, not methods
  that can be stored and called indirectly.
- Independent class rules, functions, receivers, scopes, frames, graph
  subscriptions and transaction rollback are inherited unchanged. There are
  no new arity, numeric, type or range errors. Strict unknown arguments keep
  `ReferenceError("<name> is not defined")`, even after Undefined; numeric
  `Math = 7` keeps `TypeError("number.exp2 is not a function")`.
- Sibling helpers, unknown/random static errors and accepted syntax remain
  unchanged. No aliases, parser/lexer changes, public Rust APIs/types,
  dependencies, versions or resource limits are introduced.

---

## Reactive inverse interpolation

### Unclamped inverse interpolation relationships stay live

```
start = 2
end = 10
sample = 4
weight = Math.inverseLerp(start, end, sample)
assert(weight, 0.25)
sample = 6
assert(weight, 0.5)
start = 0
assert(weight, 0.6)
end = 12
assert(weight, 0.5)
sample = 18
assert(weight, 1.5)
```

- This is the sealed `tests/reference.md` entry, omitting only its title comment.
- Accepted [NUC 38](nuc-0038.md) adds only unshadowed static
  `Math.inverseLerp(start, end, value)`. The authoritative
  [inverse-interpolation section](../nucleoid.spec.md#reactive-inverse-interpolation)
  and its five entries specify the accepted contract.
- After Math's unchanged any-supplied-Undefined gate, including extras,
  intrinsically convert start, end and value with `Value::to_number`.
  The dispatcher already converts start; each omitted input is numeric NaN.
- Compute separate ordinary `numerator = value - start`,
  `denominator = end - start`, then `numerator / denominator`, in that order.
  The accepted `(value - start) / (end - start)` remains available.
- Do not clamp, smooth, validate or reorder ranges, factor, fuse, scale to
  avoid overflow or shortcut endpoints or zero width. Descending ranges and
  results below zero or above one are intentional.
- Equal finite endpoints give ordinary NaN or signed infinity. Nonfinite
  inputs, span overflow, cancellation, underflow, subnormals and signed zeros
  retain exactly that composition. The overflow span from `-Number.MAX_VALUE`
  to `Number.MAX_VALUE` with value zero yields zero, not `0.5`.
- Expected NaNs compare by classification; every non-NaN result, including
  zero, matches independent accepted binary subtraction/division controls
  by bits. No portable NaN-payload or zero-sign promise is added.
- Literal null converts to zero without overriding live assignment or
  parameter-null deferral. All Value kinds retain their existing numeric
  conversion; nonnumeric values yield numeric NaN, not new errors.
- Every supplied expression runs once left to right, including expressions
  after Undefined. Ignored extras are neither converted nor used, but retain
  reads, effects, dependencies, strict failures, deferral and rollback.
- Intrinsic conversion never resolves, invokes or tracks shadowable Number
  or Math methods. State/local Math shadowing and ordinary `.Math` members
  keep lookup; reading `Math.inverseLerp` as a value stays unsupported/
  Undefined, not a callable reference.
- Wrappers, frames, receivers, aliases, scopes, per-instance class rules,
  property deletion/recovery and graph transactions remain unchanged.
  Strict unknown arguments keep `ReferenceError("<name> is not defined")`;
  unknown Math methods and `Math.random()` retain their exact inherited errors.
- No sibling Math arm/helper/control, accepted syntax, parser, lexer, AST,
  generator, CLI implementation, API/representation, namespace, dependency,
  version, MSRV, edition or resource bound changes. Inherited limitations remain.
- New intrinsic RHS ASTs and immediate dependency graphs may differ from
  emulation; non-RHS source/ASTs, assertions, independent controls, values,
  transitive reads/effects/errors, returns and executed coverage are protected.

---

## Reactive range remapping

### Unclamped remapping relationships stay live

```
start = 2
end = 10
targetStart = 10
targetEnd = 30
sample = 4
mapped = Math.remap(start, end, targetStart, targetEnd, sample)
assert(mapped, 15)
sample = 6
assert(mapped, 20)
targetStart = 0
assert(mapped, 15)
targetEnd = 40
assert(mapped, 20)
sample = 18
assert(mapped, 80)
```

- This is the sealed `tests/reference.md` entry, omitting only its title comment.
- Accepted [NUC 39](nuc-0039.md) adds only unshadowed static
  `Math.remap(start, end, targetStart, targetEnd, value)` as an accepted intrinsic.
- The authoritative
  [range-remapping section](../nucleoid.spec.md#reactive-range-remapping)
  and its five executable entries specify the accepted contract.
- Retain Math's any-supplied-Undefined gate, including extras, and its existing
  first-input conversion for start. Convert end, targetStart, targetEnd and
  value intrinsically in that order; each omission supplies numeric NaN.
- Compute six separate ordinary operations: `numerator = value - start`,
  `denominator = end - start`, `weight = numerator / denominator`,
  `delta = targetEnd - targetStart`, `scaled = delta * weight`,
  `result = targetStart + scaled`, in that order.
- The accepted nested spelling
  `Math.lerp(targetStart, targetEnd, Math.inverseLerp(start, end, value))`
  remains available. It evaluates target arguments first; the new call uses
  its listed five-argument order. The sealed counterparts use pure reads.
- Do not clamp, smooth, validate, reorder, factor, fuse, raise precision,
  rescale or shortcut endpoints, equal targets or zero source width.
  Descending ranges and extrapolation are intentional.
- NaNs, infinities, overflow, underflow, cancellation, subnormals and signed
  zeros retain that exact numeric composition. An overflowing source span
  can normalize to zero, not a midpoint. Equal targets with infinite weight
  give NaN; overflowing target span at weight `0.25` can give positive infinity.
- NaNs compare by classification; every other result, including zero, matches
  independent accepted arithmetic by bits, with no portable zero-sign promise.
- Intrinsic literal null converts to zero without overriding live assignment
  or parameter-null deferral. All Value kinds keep existing conversion;
  nonnumeric values yield numeric NaN, not new errors.
- Every supplied expression runs once left to right, including expressions
  after Undefined and extras. Extra values are not converted or used, but
  retain effects, reads, dependencies, strict errors, deletion and recovery.
- Intrinsic conversion never resolves, invokes or tracks shadowable Number
  or Math methods. State/local Math shadowing and ordinary `.Math` members
  keep lookup. Reading `Math.remap` remains unsupported/Undefined, not callable.
- Wrappers, frames, receivers, aliases, scopes, per-instance class rules,
  property deletion/recovery, error positions, transactions and rollback remain.
- Strict missing arguments retain `ReferenceError("missing is not defined")`.
  Numeric `Math = 7` retains `TypeError("number.remap is not a function")`.
  Unknown Math methods and `Math.random()` retain their inherited errors.
- Literal `Math.remap(` at EOF gives `SyntaxError("Unexpected end of input")`
  at line 1, column 12; a final newline moves that EOF to line 2, column 1.
- Existing Math arms/helpers/constants/conversions, accepted syntax, parser,
  lexer, AST, generator, CLI implementation, public APIs, dependencies,
  versions, MSRV, edition and resource bounds remain unchanged.
- Protect historical source/full ASTs and new non-RHS source/typed metadata,
  independent inverseLerp controls, ordered assertions, expected returns and
  actual coverage, not intentional new-RHS/immediate-graph differences.

---

### Reactive flat mapping

- Accepted [NUC 41](nuc-0041.md) specifies `list.flatMap(callback)` and
  `ClassName.flatMap(callback)` as the same shared read-only list query.
- Snapshot the receiver before arguments, evaluate every supplied expression
  left to right (including unused extras), then validate the first value as
  a function. A missing or non-function callback raises exactly
  `TypeError("flatMap expects a function")`, including on empty inputs.
- Invoke once per snapshotted item, in order, with only that item as the
  argument, as for `map`. Missing callback parameters remain `null`;
  no index, receiver argument, closure capture or automatic `this` is added.
- Flatten a returned list exactly one level, preserving deeper lists,
  duplicates and object/function identities; other values contribute one item,
  and an empty returned list contributes none. The query itself never writes
  its receiver, although callbacks retain their normal effects.
- Returning null or undefined does not manufacture a read. Actual null reads
  still affect enclosing assignment settlement; missing/deleted reads retain
  their existing behavior. A null/undefined receiver keeps ordinary method
  routing, including skipping argument evaluation.
- Callback/member reads, population changes, redefinitions, `.value`
  snapshots, obsolete dependency removal, errors, rollback and recovery use
  the existing machinery. A frozen function keeps its definition, not a
  frozen copy of state read from its body.
- The executable reference entry `Nucleoid flat maps live related collections`
  tracks team-tag updates. The root
  [specification](../nucleoid.spec.md#reactive-flat-mapping) also covers ordered
  projection, list mutation, population growth, snapshots and failed redefinition.
- `map`, `reduce`, `concat` and custom object/instance/global bindings named
  `flatMap` remain available unchanged. Native methods do not gain first-class
  bound-property lookup, recursion, a depth option or a feature-specific cap.
- The accepted map/reduce/concat composition can read a null intermediate
  through its reducer; native flat mapping adds no artificial read to emulate
  that effect. Intentional runtime-call RHS rewrites change AST and canonical
  graph source; full-AST/graph-source equality is not promised.

### In-place list sorting

```
scores = [30, 10, 20]
ranked = scores.slice().sort((a, b) => a - b)
best = scores.sort((a, b) => b - a)[0]
assert(scores, [30, 20, 10])
assert(ranked, [10, 20, 30])
assert(best, 30)
scores.push(40)
assert(scores, [40, 30, 20, 10])
assert(ranked, [10, 20, 30, 40])
assert(best, 40)
assert([3, 1, 2].sort(), [1, 2, 3])
scores
```

- This is the complete `Nucleoid sorts lists in place` program in
  `tests/reference.md`, without its title and expected-return metadata; it
  returns `[40,30,20,10]`.
- [NUC 58](nuc-0058.md) makes `sort` reorder a writable receiver in place and
  return the sorted list, as JavaScript's `Array.prototype.sort` does. `sort`
  used to return a sorted copy and leave every receiver unchanged; the change
  for writable receivers is deliberate.
- Ordering is unchanged. A comparator places each item, in input order, before
  the first placed item it compares below zero, so equal items keep their
  order. Without a function argument, items order stably by their string
  forms: `[10, 9, 1]` sorts to `[1, 10, 9]`.
- Writable receivers are variables, parameter and loop bindings, and real
  object properties, including members reached through `this`, `self`,
  `$Class`, a call result or a version-selected object. The write appends a
  version when the order changes and re-runs dependents, as `push`, `pop` and
  `shift` do; the receiver is read once and never evaluated again to write.
- List literals, call results such as `scores.slice()`, indexes such as
  `grid[0]`, `.value` snapshots, selected versions such as `scores@first`, and
  class populations such as `Item.sort(comparator)` keep returning a sorted
  copy and write nothing. Sort `scores.slice()` to keep the source order.
- Lists are values, so sorting a parameter or loop binding leaves the list it
  was copied from unchanged.
- A `sort` in a declaration is a standing sort: `best = scores.sort(comparator)[0]`
  keeps `scores` sorted whenever it, the comparator or what the comparator
  reads changes. Two standing sorts of one list with different orders re-run
  each other and raise `TypeError("Propagation did not settle")`.
- The authoritative contract is the
  [in-place list sorting](../nucleoid.spec.md#in-place-list-sorting) section.

### Instance list ranges and list queries

```
class Task(title, done):
    this.title = title
    this.done = done
plan = Task("plan", true)
build = Task("build", false)
recent = Task[-1:]
everyone = Task.list()
pending = Task.findIndex(task => not task.done)
newest = Task.at(-1).title
assert(recent, [build])
assert(everyone, Task.all())
assert(pending, 1)
ship = Task("ship", false)
assert(recent, [ship])
assert(everyone, [plan, build, ship])
assert(newest, "ship")
build.done = true
assert(pending, 2)
assert(Task.lastIndexOf(build), 1)
everyone.map(task => task.title)
```

- This is the complete
  `Nucleoid reads instance lists with ranges and list queries` program in
  `tests/reference.md`, without its title and expected-return metadata; it
  returns `["plan","build","ship"]`.
- [NUC 61](nuc-0061.md) lets a class's instance list take list ranges:
  `Task[start:end]` returns its instances in population order with the bound
  rules of list slices, equals `Task.slice(start, end)`, and depends on the
  population and its bounds. `Task.list()` and `Task.all()` return every
  instance; plain lists have neither method.
- Lists and instance lists gain `at`, `lastIndexOf`, `findIndex`, `findLast`
  and `findLastIndex`, beside the existing `includes`, `indexOf`, `concat` and
  `slice`. All are read-only and live: a declaration follows the list or
  population, its arguments and what its callback reads.
- `at(index)` truncates the index and counts a negative one from the end, as
  in JavaScript. Outside the receiver it returns what bracket indexing returns
  there: undefined for a list, stored as `null`, and `null` for an instance
  list. `findLast` and `findLastIndex` search from the last item back, and the
  searches return `null` or `-1` when nothing matches, as `find` returns
  `null`.
- `includes`, `indexOf` and `lastIndexOf` use the deep equality of `assert`,
  so lists and objects match on their contents and distinct instances never
  match each other; a JavaScript `fromIndex` is ignored. `concat` spreads list
  arguments one level and appends any other value, including an instance list
  named by its class, as one item.
- The authoritative contract is the
  [instance list ranges and list queries](../nucleoid.spec.md#instance-list-ranges-and-list-queries)
  section.

### List and population sums

```
scores = [3, 4, 5]
total = scores.sum()
squares = scores.sum(score -> score * score)
class Order(amount): this.amount = amount
first = Order(20)
revenue = Order.sum(order -> order.amount)
assert(total, 12)
assert(squares, 50)
assert(revenue, 20)
scores.push(8)
second = Order(15)
assert(total, 20)
assert(squares, 114)
assert(revenue, 35)
scores = []
assert(total, 0)
revenue
```

- This is the complete
  `Nucleoid totals lists and populations with sum` program in
  `tests/reference.md`, without its title and expected-return metadata; it
  returns `35`.
- [NUC 68](nuc-0068.md) adds `sum()` and `sum(selector)` to lists and class
  instance lists. The values are converted to numbers and added left to right
  from `0`, so a list of numbers totals exactly what
  `reduce((total, item) => total + item, 0)` returns, and an empty list or
  class totals `0`.
- Values convert as for `-`, `*` and the Math helpers: `true` counts `1`;
  `null`, `""` and `[]` count `0`; a numeric string counts its number; objects
  and other strings are NaN. An undefined value makes the total undefined,
  stored as `null`. A `null` item counts `0` without settling a declaration to
  `null`, while the reads inside a selector follow the existing rules.
- The selector is called once per item, in order, with only the item, as
  `map` calls its callback. A supplied selector that is not a function,
  including `null`, raises `TypeError("sum expects a function")`, even on an
  empty receiver.
- Sums are read-only and live: a declaration follows the list or population,
  the arguments and what the selector reads, in class-level rules too. On a
  class name, `Name.sum(...)` is the population query even when the class
  declares an instance method `sum`; instances keep calling the method.
- The authoritative contract is the
  [list and population sums](../nucleoid.spec.md#list-and-population-sums)
  section.

### List and population extremes

```
scores = [9, 3, 7]
best = scores.max()
worst = scores.min()
class Bid(amount): this.amount = amount
Bid(40)
top = Bid.max(bid -> bid.amount)
assert(best, 9)
assert(worst, 3)
assert(top, 40)
scores.push(12)
scores.shift()
Bid(65)
Bid(50)
assert(best, 12)
assert(worst, 3)
assert(top, 65)
scores = []
assert(best, Number.NEGATIVE_INFINITY)
top
```

- This is the complete
  `Nucleoid finds extremes of lists and populations with max and min`
  program in `tests/reference.md`, without its title and expected-return
  metadata; it returns `65`.
- [NUC 69](nuc-0069.md) adds `max()`, `min()`, `max(selector)` and
  `min(selector)` to lists and class instance lists. The result is
  `Math.max` or `Math.min` applied to the values in receiver order, so an
  empty list or class gives `-Infinity` for `max` and `Infinity` for `min`.
- The result is always a number: values convert as for `-`, `*` and the Math
  helpers, so `["10", "9"].max()` is `10`, `true` counts `1`, `null` counts
  `0`, and objects and other strings make the result NaN. An undefined value
  makes the result undefined, stored as `null`. A `null` item counts `0`
  without settling a declaration to `null`, while the reads inside a selector
  follow the existing rules.
- A tie between `0` and `-0` gives the same result as `Math.max` or
  `Math.min` on the same values in the same order; the language does not
  otherwise fix the sign of a zero tie. The current x86-64 reference build
  keeps the later zero, so there `[0, -0].max()` is `-0`, where JavaScript's
  `Math.max(0, -0)` is `0`.
- The selector is called once per item, in order, with only the item, as
  `map` calls its callback. A supplied selector that is not a function,
  including `null`, raises `TypeError("max expects a function")` or
  `TypeError("min expects a function")`, even on an empty receiver.
- Extremes are read-only and live: a declaration follows the list or
  population, the arguments and what the selector reads, in class-level
  rules too. On a class name, `Name.max(...)` and `Name.min(...)` are
  population queries even when the class declares instance methods `max` or
  `min`; instances keep calling the methods.
- The authoritative contract is the
  [list and population extremes](../nucleoid.spec.md#list-and-population-extremes)
  section.

### List and population distinct values

```
tags = ["red", "blue", "red"]
colors = tags.distinct()
lengths = tags.distinct(tag -> tag.length)
class Tile(color): this.color = color
Tile("red")
Tile("green")
Tile("red")
palette = Tile.distinct(tile -> tile.color)
assert(colors, ["red", "blue"])
assert(lengths, [3, 4])
assert(palette, ["red", "green"])
tags.push("green")
tags.shift()
Tile("blue")
assert(colors, ["blue", "red", "green"])
assert(lengths, [4, 3, 5])
assert(palette, ["red", "green", "blue"])
tags = []
assert(colors, [])
palette.length
```

- This is the complete
  `Nucleoid lists the distinct values of lists and populations` program in
  `tests/reference.md`, without its title and expected-return metadata; it
  returns `3`.
- [NUC 71](nuc-0071.md) adds `distinct()` and `distinct(selector)` to lists
  and class instance lists. The result is a new list of the items, or of the
  selected values, without repeats, keeping each first occurrence in receiver
  order, and an empty list or class gives `[]`. `xs.distinct(selector)` equals
  `xs.map(selector).distinct()`.
- Repeats are decided by the deep equality of `assert`, `includes` and
  `indexOf`, and values are never converted: `1`, `"1"` and `true` stay three
  values, `null` and undefined are one, `0` and `-0` are one with the first
  kept, every NaN is kept, records match on their contents and instances only
  themselves. A `null` item is kept without settling a declaration to `null`,
  while the reads inside a selector follow the existing rules.
- The selector is called once per item, in order, with only the item, as
  `map` calls its callback. A supplied selector that is not a function,
  including `null`, raises `TypeError("distinct expects a function")`, even on
  an empty receiver.
- Distinct values are read-only and live: a declaration follows the list or
  population, the arguments and what the selector reads, in class-level rules
  too, but not the properties of the objects it compares. On a class name,
  `Name.distinct(...)` is the population query even when the class declares
  an instance method `distinct`; instances keep calling the method.
- The authoritative contract is the
  [list and population distinct values](../nucleoid.spec.md#list-and-population-distinct-values)
  section.

### List and population predicate counts

```
numbers = [1, 4, 7, 10]
limit = 5
large = numbers.count(number -> number > limit)
assert: large, 2
numbers.push(12)
assert: large, 3
class Job(done): this.done = done
pending = Job.count(job -> not job.done)
first = Job(false)
second = Job(true)
assert: pending, 1
second.done = false
assert: pending, 2
delete first.done
delete first
assert: pending, 1
limit = 9
assert: large, 2
```

- This is the complete `Nucleoid counts matching items and instances`
  program in `tests/reference.md`, without its title.
- [NUC 73](nuc-0073.md) adds `count(predicate)` to every list or class
  population receiver supported by `filter`, including class aliases. It
  returns the Number from `filter(predicate).length` without allocating the
  filtered result list; a valid empty receiver gives `0`.
- The predicate is called once per item, in receiver order, with only the
  item and the truthiness rules of `filter`. It visits every item without
  short-circuiting. A missing or non-function predicate, including `null`,
  raises `TypeError("count expects a function")`, even on an empty receiver;
  there is no no-argument overload.
- The receiver snapshot precedes argument evaluation. All supplied arguments,
  including ignored extras, run once, left to right. Counts inherit `filter`'s
  dependencies, nullish reads, deferred declarations, `.value`, state versions,
  callback errors and rollback; a null or undefined receiver skips arguments
  and yields undefined.
- Counts stay live in declarations and class-level rules. Bindings, properties
  and instance methods named `count` keep their behavior, while
  `Name.count(predicate)` on a class or class alias queries its population.
  All accepted `filter(predicate).length` forms remain unchanged.
- The authoritative contract is the
  [list and population predicate counts](../nucleoid.spec.md#list-and-population-predicate-counts)
  section.

### List and population modes

```
sizes = ["M", "XL", "M", "XL"]
top = sizes.mode()
width = sizes.mode(size -> size.length)
class Vote(choice): this.choice = choice
Vote("tea")
Vote("coffee")
Vote("coffee")
favorite = Vote.mode(vote -> vote.choice)
assert(top, "M")
assert(width, 1)
assert(favorite, "coffee")
sizes.shift()
Vote("tea")
Vote("tea")
assert(top, "XL")
assert(width, 2)
assert(favorite, "tea")
sizes = []
assert(top, null)
favorite
```

- This is the complete
  `Nucleoid finds the most frequent items of lists and populations` program
  in `tests/reference.md`, without its title and expected-return metadata;
  it returns `"tea"`.
- [NUC 75](nuc-0075.md) adds `mode()` and `mode(selector)` to lists and
  class instance lists. The result is the most frequent item, or selected
  value, itself; on a tie, the one that comes first in receiver order wins,
  so `[3, 4, 4, 3].mode()` is `3`. An empty list or class gives undefined,
  which a declaration stores as `null`. `xs.mode(selector)` equals
  `xs.map(selector).mode()`.
- Items are counted against the values `distinct` keeps, with the deep
  equality of `assert`, `includes` and `indexOf`, and never converted:
  `1`, `"1"` and `true` stay three values, `null` and undefined are one,
  `0` and `-0` are one with the first returned, every NaN, function and
  regular expression counts once on its own, records match on their contents
  and instances only themselves. The mode of a nonempty receiver is always
  one of the values that `distinct` keeps, and an item that matches several
  of them counts only for the earliest.
- The selector is called once per item, in order, with only the item, as
  `map` calls its callback. A supplied selector that is not a function,
  including `null`, raises `TypeError("mode expects a function")`, even on an
  empty receiver.
- Modes are read-only and live: a declaration follows the list or
  population, the arguments and what the selector reads, in class-level rules
  too, but not the properties of the objects it compares. On a class name,
  `Name.mode(...)` is the population query even when the class declares an
  instance method `mode`; instances keep calling the method.
- The authoritative contract is the
  [list and population modes](../nucleoid.spec.md#list-and-population-modes)
  section.

### Noun list queries with of

```
sizes = ["M", "XL", "M", "XL"]
top = mode of sizes
class Order(amount): this.amount = amount
Order(20)
Order(45)
revenue = sum of Order.map(order -> order.amount)
largest = max of Order.map(order -> order.amount)
assert(top, "M")
assert(revenue, 65)
assert(largest, 45)
sizes.shift()
Order(80)
assert(top, "XL")
assert(revenue, 145)
assert(largest, 80)
assert(max of sizes.map(size -> size.length), sizes.max(size -> size.length))
largest
```

- This is the complete
  `Nucleoid reads list and population queries named by nouns with of`
  program in `tests/reference.md`, without its title and expected-return
  metadata; it returns `80`.
- Revision 2 of [NUC 75](nuc-0075.md) adds `NOUN of OPERAND` for the four
  list queries named by nouns: `sum`, `max`, `min` and `mode`. Queries named
  by verbs keep the method call, as `xs.sort()` and `xs.push(x)` do.
- `NOUN of OPERAND` means exactly `OPERAND.NOUN()`: the same receivers,
  results, errors, evaluation order and null-receiver route, and the same
  dependencies and liveness in declarations and class-level rules, such as
  `$Obj.fill_color = mode of $Obj.parts`. The noun never reads a binding:
  after `def sum(values): return 0`, `sum of [1, 2]` is still `3`.
- It is a prefix operator like `-` and `typeof`: its operand takes the whole
  postfix chain, `sum of xs.map(f)`, and binary operators end it,
  `sum of xs + 1` is `(sum of xs) + 1`. It applies to a whole power, so
  `sum of xs ** 2` is `sum of (xs ** 2)`; write `(sum of xs) ** 2` to square a
  total.
- A selector stays a method argument, `xs.max(selector)`, which equals
  `max of xs.map(selector)` for every function selector.
- `for mode of rows:` still names its loop variable `mode`. At the top level
  of a template interpolation, `${sum of xs}` still reads the name `sum`;
  write `${(sum of xs)}` for the query.
- The query renders as written: `why` explains `top = mode of sizes` as
  `top = mode of sizes`, and `(mode of words).length` keeps its parentheses.
- The authoritative contract is the
  [noun list queries with of](../nucleoid.spec.md#noun-list-queries-with-of)
  section.

### List and population extreme items

```
scores = [{name: "ana", points: 9}, {name: "ben", points: 3}, {name: "cy", points: 9}, {name: "dan", points: 3}]
best = scores.maxBy(score -> score.points)
worst = scores.minBy(score -> score.points)
class Bid(who, amount):
    this.who = who
    this.amount = amount
Bid("ada", 40)
top = Bid.maxBy(bid -> bid.amount)
assert(best.name, "ana")
assert(worst.name, "ben")
assert(top.who, "ada")
scores.push({name: "eve", points: 1})
scores.shift()
Bid("bo", 65)
Bid("cy", 50)
assert(best.name, "cy")
assert(worst.name, "eve")
assert(top.who, "bo")
scores = []
assert(best, null)
top.who
```

- This is the complete
  `Nucleoid finds the extreme items of lists and populations with maxBy and minBy`
  program in `tests/reference.md`, without its title and expected-return
  metadata; it returns `"bo"`.
- [NUC 77](nuc-0077.md) adds `maxBy(selector)` and `minBy(selector)` to lists
  and class instance lists. The result is the first item whose selected key
  is the largest, or the smallest, and it is the item itself. On a tie the
  item that comes first wins, so `best` is `ana` beside `cy` and `worst` is
  `ben` beside `dan`. An empty list or class gives undefined, which a
  declaration stores as `null`.
- Keys compare as `>` and `<` compare them: two strings as text and every
  other pair as numbers, so dates compare by time. A key that cannot be
  compared with itself, such as undefined, NaN or an object, makes the result
  undefined. For keys that are not strings, the key of `xs.maxBy(f)` converts
  to `xs.max(f)` and the key of `xs.minBy(f)` to `xs.min(f)`.
- The selector is required and is called once per item, in order, with only
  the item. A missing selector, or one that is not a function, including
  `null`, raises `TypeError("maxBy expects a function")` or
  `TypeError("minBy expects a function")`, even on an empty receiver.
- Extreme items are read-only and live: a declaration follows the list or
  population, the arguments and what the selector reads, in class-level rules
  too. On a class name, `Name.maxBy(...)` is the population query even when
  the class declares an instance method `maxBy`; instances keep calling the
  method.
- The authoritative contract is the
  [list and population extreme items](../nucleoid.spec.md#list-and-population-extreme-items)
  section.

### List and population averages

```
scores = [70, 85, 90]
mean = scores.average()
passing = scores.filter(score -> score >= 80).average()
class Student(name, points, total):
    this.name = name
    this.points = points
    this.total = total
grade = Student.average(student -> student.points / student.total * 100)
Student("ana", 45, 50)
bo = Student("bo", 30, 50)
assert(mean, 81.66666666666667)
assert(passing, 87.5)
assert(grade, 75)
scores.push(95)
bo.points = 40
Student("cy", 20, 40)
assert(mean, 85)
assert(passing, 90)
assert(grade, 73.33333333333333)
scores = []
assert(Number.isNaN(mean), true)
grade
```

- This is the complete
  `Nucleoid averages lists and populations with average` program in
  `tests/reference.md`, without its title and expected-return metadata; it
  returns `73.33333333333333`.
- [NUC 80](nuc-0080.md) adds `average()` and `average(selector)` to lists and
  class instance lists. The result is exactly `sum(...)` divided by `length`
  with the same arguments, using the division of `/`, so a list of numbers
  averages bit for bit to `xs.sum() / xs.length`, and an empty list or class
  averages to NaN, the value of `0 / 0`.
- Values convert as `sum` converts them, and every item counts toward the
  divisor: `[2, null].average()` is `1`. An undefined value makes the mean
  undefined, stored as `null`. A `null` item counts `0` without settling a
  declaration to `null`, while the reads inside a selector follow the existing
  rules.
- The selector is called once per item, in order, with only the item. A
  supplied selector that is not a function, including `null`, raises
  `TypeError("average expects a function")`, even on an empty receiver.
- Averages are read-only and live: a declaration follows the list or
  population, the arguments and what the selector reads, in class-level rules
  too. On a class name, `Name.average(...)` is the population query even when
  the class declares an instance method `average`; instances keep calling the
  method. `average` is not a noun query, so `average of xs` stays a syntax
  error.
- The authoritative contract is the
  [list and population averages](../nucleoid.spec.md#list-and-population-averages)
  section.

### List and population medians

```
times = [42, 120]
typical = times.median()
class Runner(name, laps):
    this.name = name
    this.laps = laps
Runner("ana", [61, 58, 102])
bo = Runner("bo", [66, 64])
pace = Runner.median(runner -> runner.laps.median())
assert(typical, 81)
assert(pace, 63)
times.push(35)
bo.laps.push(59)
Runner("cy", [110])
assert(typical, 42)
assert(pace, 64)
pace
```

- This is the complete
  `Nucleoid finds the medians of lists and populations with median` program
  in `tests/reference.md`, without its title and expected-return metadata;
  it returns `64`.
- [NUC 84](nuc-0084.md) adds `median()` and `median(selector)` to lists and
  class instance lists. The values are ordered as numbers by a stable sort of
  a private copy: an odd count gives the middle value itself and an even count
  `(lower + upper) / 2`, so `[4, 1, 3, 2].median()` is `2.5`. The receiver is
  never sorted or written.
- Values convert as `sum` converts them. An empty list or class and any value
  that converts to NaN give NaN. An undefined value makes the median
  undefined, stored as `null`, and wins over a NaN value, as in `average`. A
  `null` item counts `0` without settling a declaration to `null`, while the
  reads inside a selector follow the existing rules.
- `0` and `-0` keep their receiver order, an odd count keeps the middle
  value's sign, and `[-0, -0].median()` is `-0` where `average()` gives `0`.
  `[1e308, 1e308].median()` overflows to `Infinity`, and `-Infinity` beside
  `Infinity` gives NaN.
- The selector is called once per item, in order, with only the item. A
  supplied selector that is not a function, including `null`, raises
  `TypeError("median expects a function")`, even on an empty receiver.
- Medians are read-only and live: a declaration follows the list or
  population, the arguments and what the selector reads, in class-level rules
  too. On a class name, `Name.median(...)` is the population query even when
  the class declares an instance method `median`; instances keep calling the
  method. `median` is not a noun query, so `median of xs` stays a syntax
  error.
- The authoritative contract is the
  [list and population medians](../nucleoid.spec.md#list-and-population-medians)
  section.

---

## 12. Reasoning Operations

- `why reference` selects the reference and its transitive dependencies.
- `affects reference` selects its transitive dependents, not the reference itself.
- `reference |> why` is equivalent to `why reference`; pipelines compose left to right.
- `model |> why` selects the current model, excluding pending nodes and reasoning observers.
- A selection is an ordered list of steps with `node`, `holds`, `rule`, `state`, and `from` fields.
- `state` is `"stated"` for an independent fact or `"derived"` for a relationship or class-level rule.
- List functions such as `map`, `find`, and `some` can inspect the steps.

```
a = 1
b = a + 2
c = b * 2
trace = why c
```

- `trace[0]` names `c`, holds `6`, and derives from `["b"]`.

### State-filtered selections (NUC 48 candidate)

- `selection |> stated` and `selection |> derived` retain steps with the
  corresponding existing `StepState`, in their original order, without
  changing `node`, `holds`, `rule`, `state`, or `from`.
- `result |> why |> stated` separates supporting facts from the derived
  conclusions selected by `result |> why |> derived`. Class-governed steps
  with empty `from` remain derived; an empty input-edge list is not a new
  provenance test.
- Prefix `stated seed` and `derived result` filter one reference only;
  neither implicitly runs `why`. `model |> stated` and `model |> derived`
  filter the model directly. Nested pipelines remain graph selections until
  the existing final materialization.
- Keep every source-stage value and structural observation, including
  excluded steps. Empty results, equal-valued state/declaration rewrites,
  ancestry replacement, deletion, functions, class rules, observer exclusion,
  `.value` snapshots and transaction rollback retain their live behavior.
- These are ordinary contextual identifiers, not lexer keywords.
  Bindings, parameters, keys, properties, functions and methods named
  `stated` or `derived` are unchanged; `stated(seed)` remains an ordinary call.
  The accepted list `.filter` spelling is still available.
- A local/parameter, state variable, function or class binding shadows either
  new stage name: a pipeline then retains
  `TypeError("stated is not an operation")` or
  `TypeError("derived is not an operation")` before selecting its source.
  Comma-free object fields still accept ordinary values with those names.
  Existing `why`/`affects` binding behavior is unchanged.
- Invalid sources retain `TypeError("Cannot explain 1")` for `1 |> stated`
  and `ReferenceError("missing is not defined")` for `stated missing`.
- Normative detail: [state-filtered reasoning selections](../nucleoid.spec.md#state-filtered-reasoning-selections)
  and the seven appended entries. The executable reference counterpart is
  `Reasoning state filters distinguish facts from conclusions`.

```
seed = 2
middle = seed + 3
result = middle * 2
facts = result |> why |> stated
conclusions = result |> why |> derived
assert(facts.map(step => step.node), ["seed"])
assert(facts[0].holds, 2)
assert(facts[0].from, [])
assert(conclusions.map(step => step.node), ["result", "middle"])
assert(conclusions[0].holds, 10)
assert(conclusions[0].from, ["middle"])
assert(conclusions[0].state, "derived")
```

- The equivalent accepted computations remain
  `facts = (why result).filter(step => step.state == "stated")` and
  `conclusions = (why result).filter(step => step.state == "derived")`.

### Depth-limited selections

```
price = 10
qty = 3
shipping = 5
subtotal = price * qty
total = subtotal + shipping
direct = total |> why |> depth(1)
assert(direct.map(step => step.node), ["total", "subtotal", "shipping"])
assert((why total).length, 5)
price = 12
direct.map(step => step.holds)
```

- This is the complete `Nucleoid limits explanations to direct causes with depth`
  program in `tests/reference.md`, without its title and expected-return
  metadata; it returns `[41,36,5]`.
- [NUC 82](nuc-0082.md) adds the stage `selection |> depth(limit)`. It keeps
  the steps whose distance from where the walk that selected them began is at
  most `limit`, in their order and with their fields unchanged, so
  `total |> why |> depth(1)` is `total` and what it reads directly, while
  `why total` selects every fact behind it.
- A step's distance is the length of the shortest path by which the most
  recent `why` or `affects` reached it from its input, whatever order the walk
  lists the steps in. `why` keeps its inputs at distance 0; every `affects`
  step is at distance 1 or more, so `price |> affects |> depth(1)` is what
  reads the price directly and `depth(0)` after `affects` is empty. A
  reference's own step and `model` are at distance 0, and `stated`, `derived`
  and `depth` keep distances, so they compose in either order.
- The limit is an expression evaluated in place, so `depth(n)` follows `n`. It
  must be a non-negative integer; any other value raises
  `TypeError("<limit> is not a non-negative integer")`. A `null` or undefined
  limit makes the selection undefined, which a declaration stores as `null`
  until the limit is known.
- Only `|>` followed by `depth` and `(` is the stage; `depth` stays an
  ordinary name everywhere else, and a bare `|> depth` keeps its parse-time
  `TypeError("depth is not an operation")`. A local or parameter, variable,
  function or class named `depth` shadows the stage, as bindings shadow the
  state filters: the stage then raises `TypeError("depth is not an operation")`
  before selecting its source.
- The selection stays live through every observation its source made,
  including the steps it drops, and through what its limit reads. It renders
  as `selection|>depth(limit)`.
- The authoritative contract is the
  [depth-limited reasoning selections](../nucleoid.spec.md#depth-limited-reasoning-selections)
  section.

### Derivation path selections

```
price = 10
qty = 3
shipping = 5
subtotal = price * qty
total = subtotal + shipping
link = total |> why |> path(price)
assert(link.map(step => step.node), ["total", "subtotal", "price"])
assert((why total).length, 5)
price = 12
link.map(step => step.holds)
```

- This is the complete `Nucleoid explains how one value reaches another with path`
  program in `tests/reference.md`, without its title and expected-return
  metadata; it returns `[41,36,12]`.
- [NUC 88](nuc-0088.md) adds the stage `selection |> path(target)`. It keeps
  the steps that lie on a dependency path between the walk that selected them
  and `target`, in their order and with their fields unchanged, so
  `total |> why |> path(price)` is how the price reaches the total, while
  `why total` also selects the shipping and the quantity.
- The most recent `why` or `affects` sets the direction. After `why` a step is
  kept when it is the target or depends on it, which are the steps whose node
  `affects target` selects, and after `affects` when the target depends on it,
  which are the steps whose node `why target` selects. A reference's own step
  and `model` use the direction of `why`. `stated`, `derived`, `depth`, `diff`
  and `path` keep the direction and the steps' distances, so they compose in
  either order.
- The target is one node, resolved as a pipeline's source is. A target that is
  not a node of the model, such as an absent property or a deleted name, selects
  nothing until it is defined. A literal or any other expression that is not a
  reference raises `TypeError("Cannot explain <target>")`, and an undefined
  name raises `ReferenceError("<name> is not defined")`.
- Only `|>` followed by `path` and `(` is the stage; `path` stays an ordinary
  name everywhere else, and a bare `|> path` keeps its parse-time
  `TypeError("path is not an operation")`. A local or parameter, variable,
  function or class named `path` shadows the stage, as bindings shadow the
  state filters, `depth` and `diff`: the stage then raises
  `TypeError("path is not an operation")` before selecting its source.
- The selection stays live through every observation its source made, the
  structure between its steps and the target, and the target being declared,
  replaced or deleted. It renders as `selection|>path(target)`.
- The authoritative contract is the
  [derivation path selections](../nucleoid.spec.md#derivation-path-selections)
  section.

### Live structure

- A reasoning assignment follows both values and graph structure.
- Declaring, replacing, or deleting a relationship refreshes its derivation and impact, even when its value stays the same.
- Subscriptions are replaced on every evaluation, including reasoning evaluated through a function or a class-level property rule.
- An empty impact selection still follows its source; an empty model selection still follows model membership.
- Replacing an observer with an ordinary relationship adds it to the model; restoring the reasoning expression removes it again.
- An explanation of an absent property is empty and fills in when the property is defined.
- Deleting an explained source empties its selection; defining it again restores the explanation.
- A name that was never defined raises `ReferenceError`, as an ordinary read does.

```
a = 1
exposure = affects a
b = a + 1
c = b * 2
```

- `exposure` changes from `[]` to steps for `b` and `c` without changing `a`.
- After `b = 0`, `exposure` is empty again.

### Observation and rollback

- Reasoning observers are neither selected nor traversed by graph walks; observing a model does not change its derivation or impact.
- `.value` freezes a selection just as it freezes any other reading.
- Structural refreshes use the propagation queue and wait for queued upstream changes to settle.
- A failed transaction restores the selection, its subscriptions, and its pending work.
- A caught failure discards only structural refreshes queued by its failing body, retaining work queued before it.
- The accepted specification already implements selections, pipelines,
  `model`, `why`, and `affects`; the NUC 48 candidate adds only the two
  state filters, [NUC 62](nuc-0062.md) adds NUC 11's `all` and `any`
  quantifiers, [NUC 65](nuc-0065.md) adds its `suppose` hypotheticals
  without `diff`, the [NUC 82](nuc-0082.md) candidate adds its `depth` stage
  without the `depth(x, n)` call form, and the [NUC 83](nuc-0083.md)
  candidate adds its `diff` as the `|> diff` stage, without the bare `diff`
  and its `before` and `after` fields, and the [NUC 88](nuc-0088.md)
  candidate adds a `path(target)` stage that NUC 11 does not plan. Other
  NUC 11 operations remain proposals, not dependencies.

### Quantifiers

```
class Check
first = Check()
first.passed = true
ready = all c in Check: c.passed
failing = any c in Check: not c.passed
assert(ready, true)
assert(failing, false)
second = Check()
assert(ready, false)
assert(failing, true)
second.passed = true
assert(ready, true)
assert(failing, false)
ready
```

- This is the complete `Nucleoid states live facts with all and any` program
  in `tests/reference.md`, without its title and expected-return metadata; it
  returns `true`.
- [NUC 62](nuc-0062.md) adds `all NAME in COLLECTION: CONDITION` and
  `any NAME in COLLECTION: CONDITION`. The collection is a declared class's
  instances, in population order, or a list, including a reasoning selection
  such as `why c`. `all` is `false` at the first member whose condition is
  falsy and `true` otherwise, so an empty collection is `true`; `any` is
  `true` at the first truthy member and `false` otherwise.
- A declaration that uses a quantifier is live, with the reads and
  dependencies of `COLLECTION.every(NAME => CONDITION)` and
  `COLLECTION.some(NAME => CONDITION)`: it follows the population or list and
  what its condition read for the members it visited. A null collection or a
  null read stores `null`, and a collection that is neither null nor a list or
  instance list raises `TypeError("<collection> is not a collection")`.
- The condition is evaluated in place: it reads enclosing quantifier binders,
  function parameters and locals, `this`, and the instance of a class-level
  rule, so `all t in Task: any w in Worker: w.skill == t.skill` nests. The
  binder is not defined after the quantifier, and functions called from the
  condition still do not see it.
- The condition extends as far right as possible, like an arrow body, so
  parenthesize a quantifier to combine it, as in
  `(all v in scores: v > 0) ? "clean" : "dirty"`. Its collection ends at the
  quantifier's own colon. `if`, `else if` and direct-alternative headers, with
  or without their own colon, take quantifier conditions, and their header
  scans, including the native REPL's, pair that colon with its `all` or
  `any`.
- `all` and `any` stay ordinary names everywhere else, including
  `Name.all()`, the `.any(callback)` alias, variables, functions, keys and
  parameters. At the top level of a template interpolation a quantifier needs
  parentheses: `` `${(all v in xs: v)}` ``.
- The authoritative contract is the
  [quantifiers](../nucleoid.spec.md#quantifiers) section.

### Hypotheticals

- `suppose PREMISE:` followed by a suite runs the premise statement and the
  suite in a hypothetical world, then rolls back everything they did:
  writes, created instances, declarations, propagation, reasoning observers
  and local bindings. `suppose:` has no premise. The suite is an indented
  block or same-line statements separated by `;`, as for `if`.
- Rules propagate inside the block exactly as they do outside, so
  assertions there see the values the model derives in that world.
- `TARGET = suppose ...` keeps the value of the block's last expression as
  a snapshot. It is assigned as a literal with no dependency, so it does not
  follow later changes. An assigned block that does not end in an
  expression raises `TypeError("suppose has no value")` before it runs.
  Without a target, the statement's value is the block's value, or `null`,
  and the native REPL prints it.
- Values leave the block as plain data. Objects, instances and reasoning
  steps are copied into plain objects that keep the values they had in the
  hypothetical world, a class declared inside the block leaves as `null`,
  and a function stays callable. An error, a thrown value or a `return`
  leaves after the rollback.
- Blocks nest, and they work in functions, methods, loops and class-level
  rules; each run of the enclosing statement runs the block again. A block
  adds no dependency to whatever runs it. `suppose` stays an ordinary name
  everywhere else, and a colon-free `suppose x = 1` keeps meaning the name
  `suppose` followed by `x = 1`.

```
class Item(price): this.price = price
$Item.premium = $Item.price > 10
premium = Item.filter(item -> item.premium).length
pen = Item(3)
book = Item(12)
suppose Item(20): assert: premium, 2
raised = suppose pen.price = 15: premium
assert: raised, 2
assert: premium, 1
assert: Item.length, 2
suppose book.price = 5: premium
```

- This is the complete `Nucleoid answers what if questions with suppose`
  program in `tests/reference.md`, without its title and expected-return
  metadata; it returns `0`.
- The authoritative contract is the
  [hypotheticals](../nucleoid.spec.md#hypotheticals) section.

### Hypothetical differences

```
price = 10
qty = 3
shipping = 5
subtotal = price * qty
total = subtotal + shipping
free = total > 100
def impact() => (model |> diff).map(step => step.node)
bulk = suppose qty = 4: impact()
assert(bulk, ["qty", "subtotal", "total"])
price = 30
bulk = suppose qty = 4: impact()
assert(free, false)
bulk
```

- This is the complete `Nucleoid lists what a hypothetical changes with diff`
  program in `tests/reference.md`, without its title and expected-return
  metadata; it returns `["qty","subtotal","total","free"]`.
- [NUC 83](nuc-0083.md) adds the stage `selection |> diff`. Inside a
  `suppose` block it keeps the steps whose node the model did not hold when
  the innermost running block began, its premise included, or whose value
  differs from the value the node held then, in their order and with their
  fields unchanged. `model |> diff` therefore lists what a hypothetical
  added or changed without naming any node: a fourth unit changes the
  quantity, the subtotal and the total, and also `free` once the price is 30.
- Values compare as a write compares the value it stores with the one it
  replaces: numbers by value and sign with `NaN` equal to `NaN`, lists item
  by item, and objects, instances and function values by identity. Classes,
  named functions, conditions and blocks hold `null` in a step, so only
  their appearance counts. An equal-valued write, a restored value and a
  write undone by a caught failure are not differences.
- The running block is dynamic: a function, method or lambda called inside a
  block compares with that block, and a nested block with its own start.
  Outside every block the stage raises
  `TypeError("Cannot use 'diff' outside suppose")`.
- Only `|>` followed by `diff` is the stage; `diff` stays an ordinary name
  everywhere else, so a bare `diff` is still a reference. A local or
  parameter, variable, function or class named `diff` shadows the stage, as
  bindings shadow the state filters and `depth`: the stage then raises
  `TypeError("diff is not an operation")` before selecting its source.
- The selection stays live through every observation its source made,
  composes with `why`, `affects`, `stated`, `derived` and `depth` in any
  order, and renders as `selection|>diff`.
- The authoritative contract is the
  [hypothetical differences](../nucleoid.spec.md#hypothetical-differences)
  section.

Implemented semantic basis: the [root specification](../nucleoid.spec.md).
Proposal context: [NUC 11](nuc-0011.md), [NUC 48](nuc-0048.md),
[NUC 62](nuc-0062.md) and [NUC 65](nuc-0065.md).

---

## 13. Error Reference

Messages are exact and are part of the observable behaviour of the language.

### ReferenceError

| Condition | Message |
| --- | --- |
| Reading an undefined variable | `<name> is not defined` |
| Throwing an undefined reference | `<name> is not defined` |
| Using a deleted variable or instance | `<name> is not defined` |
| Creating an instance of an undeclared type | `<Name> is not defined` |
| Writing through an undefined instance property | `<path> is not defined` |
| Writing through an undefined class-level reference | `<Name>.<property> is not defined` |
| Reading `.value` of an undefined property | `<path> is not defined` |
| Reading `.value` of a property selector's undefined argument | `.value is not defined` |
| Declaring a variable with no value | `Missing definition` |

### TypeError

| Condition | Message |
| --- | --- |
| An assignment that would close a dependency cycle, unless every read on it is a fixed class history anchor | `Circular Dependency` |
| A type declaration that would close an inheritance cycle | `Circular Inheritance` |
| Deleting an instance that still has properties | `Cannot delete object '<id>'` |
| Using `value` as a property name | `Cannot use 'value' as a property` |
| Calling an unknown function of a built-in or list | `<Object>.<name> is not a function` |
| Missing or non-function `flatMap` callback, even on an empty receiver | `flatMap expects a function` |
| Missing or non-function `find`, `findIndex`, `findLast` or `findLastIndex` callback, even on an empty receiver | `<name> expects a function` |
| A supplied `sum` selector that is not a function, including `null`, even on an empty receiver | `sum expects a function` |
| A supplied `average` selector that is not a function, including `null`, even on an empty receiver | `average expects a function` |
| A supplied `median` selector that is not a function, including `null`, even on an empty receiver | `median expects a function` |
| A supplied `max` or `min` selector that is not a function, including `null`, even on an empty receiver | `max expects a function` or `min expects a function` |
| A supplied `distinct` selector that is not a function, including `null`, even on an empty receiver | `distinct expects a function` |
| Missing or non-function `count` predicate, including `null`, even on an empty receiver | `count expects a function` |
| A supplied `mode` selector that is not a function, including `null`, even on an empty receiver | `mode expects a function` |
| Missing or non-function `maxBy` or `minBy` selector, including `null`, even on an empty receiver | `maxBy expects a function` or `minBy expects a function` |
| An `all` or `any` quantifier over a value that is neither null nor a list or a declared class's instances | `<collection> is not a collection` |
| A `depth` stage limit that is neither `null`, undefined nor a non-negative integer number | `<limit> is not a non-negative integer` |
| A two-name `for key, value of source` loop over a value that is not an object | `<source> is not an object` |
| A spread entry `...source` whose source is neither an object nor `null` or undefined | `<source> is not an object` |
| Assigning, adding or deleting a property of an enum's object, through any name that holds it | `Enums are read-only` |
| Assigning a `suppose` block whose last statement is not an expression | `suppose has no value` |
| Evaluating a `diff` stage while no `suppose` block is running | `Cannot use 'diff' outside suppose` |
| `:=` on a class-level property outside a class-level rule | `Cannot use ':=' on a class-level property` |
| A `:=` declaration whose value nests lists 192 levels or more | `Maximum expression depth exceeded` |
| Explaining something that is not a reference, such as the power in `why x ** 2` | `Cannot explain <expression>`, for example `Cannot explain x**2` |
| Repeated string output is too large to represent or allocate | `Repeated string is too large` |
| A `while` loop whose condition still holds after 100,000 iterations of one run | `Loop did not terminate` |
| An undeclared constructor label | `Unknown constructor argument '<name>' for <class>` |
| A repeated constructor binding, including positional plus named | `Duplicate constructor argument '<name>' for <class>` |
| A positional constructor argument following a label | `Positional constructor argument after named arguments for <class>` |
| A label targeting a rest formal | `Rest parameter '<name>' cannot be a named constructor argument for <class>` |

### SyntaxError

| Condition | Message |
| --- | --- |
| Declaring a class-level rule in a non-class block | `Cannot define class declaration in non-class block` |
| A character that begins no token | `Unexpected character '<character>'` |
| A token other than the one required | `Expected <what> but found <what>` |
| A formal after a rest formal | `Rest parameter must be last` |
| A default on a rest formal | `Rest parameter cannot have a default` |
| A rest name duplicating a fixed formal | `Rest parameter '<name>' duplicates a fixed parameter` |
| An unsupported compound target | `Invalid compound assignment target` |
| An unsupported `:=` target, including an assignment such as `y = x := 1` | `Invalid value assignment target` |
| `**` or `**=` where an operand is required | `Unexpected '**'`, `Unexpected '**='` |
| Deleting something that is not a variable, property or instance, such as the power in `delete p.v ** 2` | `Cannot delete <expression>`, for example `Cannot delete p.v**2` |
| A block or template that is never closed | `Unterminated block`, `Unterminated template expression` |
| Nesting or a chain past the runtime's limit | `Expressions are nested too deeply`, `Expression has too many operands` |
| A postfix chain, a compound path or a property selector beyond its limit | `Expression has too many parts` |
| A destructuring with more than 128 targets | `Destructuring has too many targets` |
| An enum member named twice in one declaration, at the second | `Enum '<name>' already has member '<member>'` |
| A one-line enum with a single member | `Enum '<name>' needs at least two members on one line` |
| An enum member value that is not a scalar literal | `Expected a scalar literal enum member value` |
| An enum member value that is a number but not finite | `Enum member value must be a finite number` |
| A constructor-header star at a parameter position | `Expected a parameter but found '*'` |
| A `this.` or `self.` prefix outside a constructor parameter list, or not followed by one name and then `,`, `)`, `:` or `=` | `Expected a parameter but found keyword 'this'` |
| An `extends` clause whose parent is missing or not a name | `Expected a parent class name but found <what>` |
| A second parent after `class Name extends Parent:` | `Class '<Name>' already extends '<Parent>'` |
| A list comprehension whose name is not followed by `in` or `of` | `Expected 'in' or 'of' but found <what>` |

### Where an error happened

- Every error records a line and column: a syntax error at the token the parser
  stopped at, a runtime error at the start of the top-level statement that was
  running.
- The position is part of how an error is reported, not part of what it is. What a
  `catch` binds is `<Kind>: <message>` with no position, so a program can compare
  against it wherever the statement is written.

```
ReferenceError: e is not defined (line 3, column 1)
```

Full detail: [NUC 9](nuc-0009.md).

---

## 14. Syntax Summary

The complete grammar, as the parser accepts it, is in [grammar.md](grammar.md).

```
name = expression                      variable assignment
name := expression                     value assignment: once, no dependency (NUC 66)
name.property := expression            value assignment to a property (NUC 66)
name is expression                     is assignment: name = expression (NUC 93)
name.property is word                  a bare word reads its name, else its text "word" (NUC 93)
name += expression                     compound assignment; also -=, *=, **=, /=, %=
{a, b} = expression                    one declaration per field: a = expression.a (NUC 67)
{key: name} = expression               renamed field: name = expression.key (NUC 67)
a, b = expression                      one declaration per position: a = expression[0] (NUC 67)
enum Name: A, B, C = 1                 read-only object; a bare member holds its own name (NUC 90)
enum Name:                             the same enum, one member per indented line (NUC 90)
    A
    B = "b"
name = a; name = b                     several statements on one line
name: type                             declaration without value -- rejected
name.property = expression             property assignment
$Type.property = expression            class-level rule
delete name                            remove a variable, property, or instance

class Name                             empty type declaration
class Name(param: type)                 empty type with parameters; no implicit properties

class Name(param: type):               type with a constructor
    this.param = param

class Name(this.param: type)           the same constructor, assigned by the prefix (NUC 87)
class Name(self.a, b = 1, ...this.c)   either receiver; defaults and a final rest kept
class Name:
    def init(this.param):              the same prefix in an init of the class body

instance = Name(param=expression)      label binding without caller assignment
instance = Name((param=expression))    explicit positional assignment argument
instance = Name(first, second=value)   positional arguments followed by labels

class Name:                            type with a typed attribute
    attribute: type

    def init(param: type):
        this.attribute = param

class Sub: Super                       subtype
    def init(a, b):
        super(a)
        this.b = b
class Sub extends Super:               same subtype, extends spelling (NUC 78)
    statement
class Sub(param) extends Super         bodyless subtype, as class Sub(param): Super with pass (NUC 78)

super                                  zero-argument super() expression
super(arguments)                       explicit parent-constructor arguments

if condition:                          conditional
    statement
if condition                           same conditional, genuine indentation only (NUC 49)
    statement
else if condition:
    statement
else if condition                      optional colon at genuine indentation
    statement
else condition:                        optional direct alternative, indented only (NUC 43)
    statement
else condition                         same direct alternative without its colon
    statement
elif condition:                        equivalent existing Else alias, indented only
    statement
elif condition                         same direct alternative without its colon
    statement
else:
    statement
elif:                                  existing unconditional Else alias
    statement
else                                   existing colon-free final indented suite
    statement
elif                                   same unconditional alias without a colon
    statement

condition ? consequent : alternate     conditional expression (NUC 59)
consequent if condition else alternate same expression, Python spelling
left ?? right                          left unless it is null, else right (NUC 60)
if value is "EXACT":                   value == "EXACT" in a condition or other enclosed position (NUC 86)
if value is EXACT:                     a bare word reads its variable, else its own text "EXACT" (NUC 86)
flag = (value is not EXACT)            != with the same word rule; bare at statement level is a name (NUC 86)
base ** exponent                       exactly Math.pow(base, exponent), right-associative (NUC 64)
-x ** 2                                prefix operators apply to the whole power: -(x ** 2) (NUC 64)
name **= exponent                      name = name ** (exponent) (NUC 64)

for name of source:                    class/object-list/scalar-string iteration
for name in source:                    equivalent explicit iteration
for key, value of source:              object entries in insertion order (NUC 72)
for key, value in source:              equivalent two-name iteration (NUC 72)
for source:                            indented iteration with local it
    statement
while condition:                       repeat while the condition holds (NUC 63)
    statement
while condition: statement             same loop with a same-line body

value => expression                    anonymous arrow; also (a, b) => expression
value => { statement }                 anonymous arrow with a statement body
value -> expression                    optional spelling wherever => is accepted (NUC 57)
lambda a, b: expression                optional lambda, exactly (a, b) => (expression) (NUC 57)
f(.a.b)                                property selector argument, exactly f(v => v.a.b) (NUC 79)
[expr for name in source]              list comprehension, exactly source.map(name => expr) (NUC 92)
[expr for name of source if cond]      source.filter(name => cond).map(name => expr) (NUC 92)
def name(params) => expression         single-return function
def name(head = 1, ...tail) => expression final positional rest list (NUC 74)
def name(params) =>                    optional indented single expression (NUC 32)
    expression
def name left, right => expression    untyped multiple-parameter named arrow
def name left right => expression     optional comma-free untyped named arrow (NUC 37)
def name left right:                  optional comma-free untyped block (NUC 40)
def name left: int, right => expression typed/mixed multiple-parameter arrow (NUC 29)
def name amount => expression         one untyped parameter (NUC 18)
def name amount: type => expression   one typed parameter (NUC 20)
def name(params):                      block function
    statement
def name left, right:                  untyped multiple parameters, indented only (NUC 28)
    statement
def name left: int, right:             typed/mixed multiple parameters, indented only (NUC 30)
    statement
def name amount:                       one untyped parameter, indented only (NUC 19)
    statement
def name amount: type:                 one typed parameter, indented only (NUC 24)
    statement
function name(params) { statement }    the same block function, braced spelling (NUC 89)

{ statement; statement }               block
record = { amount, label }             object shorthand (candidate NUC 17)
record = {amount: amount unit: "C"}    optional explicit-property comma (NUC 44)
record = {...base, qty: 1}             copy base's entries in order; later entries win (NUC 81)

throw value                            raise a value
return expression                      yield from a block or function
assert(actual, expected)                existing equality-check helper
assert: actual, expected                optional statement spelling (Accepted NUC 45)

name.value                             read without depending
items.flatMap(callback)                 ordered one-level projection (Accepted NUC 41)
Math.remap(start, end, targetStart, targetEnd, value) unclamped static remapping (Accepted NUC 39)
Math.toRadians(value)                   static division-first degree conversion (NUC 50 candidate)
Math.exp2(value)                        static powf-based base-two power (Accepted NUC 56)
Name[0]  Name["id"]  Name.find(...)    address an instance
value[start:end]  value[-n:]           slice, negative counts from the end
Name[start:end]  Name[:]  Name[-n:]    instance list range (NUC 61)
Name.list()  Name.all()                every instance, in population order (NUC 61)
items.at(-1)  items.findLast(callback) read or search from the end (NUC 61)
items.sum()  Name.sum(selector)        live numeric total (NUC 68)
items.max()  Name.min(selector)        live numeric extreme (NUC 69)
items.distinct()  Name.distinct(selector) live values without repeats (NUC 71)
items.count(predicate)  Name.count(predicate) live predicate count (NUC 73)
items.mode()  Name.mode(selector)      live most frequent item (NUC 75)
items.maxBy(key)  Name.minBy(key)     live first item with the largest or smallest key (NUC 77)
items.average()  Name.average(selector) live mean: exactly items.sum() / items.length (NUC 80)
items.median()  Name.median(selector)  live middle value; even counts take (lower + upper) / 2 (NUC 84)
mode of items  sum of Name.map(f)      noun query: exactly items.mode(), Name.map(f).sum() (NUC 75)
sum of items ** 2                      a noun query applies to the whole power: sum of (items ** 2) (NUC 75)

why reference                         derive a live explanation
affects reference                     inspect live downstream impact
reference |> why |> affects           compose reasoning stages
model |> why                          inspect the current model
reference |> why |> stated            select stated supporting facts (NUC 48 candidate)
reference |> why |> derived           select derived explanation steps (NUC 48 candidate)
stated reference                      filter one reference without traversing (NUC 48 candidate)
derived reference                     filter one reference without traversing (NUC 48 candidate)
model |> stated                       filter the current model by stated state (NUC 48 candidate)
model |> derived                      filter the current model by derived state (NUC 48 candidate)
reference |> why |> depth(1)          the reference and what it reads directly (NUC 82)
reference |> affects |> depth(1)      what reads the reference directly (NUC 82)
selection |> depth(limit)             steps within limit steps of where the last walk began (NUC 82)
reference |> why |> path(target)      how target reaches the reference (NUC 88)
reference |> affects |> path(target)  what the reference moves on its way to target (NUC 88)
selection |> path(target)             steps on a dependency path to target (NUC 88)
all name in collection: condition     true when the condition holds for every member (NUC 62)
any name in collection: condition     true when the condition holds for some member (NUC 62)
(all name in collection: condition)   parenthesize to combine with what follows (NUC 62)
suppose premise: suite                run premise and suite, then roll both back (NUC 65)
suppose: suite                        a hypothetical without a premise (NUC 65)
name = suppose premise: suite         keep the block's last expression value (NUC 65)
model |> diff                         what the running suppose block added or changed (NUC 83)
reference |> why |> diff              the changed steps of an explanation (NUC 83)
selection |> diff                     steps new or changed since the running block began (NUC 83)
```

---

## Copyright

Copyright 2020 Nucleoid

This document is licensed under the Apache License, Version 2.0. See
[LICENSE](../LICENSE) for the full text.

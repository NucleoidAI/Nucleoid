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
Acceptance requires every case to use the same or fewer tokens and behavior
checks to pass; any increase rejects it. Draft or rejected proposals do not
change this implemented reference. See [NUC 1](nuc-0001.md#proposal-evaluation).

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

### Null and undefined

- Any `null` dependency yields `null`.
- Reading something undefined yields `null` and re-evaluates when it arrives.

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
  only the existing newline/semicolon separators; fields still need commas.
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

### Operators

- Arithmetic: `+`, `-`, `*`, `/`, `%`.
- Compound assignment: `+=`, `-=`, `*=`, `/=`, `%=`; each operator is one token.
- Logical compound assignment: `&&=`, `||=`; adjacent characters form
  one token. `&& =`, `|| =`, `and=`, and `or=` do not form compound operators.
- Comparison: `==`, `!=`, `<`, `>`, `<=`, `>=`.
- Logical: `and` / `&&`, `or` / `||`, `not` / `!`.
- Parentheses override precedence.
- `+` concatenates when an operand is a string.

```
assert(a + b * c, 14)
assert((a + b) * c, 20)
```

- A comparison may be assigned directly: `alarm = reading > threshold`.

### Indexing, slicing, and length

- Lists and instance lists index with brackets; the index may be a variable.
- Object bracket access has the same null, undefined, and dependency behavior as a dotted property read.
- A range is written `[start:end]`; either bound may be omitted, and a negative bound counts from the end.
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
- An indented body still requires a colon; inheritance keeps its existing colon grammar.
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

### Instance keywords

- `this` and `self` are interchangeable keywords for the current instance in constructors, methods, and class-level rules.
- Either spelling reads and writes the same properties; both may be mixed in one body without declaring a receiver parameter.
- Neither keyword may be rebound. Using either without a current instance raises `ReferenceError("this is not defined")`; rebinding either raises `TypeError("Cannot assign to 'this'")`.

### Instances

- Creating an instance registers it in the type's list and returns it.
- The variable name becomes the instance `id`.
- An instance created without a variable name gets a generated `id`.
- Instances are addressed by index `Name[0]`, by id `Name["id"]`, or by query `Name.find(...)`.

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

Full detail: [NUC 3](nuc-0003.md); accepted compact headers: [NUC 14](nuc-0014.md); candidate bare calls: [NUC 16](nuc-0016.md).

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

Full detail: [NUC 5](nuc-0005.md).

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
- A false condition leaves the target untouched rather than clearing it.
- Re-declaring a conditional replaces it.
- Conditionals apply in the order received; when several match, the last one holds.
- Re-evaluating one conditional does not move it after the others.
- A conditional can run again when an input changes later in the same cascade. An unchanged write does not restart feedback.

```
if toy1.color == "RED":
    toy1.shape = "CIRCLE"
```

- `null` until `toy1.color` is `"RED"`.

### Truthiness

- A condition need not be a comparison; a bare value is tested for presence.
- `null`, an undefined property, and `0` are false; a non-empty value is true.
- An instance is true, so a query result can be tested directly.
- `not` inverts the test.

```
if $Device.profile:
    $Device.active = true
```

- Only devices that have a profile become active.

### for ... of

- `for <name> of <source>` iterates a class list or a list value.
- The body runs once, over the values present at the time; it does not re-run later.
- Nested conditions and blocks are part of that one-shot body, not independent standing rules.
- Only values that are instances defined in the state are visited.
- Loops may nest, producing the cross product.

```
for question of Question:
    Summary(question)
```

Full detail: [NUC 7](nuc-0007.md).

---

## 9. Functions

- `def` defines a function in the state; a function with no `return` yields `null`.
- `def name(params) => expression` is exactly a named block containing `return expression`; methods and parameter annotations use the same form.
- [NUC 18](nuc-0018.md) specifies `def name amount => expression` as exactly `def name(amount) => expression` for one untyped ordinary identifier parameter, including methods and `init`.
- [NUC 19](nuc-0019.md) specifies `def name amount:` followed by an indented suite as exactly `def name(amount):` with that same suite.
- [NUC 20](nuc-0020.md) specifies `def name amount: int => expression` as exactly `def name(amount: int) => expression` for one ordinary identifier parameter and one existing ordinary-identifier annotation, including methods and `init`.
- [NUC 24](nuc-0024.md) describes `def name amount: int:` followed by a real indented suite as exactly `def name(amount: int):` with that same suite, including methods and `init`.
- [Accepted NUC 27](nuc-0027.md) adds optional `def name left, right => expression`, exactly equivalent to `def name(left, right) => expression`, including methods and `init`.
- [NUC 28](nuc-0028.md) adds optional `def name left, right:` with a real indented body, exactly equivalent to `def name(left, right):` with that same body, including methods and `init`.
- The multiple-parameter arrow guard requires the complete immediate sequence `Identifier Comma Identifier (Comma Identifier)* Arrow` on the same logical header line. Every parameter is untyped; the non-consuming guard skips no separators or comment-induced logical breaks.
- The multiple-parameter block guard shares the complete identifier/comma scan, ending in `Colon` instead. Require immediate `Newline` or `Indent` and then a real `Indent` after only normal newline/semicolon layout; `:;` never qualifies. The final colon and all layout/body tokens are left for the existing suite parser.
- The typed-arrow guard requires immediate `Identifier`, `Colon`, `Identifier`, `Arrow` tokens after the required definition name. The parameter name and `type_name: Some(annotation)` metadata are unchanged; the annotation is not a resolved variable or a new runtime type constraint.
- The typed-block guard requires immediate `Identifier`, `Colon`, `Identifier`, `Colon` tokens. The next token is `Newline` or `Indent`, and the first other token after only newline/semicolon layout is `Indent`; the guard consumes nothing before the existing parameter and suite parsers.
- For the single-parameter untyped forms, a bare identifier must be immediately followed by `=>`, or by `:` whose body starts with a logical newline/indent and supplies a real `Indent` after normal newline/semicolon layout. The colon guard only looks ahead; the existing suite parser consumes the body.
- No logical newline, semicolon or comment-induced logical break is skipped within a bare header. Blank/comment lines before an indented body retain the existing layout rules.
- All parenthesized definitions remain available, including `def add(left, right) => left + right` and `def add(left, right):` with the same indented body. Nullary, typed/mixed multiple-parameter, multiline-parameter, inline and brace-suite definitions keep their parentheses, as do calls; typed single-parameter indented blocks have the optional spelling below.
- Bare inline/braced or missing-indent blocks, typed/mixed multiple-parameter arrows/blocks, trailing bare commas, defaulted/destructured/variadic parameters, dotted/union/generic/keyword/literal annotations, anonymous colon blocks, return annotations and call-parenthesis omissions are not added; class and anonymous parameter lists keep their existing grammar.
- NUC18/20 single-parameter arrows and NUC19/24's exact untyped/typed indented-colon guards are unchanged; named return-block lowering, parameter metadata/order, argument binding, runtime behavior and canonical rendering remain those of the parenthesized counterparts.
- Braces after a named arrow are object literals, not statement blocks. Parenthesized expressions may span physical lines.
- A call in an assignment depends on the function and its arguments.
- Redefining a function updates existing dependents.
- A named function and a variable binding under the same name replace one another; reads and calls both use the latest binding.
- Named function references stay live when aliased, returned, or passed as callbacks.
- Static Math method reads are not callable references; `Math.clamp` retains
  that limitation, as specified by [NUC 21](nuc-0021.md).
- Calls through variables, list items, and object properties depend on the callable's storage location.
- A null callable, an unassigned callable property, or a callable deleted during propagation leaves its dependent result null until rebound. Other non-callable values raise `TypeError`.
- `.value` freezes the function definition, not the state read when its body executes.
- Functions may recurse and may have multiple `return` statements.

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
def area width, height => width * height * scale
width = 3
height = 4
result = area(width, height)
assert(result, 12)
scale = 2
assert(result, 24)
width = 5
assert(result, 40)
```

- This is the executable reference's exact paired `Nucleoid preserves bare multiple parameter arrow relationships` example; `def area(width, height) => width * height * scale` remains valid and behavior-equivalent.
- Reuse the existing named return block and expression-body parser, with no new lexer/public AST/runtime types, capture, parameter-count bound or canonical spelling.
- `def f left, right, => left`, typed/mixed bare lists, multiple-parameter blocks without real indentation, bare inline/braced suites and logical header breaks retain `SyntaxError("Expected '(' but found identifier 'left'")` at line 1, column 7; unsupported prefixes are not consumed.
- `def f left, right =>` gives `SyntaxError("Unexpected end of input")` at line 1, column 21; a final newline gives `SyntaxError("Unexpected end of line")` at line 2, column 1. Named-arrow braces remain objects; a body beginning with `(` on the header line may span physical lines.
- Full grammar and error positions are in the authoritative [bare multiple-parameter named arrows](../nucleoid.spec.md#bare-multiple-parameter-named-arrows) section.

### Bare typed single parameter named arrows

```
def increment amount: int => amount + 1
def text value: str => value

assert(increment(4), 5)
assert(text("ok"), "ok")
```

- This is the executable reference's paired typed-arrow example; `def increment(amount: int) => amount + 1` and `def text(value: str) => value` remain valid and behavior-equivalent.

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

- This is the executable reference's exact paired `Nucleoid preserves bare multiple parameter block relationships` example; `def area(width, height):` with the same body remains valid and behavior-equivalent.
- Two or more untyped ordinary identifiers retain their order and `type_name: None`; the complete named block AST and canonical rendering are identical to the parenthesized form. There is no parameter-count cap.
- The body uses the existing suite grammar, including multiple statements, nested definitions, branches and existing inner inline/braced statements. Explicit returns, bare returns and no return keep their existing semantics; the latter two return `null`.
- `def f left, right:` without a real indent, typed/mixed multiple headers, trailing bare commas, inline/braced bare suites, `:;` and logical header breaks keep `SyntaxError("Expected '(' but found identifier 'left'")` at line 1, column 7.
- After the qualifying header, four-space-indented `return left +)` keeps `SyntaxError("Unexpected ')'")` at line 2, column 19; `return @` keeps `SyntaxError("Unexpected character '@'")` at line 2, column 12.
- Full grammar, exact errors and positions are in the authoritative [bare multiple-parameter indented block definitions](../nucleoid.spec.md#bare-multiple-parameter-indented-block-definitions) section. Receiver/init/super behavior, shadowing, arity, live references, definition snapshots, redefinition, dependencies, graph keys, transactions and resource limits are unchanged.

### Lambdas

- Three equivalent forms; parameters shadow outer names, including outer functions when the parameter is null or non-callable.

```
list.find(function(element) { return element == 3 })
list.find(element => { return element == 2 })
list.find(element => element == 1)
```

- Calls over class lists depend on the population and the properties they read.

Full detail: [NUC 8](nuc-0008.md), [NUC 12](nuc-0012.md), [NUC 18](nuc-0018.md), [NUC 19](nuc-0019.md), [NUC 20](nuc-0020.md), [NUC 24](nuc-0024.md), [NUC 27](nuc-0027.md), and [NUC 28](nuc-0028.md).

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

```
a = 5

if a > 5:
    throw 'INVALID_VALUE'
```

- Assigning `a = 6` throws; afterwards `a` is `5`.

Full detail: [NUC 9](nuc-0009.md).

---

## 11. Built-in Objects

- Built-in calls participate in the dependency graph like any other expression.
- Mutating a list re-evaluates its dependents.
- `Math.max` and `Math.min` propagate `NaN`, including when an argument cannot be coerced to a number.
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

| Object | Members |
| --- | --- |
| `Object` | `Object()` -- a plain object, not an instance of any type |
| `Boolean` | `Boolean(value)` |
| `Number` | `Number.MAX_INTEGER` |
| `Math` | `pow`, `sqrt`, `floor`, `round`, `max`, `min`, `clamp` ([NUC 21](nuc-0021.md)), `lerp` ([NUC 22](nuc-0022.md)), `saturate` ([NUC 23](nuc-0023.md)) |
| `String` | `String.fromCharCode`; instance `charAt`, `length`, `lower`, `replace`, `repeat`, `split` |
| `Date` | `Date()`, `Date(string)`, `Date(number)`, `Date.now()`, `Date.parse()`; instance `getTime`, `getYear`, `toDateString` |
| `List` | `push`, `pop`, `filter`, `map`, `reduce`, `find`, `some`, `every`, `slice`, `sort`, `join`, `length` |
| `Class` | the global list of declared types; `Class.length` |

```
prices = [10, 20, 30]
rate = 2
doubled = prices.map(p => p * rate)
```

- `doubled[2]` is `60`; after `rate = 3`, `90`; after `prices.push(40)`, `doubled[3]` is `120`.

- Class lists support the same functions and depend on the population.

```
open = Task.filter(t => not t.done).length
```

- Regular expressions support `.test(...)` and may be used in any condition.

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
- Only selections, pipelines, `model`, `why`, and `affects` are implemented; the other NUC 11 operations remain proposals.

Full detail: [NUC 11](nuc-0011.md).

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
| Declaring a variable with no value | `Missing definition` |

### TypeError

| Condition | Message |
| --- | --- |
| An assignment that would close a dependency cycle | `Circular Dependency` |
| A type declaration that would close an inheritance cycle | `Circular Inheritance` |
| Deleting an instance that still has properties | `Cannot delete object '<id>'` |
| Using `value` as a property name | `Cannot use 'value' as a property` |
| Calling an unknown function of a built-in or list | `<Object>.<name> is not a function` |
| Repeated string output is too large to represent or allocate | `Repeated string is too large` |

### SyntaxError

| Condition | Message |
| --- | --- |
| Declaring a class-level rule in a non-class block | `Cannot define class declaration in non-class block` |
| A character that begins no token | `Unexpected character '<character>'` |
| A token other than the one required | `Expected <what> but found <what>` |
| An unsupported compound target | `Invalid compound assignment target` |
| A block or template that is never closed | `Unterminated block`, `Unterminated template expression` |
| Nesting or a chain past the runtime's limit | `Expressions are nested too deeply`, `Expression has too many operands` |
| A postfix chain or a compound path beyond its limit | `Expression has too many parts` |

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

```
name = expression                      variable assignment
name += expression                     compound assignment; also -=, *=, /=, %=
name = a; name = b                     several statements on one line
name: type                             declaration without value -- rejected
name.property = expression             property assignment
$Type.property = expression            class-level rule
delete name                            remove a variable, property, or instance

class Name                             empty type declaration
class Name(param: type)                 empty type with parameters; no implicit properties

class Name(param: type):               type with a constructor
    this.param = param

class Name:                            type with a typed attribute
    attribute: type

    def init(param: type):
        this.attribute = param

class Sub: Super                       subtype
    def init(a, b):
        super(a)
        this.b = b

super                                  zero-argument super() expression
super(arguments)                       explicit parent-constructor arguments

if condition:                          conditional
    statement
else if condition:
    statement
else:
    statement

for name of source:                    iteration
    statement

def name(params) => expression         single-return function
def name left, right => expression    untyped multiple-parameter named arrow
def name amount => expression         one untyped parameter (NUC 18)
def name amount: type => expression   one typed parameter (NUC 20)
def name(params):                      block function
    statement
def name amount:                       one untyped parameter, indented only (NUC 19)
    statement
def name amount: type:                 one typed parameter, indented only (NUC 24)
    statement

{ statement; statement }               block
record = { amount, label }             object shorthand (candidate NUC 17)

throw value                            raise a value
return expression                      yield from a block or function

name.value                             read without depending
Name[0]  Name["id"]  Name.find(...)    address an instance
value[start:end]  value[-n:]           slice, negative counts from the end

why reference                         derive a live explanation
affects reference                     inspect live downstream impact
reference |> why |> affects           compose reasoning stages
model |> why                          inspect the current model
```

---

## Copyright

Copyright 2020 Nucleoid

This document is licensed under the Apache License, Version 2.0. See
[LICENSE](../LICENSE) for the full text.

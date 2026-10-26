# Nucleoid Language Reference

NUCs in `docs/` propose language changes; draft or rejected proposals do not
override this specification. Accepted rewrites preserve case behavior and must
not increase any case's lexer token count under the process in
`docs/nuc-0001.md`.

`this` and `self` are interchangeable keywords for the current instance in
constructors, methods, and class-level rules. Both spellings share the same
receiver, property access, and errors. Neither keyword may be rebound, and using
either without a current instance raises `ReferenceError("this is not defined")`.

Expression nesting is bounded, including consecutive unary operators and
`new`/`delete` prefixes. Excessive nesting raises
`SyntaxError("Expressions are nested too deeply")` rather than overflowing
the host stack.

Compound arithmetic assignments `+=`, `-=`, `*=`, `/=`, and `%=` lower
`target op= expression` to `target = target op (expression)` at the existing
right-associative assignment precedence. They preserve standing relationships,
self-reference, `.value` snapshots, locals, receivers, transactions, return
values, and canonical AST rendering. A tracked right-hand dependency still
re-evaluates the relationship; a class-level self-reference still raises
`TypeError("Circular Dependency")` rather than becoming an imperative update.

Logical compound assignments `&&=` and `||=` are optional single-token
spellings of `target = target && (rhs)` and `target = target || (rhs)`.
They lower to the existing `Expr::Logical` inside the existing assignment
node, with the same full right operand, assignment precedence and right
associativity as arithmetic compounds. A standing assignment always installs
the expanded relationship, even when the logical right operand is skipped;
these are not conditional writes. Logical evaluation preserves operand values
and short-circuiting, not Boolean coercion. Existing null/deferred assignment
handling and `.value` freezing before evaluation remain unchanged, including
snapshot reads in a skipped logical operand. Names, properties, locals,
receivers, dependencies, class-level cycle checks, expression returns,
transactions, graph keys and canonical expanded rendering use the existing
assignment behavior.

Compound targets are names or plain dotted paths rooted in a name, `$Class`,
`this`, or `self`; a class or receiver root needs a property. Parentheses may
group such a path. Computed targets, including indexing, slicing, calls, and
assignments inside the target, raise
`SyntaxError("Invalid compound assignment target")`. Validation precedes
target cloning and permits at most 128 dotted links across grouped paths;
exceeding that bound raises `SyntaxError("Expression has too many parts")`.
Existing expression, statement, and operator-chain limits remain in force.
Ordinary `=` syntax and its existing target handling remain unchanged.
The bound and validation apply equally to arithmetic and logical compounds.

Each compound operator is one lexer token. Adjacent unary signs are separate
tokens; `/=` is distinguished from division and from a regular expression
such as `/=/` in an operand position, including after `&&=` and `||=`.
Diagnostics name the operator exactly as `'+='`, `'-='`, `'*='`, `'/='`,
`'%='`, `'&&='`, or `'||='`. The compound characters must be adjacent;
`&& =`, `|| =`, `and=`, and `or=` are not compound operators. Existing `=`,
`&&`, `||`, `and`, `or`, arithmetic compound operators, comparisons, arrow
tokens, and regex operand disambiguation retain their syntax and behavior.

Propagation settles queued upstream dependencies before their dependents and
preserves declaration order among ready rules. A rule may run again when another
dependency changes later in the same cascade; storing the same value does not
restart feedback. Nonsettling propagation raises
`TypeError("Propagation did not settle")` and rolls back the transaction.

Reasoning selections subscribe to both values and graph structure. Declaring,
replacing, or deleting a model node refreshes affected selections even when its
value is unchanged. Structural subscriptions are replaced on re-evaluation and
rolled back with their transaction. Reasoning observers are neither selected nor
traversed by graph walks, and are excluded from the membership of `model`.

An empty root class may be declared as `class Entity`, or with the existing
optional parameter list as `class Shell(code: int)`. Both lower to the same
class declaration as an indented pass-only body: no parent, fields, constructor
statements, or methods. Parameters do not implicitly create or assign
properties. Registration, instance lists, rules, redefinition, graph keys, and
transactions are unchanged; explicit colon/body declarations remain valid.

A bodyless header ends at a logical newline, semicolon, end of input, or the
enclosing block's closing brace or dedent. The terminator and the following
statement are not part of a constructor. Newlines inside a parameter list are
not statement boundaries. A header followed by indentation still requires a
colon, including across blank lines, comments, or semicolons. Without it the
parser raises `SyntaxError("Expected ':' but found end of line")`, or
`SyntaxError("Expected ':' but found ';'")` when the first separator is a
semicolon. Other unsupported tokens use `Expected ':' but found <token>` with
the existing token descriptions. Existing name, parameter, nesting, and
operator-chain diagnostics and source positions are preserved.

## Reactive literal string replacement

- `receiver.replaceAll(search, replacement)` is a string-instance method that
  returns a new string without mutating the receiver.
- Match the search literally, non-overlapping and left to right. Insert the
  replacement literally; regular expressions, callbacks, dollar expansion and
  backslash expansion are not replacement modes.
- Empty search inserts at the start, between Unicode scalar values and at the
  end. The empty receiver has one boundary, not two. Scalar boundaries agree
  with existing string indexing and empty-separator `split`, not UTF-16 units.
- Snapshot the receiver before evaluating arguments. Every supplied argument
  expression runs once, left to right, including unused extras, before string
  conversion or replacement. Earlier failures stop later expressions; argument
  effects and failures use existing transactions and source positions.
- Convert only the first two argument values using the same `Value::to_string`
  behavior as `replace`. Each omission supplies `Undefined`, whose ordinary
  spelling is `"undefined"`. Lists, objects, classes, functions, dates, regexes,
  numbers, booleans and literal null retain their existing string conversion.
- Default/literal values do not manufacture binding reads. Actually reading a
  null or undefined binding/member/parameter still has its inherited
  null/deferred assignment behavior, including reads in unused extra arguments.
  Null or undefined receivers still skip arguments and can recover when defined.
- Actual receiver and argument reads remain reactive. Re-evaluation replaces
  obsolete dependencies; global/property rules, class rules, ordinary scopes,
  noncapture, live/frozen function definitions and `.value` remain unchanged.
- The operation checks complete output-byte arithmetic and fallibly reserves
  the output before writing. Arithmetic overflow or failed output reservation
  raises `TypeError("Replaced string is too large")`; no arbitrary cap is added.
  Existing conversion/allocation behavior outside this output buffer is unchanged.
- Existing `replace` stays first-only; `split`, `join` and other methods keep
  their behavior. Native method properties remain non-first-class. Ordinary
  functions, callable object properties and instance methods named `replaceAll`
  retain their old routing; static `String.replaceAll` remains unsupported and
  raises `TypeError("String.replaceAll is not a function")` after arguments.
- No alias, grammar, public Rust API type/signature, dependency or version
  change is part of this addition. [Accepted NUC 42](docs/nuc-0042.md) explains
  the design; the five appended literal-replacement entries specify executable
  results, Unicode boundaries, live inputs, class rules and rollback/recovery.

## Bare single-parameter named arrows

A named arrow definition or method may spell `def double(amount) => amount * 2`
as `def double amount => amount * 2`. Only an ordinary identifier immediately
followed by `=>` after the required definition name qualifies. It becomes one
untyped parameter in the existing named function's block containing
`return expression`; expression precedence, argument binding, receiver and
initializer handling, reactive dependencies, callable snapshots, transactions,
resource bounds, and canonical rendering are unchanged.

No logical newline or semicolon is skipped in the header. The existing inline
body skips no separator before its first expression; NUC32 separately permits
the genuinely indented single-expression layout specified below. An expression
that starts with a parenthesis on the header line can still continue over
physical lines. Named arrow braces still denote object expressions; returned
anonymous functions keep their own existing body grammar and do not gain
closure capture.

Apart from NUC27's untyped, NUC37's comma-free untyped and NUC29's typed/mixed multiple-parameter arrows,
NUC28's untyped and NUC30's typed/mixed multiple-parameter indented blocks and
the narrow typed-arrow and single-parameter indented-colon forms below, other
prefixes still require parameter parentheses. Defaulted, destructured and nullary
bare parameters are not supported; bare inline suites and brace suites are not added.
For `def f amount:`, `def f amount: int, other, => amount`,
`def f amount, other, => amount`, and headers separating `amount` from `=>`
by a logical newline or semicolon, the error remains
`SyntaxError("Expected '(' but found identifier 'amount'")` at line 1, column 7.
`def f => 1` remains `SyntaxError("Expected '(' but found '=>'")` at line 1,
column 7. Parenthesized typed, untyped, empty, multiple-parameter and block
definitions remain valid. No call-site omission or ordinary-identifier
automatic invocation is introduced.

## Bare multiple-parameter named arrows

- NUC27 optionally spells `def add(left, right) => left + right` as
  `def add left, right => left + right`, including named methods and `init`.
  Require at least two untyped ordinary identifier parameters on the same
  logical header line, with the complete comma-separated sequence immediately
  followed by `Arrow`.
- After `def` and the required ordinary definition name, the additional token
  grammar is `Identifier Comma Identifier (Comma Identifier)* Arrow expression`.
  Whitespace may separate tokens; no logical newline, semicolon, indentation
  token or comment-induced logical break is skipped anywhere in this guard.
  A comma must always be followed by another ordinary identifier, never an arrow.
- The lookahead consumes nothing, clones no AST and allocates no parameter names.
  Once the complete sequence qualifies, the existing parameter parser consumes
  each identifier in order as `Parameter { name, type_name: None }`; the existing
  named-arrow expression path creates the same named `FunctionBody::Block`
  containing `Stmt::Return(Some(expression))`.
- Precedence, assignment-expression returns, objects, lists, templates and
  returned anonymous functions retain their existing expression grammar.
  Leading braces after the named arrow are object expressions, not statement
  blocks. The existing inline path skips no separator before the first body
  expression; an opening parenthesis on that logical line may continue over
  physical lines. NUC32 separately permits a real-indented single expression
  without changing this immediate parameter/header guard.
- Binding, missing/null/extra arguments, local shadowing, receivers, initializer
  adoption, `this`/`self`/`super`, live callable references, `.value` definition
  snapshots, redefinition, reactive dependencies, transactions, graph keys and
  canonical rendering are identical to the parenthesized counterpart.
  Returned anonymous functions do not gain local capture.
- No new parameter-count cap is introduced: the existing flat parameter parser
  has no per-parameter operator-chain bound. Expression nesting and operator/
  postfix-chain limits remain in force for the body.
- All parenthesized forms remain valid, including
  `def add(left, right) => left + right`, typed/mixed, trailing-comma and multiline
  lists, nullary definitions and block suites. NUC18/19/20/24's single-parameter
  arrows and indented blocks are unchanged. Calls, anonymous arrows, class
  parameter lists, keywords and ordinary-identifier references are unchanged.
- This arrow extension does not itself add block headers; NUC28's separate
  untyped and NUC30's typed/mixed multiple-parameter indented-block guards are
  specified below.
  NUC29 separately extends arrows to typed/mixed lists below; NUC27's own
  identifier-only grammar remains unchanged. Bare inline/braced blocks, defaults,
  destructuring, rest parameters, trailing commas, return annotations and
  call-parenthesis omission are not added.
- An incomplete or unsupported tokenizable prefix falls through without
  consuming any prefix token to the existing parenthesized parameter parser.
  For `def f left, right`, `def f left, right, => left`,
  `def f left, right: => left`, `def f left: int, right, => left`,
  `def f left, right:` without a real indented body, defaults/destructuring/rest
  or a logical break in the bare list, preserve
  `SyntaxError("Expected '(' but found identifier 'left'")` at line 1, column 7.
  `def f this, right => right` instead names `keyword 'this'` at that position.
  A newline immediately after `def f` still gives
  `SyntaxError("Expected '(' but found end of line")` at line 1, column 5;
  a semicolon there gives `SyntaxError("Expected '(' but found ';'")` at column 6.
- For the exact bytes `def f left, right =>`, preserve
  `SyntaxError("Unexpected end of input")` at line 1, column 21.
  A final newline gives `SyntaxError("Unexpected end of line")` at line 2,
  column 1. A body ` (left +)` gives `SyntaxError("Unexpected ')'")` at line 1,
  column 30; ` { return left }` gives
  `SyntaxError("Expected a key but found keyword 'return'")` at column 31;
  ` { left = 1 }` gives `SyntaxError("Expected ':' but found '='")` at column 29.
  An unparenthesized multiline object body still gives
  `SyntaxError("Expected a key but found an indented block")` at line 2, column 5.
- Lexical failures still precede parsing:
  `def f left, right => @` gives `SyntaxError("Unexpected character '@'")`
  at line 1, column 22. The malformed parenthesized
  `def f(left, right:) => left` keeps
  `SyntaxError("Expected a type but found ')'")` at line 1, column 21.
  Existing runtime errors, caught values and top-level statement positions
  are unchanged.
- The five `bare multiple parameter` entries below exercise forms, reactivity,
  methods, nested definitions and expression returns; the last explicitly
  returns `21`. They preserve their pre-executed accepted-syntax counterparts.

## Comma-free bare named arrows

- NUC37 adds an optional spelling of an already-bare, wholly untyped
  named arrow: `def add left right => left + right` is exactly
  `def add left, right => left + right`. Both remain exactly equivalent to
  `def add(left, right) => left + right`.
- This is an additional guard, not a replacement or relaxation of NUC27's
  comma-separated grammar or the existing typed/mixed and suite-colon guards.
  After unchanged `Def Identifier`, qualify the complete immediate prefix
  `Identifier Identifier Identifier* Arrow` on one logical header line.
- Require at least two ordinary identifiers, no annotation or comma anywhere
  in this new prefix, and the arrow immediately after the last identifier.
  Inspect tokens without consuming them, allocating parameter names, skipping
  `Newline`, `Semicolon`, `Indent` or `Dedent`, backtracking or error recovery.
  Whitespace and tabs separate names; comments cannot bridge a logical break.
- Only after complete qualification, consume the checked untyped identifiers.
  Keep ordered `Parameter { name, type_name: None }` values,
  the same function name and `FunctionBody::Block` containing exactly
  `Return(Some(expression))`. No token insertion or new lexer/public AST,
  runtime, generator, API, dependency or flat parameter-count bound is added.
- Reuse the entire named-arrow expression parser, including NUC32's genuine
  indented single expression, header-line opening parentheses continued over
  physical lines, leading object expressions and returned anonymous functions.
  All body bytes and punctuation, including object/anonymous/call commas, keep
  their existing meaning; this is not a new statement suite or implicit return.
- The same additional guard applies wherever named definitions already occur:
  methods, `init`, nesting and redefinition. Binding, missing/null/extra inputs,
  duplicate names, parameter locals and shadowing, `this`/`self`/`super`,
  initializer adoption, live references and `.value` snapshots are unchanged.
  Preserve noncapture, dependencies, graph keys and canonical source, reasoning,
  statement positions, resource bounds, transactions and rollback.
- Keep all comma-bearing, parenthesized, typed/mixed, single/nullary, block,
  method and initializer forms. Mixed separators such as `left right, third`
  or `left, right third`, comma-free typed lists,
  parenthesized comma omission, anonymous parameter comma omission, class
  lists, calls, defaults, rest/destructuring, return annotations and bare
  nullary headers are not added. NUC37 itself adds no colon suites; the separate
  NUC40 guard below covers only wholly untyped real-indented blocks.
  `add left right` outside `def` remains three
  ordinary expression statements, not a call.
- Unsupported complete prefixes fall through at their original position.
  Exact `def f left right, third => left`, `def f left, right third => left`,
  `def f left: int right => left`, `def f left right: int => left`,
  `def f left right: return left`, and logical
  header breaks all retain
  `SyntaxError("Expected '(' but found identifier 'left'")` at line 1, column 7.
  `def f this right => right` names `keyword 'this'` at that same position.
- Exact `def f left right =>` gives `SyntaxError("Unexpected end of input")`
  at line 1, column 20. A final newline or an unindented next expression gives
  `SyntaxError("Unexpected end of line")` at line 2, column 1.
  `def f left right =>;` gives `SyntaxError("Unexpected ';'")` at column 20.
- With that header, body ` (left +)` gives `SyntaxError("Unexpected ')'")`
  at line 1, column 29; ` { return left }` gives
  `SyntaxError("Expected a key but found keyword 'return'")` at column 30;
  ` { left = 1 }` gives `SyntaxError("Expected ':' but found '='")` at column 28;
  ` @` gives `SyntaxError("Unexpected character '@'")` at column 21.
  An unparenthesized multiline object still gives
  `SyntaxError("Expected a key but found an indented block")` at line 2, column 5.
- After a real newline, four-space-indented `left` followed by a second
  four-space-indented `right` gives
  `SyntaxError("Expected the end of a block but found identifier 'right'")`
  at line 3, column 5. Four-space `return left` gives
  `SyntaxError("Unexpected keyword 'return'")` at line 2, column 12.
  CRLF, tab columns, EOF, lexical failures and all other body/resource/runtime
  errors use the existing machinery, never a caught/retried parse.
- The five appended `comma-free` specification entries trace basic forms,
  live inputs, methods/init, nesting/returned anonymous functions and real
  caught redefinition rollback. Their 23 assertions execute in order; the
  rollback entry explicitly returns `21`. The paired reference, README and
  examples entries supply seven more protected assertions. All eight preserve
  the pre-executed accepted comma-bearing counterparts.

## Bare single-parameter indented block definitions

NUC 19 specifies `def double amount:` followed by an indented suite as exactly
`def double(amount):` followed by that same suite. Only one untyped ordinary
identifier after the required definition name qualifies for this single-parameter
guard. NUC28 separately adds the untyped multiple-parameter guard below.
Methods and `init` use the same function-statement parser and optional spellings.

The identifier must be immediately followed by `Colon`. The token after that
colon must be `Newline` or `Indent`, and the first token after any `Newline` or
`Semicolon` layout from that body start must be `Indent`. The guard only looks
ahead: it consumes no separators or body tokens. The existing colon-suite
parser consumes the body unchanged, including blank/comment lines and its
normal layout after the initial logical newline. No logical newline or
semicolon may separate `def`, the required name, the bare parameter, and colon.

All parenthesized signatures and indented, inline, or braced suites remain
valid, including typed, empty, multiple-parameter, trailing-comma, and multiline
parameter lists. NUC 18's bare named arrows remain unchanged. The single typed
indented spelling below is also optional; NUC30 separately adds typed/mixed
multiple-parameter indented blocks. Defaulted, destructured, nullary,
anonymous-colon, inline or braced bare block definitions are not added.
Return annotations, calls, ordinary-identifier
invocation, expression precedence, and keyword meanings are unchanged.

The result is the same `Parameter` with no type annotation and the same named
`FunctionBody::Block`. A block without `return` still returns `null`. Argument
binding and shadowing, receiver spellings, initializer adoption, explicit and
bare `super`, reactive dependencies, live references, `.value` snapshots, named
replacement, canonical graph/source rendering, transactions, and all existing
resource bounds retain their accepted behavior; there is no new local capture.

A header without a real indent, including `def f amount:` at end of input,
still raises `SyntaxError("Expected '(' but found identifier 'amount'")` at
line 1, column 7. The same error and position apply to `def f amount: int:`
at end of input without a body,
`def f amount: return amount`, `def f amount: { return amount }`,
`def f amount:;` even before a later indent, malformed multiple/default/return-annotation
prefixes, and a logical newline or semicolon between the parameter and colon.
`def f:` followed by an indented body still raises
`SyntaxError("Expected '(' but found ':'")` at line 1, column 6.
The unchanged malformed parenthesized form `def f(amount:):` raises
`SyntaxError("Expected a type but found ')'")` at line 1, column 15.
After a qualifying bare header, `return amount +)` on the next indented line
still raises `SyntaxError("Unexpected ')'")` at line 2, column 21; `return @`
raises `SyntaxError("Unexpected character '@'")` at line 2, column 12.

## Bare multiple-parameter indented block definitions

- NUC28 optionally spells `def add(left, right):` with an indented body as
  `def add left, right:` with exactly the same body, including methods and
  `init`. Require two or more untyped ordinary identifier parameters; three
  and larger flat lists use the same uncapped parameter grammar.
- The separate NUC40 guard below optionally omits every comma in
  these wholly untyped headers; the comma-bearing NUC28 guard stays unchanged.
- After `def` and the required ordinary definition name, look ahead for the
  complete `Identifier Comma Identifier (Comma Identifier)* Colon` sequence.
  Every comma needs another ordinary identifier; the header has no trailing
  comma, annotation or logical break. Skip no newline, semicolon, indentation
  token or comment-induced logical break within the header.
- The token immediately after the colon must be `Newline` or `Indent`.
  Looking past only `Newline` and `Semicolon` layout from that initial body
  break, the first non-layout token must be a real `Indent`. A colon immediately
  followed by a semicolon never qualifies, even if indentation appears later.
- The allocation-free, linear lookahead shares the complete-list scan with
  multiple-parameter arrows and NUC30's separate typed/mixed block extension.
  NUC28's own identifier-only lists remain untyped. It consumes no parameters,
  separators or body tokens. Once qualified,
  consume only the identifier/comma list as ordered
  `Parameter { name, type_name: None }` values, leaving its final colon for the
  existing `suite()` parser. Blank/comment lines and normal layout before a
  genuine indented body retain the existing suite grammar.
- Preserve the entire named `FunctionBody::Block`, metadata and canonical
  rendering of the parenthesized counterpart, not an implicit expression return.
  Multiple statements, branches, nested definitions and existing inner inline
  or braced statements remain valid inside the indented body. Explicit `return`,
  bare `return` and a body without `return` retain their existing behavior;
  the latter two yield `null`.
- Missing/null/extra arguments, local assignments and shadowing, receivers,
  initializer adoption, `this`/`self`/`super`, objects/lists/templates and returned
  noncapturing functions are unchanged. Live references, `.value` definition
  snapshots, redefinition, reactive inputs, graph keys/dependencies, errors,
  transactions and rollback remain identical to the parenthesized counterpart.
- Preserve NUC27 arrows and the exact NUC18/19/20/24 single-parameter guards.
  Every parenthesized typed/untyped, empty/multiple, trailing-comma/multiline,
  inline/braced/indented alternative remains available. Calls, anonymous
  functions, class syntax, keywords, lexer/AST/runtime/generator/public API,
  expression nesting and chain bounds are unchanged; add no parameter-count cap.
- NUC30 separately adds typed/mixed multiple-parameter indented block headers;
  it does not change NUC28's identifier-only forms. Bare inline/braced suites,
  trailing bare commas, defaults, destructuring, rest parameters, return
  annotations and call-parenthesis omission are not added.
- An incomplete or unsupported tokenizable prefix is left untouched for the
  existing parenthesized parameter parser. `def f left, right:` without a real
  indent, `def f left, right: return left`, `def f left, right: { return left }`,
  `def f left, right:;` even before a later indent,
  `def f left, right,:` with an indented body, malformed annotations and logical
  breaks within the header retain
  `SyntaxError("Expected '(' but found identifier 'left'")` at line 1, column 7.
- `def f this, right:` with an indented body instead names `keyword 'this'` at
  that position. A newline immediately after `def f` gives
  `SyntaxError("Expected '(' but found end of line")` at line 1, column 5;
  a semicolon there gives `SyntaxError("Expected '(' but found ';'")` at column 6.
  The parenthesized `def f(left, right:):` keeps
  `SyntaxError("Expected a type but found ')'")` at line 1, column 20.
- After `def f left, right:` and a newline, four-space-indented
  `return left +)` raises `SyntaxError("Unexpected ')'")` at line 2, column 19;
  `return { left = 1 }` raises `SyntaxError("Expected ':' but found '='")`
  at the same position. `return @` raises
  `SyntaxError("Unexpected character '@'")` at line 2, column 12.
  Lexical failures still precede parsing; runtime error messages, caught values
  and top-level statement positions retain their existing rules.
- The five `bare multiple parameter block` entries below cover two/three
  parameters, multiple statements/branches, reactivity, references/snapshots,
  methods/initializers, arity/shadowing/returns, nested definitions and rollback.
  The rollback entry executes its catch assertion and explicitly ends in `21`.

## Comma-free bare indented blocks

- NUC40 adds the optional `def add left right:` spelling of
  `def add left, right:` followed by exactly the same genuine indented
  statement suite. The parenthesized `def add(left, right):` alternative
  also remains supported and behavior-equivalent.
- After unchanged `Def Identifier`, non-consumingly qualify the complete
  immediate `Identifier Identifier Identifier* Colon` prefix. Require at
  least two wholly untyped ordinary identifiers and no parameter comma
  anywhere in this new prefix. Omit every comma or none; typed/mixed lists
  retain their accepted commas.
- Share the existing untyped identifier-run scan with the arrow guard,
  but keep that guard arrow-only. Do not allocate parameter names, consume
  tokens, skip logical header layout, backtrack or insert synthetic commas
  while qualifying the prefix. Horizontal spaces and tabs may separate names;
  comments never join a header across a logical newline.
- Require the final colon and exactly the existing real-indentation guard:
  immediately after it must be `Newline` or `Indent`; after only the already
  accepted `Newline`/`Semicolon` separators the first other token must be an
  actual `Indent`. Blank and comment lines retain their existing meaning.
  A same-line `:;` cannot qualify, even if a later line is indented.
- Once the whole header and layout qualify, consume only the checked names
  as ordered `Parameter { name, type_name: None }` values. Leave the final
  suite colon for unchanged `suite()`, never treating it as a type annotation.
  Preserve the same function name, complete named `FunctionBody::Block`,
  complete AST/metadata, canonical rendering and runtime path.
- Apply the additional guard wherever named block definitions already occur:
  module, nested bodies, methods, `init` and redefinition. Duplicate names
  keep their accepted binding order; flat parameter lists remain uncapped.
- Keep multi-statement and nested bodies, branches and existing inner inline
  or braced statements. All body bytes, list/object/template/anonymous
  punctuation and call commas are unchanged. Explicit `return`, bare `return`
  and no return keep their existing behavior; the latter two yield `null`.
- Preserve missing/null/extra inputs, unchecked arguments, parameter assignment,
  variable/callable shadowing, noncapture, caller scopes/frames, callbacks,
  live aliases, `.value` snapshots and redefinition. Receivers, `this`/`self`,
  explicit/bare `super`, adoption, graph source/keys/read dependencies/order,
  reasoning, errors/positions, transactions, rollback and recovery are unchanged.
  This adds no inherited-method lookup or other runtime capability.
- Preserve all accepted comma-bearing, typed/mixed, single/nullary,
  parenthesized/trailing-comma/multiline forms and NUC37 comma-free arrows.
  Do not add colon-free blocks, mixed comma omission, typed/mixed comma-free
  lists, parenthesized/anonymous/call/class comma omission, inline/braced bare
  suites, defaults/rest/destructuring, return annotations or bare nullary forms.
  Lexer, public AST/API, generator, runtime representation and existing
  expression/statement/postfix resource bounds do not change.
- Failed complete prefixes consume nothing and use the unchanged fallback
  diagnostics and positions. Exact `def f left right`, `def f left right:`
  without real indentation, `def f left right: return left`,
  `def f left right: { return left }`, `def f left right:;` before an indent,
  mixed separators/types and logical header breaks retain
  `SyntaxError("Expected '(' but found identifier 'left'")` at line 1, column 7.
- Exact `def f this right:` before an indented body instead names
  `keyword 'this'` at that position. `def f` followed immediately by a newline
  keeps `SyntaxError("Expected '(' but found end of line")` at line 1, column 5;
  a semicolon there keeps `SyntaxError("Expected '(' but found ';'")` at column 6.
- After `def f left right:` and a newline, four-space `return left +)`
  keeps `SyntaxError("Unexpected ')'")` at line 2, column 19;
  `return { left = 1 }` keeps `SyntaxError("Expected ':' but found '='")`
  at that position; `return @` keeps
  `SyntaxError("Unexpected character '@'")` at line 2, column 12.
  Body errors are not retried as another header. Native LF/CRLF/tab file
  bytes, literal EOF, stdin/REPL pending/blank-line commit and recovery use
  unchanged machinery.
- The REPL recognizes colon-suite headers with trailing comments and buffers
  their bodies until blank-line commit or EOF. Comma-free and comma-separated
  headers preserve matching results and graph output.
- The five appended `comma-free block` entries cover forms, explicit/bare/no
  returns, reactive aliases/snapshots/redefinition, explicit method/init
  controls, binding/arity/nesting/anonymous bodies and an actually executed
  rollback catch. The rollback entry explicitly returns `21`.
  Their ordered assertion counts are `8, 8, 10, 10, 7`; the paired reference,
  README and examples entries contribute `3, 2, 3`, respectively.

## Bare typed single-parameter named arrows

A named arrow definition or method may spell `def double(amount: int) => amount * 2`
as `def double amount: int => amount * 2`. After `def` and the required name, only
the immediate tokens `Identifier`, `Colon`, `Identifier`, `Arrow` qualify for
this additional spelling. The first identifier is the parameter name; the
second is the existing single-identifier annotation name. The existing parameter
parser preserves `Parameter { name, type_name: Some(annotation) }`. No annotation
variable is resolved and no new type constraint is enforced: an undeclared
`ArbitraryType` annotation, missing/null arguments, extra arguments, and string
arguments retain their parenthesized behavior.

The body is the same named `FunctionBody::Block` containing
`Stmt::Return(Some(expression))`, not an anonymous expression body. Precedence,
objects and templates, returned anonymous functions and their noncapturing
rules, argument binding and local shadowing, receiver/init/super handling,
reactive inputs, live callable references, `.value` definition snapshots,
redefinition rollback, canonical rendering, and resource bounds are unchanged.
Calls still use parentheses; ordinary identifiers are not automatically invoked.
Some inherited parameter-dependent local/nested noncapturing calls can fail on
later propagation with `ReferenceError("amount is not defined")`; this spelling
does not repair those failures or change the paired scope/rollback behavior.

This new guard skips no logical newline, semicolon, or comment-induced logical
break anywhere between the required name, parameter, colon, annotation, and
arrow. The existing inline body still skips no initial separator, and a
parenthesis starting on the header line may continue over physical lines.
NUC32 separately adds the real-indented single-expression body below without
changing this header guard. Parenthesized
typed and untyped signatures, multiline/trailing-comma/empty/multiple parameter
lists, class parameter lists and anonymous functions remain valid. NUC 18's
untyped bare arrows and NUC 19's exact untyped indented-colon guard are unchanged.
Representative typed parenthesized alternatives remain executable below.

Typed bare inline or braced suites, nullary/defaulted/destructured parameters,
dotted/union/generic/keyword annotation
forms, return annotations, and logical breaks in the bare header are not added.
NUC27's untyped and NUC29's typed/mixed multiple-parameter arrows have separate
complete-list guards; neither changes this single-parameter guard. The typed
indented-block spelling and NUC30's separate typed/mixed multiple block spelling
below do not change this arrow guard. Named-arrow
leading braces remain object expressions; a returned anonymous arrow retains its
own block grammar. No public token, AST, runtime, generator, API, or bound changes.

Unsupported tokenizable prefixes such as `def f amount: int:` without a body,
`def f amount: int, other, => amount`, `def f amount: Int.Name => amount`,
`def f amount: List[int] => amount`, `def f amount: this => amount`, and a logical
break before the typed arrow still raise
`SyntaxError("Expected '(' but found identifier 'amount'")` at line 1, column 7.
Lexical failures still precede parsing; `def f amount: int | str => amount`
raises `SyntaxError("Unexpected character '|'")` at line 1, column 19.
The unchanged parenthesized `def f(amount:) => amount` raises
`SyntaxError("Expected a type but found ')'")` at line 1, column 16.

For the exact bytes `def f amount: int =>`, the existing expression parser raises
`SyntaxError("Unexpected end of input")` at line 1, column 21. Adding a final
newline gives `SyntaxError("Unexpected end of line")` at line 2, column 1.
`def f amount: int => (amount +)` gives `SyntaxError("Unexpected ')'")` at line 1,
column 32. A body `{ return amount }` gives
`SyntaxError("Expected a key but found keyword 'return'")` at line 1, column 31;
`{ amount = 1 }` gives `SyntaxError("Expected ':' but found '='")` at that same
position. An unparenthesized multiline object body with indentation retains
`SyntaxError("Expected a key but found an indented block")` at line 2, column 5.
Undefined body reads keep their existing `ReferenceError` messages and top-level
statement positions; caught values and transaction recovery are unchanged.

## Bare typed multiple-parameter named arrows

- NUC29 optionally spells `def add(left: int, right: int) => left + right`
  as `def add left: int, right: int => left + right`, and mixed signatures such
  as `def scaled left: number, right, factor: number => (left + right) * factor`
  as their exact parenthesized counterparts, including methods and `init`.
- After `def` and the required ordinary definition name, recognize the complete
  immediate token sequence
  `Identifier (Colon Identifier)? (Comma Identifier (Colon Identifier)?)+ Arrow`.
  Require at least two ordinary identifier parameters. At least one annotation
  identifies this extension; entirely untyped multiple lists retain NUC27's
  behavior. Every annotation is exactly the existing `Colon Identifier`
  metadata syntax, not a keyword, dollar name, dotted, generic, nullable or
  union type grammar.
- Look ahead linearly without consuming tokens, cloning ASTs, allocating
  parameter names or skipping logical `Newline`, `Semicolon`, `Indent`,
  `Dedent` or comment-induced logical breaks. Every comma requires another
  ordinary parameter identifier; the final arrow must immediately follow the
  complete list. Unsupported prefixes are never partially consumed.
- After successful lookahead, reuse `parameter()` to preserve each name and
  `type_name: Some(annotation)` or `None` in exactly the original order.
  Reuse the named-arrow expression path and named `FunctionBody::Block`
  containing `Stmt::Return(Some(expression))`; the full AST and canonical
  rendering are identical to the parenthesized form.
- An annotation is not resolved as a variable, added as a dependency or checked
  at runtime. Missing/null/extra/string arguments, local assignments and
  shadowing retain their existing binding. A null callable parameter still
  yields the existing deferred `null`; a non-callable `operation` raises
  `TypeError("operation is not a function")`, even when an outer function
  has that name.
- Keep expression precedence, assignments, objects, lists, templates and
  returned anonymous functions unchanged. Leading named-arrow braces remain
  object expressions, not statement blocks; a returned anonymous function
  retains its own body grammar and noncapturing scope. The existing inline path
  skips no separator before the first body expression. A parenthesis beginning
  on that logical header line may continue over physical lines. NUC32 separately
  adds the real-indented single-expression body without changing this header
  guard.
- Receivers, `this`/`self`, initializer adoption, explicit/bare `super`, live
  callable aliases, `.value` definition snapshots, redefinition, reactive
  arguments and external inputs, graph keys/dependencies, reasoning,
  transactions, caught error values, rollback and top-level statement
  positions retain their parenthesized behavior.
- Preserve every parenthesized typed/untyped/mixed, empty/multiple,
  trailing-comma/multiline, arrow/indented/inline/braced alternative; for example,
  `def add(left: int, right: int) => left + right` remains valid. NUC18/20's
  single arrows, NUC19/24's exact single block guards and NUC27/28's untyped
  multiple arrows/indented blocks retain their behavior. In particular,
  `def f left, right:` with an indented `int => left` body remains an untyped
  block with no implicit return, not an annotated header.
- NUC29 does not itself add block headers. NUC30 separately extends the shared
  complete-list scanner to typed/mixed multiple indented blocks below; bare
  inline and braced block headers remain unsupported.
  Add no trailing bare commas, missing types/parameters, defaults, destructuring,
  rest, return annotations, multiline/broken bare headers, anonymous or class
  header shorthand, call-parenthesis omission, keyword or type grammar.
- Add no parameter-count cap: flat all-typed/mixed lists through 1,024 parameters
  retain the existing uncapped parameter grammar, not the 128-link expression
  bound. Excessive body nesting, operator chains and postfix chains retain
  `SyntaxError("Expressions are nested too deeply")`,
  `SyntaxError("Expression has too many operands")` and
  `SyntaxError("Expression has too many parts")` respectively. No lexer,
  public AST, runtime, generator, API, resource-bound or inherited-error change.
- Incomplete or unsupported tokenizable prefixes fall back untouched to the
  existing parenthesized parameter parser. `def f left: int, right`,
  `def f left: int, right, => left`, missing annotations such as
  `def f left: int, right: => left`, unsupported type/parameter forms, logical
  header breaks and bare inline/braced block headers retain
  `SyntaxError("Expected '(' but found identifier 'left'")` at line 1, column 7.
  `def f this: int, right => right` instead names `keyword 'this'` at that
  position. A newline immediately after `def f` gives
  `SyntaxError("Expected '(' but found end of line")` at line 1, column 5;
  a semicolon there gives `SyntaxError("Expected '(' but found ';'")` at column 6.
  Inside `class Gauge:` and a newline, four-space-indented
  `def reading left: int, right, => left` keeps the parameter-boundary error
  at line 2, column 17.
- For the exact bytes `def f left: int, right =>`, preserve
  `SyntaxError("Unexpected end of input")` at line 1, column 26.
  A final newline gives `SyntaxError("Unexpected end of line")` at line 2,
  column 1. A body ` (left +)` gives `SyntaxError("Unexpected ')'")` at line 1,
  column 35; ` { return left }` gives
  `SyntaxError("Expected a key but found keyword 'return'")` at column 36;
  ` { left = 1 }` gives `SyntaxError("Expected ':' but found '='")` at column 34.
  An unparenthesized multiline object body with indentation keeps
  `SyntaxError("Expected a key but found an indented block")` at line 2, column 5.
- Lexical failures precede parsing: `def f left: int, right => @` gives
  `SyntaxError("Unexpected character '@'")` at line 1, column 27;
  `def f left: int | str, right => left` gives
  `SyntaxError("Unexpected character '|'")` at column 17;
  `def f left: int?, right => left` gives
  `SyntaxError("Unexpected character '?'")` at column 16.
  The malformed parenthesized `def f(left: int, right:) => left` keeps
  `SyntaxError("Expected a type but found ')'")` at line 1, column 26.
- Undefined body/object reads retain `ReferenceError("missing is not defined")`.
  A returned anonymous function reading uncaptured `left` retains
  `ReferenceError("left is not defined")`; `this` or `self` without an instance
  retains `ReferenceError("this is not defined")`. Existing parameter-dependent
  local/nested noncapturing propagation failures are not repaired. Runtime
  errors retain their existing top-level statement positions; caught values
  omit positions as before.
- The five appended `bare typed multiple parameter` entries cover all-typed
  and mixed positions, reactivity, aliases/snapshots/redefinition, methods/init,
  arity/shadowing/null/non-callables/nesting and failed-redefinition rollback.
  The non-callable catch executes with `invoke(7, 3)`, not deferred `null`;
  the rollback entry executes its catch and explicitly ends in `21`.

## Bare typed single-parameter indented block definitions

- `def double amount: int:` followed by a real indented suite is exactly
  `def double(amount: int):` followed by that same suite, including methods and
  `init`. This optional spelling extends NUC19 and NUC20 without replacing them.
- After `def` and the required ordinary definition name, require immediate
  `Identifier(parameter)`, `Colon`, `Identifier(annotation)`, `Colon` tokens.
  The next token must be `Newline` or `Indent`; looking past only `Newline` and
  `Semicolon` tokens from that body start, the first other token must be `Indent`.
- This guard consumes nothing and skips no logical newline, semicolon, or
  comment-induced logical break within the header. The existing parameter
  parser consumes the typed prefix and the unchanged suite parser consumes the
  colon/body. Blank/comment lines and normal semicolon layout after the initial
  body newline remain valid only with a real indent.
- Preserve `Parameter { name, type_name: Some(annotation) }` and the complete
  named `FunctionBody::Block`. An ordinary annotation such as `ArbitraryType`
  remains metadata, not a variable lookup, dependency, or runtime type constraint.
- Retain NUC19's exact untyped bare-block guard and `type_name: None`; its suite
  colon is never consumed as an annotation. NUC18/20's immediate arrow guards
  and all parenthesized typed/untyped, empty/multiple, trailing-comma/multiline,
  inline/braced/indented, class and anonymous signatures remain unchanged.
- This single-parameter guard adds only one ordinary parameter name and one
  ordinary annotation name; NUC30 separately extends multiple indented blocks.
  Missing/dotted/union/generic/keyword/literal annotations,
  defaulted/destructured/variadic/nullary bare parameters, return annotations, anonymous
  colon functions, call-site omission and automatic invocation are not added.
- Typed bare inline/braced suites, header-line semicolons, missing indentation
  and comments-only bodies do not qualify. Nonqualifying tokenizable prefixes
  retain `SyntaxError("Expected '(' but found identifier 'amount'")` at line 1,
  column 7 for `def f amount: int:` without a real body. Lexical failures still
  precede parsing; `def f amount: int | str:` raises
  `SyntaxError("Unexpected character '|'")` at line 1, column 19.
- A logical newline after `def f` still gives
  `SyntaxError("Expected '(' but found end of line")` at line 1, column 5; a
  semicolon there gives `SyntaxError("Expected '(' but found ';'")` at column 6.
  The parenthesized `def f(amount:):` still gives
  `SyntaxError("Expected a type but found ')'")` at line 1, column 15.
- After a qualifying header, indented `return amount +)` still raises
  `SyntaxError("Unexpected ')'")` at line 2, column 21, and `return @` gives
  `SyntaxError("Unexpected character '@'")` at line 2, column 12.
- Explicit and bare returns, no implicit return, literal locals, shadowing,
  missing/null/extra/string argument binding, `this`/`self`, initializer adoption,
  explicit/bare `super`, live callables, definition snapshots with external
  dependencies, replacement, caught errors, rollback and recovery are unchanged.
- Inherited parameter-dependent local/nested noncapturing failures remain
  paired with the explicit syntax; no capture or propagation repair is added.
  Precedence, resource limits, lexer/public AST/API, runtime, generator, numeric
  conversion, built-ins, class/call/scope and dependency semantics are unchanged.
- The five corresponding entries at the end of the executable specification
  preserve reactive definitions, references/snapshots, methods/initializers,
  binding/explicit alternatives and failed-redefinition rollback.

## Bare typed multiple-parameter indented block definitions

- NUC30 optionally spells `def add(left: int, right: int):` with a real indented
  body as `def add left: int, right: int:` with exactly the same body, including
  named methods and `init`. Mixed lists such as `def add left: int, right:`
  preserve every parameter name, annotation and position in the list.
- After `def` and the required ordinary definition name, recognize the complete
  immediate sequence
  `Identifier (Colon Identifier)? (Comma Identifier (Colon Identifier)?)+ Colon`.
  Require at least two ordinary identifier parameters; at least one annotation
  describes NUC30's new forms. Entirely untyped lists remain NUC28 behavior.
  Each comma needs another parameter, with no trailing bare comma.
- Each annotation is only the existing `Colon Identifier` metadata syntax.
  Arbitrary undeclared ordinary names such as `ArbitraryType` are legal without
  lookup, graph dependencies or type enforcement. Add no keyword/dollar/dotted/
  generic/nullable/union annotation grammar, defaults, destructuring or rest.
- Extend the allocation-free, linear, non-consuming shared complete-list scan
  while preserving every NUC29 arrow. Skip no logical `Newline`, `Semicolon`,
  `Indent`, `Dedent` or comment-induced break anywhere within the header.
  Incomplete or unsupported lookahead consumes nothing and falls back normally.
- Immediately after the final suite colon require `Newline` or `Indent`.
  After only normal `Newline`/`Semicolon` layout from that body start, require
  a real `Indent`. A colon immediately followed by `Semicolon` never qualifies,
  even if indentation appears later. Blank/comment lines and the existing
  indented body's layout remain the unchanged `suite()` parser's responsibility.
- Once qualified, consume names, optional annotations and commas in order using
  the existing `parameter()` wherever applicable. The colon after a final
  untyped parameter belongs to `suite()`, not annotation parsing. A final typed
  parameter instead has its annotation `Colon Identifier` followed by a separate
  suite `Colon`. Leave the suite colon and all body/layout tokens for `suite()`.
  Do not relax `parameter()` or malformed parenthesized annotation diagnostics.
- Preserve the complete named `FunctionBody::Block`, ordered
  `Parameter { name, type_name: Some(annotation) | None }` values, AST and
  canonical rendering of the parenthesized definition. There is no implicit
  final-expression return; explicit, bare and absent returns keep their behavior,
  with bare/absent returns yielding `null`.
- Local assignments, multiple statements, branches, nested definitions and
  ordinary inner inline/braced statements retain the existing suite grammar.
  Missing/null/extra/string arguments, local shadowing, null callable deferral
  and non-callable errors are unchanged. A null `operation` stays deferred;
  a non-callable one still raises `TypeError("operation is not a function")`.
- Preserve noncapturing scopes, methods, `init`, `this`/`self`, explicit and bare
  `super`, initializer adoption and instance identity. Reactive arguments and
  body inputs, live aliases, `.value` definition snapshots, redefinition,
  graph IDs/dependencies, reasoning, transactions, caught values and rollback
  retain the parenthesized behavior. Inherited parameter-dependent local/nested
  propagation failures are not repaired by this syntax.
- Preserve NUC24's exact single typed block guard, every other accepted single
  guard, NUC28 untyped multiple blocks, NUC29 typed/mixed multiple arrows and all
  parenthesized alternatives, including nullary, multiline, trailing-comma,
  inline and braced definitions. Calls, anonymous/class grammar, lexer, public
  AST, runtime, generator and API are unchanged.
- Add no bare multiple inline/braced or missing-indent suites, return
  annotations or call syntax. Flat typed/mixed lists remain uncapped through
  1,024 parameters; body nesting/operator/postfix resource bounds are unchanged.
- `def f left: int, right:` at EOF, without a real indent, with only comments,
  with an inline/braced body or with immediate `:;` retains
  `SyntaxError("Expected '(' but found identifier 'left'")` at line 1, column 7.
  The same message/position covers trailing commas, missing names/types,
  unsupported annotations/parameters and logical header breaks beginning
  `def f left`. A final untyped parameter followed by the suite colon is valid,
  not a missing annotation.
- `def f this: int, right:` with an indented body instead names `keyword 'this'`
  at that position. A newline immediately after `def f` keeps
  `SyntaxError("Expected '(' but found end of line")` at line 1, column 5;
  a semicolon there keeps `SyntaxError("Expected '(' but found ';'")` at column 6.
  Four-space-indented `def reading left: int, right,:` after `class Gauge:`
  keeps the parameter-boundary error at line 2, column 17.
- Parenthesized `def f(left: int, right:):` keeps
  `SyntaxError("Expected a type but found ')'")` at line 1, column 25;
  `def f(a:):` keeps the same message at column 10.
  Lexical errors still precede parsing: `def f left: int | str, right:`
  gives `SyntaxError("Unexpected character '|'")` at line 1, column 17, and
  `def f left: int?, right:` gives `SyntaxError("Unexpected character '?'")`
  at column 16.
- After a qualifying typed/mixed header and newline, four-space-indented
  `return left +)` gives `SyntaxError("Unexpected ')'")` at line 2, column 19;
  `return { left = 1 }` gives `SyntaxError("Expected ':' but found '='")`
  at that same position. `return @` gives
  `SyntaxError("Unexpected character '@'")` at line 2, column 12.
- Undefined body/object reads keep `ReferenceError("missing is not defined")`;
  uncaptured `left` keeps `ReferenceError("left is not defined")`, and unbound
  `this`/`self` keeps `ReferenceError("this is not defined")`. Runtime errors keep
  their existing top-level positions; caught values omit positions as before.
  Excessive body nesting/operator/postfix chains retain
  `SyntaxError("Expressions are nested too deeply")`,
  `SyntaxError("Expression has too many operands")` and
  `SyntaxError("Expression has too many parts")` at their existing positions.
- The five appended `bare typed multiple parameter block` entries cover
  fully typed/mixed annotation positions, multiple statements/branches,
  aliases/snapshots/redefinition, methods/init/super/property mutation/identity,
  arity/unchecked arguments/shadowing/returns/nesting and failed-redefinition
  catch/rollback. The rollback entry executes its catch and explicitly ends
  with `reading(8, 9)`, returning `21`.

## Indented named arrow expressions

- NUC32 optionally follows an existing named definition's `=>` with a genuinely
  indented body containing exactly one expression. It is exactly the same named
  definition with `:` and the sole statement `return expression`.
- Reuse every existing parameter spelling: parenthesized nullary, single,
  multiple, annotated, trailing-comma and multiline lists; bare untyped/typed
  single parameters; bare untyped, mixed and fully typed multiple parameters.
  Keep parameter metadata and order, all complete header guards, and methods,
  nested definitions and `init`. `def f =>` and `def f:` remain invalid.
- After consuming only the existing arrow, qualify layout without consuming it:
  the immediate token must be `Newline` or `Indent`, and the first token after
  only `Newline`/`Semicolon` layout must be a real `Indent`.
  If either condition fails, use the unchanged inline expression path without
  skipping any token. Unsupported headers retain their original parameter path.
- On qualification, consume that newline/semicolon layout and the `Indent`,
  skip normal leading newline/semicolon body layout, call unchanged
  `expression()` exactly once, skip normal trailing newline/semicolon layout,
  and require and consume the closing `Dedent`.
- Produce exactly `FunctionBody::Block(vec![Stmt::Return(Some(expression))])`.
  This is neither an arbitrary statement suite nor an implicit return of a
  suite's last statement. Reject a second expression, statement or deeper
  unexpected indent before the closing `Dedent`; never silently discard input.
- An immediate semicolon after `=>` cannot qualify, even before a later indent.
  After the initial logical newline, semicolon-only lines may precede the real
  indent or the expression. Trailing semicolons are allowed, but `value; other`
  inside the body is two expressions and is rejected.
- Blank/comment lines do not create an indent. Existing CRLF, tab indentation,
  expression precedence and parenthesized/bracketed multiline layout are
  unchanged. An expression may start with `(` on the indented line and continue
  across physical lines. Leading named-arrow braces remain object expressions.
- Existing valid inline arrows and following statements are never reinterpreted.
  Colon suites still permit multiple statements, absent/bare/explicit returns
  and their existing inline/braced alternatives. Anonymous expression/block
  `FunctionBody` variants retain their grammar; no anonymous indented-arrow
  path, closure capture, token, keyword, public AST, runtime or API is added.
- Binding, arity, unchecked annotations, null callable deferral, non-callable
  errors, assignment returns, receivers, `this`/`self`, explicit/bare `super`,
  initializer adoption/identity, live references, `.value` definition snapshots,
  redefinition, graph dependencies/reasoning, state propagation and transactions/
  rollback are unchanged. Existing statement/expression nesting, operator and
  postfix bounds apply, with no flat-parameter cap added.
- For exact `def f x =>`, retain `SyntaxError("Unexpected end of input")` at
  line 1, column 11. A final newline, or exact `def f x =>\nx`, retains
  `SyntaxError("Unexpected end of line")` at line 2, column 1.
  Exact `def f x =>;\n    x` retains `SyntaxError("Unexpected ';'")` at
  line 1, column 11. No real-indent failure consumes layout speculatively.
- After exact `def f x =>` and a newline, four-space-indented `x` followed by
  four-space-indented `x + 1` raises
  `SyntaxError("Expected the end of a block but found identifier 'x'")` at
  line 3, column 5. The sole line `x; x + 1` raises the same message at line 2,
  column 8; `x 1` raises
  `SyntaxError("Expected the end of a block but found number 1")` at line 2,
  column 7. Unexpected further indentation reports
  `SyntaxError("Expected the end of a block but found an indented block")`
  at that physical indentation token.
- With that same header and four-space body indentation, `return x` retains
  the expression error `SyntaxError("Unexpected keyword 'return'")` at line 2,
  column 12; `(x +)` gives `SyntaxError("Unexpected ')'")` at line 2, column 10;
  `{ return x }` gives
  `SyntaxError("Expected a key but found keyword 'return'")` at line 2, column 14;
  `@` gives `SyntaxError("Unexpected character '@'")` at line 2, column 5.
  Other inherited lexical/expression/resource/runtime errors keep their exact
  messages and physical/top-level positions; no body error is caught or retried.
- Exact `def f =>\n    1` retains
  `SyntaxError("Expected '(' but found '=>'")` at line 1, column 7;
  `def f:\n    1` retains `SyntaxError("Expected '(' but found ':'")` at
  line 1, column 6. Malformed complete parameter lists remain untouched.
- The native REPL additionally buffers a lexical named header whose only arrow
  is the final significant token, including a comment after `=>`. The parser
  validates the unchanged parameters and real indent on blank-line commit or
  stdin EOF. Inline arrows, direct expressions, anonymous arrows and arrow text
  in strings, regexes, templates or comments do not select this continuation.
  Native source-file execution uses the same parser. Structured failures clear
  the input buffer and permit later input without changing existing state.
- The five `indented named arrow` entries below trace forms, binding/returned
  values, methods/init, reactivity/aliases/snapshots and caught redefinition
  rollback. Their 36 assertions execute, including the rollback catch, and their
  explicit results are `20` and `21`. The paired reference, README and examples
  entries supply the other five assertions, without adding unpaired cases.

## Reactive numeric clamping

- `Math.clamp(value, lower, upper)` is a static runtime operation, exactly the
  ordered numeric composition `Math.min(Math.max(value, lower), upper)` using
  the existing numeric coercion and both existing extremum folds.
- Numbers retain their values; literal `null` coerces to zero, booleans to zero
  or one, trimmed numeric strings and empty/singleton lists use their existing
  conversion, and dates use their millisecond value. Other nonnumeric values
  coerce to numeric `NaN`, not a new type error.
- Numeric `NaN` in any of the first three positions propagates. Reversed numeric
  bounds yield the upper bound; equal bounds, infinities and signed zeros
  follow that same ordered composition. Bounds are not validated or reordered,
  and these are defined results rather than swallowed errors.
- Each omitted position among the first three supplies numeric `NaN`, as in
  existing fixed-arity Math operations. Extra values are numerically ignored.
- The existing call path evaluates every supplied argument expression from
  left to right before builtin dispatch, including extras, and tracks its reads.
  The unchanged Math gate returns `Undefined` if any supplied value is
  `Undefined`, including an extra; a dependent assignment stores `null` and
  recovers when its missing property is defined. Omission is not such a read.
- Existing null/deferred-read assignment handling takes precedence over the
  numeric result, just as for `Math.min` and `Math.max`; a live input containing
  `null` is not the same assignment context as a literal `null` argument.
- Strict unknown identifiers still raise `ReferenceError("<name> is not defined")`
  during argument evaluation, even in extras or after an undefined property.
  Failures retain top-level statement positions, caught values, transaction
  rollback, graph dependencies and subsequent reactive recovery.
- State variables and local bindings named `Math` retain their namespace shadowing.
  Ordinary `.Math` properties and methods keep their existing lookup behavior.
  `Math.clamp` read as a value is unsupported and returns `Undefined`, like
  other Math method references; this adds no first-class callable or alias.
- Other Math functions and constants are unchanged. Unknown static methods
  retain `TypeError("Math.<name> is not a function")`; `Math.random()` retains
  `TypeError("Math.random is not supported")`, after the same undefined gate.
- This capability adds no grammar, token, parser/generator rule, public Rust
  value/AST/API representation, dependency, type constraint or error kind.
  All accepted signatures and runtime programs remain available; no new
  closure capture or change to inherited resource/scope limitations is implied.

## Reactive numeric interpolation

- `Math.lerp(start, end, weight)` is one static numeric runtime operation.
  Coerce the first three supplied values with the existing `Value::to_number`,
  then perform exactly these ordinary `f64` operations in order:
  `delta = end - start`, `scaled = delta * weight`, `result = start + scaled`.
- The result is numeric even for string inputs: conversion is intrinsic, not
  a call to a shadowable `Number()` namespace or string-concatenating `+`.
  Numbers retain their values, literal `null` converts to zero, booleans to
  zero/one, trimmed strings and empty/singleton lists use their existing numeric
  conversion, and dates use milliseconds. Other nonnumeric values yield numeric
  `NaN`, not a new type error.
- Weights below zero or above one extrapolate. Do not clamp weights, validate
  or reorder endpoints, use a weighted alternative formula, fuse operations,
  raise precision, or special-case a midpoint, equal endpoints or weights zero/one.
  Equal/reversed endpoints, overflow, cancellation, infinities, numeric `NaN`
  in any position and signed zeros follow the same ordered ordinary arithmetic.
- In particular, `0 + Infinity * 0` and `Infinity - Infinity` produce numeric
  `NaN`, not an endpoint shortcut; finite endpoint subtraction can overflow
  before scaling. Rounding is that of each individual accepted operation.
- Each omitted first/second/third argument supplies numeric `NaN`.
  Extra values are ignored numerically, not omitted from evaluation.
- The unchanged call path evaluates every supplied argument expression exactly
  once from left to right before dispatch and tracks every read, including extras.
  The existing Math gate returns `Undefined` for any supplied `Undefined` value,
  including an extra; a dependent assignment stores `null` and recovers when
  its missing property is defined. An omitted numeric input is not such a read.
- Literal-null numeric conversion does not override existing live-null or
  deferred-read assignment handling. Class rules, ordinary property reads,
  deletion/recovery, named wrappers, dependencies and transactions are unchanged.
- Strict unknown identifiers still raise `ReferenceError("<name> is not defined")`
  during argument evaluation, including extras or after an undefined property.
  Failures keep their existing positions, caught values, rollback and recovery.
- State and local bindings named `Math` still shadow the static namespace.
  Ordinary `.Math` members retain their lookup behavior. `Math.lerp` read as
  a value remains unsupported/`Undefined`, like other Math method references;
  no first-class callable value or alias is introduced.
- `Math.clamp`, `min`, `max`, `round`, all other Math functions/constants and
  their errors are unchanged. Unknown methods keep
  `TypeError("Math.<name> is not a function")`; `Math.random()` keeps
  `TypeError("Math.random is not supported")`, after the same undefined gate.
- No lexer, parser, generator, AST, public Global/Value/API, coercion, operator,
  call/member resolution, class, scope, dependency or resource-limit change is
  introduced. Existing signatures and inherited noncapturing scope limitations
  remain; no new closure capture or extreme Windows debug stack claim is implied.

## Reactive unit-interval saturation

- `Math.saturate(value)` is a static numeric runtime operation, exactly the
  accepted ordered `Math.clamp(value, 0, 1)`, or
  `Math.min(Math.max(value, 0), 1)` through both existing extremum folds.
- After the unchanged any-supplied-`Undefined` gate, convert the first supplied
  value intrinsically with `Value::to_number`; omission supplies numeric `NaN`.
  Do not resolve or depend on a shadowable `Number` or `Math.clamp` name.
- Numbers retain their values, literal `null` converts to zero, booleans to
  zero/one, trimmed strings and empty/singleton lists keep their existing numeric
  conversion, and dates contribute milliseconds. Other nonnumeric conversions
  yield numeric `NaN`, not a new type error.
- Apply the existing maximum fold to the converted number and numeric zero,
  then the existing minimum fold to that result and numeric one, in that order.
  Numeric `NaN`, infinities, signed zeros, subnormal values and boundary values
  retain exactly that ordered composition's results and bits.
- Do not use host `f64::clamp`, validate inputs, shortcut endpoints, skip
  conversion, reorder operations or change either accepted extremum fold.
  The existing `Math.clamp(value, lower, upper)` form remains fully available.
- Extra values are ignored numerically, but every supplied expression runs
  exactly once from left to right through the unchanged call path, retaining
  its effects and read dependencies.
- Any supplied `Undefined`, including an extra, returns `Undefined` before
  numeric dispatch; omission is not a supplied undefined read. Dependent
  assignments store `null` and recover when a missing property becomes defined.
- Literal-null conversion does not override existing live-null or deferred
  assignment handling. Class rules, named wrappers, property deletion/recovery,
  dependency edges and transactions keep their accepted behavior.
- Strict unknown identifiers still raise `ReferenceError("<name> is not defined")`
  during argument evaluation, including extras or after an undefined property.
  Keep the existing positions, caught values, rollback and subsequent recovery.
- State and local bindings named `Math` retain namespace shadowing; ordinary
  `.Math` properties and methods retain member lookup. `Math.saturate` read as
  a value stays unsupported/`Undefined`, like other Math method references;
  no callable alias or first-class method value is added.
- `Math.clamp`, `lerp`, `min`, `max`, `round`, all other Math members/constants
  and their errors are unchanged. Unknown static methods retain
  `TypeError("Math.<name> is not a function")`; `Math.random()` retains
  `TypeError("Math.random is not supported")`, after the same undefined gate.
- This adds no grammar, lexer, parser, generator, public Global/Value/AST/API,
  coercion, operator, call/member, class, scope, dependency or resource-limit
  change. All accepted syntax and inherited noncapturing limitations remain.

## Reactive cubic smoothstep

- `Math.smoothstep(lower, upper, value)` is a static numeric runtime operation.
  Its argument order is lower bound, upper bound, value, unlike the value-first
  `Math.clamp(value, lower, upper)`.
- After the unchanged any-supplied-`Undefined` gate, convert the first three
  supplied values intrinsically with `Value::to_number` in lower, upper, value
  order. The existing Math dispatcher already converts the first value.
  Each omitted input supplies numeric `NaN`, not a supplied `Undefined`.
- Perform exactly these separate ordinary `f64` operations in order:
  `numerator = value - lower`, `denominator = upper - lower`,
  `normalize = numerator / denominator`,
  `t = numeric_clamp(normalize, 0.0, 1.0)`, `square = t * t`,
  `scaled = 2.0 * t`, `falloff = 3.0 - scaled`, `result = square * falloff`.
- The existing private numeric clamp applies the accepted maximum fold to
  `[normalize, 0]`, then the accepted minimum fold to `[bounded, 1]`.
  This is the independent accepted normalization
  `Math.saturate((value - lower) / (upper - lower))` followed by the ordered
  cubic `t * t * (3 - 2 * t)` after intrinsic numeric conversion.
- Do not use `f64::clamp`, `powf`, `mul_add`, a reordered polynomial, an endpoint
  shortcut, a post-polynomial clamp, range validation or a conversion skip.
  Equal/reversed bounds, overflow, infinities, numeric `NaN`, subnormals and
  signed zeros follow that exact composition, not an undefined-range convention.
- Equal bounds with the same value produce numeric `NaN`; values below/above
  equal finite bounds normalize to negative/positive infinity before clamping.
  Reversed finite bounds are normalized in their supplied order.
  Infinite subtraction or overflow can produce numeric `NaN` even at an endpoint.
- Compare expected numeric `NaN` by classification; other numeric results,
  including zeros, retain the host accepted composition's bits.
  No portable sign promise replaces the existing extrema zero-tie behavior.
- Numbers retain their values, literal `null` converts to zero, booleans to
  zero/one, trimmed strings and empty/singleton lists keep existing numeric
  conversion, and dates contribute milliseconds. Nonnumeric values yield
  numeric `NaN`, not a new type error.
- Every supplied expression runs exactly once left to right through the
  unchanged call path, including unused extras. Keep all reads, effects and
  dependencies. Extras are neither numerically converted nor used.
- Any supplied `Undefined`, including an extra, returns `Undefined` before
  conversion. Dependent assignments retain existing live-null/deferred handling,
  storing `null` and recovering when missing properties are defined.
  Literal-null conversion does not override live assignment or parameter context.
- Strict unknown identifiers still raise `ReferenceError("<name> is not defined")`
  during argument evaluation, even in extras or after an undefined property.
  Positions, caught values, transactional rollback and later recovery are unchanged.
- State/local `Math` bindings shadow the static namespace as before.
  Ordinary `.Math` members and methods keep their lookup.
  Reading `Math.smoothstep` as a value remains unsupported/`Undefined`, like
  other Math method reads; no first-class method alias is introduced.
- The intrinsic neither invokes nor tracks shadowable `Number` or
  `Math.saturate`; it does not recursively call the public Math dispatcher.
  Named wrappers, methods, initializers, class rules, deletion and reactivity
  retain the existing call, scope and graph behavior.
- All other Math functions, constants and errors remain unchanged.
  Unknown static methods retain `TypeError("Math.<name> is not a function")`;
  `Math.random()` retains `TypeError("Math.random is not supported")`, after
  the existing supplied-undefined gate.
- No type, public Global/Value/AST/API, dependency, grammar, lexer, parser,
  generator, CLI, scope, graph, evaluation or resource-limit change is added.
  Every accepted syntax form and its behavior remains available.
  Existing noncapturing and parameter-dependent-local failures are not repaired,
  and no broader Windows debug stack-safety claim is made.
- The five entries titled `Nucleoid keeps cubic smoothstep reactive to all three
  inputs`, `Nucleoid preserves numeric coercion while smoothing values`,
  `Nucleoid preserves special values and omitted smoothstep inputs`,
  `Nucleoid applies cubic smoothstep to live class rules`, and `Nucleoid preserves
  deferred reads and rollback for cubic smoothstep` pair this contract with
  independent accepted unit-normalization controls and unchanged assertions.

## Reactive quintic smootherstep

- Accepted [NUC 35](docs/nuc-0035.md) adds only static
  `Math.smootherstep(lower, upper, value)`, the quintic counterpart to cubic
  `Math.smoothstep`. Bounds precede the value, unlike `Math.clamp`.
- After the unchanged any-supplied-`Undefined` gate, convert lower, upper and
  value intrinsically with `Value::to_number`, in that order. The dispatcher
  already converts the first input. Each omission supplies numeric `NaN`,
  not a supplied `Undefined`.
- Perform exactly these separate ordinary `f64` operations in order:
  `numerator = value - lower`, `denominator = upper - lower`,
  `normalize = numerator / denominator`,
  `t = numeric_clamp(normalize, 0.0, 1.0)`, `square = t * t`,
  `cube = square * t`, `scaled = 6.0 * t`, `offset = scaled - 15.0`,
  `weighted = t * offset`, `polynomial = weighted + 10.0`,
  `result = cube * polynomial`.
- The unchanged private numeric clamp applies the accepted maximum fold to
  `[normalize, 0]`, then the accepted minimum fold to `[bounded, 1]`.
  Independent accepted controls use
  `unit = Math.saturate((value - lower) / (upper - lower))` and
  `unit * unit * unit * (unit * (6 * unit - 15) + 10)`.
- Do not expand/reassociate the polynomial as `6t^5 - 15t^4 + 10t^3`, use
  `powf`, `mul_add`, host `f64::clamp`, endpoint shortcuts, bound validation
  or reordering, conversion skips, or a post-polynomial clamp.
- Equal/reversed bounds, infinite subtraction, overflow, NaNs, subnormals and
  signed zeros follow this exact composition. Equal finite bounds with the
  same value yield numeric `NaN`; lower/higher values normalize to
  negative/positive infinity before clamping. Overflow can produce NaN at an
  endpoint. Reversed bounds are used in their supplied order.
- Expected NaNs compare by classification; every non-NaN result, including
  zero, retains the independent accepted host composition's bits.
  No portable NaN-payload or zero-sign convention is introduced.
- The rounded polynomial can overshoot the unit interval, even though `t` is
  clamped. Do not promise that every numeric result lies in `[0, 1]`.
  The mathematical quintic has a smoother endpoint transition than the cubic;
  this is not a claim of exact floating-point derivatives.
- Numbers retain their values, literal null converts to zero, booleans to
  zero/one, trimmed strings and empty/singleton lists retain existing numeric
  conversion, and dates contribute milliseconds. Other nonnumeric conversions
  yield numeric `NaN`, not a new type error.
- Every supplied expression runs once, left to right, including unused extras.
  Extras are not converted numerically, but retain reads, effects, dependencies,
  strict errors and the any-supplied-`Undefined` gate.
- Any supplied `Undefined`, including an extra, returns `Undefined` before
  conversion. Existing live-null/deferred assignments and parameters still
  defer and recover; intrinsic literal-null conversion does not override them.
- Strict unknown identifiers retain `ReferenceError("<name> is not defined")`,
  including in extras or after an undefined property. Preserve positions,
  caught values, transactional rollback and later recovery.
- State/local `Math` bindings keep shadowing the namespace; ordinary `.Math`
  members retain lookup. Reading `Math.smootherstep` as a value stays
  unsupported/`Undefined`, not a first-class callable reference.
- Intrinsic conversion neither invokes nor tracks shadowable `Number` or
  `Math.saturate`, and does not recursively invoke public Math dispatch.
  Wrappers, call frames, receivers, per-instance class rules, property deletion
  and deferred reads use unchanged call, scope, graph and transaction machinery.
- Cubic `Math.smoothstep`, `clamp`, `saturate`, `lerp`, all other Math members
  and constants retain their behavior. Unknown methods still raise
  `TypeError("Math.<name> is not a function")`; `Math.random()` retains
  `TypeError("Math.random is not supported")`, after the existing undefined gate.
- No grammar, parser, lexer, generator, CLI, public representation/API,
  dependency, version, MSRV or resource-limit change is added. All accepted
  syntax and inherited noncapture/parameter-local limitations remain unchanged;
  this makes no broader Windows stack-safety claim.
- The five entries titled `Nucleoid keeps quintic smootherstep reactive to all
  three inputs`, `Nucleoid preserves numeric coercion while applying quintic
  smootherstep`, `Nucleoid preserves special values and omitted smootherstep
  inputs`, `Nucleoid applies quintic smootherstep to live class rules`, and
  `Nucleoid preserves deferred reads and rollback for quintic smootherstep`
  trace the contract with independent unit controls and protected assertions.
- A direct intrinsic reads lower/upper/value, while its emulation reads the
  unit-control node. Immediate graphs and whole RHS ASTs intentionally differ;
  asserted controls, values and transitive behavior must agree.

## Reactive fractional part

- Accepted [NUC 36](docs/nuc-0036.md) adds only the unshadowed static
  signature `Math.fract(value)`, the intrinsic numeric composition of
  `value - Math.floor(value)`, not a truncation-based fractional part.
- Reuse Math's existing any-supplied-`Undefined` gate, including extras, before
  its existing first-input intrinsic `Value::to_number` conversion.
  An omitted first argument is numeric `NaN`, not a supplied `Undefined`.
- Compute `floor = first.floor()`, then `result = first - floor`, as separate
  ordinary `f64` operations in exactly that order. Do not use host
  `f64::fract`, truncation, remainder, absolute value, sign correction,
  conversion skips, normalization or result clamping.
- Negative finite fractions subtract floor: `-1.25` yields `0.75`.
  Both infinities yield numeric `NaN` by subtraction; numeric NaNs remain NaN.
  Integers, subnormals and signed zeros follow the exact accepted composition.
- A tiny negative input such as `-0.00000000000000000001` can round to
  exactly `1`; there is no strict `[0, 1)` result guarantee or endpoint clamp.
  Compare expected NaNs by classification and every other numeric result,
  including zeros, by bits against independent accepted arithmetic/floor
  controls. No portable NaN-payload or zero-sign convention is introduced.
- Numbers retain their values, literal null converts to zero, booleans to
  zero/one, trimmed strings and empty/singleton lists retain existing numeric
  conversion, and dates contribute milliseconds. Other nonnumeric conversions
  produce numeric `NaN`, not a new type error.
- Every supplied expression runs once, left to right, including ignored extras.
  Extra values are not numerically converted, but their reads, effects,
  dependencies, strict errors and the supplied-`Undefined` gate remain.
- Supplied `Undefined` returns `Undefined` before conversion. Live assignment
  and parameter-null deferral still take precedence over intrinsic literal-null
  conversion; missing properties, deletion and later recovery remain reactive.
- Strict unknown identifiers retain `ReferenceError("<name> is not defined")`,
  including extras and expressions after an undefined property. Preserve
  statement positions, caught values, graph transactions, rollback and recovery.
- Do not resolve, invoke or track shadowable `Number` or `Math.floor` bindings.
  Use intrinsic conversion and host floor, not public method dispatch.
- State/local `Math` bindings keep shadowing the namespace. Ordinary `.Math`
  members keep lookup; reading `Math.fract` as a value remains unsupported/
  `Undefined`, not a callable reference. Wrappers, call frames, receivers,
  per-instance class rules and scopes use unchanged machinery.
- All existing Math arms, numeric helpers, constants and controls stay
  unchanged. Unknown methods retain `TypeError("Math.<name> is not a function")`;
  `Math.random()` retains `TypeError("Math.random is not supported")`, after
  the same undefined gate. Numeric `Math = 7` retains the ordinary
  `TypeError("number.fract is not a function")` when called through that value.
- No parser, lexer, AST, generator, CLI implementation, public API/
  representation, dependency, version, MSRV or resource-limit change is added.
  Accepted syntax and inherited noncapture/parameter-local limitations remain.
- The five entries titled `Nucleoid keeps floor fractional parts reactive`,
  `Nucleoid preserves numeric coercion while computing floor fractional parts`,
  `Nucleoid preserves nonfinite fractional parts omissions and rounded
  endpoints`, `Nucleoid applies floor fractional parts to live class rules`,
  and `Nucleoid preserves deferred fractional parts and rollback` trace this
  contract with independent `whole = Math.floor(...)` controls.
- New intrinsic RHS ASTs intentionally differ from arithmetic/floor controls;
  values, independent controls and transitive behavior must agree, without
  requiring identical whole-new-program ASTs or immediate graph shapes.

## Reactive inverse interpolation

- Accepted [NUC 38](docs/nuc-0038.md) adds only the unshadowed static
  signature `Math.inverseLerp(start, end, value)`, an unclamped numeric
  normalization. The existing `(value - start) / (end - start)` remains valid.
- Reuse Math's unchanged any-supplied-`Undefined` gate, including extras,
  before its existing first-input intrinsic conversion. That first value is
  `start`; intrinsically convert positions one and two to `end` and `value`
  with the unchanged `Value::to_number`. Each omission supplies numeric NaN,
  not a supplied Undefined or a new arity error.
- Compute exactly these separate ordinary `f64` operations in this order:
  `numerator = value - start`, `denominator = end - start`,
  `result = numerator / denominator`.
- Do not clamp, smooth, validate or reorder ranges, factor the expression,
  fuse operations, scale to avoid overflow, or shortcut endpoints or zero width.
  Descending ranges are valid; results outside `[0, 1]` are intentional.
- Equal finite endpoints give ordinary zero-over-zero NaN or nonzero-over-zero
  signed infinity. Infinities, NaNs, span overflow, cancellation, subnormals,
  underflow and signed zeros retain the exact ordered arithmetic composition.
  In particular, `start = -Number.MAX_VALUE`, `end = Number.MAX_VALUE`,
  `value = 0` gives zero, not a scaled midpoint of `0.5`.
- Compare expected NaNs by classification, not payload. Every non-NaN
  result, including zero, matches independent accepted binary subtraction
  and division controls by bits, without a portable zero-sign promise.
- Numbers retain their values, literal null converts to zero, booleans to
  zero/one, trimmed strings and empty/singleton lists keep existing numeric
  conversion, and dates contribute milliseconds. Other nonnumeric values
  convert to numeric NaN, not a new type error.
- Every supplied expression runs exactly once left to right, including
  ignored extras and expressions after an Undefined. Extra resulting values
  are neither numerically converted nor used; keep all reads, effects,
  dependencies, strict failures, deferral and rollback.
- Any supplied Undefined returns Undefined before conversion; dependent
  assignments retain inherited live-null/deferred handling and recovery.
  Intrinsic literal-null conversion does not override live assignment or
  parameter-null deferral. Missing properties and deletion remain reactive.
- Intrinsic conversion never resolves, invokes or tracks shadowable `Number`
  or other Math methods, and never recursively calls public Math dispatch.
- State/local `Math` bindings retain namespace shadowing; ordinary `.Math`
  properties and methods retain lookup. Reading `Math.inverseLerp` as a
  value is unsupported/Undefined, not a first-class callable reference.
  Wrappers, frames, receivers, aliases and per-instance class rules are unchanged.
- Strict arguments retain `ReferenceError("<name> is not defined")`, including
  extras and expressions after an undefined property. Unknown static methods
  retain `TypeError("Math.<name> is not a function")`; `Math.random()` retains
  `TypeError("Math.random is not supported")`, after the existing Undefined gate.
  Calling through numeric `Math = 7` retains ordinary member dispatch:
  `TypeError("number.inverseLerp is not a function")`.
- Error positions, caught values, graph subscriptions, transactions and
  recovery remain unchanged. User throws, including `"ROLLBACK"`, still roll
  back. No numeric, coercion, range-validation or arity errors are introduced.
- All existing Math arms, helpers, constants, conversions and errors remain
  unchanged, including `lerp`, `clamp`, `saturate`, `smoothstep`, `smootherstep`
  and `fract`. All accepted syntax, including NUC37 forms, remains available.
  No parser, lexer, AST, generator, CLI, API/representation, namespace,
  dependency, version, MSRV, edition or resource-bound change is introduced.
- The five entries titled `Nucleoid keeps inverse interpolation reactive`,
  `Nucleoid preserves intrinsic conversion during inverse interpolation`,
  `Nucleoid preserves inverse interpolation endpoints nonfinite inputs and
  omissions`, `Nucleoid applies inverse interpolation to live class rules`,
  and `Nucleoid preserves deferred inverse interpolation and rollback` trace
  this contract, retaining independent Math.lerp reconstruction controls.
- New intrinsic RHS ASTs and immediate dependency graphs may intentionally
  differ from emulation. Protect non-RHS source/ASTs, canonical assertions,
  independent controls, observable values, transitive reads/effects/errors,
  explicit returns and actual assertion coverage, not whole-new-program AST
  or immediate-graph identity. Historical programs and their ASTs are unchanged.
- Inherited noncapture, parameter-dependent-local, Windows debug stack and
  old whole-suite Gitless discovery limitations are not repaired or broadened.

## Reactive range remapping

- Accepted [NUC 39](docs/nuc-0039.md) adds only the unshadowed static
  signature `Math.remap(start, end, targetStart, targetEnd, value)`.
  The accepted numeric composition
  `Math.lerp(targetStart, targetEnd, Math.inverseLerp(start, end, value))`
  remains valid; remapping is unclamped and both ranges may descend.
- Retain Math's existing any-supplied-Undefined gate, including extras,
  before its unchanged first-input intrinsic conversion for start.
  Intrinsically convert positions one through four to end, targetStart,
  targetEnd and value, in that order, with `Value::to_number`.
  Each missing input supplies numeric NaN, not a new arity error.
- Compute six separate ordinary `f64` operations in exactly this order:
  `numerator = value - start`, `denominator = end - start`,
  `weight = numerator / denominator`, `delta = targetEnd - targetStart`,
  `scaled = delta * weight`, `result = targetStart + scaled`.
- Do not clamp, smooth, validate, reorder ranges, factor, fuse, use higher
  precision, rescale, shortcut endpoints, equal outputs or zero source width.
  Extrapolation, NaNs, infinities, signed zeros, overflow, underflow,
  subnormals and cancellation retain the exact ordered accepted composition.
- Source `[-Number.MAX_VALUE, Number.MAX_VALUE]` with sample zero normalizes
  to zero, not `0.5`. Equal target endpoints `4, 4` with infinite weight give
  NaN, not four. Target `[-Number.MAX_VALUE, Number.MAX_VALUE]` at weight
  `0.25` can overflow to positive infinity, not a rescaled finite result.
- Compare NaNs by classification, and every non-NaN result, including zero,
  by bits against independent accepted arithmetic. Introduce no portable
  NaN-payload or zero-sign guarantee.
- Numbers, literal null, booleans, trimmed strings, empty/singleton lists
  and dates keep intrinsic numeric conversion; other nonnumeric values
  yield numeric NaN. Conversion neither resolves, invokes nor tracks
  shadowable Number or other Math bindings, nor calls public Math dispatch.
- Evaluate every supplied expression once left to right in the new call's
  five-input order, including extras and expressions after Undefined.
  Extra values are not converted or used, but retain reads, effects,
  dependencies, strict failures, deferral and rollback.
- The accepted nested composition evaluates target arguments before its
  inner source arguments. Do not claim effect-order equivalence for arbitrary
  expressions; the paired measured programs use pure variable/literal reads.
- Supplied Undefined preserves inherited deferred assignments, live-null
  and parameter-null handling, missing properties, deletion and recovery.
  Intrinsic literal-null conversion does not override those inherited rules.
- Preserve namespace/local Math shadowing and ordinary `.Math` lookup.
  Reading `Math.remap` as a value stays unsupported/Undefined, not a
  first-class callable alias. Wrappers, frames, scopes, receivers, class
  rules, call APIs, error positions, transactions and resource bounds remain.
- Strict unknown arguments, including extras after Undefined, retain
  `ReferenceError("<name> is not defined")`. Unsupported random calls retain
  `TypeError("Math.random is not supported")`; unknown static calls retain
  `TypeError("Math.<name> is not a function")`. Numeric `Math = 7` gives
  `TypeError("number.remap is not a function")` through ordinary dispatch.
- Literal EOF after `Math.remap(` gives
  `SyntaxError("Unexpected end of input")` at line 1, column 12; with a final
  newline it gives that same error at line 2, column 1. Numeric conversion,
  omissions, nonfinite values and ranges introduce no new errors.
- All existing Math arms, helpers, constants, conversions and errors remain
  unchanged, including sign, lerp, inverseLerp, clamp, saturate, smoothstep,
  smootherstep and fract. All accepted syntax remains available. Add no parser,
  lexer, AST, generator, CLI implementation, public API/representation,
  dependency, version, MSRV or edition change.
- The five entries titled `Nucleoid keeps range remapping reactive`,
  `Nucleoid preserves intrinsic conversion during range remapping`,
  `Nucleoid preserves remapping endpoints nonfinite inputs and omissions`,
  `Nucleoid applies range remapping to live class rules`, and
  `Nucleoid preserves deferred range remapping and rollback` trace this
  contract; keep the independent inverseLerp unit control unchanged.
- New RHS ASTs and immediate graphs intentionally differ from the nested
  composition. Preserve non-RHS source/AST metadata, canonical assertions,
  expected returns, observable values, transitive behavior and actual coverage,
  including both real catches, not whole-new-program immediate-graph identity.
- Preserve every historical program, its boundary whitespace and full AST.
  Inherited noncapture, parameter-dependent-local, Windows debug stack and
  old whole-suite Gitless discovery limitations are not repaired.

## Reactive flat mapping

- `list.flatMap(callback)` is a read-only ordered zero/one/many projection.
  Class populations support the same query through the shared list-query
  route. The query itself does not mutate its receiver; callbacks retain
  their ordinary effects.
- Snapshot the receiver before evaluating supplied arguments, exactly as for
  `map`. Evaluate all supplied argument expressions once, left to right,
  including extras, before validating or invoking the callback. A later
  argument error therefore precedes callback validation. A null or undefined
  receiver retains existing call routing and does not evaluate the arguments.
- The first supplied value must be a function, even for an empty receiver;
  a missing or non-function callback raises
  `TypeError("flatMap expects a function")`.
- Invoke the callback once for each snapshotted item in order, with exactly
  one argument and the same invocation and receiver semantics as `map`.
  Additional declared parameters are bound to `null`, not an index or receiver.
  Anonymous callbacks do not gain closure capture.
- A returned list contributes its immediate items. Any other returned value
  contributes one item. Preserve order, duplicates, object and function
  identities, and deeper lists without recursively flattening them. A returned
  empty list contributes no items; strings are not split into characters.
- Preserve returned `null` and `undefined` as values without manufacturing
  reads. Reading a null binding, property or parameter still sets existing
  null tracking and can make the enclosing assignment settle to `null`.
  Missing-property and deleted-name reads retain their actual deferred
  behavior; nested undefined values are not unconditionally converted or
  used to defer the complete result.
- Callback, receiver and argument reads use the existing dependency graph.
  List mutations, class population changes, member updates and named-function
  redefinitions refresh dependents; obsolete read dependencies are removed.
  `.value` freezes its ordinary reading: a frozen function definition still
  reads live state from its body.
- Propagate callback and argument errors unchanged through existing
  transactions, including rollback of effects, relationships and subscriptions,
  and ordinary recovery afterward. Existing depth and propagation bounds apply;
  no flat-mapping-specific cap, depth option or recursion is added.
- Existing `map`, `reduce`, `concat` and all accepted syntax remain available.
  Object/instance members and user functions or properties named `flatMap`
  keep their ordinary dispatch; native query methods do not become first-class
  bound properties. No parser, AST, keyword, public Rust type or dependency
  changes are required.
- The five executable flat-mapping entries below cover one-level projection,
  live lists, class members/populations, callable snapshots and failed-callback
  rollback. The accepted map/reduce/concat composition remains useful but is
  not an unconditional whole-program equivalence: its reducer reads an
  intermediate null item, while native flat mapping adds no such read.
  Rewritten runtime-call right-hand sides intentionally have different AST
  and canonical graph source; preserved assertions, returns and actual
  dependencies are checked rather than claiming complete graph-source equality.

## Implicit iterator indented loops

- `for NAME:` with a genuine indented body is optionally exactly
  `for it of NAME:` with that same body. `NAME` is one ordinary identifier
  denoting the source expression, not the iteration variable.
- After `for`, recognize the complete immediate `Identifier Colon` prefix
  without consuming tokens, allocating names, cloning an AST or skipping header
  separators. Immediately after the colon require `Newline` or `Indent`, then
  require a real `Indent` after only existing `Newline`/`Semicolon` layout.
  A colon immediately followed by a semicolon (`:;`) never qualifies.
- Leave the identifier for the ordinary expression parser and the colon,
  layout and complete body for unchanged `suite()`. Lower directly to the
  existing `Stmt::For { variable: "it", iterable: Identifier(NAME), body }`,
  preserving the explicit form's full AST and canonical `for it of NAME`.
- Blank/comment lines and normal body separators remain suite layout. No
  omitted source, inline/braced/missing-indent shorthand, parenthesized source,
  property/index/call/literal/class-reference source or other expression
  shorthand is added. `for it:` uses the outer source named `it`.
- Retain both explicit `for variable of expression` and
  `for variable in expression`, their arbitrary source expressions,
  inline/braced/indented suites and existing diagnostics. No new tokens,
  keywords, public AST, runtime types or general iterable support are added.
- A loop evaluates its source once and iterates a snapshot once, not a standing
  declaration. A class value supplies its current ordered registered instances;
  a list supplies only object references still registered with a class.
  Primitives, null elements, unregistered references, plain `Object()` instances
  and literal objects in lists are skipped, preserving order and duplicates.
  A String source supplies Unicode scalars as specified in
  [string iteration](#string-iteration); this does not broaden list iteration.
- Source and body mutations do not extend that snapshot. Later input updates
  and newly created instances do not re-run the body; another loop is required.
  Existing imperative writes, graph keys, dependency/shape tracking, class
  populations, receiver lookup and transactions are unchanged.
- Evaluate the source before introducing the iteration binding. Each iteration
  pushes a scope, declares local `it` and enters imperative mode; completion,
  return or error restores that mode and scope. Nested `it` shadows and restores
  outer/global `it`; `this`/`self` still denote the enclosing receiver.
- Conditions, nested loops, multiple statements, explicit/bare returns and
  absent returns retain existing control flow. A return propagates out of the
  loop; an empty or normally completed loop yields `null`. No implicit return,
  closure capture or callable auto-invocation is introduced.
- A nonqualifying tokenizable prefix follows the untouched explicit-loop path.
  `for values:` at EOF, without a real indent, with only comments, with an
  inline/braced body or with immediate `:;` retains
  `SyntaxError("Expected 'of' in for statement")` at line 1, column 11.
  `for values.items:` fails with that message at line 1, column 11.
  `for values\n:` fails with it at line 1, column 5.
- `for (values):` retains `SyntaxError("Expected a name but found '('")`
  at line 1, column 6; `for :` retains
  `SyntaxError("Expected a name but found ':'")` at line 1, column 6.
  Lexical failures and errors inside a qualifying body keep their existing
  diagnostics and positions.
- Evaluating an undefined source retains
  `ReferenceError("NAME is not defined")`. A noniterable named source, including
  literal `null` stored in `broken`, retains
  `TypeError("broken is not iterable")`; a callable source is not invoked.
  Runtime errors use the existing enclosing top-level statement position.
  Thrown values, caught errors, rollback and subsequent recovery are unchanged.
- The shared private real-indented-suite predicate preserves the exact
  single-untyped, single-typed and multiple bare-function block guards; it does
  not relax parameter, arrow or suite grammar. Existing expression/statement
  nesting, operand and postfix limits remain in force.
- Executable traces are the five appended `Nucleoid ... implicit iterator ...`
  entries below. Historical explicit loops remain unchanged, including
  `Nucleoid creates a for of statement`,
  `Nucleoid creates a block of for statement without dependencies` and
  `Nucleoid loops through only defined objects in a for of statement`.

## String iteration

- Existing `for variable of expression`, `for variable in expression` and
  genuine-indented `for NAME:` forms also accept a String source, without new
  grammar. Explicit sources keep arbitrary expressions and all existing suite
  forms; the named-source shorthand still requires one ordinary identifier.
- Evaluate the source exactly once, before introducing the iteration binding.
  Fully snapshot one String value per Unicode scalar in source order before
  any body executes, using the same scalar decomposition as `text.split("")`,
  string indexing and string length. An empty String executes zero bodies.
- Preserve each scalar exactly, including supplementary characters, combining
  marks, zero-width joiners and embedded NUL. Do not split bytes, UTF-16 units
  or grapheme clusters; do not normalize, trim or coerce the source.
- Each iteration uses the existing temporary scope and imperative body.
  Nested bindings shadow and restore outer/global `it`; source mutation cannot
  extend or replace the current snapshot. Later source changes never re-run a
  completed body; another For statement takes a fresh snapshot.
- Enclosing `this`/`self`, explicit/bare returns, normal completion, thrown
  values, statement positions, transactions and recovery use the unchanged
  For route. Early return exits the loop; normal/empty completion is `null`,
  not an implicit return of a body expression or collected list.
- Keep the Class and List branches unchanged: classes supply their current
  ordered instances; lists visit only registered class-object references,
  retaining duplicates and skipping all primitive entries, including Strings.
  `for character of text.split("")` therefore still skips those primitive
  list items; it is not an accepted counterpart for direct String traversal.
- A callable source is not invoked automatically. All other noniterable kinds
  retain `TypeError("broken is not iterable")` for a source named `broken`;
  an undefined source retains `ReferenceError("missing is not defined")`.
  These use the enclosing top-level statement position, as before.
- No general iterator protocol, callback, coercion, standing declaration or
  new error is introduced. Existing split/map expressions remain supported;
  their map results and callback ASTs are not equivalent to a For statement's
  `null`. The paired examples discard those intermediate map results and
  preserve their final controls, observations and explicit expected return.
- The six appended entries trace source order and return
  (`Nucleoid iterates string characters in source order`), exact scalar
  decomposition (`Nucleoid preserves Unicode scalars while iterating strings`),
  stable one-shot snapshots (`Nucleoid snapshots string sources before loop bodies`),
  nested local restoration (`Nucleoid keeps string iterator scopes and nested loops independent`),
  zero-body traversal (`Nucleoid skips empty strings without executing loop bodies`)
  and caught throw/rollback/recovery
  (`Nucleoid rolls back failed string iteration and restores caller scope`).
- The existing `Nucleoid preserves implicit iterator loop scopes`,
  `Nucleoid preserves implicit iterator loop return flow`,
  `Nucleoid rolls back failed implicit iterator loops` and
  `Nucleoid loops through only defined objects in a for of statement` entries
  remain the shared scope, control-flow, error and List contracts.
  `Nucleoid treats this and self as interchangeable instance keywords` and
  `Nucleoid rejects this and self outside an instance or as assignment targets`
  retain the enclosing receiver and exact missing-receiver error contracts.

## Colon-free indented else suites

- A final `else` may omit its colon only before a genuine indented suite.
  The spelling is optional and has exactly the complete AST and behavior of
  `else:` with that same body; the alternate remains a `Stmt::Block`.
- After the existing `Else` token, require immediate `Newline` or `Indent`.
  Look ahead across only `Newline`/`Semicolon` tokens; the first other token
  must be `Indent`. Consume nothing on failed qualification, then use the
  unchanged colon-required suite parser and its diagnostics.
- Reuse the existing suite-body parser, including leading/trailing layout,
  all statements, nested suites, optional final `Dedent`, and EOF handling.
  Blank/comment lines create no indent; LF, CRLF and accepted tabs keep the
  existing lexer rules. A semicolon on the header line never qualifies.
- Conditional headers follow the optional-colon rule below; `for`, definition,
  class, try/catch and anonymous headers keep their accepted grammars.
  Colon-bearing indented, inline, semicolon-separated and braced alternatives
  remain valid, with unchanged dangling-else association.
- Strings, regexes, templates and comments containing `else:` are not headers.
  No token, keyword, public AST/API, evaluation, scope, frame, graph, return,
  throw, rollback or resource-limit behavior changes. Canonical rendering
  remains unchanged; colon omission is a source-only choice.
- For exact `if false: pass\nelse`, failed qualification raises
  `SyntaxError("Expected ':' but found end of input")` at line 2, column 5.
  A trailing newline, comments-only body or unindented `pass` instead raises
  `SyntaxError("Expected ':' but found end of line")` at line 2, column 1.
- Exact `if false: pass\nelse;\n    pass` keeps
  `SyntaxError("Expected ':' but found ';'")` at line 2, column 5.
  Same-line `else pass` and `else { pass }` keep
  `"Expected ':' but found keyword 'pass'"` and
  `"Expected ':' but found '{'"`, respectively, at line 2, column 6.
- Exact `if false: pass\nelse:\npass` retains
  `SyntaxError("Expected an indented block but found keyword 'pass'")`
  at line 3, column 1. After a qualifying bare `else`, four-space-indented
  `value = (1 +)` raises `SyntaxError("Unexpected ')'")` at line 3, column 18;
  `value = @` raises `SyntaxError("Unexpected character '@'")` at column 13.
- The five appended entries cover nested live branches, per-instance class
  rules, function returns, caught rollback, and legacy/literal controls.
  They execute 33 assertions and preserve explicit returns `17` and `6`.
  The reference, README and examples counterparts execute 12 more assertions.

## Direct conditional alternatives

- Within an existing conditional chain, `else CONDITION:` and
  `elif CONDITION:` optionally spell `else if CONDITION:` only when the
  complete balanced condition header precedes a genuine indented suite.
  Their suite colon is optional under the conditional-header rule below.
  Both lower to the same complete nested `Stmt::If`, not an unconditional
  block or a new runtime operation.
- `elif` remains the existing lexical alias for `else`, not a new keyword.
  Explicit `else if` and `elif if` keep priority and all their accepted bodies,
  including inline, semicolon-separated and braced suites. Final `else:` and
  `elif:` remain unconditional; final colon-free `else` and `elif` retain the
  genuine-indentation rule above.
- After `Else`, a private, non-consuming token scan qualifies only a nonempty
  balanced header. Match parentheses, brackets and braces; ignore their nested
  colons, including slices, object fields, calls and parameter annotations.
  A top-level colon, newline or indent ends the scan and must qualify with the
  existing real-indent predicate; mismatched delimiters, semicolons, dedents
  and EOF fail. Do not skip header separators, parse a speculative expression,
  catch/rewind an error or evaluate a condition during lookahead.
- With a colon, require immediate `Newline` or `Indent` after it; without one,
  require that same layout immediately after the balanced nonempty condition.
  A real `Indent` must follow only the existing newline/semicolon layout.
  Blank/comment-only lines do not create an indent. LF, CRLF, tabs and delimited
  multiline expressions retain their existing lexer and suite behavior.
- Qualification does not validate an expression. Once qualified, the ordinary
  condition and whole-suite parsers report their real errors. If it fails,
  consume nothing and use the original final-else parser and its original
  source positions and diagnostics. New direct conditions have no inline,
  braced or `:;` shortcut and cannot stand alone.
- Complete ASTs, canonical graph keys and top-level statement positions match
  their explicit counterparts. Conditions still use existing truthiness,
  first-match and short-circuit evaluation, actual reads and dependency
  replacement. Class-level rules evaluate independently per instance.
- Ordinary scopes, receiver frames, explicit returns, live and frozen
  functions, noncapture, propagation ordering, transactions, rollback and
  resource limits are unchanged. This syntax does not repair the inherited
  queued-method receiver failure: a receiver-property change may re-run a
  method body after its receiver frame and raise
  `ReferenceError("this is not defined")`.
- Exact `if false: pass\nelse ready: pass`, `else ready: { pass }`,
  `else ready:;\n    pass`, `else ready:\npass` and
  `else ready\n    # no body` keep
  `SyntaxError("Expected ':' but found identifier 'ready'")` at line 2,
  column 6. The same `elif` prefixes retain the same diagnostics.
- Exact `if false: pass\nelse true +:\n    pass` qualifies, then raises
  `SyntaxError("Unexpected ':'")` at line 2, column 12.
  Exact `if false: pass\nelse true:\n    value = (1 +)` raises
  `SyntaxError("Unexpected ')'")` at line 3, column 18.
  A real `missing` read in that suite retains
  `ReferenceError("missing is not defined")` at the enclosing top-level
  conditional's line 1, column 1.
- Expression/statement nesting remains bounded at 64 and operator/postfix
  chains at 128, with existing error messages and positions. Existing
  recursive else-if-chain quirks are not changed. No lexer, public token,
  AST, Rust API, canonical AST renderer, runtime or dependency change is
  introduced by the alternative spellings.
- The document/JSONL statement exporter keeps both complete `else` and `elif`
  keywords with the conditional, including tab-separated and compact
  delimiter-, string- and regex-started conditions accepted by the lexer.
  Its keyword boundary follows `[A-Za-z0-9_]`, so `elsewhere`, `else1` and
  `elif_name` remain independent statements. It does not validate conditions
  or relax the parser's balanced-header and genuine-suite rules. The top-level
  class-rule entry below exercises direct `else` through generated spec tests.
- The five entries titled for direct alternatives cover routing,
  independent class rules, first-match/short-circuit reads, receiver-scope
  returns and rollback. All 33 assertions execute, including both catches;
  the two explicit string returns are `"wait"` and `"skip"`. The appended
  reference, README and examples cases execute 17 further assertions.
  The conditional-colon revision below changes only these same eight programs;
  other historical cases remain unchanged accepted-syntax controls.

## Colon-free indented conditional suites

- An initial `if CONDITION`, explicit `else if CONDITION` or `elif if CONDITION`,
  and accepted direct `else CONDITION` or `elif CONDITION` may omit the suite
  colon only before a genuine indented body. The spelling is additive and
  source-only; every accepted colon-bearing form remains valid.
- Parse each condition with the unchanged expression parser. With an explicit
  colon, retain `suite()` unchanged, including inline, semicolon-separated,
  braced and indented bodies wherever that header already accepts them.
  Otherwise, immediately require `Newline` or `Indent`, then a real `Indent`
  after only `Newline`/`Semicolon` layout. Qualify without consuming anything;
  only success selects the existing complete `suite_body()` parser.
- Direct alternatives additionally require the complete nonempty balanced
  header scan above, matching parentheses, brackets and braces. Nested calls,
  slices, object fields, function expressions, annotations, strings, regexes,
  templates and comments cannot manufacture a header delimiter. A mismatch or
  unbalanced prefix retains the original final-else error, without speculative
  parsing, error catching or rewinding.
- A header-line semicolon, absent body, comment-only body or unindented body
  does not qualify. Blank/comment lines create no indent; LF, CRLF, tabs,
  comments and layout inside balanced expressions retain existing lexer rules.
  A colon-free inline or braced body is not added.
- Final bare `else` and `elif` remain unconditional blocks under their already
  accepted indentation rule. Mixed old/new suites keep recursive branch
  association and the full `Stmt::If` tree, canonical rendering, statement
  positions, graph keys, metadata, ordered actual reads and dependencies.
- Truthiness, first-match and short-circuit order, independent per-instance
  class rules, live/frozen function behavior, caller/receiver scopes, explicit
  returns, rollback and recovery are unchanged. Expression/statement nesting
  and chain bounds, including inherited recursive-chain quirks, are unchanged.
  No general optional-colon `suite()`, unrelated header grammar or receiver
  repair is introduced.
- Exact `if true` at physical EOF raises
  `SyntaxError("Expected ':' but found end of input")` at line 1, column 8.
  A trailing newline, comment-only body or unindented `pass` instead raises
  `SyntaxError("Expected ':' but found end of line")` at line 1, column 4.
  Exact `if true;\n    pass` retains
  `SyntaxError("Expected ':' but found ';'")` at line 1, column 8.
- Exact `if true pass` and `if true { pass }` retain
  `SyntaxError("Expected ':' but found keyword 'pass'")` and
  `SyntaxError("Expected ':' but found '{'")` at line 1, column 9.
  Nonqualifying direct headers retain the final-else errors above.
  A qualified `if true\n    value = (1 +)` reports
  `SyntaxError("Unexpected ')'")` at line 2, column 18, and a real missing read
  keeps the enclosing top-level statement's runtime position.
- The native REPL lexically recognizes an initial colon-free conditional header
  and buffers through blank-line commit or EOF, with automatic body indentation.
  Explicit/direct alternatives only adjust indentation in that existing buffer.
  Lexical comments and balanced expression delimiters are not suffix guesses;
  the real parser still validates the entire submitted source, exposes errors
  and preserves subsequent command/error recovery.
- The existing production document/JSONL splitter already retains full
  conditional chains, comments and indentation. It is unchanged; actual
  per-statement export execution and generated spec tests cover the shorter
  headers alongside legacy controls.
- This revision rewrites only the five direct-alternative spec programs and
  their existing reference, README and examples counterparts. Their identities,
  order, 50 protected/executed assertions and two explicit returns are unchanged.
  Only 29 qualified condition-bearing header-colon bytes are removed. The 11
  unconditional final-alternative colons remain byte-exact accepted syntax.
  All other 625 of the 633 measured programs remain byte-exact legacy controls;
  no measured identity is added.

## Comma-free explicit object properties

- [NUC 44](docs/nuc-0044.md) optionally omits a comma
  between explicit object properties, as in `{amount: amount unit: "C"}`.
  Comma-bearing, mixed and trailing-comma forms remain supported.
- Parse the entire ordinary explicit value expression first, then apply
  the original newline/semicolon skipping and give an existing comma priority.
  Without a comma, continue only when the next two unconsumed tokens are
  `Str`, `Identifier` or `Number` immediately followed by `Colon`.
  The non-consuming guard does not parse, rewind, reinterpret or stop a value.
- Reuse the existing key formatting, colon and value parser. Preserve the
  complete ordered object entries, including duplicates; canonical rendering
  remains quoted and comma-separated, such as `{"amount":amount,"unit":"C"}`.
- Shorthand qualification is unchanged: an identifier needs a following
  comma or closing brace through only the existing newline/semicolon tokens.
  `{amount unit: "C"}` and `{amount: 1 unit}` do not qualify. Keep the
  commas around shorthand wherever its original rule requires them.
- Ordinary operators, assignments, postfixes, calls, arrows, functions and
  nested objects take their existing expression priority. No new key variants,
  colon continuation, brace indentation, list/call/parameter separators or
  statement-block syntax is introduced. A failed guard retains the original
  closing-brace path, diagnostic and source position.
- Scopes, class receivers, aliases, duplicate evaluation/replacement order,
  actual reactive reads, `.value` snapshots, transactions, caught whole errors,
  rollback and resource limits are unchanged. `value` remains the reserved
  snapshot API, not an ordinary mutable property.
- Exact `record = { amount other }` retains
  `SyntaxError("Expected ':' but found identifier 'other'")` at line 1,
  column 19. Exact `record = { amount: 1 next }` retains
  `SyntaxError("Expected '}' but found identifier 'next'")` at line 1,
  column 22. Exact `record = { amount: }` retains
  `SyntaxError("Unexpected '}'")` at line 1, column 21.
- Exact `record = { amount\n: amount }` retains
  `SyntaxError("Expected ':' but found end of line")` at line 1, column 12.
  Exact `record = {\n    amount: amount,\n}` retains
  `SyntaxError("Expected a key but found an indented block")` at line 2,
  column 5. The existing lexer, exporter and CLI layout limits are not repaired.
- The five appended `comma-free record` entries below specify live explicit
  fields; quoted/numeric duplicate keys and named state writes; nested records
  with unchanged list/call commas; function scope and snapshots; and rollback
  after an actually evaluated missing read. Their 28 assertions and returns
  `"F"` and `"old"` retain the pre-executed accepted-syntax counterparts.
  The appended reference, README and examples cases retain 12 more assertions.
- All 587 earlier measured programs and 25 earlier canonical training files
  remain unchanged. Independent evaluation accepted all 595 paired programs:
  eight use fewer lexer tokens, 587 are unchanged, and none increased.
  All 2,706 protected assertions executed and all 46 expected returns matched.

## Colon-led assertion statements

- Accepted [NUC 45](docs/nuc-0045.md) adds the optional statement
  `assert: actual, expected`, lowering exactly to
  `Stmt::Expression(Expr::Call { callee: Identifier("assert"),
  arguments: [actual, expected] })`. Canonical rendering remains
  `assert(actual,expected)`, not the colon-led source spelling.
- Recognize only the lowercase identifier `assert` immediately followed by
  an existing colon token in statement position. Ordinary lexer whitespace
  is permitted; do not skip new layout or introduce multiline continuation.
  Parse exactly two whole ordinary expressions separated by the existing
  comma. No extra argument or trailing comma is added to this spelling.
- Existing typed declarations take priority with exactly their original
  following-identifier and newline/semicolon/dedent/end-of-input boundaries.
  `assert: int` still parses as a declaration and raises
  `ReferenceError("Missing definition")`. No right-brace boundary is added.
- Keep `assert(actual, expected)` and all its existing expression, argument
  count and optional trailing-comma behavior. Bindings, parameters, type
  annotations, class fields and object keys named `assert` are unchanged.
  A direct call named `assert` is the existing helper even if that name is
  bound; it does not use normal callable shadowing.
- Evaluate both arguments once, left to right. Preserve deep equality,
  collected failures, counters, null helper result, error propagation,
  transactional rollback, scopes, receivers and `.value` snapshots.
  Assertion expression statements do not replace the program's result or
  install dependency nodes or reasoning observers. A check is not itself a
  reactive rule; enclosing functions/rules retain existing re-execution.
- Preserve statement depth, outer call-expression depth, argument-expression
  depth and operator/postfix chain limits. The lexer, public AST, runtime,
  canonical generator and CLI gain no new representation.
- A missing comma uses `SyntaxError("Expected ',' but found <token>")`.
  Missing operands use existing `Unexpected <token>` expression diagnostics.
  A comma after the second expression raises
  `SyntaxError("Assertion statements expect exactly two arguments")`.
  `assert: 1,\n1` and `assert:\n    1, 1` raise
  `SyntaxError("Unexpected end of line")`; no implicit continuation is added.
  Resource errors remain `Expressions are nested too deeply`,
  `Statements are nested too deeply`, `Expression has too many operands`
  and `Expression has too many parts`, all `SyntaxError`.
- The five appended colon-led entries specify values and structural equality;
  graph exclusion and reasoning; local scope and returns; argument effects;
  and whole caught errors with rollback. Together with the appended reference,
  README and examples cases they preserve all eight pre-executed accepted
  controls, 30 executed assertions and explicit returns `7` and `true`.
- All 595 historical measured programs and 26 earlier training files (25
  synthesized shards plus ARC) remain raw-exact. The new four-record shard is
  `dataset/nucleoid.synth.26.jsonl`. Independent evaluation accepted all 603
  paired cases: eight use fewer lexer tokens, 595 are unchanged, and none
  increased. All 2,736 protected assertions executed and all 48 explicit
  expected returns matched.

## Colon-free indented exception suites

- `try` and a complete existing `catch` header may independently omit their
  suite colons only before a genuine indented body. This is optional syntax
  for the identical complete `Stmt::Try { body, parameter, catch }`.
- Keep the default `error` binding, `catch name`, `catch(name)` and
  `catch (name)`. Parse the existing complete parameter form first; do not
  add expressions, filters, multiple catches or `finally`.
- An explicit colon takes the original suite path first, including inline,
  semicolon-separated, braced and indented bodies. Without it, require
  immediate `Newline` or `Indent`, then a first non-`Newline`/`Semicolon`
  token of `Indent`, using the existing `has_indented_body` predicate before
  consuming any layout. Failed qualification uses the original colon-required
  path with its exact errors and positions.
- Share the existing `suite_body` parser, complete statement nesting budget,
  separators, dedent and EOF handling. LF, CRLF, tabs, comments and blank
  lines follow the existing lexer; comments alone do not create a body.
  A header-line semicolon, inline statement or bare brace cannot qualify.
- Other condition, loop, class and function headers retain their independently
  specified grammar and qualification rules. No lexer, keyword, public AST,
  canonical generator, runtime, dependency or version changes.
  Canonical keys and all colon-bearing accepted source forms remain unchanged.
- In the native REPL, a complete lexical bare `try` header, including accepted
  layout and trailing comments, starts the existing multiline buffer through
  its catch. A standalone `catch` does not start buffering. Blank-line commit
  and EOF flush parse the original source and print actual diagnostics.
  Literals, comment-only lines, identifier prefixes, assignments, extra tokens
  and invalid lexemes do not qualify as bare `try` headers. Existing colon/arrow
  buffering and command handling remain unchanged.
- Keep the original savepoint, rollback, pending propagation/deletion tracking,
  dependencies, class rules/instances, whole thrown values/errors, scope,
  noncapture, receivers, snapshots, reasoning and return flow. This is not a
  runtime repair or a dependency on an unimplemented transaction proposal.
- Exact `try` keeps `SyntaxError("Expected ':' but found end of input")`
  at line 1, column 4. Exact `try\n` keeps
  `SyntaxError("Expected ':' but found end of line")` at line 1, column 1.
  Exact `try;\n    pass\ncatch: pass` keeps
  `SyntaxError("Expected ':' but found ';'")` at line 1, column 4.
- Exact `try: pass\ncatch` keeps
  `SyntaxError("Expected ':' but found end of input")` at line 2, column 6.
  Exact `try: pass\ncatch issue pass` keeps
  `SyntaxError("Expected ':' but found keyword 'pass'")` at line 2, column 13.
  Malformed parenthesized bindings retain the original name/closing-parenthesis
  diagnostics. After qualification, lexical/body errors retain their actual
  physical positions; missing catches still require `catch`.
  Exact `try: pass\ncatch\n    value = (1 +)` raises
  `SyntaxError("Unexpected ')'")` at line 3, column 18; the colon-bearing
  catch has the same error. In `try\n    value = (1 +)\ncatch: pass`, the
  inherited next-newline position is line 2, column 17, also unchanged from
  its colon-bearing try control. Source-column quirks are not repaired.
- The five appended exception entries trace whole thrown values/default and
  parenthesized bindings (four checks), dependency rollback (four checks,
  return `12`), nested catch association (two checks), function returns
  (two checks) and class-rule rollback (four checks).
  The appended reference, README and examples entries retain another eleven
  checks and return `true` in the examples entry: twenty-seven checks and
  two explicit returns across exactly eight pre-executed counterparts.
- Keep all 603 historical measured sources and all 27 earlier training files
  unchanged. Four development-executed, self-contained recovery records in
  `dataset/nucleoid.synth.27.jsonl` bring the training metadata to 408 spec
  records and 1,107 synthesized records in 27 synthesized shards plus ARC.
  Training records are not new measured baselines or normative test cases.
- Independent evaluation accepted NUC 46 against
  `4ed54c0994e1425d2345f2758b7431c9f36f7562`: all 611 cases passed, eight
  use fewer lexer tokens, 603 are unchanged, and none increased. Total tokens
  changed from 58,091 to 58,069 without offsetting any per-case increase.
- All 2,763 protected assertions executed and all 50 explicit expected returns
  matched. Accepted-syntax controls and candidate programs preserved complete
  ordered ASTs, positions, canonical assertions and actual coverage, including
  root-level native REPL recovery. Full per-case counts are in
  [NUC 46](docs/nuc-0046.md#reference-implementation).

## Default named constructors

- An ordinary `class Student(name, age):` accepts `Student("Can", 20)`,
  `Student(name="Can", age=20)`, `Student(age=20, name="Can")`, and
  `Student("Can", age=20)`. Typed parameters retain annotation metadata.
  No header marker or separate named-only class mode exists.
- This revision intentionally removes the previously accepted class-header
  star and reinterprets bare constructor `name=value` arguments as labels.
  These are the two explicitly authorized migrations, not universal source
  compatibility. The existing case titles containing "opted-in" are stable
  corpus identities, not a continuing header requirement.
- Only a bare identifier followed by ordinary `=` at the call-argument
  boundary is a label. It does not read, write, or create a dependency on a
  caller variable. The value expression keeps normal scope and side effects.
- Explicit grouping preserves an assignment argument:
  `Legacy((b=2), (a=1))` passes `2, 1` positionally and assigns caller `b` and
  `a`. Compound assignments and member-target assignments remain positional.
  Rendering/reparsing retains this distinction; ordinary expressions do not
  gain persistent grouping nodes.
- Functions, methods, built-ins, and direct `init` calls retain positional
  assignment-expression arguments, including `pair(b=2, a=1)`. Class aliases
  gain no construction route. Existing assignment-instantiation versus
  ordinary-call function/local shadowing differences remain unchanged.
- For calls containing labels, validate the entire binding plan before any
  argument effects or `.value` reads. Positional arguments fill successive
  declared slots and must precede all labels. Reject the first unknown name,
  repeated binding (including positional plus named), or positional argument
  following a named argument, in source order.
- Evaluate supplied values exactly once in source order in the caller's
  scope, then bind them in declaration order. Omitted parameters without
  literal defaults retain `null`; named binding itself adds no defaults or
  automatic property assignments. Literal defaults are specified separately.
  Surplus positional values are still evaluated and ignored. Existing
  duplicate formal names and purely positional binding are not redesigned.
- Use the nearest nonempty constructor body's signature, including adopted
  `def init` signatures and inherited constructors. An empty child header
  cannot replace that effective signature. Constructor scopes, receivers,
  field declarations, inheritance grammar and instance populations are unchanged.
- Named/mixed calls pin the validated effective signature/body before values
  run, even if a value redeclares the class. Purely positional calls retain
  constructor selection after argument evaluation and existing `.value`
  preparation. Redeclaration preserves instances and class rules.
- Constructor values remain snapshots. A surrounding standing expression may
  still re-evaluate on its own actual reads and create another instance.
  `.value` remains a snapshot read without a dependency.
- All existing direct construction routes support this binding: ordinary
  expression calls, assignment-named instances, nested properties, `new`, and
  shared class rules. Named/mixed shared construction uses the actual
  declaration caller's scope; purely positional shared construction keeps
  its existing global-only scope and diagnostics.
- `super(b=second, a=first)` binds labels against the effective parent
  constructor on the same receiver. Positional and mixed forwarding work
  too. Constructor context advances from the executing constructor; method
  fallback uses the receiver's actual class. Bare `super`, `super()`,
  no-parent calls and missing-receiver evaluation/error order are unchanged.
- Class-rule root and block instance-read checks distinguish labels from
  assignment targets and real RHS references using actual call routing.
  Label names never become dependency/root variables. Canonical call source
  remains `Pair(b=2,a=1)`; explicit assignment source remains
  `Pair((b=2),(a=1))`. Default-free class statement keys remain `class Pair`.
- Binding failures precede object registration. Existing transactions and
  catch savepoints still restore partial instances, populations, properties,
  class metadata and graph writes after RHS/body/parent/rule failures.
  Caller-local assignment effects and fresh `Runtime::run` scopes retain
  their existing rules. The exporter keeps scope-sensitive programs whole.
- A constructor header star raises the ordinary parameter diagnostic
  `SyntaxError("Expected a parameter but found '*'")`; no star form is
  retained. Other parameter/suite diagnostics and source positions remain.
- An unknown label raises
  `TypeError("Unknown constructor argument '<name>' for <class>")`.
  A repeated binding raises
  `TypeError("Duplicate constructor argument '<name>' for <class>")`.
  A positional argument after a label raises
  `TypeError("Positional constructor argument after named arguments for <class>")`.
  `<class>` is the effective constructor's declaring class, including
  inheritance and `super`; placement and caught-value rendering are unchanged.
- Existing nesting, statement and operator-chain limits apply; no parameter
  count limit, dependency, project version or ARC change is introduced.
- The eight historical constructor entries and four new pre-executed Student
  and ordinary-function counterparts below protect all canonical assertions,
  expected returns, case identities and executed assertion coverage.

## State-filtered reasoning selections

- `selection |> stated` retains only steps whose existing typed state is
  `StepState::Stated`; `selection |> derived` retains only `StepState::Derived`.
  Neither stage infers state from the length of `from`, introduces provenance,
  changes a step's fields, nor changes the relative order of retained steps.
- `result |> why |> stated` selects the stated supporting facts in the
  explanation; `result |> why |> derived` selects its derived steps.
  Class-governed steps remain derived even when `from` is empty.
- Prefix `stated seed` and `derived result` filter just that reference's
  one-step selection. They do not implicitly traverse ancestors with `why`.
  A filtered-out reference produces an empty selection.
- `model |> stated` and `model |> derived` filter current model membership
  directly, still excluding pending nodes and reasoning observers.
  Nested pipelines compose as graph selections before existing materialization
  into the ordered `node`, `holds`, `rule`, `state`, and `from` records.
- Preserve every value and structural observation established by the source
  selection, including steps a filter excludes. Empty results still observe
  their sources and model membership. Equal-valued declaration/state changes,
  ancestry replacement, deletion, functions, class rules, observer replacement,
  `.value` snapshots, propagation ordering and rollback retain existing behavior.
- `stated` and `derived` remain ordinary lexer identifiers. Only the existing
  contextual stage positions recognize them; variables, parameters, keys,
  properties, user functions and methods retain their accepted meanings.
  `stated(seed)` and `derived(result)` remain ordinary calls, not added stage
  spellings. Existing `why`, `affects`, and list `.filter` forms remain supported.
- A new stage name shadowed by a local/parameter, state variable, function or
  class is not an operation: `seed |> stated` and `seed |> derived` then retain
  `TypeError("stated is not an operation")` and
  `TypeError("derived is not an operation")`, respectively, before selecting
  the source. Do not change the accepted `why`/`affects` shadowing behavior.
  An identifier value before a comma-free explicit object field remains an
  ordinary value, even when its name is `stated` or `derived`.
- Invalid sources retain existing reference diagnostics:
  `1 |> stated` raises `TypeError("Cannot explain 1")`, and `stated missing`
  raises `ReferenceError("missing is not defined")`. Filtering adds no new
  error type, diagnostic wording, parser precedence or resource bound.
- The seven appended state-filter entries below preserve their pre-executed
  accepted `.filter(step => step.state == ...)` counterparts, including the
  individual-reference counterparts' `.slice(0, 1)`, assertions and returns.

## Reactive degree to radian conversion

- The [NUC 50 candidate](docs/nuc-0050.md) adds only unshadowed static
  `Math.toRadians(value)`. The accepted arithmetic
  `value * (Math.PI / 180)` remains supported.
- Reuse Math's existing any-supplied-`Undefined` gate, including ignored
  extras, before its existing first-input intrinsic `Value::to_number`
  conversion. An omitted first input supplies numeric NaN, not Undefined.
- Compute `factor = intrinsic f64 PI / 180.0`, then `result = first * factor`,
  as separate ordinary `f64` operations in exactly that order. Do not compute
  `(first * PI) / 180`, use higher precision, rescale, clamp, normalize or
  shortcut inputs. In particular, finite `Number.MAX_VALUE` conversions
  remain finite instead of overflowing in an intermediate multiplication.
- Signed zeros and subnormal rounding follow the ordered composition exactly.
  The positive factor preserves zero signs, including underflow to zero;
  positive/negative infinities retain their signs. Compare expected NaNs by
  classification, not payload, and every non-NaN result, including zero,
  by bits against independent accepted division-then-multiplication controls.
- Numbers retain their values, literal null converts to zero, booleans to
  zero/one, trimmed strings and empty/singleton lists keep existing numeric
  conversion, and dates contribute milliseconds. Other nonnumeric
  conversions produce numeric NaN, not a new type error.
- Every supplied expression evaluates exactly once left to right per call,
  including ignored extras and expressions after an Undefined. Extra values
  are neither converted nor used numerically; their reads, effects,
  dependencies, strict errors and the supplied-Undefined gate remain.
- Any supplied Undefined returns Undefined before conversion. Missing
  properties, deletion and later definition retain deferral and recovery.
  Intrinsic literal-null conversion does not override live assignment or
  parameter-null deferral. Class rules remain independent per instance.
- Intrinsic conversion and the factor never resolve, invoke or track
  shadowable `Number` or `Math.PI` bindings. State/local Math bindings still
  shadow the namespace; ordinary `.Math` members retain lookup. Reading
  `Math.toRadians` as a value remains unsupported/Undefined, not a first-class
  callable. Wrappers, call frames, receivers and scopes are unchanged.
- Introduce no numeric, coercion, range or arity errors. Strict unknown
  arguments, including extras or arguments after Undefined, retain
  `ReferenceError("<name> is not defined")`. Preserve exact error positions,
  caught values, graph subscriptions, transaction rollback and recovery.
- Unknown static methods retain `TypeError("Math.<name> is not a function")`;
  `Math.random()` retains `TypeError("Math.random is not supported")`, after
  the unchanged supplied-Undefined gate. Calling through numeric `Math = 7`
  retains `TypeError("number.toRadians is not a function")`.
- All other Math arms, constants and helpers are unchanged. Add no alias,
  `Math.toDegrees`, parser syntax, public API/type, runtime representation,
  dependency, configurability or resource limit. Accepted syntax and
  inherited noncapture/parameter-local limitations remain unchanged.
- The six appended entries titled `Nucleoid keeps degree to radian conversion
  reactive`, `Nucleoid preserves numeric coercion while converting degrees to
  radians`, `Nucleoid preserves radian conversion extremes omissions and signed
  zeros`, `Nucleoid applies radian conversion to independent class rules`,
  `Nucleoid restores radian conversion relationships after failed transactions`,
  and `Nucleoid preserves ignored radian conversion reads and missing extras`
  trace this contract with unchanged numeric/assertion controls and returns.
- New call RHS ASTs intentionally differ from arithmetic counterparts.
  Preserve protected assertions, values, actual coverage and transitive
  behavior without requiring identical rule-source strings or immediate
  dependency graphs. Unchanged historical programs retain their complete ASTs.

## Read-only state versions

- A postfix `reference@0`, `@1`, ... reads a non-negative, oldest-first
  version of a variable or actual object property. `@first` means zero,
  `@last` means the newest entry, and `@previous` means the entry before it.
  Decimal digit literals must fit the host's `usize`; signs, decimal points,
  exponent spellings, expressions and unknown selector words are not selectors.
  The three words remain ordinary identifiers everywhere except after `@`.
- Versioning is automatic. Record the initial stored value at zero and append
  only on actual `Value::same_value` changes, including reactive propagation,
  constructors, initial object-property maps and mutable-list write-backs.
  NaNs compare unchanged, signed zeros differ, lists compare by contents and
  objects by identity. Reads and equal writes never append a source version.
- Histories belong to underlying variable slots and object identity/property
  slots, not textual aliases. A selected object variable switching identities
  selects the new owner's property history. Computed object-property reads
  evaluate their receiver and key exactly once. No list-element slots,
  arbitrary temporary histories, synthetic fallback entries, hidden object
  properties, wrappers or altered JSON/value representations are introduced.
- Global/state histories persist across `Runtime::run` calls, independently of
  the committed transaction log. Retain all entries until slot deletion,
  object removal or `State::clear`/`Runtime::clear`; redefining a deleted slot
  starts again at zero. Local/parameter/loop/catch bindings use the same
  innermost scope and frame lifetime as ordinary reads and writes.
- Successful state writes are immediately readable. Failed runs and caught
  savepoints undo state history appends, deletions and delete/recreate resets
  atomically with their corresponding state writes. Restoration adds nothing.
  Locals retain the existing Scope execution semantics rather than becoming
  standing or transactional state declarations.
- A selected version is an ordinary Value. Lists are ordinary value copies;
  historical object references retain identity and can reflect later property
  mutations. Real properties of a returned object are ordinary writable
  storage, not deep-frozen snapshots. Normal member, index, slice, call and
  `.value` snapshot composition remains available.
- Direct assignment, compound assignment, deletion, or mutable-list write-back
  into a selected version raises `TypeError("State versions are read-only")`.
  Copy a historical list into an ordinary binding before mutating the copy.
  A non-slot or namespace reference raises
  `TypeError("State versions require a variable or object property")`.
- A missing slot, out-of-range index, or `@previous` with fewer than two
  entries reads Undefined and settles to deferred Null in assignments.
  Retain the source dependencies for later creation/recovery. A valid old
  value remains readable even while the current source is Null or Undefined;
  only the selected result contributes its nullishness.
- Malformed selectors raise
  `SyntaxError("Expected a non-negative integer, 'first', 'last', or 'previous' after '@'")`.
  Existing diagnostics and chain/depth limits still apply. Ordinary dot fields
  `first`, `last` and `previous`, `.value`, negative list indexing and slicing,
  builtin/class/function namespaces, calls, one-shot execution, graph
  propagation and cycle behavior are unchanged.
- The six appended entries beginning `Nucleoid reads zero based and named
  versions of instance properties` through `Nucleoid restores version histories
  on rollback and resets deleted bindings` specify the measured runtime
  examples.

## Literal parameter defaults

- Only explicitly parenthesized user-defined formal lists accept
  `name = literal` or `name: Type = literal`. This covers named block/arrow
  functions, function expressions, parenthesized arrows, user methods,
  explicit class parameters and adopted `init` parameters.
- A default is a finite numeric literal with at most one optional `+` or `-`,
  a quoted String, a Boolean literal or null (including existing null literal
  aliases). Numeric sign bits, including `-0`, are retained. Defaults are
  immutable scalar data, never evaluated expressions or captured names.
- Arrays, objects, regexes, templates, identifier references, calls, grouped
  or compound expressions and nonfinite numbers are not defaults. An
  unsupported or missing literal raises
  `SyntaxError("Expected a scalar literal parameter default")`. A nonfinite
  numeric token raises `SyntaxError("Parameter default must be a finite number")`.
  Tokens after a scalar other than `,` or `)` raise
  `SyntaxError("Expected ',' or ')' after parameter default")`.
- Only an absent positional argument or absent constructor key uses the
  default. A supplied Null, Undefined, false, zero, empty String or any other
  value retains its existing evaluation and value semantics, including
  deferred/null propagation and `.value` snapshot boundaries. There is no
  nullish/falsy substitution. Missing arguments without defaults still bind
  Null; extra arguments and non-enforcing annotations keep their behavior.
- Defaults may occupy any formal slot, including one before a required slot.
  The effective scalar is chosen before declaring the fresh Scope binding:
  version zero is that value, not a synthetic Null followed by another write.
  Invocation and constructor frame lifetimes, local shadowing/noncapture,
  nontransactional locals and transactional State boundaries are unchanged.
- Ordinary, aliased, callback, method and function-value calls share existing
  invocation binding. Constructor positional/named/mixed calls, inherited
  constructors and `super` use the selected signature's defaults. Strict
  label validation, source order/single evaluation, receiver capture, and
  delayed positional versus pinned named constructor selection stay unchanged.
- Function argument assignments remain ordinary positional expressions, not
  named function arguments. Bare/comma-free signatures gain no default syntax.
  Automatic named constructors without formal slots, native built-ins, old
  grouped assignments and all previously accepted signatures remain unchanged.
- Live function callers observe redefinitions normally; frozen and historical
  function Values retain the captured definition and its defaults under
  existing identity semantics. Failed redefinitions restore function/class
  metadata, State and graph atomically through ordinary rollback.
- Defaults do not change state-version namespace restrictions, negative
  selector rejection, ordinary dot fields, snapshots or serializers. The six
  appended entries beginning `Nucleoid fills omitted literal default parameters`
  through `Nucleoid restores defaulted function callers after failed
  redefinitions` specify the measured behavior.

```
# Nucleoid runs a statement in the state

# i is 1
i = 1

assert(i == 1, true)

---

# Nucleoid runs a expression statement

# j is 1
j = 1

assert(j + 2, 3)

---

# Nucleoid returns value of variable

# k is 1
k = 1

k

# return: 1

---

# Nucleoid throws an error if variable is not defined

try:
    # t is e plus 1
    t = e + 1
catch error:
    assert(error, ReferenceError("e is not defined"))

---

# Nucleoid throws an error inside a block

# k is 99
k = 99

try:
    # if k is greater than or equal to 99, then throw "INVALID"
    if k >= 99:
        throw "INVALID"
catch error:
    assert(error, "INVALID")

---

# Nucleoid throws an error as a variable

# length is 0.1
length = 0.1

try:
    # if length is less than 1, then throw length
    if length < 1:
        throw length
catch error:
    assert(error, 0.1)

try:
    # if length is less than 1.1, then throw 'length'
    if length < 1.1:
        throw 'length'
catch error:
    assert(error, "length")

---

# Nucleoid creates a class with constructor

# There is a Shape type,
# which has a type as a string
class Shape(type: str):
    this.type = type

# shape1 is a Shape whose type is "Square"
shape1 = Shape("Square")

assert(shape1, { "id": "shape1", "type": "Square" })

---

# Nucleoid creates a class with a constructor and a typed attribute

# There is a Shape type,
# which has a type as a string
class Shape:
    type: str

    def init(type: str):
        this.type = type

# shape1 is a Shape whose type is "Rectangle"
shape1 = Shape("Rectangle")

assert(shape1, { "id": "shape1", "type": "Rectangle" })

---

# Nucleoid adds an object to the class's object list

# There is a Student type
class Student

# user0 is a Student
user0 = Student()

assert(Student.find(student => student.id == "user0"), { "id": "user0" })
assert(Student["user0"], { "id": "user0" })

---

# Nucleoid preserves class and object lists when a class is updated

# There is a User type
class User

# There is a User
User()

assert(Class.length, 1)
assert(User.length, 1)

# There is a User type
class User

assert(Class.length, 1)
assert(User.length, 1)

# There is a User
User()

assert(Class.length, 1)
assert(User.length, 2)

---

# Nucleoid places an instance in the list of the class when created

# There is a Student type
class Student

assert(typeof Student, List)

# student1 is a Student
student1 = Student()

assert(Student.length, 1)

---

# Nucleoid creates a class and a subclass

# There is a Person type,
# which has a name as a string
class Person(name: str):
    this.name = name

# There is a Student type,
# which is a subtype of Person
# and has a school as a string
class Student: Person
    def init name school:
        super(name)
        this.name = name
        this.school = school

# student1 is a Student,
# whose name is "Emma"
# and whose school is "Riverside High"
student1 = Student("Emma", "Riverside High")

assert(student1, { "id": "student1", "name": "Emma", "school": "Riverside High" })

---

# Nucleoid runs a class-level property assignment

# There is a Human type,
# which has a name as a string
class Human(name: str):
    this.name = name

# All humans are mortal
$Human.mortal = true

# human1 is a Human whose name is "Socrates"
human1 = Human("Socrates")

assert(human1.mortal, true)

---

# Nucleoid runs a class-level conditional

# There is a Device type,
# which has a profile as a string
class Device(profile: str):
    this.profile = profile

# Any device that has a profile is active
if $Device.profile:
    $Device.active = true

# device1 has no profile
device1 = Device()

# device2 has profile "PROFILE-1"
device2 = Device("PROFILE-1")

assert(device1.active, null)
assert(device2.active, true)

---

# Nucleoid creates an instance in a block and assigns it to a property

# There is a Room type
class Room

# There is a Meeting type
class Meeting

# room1 is a Room
room1 = Room()

# Any meeting's time is now plus " @ " plus the meeting's date as a date string
$Meeting.time = Date.now() + " @ " + $Meeting.date.toDateString()

# meeting is a Meeting whose date is January 1, 2020,
# and room1's meeting is meeting
{
    meeting = Meeting()
    meeting.date = Date("2020-1-1")
    room1.meeting = meeting
}

assert(room1.meeting.date.toDateString(), "Wed Jan 01 2020")
assert(room1.meeting.time[-17:], "@ Wed Jan 01 2020")

---

# Nucleoid creates nested instances in a block and assigns them to a property

# There is a Timesheet type
class Timesheet

# There is a Task type
class Task

# There is a Project type
class Project

# Any project's code is "N-" plus the project's number
$Project.code = "N-" + $Project.number

# timesheet1 is a Timesheet
timesheet1 = Timesheet()

# task is a Task whose project is a Project whose number is 3668347,
# and timesheet1's task is task
{
    task = Task()
    task.project = Project()
    task.project.number = 3668347
    timesheet1.task = task
}

assert(timesheet1.task.project.number, 3668347)
assert(timesheet1.task.project.code, "N-3668347")

---

# Nucleoid creates a local variable in a block and uses in assignment

# integer is 30
integer = 30

# equivalency is null
equivalency = null

# while in the block, division is a local variable that is integer divided by 10,
# and equivalency is division times 10
{
    division = integer / 10
    equivalency = division * 10
}

assert(equivalency, 30)

# integer is 40
integer = 40

assert(equivalency, 40)

---

# Nucleoid creates a standard built-in object as a local variable inside a block

# while in the block, f is a local Boolean that is false,
# and condition is f
{
    f = Boolean(false)
    condition = f
}

assert(condition, false)

---

# Nucleoid creates and assigns an instance to a local variable inside a block

# There is a Device type
class Device

# Any device's renew is the device's creation time plus 604800000
$Device.renew = $Device.created + 604800000

# device is a Device whose creation time is now
{
    device = Device()
    device.created = Date.now()
}

assert(Device[0].renew - Device[0].created, 604800000)

---

# Nucleoid creates and assigns an instance with a constructor to a local variable inside a block

# There is a Member type,
# which has a first as a string
# and a last as a string
class Member(first: str, last: str):
    this.first = first
    this.last = last

# Any member's display is the member's last plus ", " plus the member's first
$Member.display = $Member.last + ", " + $Member.first

# member is a Member whose first is "First" and whose last is "Last"
{
    member = Member("First", "Last")
}

assert(Member[0].display, "Last, First")

---

# Nucleoid creates an object in a block and assigns it to a class-level property before instantiation

# There is a Member type
class Member

# registration is an Object whose date is January 2, 2019,
# and any member's registration is registration
{
    registration = Object()
    registration.date = Date("2019-1-2")
    $Member.registration = registration
}

# member1 is a Member
member1 = Member()

assert(member1.registration.date.toDateString(), "Wed Jan 02 2019")
assert(member1.registration.age, null)

---

# Nucleoid creates an object in a block and assigns it to a class-level property after instantiation

# There is a Distance type
class Distance

# distance1 is a Distance
distance1 = Distance()

# location is an Object whose coordinates is "40.6976701,-74.2598779",
# and any distance's starting point is location
{
    location = Object()
    location.coordinates = "40.6976701,-74.2598779"
    $Distance.startingPoint = location
}

assert(distance1.startingPoint.coordinates, "40.6976701,-74.2598779")
assert(distance1.startingPoint.print, null)

---

# Nucleoid calls function in an assignment

# multiply returns the product of two factors
def multiply first_factor second_factor:
    product = first_factor * second_factor
    return product

# x is 1
x = 1

# y is 2
y = 2

# z is the result of the multiply function call with x and y, plus 1
z = multiply(x, y) + 1

assert(z, 3)

---

# Nucleoid assigns a block in a function as a dependency

# There is a Student type
class Student

# student1 is a Student whose age is 7
student1 = Student()
student1.age = 7

# student2 is a Student whose age is 8
student2 = Student()
student2.age = 8

# student3 is a Student whose age is 9
student3 = Student()
student3.age = 9

# age is 8
age = 8

# student is the Student whose age is age
student = Student.find(s => s.age == age)

assert(student, student2)
assert(student, { "id": "student2", "age": 8 })

# age is 9
age = 9

assert(student, student3)
assert(student, { "id": "student3", "age": 9 })

---

# Nucleoid supports chained functions with a parameter in an expression

# There is a Result type,
# which has a score as a number
class Result(score: int):
    this.score = score

# There are Results whose scores are 10, 15 and 20
Result(10); Result(15); Result(20)

# upperThreshold is 18
upperThreshold = 18

# lowerThreshold is 12
lowerThreshold = 12

# list is Results whose score is greater than lowerThreshold,
# filtered to those whose score is less than upperThreshold
list = Result.filter(r => r.score > lowerThreshold).filter(r => r.score < upperThreshold)

assert(list.length, 1)
assert(list[0].score, 15)

# lowerThreshold is 7
lowerThreshold = 7

assert(list.length, 2)
assert(list[0].score, 10)
assert(list[1].score, 15)

# upperThreshold is 14
upperThreshold = 14

assert(list.length, 1)
assert(list[0].score, 10)

---

# Nucleoid supports an array with brackets

# states is a list of "NY", "GA", "CT" and "MI"
states = ["NY", "GA", "CT", "MI"]

# The value of states at index 2
states[2]

# return: "CT"

---

# Nucleoid throws an error if a variable in an expression is not defined

try:
    # whether e is equal to 2.71828
    e == 2.71828
catch error:
    assert(error, ReferenceError("e is not defined"))

---

# Nucleoid retrieves the value of a variable

# number is -1
number = -1

number

# return: -1

---

# Nucleoid creates a property assignment on a local variable only if the instance is defined

# There is a Ticket type
class Ticket

try:
    # while in the block, ticket is a local Ticket,
    # and ticket's event's group is "ENTERTAINMENT"
    {
        ticket = Ticket()
        ticket.event.group = "ENTERTAINMENT"
    }
catch error:
    assert(error, ReferenceError("ticket.event is not defined"))

---

# Nucleoid declares a local variable as undefined

# There is a Device type,
# which has a code as a string
class Device(code: str):
    this.code = code

# device1 is a Device whose code is "A0"
device1 = Device("A0")

# device2 is a Device whose code is "B1"
device2 = Device("B1")

# While in the block, device is a local variable that is the Device whose code is "A0",
# and if there is no device, then throw "INVALID_DEVICE",
# and return device
{
    device = Device.find(d => d.code == "A0")
    if not device:
        throw "INVALID_DEVICE"
    return device
}

# return: { "id": "device1", "code": "A0" }

---

# Nucleoid rejects a local variable declared as undefined

# There is a Device type,
# which has a code as a string
class Device(code: str):
    this.code = code

# device1 is a Device whose code is "A0"
device1 = Device("A0")

# device2 is a Device whose code is "B1"
device2 = Device("B1")

try:
    # While in the block, device is a local variable that is the Device whose code is "A1",
    # and if there is no device, then throw "INVALID_DEVICE"
    {
        device = Device.find(d => d.code == "A1")
        if not device:
            throw "INVALID_DEVICE"
        return device
    }
catch error:
    assert(error, "INVALID_DEVICE")

---

# Nucleoid creates a standard built-in object as a property of a local variable

# There is a Shipment type
class Shipment

# While in the block, shipment is a local Shipment whose date is January 3, 2019,
# and shipment1 is shipment
{
    shipment = Shipment()
    shipment.date = Date("2019-1-3")
    shipment1 = shipment
}

assert(shipment1.date.toDateString(), "Thu Jan 03 2019")

---

# Nucleoid creates a property of a local variable in a different scope

# There is a User type
class User

# user0 is a User
user0 = User()

# While in the block, user is a local variable that is the User whose id is "user0",
# and if there is a user, then the user's name is "TEST"
{
    user = User["user0"]
    if user:
        user.name = "TEST"
}

assert(user0.name, "TEST")

---

# Nucleoid assigns a variable declaratively

# a is 1
a = 1

# b is 2
b = 2

# c is sum of a and b
c = a + b

assert(c, 3)

# a is 2
a = 2

assert(c, 4)

---

# Nucleoid creates if statement of variable

# m is false
m = false

# n is false
n = false

# if m is true, then n is m and true
if m == true:
    n = m and true

assert(n, false)

# m is true
m = true

assert(n, true)

---

# Nucleoid updates if block of variable

# p is 0.01
p = 0.01

# s is 0.02
s = 0.02

# if p is less than 1, then r is p times 10
if p < 1:
    r = p * 10

# if p is less than 1, then r is s times 10
if p < 1:
    r = s * 10

assert(r, 0.2)

# s is 0.03
s = 0.03

assert(r, 0.3)

---

# Nucleoid creates else if statement of variable

# g is 11
g = 11

# earth is 9.8
earth = 9.8

# mars is 3.71
mars = 3.71

# mass is 10
mass = 10

# if g is greater than 9, then weight is earth times mass,
# else if g is greater than 3, then weight is mars times mass
if g > 9:
    weight = earth * mass
else if g > 3:
    weight = mars * mass

# g is 5
g = 5

assert(weight, 37.1)

# mars is 3.72
mars = 3.72

assert(weight, 37.2)

---

# Nucleoid creates multiple else if statement of variable

# fraction is -0.1
fraction = -0.1

# point is 1
point = 1

# if fraction is greater than 1, then score is fraction times point times 3,
# else if fraction is greater than 0, then score is fraction times point times 2,
# else score is fraction times point
if fraction > 1:
    score = fraction * point * 3
else if fraction > 0:
    score = fraction * point * 2
else
    score = fraction * point

assert(score, -0.1)

# point is 2
point = 2

assert(score, -0.2)

---

# Nucleoid runs dependent statements in the same transaction

# There is a Vehicle type,
# and any vehicle's tag is "US-" plus the vehicle's plate
class Vehicle

$Vehicle.tag = "US-" + $Vehicle.plate

# vehicle1 is a Vehicle
vehicle1 = Vehicle()

# vehicle1's plate is "XSJ422"
vehicle1.plate = "XSJ422"

assert(vehicle1.tag, "US-XSJ422")

---

# Nucleoid runs dependencies in order as received

# any is 0
any = 0

# if any is bigger than 1, then result is 1
if any > 1:
    result = 1

# if any is bigger than 2, then result is 2
if any > 2:
    result = 2

# if any is bigger than 3, then result is 3
if any > 3:
    result = 3

# if any is bigger than 2, then result is 4
if any > 2:
    result = 4

# if any is bigger than 1, then result is 5
if any > 1:
    result = 5

# any is 4
any = 4

assert(result, 5)

---

# Nucleoid searches a variable in scope before the state

# e is 2.71828
e = 2.71828

# number is null
number = null

# a local e shadows the outer e inside the block,
# and number is the local e
{
    e = 3
    number = e
}

assert(number, 3)

---

# Nucleoid uses local variable at lowest scope as priority

# pi is 3.14
pi = 3.14

# number is pi
number = pi

# a local pi shadows the outer pi inside the block
{
    pi = 3.141
    number = pi
}

assert(number, 3.141)

---

# Nucleoid assigns undefined if any dependency in expression is undefined

# There is a Person type
class Person

# person1 is a Person
person1 = Person()

# person1's last name is "Brown"
person1.lastName = "Brown"

# person1's full name is first and last name
person1.fullName = person1.firstName + " " + person1.lastName

assert(person1.fullName, null)

---

# Nucleoid keeps as null if any dependencies as in local is null

# a is 1
a = 1

# c is null
c = null

# while in the block, b is a local variable that is null,
# and c is b divided by a
{
    b = null
    c = b / a
}

assert(c, null)

---

# Nucleoid keeps as null if any dependencies in expression is null

# There is a Schedule type
class Schedule

# schedule1 is a Schedule whose expression is "0 */2 * * *" and whose script is null
schedule1 = Schedule()
schedule1.expression = "0 */2 * * *"
schedule1.script = null

# schedule1's run is schedule1's expression plus " " plus schedule1's script
schedule1.run = schedule1.expression + " " + schedule1.script

assert(schedule1.run, null)

---

# Nucleoid assigns null if there is null pointer in expression

# There is a Product type
class Product

# product1 is a Product
product1 = Product()

# score is product1's quality score
score = product1.quality.score

assert(score, null)

---

# Nucleoid assigns a unique variable for an instance without a variable name

# There is a Vehicle type
class Vehicle

# There is a Vehicle
Vehicle()

assert(Vehicle.length, 1)
assert(Vehicle[0].id != null, true)

---

# Nucleoid creates a function in state
# generate returns number times 10
# generate returns number times 10
def generate number => number * 10

# random is 10
random = 10

# number is the result of the generate function call with random
number = generate(random)

assert(number, 100)

# random is 20
random = 20

assert(number, 200)

---

# Nucleoid assigns a function as a dependency

# list is an empty list
list = []

# count is the list filtered to odd numbers
count = list.filter(n => n % 2)

# Add 1 to the list
list.push(1)

assert(count.length, 1)

# Add 2 to the list
list.push(2)

assert(count.length, 1)

# Add 3 to the list
list.push(3)

assert(count.length, 2)

# Remove the last item from the list
list.pop()

assert(count.length, 1)

---

# Nucleoid supports a regular expression literal

# There is a User type
class User

# If any user's password does not match /.{4,8}/, then throw 'INVALID_PASSWORD'
if not /.{4,8}/.test($User.password):
    throw 'INVALID_PASSWORD'

# user1 is a User
user1 = User()

assert(user1.password, null)

try:
    # user1's password is 'PAS'
    user1.password = 'PAS'
catch error:
    assert(error, "INVALID_PASSWORD")

---

# Nucleoid rejects defining a class declaration in a non-class block

# There is a Person type
class Person

# person1 is a Person whose weight is 90 and whose height is 1.8
person1 = Person()
person1.weight = 90
person1.height = 1.8

try:
    # while in the block, weight is a local variable that is person1's weight,
    # and height is a local variable that is person1's height,
    # and any person's bmi is weight divided by height squared
    {
        weight = person1.weight
        height = person1.height
        $Person.bmi = weight / (height * height)
    }
catch error:
    assert(error, SyntaxError("Cannot define class declaration in non-class block"))

---

# Nucleoid detects a circular dependency

# number1 is 10
number1 = 10

# number2 is number1 times 10
number2 = number1 * 10

assert(number2, 100)

try:
    # number1 is number2 times 10
    number1 = number2 * 10
catch error:
    assert(error, TypeError("Circular Dependency"))

---

# Nucleoid rolls back a variable if an exception is thrown

# a is 5
a = 5

# if a is greater than 5, then throw 'INVALID_VALUE'
if a > 5:
    throw 'INVALID_VALUE'

try:
    # a is 6
    a = 6
catch error:
    assert(error, "INVALID_VALUE")

assert(a, 5)

---

# Nucleoid rolls back a property if an exception is thrown

# There is an Item type
class Item

# If any item's sku is 'A', then throw 'INVALID_SKU'
if $Item.sku == 'A':
    throw 'INVALID_SKU'

# item1 is an Item
item1 = Item()

try:
    # item1's sku is 'A'
    item1.sku = 'A'
catch error:
    assert(error, "INVALID_SKU")

assert(item1.sku, null)

---

# Nucleoid rolls back an instance if an exception is thrown

# There is a User type,
# which has a first as a string
# and a last as a string
class User(first: str, last: str):
    this.first = first
    this.last = last

# If any user's first is shorter than 3 characters, then throw 'INVALID_USER'
if $User.first.length < 3:
    throw 'INVALID_USER'

try:
    # user1 is a User whose first is 'F' and whose last is 'L'
    user1 = User('F', 'L')
catch error:
    assert(error, "INVALID_USER")

assert(User.length, 0)

try:
    # user1
    user1
catch error:
    assert(error, ReferenceError("user1 is not defined"))

---

# Nucleoid updates a variable assignment

# a is 1
a = 1

# b is 2
b = 2

# c is a plus 3
c = a + 3

assert(c, 4)

# c is b plus 3
c = b + 3

assert(c, 5)

# b is 4
b = 4

assert(c, 7)

---

# Nucleoid uses only the value when a variable references itself

# radius is 10
radius = 10

# radius is radius plus 10
radius += 10

assert(radius, 20)

---

# Nucleoid deletes a variable assignment

# t is 1
t = 1

# q is t plus 1
q = t + 1

assert(q, 2)

# q is deleted
delete q

# t is 2
t = 2

try:
    # q
    q
catch error:
    assert(error, ReferenceError("q is not defined"))

---

# Nucleoid returns the assigned value in a variable assignment

# x is 1
x = 1

# return: 1

---

# Nucleoid assigns a parameter in a function as a dependency

# str1 is "ABC"
str1 = "ABC"

# str2 is str1 lowercased plus "d"
str2 = str1.lower() + "d"

# str3 is str2 concatenated with str1
str3 = str2 + str1

assert(str2, "abcd")
assert(str3, "abcdABC")

# str1 is "AAA"
str1 = "AAA"

assert(str2, "aaad")
assert(str3, "aaadAAA")

---

# Nucleoid uses value property to indicate using only value of variable

# goldenRatio is 1.618
goldenRatio = 1.618

# altitude is 10
altitude = 10

# width is goldenRatio's value times altitude
width = goldenRatio.value * altitude

# depth is goldenRatio's value times altitude
depth = goldenRatio.value * altitude

assert(width, 16.18)
assert(depth, 16.18)

# goldenRatio is 1.62
goldenRatio = 1.62

assert(width, 16.18)
assert(depth, 16.18)

# altitude is 100
altitude = 100

assert(width, 161.8)
assert(depth, 161.8)

---

# Nucleoid creates a nested object in a block and assigns it to a class-level property before instantiation

# There is an Account type
class Account

# There is a Currency type
class Currency

# Any currency's description is "Code:" plus the currency's code
$Currency.description = "Code:" + $Currency.code

# balance is an Object whose currency is an Object whose code is "USD",
# and any account's balance is balance
{
    balance = Object()
    balance.currency = Object()
    balance.currency.code = "USD"
    $Account.balance = balance
}

# account1 is an Account
account1 = Account()

assert(account1.balance.currency.code, "USD")
assert(account1.balance.currency.description, null)

---

# Nucleoid creates a nested object in a block and assigns it to a class-level property after instantiation

# There is a Warehouse type
class Warehouse

# warehouse1 is a Warehouse
warehouse1 = Warehouse()

# inventory is an Object whose item is an Object whose sku is "699546085767",
# and any warehouse's inventory is inventory
{
    inventory = Object()
    inventory.item = Object()
    inventory.item.sku = "699546085767"
    $Warehouse.inventory = inventory
}

assert(warehouse1.inventory.item.sku, "699546085767")
assert(warehouse1.inventory.item.description, null)

---

# Nucleoid creates an instance inside a block

# There is a Device type,
# which has a name as a string
class Device(name: str):
    this.name = name

# No device is deleted
$Device.deleted = false

# Any device's key is "X-" plus the device's name
$Device.key = "X-" + $Device.name

# name is "Hall",
# and device1 is a Device whose name is name
{
    name = "Hall"
    device1 = Device(name)
}

assert(device1.name, "Hall")
assert(device1.key, "X-Hall")
assert(device1.deleted, false)

---

# Nucleoid creates an instance inside a block without a variable name

# There is a Summary type,
# which has a rate as a number
class Summary(rate: int):
    this.rate = rate

# Any summary's score is the summary's rate times 100
$Summary.score = $Summary.rate * 100

# rate is 4,
# and there is a Summary whose rate is rate
{
    rate = 4
    Summary(rate)
}

assert(Summary[0].rate, 4)
assert(Summary[0].score, 400)

---

# Nucleoid creates a local variable inside a block

# a is 5
a = 5

# b is 10
b = 10

# if a is greater than 9, then while in the block, c is a plus b,
# and d is c times 10
if a > 9:
    c = a + b
    d = c * 10

# a is 10
a = 10

assert(d, 200)

# a is 15
a = 15

assert(d, 250)

# b is 20
b = 20

assert(d, 350)

---

# Nucleoid runs a local variable as an object before declaration

# There is a Plane type
class Plane

# There is a Trip type
class Trip

# plane1 is a Plane whose speed is 903
plane1 = Plane()
plane1.speed = 903

# trip1 is a Trip whose distance is 5540
trip1 = Trip()
trip1.distance = 5540

# while in the block trip is any plane's trip,
# and any plane's time is trip's distance divided by the plane's speed
{
    trip = $Plane.trip
    $Plane.time = trip.distance / $Plane.speed
}

# plane1's trip is trip1
plane1.trip = trip1

assert(plane1.time, 6.135105204872647)

---

# Nucleoid runs a local variable as an object after declaration

# There is a Seller type
class Seller

# There is a Commission type
class Commission

# seller1 is a Seller whose sales is 1000000
seller1 = Seller()
seller1.sales = 1000000

# comm1 is a Commission whose rate is 0.05
comm1 = Commission()
comm1.rate = 0.05

# seller1's commission is comm1
seller1.commission = comm1

# While in the block, commission is any seller's commission,
# and any seller's pay is the seller's sales times commission's rate
{
    commission = $Seller.commission
    $Seller.pay = $Seller.sales * commission.rate
}

assert(seller1.pay, 50000)

---

# Nucleoid assigns a property on a local variable after initialization

# There is a Stock type
class Stock

# There is a Trade type
class Trade

# stock1 is a Stock whose price is 100
stock1 = Stock()
stock1.price = 100

# trade1 is a Trade whose quantity is 1
trade1 = Trade()
trade1.quantity = 1

# stock1's trade is trade1
stock1.trade = trade1

# While in the block, trade is any stock's trade,
# and trade's worth is the stock's price times trade's quantity
{
    trade = $Stock.trade
    trade.worth = $Stock.price * trade.quantity
}

assert(trade1.worth, 100)

---

# Nucleoid reassigns a shadowing local variable in a nested block

# barcode is "barcode"
barcode = "barcode"

# a local barcode shadows the outer barcode inside the block,
# and a nested block reassigns the local barcode to "barcode2"
{
    barcode = "barcode"
    {
        barcode = "barcode2"
        {
            assert(barcode, "barcode2")
        }
    }
}

assert(barcode, "barcode")

---

# Nucleoid holds the result of a function in a local variable

# bugs is an empty list
bugs = []

# ticket is 1
ticket = 1

# There is a Bug type
class Bug

# bug1 is a Bug whose ticket is 1 and whose priority is "LOW",
# and bug1 is added to bugs
bug1 = Bug()
bug1.ticket = 1
bug1.priority = "LOW"
bugs.push(bug1)

# bug2 is a Bug whose ticket is 2 and whose priority is "MEDIUM",
# and bug2 is added to bugs
bug2 = Bug()
bug2.ticket = 2
bug2.priority = "MEDIUM"
bugs.push(bug2)

# While in the block, bug is the bug in bugs whose ticket is ticket,
# and bug is selected
{
    bug = bugs.find(b => b.ticket == ticket)
    bug.selected = true
}

assert(bug1.selected, true)
assert(bug2.selected, null)

# ticket is 2
ticket = 2

assert(bug2.selected, true)

---

# Nucleoid runs a block statement of variable

# h is 1
h = 1

# while in the block, value is a local variable that is h times 2,
# and j is value times 2
{
    value = h * 2
    j = value * 2
}

assert(j, 4)

# h is 2
h = 2

assert(j, 8)

---

# Nucleoid runs a nested block statement of variable

# radius is 10
radius = 10

# while in the block, area is a local variable that is radius squared times 3.14,
# and in a nested block, volume is area times 5
{
    area = Math.pow(radius, 2) * 3.14
    {
        volume = area * 5
    }
}

assert(volume, 1570)

---

# Nucleoid runs a nested if statement of variable

# gravity is 9.8
gravity = 9.8

# time is 10
time = 10

# distance is 480
distance = 480

# target is true
target = true

# while in the block, dist is a local variable that is one half times gravity times time times time,
# and if dist is greater than distance, then hit is target
{
    dist = 1 / 2 * gravity * time * time
    if dist > distance:
        hit = target
}

assert(hit, true)

# target is false
target = false

assert(hit, false)

---

# Nucleoid runs a nested else statement of variable

# percentage is 28
percentage = 28

# density is 0.899
density = 0.899

# substance is "NH3"
substance = "NH3"

# molarConcentration is null
molarConcentration = null

# fallback is 0
fallback = 0

# while in the block, concentration is a local variable that is percentage times density divided by 100 times 1000,
# and if substance is "NH3", then molarConcentration is concentration divided by 17.04,
# else molarConcentration is fallback
{
    concentration = percentage * density / 100 * 1000
    if substance == "NH3":
        molarConcentration = concentration / 17.04
    else
        molarConcentration = fallback
}

# substance is "NH16"
substance = "NH16"

# fallback is 1
fallback = 1

assert(molarConcentration, 1)

---

# Nucleoid assigns a variable to a reference

# a is 1
a = 1

# b is a
b = a

assert(b, 1)

# a is 2
a = 2

assert(b, 2)

---

# Nucleoid assigns an object to a variable

# There is a Model type
class Model

# model1 is a Model
model1 = Model()

assert(typeof model1, Object)

---

# Nucleoid defines a class in the state

# There is an Entity type
class Entity

assert(typeof $Entity, Class)

---

# Nucleoid rejects creating an instance if the class does not exist

try:
    # chart1 is a Chart
    chart1 = Chart()
catch error:
    assert(error, ReferenceError("Chart is not defined"))

# There is a Chart type
class Chart

# chart1 is a Chart
chart1 = Chart()

try:
    # chart1's plot is a Plot
    chart1.plot = Plot()
catch error:
    assert(error, ReferenceError("Plot is not defined"))

try:
    # any chart's plot is a Plot
    $Chart.plot = Plot()
catch error:
    assert(error, ReferenceError("Plot is not defined"))

---

# Nucleoid creates a property assignment before declaration

# There is an Order type
class Order

# order1 is an Order
order1 = Order()

# order1's upc is "04061" plus order1's barcode
order1.upc = "04061" + order1.barcode

assert(order1.upc, null)

# order1's barcode is "94067"
order1.barcode = "94067"

assert(order1.upc, "0406194067")

---

# Nucleoid creates a property assignment after declaration

# There is a User type
class User

# user1 is a User
user1 = User()

# user1's name is "sample"
user1.name = "sample"

# user1's email is user1's name plus "@example.com"
user1.email = user1.name + "@example.com"

assert(user1.email, "sample@example.com")

# user1's name is "samplex"
user1.name = "samplex"

assert(user1.email, "samplex@example.com")

---

# Nucleoid creates a property assignment only if the instance is defined

# There is a Channel type
class Channel

# channel1 is a Channel
channel1 = Channel()

try:
    # channel1's frequency's type is "ANGULAR"
    channel1.frequency.type = "ANGULAR"
catch error:
    assert(error, ReferenceError("channel1.frequency is not defined"))

---

# Nucleoid creates an object and assigns it to a variable

# There is an Item type,
# which has a name as a string
class Item(name: str):
    this.name = name

# item1 is an Item whose name is "NAME-1"
item1 = Item("NAME-1")

assert(item1, { "id": "item1", "name": "NAME-1" })

# item2 is an Item with no name
item2 = Item()

assert(item2, { "id": "item2", "name": null })

---

# Nucleoid creates an object assignment as a property only if the instance is defined

# There is a Worker type
class Worker

# There is a Schedule type
class Schedule

# worker1 is a Worker
worker1 = Worker()

try:
    # worker1's duty's schedule is a Schedule
    worker1.duty.schedule = Schedule()
catch error:
    assert(error, ReferenceError("worker1.duty is not defined"))

---

# Nucleoid uses only the value when a property references itself

# There is a Construction type
class Construction

# construction1 is a Construction
construction1 = Construction()

# construction1's timeline is 120
construction1.timeline = 120

# construction1's timeline is 2 times construction1's timeline
construction1.timeline = 2 * construction1.timeline

assert(construction1.timeline, 240)

---

# Nucleoid assigns an object to a property before initialization

# There is an Agent type
class Agent

# There is a Distance type
class Distance

# Any distance's total is the square root of the distance's x squared plus the distance's y squared
$Distance.total = Math.sqrt($Distance.x * $Distance.x + $Distance.y * $Distance.y)

# agent1 is an Agent
agent1 = Agent()

# agent1's distance is a Distance
agent1.distance = Distance()

assert(agent1.distance.total, null)

# agent1's distance's x is 3 and its y is 4
agent1.distance.x = 3
agent1.distance.y = 4

assert(agent1.distance.total, 5)

---

# Nucleoid assigns an object to a property after initialization

# There is a Product type
class Product

# product1 is a Product
product1 = Product()

# There is a Quality type
class Quality

# product1's quality is a Quality whose score is 15
product1.quality = Quality()
product1.quality.score = 15

assert(product1.quality.class, null)

# Any quality's class is the character with the code 65 plus the floor of the quality's score divided by 10
$Quality.class = String.fromCharCode(65 + Math.floor($Quality.score / 10))

assert(product1.quality.class, "B")

---

# Nucleoid rejects value as a property name

# There is a Schedule type
class Schedule

# There is a Place type
class Place

# value is a Schedule
value = Schedule()

assert(value, { "id": "value" })

try:
    # value's value is a Place
    value.value = Place()
catch error:
    assert(error, TypeError("Cannot use 'value' as a property"))

---

# Nucleoid rejects value as a property name in a value assignment

# There is a Value type
class Value

# value is a Value
value = Value()

assert(value, { "id": "value" })

try:
    # value's value is 2147483647
    value.value = 2147483647
catch error:
    assert(error, TypeError("Cannot use 'value' as a property"))

---

# Nucleoid uses value property to indicate using only value of property

# There is a Weight type
class Weight

# weight1 is a Weight whose gravity is 1.352 and whose mass is 1000
weight1 = Weight()
weight1.gravity = 1.352
weight1.mass = 1000

# weight1's force is weight1's gravity times weight1's mass's value
weight1.force = weight1.gravity * weight1.mass.value

assert(weight1.force, 1352)

# weight1's mass is 2000
weight1.mass = 2000

assert(weight1.force, 1352)

# weight1's gravity is 2
weight1.gravity = 2

assert(weight1.force, 2000)

---

# Nucleoid uses value property in an if condition to indicate using only value of property

# There is a Question type
class Question

# question1 is a Question whose text is "How was the service?"
question1 = Question()
question1.text = "How was the service?"

# if question1's text is not question1's text's value, then throw "QUESTION_ARCHIVED"
if question1.text != question1.text.value:
    throw "QUESTION_ARCHIVED"

assert(question1.text, "How was the service?")

try:
    # question1's text is "How would you rate us?"
    question1.text = "How would you rate us?"
catch error:
    assert(error, "QUESTION_ARCHIVED")

---

# Nucleoid rejects value of a property if the property is not defined

# There is a Travel type
class Travel

# travel1 is a Travel whose speed is 65
travel1 = Travel()
travel1.speed = 65

# travel1's duration is travel1's distance divided by travel1's speed
travel1.duration = travel1.distance / travel1.speed

assert(travel1.duration, null)

try:
    # travel1's time is travel1's distance's value divided by travel1's speed
    travel1.time = travel1.distance.value / travel1.speed
catch error:
    assert(error, ReferenceError("travel1.distance is not defined"))

---

# Nucleoid uses the value of a null property as zero

# There is an Interest type
class Interest

# interest1 is an Interest whose rate is 3 and whose amount is null
interest1 = Interest()
interest1.rate = 3
interest1.amount = null

# interest1's annual is interest1's rate times interest1's amount's value divided by 100
interest1.annual = interest1.rate * interest1.amount.value / 100

assert(interest1.annual, 0)

# interest1's amount is 10000
interest1.amount = 10000

assert(interest1.annual, 0)

---

# Nucleoid rejects value as a property name in a block

# There is an Alarm type
class Alarm

try:
    # while in the block, value is a local Alarm,
    # and value's value is "22:00"
    {
        value = Alarm()
        value.value = "22:00"
    }
catch error:
    assert(error, TypeError("Cannot use 'value' as a property"))

---

# Nucleoid keeps same as its value when the value property is used for a local

# speedOfLight is 299792
speedOfLight = 299792

# roundTrip is null
roundTrip = null

# while in the block, time is a local variable that is speedOfLight divided by 225623,
# and roundTrip is time's value times 2
{
    time = speedOfLight / 225623
    roundTrip = time.value * 2
}

assert(roundTrip, 2.6574595675086314)

---

# Nucleoid uses value property in a class-level assignment

# count is 0
count = 0

# There is a Device type
class Device

# device1 is a Device
device1 = Device()

# while in the block, any device's code is "A" plus count's value,
# and count is count plus 1
{
    $Device.code = "A" + count.value
    count += 1
}

assert(device1.code, "A0")

---

# Nucleoid uses value property on a class-level property chain

# There is a Summary type,
# which has a question as a Question
class Summary(question):
    this.question = question

# There is a Question type
class Question

# Any summary's count is the value of the summary's question's count
$Summary.count = $Summary.question.count.value

# question1 is a Question whose count is 10
question1 = Question()
question1.count = 10

# summary1 is a Summary whose question is question1
summary1 = Summary(question1)

assert(summary1.count, 10)

# question1's count is 11
question1.count = 11

assert(question1.count, 11)
assert(summary1.count, 10)

---

# Nucleoid updates if block of property

# There is an Account type
class Account

# account1 is an Account whose balance is 1000
account1 = Account()
account1.balance = 1000

# if account1's balance is less than 1500, then account1's status is "OK"
if account1.balance < 1500:
    account1.status = "OK"

assert(account1.status, "OK")

# if account1's balance is less than 1500, then account1's status is "LOW"
if account1.balance < 1500:
    account1.status = "LOW"

assert(account1.status, "LOW")

---

# Nucleoid creates an else statement of variable

# compound is 0.0001
compound = 0.0001

# acidic is 'ACIDIC'
acidic = 'ACIDIC'

# basic is 'BASIC'
basic = 'BASIC'

# if compound is greater than 0.0000001, then pH is acidic,
# else pH is basic
if compound > 0.0000001:
    pH = acidic
else
    pH = basic

assert(pH, "ACIDIC")

# compound is 0.000000001
compound = 0.000000001

assert(pH, "BASIC")

# basic is '+7'
basic = '+7'

assert(pH, "+7")

---

# Nucleoid creates if statement of property

# There is a Toy type
class Toy

# toy1 is a Toy whose color is "BLUE"
toy1 = Toy()
toy1.color = "BLUE"

# if toy1's color is "RED", then toy1's shape is "CIRCLE"
if toy1.color == "RED":
    toy1.shape = "CIRCLE"

assert(toy1.shape, null)

# toy1's color is "RED"
toy1.color = "RED"

assert(toy1.shape, "CIRCLE")

---

# Nucleoid creates else statement of property

# There is an Engine type
class Engine

# engine1 is an Engine whose type is "V8"
engine1 = Engine()
engine1.type = "V8"

# mpl is "MPL"
mpl = "MPL"

# bsd is "BSD"
bsd = "BSD"

# if engine1's type is "Gecko", then engine1's license is mpl,
# else engine1's license is bsd
if engine1.type == "Gecko":
    engine1.license = mpl
else
    engine1.license = bsd

assert(engine1.license, "BSD")

# bsd is "Berkeley Software Distribution"
bsd = "Berkeley Software Distribution"

assert(engine1.license, "Berkeley Software Distribution")

# engine1's type is "Gecko"
engine1.type = "Gecko"

assert(engine1.license, "MPL")

---

# Nucleoid creates else statement of property with property dependencies

# There is a Contact type
class Contact

# contact1 is a Contact whose type is "PERSON",
# whose first is "First" and whose last is "Last"
contact1 = Contact()
contact1.type = "PERSON"
contact1.first = "First"
contact1.last = "Last"

# if contact1's type is "BUSINESS", then contact1's full is "B" plus contact1's first,
# else contact1's full is contact1's first plus " " plus contact1's last
if contact1.type == "BUSINESS":
    contact1.full = "B" + contact1.first
else
    contact1.full = contact1.first + " " + contact1.last

assert(contact1.full, "First Last")

# contact1's first is "F" and contact1's last is "L"
contact1.first = "F"
contact1.last = "L"

assert(contact1.full, "F L")

# contact1's type is "BUSINESS"
contact1.type = "BUSINESS"

assert(contact1.full, "BF")

---

# Nucleoid creates multiple else if statement of property

# There is a Taxpayer type
class Taxpayer

# taxpayer1 is a Taxpayer whose income is 60000 and whose member is 1
taxpayer1 = Taxpayer()
taxpayer1.income = 60000
taxpayer1.member = 1

# rate is 22
rate = 22

# if taxpayer1's member is greater than 4, then taxpayer1's tax is taxpayer1's income times rate divided by 100 minus 2000,
# else if taxpayer1's member is greater than 2, then taxpayer1's tax is taxpayer1's income times rate divided by 100 minus 1000,
# else taxpayer1's tax is taxpayer1's income times rate divided by 100
if taxpayer1.member > 4:
    taxpayer1.tax = taxpayer1.income * rate / 100 - 2000
else if taxpayer1.member > 2:
    taxpayer1.tax = taxpayer1.income * rate / 100 - 1000
else
    taxpayer1.tax = taxpayer1.income * rate / 100

assert(taxpayer1.tax, 13200)

# rate is 23
rate = 23

assert(taxpayer1.tax, 13800)

# taxpayer1's member is 3
taxpayer1.member = 3

assert(taxpayer1.tax, 12800)

# taxpayer1's member is 5
taxpayer1.member = 5

assert(taxpayer1.tax, 11800)

---

# Nucleoid updates property assignment

# There is a Matter type
class Matter

# matter1 is a Matter whose mass is 10
matter1 = Matter()
matter1.mass = 10

# matter1's weight is matter1's mass times 9.8
matter1.weight = matter1.mass * 9.8

assert(matter1.weight, 98)

# matter1's weight is matter1's mass times 3.7
matter1.weight = matter1.mass * 3.7

assert(matter1.weight, 37)

# matter1's mass is 20
matter1.mass = 20

assert(matter1.weight, 74)

---

# Nucleoid deletes an instance

# There is a Circle type
class Circle

# circle1 is a Circle
circle1 = Circle()

# circle1 is deleted
delete circle1

assert(Circle["circle1"], null)
assert(Circle.find(circle => circle.id == "circle1"), null)

try:
    # circle1
    circle1
catch error:
    assert(error, ReferenceError("circle1 is not defined"))

---

# Nucleoid deletes an instance by reference

# There is an Item type
class Item

# item1 is an Item
item1 = Item()

# item2 is an Item
item2 = Item()

assert(Item["item1"], { "id": "item1" })

# The Item whose id is "item1" is deleted
delete Item["item1"]

assert(Item["item1"], null)
assert(Item["item2"], { "id": "item2" })

# While in the block, item is a local variable that is "item2",
# and the Item whose id is item is deleted
{
    item = "item2"
    delete Item[item]
}

assert(Item["item2"], null)

---

# Nucleoid returns a boolean when deleting an object

# There is a Location type
class Location

# location1 is a Location
location1 = Location()

# Deleting location1 returns true
assert(delete location1, true)

# Deleting location2, which is not defined, returns false
assert(delete location2, false)

---

# Nucleoid rejects deleting an instance if it has any properties

# There is a Channel type
class Channel

# channel1 is a Channel
channel1 = Channel()

# channel1's frequency is 440
channel1.frequency = 440

try:
    # channel1 is deleted
    delete channel1
catch error:
    assert(error, TypeError("Cannot delete object 'channel1'"))

assert(channel1.frequency, 440)

# channel1's frequency is deleted
delete channel1.frequency

# channel1 is deleted
delete channel1

assert(Channel["channel1"], null)

---

# Nucleoid rejects deleting an instance if it has an object as a property

# There is a Shape type
class Shape

# There is a Type type
class Type

# shape1 is a Shape
shape1 = Shape()

# shape1's type is a Type
shape1.type = Type()

try:
    # shape1 is deleted
    delete shape1
catch error:
    assert(error, TypeError("Cannot delete object 'shape1'"))

# shape1's type is deleted
delete shape1.type

# shape1 is deleted
delete shape1

assert(Shape["shape1"], null)

---

# Nucleoid deletes a property assignment

# There is an Agent type
class Agent

# agent is an Agent
agent = Agent()

# agent's time is 52926163455
agent.time = 52926163455

# agent's location is "CITY"
agent.location = "CITY"

# agent's report is agent's time plus "@" plus agent's location
agent.report = agent.time + "@" + agent.location

assert(agent.report, "52926163455@CITY")

# agent's time is deleted
delete agent.time

assert(agent.report, null)

# agent's report is deleted
delete agent.report

assert(agent.report, null)

---

# Nucleoid runs a block statement of property

# There is an Item type
class Item

# item1 is an Item
item1 = Item()

# item1's sku is "0000001"
item1.sku = "0000001"

# while in the block, custom is a local variable that is "US" plus item1's sku,
# and item1's custom is custom
{
    custom = "US" + item1.sku
    item1.custom = custom
}

assert(item1.custom, "US0000001")

# item1's sku is "0000002"
item1.sku = "0000002"

assert(item1.custom, "US0000002")

---

# Nucleoid runs a nested block statement of property

# There is a Figure type
class Figure

# figure1 is a Figure
figure1 = Figure()

# figure1's width is 9
figure1.width = 9

# figure1's height is 10
figure1.height = 10

# while in the block, base is a local variable that is figure1's width squared,
# and in a nested block, figure1's volume is base times figure1's height
{
    base = Math.pow(figure1.width, 2)
    {
        figure1.volume = base * figure1.height
    }
}

assert(figure1.volume, 810)

# figure1's height is 9
figure1.height = 9

assert(figure1.volume, 729)

---

# Nucleoid runs a nested if statement of property

# There is a Sale type
class Sale

# sale1 is a Sale
sale1 = Sale()

# sale1's price is 50
sale1.price = 50

# sale1's quantity is 2
sale1.quantity = 2

# while in the block, amount is a local variable that is sale1's price times sale1's quantity,
# and if amount is greater than 100, then sale1's tax is amount times 10 divided by 100
{
    amount = sale1.price * sale1.quantity
    if amount > 100:
        sale1.tax = amount * 10 / 100
}

assert(sale1.tax, null)

# sale1's quantity is 3
sale1.quantity = 3

assert(sale1.tax, 15)

---

# Nucleoid creates a nested else statement of property

# There is a Chart type
class Chart

# chart1 is a Chart
chart1 = Chart()

# chart1's percentage is 1
chart1.percentage = 1

# invalid is "INVALID"
invalid = "INVALID"

# valid is "VALID"
valid = "VALID"

# while in the block, ratio is a local variable that is chart1's percentage divided by 100,
# and if ratio is greater than 1, then chart1's status is invalid,
# else chart1's status is valid
{
    ratio = chart1.percentage / 100
    if ratio > 1:
        chart1.status = invalid
    else
        chart1.status = valid
}

assert(chart1.status, "VALID")

# valid is "V"
valid = "V"

assert(chart1.status, "V")

---

# Nucleoid creates a property assignment with multiple properties

# There is a Person type
class Person

# person1 is a Person
person1 = Person()

# There is an Address type
class Address

# address1 is an Address
address1 = Address()

# Any address's print is the address's city plus ", " plus the address's state
$Address.print = $Address.city + ", " + $Address.state

# person1's address is an Address
person1.address = Address()

# person1's address's city is "Syracuse"
person1.address.city = "Syracuse"

# person1's address's state is "NY"
person1.address.state = "NY"

assert(person1.address.print, "Syracuse, NY")

---

# Nucleoid creates a property assignment with multiple properties as part of a declaration

# There is a Server type
class Server

# server1 is a Server
server1 = Server()

# server1's name is "HOST1"
server1.name = "HOST1"

# There is an IP type
class IP

# ip1 is an IP
ip1 = IP()

# server1's ip is ip1
server1.ip = ip1

# ip1's address is "10.0.0.1"
ip1.address = "10.0.0.1"

# server1's summary is server1's name plus "@" plus server1's ip's address
server1.summary = server1.name + "@" + server1.ip.address

assert(server1.summary, "HOST1@10.0.0.1")

# ip1's address is "10.0.0.2"
ip1.address = "10.0.0.2"

assert(server1.summary, "HOST1@10.0.0.2")

---

# Nucleoid creates a dependency on behalf if a property has a reference

# There is a Schedule type
class Schedule

# schedule1 is a Schedule
schedule1 = Schedule()

# There is a Template type
class Template

# template1 is a Template
template1 = Template()

# template1's type is "W"
template1.type = "W"

# schedule1's template is template1
schedule1.template = template1

# schedule1's template's name is schedule1's template's type plus "-0001"
schedule1.template.name = schedule1.template.type + "-0001"

assert(template1.name, "W-0001")
assert(schedule1.template.name, "W-0001")

# template1's type is "D"
template1.type = "D"

assert(template1.name, "D-0001")

# template1's shape is template1's type plus "-Form"
template1.shape = template1.type + "-Form"

assert(template1.shape, "D-Form")
assert(schedule1.template.shape, "D-Form")

# template1's type is "C"
template1.type = "C"

assert(template1.shape, "C-Form")
assert(schedule1.template.shape, "C-Form")

---

# Nucleoid creates a dependency on behalf if a local variable has a reference

# There is a Vote type
class Vote

# vote1 is a Vote whose rate is 4
vote1 = Vote()
vote1.rate = 4

# There is a Question type
class Question

# Any question's rate is 0
$Question.rate = 0

# Any question's count is 0
$Question.count = 0

# question1 is a Question
question1 = Question()

# vote1's question is question1
vote1.question = question1

# While in the block, question is vote1's question,
# and question's rate is question's rate times question's count plus vote1's rate,
# divided by question's count plus 1,
# and question's count is question's count plus 1
{
    question = vote1.question
    question.rate = (question.rate * question.count + vote1.rate) / (question.count + 1)
    question.count += 1
}

assert(question1.rate, 4)
assert(question1.count, 1)

# vote1's rate is 5
vote1.rate = 5

assert(question1.rate, 4.5)

---

# Nucleoid runs an expression statement of class

# There is an Element type
class Element

# alkalis is an empty list
alkalis = []

# element1 is an Element whose number is 3
element1 = Element()
element1.number = 3

# While in the block, number is any element's number,
# and if number is 3, then the element is added to alkalis
{
    number = $Element.number
    if number == 3:
        alkalis.push($Element)
}

assert(alkalis.pop(), element1)

---

# Nucleoid rejects a variable declaration without definition

try:
    # a is declared as a number but not defined
    a: int
catch error:
    assert(error, ReferenceError("Missing definition"))

---

# Nucleoid creates a dependency based on the length of an identifier

# str1 is "ABC"
str1 = "ABC"

# i1 is str1's length plus 1
i1 = str1.length + 1

assert(i1, 4)

# str1 is "ABCD"
str1 = "ABCD"

assert(i1, 5)

# if str1's length is greater than 5, then i2 is i1
if str1.length > 5:
    i2 = i1

# str1 is "ABCDEF"
str1 = "ABCDEF"

assert(i2, 7)

---

# Nucleoid adds a created class to the class list

assert(Class.length, 0)

# There is a Student type
class Student

assert(Class.length, 1)

# There is a User type
class User

assert(Class.length, 2)

---

# Nucleoid updates a class definition

# There is a Message type
class Message

# No message is read
$Message.read = false

# message1 is a Message
message1 = Message()

# There is a Message type,
# which has a payload as a string
class Message(payload: str):
    this.payload = payload

assert(message1.read, false)
assert(message1.payload, null)

# message2 is a Message whose payload is "MESSAGE"
message2 = Message("MESSAGE")

assert(message2.read, false)
assert(message2.payload, "MESSAGE")

---

# Nucleoid supports a string in an expression

assert('New String', "New String")
assert("New String", "New String")
assert(`New String`, "New String")

# a is 123
a = 123

assert(`New ${a} String`, "New 123 String")

---

# Nucleoid supports logical operators

# condition is false
condition = false

assert(condition or true, true)
assert(condition || true, true)

assert(not condition and true, true)
assert(!condition && true, true)

---

# Nucleoid supports standard built-in objects

# max is the maximum integer
max = Number.MAX_INTEGER

assert(max, 9007199254740991)

# now is the current time
now = Date.now()

assert(now > 0, true)

assert(String(Math.max(Number.NaN)), "NaN")
assert(String(Math.min(Number.NaN)), "NaN")
assert(String(Math.max(Number.NaN, 1)), "NaN")
assert(String(Math.min(Number.NaN, 1)), "NaN")
assert(String(Math.max(1, Number.NaN)), "NaN")
assert(String(Math.min(1, Number.NaN)), "NaN")
assert(String(Math.max(1, Number.NaN, 2)), "NaN")
assert(String(Math.min(1, Number.NaN, 2)), "NaN")
assert(String(Math.max(Number.POSITIVE_INFINITY, Number.NaN)), "NaN")
assert(String(Math.min(Number.NEGATIVE_INFINITY, Number.NaN)), "NaN")
assert(String(Math.max(1, "invalid")), "NaN")
assert(String(Math.min(1, "invalid")), "NaN")
assert(String(Math.max(1, Math.sqrt(-1))), "NaN")
assert(String(Math.min(1, Math.sqrt(-1))), "NaN")

assert(Math.max(), Number.NEGATIVE_INFINITY)
assert(Math.min(), Number.POSITIVE_INFINITY)
assert(Math.max("7", 5), 7)
assert(Math.min("7", 5), 5)
assert(Math.max(Number.NEGATIVE_INFINITY, 5), 5)
assert(Math.min(Number.POSITIVE_INFINITY, 5), 5)

a = 5
b = 9
peak = Math.max(a, b)
floor = Math.min(a, b)

assert(peak, 9)
assert(floor, 5)

b = Number.NaN

assert(String(peak), "NaN")
assert(String(floor), "NaN")

b = 3

assert(peak, 5)
assert(floor, 3)

b = null

assert(peak, null)
assert(floor, null)

b = 2

assert(peak, 5)
assert(floor, 2)

---

# Nucleoid supports creating standard built-in objects

# date is July 24, 2019
date = Date("2019-7-24")

assert(date.getYear(), 119)

---

# Nucleoid supports built-in objects

# date1 is the current date
date1 = Date()

# date2 is a date whose time is date1's time
date2 = Date(date1.getTime())

assert(date1.getTime() == date2.getTime(), true)

# date3 is the parsed date of "04 Dec 1995 00:12:00 GMT"
date3 = Date.parse("04 Dec 1995 00:12:00 GMT")

assert(date3, 818035920000)

try:
    # date4 is the wrong date
    date4 = Date.wrong()
catch error:
    assert(error, TypeError("Date.wrong is not a function"))

---

# Nucleoid calls a function with no return

# a is 1
a = 1

# copy assigns val to b
def copy val:
    b = val

# Call copy with a
copy(a)

# return: null

---

# Nucleoid calls a function with a return value

# a is 1
a = 1

# copy assigns val to b and returns val
def copy val:
    b = val
    return val

# Call copy with a
copy(a)

# return: 1

---

# Nucleoid supports a function in an expression

# list is a list of 1, 2 and 3
list = [1, 2, 3]

assert(list.find(function(element) { return element == 3 }), 3)
assert(list.find(element => { return element == 2 }), 2)
assert(list.find(element => element == 1), 1)
assert(list.find(element => (element == 1)), 1)

---

# Nucleoid supports a function with a parameter in an expression

# samples is a list of 38.2, 39.1, 38.8 and 39
samples = [38.2, 39.1, 38.8, 39]

# ratio is 2.1
ratio = 2.1

# element is 38.5
element = 38.5

# The element parameter shadows the outer element in each function
assert(samples.find(function(element) { result = element * ratio; return result == 81.48 }), 38.8)
assert(samples.find(element => { result = element * ratio; return result == 81.48 }), 38.8)
assert(samples.find(element => element == 38.8), 38.8)
assert(samples.find(element => (element == 38.8)), 38.8)

---

# Nucleoid creates a variable statement with JSON

# While in the block, payload is a local variable whose data is "TEST"
# and whose nested data is "NESTED_TEST"
{
    payload = { "data": "TEST", "nested": { "data": "NESTED_TEST" } }
    assert(payload.data, "TEST")
    assert(payload.nested.data, "NESTED_TEST")
}

# message is an object whose pid is 1200
message = { "pid": 1200 }

assert(message.pid, 1200)

# While in the block, scope is a local variable whose query is "test",
# and i is a local variable whose test is scope's query
{
    scope = { "query": "test" }
    i = { "test": scope.query }
    assert(i.test, "test")
}

---

# Nucleoid returns an inline JSON object

# While in the block, return an object whose number is 123,
# whose string is "ABC" and whose bool is true
{
    return { "number": 123, "string": "ABC", "bool": true }
}

# return: { "number": 123, "string": "ABC", "bool": true }

---

# Nucleoid returns an inline JSON array

# While in the block, return a list of an object whose number is 123,
# whose string is "ABC" and whose bool is true
{
    return [{ "number": 123, "string": "ABC", "bool": true }]
}

# return: [{ "number": 123, "string": "ABC", "bool": true }]

---

# Nucleoid returns an inline object

# While in the block, return an object whose number is 123,
# whose string is "ABC" and whose bool is true
{
    return { number: 123, string: "ABC", bool: true }
}

# return: { "number": 123, "string": "ABC", "bool": true }

---

# Nucleoid returns an inline array

# While in the block, return a list of an object whose number is 123,
# whose string is "ABC" and whose bool is true
{
    return [{ number: 123, string: "ABC", bool: true }]
}

# return: [{ "number": 123, "string": "ABC", "bool": true }]

---

# Nucleoid supports nested functions as a parameter in an expression

# name is "AbCDE"
name = "AbCDE"

# pointer is 0
pointer = 0

# if the character of name at pointer is not an uppercase letter,
# then throw "INVALID_FIRST_CHARACTER"
if not /[A-Z]/.test(name.charAt(pointer)):
    throw "INVALID_FIRST_CHARACTER"

try:
    # name is "bbCDE"
    name = "bbCDE"
catch error:
    assert(error, "INVALID_FIRST_CHARACTER")

# name is "CbCDE"
name = "CbCDE"

try:
    # pointer is 1
    pointer = 1
catch error:
    assert(error, "INVALID_FIRST_CHARACTER")

---

# Nucleoid supports a property of chained functions in an expression

# There is a User type
class User

# There is a Registration type
class Registration

# user1 is a User
user1 = User()

# registration1 is a Registration whose user is user1
registration1 = Registration()
registration1.user = user1

# registration2 is a Registration whose user is user1
registration2 = Registration()
registration2.user = user1

try:
    # if the number of registrations whose user is any user is more than one,
    # then throw "USER_ALREADY_REGISTERED"
    if Registration.filter(r => r.user == $User).length > 1:
        throw "USER_ALREADY_REGISTERED"
catch error:
    assert(error, "USER_ALREADY_REGISTERED")

---

# Nucleoid throws an error as a string

try:
    # throw 'INVALID'
    throw 'INVALID'
catch error:
    assert(error, "INVALID")

try:
    # throw "INVALID"
    throw "INVALID"
catch error:
    assert(error, "INVALID")

---

# Nucleoid throws an error as an integer

try:
    # throw 123
    throw 123
catch error:
    assert(error, 123)

---

# Nucleoid throws a reference error if the thrown value is not defined

try:
    # throw abc
    throw abc
catch error:
    assert(error, ReferenceError("abc is not defined"))

---

# Nucleoid creates a class assignment before initialization

# There is a Review type
class Review

# Any review's rate is the review's sum divided by 10
$Review.rate = $Review.sum / 10

# review1 is a Review
review1 = Review()

assert(review1.rate, null)

# review1's sum is 42
review1.sum = 42

assert(review1.rate, 4.2)

---

# Nucleoid creates a class assignment after initialization

# There is a Shape type
class Shape

# shape1 is a Shape whose edge is 3
shape1 = Shape()
shape1.edge = 3

# shape2 is a Shape whose edge is 3
shape2 = Shape()
shape2.edge = 3

# Any shape's angle is the shape's edge minus 2, times 180
$Shape.angle = ($Shape.edge - 2) * 180

assert(shape1.angle, 180)
assert(shape2.angle, 180)

# shape1's edge is 4
shape1.edge = 4

assert(shape1.angle, 360)
assert(shape2.angle, 180)

---

# Nucleoid updates a class assignment

# There is an Employee type
class Employee

# employee1 is an Employee whose id is 1
employee1 = Employee()
employee1.id = 1

# Any employee's username is "E" plus the employee's id
$Employee.username = "E" + $Employee.id

assert(employee1.username, "E1")

# Any employee's username is "F" plus the employee's id
$Employee.username = "F" + $Employee.id

assert(employee1.username, "F1")

# employee1's id is 2
employee1.id = 2

assert(employee1.username, "F2")

---

# Nucleoid creates an if statement of class before initialization

# There is a Ticket type
class Ticket

# Any ticket whose date is after January 1, 1993 is expired
if $Ticket.date > Date("1993-1-1"):
    $Ticket.status = "EXPIRED"

# ticket1 is a Ticket
ticket1 = Ticket()

assert(ticket1.status, null)

# ticket1's date is February 1, 1993
ticket1.date = Date("1993-2-1")

assert(ticket1.status, "EXPIRED")

# ticket2 is a Ticket
ticket2 = Ticket()

assert(ticket2.status, null)

---

# Nucleoid creates an if statement of class after initialization

# There is a Student type
class Student

# student1 is a Student whose age is 2 and whose class is "Daycare"
student1 = Student()
student1.age = 2
student1.class = "Daycare"

# student2 is a Student whose age is 2 and whose class is "Daycare"
student2 = Student()
student2.age = 2
student2.class = "Daycare"

# Any student whose age is 3 is in Preschool
if $Student.age == 3:
    $Student.class = "Preschool"

assert(student1.class, "Daycare")
assert(student2.class, "Daycare")

# student1's age is 3
student1.age = 3

assert(student1.class, "Preschool")
assert(student2.class, "Daycare")

---

# Nucleoid updates an if block of class

# There is an Inventory type
class Inventory

# inventory1 is an Inventory whose quantity is 0
inventory1 = Inventory()
inventory1.quantity = 0

# inventory2 is an Inventory whose quantity is 1000
inventory2 = Inventory()
inventory2.quantity = 1000

# Any inventory whose quantity is 0 needs replenishment
if $Inventory.quantity == 0:
    $Inventory.replenishment = true

assert(inventory1.replenishment, true)
assert(inventory2.replenishment, null)

# Any inventory whose quantity is 0 does not need replenishment
if $Inventory.quantity == 0:
    $Inventory.replenishment = false

assert(inventory1.replenishment, false)
assert(inventory2.replenishment, null)

---

# Nucleoid creates an else statement of class before initialization

# There is a Count type
class Count

# If any count's max is greater than 1000, then the count's reset is urgent,
# else the count's reset is regular
if $Count.max > 1000:
    $Count.reset = urgent
else
    $Count.reset = regular

# urgent is "URGENT"
urgent = "URGENT"

# regular is "REGULAR"
regular = "REGULAR"

# count1 is a Count
count1 = Count()

# count1's max is 850
count1.max = 850

assert(count1.reset, "REGULAR")

# regular is "R"
regular = "R"

assert(count1.reset, "R")

---

# Nucleoid creates an else statement of class after initialization

# There is a Concentration type
class Concentration

# serialDilution is "(c1V1+c2V2)/(V1+V2)"
serialDilution = "(c1V1+c2V2)/(V1+V2)"

# directDilution is "c1/V1"
directDilution = "c1/V1"

# concentration1 is a Concentration whose substances is 2
concentration1 = Concentration()
concentration1.substances = 2

# If any concentration's substances is 1, then the concentration's formula is directDilution,
# else the concentration's formula is serialDilution
if $Concentration.substances == 1:
    $Concentration.formula = directDilution
else
    $Concentration.formula = serialDilution

assert(concentration1.formula, "(c1V1+c2V2)/(V1+V2)")

# serialDilution is "(c1V1+c2V2+c3V3)/(V1+V2+V3)"
serialDilution = "(c1V1+c2V2+c3V3)/(V1+V2+V3)"

assert(concentration1.formula, "(c1V1+c2V2+c3V3)/(V1+V2+V3)")

---

# Nucleoid creates an else if statement of class before initialization

# There is a Storage type
class Storage

# normal is "NORMAL", low is "LOW", and empty is "EMPTY"
normal = "NORMAL"; low = "LOW"; empty = "EMPTY"

# If any storage's capacity is greater than 25, then the storage's status is normal,
# else if the storage's capacity is greater than 0, then the storage's status is low,
# else the storage's status is empty
if $Storage.capacity > 25:
    $Storage.status = normal
else if $Storage.capacity > 0:
    $Storage.status = low
else
    $Storage.status = empty

# storage1 is a Storage
storage1 = Storage()

# storage1's capacity is 23
storage1.capacity = 23

assert(storage1.status, "LOW")

# low is "L"
low = "L"

assert(storage1.status, "L")

---

# Nucleoid creates an else if statement of class after initialization

# There is a Registration type
class Registration

# yes is "YES", pending is "PENDING", and no is "NO"
yes = "YES"; pending = "PENDING"; no = "NO"

# registration1 is a Registration whose available is 0
registration1 = Registration()
registration1.available = 0

# If any registration's available is greater than 10, then the registration's accepted is yes,
# else if the registration's available is greater than 0, then the registration's accepted is pending,
# else the registration's accepted is no
if $Registration.available > 10:
    $Registration.accepted = yes
else if $Registration.available > 0:
    $Registration.accepted = pending
else
    $Registration.accepted = no

assert(registration1.accepted, "NO")

# yes is true, and no is false
yes = true; no = false

assert(registration1.accepted, false)

---

# Nucleoid creates multiple else if statement of class before initialization

# There is a Capacity type
class Capacity

# If any capacity's spare divided by the capacity's available is greater than 0.5,
# then the capacity's total is the capacity's available plus the capacity's spare,
# else if any capacity's spare divided by the capacity's available is greater than 0.1,
# then the capacity's total is the capacity's available plus the capacity's spare times 2,
# else the capacity's total is the capacity's available plus the capacity's spare times 3
if $Capacity.spare / $Capacity.available > 0.5:
    $Capacity.total = $Capacity.available + $Capacity.spare
else if $Capacity.spare / $Capacity.available > 0.1:
    $Capacity.total = $Capacity.available + $Capacity.spare * 2
else
    $Capacity.total = $Capacity.available + $Capacity.spare * 3

# capacity1 is a Capacity
capacity1 = Capacity()

# capacity1's available is 100
capacity1.available = 100

# capacity1's spare is 5
capacity1.spare = 5

assert(capacity1.total, 115)

# capacity1's spare is 1
capacity1.spare = 1

assert(capacity1.total, 103)

---

# Nucleoid creates multiple else if statement of class after initialization

# There is a Shape type
class Shape

# shape1 is a Shape whose type is "RECTANGLE",
# whose x is 5 and whose y is 6
shape1 = Shape()
shape1.type = "RECTANGLE"
shape1.x = 5
shape1.y = 6

# If any shape's type is "SQUARE", then the shape's area is the shape's x squared,
# else if any shape's type is "TRIANGLE", then the shape's area is the shape's x times the shape's y divided by 2,
# else the shape's area is the shape's x times the shape's y
if $Shape.type == "SQUARE":
    $Shape.area = Math.pow($Shape.x, 2)
else if $Shape.type == "TRIANGLE":
    $Shape.area = $Shape.x * $Shape.y / 2
else
    $Shape.area = $Shape.x * $Shape.y

assert(shape1.area, 30)

# shape1's x is 7
shape1.x = 7

assert(shape1.area, 42)

---

# Nucleoid runs a block statement of class before initialization

# There is a Stock type
class Stock

# while in the block, change is a local variable that is any stock's before times 4 divided by 100,
# and the stock's after is the stock's before plus change
{
    change = $Stock.before * 4 / 100
    $Stock.after = $Stock.before + change
}

# stock1 is a Stock
stock1 = Stock()

assert(stock1.after, null)

# stock1's before is 57.25
stock1.before = 57.25

assert(stock1.after, 59.54)

# stock1's before is 59.5
stock1.before = 59.5

assert(stock1.after, 61.88)

---

# Nucleoid runs a block statement of class after initialization

# There is a Purchase type
class Purchase

# purchase1 is a Purchase whose price is 99
purchase1 = Purchase()
purchase1.price = 99

# while in the block, retail is a local variable that is any purchase's price times 1.15,
# and the purchase's retail price is retail
{
    retail = $Purchase.price * 1.15
    $Purchase.retailPrice = retail
}

assert(purchase1.retailPrice, 113.85)

# purchase1's price is 199
purchase1.price = 199

assert(purchase1.retailPrice, 228.85)

---

# Nucleoid runs a nested block statement of class before initialization

# There is a Compound type
class Compound

# while in the block, mol is a local variable that is 69.94 divided by any compound's substance,
# and in a nested block, the compound's sample is the floor of mol times the compound's mol
{
    mol = 69.94 / $Compound.substance
    {
        $Compound.sample = Math.floor(mol * $Compound.mol)
    }
}

# compound1 is a Compound
compound1 = Compound()

# compound1's substance is 55.85
compound1.substance = 55.85

# compound1's mol is 1000
compound1.mol = 1000

assert(compound1.sample, 1252)

---

# Nucleoid runs a nested block statement of class after initialization

# There is a Bug type
class Bug

# bug1 is a Bug whose initial score is 1000
# and whose aging is 24
bug1 = Bug()
bug1.initialScore = 1000
bug1.aging = 24

# while in the block, score is a local variable that is any bug's aging times 10,
# and in a nested block, the bug's priority score is score plus the bug's initial score
{
    score = $Bug.aging * 10
    {
        $Bug.priorityScore = score + $Bug.initialScore
    }
}

assert(bug1.priorityScore, 1240)

---

# Nucleoid runs a nested if statement of class before initialization

# There is a Mortgage type
class Mortgage

# rate1 is "EXCEPTIONAL"
rate1 = "EXCEPTIONAL"

# while in the block, interest is a local variable that is any mortgage's annual divided by 12,
# and if interest is less than 4, then the mortgage's rate is rate1
{
    interest = $Mortgage.annual / 12
    if interest < 4:
        $Mortgage.rate = rate1
}

# mortgage1 is a Mortgage
mortgage1 = Mortgage()

# mortgage1's annual is 46
mortgage1.annual = 46

assert(mortgage1.rate, "EXCEPTIONAL")

# rate1 is "E"
rate1 = "E"

assert(mortgage1.rate, "E")

---

# Nucleoid runs a nested if statement of class after initialization

# There is a Building type
class Building

# buildingType1 is "SKYSCRAPER"
buildingType1 = "SKYSCRAPER"

# building1 is a Building whose floors is 20
building1 = Building()
building1.floors = 20

# while in the block, height is a local variable that is any building's floors times 14,
# and if height is greater than 330, then the building's type is buildingType1
{
    height = $Building.floors * 14
    if height > 330:
        $Building.type = buildingType1
}

assert(building1.type, null)

# building1's floors is 25
building1.floors = 25

assert(building1.type, "SKYSCRAPER")

# buildingType1 is "S"
buildingType1 = "S"

assert(building1.type, "S")

---

# Nucleoid creates a nested else statement of class before initialization

# There is an Account type
class Account

# noAlert is "NO_ALERT"
noAlert = "NO_ALERT"

# lowAlert is "LOW_ALERT"
lowAlert = "LOW_ALERT"

# while in the block, balance is a local variable that is any account's balance,
# and if balance is greater than 1000, then the account's alert is noAlert,
# else the account's alert is lowAlert
{
    balance = $Account.balance
    if balance > 1000:
        $Account.alert = noAlert
    else
        $Account.alert = lowAlert
}

# account1 is an Account
account1 = Account()

# account1's balance is 950
account1.balance = 950

assert(account1.alert, "LOW_ALERT")

# lowAlert is "L"
lowAlert = "L"

assert(account1.alert, "L")

---

# Nucleoid creates a nested else statement of class after initialization

# There is a Question type
class Question

# high is "HIGH"
high = "HIGH"

# low is "LOW"
low = "LOW"

# question1 is a Question whose count is 1
question1 = Question()
question1.count = 1

# while in the block, score is a local variable that is any question's count times 10,
# and if score is greater than 100, then the question's type is high,
# else the question's type is low
{
    score = $Question.count * 10
    if score > 100:
        $Question.type = high
    else
        $Question.type = low
}

assert(question1.type, "LOW")

# low is "L"
low = "L"

assert(question1.type, "L")

# question1's count is 11
question1.count = 11

assert(question1.type, "HIGH")

---

# Nucleoid creates a class assignment with multiple properties before declaration

# There is a Room type
class Room

# Any room's level is the room's number divided by 10
$Room.level = $Room.number / 10

# There is a Guest type
class Guest

# Any guest's room is a Room
$Guest.room = Room()

# guest1 is a Guest
guest1 = Guest()

# guest1's room's number is 30
guest1.room.number = 30

assert(guest1.room.level, 3)

# guest2 is a Guest
guest2 = Guest()

assert(guest2.room.number, 30)
assert(guest2.room.level, 3)

---

# Nucleoid creates a class assignment with multiple properties after declaration

# There is a Channel type
class Channel

# There is a Frequency type
class Frequency

# channel1 is a Channel
channel1 = Channel()

# Any channel's frequency is a Frequency
$Channel.frequency = Frequency()

# Any frequency's hertz is 1 divided by the frequency's period
$Frequency.hertz = 1 / $Frequency.period

assert(channel1.frequency.hertz, null)

# channel1's frequency's period is 0.0025
channel1.frequency.period = 0.0025

assert(channel1.frequency.hertz, 400)

# channel2 is a Channel
channel2 = Channel()

assert(channel2.frequency.period, 0.0025)
assert(channel2.frequency.hertz, 400)

---

# Nucleoid creates a class assignment as multiple properties as part of a declaration before initialization

# There is a Hospital type
class Hospital

# There is a Clinic type
class Clinic

# Any hospital's clinic is a Clinic
$Hospital.clinic = Clinic()

# Any hospital's patients is the hospital's clinic's beds times 746
$Hospital.patients = $Hospital.clinic.beds * 746

# hospital1 is a Hospital
hospital1 = Hospital()

assert(hospital1.patients, null)

# hospital1's clinic's beds is 2678
hospital1.clinic.beds = 2678

assert(hospital1.patients, 1997788)

# hospital1's clinic's beds is 3000
hospital1.clinic.beds = 3000

assert(hospital1.patients, 2238000)

---

# Nucleoid creates a class assignment as multiple properties as part of a declaration after initialization

# There is a Server type
class Server

# There is an OS type
class OS

# Any server's os is an OS
$Server.os = OS()

# server1 is a Server
server1 = Server()

# server1's os's version is 14
server1.os.version = 14

# Any server's build is the server's os's version plus ".526291"
$Server.build = $Server.os.version + ".526291"

assert(server1.build, "14.526291")

# server1's os's version is 15
server1.os.version = 15

assert(server1.build, "15.526291")

---

# Nucleoid creates a class assignment only if the instance is defined

# There is a Phone type
class Phone

try:
    # any phone's line's wired is true
    $Phone.line.wired = true
catch error:
    assert(error, ReferenceError("Phone.line is not defined"))

---

# Nucleoid creates a for of statement

# There is a Question type,
# which has a rate as a number
class Question(rate: int):
    this.rate = rate

# question1 is a Question whose rate is 4
question1 = Question(4)

# question2 is a Question whose rate is 5
question2 = Question(5)

# There is a Summary type,
# which has a question as a Question
class Summary(question):
    this.question = question

# Any summary's rate is the value of the summary's question's rate
$Summary.rate = $Summary.question.rate.value

# For each question of Question, there is a Summary whose question is the question
for question of Question:
    Summary(question)

assert(Summary[0].rate, 4)
assert(Summary[1].rate, 5)

---

# Nucleoid creates a block of for statement without dependencies

# There is an Item type
class Item

# item1 is an Item
item1 = Item()

# item2 is an Item
item2 = Item()

# VALUE is 10
VALUE = 10

# For each item of Item, while in the block, i is a local variable that is 10 times VALUE,
# and the item's score is i
for item of Item:
    i = 10 * VALUE
    item.score = i

# VALUE is 20
VALUE = 20

assert(item1.score, 100)
assert(item2.score, 100)

# For each item of Item, while in the block, i is a local variable that is 10 times VALUE,
# and the item's score is i
for item of Item:
    i = 10 * VALUE
    item.score = i

assert(item1.score, 200)
assert(item2.score, 200)

# item3 is an Item
item3 = Item()

assert(item3.score, null)

---

# Nucleoid loops through only defined objects in a for of statement

# array is an empty list
array = []

# There is an Item type
class Item

# item1 is an Object, and item1 is added to array
item1 = Object()
array.push(item1)

# item2 is an object whose id is "item3", and item2 is added to array
item2 = { "id": "item3" }
array.push(item2)

# item4 is an Item, and item4 is added to array
item4 = Item()
array.push(item4)

# item5 is an object whose id is "item4", and item5 is added to array
item5 = { "id": "item4" }
array.push(item5)

# count is 0
count = 0

# items is an empty list
items = []

# For each item of array, count is count plus 1,
# and the item is added to items
for item of array:
    count += 1
    items.push(item)

assert(count, 1)
assert(items.length, 1)
assert(items[0], item4)

---

# Nucleoid supports an if statement in a for of statement

# There is a Question type
class Question

# question1 is a Question
question1 = Question()

# question2 is a Question, which is archived
question2 = Question()
question2.archived = true

# question3 is a Question
question3 = Question()

# There is a Summary type,
# which has a question as a Question
class Summary(question):
    this.question = question

# Any summary's type is "DAILY"
$Summary.type = "DAILY"

# For each question of Question, if the question is not archived,
# then there is a Summary whose question is the question
for question of Question:
    if not question.archived:
        Summary(question)

assert(Summary.length, 2)
assert(Summary[0].question.id, "question1")
assert(Summary[1].question.id, "question3")
assert(Summary[0].type, "DAILY")
assert(Summary[1].type, "DAILY")

# Any summary's type is "WEEKLY"
$Summary.type = "WEEKLY"

assert(Summary[0].type, "WEEKLY")
assert(Summary[1].type, "WEEKLY")

---

# Nucleoid returns an integer in variable assignment
# test assigns 2 to a and returns the value of the assignment
# test assigns 2 to a and returns the value of the assignment
def test a => a = 2

# b is 1
b = 1

# Call test with b
test(b)

# return: 2

---

# Nucleoid returns the reference of a function call

# a is an Object
a = Object()

# c is 1
c = 1
# test assigns a to b and returns the value of the assignment
# test assigns a to b and returns the value of the assignment
def test b => b = a

assert(test(c), {})
assert(c, 1)

---

# Nucleoid returns a string value of a function call
# test assigns "abc" to a and returns the value of the assignment
# test assigns "abc" to a and returns the value of the assignment
def test a => a = "abc"

# b is 1
b = 1

# Call test with b
test(b)

# return: "abc"

---

# Nucleoid returns an object value of a function call
# test assigns an Object to a and returns the value of the assignment
# test assigns an Object to a and returns the value of the assignment
def test a => a = Object()

# b is 1
b = 1

# Call test with b
test(b)

# return: {}

---

# Nucleoid runs a function with a variable
# test returns a plus 23
# test returns a plus 23
def test a => a + 23

# data is "UUID-1"
data = "UUID-1"

# Call test with data
test(data)

# return: "UUID-123"

---

# Nucleoid returns the first return statement in a block

# While in the block, return 123, and return "abc"
{
    return 123
    return "abc"
}

# return: 123

---

# Nucleoid returns the instance itself in instance creation

# There is a Test type,
# which has a prop as a number
class Test(prop: int):
    this.prop = prop

# There is a Test whose prop is 123
Test(123)

assert(Test[0].prop, 123)
assert(Test[0].id != null, true)

# return: { "id": "[UUID]", "prop": 123 }

---

# Nucleoid explains how a value was derived

# a is 1
a = 1

# b is a plus 2
b = a + 2

# c is b times 2
c = b * 2

assert((why c).length, 3)
assert((why c)[0].node, "c")
assert((why c)[0].holds, 6)
assert((why c)[0].rule, "c = b*2")
assert((why c)[0].state, "derived")
assert((why c)[0].from, ["b"])

---

# Nucleoid states a fact that follows from nothing else

# a is 1
a = 1

# b is a plus 2
b = a + 2

assert((why b)[1].node, "a")
assert((why b)[1].state, "stated")
assert((why b)[1].from, [])

---

# Nucleoid names the class-level rule a property was derived from

# There is a Human type,
# which has a name as a string
class Human(name: str):
    this.name = name

# Every human is mortal
$Human.mortal = true

# Socrates is a Human
socrates = Human("Socrates")

assert((why socrates.mortal).length, 1)
assert((why socrates.mortal)[0].rule, "$Human.mortal = true")
assert((why socrates.mortal)[0].state, "derived")

---

# Nucleoid reports what a value affects

# a is 1
a = 1

# b is a plus 2
b = a + 2

# c is b times 2
c = b * 2

assert((affects a).length, 2)
assert((affects a)[0].node, "b")
assert((affects a)[1].node, "c")

---

# Nucleoid chains reasoning operations

# a is 1
a = 1

# b is a plus 2
b = a + 2

# c is b times 2
c = b * 2

assert((c |> why).length, 3)
assert((a |> affects |> why).length, 3)
assert((why c).length, (c |> why).length)

---

# Nucleoid keeps an explanation up to date

# a is 1
a = 1

# b is a plus 2
b = a + 2

# c is b times 2
c = b * 2

# trace is why c
trace = why c

assert(trace[0].holds, 6)

# a is 5
a = 5

assert(trace[0].holds, 14)

---

# Nucleoid does not select a reasoning statement

# a is 1
a = 1

# b is a plus 2
b = a + 2

# trace is why b
trace = why b

assert((affects a).length, 1)
assert((affects a)[0].node, "b")

---

# Nucleoid selects the whole model

# a is 1
a = 1

# b is a plus 2
b = a + 2

assert((model |> why).length, 2)

---

# Nucleoid throws an error when explaining something that is not defined

try:
    # explain nothing
    why nothing
catch error:
    assert(error, ReferenceError("nothing is not defined"))

---

# Nucleoid treats a reasoning name as a variable when one is defined

# why is 1
why = 1

assert(why, 1)

---

# Nucleoid freezes value reads inside compound expressions

# seed and live are independent sources
seed = 1
live = 10

# Snapshots remain fixed while live dependencies change
values = [seed.value, live]
record = { "frozen": seed.value, live }
label = `${seed.value}-${live}`
text = "abcd"
start = 1
end = 3
part = text[start.value:end]
tail = text[start.value:]
head = text[:end.value]

seed = 2
live = 20
start = 2
end = 4
text = "wxyz"

assert(values, [1, 20])
assert(record, { "frozen": 1, "live": 20 })
assert(label, "1-20")
assert(part, "xyz")
assert(tail, "xyz")
assert(head, "wxy")

---

# Nucleoid updates inherited rules on existing subtype instances

class Parent

$Parent.flag = 1

class Child: Parent
    pass

class Grandchild: Child
    pass

child = Child()
grandchild = Grandchild()

$Parent.flag = 2
$Parent.note = "NEW"

assert(child.flag, 2)
assert(grandchild.flag, 2)
assert(child.note, "NEW")
assert(grandchild.note, "NEW")
assert(Parent.length, 0)
assert(Child.length, 1)

if $Parent.flag == 2:
    $Parent.active = true

assert(child.active, true)
assert(grandchild.active, true)

delete $Parent.note

assert(child.note, null)
assert(grandchild.note, null)

---

# Nucleoid preserves subtype property rules when a parent rule changes

class Parent

class Child: Parent
    pass

$Child.flag = 7
child = Child()

$Parent.flag = 2

assert(child.flag, 7)

if $Parent.id != "":
    $Parent.flag = 3

{
    $Parent.flag = 4
}

assert(child.flag, 7)

another = Child()

assert(another.flag, 7)

delete $Parent.flag

assert(child.flag, 7)
assert(another.flag, 7)

---

# Nucleoid rolls back a parent rule that fails for a subtype instance

class Parent

class Child: Parent
    pass

parent = Parent()
parent.score = 0
child = Child()
child.score = 2

try:
    if $Parent.score > 1:
        throw "LIMIT"
catch error:
    assert(error, "LIMIT")

child.score = 3
parent.score = 3
another = Child()
another.score = 4

assert(child.score, 3)
assert(parent.score, 3)
assert(another.score, 4)

---

# Nucleoid resolves super from the constructor currently executing

class Base(amount):
    this.amount = amount

class Middle: Base
    def init amount:
        super(amount + 1)
        this.middle = true

class Leaf: Middle
    def init amount:
        super(amount * 2)
        this.leaf = true

leaf = Leaf(3)

assert(leaf.amount, 7)
assert(leaf.middle, true)
assert(leaf.leaf, true)

class Passive: Middle
    pass

passive = Passive(4)

assert(passive.amount, 5)

class Final: Passive
    def init amount:
        super(amount + 2)

final = Final(4)

assert(final.amount, 7)

---

# Nucleoid rejects a class inheriting from itself

try:
    class Loop: Loop
        pass
catch error:
    assert(error, TypeError("Circular Inheritance"))

assert(Class.length, 0)

class Loop

loop = Loop()

assert(loop.id, "loop")

---

# Nucleoid rejects an inheritance cycle introduced by redeclaration

class Root

$Root.flag = true

class Child: Root
    pass

try:
    class Root: Child
        pass
catch error:
    assert(error, TypeError("Circular Inheritance"))

child = Child()

assert(child.flag, true)
assert(Class.length, 2)

---

# Nucleoid rejects class dependency cycles inside compound expressions

class Item

$Item.list = [$Item.source]

try:
    $Item.source = $Item.list
catch error:
    assert(error, TypeError("Circular Dependency"))

$Item.record = { "source": $Item.source }

try:
    $Item.source = $Item.record
catch error:
    assert(error, TypeError("Circular Dependency"))

$Item.label = `${$Item.source}`

try:
    $Item.source = $Item.label
catch error:
    assert(error, TypeError("Circular Dependency"))

$Item.part = "abcd"[$Item.source:]

try:
    $Item.source = $Item.part.length
catch error:
    assert(error, TypeError("Circular Dependency"))

$Item.head = "abcd"[:$Item.source]

try:
    $Item.source = $Item.head.length
catch error:
    assert(error, TypeError("Circular Dependency"))

$Item.copy = $Item.list[:]

try:
    $Item.source = $Item.copy[0]
catch error:
    assert(error, TypeError("Circular Dependency"))

$Item.indexed = [$Item["source"]]

try:
    $Item.source = $Item.indexed
catch error:
    assert(error, TypeError("Circular Dependency"))

$Item.source = 1
item = Item()

assert(item.list, [1])
assert(item.label, "1")
assert(item.part, "bcd")

$Item.snapshot = [$Item.source.value]
$Item.source = $Item.snapshot[0] + 1

assert(item.source, 2)
assert(item.snapshot, [1])
assert(item.list, [2])

---

# Nucleoid checks both slice bounds when declaring a class rule

class Item

try:
    $Item.part = "abcd"[missing:]
catch error:
    assert(error, ReferenceError("missing is not defined"))

try:
    $Item.part = "abcd"[:missing]
catch error:
    assert(error, ReferenceError("missing is not defined"))

start = 1
end = 3
$Item.part = "abcd"[start:end]
item = Item()

assert(item.part, "bc")

end = 4

assert(item.part, "bcd")

---

# Nucleoid propagates null through indexed object properties

class Item

item = Item()
item.amount = null
field = "amount"
direct = item.amount + 1
indexed = item[field] + 1

assert(direct, null)
assert(indexed, null)

item.amount = 4

assert(direct, 5)
assert(indexed, 5)

delete item.amount

assert(direct, null)
assert(indexed, null)

---

# Nucleoid defers a class conditional reading an undefined indexed property

class Item

if $Item["score"] == null:
    $Item.active = true

item = Item()

assert(item.active, null)

item.score = null

assert(item.active, true)

---

# Nucleoid tracks the number of declared classes as a dependency

count = Class.length
doubled = count * 2

class First

assert(count, 1)
assert(doubled, 2)

class Second

assert(count, 2)
assert(doubled, 4)

class First

assert(count, 2)

try:
    class Temporary
    throw "ABORT"
catch error:
    assert(error, "ABORT")

assert(Class.length, 2)
assert(count, 2)
assert(doubled, 4)

class Third

assert(count, 3)
assert(doubled, 6)

if count > 3:
    throw "TOO_MANY_TYPES"

try:
    class Fourth
catch error:
    assert(error, "TOO_MANY_TYPES")

assert(Class.length, 3)
assert(count, 3)
assert(doubled, 6)

---

# Nucleoid rounds numbers without losing precision at integer and half boundaries

assert(Math.round(0.49999999999999994), 0)
assert(Math.round(0.5), 1)
assert(Math.round(0.5000000000000001), 1)
assert(Math.round(-0.49999999999999994), 0)
assert(Math.round(-0.5), 0)
assert(Math.round(-0.5000000000000001), -1)
assert(Math.round(1.4999999999999998), 1)
assert(Math.round(1.5), 2)
assert(Math.round(1.5000000000000002), 2)
assert(Math.round(-1.4999999999999998), -1)
assert(Math.round(-1.5), -1)
assert(Math.round(-1.5000000000000002), -2)

assert(Math.round(4503599627370495.5), 4503599627370496)
assert(Math.round(-4503599627370495.5), -4503599627370495)
assert(Math.round(4503599627370497), 4503599627370497)
assert(Math.round(-4503599627370497), -4503599627370497)
assert(Math.round(Number.MAX_INTEGER), Number.MAX_INTEGER)
assert(Math.round(Number.MIN_INTEGER), Number.MIN_INTEGER)
assert(Math.round(Number.MAX_VALUE), Number.MAX_VALUE)
assert(Math.round(-Number.MAX_VALUE), -Number.MAX_VALUE)

assert(1 / Math.round(-0.49999999999999994), Number.NEGATIVE_INFINITY)
assert(1 / Math.round(-0.5), Number.NEGATIVE_INFINITY)
assert(1 / Math.round(-0.1), Number.NEGATIVE_INFINITY)
assert(1 / Math.round(-0.0), Number.NEGATIVE_INFINITY)
assert(1 / Math.round(0.0), Number.POSITIVE_INFINITY)
assert(1 / Math.round(5e-324), Number.POSITIVE_INFINITY)
assert(1 / Math.round(-5e-324), Number.NEGATIVE_INFINITY)
assert(Math.round(Number.POSITIVE_INFINITY), Number.POSITIVE_INFINITY)
assert(Math.round(Number.NEGATIVE_INFINITY), Number.NEGATIVE_INFINITY)
assert(String(Math.round(Number.NaN)), "NaN")

input = 4503599627370497
rounded = Math.round(input)

assert(rounded, 4503599627370497)

input = 0.49999999999999994

assert(rounded, 0)

class Reading(amount):
    this.amount = amount

$Reading.rounded = Math.round($Reading.amount)
$Reading.inverse = 1 / $Reading.rounded

reading = Reading(-0.49999999999999994)

assert(reading.rounded, 0)
assert(reading.inverse, Number.NEGATIVE_INFINITY)

reading.amount = 0.49999999999999994

assert(reading.rounded, 0)
assert(reading.inverse, Number.POSITIVE_INFINITY)

reading.amount = -0.5000000000000001

assert(reading.rounded, -1)
assert(reading.inverse, -1)

---

# Nucleoid indexes and slices strings by Unicode scalar values

accent = String.fromCharCode(233)
symbol = String.fromCharCode(128512)
text = "A" + accent + symbol + "Z"

assert(text.length, 4)
assert(text[1], accent)
assert(text[2], symbol)
assert(text[-1], "Z")
assert(text.charAt(), "A")
assert(text.charAt(1), accent)
assert(text.charAt(2), symbol)
assert(text.charAt(4), "")
assert(text.charCodeAt(1), 233)
assert(text.charCodeAt(2), 128512)
assert(Number.isNaN(text.charCodeAt(4)), true)

assert(text[1:3], accent + symbol)
assert(text[-3:-1], accent + symbol)
assert(text[3:1], "")
assert(text[:], text)
assert(text.slice(), text)
assert(text.slice(1, 3), accent + symbol)
assert(text.substring(1, 3), accent + symbol)
assert(text.substr(1, 3), accent + symbol)
assert(text.slice(-3, -1), accent + symbol)
assert(text.slice(3, 1), "")
assert(text.slice(100), "")
assert(text.slice(-100, 100), text)
assert("".slice(), "")
assert("".charAt(0), "")

index = 1
character = text.charAt(index)
part = text[index:3]

assert(character, accent)
assert(part, accent + symbol)

index = 2

assert(character, symbol)
assert(part, symbol)

---

# Nucleoid restores enclosing null tracking after failed local assignments

def fail():
    throw "EXPECTED"

def recover local value:
    try:
        local = value + fail()
    catch error:
        if error != "EXPECTED":
            throw error
    return local

input = null
result = input + recover(1, 0)

assert(result, null)

input = 2

assert(result, 3)

result = 5 + recover(1, null)

assert(result, 6)

---

# Nucleoid restores enclosing null tracking after failed property assignments

class Item

item = Item()
item.amount = 1

def fail():
    throw "EXPECTED"

def recover value:
    try:
        item.amount = value + fail()
    catch error:
        if error != "EXPECTED":
            throw error
    return item.amount

input = null
result = input + recover(0)

assert(result, null)
assert(item.amount, 1)

input = 2

assert(result, 3)

result = 5 + recover(null)

assert(result, 6)
assert(item.amount, 1)

---

# Nucleoid repeats strings with catchable allocation errors

assert("ab".repeat(2), "abab")
assert("ab".repeat(2.9), "abab")
assert("ab".repeat(0), "")
assert("ab".repeat(-1), "")
assert("ab".repeat(), "")
assert("ab".repeat(null), "")
assert("ab".repeat(Number.NaN), "")
assert("ab".repeat(Number.NEGATIVE_INFINITY), "")
assert("".repeat(Number.MAX_VALUE), "")
assert("".repeat(Number.POSITIVE_INFINITY), "")

symbol = String.fromCharCode(128512)

assert(symbol.repeat(3), symbol + symbol + symbol)

try:
    "ab".repeat(Number.MAX_VALUE)
catch error:
    assert(error, TypeError("Repeated string is too large"))

try:
    "a".repeat(Number.POSITIVE_INFINITY)
catch error:
    assert(error, TypeError("Repeated string is too large"))

count = 2
repeated = "ab".repeat(count)

try:
    count = Number.POSITIVE_INFINITY
catch error:
    assert(error, TypeError("Repeated string is too large"))

assert(count, 2)
assert(repeated, "abab")

count = 3

assert(repeated, "ababab")

---

# Nucleoid preserves declaration order when standing rules are re-evaluated

trigger = 0
enabled = false
result = 0

if enabled and trigger > 0:
    result = 1

if trigger > 0:
    result = 2

enabled = true
trigger = 1

assert(result, 2)

trigger = 0
enabled = false
enabled = true
trigger = 1

assert(result, 2)

# Explicitly declaring the first condition again gives it a new priority
if enabled and trigger > 0:
    result = 3

trigger = 2

assert(result, 3)

---

# Nucleoid settles queued upstream dependencies before their dependents

source = 1
middle = source + 1
last = middle + 1
total = source + last

assert(total, 4)

# The upstream declaration is now newer than its dependents
middle = source + 2

assert(total, 5)

source = 2

assert(middle, 4)
assert(last, 5)
assert(total, 7)

source = 3

assert(total, 9)

---

# Nucleoid settles class property dependencies before validation rules

class Reading(amount):
    this.amount = amount

$Reading.doubled = $Reading.amount * 2
$Reading.margin = $Reading.doubled - $Reading.amount

if $Reading.margin < 0:
    throw "NEGATIVE_MARGIN"

reading = Reading(1)

assert(reading.margin, 1)

$Reading.doubled = $Reading.amount * 3

assert(reading.margin, 2)

# Validation must not see the old doubled value with the new amount
reading.amount = 10

assert(reading.doubled, 30)
assert(reading.margin, 20)

---

# Nucleoid revisits dependencies changed later in the same propagation

first = 0
second = 0
total = first + second

if first > 0 and first < 4:
    second = first + 1

if second > 0 and second < 4:
    first = second + 1

first = 1

assert(first, 3)
assert(second, 4)
assert(total, 7)

if second == 3:
    throw "INVALID_SECOND"

try:
    first = 2
catch error:
    assert(error, "INVALID_SECOND")

assert(first, 3)
assert(second, 4)
assert(total, 7)

# The failed propagation leaves the runtime ready for another update
if second == 3:
    pass

first = 2

assert(first, 4)
assert(second, 3)
assert(total, 7)

---

# Nucleoid stops propagation feedback when assignments keep the same value

first = 0
second = 0

if first > 0:
    second = 1

if second > 0:
    first = 1

first = 1

assert(first, 1)
assert(second, 1)

count = 0
values = [0]

if count > 0 and count < 4:
    values = [Number.NaN]

if Number.isNaN(values[0]):
    count += 1

count = 1

assert(count, 2)
assert(Number.isNaN(values[0]), true)

---

# Nucleoid propagates changes between positive and negative zero

negative = false
zero = 0.0
inverse = 1 / zero

if negative:
    zero = -0.0
else
    zero = 0.0

assert(inverse, Number.POSITIVE_INFINITY)

negative = true

assert(inverse, Number.NEGATIVE_INFINITY)

negative = false

assert(inverse, Number.POSITIVE_INFINITY)

---

# Nucleoid keeps conditional loop bodies imperative

class Item

item = Item()
item.amount = 0
limit = 0

for current of Item:
    if limit > 0:
        current.amount = limit

limit = 2

assert(item.amount, 0)

for current of Item:
    if limit > 0:
        current.amount = limit

assert(item.amount, 2)

limit = 3

assert(item.amount, 2)

---

# Nucleoid cancels propagation queued by a caught failure

trigger = 0
source = 1
observations = 0

if source > 0:
    observations += 1

if trigger > 0:
    try:
        source = 2
        throw "ABORT"
    catch error:
        if error != "ABORT":
            throw error

# This dependent is already queued when the failing body starts
remaining = trigger + 10

trigger = 1

assert(source, 1)
assert(observations, 1)
assert(remaining, 11)

source = 3

assert(observations, 2)

---

# Nucleoid preserves earlier queued work across nested caught failures

trigger = 0
source = 0
observations = 0

if source > 0:
    observations += 1

if trigger > 0:
    source = 1
    try:
        source = 2
        try:
            source = 3
            throw "INNER"
        catch inner:
            if inner != "INNER":
                throw inner
        throw "OUTER"
    catch outer:
        if outer != "OUTER":
            throw outer

trigger = 1

assert(source, 1)
assert(observations, 1)

---

# Nucleoid restores deletion tracking before entering a catch body

trigger = 0
missing = null

if trigger > 0:
    try:
        temporary = 1
        delete temporary
        throw "ABORT"
    catch error:
        if error != "ABORT":
            throw error
        try:
            temporary
        catch absent:
            missing = absent

trigger = 1

assert(missing, ReferenceError("temporary is not defined"))

---

# Nucleoid ends temporary undefined reads when a deletion cascade finishes

trigger = 0
source = 1
dependent = source + 1

if trigger > 0:
    delete source

def read_after_update():
    trigger = 1
    try:
        source
    catch error:
        return error
    return "NO_ERROR"

assert(read_after_update(), ReferenceError("source is not defined"))
assert(dependent, null)

source = 3

assert(dependent, 4)

---

# Nucleoid tracks named functions forwarded as values

def transform number => number * 2

def choose() => transform

callback = transform
forwarded = callback
returned = choose()
direct = forwarded(3)
indirect = returned(3)

assert(direct, 6)
assert(indirect, 6)

def transform number => number * 3

assert(direct, 9)
assert(indirect, 9)

---

# Nucleoid tracks named callbacks over lists and class populations

numbers = [1, 2, 3]

def transform number => number * 2

def matches number => number > 0

def combine total number => total + number

mapped = numbers.map(transform)
filtered = numbers.filter(matches)
found = numbers.find(matches)
all = numbers.every(matches)
any = numbers.some(matches)
total = numbers.reduce(combine, 1)

assert(mapped, [2, 4, 6])
assert(filtered, [1, 2, 3])
assert(found, 1)
assert(all, true)
assert(any, true)
assert(total, 7)

def transform number => number * 3

def matches number => number > 3

def combine total number => total * number

assert(mapped, [3, 6, 9])
assert(filtered, [])
assert(found, null)
assert(all, false)
assert(any, false)
assert(total, 6)

class Sample(amount):
    this.amount = amount

def reading sample => sample.amount * 2

sample = Sample(2)
readings = Sample.map(reading)

assert(readings, [4])

def reading sample => sample.amount * 3

assert(readings, [6])

sample.amount = 3
Sample(4)

assert(readings, [9, 12])

---

# Nucleoid tracks named functions passed to higher order calls

def transform number => number * 2

def apply operation number => operation(number)

input = 3
result = apply(transform, input)

assert(result, 6)

def transform number => number * 3

assert(result, 9)

input = 4

assert(result, 12)

---

# Nucleoid tracks calls through reassigned lambda variables

input = 3
callback = number => number * 2
result = callback(input)
callbacks = [callback]
indexed = callbacks[0](input)

assert(result, 6)
assert(indexed, 6)

callback = number => number * 3

assert(result, 9)
assert(indexed, 9)

input = 4

assert(result, 12)
assert(indexed, 12)

---

# Nucleoid tracks dotted and indexed callable properties consistently

class Handler

handler = Handler()
input = 2
dotted = handler.operation(input)
indexed = handler["operation"](input)

assert(dotted, null)
assert(indexed, null)

handler.operation = number => number + 1

assert(dotted, 3)
assert(indexed, 3)

handler.operation = number => number + 2

assert(dotted, 4)
assert(indexed, 4)

input = 3

assert(dotted, 5)
assert(indexed, 5)

---

# Nucleoid freezes a function definition without freezing its body dependencies

factor = 2

def transform number => number * factor

snapshot = transform.value
frozen = snapshot(3)
live = transform(3)

assert(frozen, 6)
assert(live, 6)

def transform number => number * factor * 10

assert(frozen, 6)
assert(live, 60)

factor = 4

assert(frozen, 12)
assert(live, 120)

---

# Nucleoid defers null function aliases and restores callers after deletion

callback = null
input = 3
offset = 0
result = callback(input) + offset

assert(result, null)

callback = number => number * 2

assert(result, 6)

callback = null

assert(result, null)

callback = number => number * 3

assert(result, 9)

delete callback

assert(result, null)

offset = 1
input = 4

assert(result, null)

missing = null

try:
    callback(3)
catch error:
    missing = error

assert(missing, ReferenceError("callback is not defined"))

trigger = 0

if trigger > 0:
    callback(3)

try:
    trigger = 1
catch error:
    missing = error

assert(missing, ReferenceError("callback is not defined"))
assert(trigger, 0)

callback = number => number * 4

assert(result, 17)

input = 5

assert(result, 21)

---

# Nucleoid restores nullable callable properties without changing frozen references

class Handler

handler = Handler()
handler.operation = number => number + 1
snapshot = handler.operation.value
frozen = snapshot(2)
input = 2
offset = 0
dotted = handler.operation(input) + offset
indexed = handler["operation"](input) + offset

handler.operation = null

assert(dotted, null)
assert(indexed, null)
assert(frozen, 3)

handler.operation = number => number + 2

assert(dotted, 4)
assert(indexed, 4)

delete handler.operation

assert(dotted, null)
assert(indexed, null)
assert(frozen, 3)

offset = 1
input = 3

assert(dotted, null)
assert(indexed, null)

handler.operation = number => number + 3

assert(dotted, 7)
assert(indexed, 7)
assert(frozen, 3)

---

# Nucleoid lets null and non callable parameters shadow outer functions

def operation number => 99

def apply operation number => operation(number)

callback = null
result = apply(callback, 3)

assert(result, null)

callback = number => number * 2

assert(result, 6)

callback = number => number * 3

assert(result, 9)

failure = null

try:
    apply(7, 3)
catch error:
    failure = error

assert(failure, TypeError("operation is not a function"))
assert(operation(3), 99)

---

# Nucleoid rolls back non callable replacements used by existing callers

callback = number => number * 2
result = callback(3)
failure = null

try:
    callback = 7
catch error:
    failure = error

assert(failure, TypeError("callback is not a function"))
assert(result, 6)
assert(callback(4), 8)

class Handler

handler = Handler()
handler.operation = number => number + 1
propertyResult = handler.operation(3)

try:
    handler.operation = 7
catch error:
    failure = error

assert(failure, TypeError("handler.operation is not a function"))
assert(propertyResult, 4)
assert(handler.operation(4), 5)

handler.operation = number => number + 2

assert(propertyResult, 5)

---

# Nucleoid rolls back failed redefinitions of named callbacks

def transform number => number * 2

numbers = [1, 2]
mapped = numbers.map(transform)
failure = null

try:
    def transform number:
        if number == 2:
            throw "REJECTED_CALLBACK"
        return number * 3
catch error:
    failure = error

assert(failure, "REJECTED_CALLBACK")
assert(mapped, [2, 4])
assert(transform(3), 6)

def transform number => number * 4

assert(mapped, [4, 8])

---

# Nucleoid replaces functions and callable variables under the same name

def callback number => number * 2

forwarded = callback
direct = callback(3)
indirect = forwarded(3)

callback = number => number * 3

assert(direct, 9)
assert(indirect, 9)

failure = null

try:
    def callback number:
        throw "REJECTED_BINDING"
catch error:
    failure = error

assert(failure, "REJECTED_BINDING")
assert(direct, 9)
assert(indirect, 9)
assert(callback(4), 12)

def callback number => number * 4

assert(direct, 12)
assert(indirect, 12)

try:
    callback = 7
catch error:
    failure = error

assert(failure, TypeError("callback is not a function"))
assert(direct, 12)
assert(indirect, 12)
assert(callback(4), 16)

callback = null

assert(direct, null)
assert(indirect, null)

def callback number => number * 5

assert(direct, 15)
assert(indirect, 15)

callback = number => number * 6

assert(direct, 18)
assert(indirect, 18)

delete callback

assert(direct, null)
assert(indirect, null)

try:
    callback(3)
catch error:
    failure = error

assert(failure, ReferenceError("callback is not defined"))

def callback number => number * 7

assert(direct, 21)
assert(indirect, 21)

---

# Nucleoid keeps live reasoning impact selections current

# a is a source with no dependents yet
a = 1
exposure = affects a

assert(exposure, [])

# New relationships change the impact without changing a
b = a + 1
c = b * 2

assert(exposure.map(step => step.node), ["b", "c"])

# Replacing a relationship removes and restores its downstream impact
b = 0

assert(exposure, [])

b = a + 1

assert(exposure.map(step => step.node), ["b", "c"])

---

# Nucleoid keeps live reasoning current through equal valued reference changes

class Plan(rate):
    this.rate = rate

class User

basic = Plan(10)
pro = Plan(10)
user = User()
user.plan = basic
user.bill = user.plan.rate * 2
trace = why user.bill

# Repointing the reference changes the derivation but not the bill
user.plan = pro

assert(trace[0].holds, 20)
assert(trace[0].from, ["user", "user.plan", "pro.rate"])
assert(trace.some(step => step.node == "pro.rate"), true)
assert(trace.some(step => step.node == "basic.rate"), false)

pro.rate = 15

assert(trace[0].holds, 30)

---

# Nucleoid keeps live reasoning model selections current

# The empty model is still a live selection
inventory = model |> why

assert(inventory, [])

a = 1
b = a + 1

assert(inventory.map(step => step.node), ["a", "b"])

delete b

assert(inventory.map(step => step.node), ["a"])

b = a + 2
trace = why b

assert(inventory.map(step => step.node), ["a", "b"])
assert(inventory[1].holds, 3)

# An ordinary replacement joins the model; restoring reasoning removes it
trace = a + 0

assert(inventory.map(step => step.node), ["a", "b", "trace"])
assert((affects a).map(step => step.node), ["b", "trace"])

trace = why b

assert(inventory.map(step => step.node), ["a", "b"])
assert((affects a).map(step => step.node), ["b"])

---

# Nucleoid keeps live reasoning subscribed to an absent property

class Item

item = Item()
trace = why item.amount

assert(trace, [])

item.amount = 3

assert(trace[0].holds, 3)
assert(trace[0].node, "item.amount")

delete item.amount

assert(trace, [])

item.amount = 4

assert(trace[0].holds, 4)

---

# Nucleoid keeps live reasoning observers outside impact walks

a = 1
b = a + 1
trace = why b
size = trace.length
exposure = affects a

assert(exposure.map(step => step.node), ["b"])

secondTrace = why b
secondSize = secondTrace.length

assert(exposure.map(step => step.node), ["b"])

---

# Nucleoid keeps live reasoning snapshots fixed

a = 1
exposure = affects a
frozen = exposure.value

b = a + 1

assert(exposure.map(step => step.node), ["b"])
assert(frozen, [])

a = 2

assert(exposure[0].holds, 3)
assert(frozen, [])

---

# Nucleoid restores live reasoning subscriptions after a caught failure

a = 1
alternative = 1
b = a + 1
trace = why b
exposure = affects a

try:
    b = alternative + 1
    throw "REJECTED_REWRITE"
catch error:
    assert(error, "REJECTED_REWRITE")

assert(trace[0].from, ["a"])
assert(exposure.map(step => step.node), ["b"])

alternative = 4

assert(trace[0].holds, 2)

a = 3

assert(trace[0].holds, 4)

b = alternative + 1

assert(trace[0].from, ["alternative"])
assert(exposure, [])

---

# Nucleoid refreshes live reasoning declaration metadata without a value change

a = 1
b = a + 2
trace = why b

b = a + 1 + 1

assert(trace[0].holds, 3)
assert(trace[0].rule, "b = a+1+1")
assert(trace[0].from, ["a"])

b = 3

assert(trace[0].state, "stated")
assert(trace.map(step => step.node), ["b"])

b = a + 2

assert(trace[0].state, "derived")
assert(trace.map(step => step.node), ["b", "a"])

---

# Nucleoid replaces live reasoning ancestry and its subscriptions

a = 1
b = a + 2
c = b * 2
trace = why c
d = 0

b = a + d + 2

assert(trace.length, 4)
assert(trace[0].holds, 6)
assert(trace[1].from, ["a", "d"])

d = 5

assert(trace[0].holds, 16)

b = a + 2
d = 10

assert(trace.length, 3)
assert(trace[0].holds, 6)
assert(trace.some(step => step.node == "d"), false)

---

# Nucleoid preserves declaration order in live reasoning impact selections

source = 1
left = source + 1
right = source + 2
exposure = affects source

assert(exposure.map(step => step.node), ["left", "right"])

left = source + 1

assert(exposure.map(step => step.node), ["right", "left"])

source = 2

assert(exposure.map(step => step.node), ["right", "left"])
assert(exposure[0].holds, 4)
assert(exposure[1].holds, 3)

---

# Nucleoid keeps live reasoning current through function calls

a = 1

def explain() => affects a

exposure = explain()
b = a + 1

assert(exposure.map(step => step.node), ["b"])
assert((affects a).map(step => step.node), ["b"])

b = 0

assert(exposure, [])

---

# Nucleoid keeps class level reasoning properties live

class Reading(level):
    this.level = level

class Audit(target):
    this.target = target

reading = Reading(1)
audit = Audit(reading)

$Audit.exposure = affects $Audit.target.level

assert(audit.exposure, [])

reading.double = reading.level * 2

assert(audit.exposure.map(step => step.node), ["reading.double"])
assert(audit.exposure[0].holds, 2)

reading.level = 3

assert(audit.exposure[0].holds, 6)

reading.double = 6

assert(audit.exposure, [])

---

# Nucleoid enforces live reasoning guards on structural changes

source = 1
limit = 1

if (affects source).length > limit:
    throw "TOO_MANY_DEPENDENTS"

first = source + 1

try:
    second = source + 2
catch error:
    assert(error, "TOO_MANY_DEPENDENTS")

assert((affects source).map(step => step.node), ["first"])

limit = 2
second = source + 2

assert((affects source).map(step => step.node), ["first", "second"])

---

# Nucleoid clears and restores a live explanation when its source is deleted

a = 1
trace = why a

delete a

assert(trace, [])

a = 2

assert(trace[0].holds, 2)
assert(trace[0].node, "a")

# An ordinary replacement stops observing the former source
trace = 99
a = 3

assert(trace, 99)
assert((affects a), [])

---

# Nucleoid preserves queued live reasoning across a caught structural failure

trigger = 0
inventory = model |> why

if trigger > 0:
    try:
        class Temporary
        throw "ABORT"
    catch error:
        if error != "ABORT":
            throw error

remaining = trigger + 10
trigger = 1

assert(inventory.some(step => step.node == "$Temporary"), false)
assert(inventory.find(step => step.node == "trigger").holds, 1)
assert(inventory.find(step => step.node == "remaining").holds, 11)

---

# Nucleoid treats this and self as interchangeable instance keywords

class Counter(count: int):
    self.count = count

    def same_receiver() => this == self

    def set_this(count: int):
        this.count = count
        return self.count

    def set_self(count: int):
        self.count = count
        return this.count

class SpecialCounter: Counter
    def init(count: int):
        super(count)
        this.kind = "SPECIAL"
        self.active = true

$Counter.doubled = this.count + self.count

counter = Counter(2)
other = Counter(5)
special = SpecialCounter(7)

assert(counter.count, 2)
assert(other.count, 5)
assert(special.count, 7)
assert(special.kind, "SPECIAL")
assert(special.active, true)
assert(counter.same_receiver(), true)
assert(other.same_receiver(), true)
assert(counter.doubled, 4)
assert(other.doubled, 10)
assert(special.doubled, 14)

assert(counter.set_self(3), 3)
assert(counter.count, 3)
assert(counter.doubled, 6)
assert(other.count, 5)

assert(other.set_this(7), 7)
assert(other.count, 7)
assert(other.doubled, 14)
assert(counter.count, 3)

---

# Nucleoid rejects this and self outside an instance or as assignment targets

try:
    this
catch error:
    assert(error, ReferenceError("this is not defined"))

try:
    self
catch error:
    assert(error, ReferenceError("this is not defined"))

try:
    this = 1
catch error:
    assert(error, TypeError("Cannot assign to 'this'"))

try:
    self = 1
catch error:
    assert(error, TypeError("Cannot assign to 'this'"))

---

# Nucleoid keeps expression bodied named functions reactive

source = 3

def next number: int => number + 1

def twice number: int => next(number) * 2

result = twice(source)
assert(result, 8)
source = 5
assert(result, 12)

def next number: int => number + 2

assert(result, 14)

---

# Nucleoid keeps expression bodied methods and initializers equivalent

class Gauge:
    def init amount: int => this.amount = amount

    def reading() => self.amount

    def set amount: int => this.amount = amount

gauge = Gauge(4)
observed = gauge.reading()
assert(gauge.id, "gauge")
assert(gauge.amount, 4)
assert(observed, 4)
assert(gauge.set(7), 7)
assert(gauge.amount, 7)
assert(observed, 7)

---

# Nucleoid preserves expression body returns and callable references

def literal() => 42

reference = literal
frozen = literal.value
assert(reference(), 42)

def literal() => 43

assert(reference(), 43)
assert(frozen(), 42)

def assigned value => value = 2

assert(assigned(99), 2)

def mapper() => value => value * 2

callback = mapper()
assert(callback(3), 6)

def no_return():
    42

assert(no_return(), null)

---

# Nucleoid keeps compound arithmetic assignments equivalent
value = 20
value += 4
assert(value, 24)
value -= 6
assert(value, 18)
value *= 2 + 1
assert(value, 54)
value /= 3
assert(value, 18)
value %= 5
assert(value, 3)
text = "A"
text += "B"
assert(text, "AB")
ratio = 2
ratio *= -3
assert(ratio, -6)
empty = null
empty += 1
assert(empty, null)

---

# Nucleoid keeps compound assignments reactive
amount = 10
step = 2
amount += step
doubled = amount * 2
assert(amount, 12)
assert(doubled, 24)
step = 3
assert(amount, 15)
assert(doubled, 30)
amount += step.value
assert(amount, 18)
step = 5
assert(amount, 18)
assert(doubled, 36)

---

# Nucleoid keeps compound property and local assignments equivalent
class Meter(reading: int):
    this.reading = reading
    def adjust(delta: int):
        delta *= 2
        this.reading += delta
        return this.reading
meter1 = Meter(10)
meter1.reading += 2
assert(meter1.reading, 12)
assert(meter1.adjust(3), 18)
assert(meter1.reading, 18)
class Scale
scale1 = Scale()
$Scale.factor = 2
try:
    $Scale.factor *= 3
catch error:
    assert(error, TypeError("Circular Dependency"))
assert(scale1.factor, 2)

---

# Nucleoid rolls back failed compound assignments
balance = 10
step = 2
if balance > 15:
    throw "LIMIT"
try:
    balance += step * 4
catch error:
    assert(error, "LIMIT")
assert(balance, 10)
step = 3
assert(balance, 10)
balance += step
assert(balance, 13)
first = 2
second = first * 2
try:
    first += second
catch error:
    assert(error, TypeError("Circular Dependency"))
assert(first, 2)
assert(second, 4)
try:
    missing += 1
catch error:
    assert(error, ReferenceError("missing is not defined"))

---

# Nucleoid returns compound assignment expressions
def add current delta => current += delta
def subtract current delta => current -= delta
def multiply current delta => current *= delta
def divide current delta => current /= delta
def remainder current delta => current %= delta
def combined a b => a += b *= 2
assert(add(5, 3), 8)
assert(subtract(5, 3), 2)
assert(multiply(5, 3), 15)
assert(divide(6, 3), 2)
assert(remainder(5, 3), 2)
assert(combined(1, 3), 7)

---

# Nucleoid keeps bodyless class declarations equivalent
class Entity
assert(typeof $Entity, Class)
assert(typeof Entity, List)
assert(Class.length, 1)
assert(Entity.length, 0)
entity1 = Entity()
assert(entity1, {id: "entity1"})
assert(Entity.length, 1)

---

# Nucleoid keeps rules reactive on bodyless classes
class Item
rate = 2
$Item.price = $Item.quantity * rate
item1 = Item()
assert(item1.price, null)
item1.quantity = 3
assert(item1.price, 6)
item2 = Item()
item2.quantity = 4
assert(item2.price, 8)
rate = 3
assert(item1.price, 9)
assert(item2.price, 12)
item1.quantity = 5
assert(item1.price, 15)
assert(item2.price, 12)
assert(Item.length, 2)

---

# Nucleoid preserves parameters and scope in bodyless classes
class Shell(code: int)
first = Shell(5)
second = Shell()
assert(first, {id: "first"})
assert(first.code, null)
assert(second.code, null)
assert(Shell.length, 2)
{
    class Token
    token1 = Token()
    assert(Token[0], token1)
}
assert(Token.length, 1)
assert(Class.length, 2)

---

# Nucleoid preserves bodyless class populations and inheritance
class Parent
first = Parent()
assert(Parent.length, 1)
class Parent
assert(Class.length, 1)
assert(Parent.length, 1)
$Parent.label = "BASE"
class Child: Parent
    pass
child1 = Child()
assert(Class.length, 2)
assert(Parent.length, 1)
assert(Child.length, 1)
assert(first.label, "BASE")
assert(child1.label, "BASE")

---

# Nucleoid rolls back creation of bodyless class instances
class Limited
count = Limited.length
if count > 1:
    throw "LIMIT"
limited1 = Limited()
try:
    limited2 = Limited()
catch error:
    assert(error, "LIMIT")
assert(Limited.length, 1)
assert(count, 1)
assert(limited1, {id: "limited1"})
try:
    limited2
catch error:
    assert(error, ReferenceError("limited2 is not defined"))

---

# Nucleoid invokes a parent constructor with bare super on the same instance

class Parent:
    def init():
        this.amount = 4

class Child: Parent
    def init():
        super
        this.amount = this.amount + 1

child = Child()

assert(child.amount, 5)
assert(child.id, "child")
assert(Parent.length, 0)
assert(Child.length, 1)

---

# Nucleoid resolves bare super through the executing constructor chain

class Base:
    def init():
        this.trace = "A"

class Middle: Base
    def init():
        super
        this.trace = this.trace + "B"

class Leaf: Middle
    def init():
        super
        this.trace = this.trace + "C"

class Passive: Leaf
    pass

leaf = Leaf()
passive = Passive()

assert(leaf.trace, "ABC")
assert(passive.trace, "ABC")
assert(Base.length, 0)
assert(Middle.length, 0)
assert(Leaf.length, 1)
assert(Passive.length, 1)

---

# Nucleoid preserves omitted parent arguments with bare super

class Parent:
    def init amount:
        this.amount = amount

class Child: Parent
    def init():
        super

class Explicit: Parent
    def init amount:
        super(amount)

child = Child()
explicit = Explicit(9)

assert(child.amount, null)
assert(explicit.amount, 9)

---

# Nucleoid preserves bare super expression and receiver behavior

class Parent:
    def init():
        this.amount = 3

class Child: Parent
    def init() => super
    def reset() => super

child = Child()

assert(child.amount, 3)
child.amount = 9
assert(child.reset(), null)
assert(child.amount, 3)

class Root:
    def init():
        this.result = super

root = Root()

assert(root.result, null)

try:
    super
catch error:
    assert(error, ReferenceError("super is not defined"))

---

# Nucleoid rolls back a failing parent constructor called with bare super

kept = { amount: 5 }

class Parent:
    def init():
        kept.amount = 9
        this.amount = 7
        throw "BLOCKED"

class Child: Parent
    def init():
        super

try:
    failed = Child()
catch error:
    assert(error, "BLOCKED")

assert(kept.amount, 5)
assert(Parent.length, 0)
assert(Child.length, 0)

try:
    failed
catch error:
    assert(error, ReferenceError("failed is not defined"))

---

# Nucleoid keeps shorthand object fields reactive

price = 4
quantity = 3
order = { price, quantity }
total = order.price * order.quantity

assert(order.price, 4)
assert(order.quantity, 3)
assert(total, 12)

price = 5

assert(order.price, 5)
assert(total, 15)

quantity = 2

assert(order.quantity, 2)
assert(total, 10)

---

# Nucleoid resolves shorthand object fields in their lexical scope

label = "outer"

def pack label =>
    { label }

local = pack("inner")

assert(local.label, "inner")

label = "changed"

assert(local.label, "inner")

builder = amount => ({ amount })
built = builder(7)

assert(built.amount, 7)

def wrap amount => { amount }

wrapped = wrap(5)

assert(wrapped.amount, 5)

class Parcel:
    def init amount:
        this.payload = { amount }

parcel = Parcel(8)

assert(parcel.payload.amount, 8)

---

# Nucleoid preserves snapshots and undefined reads in shorthand objects

amount = 2
live = { amount }
frozen = { amount: amount.value }

assert(live.amount, 2)
assert(frozen.amount, 2)

amount = 5

assert(live.amount, 5)
assert(frozen.amount, 2)

try:
    pending = { missing }
catch error:
    assert(error, ReferenceError("missing is not defined"))

missing = 8
pending = { missing }

assert(pending.missing, 8)

---

# Nucleoid preserves mixed nested and duplicate shorthand object fields

amount = 3
label = "A"
other = 7
mixed = ({
    amount,
    label,
    "quoted-key": label,
    1: amount,
    amount: other,
})

assert(mixed.amount, 7)
assert(mixed.label, "A")
assert(mixed["quoted-key"], "A")
assert(mixed[1], 3)

amount = 5

assert(mixed.amount, 7)
assert(mixed[1], 5)

other = 9

assert(mixed.amount, 9)

nested = { mixed, items: [{ amount }] }

assert(nested.mixed.amount, 9)
assert(nested.items[0].amount, 5)

---

# Nucleoid restores shorthand object dependencies after a failed transaction

amount = 3
alternate = 7
record = { amount }
derived = record.amount * 2

try:
    record = { alternate }
    assert(record.alternate, 7)
    throw "ABORT"
catch error:
    assert(error, "ABORT")

assert(record.amount, 3)
assert(derived, 6)

alternate = 9

assert(record.amount, 3)

amount = 5

assert(record.amount, 5)
assert(derived, 10)

---

# Nucleoid keeps bare single parameter named arrows reactive

base = 2

def scaled amount => amount + base

result = scaled(3)

assert(result, 5)

base = 4

assert(result, 7)

def scaled amount => amount * base

assert(result, 12)

---

# Nucleoid preserves callable references for bare single parameter named arrows

def change amount => amount + 1

live = change
snapshot = change.value

assert(live(2), 3)
assert(snapshot(2), 3)

def change amount => amount * 3

assert(live(2), 6)
assert(snapshot(2), 3)

---

# Nucleoid preserves bare single parameter methods and initializers

class Gauge:
    def init amount => this.amount = amount
    def reading offset => self.amount + offset

gauge = Gauge(4)

assert(gauge.amount, 4)
assert(gauge.reading(3), 7)

gauge.amount = 6

assert(gauge.reading(3), 9)

---

# Nucleoid keeps bare single parameter binding and explicit alternatives equivalent

amount = 10

def identity amount => amount

assert(identity(3), 3)
assert(identity(), null)
assert(identity(3, 4), 3)
assert(amount, 10)

def typed(amount: int) => amount + 1

def block(amount) =>
    amount + 2

assert(typed(4), 5)
assert(block(4), 6)

---

# Nucleoid rolls back failed bare single parameter arrow redefinitions

source = 3

def reading amount => source + amount

total = reading(2)

assert(total, 5)

try:
    def reading amount => missing + amount
catch error:
    assert(error, ReferenceError("missing is not defined"))

assert(total, 5)

source = 4

assert(total, 6)

---

# Nucleoid keeps bare single parameter block definitions reactive

base = 2

def scaled amount =>
    amount + base

result = scaled(3)

assert(result, 5)

base = 4

assert(result, 7)

def scaled amount =>
    amount * base

assert(result, 12)

---

# Nucleoid preserves callable references for bare single parameter block definitions

def change amount =>
    amount + 1

live = change
snapshot = change.value

assert(live(2), 3)
assert(snapshot(2), 3)

def change amount =>
    amount * 3

assert(live(2), 6)
assert(snapshot(2), 3)

---

# Nucleoid preserves bare single parameter block methods and initializers

class Gauge:
    def init amount:
        this.amount = amount
    def reading offset =>
        self.amount + offset

gauge = Gauge(4)

assert(gauge.amount, 4)
assert(gauge.reading(3), 7)

gauge.amount = 6

assert(gauge.reading(3), 9)

---

# Nucleoid keeps bare single parameter block binding and explicit alternatives equivalent

amount = 10

def identity amount =>
    amount

def ignored amount:
    amount + 100

def typed(amount: int) =>
    amount + 1

def pair left right =>
    left + right

def unit() =>
    42

def arrow amount => amount + 3

assert(identity(3), 3)
assert(identity(), null)
assert(identity(3, 4), 3)
assert(amount, 10)
assert(ignored(5), null)
assert(typed(4), 5)
assert(pair(2, 3), 5)
assert(unit(), 42)
assert(arrow(4), 7)
assert(amount, 10)

---

# Nucleoid rolls back failed bare single parameter block redefinitions

source = 3

def reading amount =>
    source + amount

total = reading(2)

assert(total, 5)

try:
    def reading amount =>
        missing + amount
catch error:
    assert(error, ReferenceError("missing is not defined"))

assert(total, 5)

source = 4

assert(total, 6)

---

# Nucleoid keeps bare typed single parameter named arrows reactive

base = 2

def scaled amount: int => amount + base

result = scaled(3)

assert(result, 5)

base = 4

assert(result, 7)

def scaled amount: int => amount * base

assert(result, 12)

---

# Nucleoid preserves callable references for bare typed single parameter named arrows

def change amount: int => amount + 1

live = change
snapshot = change.value
first = live(2)
old = snapshot(2)

assert(first, 3)
assert(old, 3)

def change amount: int => amount * 2

assert(first, 4)
assert(old, 3)

---

# Nucleoid preserves bare typed single parameter methods and initializers

class Gauge:
    def init amount: int => this.amount = amount
    def reading offset: int => self.amount + offset

gauge = Gauge(4)

assert(gauge.amount, 4)
assert(gauge.reading(3), 7)

gauge.amount = 6

assert(gauge.reading(3), 9)

---

# Nucleoid keeps bare typed single parameter binding and explicit alternatives equivalent

amount = 10

def identity amount: ArbitraryType => amount
def text label: str => label
def bare value => value + 1
def block(amount: int) =>
    amount + 2
def pair left: int, right: int => left + right
def unit() => 42

assert(identity(3), 3)
assert(identity(), null)
assert(identity(3, 4), 3)
assert(identity("text"), "text")
assert(text("hello"), "hello")
assert(amount, 10)
assert(bare(4), 5)
assert(block(4), 6)
assert(pair(2, 3), 5)
assert(unit(), 42)

---

# Nucleoid rolls back failed bare typed single parameter arrow redefinitions

source = 3

def reading amount: int => source + amount

total = reading(2)

try:
    def reading amount: int => missing + amount
catch error:
    assert(error, ReferenceError("missing is not defined"))

assert(total, 5)

source = 4

assert(total, 6)
assert(reading(3), 7)

---

# Nucleoid keeps numeric clamping reactive to all three inputs

value = 12
lower = 0
upper = 10

clipped = Math.clamp(value, lower, upper)

assert(clipped, 10)

value = 5

assert(clipped, 5)

lower = 7

assert(clipped, 7)

upper = 6

assert(clipped, 6)

lower = 0
value = -3

assert(clipped, 0)

upper = 20
value = 17

assert(clipped, 17)

---

# Nucleoid preserves numeric coercion while clamping values

numeric = Math.clamp("7", 0, 5)
boolean = Math.clamp(true, 0, 2)
nil = Math.clamp(null, -1, 1)
empty = Math.clamp([], 1, 3)
singleton = Math.clamp(["4"], 0, 3)
invalid = Math.clamp("invalid", 0, 5)

assert(numeric, 5)
assert(boolean, 1)
assert(nil, 0)
assert(empty, 1)
assert(singleton, 3)
assert(String(invalid), "NaN")

---

# Nucleoid preserves nonfinite equal reversed and omitted clamp bounds

positive = Math.clamp(Number.POSITIVE_INFINITY, -10, 10)
negative = Math.clamp(Number.NEGATIVE_INFINITY, -10, 10)
unbounded = Math.clamp(Number.POSITIVE_INFINITY, Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY)
equal = Math.clamp(3, 4, 4)
reversed = Math.clamp(5, 10, 0)
nan_input = Math.clamp(Number.NaN, 0, 10)
nan_lower = Math.clamp(2, Number.NaN, 10)
nan_upper = Math.clamp(2, 0, Number.NaN)
missing_all = Math.clamp()
missing_lower = Math.clamp(2)
missing_upper = Math.clamp(2, 0)
extra = Math.clamp(3, 0, 5, 99)

assert(positive, 10)
assert(negative, -10)
assert(unbounded, Number.POSITIVE_INFINITY)
assert(equal, 4)
assert(reversed, 0)
assert(String(nan_input), "NaN")
assert(String(nan_lower), "NaN")
assert(String(nan_upper), "NaN")
assert(String(missing_all), "NaN")
assert(String(missing_lower), "NaN")
assert(String(missing_upper), "NaN")
assert(extra, 3)

---

# Nucleoid applies numeric clamping to live class rules

class Reading(amount):
    this.amount = amount

$Reading.bounded = Math.clamp($Reading.amount, 0, 10)

first = Reading(12)
second = Reading(-3)

assert(first.bounded, 10)
assert(second.bounded, 0)

first.amount = 4

assert(first.bounded, 4)

second.amount = 8

assert(second.bounded, 8)

---

# Nucleoid preserves deferred reads and rollback for numeric clamping

class Sensor
sensor = Sensor()

observed = Math.clamp(sensor.amount, 0, 10)

assert(observed, null)

sensor.amount = 12

assert(observed, 10)

delete sensor.amount

assert(observed, null)

sensor.amount = 3

assert(observed, 3)

try:
    observed = Math.clamp(missing, 0, 10)
catch error:
    assert(error, ReferenceError("missing is not defined"))

assert(observed, 3)

sensor.amount = 6

assert(observed, 6)

---

# Nucleoid keeps numeric interpolation reactive to all three inputs

start = 0
end = 10
weight = 0.25

position = Math.lerp(start, end, weight)

assert(position, 2.5)

start = 2

assert(position, 4)

end = 18

assert(position, 6)

weight = 0.5

assert(position, 10)

weight = -0.5

assert(position, -6)

weight = 1.5

assert(position, 26)

---

# Nucleoid preserves numeric coercion while interpolating values

numeric = Math.lerp(" 2 ", "10", "0.25")
boolean = Math.lerp(false, true, true)
nil = Math.lerp(null, 4, 0.5)
empty = Math.lerp([], 4, 0.5)
singleton = Math.lerp(["2"], ["10"], ["0.25"])
date = Math.lerp(Date(0), Date(10), 0.5)
invalid = Math.lerp("invalid", 10, 0.5)

assert(numeric, 4)
assert(boolean, 1)
assert(nil, 2)
assert(empty, 2)
assert(singleton, 4)
assert(date, 5)
assert(String(invalid), "NaN")

---

# Nucleoid preserves ordered special values and omitted interpolation inputs

left = Math.lerp(2, 10, 0)
right = Math.lerp(2, 10, 1)
equal = Math.lerp(4, 4, 2)
reversed = Math.lerp(10, 2, 0.25)
extrapolated = Math.lerp(2, 10, 2)
positive = Math.lerp(0, Number.POSITIVE_INFINITY, 0.5)
negative = Math.lerp(0, Number.NEGATIVE_INFINITY, 0.5)
zero_infinite = Math.lerp(0, Number.POSITIVE_INFINITY, 0)
equal_infinite = Math.lerp(Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY, 0.5)
nan_start = Math.lerp(Number.NaN, 10, 0.5)
nan_end = Math.lerp(2, Number.NaN, 0.5)
nan_weight = Math.lerp(2, 10, Number.NaN)
missing_all = Math.lerp()
missing_end = Math.lerp(2)
missing_weight = Math.lerp(2, 10)
extra = Math.lerp(2, 10, 0.5, 99)

assert(left, 2)
assert(right, 10)
assert(equal, 4)
assert(reversed, 8)
assert(extrapolated, 18)
assert(positive, Number.POSITIVE_INFINITY)
assert(negative, Number.NEGATIVE_INFINITY)
assert(String(zero_infinite), "NaN")
assert(String(equal_infinite), "NaN")
assert(String(nan_start), "NaN")
assert(String(nan_end), "NaN")
assert(String(nan_weight), "NaN")
assert(String(missing_all), "NaN")
assert(String(missing_end), "NaN")
assert(String(missing_weight), "NaN")
assert(extra, 6)

---

# Nucleoid applies numeric interpolation to live class rules

class Blend(start, end, weight):
    this.start = start
    this.end = end
    this.weight = weight

$Blend.position = Math.lerp($Blend.start, $Blend.end, $Blend.weight)

first = Blend(0, 10, 0.25)
second = Blend(10, 0, 0.5)

assert(first.position, 2.5)
assert(second.position, 5)

first.weight = 0.75

assert(first.position, 7.5)

second.end = 20

assert(second.position, 15)

---

# Nucleoid preserves deferred reads and rollback for numeric interpolation

class Sensor
sensor = Sensor()
end = 10
weight = 0.5

position = Math.lerp(sensor.start, end, weight)

assert(position, null)

sensor.start = 2

assert(position, 6)

delete sensor.start

assert(position, null)

sensor.start = 4

assert(position, 7)

try:
    position = Math.lerp(missing, end, weight)
catch error:
    assert(error, ReferenceError("missing is not defined"))

assert(position, 7)

end = 20

assert(position, 12)

---

# Nucleoid keeps unit-interval saturation reactive to its input

reading = -2
bounded = Math.saturate(reading)

assert(bounded, 0)

reading = 0.25

assert(bounded, 0.25)

reading = 3

assert(bounded, 1)

reading = 0.5

assert(bounded, 0.5)

reading = 1

assert(bounded, 1)

reading = -1

assert(bounded, 0)

---

# Nucleoid preserves numeric coercion while saturating values

text = " 0.375 "
from_text = Math.saturate(text)

assert(from_text, 0.375)

text = "-3"

assert(from_text, 0)

text = "2"

assert(from_text, 1)

empty = Math.saturate("")
from_false = Math.saturate(false)
from_true = Math.saturate(true)
from_empty_list = Math.saturate([])
from_list = Math.saturate([0.625])
bad_list = Math.saturate([0, 1])
bad_text = Math.saturate("not a number")
literal_null = Math.saturate(null)
from_date = Math.saturate(Date(1))

assert(empty, 0)
assert(from_false, 0)
assert(from_true, 1)
assert(from_empty_list, 0)
assert(from_list, 0.625)
assert(Number.isNaN(bad_list), true)
assert(Number.isNaN(bad_text), true)
assert(literal_null, 0)
assert(from_date, 1)

---

# Nucleoid preserves special values and omitted unit saturation inputs

positive = Math.saturate(1 / 0)
negative = Math.saturate(-1 / 0)
invalid = Math.saturate(0 / 0)
positive_zero = Math.saturate(0)
negative_zero = Math.saturate(-0)
omitted = Math.saturate()
extra = Math.saturate(0.5, 99)
tiny = Math.saturate(Number.MIN_VALUE)

assert(positive, 1)
assert(negative, 0)
assert(Number.isNaN(invalid), true)
assert(1 / positive_zero, 1 / Math.min(Math.max(0, 0), 1))
assert(1 / negative_zero, 1 / Math.min(Math.max(-0, 0), 1))
assert(Number.isNaN(omitted), true)
assert(extra, 0.5)
assert(tiny, Number.MIN_VALUE)

---

# Nucleoid applies unit-interval saturation to live class rules

class Level(reading):
    this.reading = reading

$Level.bounded = Math.saturate($Level.reading)

first = Level(-2)
second = Level(0.4)

assert(first.bounded, 0)
assert(second.bounded, 0.4)

first.reading = 0.8

assert(first.bounded, 0.8)

second.reading = 2

assert(second.bounded, 1)

---

# Nucleoid preserves deferred reads and rollback for unit saturation

class Sensor
sensor = Sensor()

bounded = Math.saturate(sensor.reading)

assert(bounded, null)

sensor.reading = -2

assert(bounded, 0)

delete sensor.reading

assert(bounded, null)

sensor.reading = 0.75

assert(bounded, 0.75)

try:
    bounded = Math.saturate(missing)
catch error:
    assert(error, ReferenceError("missing is not defined"))

assert(bounded, 0.75)

sensor.reading = 2

assert(bounded, 1)

---

# Nucleoid keeps bare typed single parameter block definitions reactive

base = 2

def scaled amount: int =>
    amount + base

result = scaled(3)

assert(result, 5)

base = 4

assert(result, 7)

def scaled amount: int =>
    amount * base

assert(result, 12)

---

# Nucleoid preserves callable references for bare typed single parameter block definitions

base = 1

def change amount: ArbitraryType =>
    amount + base

live = change
snapshot = change.value
first = live(2)
old = snapshot(2)

assert(first, 3)
assert(old, 3)

base = 3

assert(first, 5)
assert(old, 5)

def change amount: int =>
    amount * base

assert(first, 6)
assert(old, 5)

base = 4

assert(first, 8)
assert(old, 6)

---

# Nucleoid preserves bare typed single parameter block methods and initializers

class Gauge:
    def init amount: int:
        this.amount = amount
    def reading offset: int =>
        self.amount + offset

gauge = Gauge(4)

assert(gauge.amount, 4)
assert(gauge.reading(3), 7)

gauge.amount = 6

assert(gauge.reading(3), 9)

---

# Nucleoid keeps bare typed single parameter block binding and explicit alternatives equivalent

amount = 10

def identity amount: ArbitraryType =>
    amount

def text label: str =>
    label

def ignored amount: int:
    amount + 100

def bare value =>
    value + 1

def typed(amount: int) =>
    amount + 2

def arrow value: int => value + 3

def pair(left: int, right: int) =>
    left + right

def unit() =>
    42

assert(identity(3), 3)
assert(identity(), null)
assert(identity(3, 4), 3)
assert(identity("text"), "text")
assert(text("hello"), "hello")
assert(ignored(5), null)
assert(bare(4), 5)
assert(typed(4), 6)
assert(arrow(4), 7)
assert(pair(2, 3), 5)
assert(unit(), 42)
assert(amount, 10)

---

# Nucleoid rolls back failed bare typed single parameter block redefinitions

source = 3

def reading amount: int =>
    source + amount

total = reading(2)

assert(total, 5)

try:
    def reading amount: int =>
        missing + amount
catch error:
    assert(error, ReferenceError("missing is not defined"))

assert(total, 5)

source = 4

assert(total, 6)
assert(reading(3), 7)

---

# Nucleoid keeps logical compound assignments equivalent

value = 0
value ||= 7
assert(value, 7)
value &&= 9
assert(value, 9)
label = ""
label ||= "ready"
assert(label, "ready")
label &&= "done"
assert(label, "done")
off = false
off &&= "unused"
assert(off, false)
on = true
on ||= "unused"
assert(on, true)
grouped = false
grouped ||= false && true
assert(grouped, false)
value &&= 0 || 3
assert(value, 3)

---

# Nucleoid keeps logical compound assignments reactive

enabled = false
fallback = false
enabled ||= fallback
assert(enabled, false)
fallback = true
assert(enabled, true)
fallback = false
assert(enabled, true)
ready = true
limit = true
ready &&= limit
assert(ready, true)
limit = false
assert(ready, false)
limit = true
assert(ready, false)
snapshot = false
source = true
snapshot ||= source.value
assert(snapshot, true)
source = false
assert(snapshot, true)

---

# Nucleoid short circuits logical compound operands

def fail():
    throw "UNREACHABLE"
retained = "ready"
retained ||= fail()
assert(retained, "ready")
stopped = false
stopped &&= fail()
assert(stopped, false)
chosen = false
chosen ||= 12
assert(chosen, 12)
chosen &&= 4
assert(chosen, 4)

---

# Nucleoid preserves logical compound property and local assignments

class Gate:
    def allow(fallback):
        this.enabled ||= fallback
        return this.enabled
    def require(allowed):
        self.enabled &&= allowed
        return self.enabled
gate = Gate()
gate.enabled = false
assert(gate.allow(true), true)
assert(gate.require(false), false)
assert(gate.enabled, false)
def prefer value fallback => value ||= fallback
def requireBoth value allowed => value &&= allowed
assert(prefer(0, 8), 8)
assert(prefer(3, 8), 3)
assert(requireBoth(2, 7), 7)
assert(requireBoth(0, 7), 0)
def combined a b => a ||= b &&= 9
assert(combined(false, 2), 9)
assert(combined(true, 2), true)
combined(false, 2)

# return: 9

---

# Nucleoid rolls back failed logical compound assignments

enabled = false
if enabled:
    throw "BLOCKED"
caught = null
try:
    enabled ||= true
catch error:
    caught = error
assert(caught, "BLOCKED")
assert(enabled, false)
ready = true
if !ready:
    throw "REQUIRED"
caught = null
try:
    ready &&= false
catch error:
    caught = error
assert(caught, "REQUIRED")
assert(ready, true)

---

# Nucleoid defines bare multiple parameter named arrows

def add left right => left + right
def scaled left right factor => (left + right) * factor
def pair left right => [left, right]
def coordinates x y => {x, y}
assert(add(2, 3), 5)
assert(scaled(2, 3, 4), 20)
assert(pair("left", "right"), ["left", "right"])
point = coordinates(2, 3)
assert(point.x, 2)
assert(point.y, 3)

---

# Nucleoid keeps bare multiple parameter arrows reactive

bias = 1
def total left right => left + right + bias
tracked = total(2, 3)
assert(tracked, 6)
bias = 4
assert(tracked, 9)
def projected first second third => first * second + third
left = 2
right = 3
offset = 4
result = projected(left, right, offset)
assert(result, 10)
left = 5
assert(result, 19)
right = 2
assert(result, 14)

---

# Nucleoid preserves bare multiple parameter methods

class Calculator:
    def combine left right => this.base + left + right
    def product left right scale => self.base + left * right * scale
calculator = Calculator()
calculator.base = 2
result = calculator.combine(3, 4)
assert(result, 9)
calculator.base = 5
assert(result, 12)
assert(calculator.product(2, 3, 4), 29)

---

# Nucleoid preserves nested bare multiple parameter arrows

def outer(initial):
    def combine left right scale => (left + right) * scale
    return combine(initial, 2, 3)
assert(outer(4), 18)
def factory(ignored) => (left, right) => left + right
custom = factory(10)
assert(custom(2, 3), 5)

---

# Nucleoid returns bare multiple parameter named arrows

def prefer current fallback => current ||= fallback
def requireBoth current condition => current &&= condition
def combined first second scale => (first + second) * scale
assert(prefer(0, 9), 9)
assert(prefer(3, 9), 3)
assert(requireBoth(2, 7), 7)
assert(requireBoth(0, 7), 0)
assert(combined(2, 5, 3), 21)
combined(2, 5, 3)

# return: 21

---

# Nucleoid defines bare multiple parameter indented blocks

def add left right:
    total = left + right
    return total
def scaled left right factor =>
    (left + right) * factor
def largest left right:
    if left > right:
        return left
    return right
assert(add(2, 3), 5)
assert(scaled(2, 3, 4), 20)
assert(largest(7, 3), 7)
assert(largest(2, 9), 9)

---

# Nucleoid keeps bare multiple parameter blocks reactive

bias = 2
def scaled left right =>
    left * right + bias
live = scaled
snapshot = scaled.value
result = live(3, 4)
frozen = snapshot(3, 4)
assert(result, 14)
assert(frozen, 14)
def scaled left right =>
    left + right + bias
assert(result, 9)
assert(frozen, 14)
bias = 4
assert(result, 11)
assert(frozen, 16)

---

# Nucleoid preserves bare multiple parameter block methods and initializers

class Parent:
    def init left right:
        this.amount = left + right
    def reading offset scale =>
        self.amount * scale + offset
class Child: Parent
    def init left right:
        super(left, right)
    def reading offset scale =>
        self.amount * scale + offset
    def increase delta extra:
        this.amount += delta + extra
        return this.amount
child = Child(2, 3)
observed = child.reading(1, 2)
assert(child.id, "child")
assert(child.amount, 5)
assert(observed, 11)
assert(child.increase(2, 1), 8)
assert(child.amount, 8)
assert(observed, 17)
assert(Parent.length, 0)
assert(Child.length, 1)

---

# Nucleoid preserves bare multiple parameter block bindings and returns

left = 10
right = 20
def pair left right =>
    [left, right]
def add left right:
    left += right
    return left
def noReturn left right:
    left + right
def bareReturn left right:
    return
def outer(initial):
    def combine left right scale =>
        (left + right) * scale
    return combine(initial, 2, 3)
assert(pair(3, 4), [3, 4])
assert(pair(3), [3, null])
assert(pair(), [null, null])
assert(pair(3, 4, 5), [3, 4])
assert(add(3, 4), 7)
assert(left, 10)
assert(right, 20)
assert(noReturn(2, 3), null)
assert(bareReturn(2, 3), null)
assert(outer(4), 18)

---

# Nucleoid rolls back failed bare multiple parameter block redefinitions

source = 3
def reading left right =>
    source + left + right
total = reading(2, 4)
assert(total, 9)
try:
    def reading left right =>
        missing + left + right
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(total, 9)
source = 4
assert(total, 10)
assert(reading(8, 9), 21)
reading(8, 9)

# return: 21

---

# Nucleoid defines bare typed multiple parameter named arrows

def add left: int, right: int => left + right
def scaled left: number, right, factor: number => (left + right) * factor
def typedLast left, right: number => left + right
def typedFirst left: number, right => left + right
assert(add(2, 3), 5)
assert(scaled(2, 3, 4), 20)
assert(typedLast(4, 5), 9)
assert(typedFirst(6, 7), 13)

---

# Nucleoid keeps bare typed multiple parameter arrows reactive

bias = 2
def scaled left: int, right: int => left * right + bias
live = scaled
snapshot = scaled.value
result = live(3, 4)
frozen = snapshot(3, 4)
assert(result, 14)
assert(frozen, 14)
def scaled left: int, right: int => left + right + bias
assert(result, 9)
assert(frozen, 14)
bias = 4
assert(result, 11)
assert(frozen, 16)

---

# Nucleoid preserves bare typed multiple parameter methods and initializers

class Parent:
    def init left: int, right: int => this.amount = left + right
    def reading offset: int, scale: int => self.amount * scale + offset
class Child: Parent
    def init left: int, right => super(left, right)
    def reading offset, scale: int => self.amount * scale + offset
child = Child(2, 3)
observed = child.reading(1, 2)
assert(child.id, "child")
assert(child.amount, 5)
assert(observed, 11)
child.amount = 8
assert(observed, 17)
assert(Parent.length, 0)
assert(Child.length, 1)

---

# Nucleoid preserves bare typed multiple parameter arrow bindings and scopes

left = 10
right = 20
def pair left: int, right: int => [left, right]
def invoke operation: Callback, amount: int => operation(amount)
def increment(amount: int) => amount + 1
def operation(amount) => amount + 100
def outer(initial: int):
    def combine left: int, right, scale: int => (left + right) * scale
    return combine(initial, 2, 3)
assert(pair(3, 4), [3, 4])
assert(pair(3), [3, null])
assert(pair(), [null, null])
assert(pair(3, 4, 5), [3, 4])
assert(pair("three", true), ["three", true])
assert(invoke(increment, 4), 5)
assert(invoke(null, 3), null)
try:
    invoke(7, 3)
catch error:
    assert(error, TypeError("operation is not a function"))
assert(outer(4), 18)
assert(left, 10)
assert(right, 20)

---

# Nucleoid rolls back failed bare typed multiple parameter arrow redefinitions

source = 3
def reading left: int, right: int => source + left + right
total = reading(2, 4)
assert(total, 9)
try:
    def reading left: int, right: int => missing + left + right
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(total, 9)
source = 4
assert(total, 10)
assert(reading(8, 9), 21)
reading(8, 9)

# return: 21

---

# Nucleoid defines bare typed multiple parameter indented blocks

def add left: int, right: int:
    total = left + right
    return total
def scaled left: number, right, factor: number =>
    (left + right) * factor
def largest left, right: number:
    if left > right:
        return left
    return right
assert(add(2, 3), 5)
assert(scaled(2, 3, 4), 20)
assert(largest(7, 3), 7)
assert(largest(2, 9), 9)

---

# Nucleoid keeps bare typed multiple parameter blocks reactive

bias = 2
def scaled left: int, right =>
    left * right + bias
live = scaled
snapshot = scaled.value
result = live(3, 4)
frozen = snapshot(3, 4)
assert(result, 14)
assert(frozen, 14)
def scaled left: int, right: int =>
    left + right + bias
assert(result, 9)
assert(frozen, 14)
bias = 4
assert(result, 11)
assert(frozen, 16)

---

# Nucleoid preserves bare typed multiple parameter block methods and initializers

class Parent:
    def init left: int, right: int:
        this.amount = left + right
    def reading offset: int, scale =>
        self.amount * scale + offset
class Child: Parent
    def init left: int, right:
        super(left, right)
    def reading offset, scale: int =>
        self.amount * scale + offset
    def increase delta: int, extra:
        this.amount += delta + extra
        return this.amount
child = Child(2, 3)
observed = child.reading(1, 2)
assert(child.id, "child")
assert(child.amount, 5)
assert(observed, 11)
assert(child.increase(2, 1), 8)
assert(child.amount, 8)
assert(observed, 17)
assert(Parent.length, 0)
assert(Child.length, 1)

---

# Nucleoid preserves bare typed multiple parameter block bindings and returns

left = 10
right = 20
def pair left: int, right: int =>
    [left, right]
def add left: int, right:
    left += right
    return left
def noReturn left: int, right: int:
    left + right
def bareReturn left, right: int:
    return
def outer(initial: int):
    def combine left: int, right, scale: int =>
        (left + right) * scale
    return combine(initial, 2, 3)
assert(pair(3, 4), [3, 4])
assert(pair(3), [3, null])
assert(pair(), [null, null])
assert(pair(3, 4, 5), [3, 4])
assert(pair("three", true), ["three", true])
assert(add(3, 4), 7)
assert(left, 10)
assert(right, 20)
assert(noReturn(2, 3), null)
assert(bareReturn(2, 3), null)
assert(outer(4), 18)

---

# Nucleoid rolls back failed bare typed multiple parameter block redefinitions

source = 3
def reading left: int, right: int =>
    source + left + right
total = reading(2, 4)
assert(total, 9)
try:
    def reading left: int, right =>
        missing + left + right
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(total, 9)
source = 4
assert(total, 10)
assert(reading(8, 9), 21)
reading(8, 9)

# return: 21

---

# Nucleoid defines implicit iterator indented loops

class Entry
first = Entry()
first.amount = 2
second = Entry()
second.amount = 5
values = [first, 99, second, null, { amount: 9 }]
seen = []
total = 0
for values:
    seen.push(it.amount)
    total += it.amount
assert(seen, [2, 5])
assert(total, 7)
empty = []
for empty:
    total += 100
assert(total, 7)

---

# Nucleoid preserves implicit iterator loop scopes

class Entry
first = Entry()
first.amount = 2
second = Entry()
second.amount = 5
outer = Entry()
outer.amount = 100
groups = [outer]
values = [first, second]
it = 50
seen = []
for groups:
    seen.push(it.amount)
    for values:
        seen.push(it.amount)
    seen.push(it.amount)
assert(seen, [100, 2, 5, 100])
assert(it, 50)
it = values
nested = []
for it:
    nested.push(it.amount)
assert(nested, [2, 5])
assert(it.length, 2)

---

# Nucleoid preserves implicit iterator loop population snapshots

class Reading
first = Reading()
first.amount = 2
second = Reading()
second.amount = 5
scale = 2
for Reading:
    it.score = it.amount * scale
assert(first.score, 4)
assert(second.score, 10)
scale = 3
assert(first.score, 4)
assert(second.score, 10)
third = Reading()
third.amount = 7
assert(third.score, null)
for Reading:
    it.score = it.amount * scale
assert(first.score, 6)
assert(second.score, 15)
assert(third.score, 21)

---

# Nucleoid preserves implicit iterator loop return flow

class Entry
first = Entry()
first.amount = 2
second = Entry()
second.amount = 5
values = [first, second]
def firstAmount(records):
    for records:
        return it.amount
    return -1
def noResult(records):
    for records:
        pass
class Reader:
    def first(records):
        for records:
            return it.amount
        return -1
reader = Reader()
assert(firstAmount(values), 2)
assert(firstAmount([]), -1)
assert(noResult(values), null)
assert(reader.first(values), 2)
assert(reader.first([]), -1)
reader.first(values)

# return: 2

---

# Nucleoid rolls back failed implicit iterator loops

class Entry
first = Entry()
first.amount = 2
second = Entry()
second.amount = 5
values = [first, second]
saved = 7
try:
    for values:
        saved += it.amount
        it.amount += 1
        if it.amount > 5:
            throw "stop"
catch error:
    assert(error, "stop")
assert(first.amount, 2)
assert(second.amount, 5)
assert(saved, 7)
broken = null
try:
    for broken:
        first.amount = 99
catch error:
    assert(error, TypeError("broken is not iterable"))
assert(first.amount, 2)
for values:
    it.score = it.amount * 2
assert(first.score, 4)
assert(second.score, 10)
first.score + second.score

# return: 14

---

# Nucleoid defines indented named arrow expressions

def literal() =>
    4
def twice amount =>
    amount * 2
def typed amount: int =>
    amount + 1
def add left right =>
    left + right
def scaled left: int, right, factor: number =>
    (left + right) * factor
assert(literal(), 4)
assert(twice(3), 6)
assert(typed(4), 5)
assert(add(2, 3), 5)
assert(scaled(2, 3, 4), 20)
scaled(2, 3, 4)

# return: 20

---

# Nucleoid keeps indented named arrows reactive

offset = 2
def adjusted number =>
    number + offset
live = adjusted
snapshot = adjusted.value
result = live(3)
frozen = snapshot(3)
assert(result, 5)
assert(frozen, 5)
def adjusted number =>
    number * offset
assert(result, 6)
assert(frozen, 5)
offset = 4
assert(result, 12)
assert(frozen, 7)

---

# Nucleoid preserves indented named arrow methods and initializers

class Parent:
    def init value =>
        this.amount = value
    def reading offset: int =>
        self.amount + offset
class Child: Parent
    def init value =>
        super(value)
    def reading offset: int =>
        this.amount * 2 + offset
    def change delta extra =>
        this.amount += delta + extra
child = Child(3)
observed = child.reading(1)
assert(child.id, "child")
assert(child.amount, 3)
assert(observed, 7)
assert(child.change(2, 1), 6)
assert(child.amount, 6)
assert(observed, 13)
assert(Parent.length, 0)
assert(Child.length, 1)

---

# Nucleoid preserves indented named arrow bindings and returned values

left = 10
right = 20
def pair left: int, right =>
    [left, right]
def point x y =>
    {x, y}
def text left right =>
    `${left},${right}`
def factory ignored =>
    value => value + 1
def assign left right =>
    left += right
def outer initial:
    def local amount =>
        amount * 2
    return local(initial)
assert(pair(3, 4), [3, 4])
assert(pair(3), [3, null])
assert(pair(), [null, null])
assert(pair("three", true), ["three", true])
p = point(2, 3)
assert(p.x, 2)
assert(p.y, 3)
assert(text(2, 3), "2,3")
callback = factory(0)
assert(callback(5), 6)
assert(assign(3, 4), 7)
assert(left, 10)
assert(right, 20)
assert(outer(4), 8)

---

# Nucleoid rolls back failed indented named arrow redefinitions

source = 3
def reading left: int, right: int =>
    source + left + right
total = reading(2, 4)
assert(total, 9)
try:
    def reading left: int, right =>
        missing + left + right
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(total, 9)
source = 4
assert(total, 10)
assert(reading(8, 9), 21)
reading(8, 9)

# return: 21

---

# Nucleoid keeps cubic smoothstep reactive to all three inputs

lower = 0
upper = 8
value = 2
unit = Math.saturate((value - lower) / (upper - lower))
eased = Math.smoothstep(lower, upper, value)
assert(unit, 0.25)
assert(eased, 0.15625)
value = 4
assert(unit, 0.5)
assert(eased, 0.5)
upper = 16
assert(unit, 0.25)
assert(eased, 0.15625)
lower = -8
assert(unit, 0.5)
assert(eased, 0.5)
value = -16
assert(unit, 0)
assert(eased, 0)
value = 24
assert(unit, 1)
assert(eased, 1)
upper = -32
assert(unit, 0)
assert(eased, 0)
value = -20
assert(unit, 0.5)
assert(eased, 0.5)
eased
# return: 0.5

---

# Nucleoid preserves numeric coercion while smoothing values

lower = " 0 "
upper = "8"
value = "2"
unit = Math.saturate((value - lower) / (upper - lower))
eased = Math.smoothstep(lower, upper, value)
assert(unit, 0.25)
assert(eased, 0.15625)
lower = false
upper = true
value = true
assert(unit, 1)
assert(eased, 1)
lower = []
upper = ["8"]
value = ["6"]
assert(unit, 0.75)
assert(eased, 0.84375)
lower = Date(0)
upper = Date(8)
value = Date(4)
assert(unit, 0.5)
assert(eased, 0.5)
value = [1, 2]
assert(Number.isNaN(unit), true)
assert(Number.isNaN(eased), true)

---

# Nucleoid preserves special values and omitted smoothstep inputs

lower = 2
upper = 2
value = 2
unit = Math.saturate((value - lower) / (upper - lower))
eased = Math.smoothstep(lower, upper, value)
assert(Number.isNaN(unit), true)
assert(Number.isNaN(eased), true)
value = 1
assert(unit, 0)
assert(eased, 0)
value = 3
assert(unit, 1)
assert(eased, 1)
upper = 0
assert(unit, 0)
assert(eased, 0)
value = 1
assert(unit, 0.5)
assert(eased, 0.5)
upper = Number.POSITIVE_INFINITY
assert(unit, 0)
assert(eased, 0)
value = Number.POSITIVE_INFINITY
assert(Number.isNaN(unit), true)
assert(Number.isNaN(eased), true)
nan = Number.NaN
value = nan
assert(Number.isNaN(unit), true)
assert(Number.isNaN(eased), true)
lower = Number.NEGATIVE_INFINITY
upper = Number.POSITIVE_INFINITY
value = 0
assert(Number.isNaN(unit), true)
assert(Number.isNaN(eased), true)
empty = Math.smoothstep()
one = Math.smoothstep(0)
two = Math.smoothstep(0, 1)
assert(Number.isNaN(empty), true)
assert(Number.isNaN(one), true)
assert(Number.isNaN(two), true)
half = Math.saturate(0.5)
withExtra = Math.smoothstep(0, 1, 0.5, 99)
ignored = Number.NaN
withIgnored = Math.smoothstep(0, 1, 0.5, ignored)
assert(half, 0.5)
assert(withExtra, 0.5)
assert(withIgnored, 0.5)

---

# Nucleoid applies cubic smoothstep to live class rules

class Curve
lower = 0
upper = 8
$Curve.unit = Math.saturate(($Curve.sample - lower) / (upper - lower))
$Curve.eased = Math.smoothstep(lower, upper, $Curve.sample)
first = Curve()
second = Curve()
first.sample = 2
second.sample = 6
assert(first.unit, 0.25)
assert(first.eased, 0.15625)
assert(second.unit, 0.75)
assert(second.eased, 0.84375)
upper = 16
assert(first.unit, 0.125)
assert(first.eased, 0.04296875)
assert(second.unit, 0.375)
assert(second.eased, 0.31640625)
first.sample = 8
assert(first.unit, 0.5)
assert(first.eased, 0.5)
third = Curve()
assert(third.unit, null)
assert(third.eased, null)
third.sample = 4
assert(third.unit, 0.25)
assert(third.eased, 0.15625)
delete first.sample
assert(first.unit, null)
assert(first.eased, null)
first.sample = 16
assert(first.unit, 1)
assert(first.eased, 1)
assert(Curve.length, 3)

---

# Nucleoid preserves deferred reads and rollback for cubic smoothstep

class Sensor
sensor = Sensor()
lower = 0
upper = 8
value = sensor.reading
unit = Math.saturate((value - lower) / (upper - lower))
eased = Math.smoothstep(lower, upper, value)
assert(value, null)
assert(unit, null)
assert(eased, null)
sensor.reading = 2
assert(unit, 0.25)
assert(eased, 0.15625)
delete sensor.reading
assert(value, null)
assert(eased, null)
sensor.reading = 4
assert(unit, 0.5)
assert(eased, 0.5)
try:
    sensor.reading = 6
    throw "ROLLBACK"
catch error:
    assert(error, "ROLLBACK")
assert(sensor.reading, 4)
assert(unit, 0.5)
assert(eased, 0.5)
try:
    lower = missing
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(lower, 0)
sensor.reading = 2
assert(unit, 0.25)
assert(eased, 0.15625)
eased
# return: 0.15625

---

# Nucleoid associates nested indented else suites with their conditions

class Gauge
gauge = Gauge()
gauge.outer = false
gauge.inner = false
first = 3
second = 5
third = 7
if gauge.outer:
    gauge.result = first
else
    if gauge.inner:
        gauge.result = second
    else
        gauge.result = third
assert(gauge.result, 7)
gauge.inner = true
assert(gauge.result, 5)
gauge.outer = true
assert(gauge.result, 3)
first = 11
assert(gauge.result, 11)
gauge.outer = false
assert(gauge.result, 5)
second = 13
assert(gauge.result, 13)
gauge.inner = false
assert(gauge.result, 7)
third = 17
assert(gauge.result, 17)
gauge.result
# return: 17

---

# Nucleoid keeps indented else class rules independent per instance

class Tier
first = Tier()
first.score = 1
second = Tier()
second.score = 9
threshold = 5
low = "LOW"
high = "HIGH"
if $Tier.score >= threshold:
    $Tier.label = high
else
    $Tier.label = low
assert(first.label, "LOW")
assert(second.label, "HIGH")
low = "L"
assert(first.label, "L")
assert(second.label, "HIGH")
first.score = 8
assert(first.label, "HIGH")
assert(second.label, "HIGH")
threshold = 10
assert(first.label, "L")
assert(second.label, "L")
second.score = 12
assert(first.label, "L")
assert(second.label, "HIGH")

---

# Nucleoid preserves returns from indented else function branches

def choose(flag, value):
    if flag:
        return value
    else
        return value + 1
enabled = false
amount = 2
chosen = choose(enabled, amount)
assert(chosen, 3)
enabled = true
assert(chosen, 2)
amount = 5
assert(chosen, 5)
enabled = false
assert(chosen, 6)
chosen
# return: 6

---

# Nucleoid rolls back throws from indented else suites

marker = 7
try:
    if false:
        marker = 8
    else
        marker = 9
        throw "ABORT"
catch error:
    assert(error, "ABORT")
assert(marker, 7)
gate = false
if gate:
    result = 1
else
    result = 2
assert(result, 2)
gate = true
assert(result, 1)

---

# Nucleoid preserves legacy suites beside indented else bodies

result = 0
if true: result = 1
else: result = 2
assert(result, 1)
if false:
    result = 3
else
    result = 4
assert(result, 4)
if true: { result = 5 }
else: { result = 6 }
assert(result, 5)
text = "else:"
assert(text, "else:")
flag = false
value = 2
if flag:
    answer = value
else # else: in a comment is not a delimiter
    answer = value + 10
assert(answer, 12)
flag = true
assert(answer, 2)
value = 6
assert(answer, 6)

---

# Nucleoid keeps quintic smootherstep reactive to all three inputs

lower = 0
upper = 8
value = 2
unit = Math.saturate((value - lower) / (upper - lower))
eased = Math.smootherstep(lower, upper, value)
assert(unit, 0.25)
assert(eased, 0.103515625)
value = 4
assert(unit, 0.5)
assert(eased, 0.5)
upper = 16
assert(unit, 0.25)
assert(eased, 0.103515625)
lower = -8
assert(unit, 0.5)
assert(eased, 0.5)
value = -16
assert(unit, 0)
assert(eased, 0)
value = 24
assert(unit, 1)
assert(eased, 1)
upper = -32
assert(unit, 0)
assert(eased, 0)
value = -20
assert(unit, 0.5)
assert(eased, 0.5)
eased
# return: 0.5

---

# Nucleoid preserves numeric coercion while applying quintic smootherstep

lower = " 0 "
upper = "8"
value = "2"
unit = Math.saturate((value - lower) / (upper - lower))
eased = Math.smootherstep(lower, upper, value)
assert(unit, 0.25)
assert(eased, 0.103515625)
lower = false
upper = true
value = true
assert(unit, 1)
assert(eased, 1)
lower = []
upper = ["8"]
value = ["6"]
assert(unit, 0.75)
assert(eased, 0.896484375)
lower = Date(0)
upper = Date(8)
value = Date(4)
assert(unit, 0.5)
assert(eased, 0.5)
value = [1, 2]
assert(Number.isNaN(unit), true)
assert(Number.isNaN(eased), true)

---

# Nucleoid preserves special values and omitted smootherstep inputs

lower = 2
upper = 2
value = 2
unit = Math.saturate((value - lower) / (upper - lower))
eased = Math.smootherstep(lower, upper, value)
assert(Number.isNaN(unit), true)
assert(Number.isNaN(eased), true)
value = 1
assert(unit, 0)
assert(eased, 0)
value = 3
assert(unit, 1)
assert(eased, 1)
upper = 0
assert(unit, 0)
assert(eased, 0)
value = 1
assert(unit, 0.5)
assert(eased, 0.5)
upper = Number.POSITIVE_INFINITY
assert(unit, 0)
assert(eased, 0)
value = Number.POSITIVE_INFINITY
assert(Number.isNaN(unit), true)
assert(Number.isNaN(eased), true)
nan = Number.NaN
value = nan
assert(Number.isNaN(unit), true)
assert(Number.isNaN(eased), true)
lower = Number.NEGATIVE_INFINITY
upper = Number.POSITIVE_INFINITY
value = 0
assert(Number.isNaN(unit), true)
assert(Number.isNaN(eased), true)
empty = Math.smootherstep()
one = Math.smootherstep(0)
two = Math.smootherstep(0, 1)
assert(Number.isNaN(empty), true)
assert(Number.isNaN(one), true)
assert(Number.isNaN(two), true)
half = Math.saturate(0.5)
withExtra = Math.smootherstep(0, 1, 0.5, 99)
ignored = Number.NaN
withIgnored = Math.smootherstep(0, 1, 0.5, ignored)
assert(half, 0.5)
assert(withExtra, 0.5)
assert(withIgnored, 0.5)

---

# Nucleoid applies quintic smootherstep to live class rules

class Curve
lower = 0
upper = 8
$Curve.unit = Math.saturate(($Curve.sample - lower) / (upper - lower))
$Curve.eased = Math.smootherstep(lower, upper, $Curve.sample)
first = Curve()
second = Curve()
first.sample = 2
second.sample = 6
assert(first.unit, 0.25)
assert(first.eased, 0.103515625)
assert(second.unit, 0.75)
assert(second.eased, 0.896484375)
upper = 16
assert(first.unit, 0.125)
assert(first.eased, 0.01605224609375)
assert(second.unit, 0.375)
assert(second.eased, 0.27520751953125)
first.sample = 8
assert(first.unit, 0.5)
assert(first.eased, 0.5)
third = Curve()
assert(third.unit, null)
assert(third.eased, null)
third.sample = 4
assert(third.unit, 0.25)
assert(third.eased, 0.103515625)
delete first.sample
assert(first.unit, null)
assert(first.eased, null)
first.sample = 16
assert(first.unit, 1)
assert(first.eased, 1)
assert(Curve.length, 3)

---

# Nucleoid preserves deferred reads and rollback for quintic smootherstep

class Sensor
sensor = Sensor()
lower = 0
upper = 8
value = sensor.reading
unit = Math.saturate((value - lower) / (upper - lower))
eased = Math.smootherstep(lower, upper, value)
assert(value, null)
assert(unit, null)
assert(eased, null)
sensor.reading = 2
assert(unit, 0.25)
assert(eased, 0.103515625)
delete sensor.reading
assert(value, null)
assert(eased, null)
sensor.reading = 4
assert(unit, 0.5)
assert(eased, 0.5)
try:
    sensor.reading = 6
    throw "ROLLBACK"
catch error:
    assert(error, "ROLLBACK")
assert(sensor.reading, 4)
assert(unit, 0.5)
assert(eased, 0.5)
try:
    lower = missing
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(lower, 0)
sensor.reading = 2
assert(unit, 0.25)
assert(eased, 0.103515625)
eased
# return: 0.103515625

---

# Nucleoid keeps floor fractional parts reactive

value = 2.75
whole = Math.floor(value)
fraction = Math.fract(value)
assert(whole, 2)
assert(fraction, 0.75)
value = -1.25
assert(whole, -2)
assert(fraction, 0.75)
value = 4.125
assert(whole, 4)
assert(fraction, 0.125)
value = -2
assert(whole, -2)
assert(fraction, 0)
value = 0
assert(whole, 0)
assert(fraction, 0)
value = -0
assert(whole, 0)
assert(fraction, 0)
value = 1.5
assert(whole, 1)
assert(fraction, 0.5)
fraction
# return: 0.5

---

# Nucleoid preserves numeric coercion while computing floor fractional parts

value = " 2.75 "
whole = Math.floor(value)
fraction = Math.fract(value)
assert(whole, 2)
assert(fraction, 0.75)
value = true
assert(whole, 1)
assert(fraction, 0)
value = false
assert(whole, 0)
assert(fraction, 0)
value = []
assert(whole, 0)
assert(fraction, 0)
value = ["3.125"]
assert(whole, 3)
assert(fraction, 0.125)
value = Date(1234)
assert(whole, 1234)
assert(fraction, 0)
value = [1, 2]
assert(Number.isNaN(whole), true)
assert(Number.isNaN(fraction), true)
value = "not-a-number"
assert(Number.isNaN(whole), true)
assert(Number.isNaN(fraction), true)

---

# Nucleoid preserves nonfinite fractional parts omissions and rounded endpoints

value = -0.00000000000000000001
whole = Math.floor(value)
fraction = Math.fract(value)
assert(whole, -1)
assert(fraction, 1)
value = Number.POSITIVE_INFINITY
assert(whole, Number.POSITIVE_INFINITY)
assert(Number.isNaN(fraction), true)
value = Number.NEGATIVE_INFINITY
assert(whole, Number.NEGATIVE_INFINITY)
assert(Number.isNaN(fraction), true)
value = Number.NaN
assert(Number.isNaN(whole), true)
assert(Number.isNaN(fraction), true)
value = Number.MAX_VALUE
assert(whole, Number.MAX_VALUE)
assert(fraction, 0)
value = -Number.MAX_VALUE
assert(whole, -Number.MAX_VALUE)
assert(fraction, 0)
empty = Math.fract()
assert(Number.isNaN(empty), true)
half = 0.5
withExtra = Math.fract(half, 99)
ignored = Number.NaN
withIgnored = Math.fract(half, ignored)
assert(half, 0.5)
assert(withExtra, 0.5)
assert(withIgnored, 0.5)
ignored = "not-a-number"
assert(withIgnored, 0.5)

---

# Nucleoid applies floor fractional parts to live class rules

class Reading
$Reading.whole = Math.floor($Reading.sample)
$Reading.fraction = Math.fract($Reading.sample)
first = Reading()
second = Reading()
first.sample = 2.25
second.sample = -1.75
assert(first.whole, 2)
assert(first.fraction, 0.25)
assert(second.whole, -2)
assert(second.fraction, 0.25)
first.sample = -1.25
assert(first.whole, -2)
assert(first.fraction, 0.75)
assert(second.fraction, 0.25)
third = Reading()
assert(third.whole, null)
assert(third.fraction, null)
third.sample = 8.125
assert(third.whole, 8)
assert(third.fraction, 0.125)
delete first.sample
assert(first.whole, null)
assert(first.fraction, null)
first.sample = -3.5
assert(first.whole, -4)
assert(first.fraction, 0.5)
assert(Reading.length, 3)

---

# Nucleoid preserves deferred fractional parts and rollback

class Sensor
sensor = Sensor()
value = sensor.reading
whole = Math.floor(value)
fraction = Math.fract(value)
withDeferredExtra = Math.fract(value, sensor.optional)
assert(value, null)
assert(whole, null)
assert(fraction, null)
assert(withDeferredExtra, null)
sensor.reading = 2.25
assert(whole, 2)
assert(fraction, 0.25)
assert(withDeferredExtra, null)
delete sensor.reading
assert(value, null)
assert(fraction, null)
sensor.reading = 3.75
assert(whole, 3)
assert(fraction, 0.75)
sensor.optional = 99
assert(withDeferredExtra, 0.75)
try:
    sensor.reading = 5.5
    throw "ROLLBACK"
catch error:
    assert(error, "ROLLBACK")
assert(sensor.reading, 3.75)
assert(whole, 3)
assert(fraction, 0.75)
assert(withDeferredExtra, 0.75)
try:
    sensor.reading = missing
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(sensor.reading, 3.75)
delete sensor.optional
assert(withDeferredExtra, null)
assert(fraction, 0.75)
sensor.optional = [1, 2]
assert(withDeferredExtra, 0.75)
sensor.reading = 2.25
assert(whole, 2)
assert(fraction, 0.25)
assert(withDeferredExtra, 0.25)
fraction
# return: 0.25

---

# Nucleoid defines comma-free bare named arrows

def add left right => left + right
def scaled left right factor => (left + right) * factor
def pair left right => [left, right]
def coordinates x y => {x, y}
def indented left right =>
    left + right
assert(add(2, 3), 5)
assert(scaled(2, 3, 4), 20)
assert(pair("left", "right"), ["left", "right"])
point = coordinates(2, 3)
assert(point.x, 2)
assert(point.y, 3)
assert(indented(2, 3), 5)

---

# Nucleoid keeps comma-free bare named arrows reactive

bias = 1
def total left right => left + right + bias
tracked = total(2, 3)
assert(tracked, 6)
bias = 4
assert(tracked, 9)
def projected first second third => first * second + third
left = 2
right = 3
offset = 4
result = projected(left, right, offset)
assert(result, 10)
left = 5
assert(result, 19)
right = 2
assert(result, 14)

---

# Nucleoid preserves comma-free methods and initializers

class Calculator:
    def init base scale => this.base = base * scale
    def combine left right => this.base + left + right
    def product left right scale => self.base + left * right * scale
calculator = Calculator(1, 2)
result = calculator.combine(3, 4)
assert(result, 9)
calculator.base = 5
assert(result, 12)
assert(calculator.product(2, 3, 4), 29)

---

# Nucleoid preserves nested comma-free bare named arrows

def outer(initial):
    def combine left right scale => (left + right) * scale
    return combine(initial, 2, 3)
assert(outer(4), 18)
def factory(ignored) => (left, right) => left + right
custom = factory(10)
assert(custom(2, 3), 5)

---

# Nucleoid rolls back comma-free named arrow redefinitions

source = 3
def reading left right => source + left + right
result = reading(2, 4)
assert(result, 9)
try:
    def reading left right => missing + left + right
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(result, 9)
source = 5
assert(result, 11)
def prefer current fallback => current ||= fallback
def combined first second scale => (first + second) * scale
assert(prefer(0, 9), 9)
assert(prefer(3, 9), 3)
assert(combined(2, 5, 3), 21)
combined(2, 5, 3)
# return: 21
---

# Nucleoid keeps inverse interpolation reactive

start = 2
end = 10
sample = 4
weight = Math.inverseLerp(start, end, sample)
reconstructed = Math.lerp(start, end, weight)
assert(weight, 0.25)
assert(reconstructed, 4)
sample = 6
assert(weight, 0.5)
assert(reconstructed, 6)
sample = 14
assert(weight, 1.5)
assert(reconstructed, 14)
sample = -2
assert(weight, -0.5)
assert(reconstructed, -2)
start = 10
end = 2
sample = 8
assert(weight, 0.25)
assert(reconstructed, 8)
sample = 12
assert(weight, -0.25)
assert(reconstructed, 12)
sample = 6
assert(weight, 0.5)
assert(reconstructed, 6)
weight
# return: 0.5

---

# Nucleoid preserves intrinsic conversion during inverse interpolation

start = " 2 "
end = "10"
sample = "4"
weight = Math.inverseLerp(start, end, sample)
assert(weight, 0.25)
assert(start, " 2 ")
start = false
end = true
sample = false
assert(weight, 0)
sample = true
assert(weight, 1)
sample = [0.5]
assert(weight, 0.5)
sample = []
assert(weight, 0)
sample = "not-a-number"
assert(Number.isNaN(weight), true)
sample = [1, 2]
assert(Number.isNaN(weight), true)
start = Date(0)
end = Date(8)
sample = Date(2)
assert(weight, 0.25)
sample = Date(6)
assert(weight, 0.75)
start = []
end = ["8"]
sample = ["2"]
assert(weight, 0.25)
sample = false
assert(weight, 0)

---

# Nucleoid preserves inverse interpolation endpoints nonfinite inputs and omissions

start = 0
end = 8
sample = 2
weight = Math.inverseLerp(start, end, sample)
assert(weight, 0.25)
sample = start
assert(weight, 0)
sample = end
assert(weight, 1)
sample = 16
assert(weight, 2)
sample = -8
assert(weight, -1)
start = 2
end = 2
sample = 2
assert(Number.isNaN(weight), true)
sample = 3
assert(weight, Number.POSITIVE_INFINITY)
sample = 1
assert(weight, Number.NEGATIVE_INFINITY)
start = 0
end = Number.POSITIVE_INFINITY
sample = 8
assert(weight, 0)
sample = Number.POSITIVE_INFINITY
assert(Number.isNaN(weight), true)
end = 8
sample = Number.NEGATIVE_INFINITY
assert(weight, Number.NEGATIVE_INFINITY)
start = Number.NEGATIVE_INFINITY
end = Number.POSITIVE_INFINITY
sample = 0
assert(Number.isNaN(weight), true)
start = -Number.MAX_VALUE
end = Number.MAX_VALUE
sample = 0
assert(weight, 0)
start = 0
end = 8
sample = Number.NaN
assert(Number.isNaN(weight), true)
empty = Math.inverseLerp()
assert(Number.isNaN(empty), true)
one = Math.inverseLerp(0)
assert(Number.isNaN(one), true)
two = Math.inverseLerp(0, 8)
assert(Number.isNaN(two), true)
literal = Math.inverseLerp(false, true, null)
assert(literal, 0)
withExtra = Math.inverseLerp(0, 8, 2, 99)
assert(withExtra, 0.25)
ignored = Number.NaN
withIgnored = Math.inverseLerp(0, 8, 2, ignored)
assert(withIgnored, 0.25)
ignored = "not-a-number"
assert(withIgnored, 0.25)
ignored = null
assert(withIgnored, null)
ignored = []
assert(withIgnored, 0.25)

---

# Nucleoid applies inverse interpolation to live class rules

class Gauge
$Gauge.weight = Math.inverseLerp($Gauge.start, $Gauge.end, $Gauge.sample)
first = Gauge()
second = Gauge()
first.start = 0
first.end = 8
first.sample = 2
second.start = 10
second.end = 2
second.sample = 8
assert(first.weight, 0.25)
assert(second.weight, 0.25)
first.sample = 12
assert(first.weight, 1.5)
assert(second.weight, 0.25)
first.end = 16
assert(first.weight, 0.75)
third = Gauge()
assert(third.weight, null)
third.start = 0
third.end = 4
third.sample = 1
assert(third.weight, 0.25)
delete first.sample
assert(first.weight, null)
first.sample = -4
assert(first.weight, -0.25)
assert(Gauge.length, 3)

---

# Nucleoid preserves deferred inverse interpolation and rollback

class Sensor
sensor = Sensor()
start = sensor.lower
end = sensor.upper
sample = sensor.reading
weight = Math.inverseLerp(start, end, sample)
withDeferredExtra = Math.inverseLerp(start, end, sample, sensor.optional)
assert(weight, null)
assert(withDeferredExtra, null)
sensor.lower = 0
sensor.upper = 8
sensor.reading = 2
assert(weight, 0.25)
assert(withDeferredExtra, null)
sensor.optional = 99
assert(withDeferredExtra, 0.25)
delete sensor.reading
assert(weight, null)
assert(withDeferredExtra, null)
sensor.reading = 6
assert(weight, 0.75)
assert(withDeferredExtra, 0.75)
try:
    sensor.reading = 4
    throw "ROLLBACK"
catch error:
    assert(error, "ROLLBACK")
assert(sensor.reading, 6)
assert(weight, 0.75)
assert(withDeferredExtra, 0.75)
try:
    sensor.reading = missing
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(sensor.reading, 6)
delete sensor.optional
assert(withDeferredExtra, null)
assert(weight, 0.75)
sensor.optional = [1, 2]
assert(withDeferredExtra, 0.75)
sensor.reading = 2
assert(weight, 0.25)
assert(withDeferredExtra, 0.25)
weight
# return: 0.25

---

# Nucleoid keeps range remapping reactive

start = 2
end = 10
targetStart = 10
targetEnd = 30
sample = 4
mapped = Math.remap(start, end, targetStart, targetEnd, sample)
unit = Math.inverseLerp(start, end, sample)
assert(mapped, 15)
assert(unit, 0.25)
sample = 6
assert(mapped, 20)
assert(unit, 0.5)
sample = 14
assert(mapped, 40)
assert(unit, 1.5)
sample = -2
assert(mapped, 0)
assert(unit, -0.5)
start = 10
end = 2
sample = 8
assert(mapped, 15)
assert(unit, 0.25)
targetStart = 30
targetEnd = 10
assert(mapped, 25)
assert(unit, 0.25)
sample = 6
assert(mapped, 20)
assert(unit, 0.5)
mapped
# return: 20

---

# Nucleoid preserves intrinsic conversion during range remapping

start = " 2 "
end = "10"
targetStart = "10"
targetEnd = "30"
sample = "4"
mapped = Math.remap(start, end, targetStart, targetEnd, sample)
assert(mapped, 15)
assert(start, " 2 ")
start = false
end = true
targetStart = false
targetEnd = true
sample = false
assert(mapped, 0)
sample = true
assert(mapped, 1)
sample = [0.5]
assert(mapped, 0.5)
sample = []
assert(mapped, 0)
sample = "not-a-number"
assert(Number.isNaN(mapped), true)
sample = [1, 2]
assert(Number.isNaN(mapped), true)
start = Date(0)
end = Date(8)
targetStart = Date(10)
targetEnd = Date(30)
sample = Date(2)
assert(mapped, 15)
sample = Date(6)
assert(mapped, 25)
start = []
end = ["8"]
targetStart = []
targetEnd = ["8"]
sample = ["2"]
assert(mapped, 2)
sample = false
assert(mapped, 0)

---

# Nucleoid preserves remapping endpoints nonfinite inputs and omissions

start = 0
end = 8
targetStart = 10
targetEnd = 30
sample = 2
mapped = Math.remap(start, end, targetStart, targetEnd, sample)
assert(mapped, 15)
sample = start
assert(mapped, 10)
sample = end
assert(mapped, 30)
sample = 16
assert(mapped, 50)
sample = -8
assert(mapped, -10)
start = 2
end = 2
sample = 2
assert(Number.isNaN(mapped), true)
sample = 3
assert(mapped, Number.POSITIVE_INFINITY)
sample = 1
assert(mapped, Number.NEGATIVE_INFINITY)
start = 0
end = Number.POSITIVE_INFINITY
sample = 8
assert(mapped, 10)
sample = Number.POSITIVE_INFINITY
assert(Number.isNaN(mapped), true)
end = 8
sample = Number.NEGATIVE_INFINITY
assert(mapped, Number.NEGATIVE_INFINITY)
start = Number.NEGATIVE_INFINITY
end = Number.POSITIVE_INFINITY
sample = 0
assert(Number.isNaN(mapped), true)
start = -Number.MAX_VALUE
end = Number.MAX_VALUE
sample = 0
assert(mapped, 10)
start = 0
end = 8
sample = Number.NaN
assert(Number.isNaN(mapped), true)
targetStart = 4
targetEnd = 4
sample = 2
assert(mapped, 4)
sample = Number.POSITIVE_INFINITY
assert(Number.isNaN(mapped), true)
targetStart = -Number.MAX_VALUE
targetEnd = Number.MAX_VALUE
sample = 2
assert(mapped, Number.POSITIVE_INFINITY)
targetStart = 0
targetEnd = 1
assert(mapped, 0.25)
empty = Math.remap()
assert(Number.isNaN(empty), true)
one = Math.remap(0)
assert(Number.isNaN(one), true)
two = Math.remap(0, 8)
assert(Number.isNaN(two), true)
three = Math.remap(0, 8, 0)
assert(Number.isNaN(three), true)
four = Math.remap(0, 8, 0, 1)
assert(Number.isNaN(four), true)
literal = Math.remap(false, true, 0, 1, null)
assert(literal, 0)
withExtra = Math.remap(0, 8, 10, 30, 2, 99)
assert(withExtra, 15)
ignored = Number.NaN
withIgnored = Math.remap(0, 8, 10, 30, 2, ignored)
assert(withIgnored, 15)
ignored = "not-a-number"
assert(withIgnored, 15)
ignored = null
assert(withIgnored, null)
ignored = []
assert(withIgnored, 15)

---

# Nucleoid applies range remapping to live class rules

class Gauge
$Gauge.mapped = Math.remap($Gauge.start, $Gauge.end, $Gauge.targetStart, $Gauge.targetEnd, $Gauge.sample)
first = Gauge()
second = Gauge()
first.start = 0
first.end = 8
first.targetStart = 10
first.targetEnd = 30
first.sample = 2
second.start = 10
second.end = 2
second.targetStart = 30
second.targetEnd = 10
second.sample = 8
assert(first.mapped, 15)
assert(second.mapped, 25)
first.sample = 12
assert(first.mapped, 40)
assert(second.mapped, 25)
first.targetEnd = 50
assert(first.mapped, 70)
third = Gauge()
assert(third.mapped, null)
third.start = 0
third.end = 4
third.targetStart = 0
third.targetEnd = 100
third.sample = 1
assert(third.mapped, 25)
delete first.sample
assert(first.mapped, null)
first.sample = -4
assert(first.mapped, -10)
assert(Gauge.length, 3)

---

# Nucleoid preserves deferred range remapping and rollback

class Sensor
sensor = Sensor()
start = sensor.lower
end = sensor.upper
targetStart = sensor.outputLower
targetEnd = sensor.outputUpper
sample = sensor.reading
mapped = Math.remap(start, end, targetStart, targetEnd, sample)
withDeferredExtra = Math.remap(start, end, targetStart, targetEnd, sample, sensor.optional)
assert(mapped, null)
assert(withDeferredExtra, null)
sensor.lower = 0
sensor.upper = 8
sensor.outputLower = 10
sensor.outputUpper = 30
sensor.reading = 2
assert(mapped, 15)
assert(withDeferredExtra, null)
sensor.optional = 99
assert(withDeferredExtra, 15)
delete sensor.reading
assert(mapped, null)
assert(withDeferredExtra, null)
sensor.reading = 6
assert(mapped, 25)
assert(withDeferredExtra, 25)
try:
    sensor.reading = 4
    throw "ROLLBACK"
catch error:
    assert(error, "ROLLBACK")
assert(sensor.reading, 6)
assert(mapped, 25)
assert(withDeferredExtra, 25)
try:
    sensor.reading = missing
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(sensor.reading, 6)
delete sensor.optional
assert(withDeferredExtra, null)
assert(mapped, 25)
sensor.optional = [1, 2]
assert(withDeferredExtra, 25)
sensor.reading = 2
assert(mapped, 15)
assert(withDeferredExtra, 15)
mapped
# return: 15
---
# Nucleoid defines comma-free bare indented blocks

def add left right:
    total = left + right
    return total
def scaled left right factor:
    return (left + right) * factor
def largest left right:
    if left > right:
        return left
    return right
def pair left right:
    return [left, right]
def coordinates x y:
    return {x, y}
def noReturn left right:
    left + right
assert(add(2, 3), 5)
assert(scaled(2, 3, 4), 20)
assert(largest(7, 3), 7)
assert(largest(2, 9), 9)
assert(pair("left", "right"), ["left", "right"])
point = coordinates(2, 3)
assert(point.x, 2)
assert(point.y, 3)
assert(noReturn(2, 3), null)
---
# Nucleoid keeps comma-free bare indented blocks reactive

offset = 1
def adjusted left right:
    return left + right + offset
alias = adjusted
frozen = adjusted.value
left = 2
right = 3
total = alias(left, right)
snapshot = total.value
assert(total, 6)
assert(snapshot, 6)
offset = 4
assert(total, 9)
left = 5
assert(total, 12)
def adjusted left right:
    return left * right + offset
assert(total, 19)
assert(frozen(2, 3), 9)
assert(snapshot, 6)
right = 2
assert(total, 14)
---
# Nucleoid preserves comma-free block methods and initializers

class Parent:
    def init left right:
        this.left = left
        this.right = right
    def total left right:
        return this.left + this.right + left + right
class Child: Parent
    def init left right:
        super(left, right)
        self.extra = 2
    def total left right:
        return this.left + this.right + left + right
child = Child(2, 3)
assert(child.left, 2)
assert(child.right, 3)
assert(child.extra, 2)
observed = child.total(4, 5)
assert(observed, 14)
child.left = 7
assert(observed, 19)
result = child.total(1, 2)
assert(result, 13)
child.right = 4
assert(observed, 20)
assert(result, 14)
assert(Parent.length, 0)
assert(Child.length, 1)
---
# Nucleoid preserves comma-free block bindings and explicit returns

left = 10
right = 20
def pair first second:
    return [first, second]
def add first second:
    first += second
    return first
def outer initial extra:
    def combine first second scale:
        return (first + second) * scale
    return combine(initial, extra, 3)
def factory ignored other:
    return (first, second) => first + second
def bareReturn first second:
    return
assert(pair(3, 4), [3, 4])
assert(pair(3), [3, null])
assert(pair(), [null, null])
assert(pair(3, 4, 99), [3, 4])
assert(add(3, 4), 7)
assert(left, 10)
assert(right, 20)
assert(outer(4, 2), 18)
callback = factory(10, 20)
assert(callback(2, 3), 5)
assert(bareReturn(3, 4), null)
---
# Nucleoid rolls back comma-free block redefinitions

source = 3
def reading left right:
    return source + left + right
result = reading(2, 4)
assert(result, 9)
try:
    def reading left right:
        return missing + left + right
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(result, 9)
source = 5
assert(result, 11)
def combined first second scale:
    return (first + second) * scale
def prefer current fallback:
    current ||= fallback
    return current
assert(prefer(0, 9), 9)
assert(prefer(3, 9), 3)
assert(combined(2, 5, 3), 21)
combined(2, 5, 3)
# return: 21

---
# Nucleoid flat maps lists by one level

numbers = [1, 2, 3]
def expand(value) => [value, value * 10]
expanded = numbers.flatMap(expand)
assert(expanded, [1, 10, 2, 20, 3, 30])
assert(numbers, [1, 2, 3])
mixed = [[[1]], [2, 3], 4, "xy", []]
flattened = mixed.flatMap(item => item)
assert(flattened, [[1], 2, 3, 4, "xy"])
assert(mixed[0], [[1]])
nullable = numbers.flatMap(item => [item, null])
assert(nullable, [1, null, 2, null, 3, null])
empty = [].flatMap(item => missing)
assert(empty, [])
---
# Nucleoid keeps flat mapped lists reactive

numbers = [1, 2]
factor = 2
def expand(value) => [value, value * factor]
expanded = numbers.flatMap(expand)
assert(expanded, [1, 2, 2, 4])
factor = 3
assert(expanded, [1, 3, 2, 6])
numbers.push(4)
assert(expanded, [1, 3, 2, 6, 4, 12])
numbers.pop()
assert(expanded, [1, 3, 2, 6])
def expand(value) => [value + factor]
assert(expanded, [4, 5])
assert(numbers, [1, 2])
---
# Nucleoid flat maps class populations

class Batch(items):
    this.items = items
batch1 = Batch([1, 2])
batch2 = Batch([3])
expanded = Batch.flatMap(batch => batch.items)
assert(expanded, [1, 2, 3])
batch1.items.push(4)
assert(expanded, [1, 2, 4, 3])
batch3 = Batch([5])
assert(expanded, [1, 2, 4, 3, 5])
try:
    delete batch2
catch error:
    assert(error, TypeError("Cannot delete object 'batch2'"))
assert(expanded, [1, 2, 4, 3, 5])
batch2.items = []
assert(expanded, [1, 2, 4, 5])
assert(Batch.length, 3)
assert(batch1.items, [1, 2, 4])
---
# Nucleoid preserves flat mapping callable snapshots

numbers = [1, 2]
factor = 2
def expand(value) => [value * factor]
callback = expand
frozen = expand.value
live = numbers.flatMap(callback)
saved = numbers.flatMap(frozen)
snapshot = live.value
assert(live, [2, 4])
assert(saved, [2, 4])
assert(snapshot, [2, 4])
def expand(value) => [value + factor]
assert(live, [3, 4])
assert(saved, [2, 4])
factor = 3
assert(live, [4, 5])
assert(saved, [3, 6])
assert(snapshot, [2, 4])
---
# Nucleoid rolls back flat mapping callback failures

numbers = [1, 2]
def expand(value) => [value, value * 2]
expanded = numbers.flatMap(expand)
assert(expanded, [1, 2, 2, 4])
try:
    def expand(value):
        throw "FLAT_MAPPING_ABORT"
catch error:
    assert(error, "FLAT_MAPPING_ABORT")
assert(expanded, [1, 2, 2, 4])
assert(numbers, [1, 2])
numbers.push(3)
assert(expanded, [1, 2, 2, 4, 3, 6])
21
# return: 21
---
# Nucleoid replaces every literal string match

text = "one|two|one"
joined = text.replaceAll("|", "/")
overlap = "aaaaa".replaceAll("aa", "X")
literal = "a.b.a".replaceAll(".", "$&")
noMatch = "abc".replaceAll("z", "")
assert(joined, "one/two/one")
assert(overlap, "XXa")
assert(literal, "a$&b$&a")
assert(noMatch, "abc")
assert("a.a.a".replace(".", "/"), "a/a.a")
assert(text, "one|two|one")
joined
# return: "one/two/one"
---
# Nucleoid replaces empty string matches at Unicode boundaries

def replaceEvery(text, search, replacement):
    if search == "":
        if text == "":
            return replacement
        return replacement + text.split("").join(replacement) + replacement
    return text.split(search).join(replacement)
accent = String.fromCharCode(233)
symbol = String.fromCharCode(128512)
text = accent + symbol
empty = "".replaceAll("", "|")
inserted = text.replaceAll("", "|")
unchanged = text.replaceAll("", "")
legacy = replaceEvery(text, "", "|")
assert(empty, "|")
assert(inserted, "|" + accent + "|" + symbol + "|")
assert(inserted, legacy)
assert(unchanged, text)
assert(text, accent + symbol)
---
# Nucleoid keeps literal replacement reactive

text = "red|blue|red"
search = "|"
replacement = "/"
normalized = text.replaceAll(search, replacement)
frozen = normalized.value
assert(normalized, "red/blue/red")
replacement = "-"
assert(normalized, "red-blue-red")
text = "blue|green"
assert(normalized, "blue-green")
search = "blue"
assert(normalized, "-|green")
replacement = "cyan"
assert(normalized, "cyan|green")
assert(frozen, "red/blue/red")
assert(text, "blue|green")
---
# Nucleoid replaces literal strings in class rules

class Page(title: str):
    this.title = title
$Page.slug = $Page.title.replaceAll(" ", "-")
first = Page("One Two One")
second = Page("Three Four")
slugs = Page.map(page => page.slug)
assert(first.slug, "One-Two-One")
assert(slugs, ["One-Two-One", "Three-Four"])
first.title = "Fresh Page"
assert(first.slug, "Fresh-Page")
assert(slugs, ["Fresh-Page", "Three-Four"])
$Page.slug = $Page.title.lower().replaceAll(" ", "_")
assert(first.slug, "fresh_page")
assert(second.slug, "three_four")
assert(slugs, ["fresh_page", "three_four"])
---
# Nucleoid rolls back literal replacement argument failures

source = "a|b"
def fail():
    source = "changed|changed"
    throw "stop"
try:
    source.replaceAll("|", fail())
catch error:
    assert(error, "stop")
assert(source, "a|b")
recovered = source.replaceAll("|", "/")
assert(recovered, "a/b")
---
# Nucleoid selects direct conditional alternatives

def classify(state, urgent):
    if state == "blocked"
        return "hold"
    else urgent
        return "fast"
    elif state == "ready"
        return "send"
    else:
        return "wait"
state = "ready"
urgent = false
label = classify(state, urgent)
assert(label, "send")
urgent = true
assert(label, "fast")
state = "blocked"
assert(label, "hold")
urgent = false
assert(label, "hold")
state = "queued"
assert(label, "wait")
label
# return: "wait"
---
# Nucleoid keeps direct conditional class alternatives reactive

class Ticket
first = Ticket()
first.state = "new"
first.priority = "normal"
second = Ticket()
second.state = "review"
second.priority = "urgent"
if $Ticket.state == "blocked"
    $Ticket.queue = "hold"
else $Ticket.priority == "urgent"
    $Ticket.queue = "fast"
elif $Ticket.state == "review"
    $Ticket.queue = "review"
else:
    $Ticket.queue = "normal"
assert(first.queue, "normal")
assert(second.queue, "fast")
first.state = "blocked"
assert(first.queue, "hold")
second.priority = "normal"
assert(second.queue, "review")
first.priority = "urgent"
assert(first.queue, "hold")
first.state = "new"
assert(first.queue, "fast")
assert(second.queue, "review")
second.state = "done"
assert(second.queue, "normal")
---
# Nucleoid preserves direct alternative evaluation order

def decide(first, second, third):
    if first
        return "first"
    else second
        return "second"
    elif third
        return "third"
    else:
        return "none"
assert(decide(true, true, true), "first")
assert(decide(false, true, true), "second")
assert(decide(false, false, true), "third")
assert(decide(false, false, false), "none")
def guarded(value):
    if value
        return "ready"
    else missing
        return "bad"
    else:
        return "other"
assert(guarded(true), "ready")
try:
    guarded(false)
catch error:
    assert(error, ReferenceError("missing is not defined"))
---
# Nucleoid preserves direct alternative scope and return flow

class Router:
    def init(prefix):
        this.prefix = prefix
    def choose(status, fallback):
        if status == "blocked"
            return this.prefix + "hold"
        else status == "ready"
            return this.prefix + "send"
        elif fallback
            return this.prefix + "later"
        else:
            return this.prefix + "skip"
router = Router("a:")
status = "ready"
live = router.choose(status, false)
snapshot = live.value
assert(live, "a:send")
assert(snapshot, "a:send")
assert(router.choose("unknown", false), "a:skip")
assert(router.choose("unknown", true), "a:later")
assert(router.choose("blocked", true), "a:hold")
assert(router.choose("ready", true), "a:send")
assert(status, "ready")
assert(router.prefix, "a:")
---
# Nucleoid rolls back failed direct alternatives

source = "ready"
def route(status):
    if status == "ready"
        return "send"
    else status == "hold"
        return "wait"
    else:
        return "skip"
live = route(source)
assert(live, "send")
try:
    source = "hold"
    def route(status):
        if status == "ready"
            return "send"
        elif status == "hold"
            throw "blocked"
        else:
            return "skip"
catch error:
    assert(error, "blocked")
assert(source, "ready")
assert(live, "send")
source = "hold"
assert(live, "wait")
source = "unknown"
assert(live, "skip")
live
# return: "skip"
---
# Nucleoid constructs comma-free explicit records

amount = 4
unit = "C"
reading = {amount: amount unit: unit active: true}
assert(reading.amount, 4)
assert(reading.unit, "C")
assert(reading.active, true)
amount = 9
assert(reading.amount, 9)
unit = "F"
assert(reading.unit, "F")
reading.unit
# return: "F"
---
# Nucleoid preserves comma-free record keys and replacement order

seed = 3
record = {"state": "open" 1: "first" "1": "latest" copy: seed copy: seed + 1}
assert(record.state, "open")
assert(record["1"], "latest")
assert(record.copy, 4)
seed = 8
assert(record.copy, 9)
record.state = "closed"
assert(record.state, "closed")
---
# Nucleoid nests comma-free records without changing list or call separators

def pair(left, right) => left + right
first = 2
packet = {meta: {kind: "sample" ready: true} data: [{reading: first unit: "C"}, {reading: pair(first, 3) unit: "K"}] tag: "batch"}
assert(packet.meta.kind, "sample")
assert(packet.data[0].reading, 2)
assert(packet.data[1].reading, 5)
assert(packet.data[1].unit, "K")
first = 7
assert(packet.data[0].reading, 7)
assert(packet.data[1].reading, 10)
---
# Nucleoid preserves scope and snapshots in comma-free records

def package(amount):
    return {reading: amount initial: amount.value label: "sample"}
source = 3
item = package(source)
view = {current: source saved: source.value packed: item.reading}
assert(item.reading, 3)
assert(view.current, 3)
assert(view.saved, 3)
source = 6
assert(item.reading, 6)
assert(view.current, 6)
assert(view.saved, 3)
assert(view.packed, 6)
---
# Nucleoid rolls back failed comma-free record values

state = 1
record = {status: "old" marker: state}
message = ""
try:
    state = 2
    record = {status: "new" marker: missing status: "last"}
catch error:
    message = error
assert(message, ReferenceError("missing is not defined"))
assert(state, 1)
assert(record.status, "old")
assert(record.marker, 1)
state = 3
assert(record.marker, 3)
record.status
# return: "old"
---
# Nucleoid checks values with colon-led assertion statements

reading = {amount: 3, unit: "C", active: true, empty: null}
label = "sensor"
assert: reading.amount, 3
assert: reading.unit, "C"
assert: reading.active, true
assert: reading.empty, null
assert: [reading.amount, label], [3, "sensor"]
assert: {kind: label, valid: reading.active}, {kind: "sensor", valid: true}
---
# Nucleoid keeps colon-led assertions out of reactive reasoning

source = 2
total = source + 1
assert: total, 3
assert: (affects source).length, 1
assert: (why total)[1].node, "source"
source = 6
assert: total, 7
assert: (why total)[1].holds, 6
total
# return: 7
---
# Nucleoid preserves scope and return flow around colon-led assertions

def check_name(name):
    saved = name.value
    assert: saved, "draft"
    return {name: name, initial: saved}
packet = check_name("draft")
assert: packet.name, "draft"
assert: packet.initial, "draft"
assert: packet.name == packet.initial, true
---
# Nucleoid preserves colon-led assertion argument evaluation order

log = []
def note(label):
    log.push(label)
    return 7
assert: note("actual"), note("expected")
assert: log, ["actual", "expected"]
assert: log.length, 2
---
# Nucleoid checks caught errors and rollback with colon-led assertions

state = 1
message = ""
try:
    state = 2
    failed = missing
catch error:
    message = error
    assert: error, ReferenceError("missing is not defined")
assert: state, 1
assert: message, ReferenceError("missing is not defined")
---
# Nucleoid catches thrown values with colon-free indented suites

status = "ready"
try
    throw "STOP"
catch problem
    assert(problem, "STOP")
assert(status, "ready")
try
    throw 7
catch (issue)
    assert(issue, 7)
try
    missing
catch
    assert(error, ReferenceError("missing is not defined"))
---
# Nucleoid restores dependencies through colon-free exception suites

source = 2
derived = source * 3
try
    source = 9
    throw "ABORT"
catch failure
    assert(failure, "ABORT")
assert(source, 2)
assert(derived, 6)
source = 4
assert(derived, 12)
derived
# return: 12
---
# Nucleoid preserves nested catch association without suite colons

state = "ready"
try
    try
        throw "INNER"
    catch inner
        throw inner + "-OUTER"
catch outer
    assert(outer, "INNER-OUTER")
assert(state, "ready")
---
# Nucleoid preserves function returns through compact exception suites

def recover(flag):
    try
        if flag:
            throw "FALLBACK"
        return "OK"
    catch issue
        return issue
first = recover(false)
second = recover(true)
assert(first, "OK")
assert(second, "FALLBACK")
---
# Nucleoid rolls back class rules with colon-free catch suites

class Parcel(state: str):
    this.state = state
parcel = Parcel("queued")
$Parcel.label = $Parcel.state + "!"
try
    parcel.state = "broken"
    throw "ABORT"
catch reason
    assert(reason, "ABORT")
assert(parcel.state, "queued")
assert(parcel.label, "queued!")
parcel.state = "sent"
assert(parcel.label, "sent!")
---
# Nucleoid binds opted-in constructor arguments by name

a = 10
b = 20
class Pair(a, b):
    this.a = a
    this.b = b
pair = Pair(b=2, a=1)
assert(pair, {"id": "pair", "a": 1, "b": 2})
assert(a, 10)
assert(b, 20)
---
# Nucleoid evaluates named constructor values in source order

'use imperative'
class Pair(a, b):
    this.a = a
    this.b = b
counter = 0
pair = Pair(b=(counter = counter + 1), a=(counter = counter + 1))
assert(pair.a, 2)
assert(pair.b, 1)
assert(counter, 2)
---
# Nucleoid inherits opted-in named constructors

class Parent(a, b):
    this.a = a
    this.b = b
class Child: Parent
    pass
child = Child(b=2, a=1)
assert(child, {"id": "child", "a": 1, "b": 2})
---
# Nucleoid binds named parameters through opted-in initializers

class Pair:
    def init(a, b):
        this.a = a
        this.b = b
pair = Pair(b=2, a=1)
assert(pair, {"id": "pair", "a": 1, "b": 2})
---
# Nucleoid keeps missing opted-in constructor parameters null

class Pair(a, b):
    this.a = a
    this.b = b
pair = Pair(a=1)
assert(pair.a, 1)
assert(pair.b, null)
---
# Nucleoid snapshots named constructor argument values

left = 1
right = 2
class Pair(a, b):
    this.a = a
    this.b = b
pair = Pair(b=right, a=left)
assert(pair.a, 1)
assert(pair.b, 2)
left = 3
assert(left, 3)
assert(pair.a, 1)
assert(pair.b, 2)
---
# Nucleoid preserves positional constructor assignment arguments

class Legacy(a, b):
    this.a = a
    this.b = b
a = b = 0
legacy = Legacy((b = 2), (a = 1))
assert(legacy.a, 2)
assert(legacy.b, 1)
assert(a, 1)
assert(b, 2)
---
# Nucleoid forwards named arguments to opted-in parent constructors

class Parent(a, b):
    this.a = a
    this.b = b
class Child(first, second): Parent
    super(b=second, a=first)
child = Child(1, 2)
assert(child, {"id": "child", "a": 1, "b": 2})
---
# Nucleoid constructs students with positional and named arguments

class Student(name):
    this.name = name
positional = Student("Can")
named = Student(name="Can")
assert(positional.name, "Can")
assert(named.name, "Can")
---
# Nucleoid mixes positional and named constructor arguments

class Student(name, age):
    this.name = name
    this.age = age
ordered = Student(name="Can", age=20)
reordered = Student(age=20, name="Can")
mixed = Student("Can", age=20)
assert(ordered, {"id": "ordered", "name": "Can", "age": 20})
assert(reordered, {"id": "reordered", "name": "Can", "age": 20})
assert(mixed, {"id": "mixed", "name": "Can", "age": 20})
---
# Nucleoid validates constructor names without caller assignments

class Student(name, age):
    this.name = name
    this.age = age
name = "Before"
student = Student(name="Can", age=20)
assert(name, "Before")
assert(student.name, "Can")
try:
    Student(nmae="Can", age=20)
catch error:
    assert(error, TypeError("Unknown constructor argument 'nmae' for Student"))
try:
    Student("Can", name="Other", age=20)
catch error:
    assert(error, TypeError("Duplicate constructor argument 'name' for Student"))
---
# Nucleoid preserves ordinary function argument assignments

'use imperative'
def pair(a, b):
    return [a, b]
a = 0
b = 0
result = pair(b=2, a=1)
assert(result, [2, 1])
assert(a, 1)
assert(b, 2)
---
# Nucleoid filters reasoning selections by stated and derived state

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
---
# Nucleoid keeps reasoning state filters live through equal valued rewrites

seed = 1
value = seed + 1
facts = value |> why |> stated
conclusions = value |> why |> derived
assert(facts.map(step => step.node), ["seed"])
assert(conclusions.map(step => step.node), ["value"])
value = 2
assert(facts.map(step => step.node), ["value"])
assert(conclusions, [])
value = seed + 1
seed = 3
assert(facts.map(step => step.node), ["seed"])
assert(facts[0].holds, 3)
assert(conclusions.map(step => step.node), ["value"])
assert(conclusions[0].holds, 4)
---
# Nucleoid observes empty state filtered model selections

facts = model |> stated
conclusions = model |> derived
assert(facts, [])
assert(conclusions, [])
seed = 1
value = seed + 1
assert(facts.map(step => step.node), ["seed"])
assert(conclusions.map(step => step.node), ["value"])
delete value
assert(conclusions, [])
seed = 2
assert(facts[0].holds, 2)
---
# Nucleoid retains derived class rules without input edges in state filters

class Subject
$Subject.active = true
item = Subject()
facts = item.active |> why |> stated
conclusions = item.active |> why |> derived
assert(facts, [])
assert(conclusions.length, 1)
assert(conclusions[0].node, "item.active")
assert(conclusions[0].from, [])
assert(conclusions[0].rule, "$Subject.active = true")
assert(conclusions[0].holds, true)
---
# Nucleoid restores state filtered explanations and preserves snapshots

seed = 1
value = seed + 1
facts = value |> why |> stated
conclusions = value |> why |> derived
frozen = facts.value
try:
    value = 2
    throw "ABORT_STATE_FILTER"
catch error:
    assert(error, "ABORT_STATE_FILTER")
assert(facts.map(step => step.node), ["seed"])
assert(conclusions.map(step => step.node), ["value"])
value = 2
seed = 5
assert(facts.map(step => step.node), ["value"])
assert(facts[0].holds, 2)
assert(conclusions, [])
assert(frozen[0].node, "seed")
assert(frozen[0].holds, 1)
---
# Nucleoid filters individual references without traversing their ancestors

seed = 1
value = seed + 1
fact = stated seed
notFact = stated value
conclusion = derived value
assert(fact.map(step => step.node), ["seed"])
assert(notFact, [])
assert(conclusion.map(step => step.node), ["value"])
---
# Nucleoid preserves reference errors while filtering reasoning states

seed = 1
value = seed + 1
facts = value |> why |> stated
try:
    1 |> stated
catch error:
    assert(error, TypeError("Cannot explain 1"))
assert(facts.map(step => step.node), ["seed"])
try:
    stated missing
catch error:
    assert(error, ReferenceError("missing is not defined"))

---
# Nucleoid keeps degree to radian conversion reactive

degrees = 180
converted = Math.toRadians(degrees)
assert(converted, Math.PI)
degrees = 90
assert(converted, Math.PI / 2)
degrees = -180
assert(converted, -Math.PI)
degrees = 360
assert(converted, Math.PI * 2)
degrees = 0
assert(converted, 0)
degrees = -0
assert(converted, 0)
assert(1 / converted, Number.NEGATIVE_INFINITY)
degrees = 90
converted
# return: 1.5707963267948966
---
# Nucleoid preserves numeric coercion while converting degrees to radians

value = " 180 "
converted = Math.toRadians(value)
assert(converted, Math.PI)
value = true
assert(converted, Math.PI / 180)
value = false
assert(converted, 0)
value = []
assert(converted, 0)
value = ["90"]
assert(converted, Math.PI / 2)
value = Date(180)
assert(converted, Math.PI)
value = "not-a-number"
assert(Number.isNaN(converted), true)
literalNull = Math.toRadians(null)
assert(literalNull, 0)
---
# Nucleoid preserves radian conversion extremes omissions and signed zeros

finite = Math.toRadians(Number.MAX_VALUE)
assert(Number.isFinite(finite), true)
negativeFinite = Math.toRadians(-Number.MAX_VALUE)
assert(Number.isFinite(negativeFinite), true)
assert(negativeFinite < 0, true)
positive = Math.toRadians(Number.POSITIVE_INFINITY)
negative = Math.toRadians(Number.NEGATIVE_INFINITY)
invalid = Math.toRadians(Number.NaN)
omitted = Math.toRadians()
zero = Math.toRadians(-0)
subnormal = Math.toRadians(Number.MIN_VALUE)
assert(positive, Number.POSITIVE_INFINITY)
assert(negative, Number.NEGATIVE_INFINITY)
assert(Number.isNaN(invalid), true)
assert(Number.isNaN(omitted), true)
assert(1 / zero, Number.NEGATIVE_INFINITY)
assert(subnormal > 0, true)
---
# Nucleoid applies radian conversion to independent class rules

class Angle(degrees):
    this.degrees = degrees
$Angle.radians = Math.toRadians($Angle.degrees)
first = Angle(180)
second = Angle(90)
assert(first.radians, Math.PI)
assert(second.radians, Math.PI / 2)
first.degrees = -180
assert(first.radians, -Math.PI)
assert(second.radians, Math.PI / 2)
delete first.degrees
assert(first.radians, null)
first.degrees = 360
assert(first.radians, Math.PI * 2)
second.degrees = 45
assert(second.radians, Math.PI / 4)
---
# Nucleoid restores radian conversion relationships after failed transactions

class Dial
dial = Dial()
dial.degrees = 180
converted = Math.toRadians(dial.degrees)
assert(converted, Math.PI)
try:
    dial.degrees = 90
    throw "ANGLE_ROLLBACK"
catch error:
    assert(error, "ANGLE_ROLLBACK")
assert(dial.degrees, 180)
assert(converted, Math.PI)
try:
    converted = Math.toRadians(missing)
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(converted, Math.PI)
dial.degrees = 90
assert(converted, Math.PI / 2)
---
# Nucleoid preserves ignored radian conversion reads and missing extras

class Probe
probe = Probe()
angle = -180
extra = 1
converted = Math.toRadians(angle, extra)
assert(converted, -Math.PI)
extra = 2
assert(converted, -Math.PI)
withMissing = Math.toRadians(angle, probe.missing)
assert(withMissing, null)
probe.missing = 1
assert(withMissing, -Math.PI)
angle = 90
assert(converted, Math.PI / 2)
assert(withMissing, Math.PI / 2)
try:
    Math.toRadians(angle, missing)
catch error:
    assert(error, ReferenceError("missing is not defined"))
---
# Nucleoid iterates string characters in source order

text = "cab"
seen = []
for character of text:
    seen.push(character)
assert(seen, ["c", "a", "b"])
assert(text, "cab")
seen
# return: ["c","a","b"]
---
# Nucleoid preserves Unicode scalars while iterating strings

text = "Aé🙂é"
seen = []
for character of text:
    seen.push(character)
assert(seen, ["A", "é", "🙂", "e", "́"])
assert(seen.length, 5)
assert(seen.join(""), text)
---
# Nucleoid snapshots string sources before loop bodies

holder = {text: "ab"}
seen = []
for character of holder.text:
    seen.push(character)
    holder.text = "z"
assert(seen, ["a", "b"])
assert(holder.text, "z")
holder.text = "cd"
assert(seen, ["a", "b"])
for character of holder.text:
    seen.push(character)
assert(seen, ["a", "b", "c", "d"])
assert(holder.text, "cd")
---
# Nucleoid keeps string iterator scopes and nested loops independent

it = "outer"
letters = "ab"
digits = "12"
seen = []
for letters:
    seen.push(it)
    for digits:
        seen.push(it)
    seen.push(it)
assert(seen, ["a", "1", "2", "a", "b", "1", "2", "b"])
assert(it, "outer")
---
# Nucleoid skips empty strings without executing loop bodies

empty = ""
seen = []
for character in empty:
    seen.push(character)
assert(seen, [])
full = "ab"
for character of full:
    seen.push(character)
assert(seen, ["a", "b"])
for character in empty:
    seen.push(character)
assert(seen, ["a", "b"])
---
# Nucleoid rolls back failed string iteration and restores caller scope

it = "outer"
seen = ["saved"]
try:
    for it of "abc":
        seen.push(it)
        if it == "b":
            throw "STOP_STRING_ITERATION"
catch error:
    assert(error, "STOP_STRING_ITERATION")
assert(seen, ["saved"])
assert(it, "outer")
for it of "c":
    seen.push(it)
assert(seen, ["saved", "c"])
assert(it, "outer")
---
# Nucleoid reads zero based and named versions of instance properties

class Student
student1 = Student()
student1.name = "Can"
student1.name = "Daphne"
student1.name = "Merve"
version0 = student1.name@0
version1 = student1.name@1
version2 = student1.name@2
firstName = student1.name@first
lastName = student1.name@last
previousName = student1.name@previous
assert(version0, "Can")
assert(version1, "Daphne")
assert(version2, "Merve")
assert(firstName, "Can")
assert(lastName, "Merve")
assert(previousName, "Daphne")
assert(student1.name, "Merve")
[version0, version1, version2, firstName, lastName, previousName]
# return: ["Can","Daphne","Merve","Can","Merve","Daphne"]
---
# Nucleoid tracks reactive variable versions without duplicate states

input = 1
amount = input * 2
firstAmount = amount@first
lastAmount = amount@last
previousAmount = amount@previous
assert(firstAmount, 2)
assert(lastAmount, 2)
assert(previousAmount, null)
input = 2
assert(amount, 4)
assert(lastAmount, 4)
assert(previousAmount, 2)
input = 2
assert(firstAmount, 2)
assert(lastAmount, 4)
assert(previousAmount, 2)
input = 3
assert(lastAmount, 6)
assert(previousAmount, 4)
assert(firstAmount, 2)
version0 = amount@0
version1 = amount@1
version2 = amount@2
version3 = amount@3
assert(version0, 2)
assert(version1, 4)
assert(version2, 6)
assert(version3, null)
---
# Nucleoid reads versions through nulls and deferred indices

value = "ready"
oldest = value@first
future = value@2
latest = value@last
previous = value@previous
assert(oldest, "ready")
assert(future, null)
assert(previous, null)
value = null
assert(oldest, "ready")
assert(latest, null)
assert(previous, "ready")
assert(future, null)
value = "back"
assert(oldest, "ready")
assert(latest, "back")
assert(previous, null)
assert(future, "back")
---
# Nucleoid isolates local and parameter version histories between calls

value = 99
def capture(value) => [value@first, value = value + 1, value@previous]
one = capture(2)
two = capture(10)
assert(one, [2, 3, 2])
assert(two, [10, 11, 10])
globalFirst = value.value
assert(globalFirst, 99)
---
# Nucleoid preserves list values object identity and ordinary fields in versions

items = [1]
items.push(2)
earlierList = items@first
latestList = items@last
previousList = items@previous
assert(earlierList, [1])
assert(latestList, [1, 2])
assert(previousList, [1])
class Record
first = Record()
first.first = "given"
first.last = "family"
first.previous = "ordinary"
second = Record()
selected = first
selected = second
earlierObject = selected@first
latestObject = selected@last
previousObject = selected@previous
first.label = "updated"
assert(earlierObject.label, "updated")
assert(latestObject, second)
assert(previousObject, first)
assert(first.first, "given")
assert(first.last, "family")
assert(first.previous, "ordinary")
---
# Nucleoid restores version histories on rollback and resets deleted bindings

value = "A"
try:
    value = "discarded"
    throw "ABORT_VERSIONS"
catch error:
    assert(error, "ABORT_VERSIONS")
value = "B"
firstValue = value@first
lastValue = value@last
previousValue = value@previous
assert(firstValue, "A")
assert(lastValue, "B")
assert(previousValue, "A")
delete value
value = "C"
resetFirst = value@first
resetPrevious = value@previous
assert(resetFirst, "C")
assert(resetPrevious, null)
assert(firstValue, "C")
assert(lastValue, "C")
assert(previousValue, null)
---
# Nucleoid fills omitted literal default parameters

def price(quantity, unit = 10) => quantity * unit
one = price(1)
two = price(2)
three = price(3)
custom = price(2, 7)
assert(one, 10)
assert(two, 20)
assert(three, 30)
assert(custom, 14)
---
# Nucleoid preserves supplied falsy arguments when defaults exist

def pick(tag, fallback = 12) => [tag, fallback]
one = pick("a")
two = pick("b")
three = pick("c")
suppliedZero = pick("z", 0)
suppliedFalse = pick("f", false)
suppliedEmpty = pick("e", "")
suppliedNull = pick("n", null).value
assert(one, ["a", 12])
assert(two, ["b", 12])
assert(three, ["c", 12])
assert(suppliedZero, ["z", 0])
assert(suppliedFalse, ["f", false])
assert(suppliedEmpty, ["e", ""])
assert(suppliedNull, ["n", null])
def flag(tag, enabled = false) => [tag, enabled]
flagA = flag("a")
flagB = flag("b")
flagC = flag("c")
assert(flagA, ["a", false])
assert(flagB, ["b", false])
assert(flagC, ["c", false])
def absent(tag, missing = null) => [tag, missing]
nullA = absent("a").value
nullB = absent("b").value
nullC = absent("c").value
assert(nullA, ["a", null])
assert(nullB, ["b", null])
assert(nullC, ["c", null])
---
# Nucleoid applies literal defaults to typed functions and parenthesized arrows

def scale(value: Number, factor: Number = 0.5) => value * factor
adjust = (value, offset = -2) => value + offset
one = scale(2)
two = scale(4)
three = scale(6)
four = adjust(1)
five = adjust(3)
six = adjust(5)
assert(one, 1)
assert(two, 2)
assert(three, 3)
assert(four, -1)
assert(five, 1)
assert(six, 3)
---
# Nucleoid starts defaulted parameter histories at their effective values

step = 100
def sample(seed, step = 4) => [seed, step = step + 1, step@first, step@last]
one = sample(1)
two = sample(2)
three = sample(3)
provided = sample(4, 9)
assert(one, [1, 5, 4, 5])
assert(two, [2, 5, 4, 5])
assert(three, [3, 5, 4, 5])
assert(provided, [4, 10, 9, 10])
assert(step, 100)
---
# Nucleoid binds independent literal constructor and method defaults

class Counter(seed = 1):
    this.count = seed
    def advance(delta = 2):
        this.count += delta
        return this.count
first = Counter()
second = Counter()
third = Counter()
one = first.advance().value
two = second.advance().value
three = third.advance().value
again = first.advance().value
supplied = Counter(5)
custom = supplied.advance(4).value
assert(first.count, 5)
assert(second.count, 3)
assert(third.count, 3)
assert(supplied.count, 9)
assert(one, 3)
assert(two, 3)
assert(three, 3)
assert(again, 5)
assert(custom, 9)
---
# Nucleoid restores defaulted function callers after failed redefinitions

def price(quantity, unit = 10) => quantity * unit
one = price(1)
two = price(2)
three = price(3)
four = price(4)
assert(one, 10)
assert(two, 20)
assert(three, 30)
assert(four, 40)
try:
    def price(quantity, unit = 10) => quantity * unit + 1
    assert(one, 11)
    assert(two, 21)
    assert(three, 31)
    assert(four, 41)
    throw "RESTORE_LITERAL_DEFAULT"
catch error:
    assert(error, "RESTORE_LITERAL_DEFAULT")
assert(one, 10)
assert(two, 20)
assert(three, 30)
assert(four, 40)
```

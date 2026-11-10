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

Compound arithmetic assignments `+=`, `-=`, `*=`, `**=`, `/=`, and `%=` lower
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
Diagnostics name the operator exactly as `'+='`, `'-='`, `'*='`, `'**='`,
`'/='`, `'%='`, `'&&='`, or `'||='`. The compound characters must be adjacent;
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

## Extends clauses for subclasses

- [NUC 78](docs/nuc-0078.md) adds `extends` as an optional spelling of a
  class header's parent, as JavaScript, TypeScript, Java, PHP, Dart, Scala and
  Groovy write it. `class Name: Parent` keeps its grammar and meaning. No
  token, keyword, syntax tree node or runtime path is added.
- After a class name and its optional parameter list, the identifier `extends`
  and one parent name form the clause, as in `class Dog extends Animal` and
  `class Dog(name, breed) extends Animal`. The parent is an ordinary
  identifier, as after `class Name:`, and need not be declared yet.
- A colon after the clause opens the class body that follows
  `class Name(params):`: an indented body of typed fields, constructor
  statements, `init` and methods, or one statement on the same line.
  `class Dog extends Animal:` with an indented body declares the class that
  `class Dog: Animal` with that body declares.
- After that colon the accepted class body grammar applies unchanged. As
  after `class Dog:` and `class Dog: Animal`, a colon that ends its line
  without an indented body takes the next statement, if any, as the class's
  one constructor statement, and such a header at the end of an indented
  block raises `SyntaxError("Unexpected the end of a block")`.
- Without a colon the header is bodyless, as `class Name` and
  `class Name(params)` are. It ends at a logical newline, semicolon, end of
  input, or the enclosing block's closing brace or dedent, and the following
  statement is not part of a constructor. `class Dog extends Animal` declares
  the class that `class Dog: Animal` with an indented `pass` declares.
- An indented body still requires the colon. A bodyless clause followed by a
  newline and indentation raises
  `SyntaxError("Expected ':' but found end of line")`, or
  `SyntaxError("Expected ':' but found ';'")` when the first separator is a
  semicolon.
- Every spelling builds the same class declaration: the same name, parent,
  parameters, fields, constructor statements and methods. Registration, the
  `Class` list, separate parent and child instance lists, `$Parent` rules on
  child instances, constructors and the nearest ancestor constructor, `init`,
  `super` and bare `super`, named constructor arguments, inheritance cycle
  checks, redeclaration, graph keys, `why`, canonical rendering and
  transactions are those of the colon spelling, which canonical source keeps
  writing.
- `extends` is not a keyword. Only the identifier `extends` directly after a
  class name or its parameter list starts the clause, a position where the
  accepted parser raised
  `SyntaxError("Expected ':' but found identifier 'extends'")`. Variables,
  properties, parameters, functions, methods, classes and parents named
  `extends` keep their meanings, and `class extends extends Animal` declares
  a class named `extends` whose parent is `Animal`.
- A parent that is not a name raises
  `SyntaxError("Expected a parent class name but found <token>")` at that
  token, with the existing token descriptions. `class Dog extends` reports
  `end of input` at line 1, column 18, and `class Dog extends 5`,
  `class Dog extends "A"` and `class Dog extends $Animal` report `number 5`,
  `a string` and `'$Animal'` at line 1, column 19.
- A second parent raises `SyntaxError("Class 'Dog' already extends 'Animal'")`
  for `class Dog extends Animal: Cat`, at `Cat`. The check covers a name
  followed by the end of the line or input after the colon, the position that
  names the parent in `class Dog: Cat`.
- Any other token after the parent raises the existing
  `SyntaxError("Expected ':' but found <token>")` at that token, as in
  `class Dog extends Animal Cat`, `class Dog extends Animal(x)`,
  `class Dog extends Animal, Cat` and `class Dog extends Animal {}`.
- Only these former syntax errors change. The REPL runs a bodyless clause at
  once and buffers a clause that ends in a colon, as it does other headers;
  the terminal workspace indents after that colon and completes `extends`.
- The six appended entries beginning
  `Nucleoid declares subclasses with extends clauses` through
  `Nucleoid rolls back extends class declarations` specify the executable
  behavior.

## Named function declarations

- [NUC 89](docs/nuc-0089.md) adds the named function declaration of
  JavaScript, TypeScript and PHP: the keyword `function`, a name, a
  parenthesized parameter list and a braced body, as in
  `function double(amount) { return amount * 2 }`. It declares the function
  that `def double(amount):` declares with the same statements. `def` keeps
  its grammar and meaning, and no token, keyword, syntax tree node or runtime
  path is added.
- The parameter list is the parenthesized list of `def`: annotations, literal
  defaults, a final rest parameter, a trailing comma and newlines inside the
  parentheses. In a class body, `init` also takes the `this.name` and
  `self.name` parameters of [NUC 87](docs/nuc-0087.md). The bare, comma-free
  and typed bare parameter lists remain spellings of `def` only. A PHP
  parameter such as `$a` remains a class reference, so
  `function f($a) { return $a }` raises
  `SyntaxError("Expected a parameter but found '$a'")`.
- The body is the braced block of an anonymous `function (params) { ... }`
  literal: statements separated by newlines, semicolons or nothing, which may
  contain colon suites and further blocks. Its `{` directly follows the
  closing parenthesis on the same line, and `{}` is an empty body, which
  returns `null`.
- The declaration is a statement and may start wherever a statement starts:
  at the top level, in indented and same-line suites, in braced blocks and
  function bodies, and after another statement on the same line, as in
  `function f(a) { return a } f(2)`. In an indented class body it declares a
  method, and `function init(...)` becomes the constructor, as `def` does
  there.
- Every spelling builds the same declaration: the same name, parameters and
  block body. The function is stored in the state when its declaration runs,
  so a call before that raises `ReferenceError("<name> is not defined")`, as
  for `def`; declarations are not hoisted. Redefinition and the callers it
  updates, live and frozen `.value` definitions, scopes, graph keys, `why`,
  `affects`, transactions and canonical rendering are those of `def`, which
  canonical source keeps writing.
- `function` followed by a name is a declaration only where a statement
  starts. In an expression the accepted error remains:
  `g = function f(a) { return a }` raises
  `SyntaxError("Expected '(' but found identifier 'f'")` at line 1, column 14.
  Anonymous literals, and `function` as a property name after `.`, keep their
  meanings.
- After the parameter list, any token other than `{` raises
  `SyntaxError("Expected '{' but found <token>")` at that token, as the
  anonymous literal does. Arrows, colon suites and return annotations are not
  added: `function f(a) => a` reports `'=>'` at line 1, column 15, and
  `function f(a): number { return a }` reports `':'` at line 1, column 14. A
  name without a parameter list raises
  `SyntaxError("Expected '(' but found <token>")`, so `function amount:`
  followed by an indented body reports `':'` at line 1, column 16.
- Only former syntax errors change. The accepted parser consumes `function`
  only as an anonymous literal, which requires `(` next, or as a property name
  after `.`, so every input in which `function` is followed by a name other
  than after `.` was a syntax error. The declaration is recognized only at
  such a token.
- The REPL buffers a pending input that contains such a declaration until its
  braces close, through blank lines, and runs it then, or after a blank line
  when the input also opened a colon header. The terminal workspace submits
  explicitly and needs no change.
- The six appended entries beginning
  `Nucleoid declares named functions with the function keyword` through
  `Nucleoid rolls back failed function declarations` specify the executable
  behavior.

## C-style comments

- `//` line comments and non-nesting `/* ... */` block comments sit beside
  `#` line comments. [NUC 76](docs/nuc-0076.md) introduced them behind a
  `--c-style-comments` flag and a `SyntaxOptions` field; both were removed,
  and every source reads these comments.
- The openers take priority over regex and division recognition outside
  literals. `//` skips to LF or EOF without consuming LF; `/*` skips through
  the first `*/`, without nesting. Other comment markers and quotes within a
  block are text. So `//` and `// comment` are comments rather than an empty
  regex, `/* comment */` is a comment rather than a regex, and `1 // 2 /` is
  `1`.
- Comments separate rather than concatenate tokens: `left/**/right` is two
  identifiers, and `+/**/=` is not `+=`. Block LFs remain layout newlines,
  suppressed inside parentheses and brackets as usual, and suppressed breaks
  do not reset operand context for division or `/=`.
- Only literal whitespace outside comments contributes indentation; block
  text and its internal whitespace have zero indentation width. Comment-only
  lines cannot create a body. Tabs keep width four. Source locations retain
  the original LF/CRLF lines and one-based character columns, including
  Unicode; a tab counts as one character in a diagnostic column.
- Ordinary division, `/=`, nonambiguous regex literals, escaped regex slashes,
  regex character classes, quoted strings and template literal text retain
  their meanings. An empty-matching regex is written `/(?:)/`; its pattern
  text is `(?:)`, not the empty string. No regex constructor is introduced.
  Template interpolations follow the same rules: their strings, regexes and
  comments protect brace boundaries, and a syntax error inside an
  interpolation reports the line and column of the original source, so
  ``x = `${a ?? }` `` reports `Unexpected end of input` at line 1, column 13.
  Outer template quoting rules are unchanged.
- `run`, `check`, the lexer's `tokenize`, the parser's `parse_program`,
  `parse` and `parse_expression`, and `statement::compile` all read comments.
- CLI file execution, plain/piped continuation and terminal submission read
  comments. Comment punctuation cannot create a continuation header, so a
  line such as `x = 1 # note:` runs at once rather than opening a buffer, as a
  line whose last code token is `:` does. Plain input
  buffers an open block through blank lines and command text until it closes;
  EOF reports it once if still open. Ctrl-C cancels pending input. Terminal
  Ctrl-J indentation reads the lines above the cursor and skips comments.
  Existing result/error formatting and REPL exit codes remain.
- An unclosed block raises `ErrorKind::Syntax` with exact message
  `Unterminated block comment`, at the original opening slash, also within
  interpolations. Its display is `SyntaxError: Unterminated block comment
  (line L, column C)`. Compilation fails before statements execute.
- `tests/c_style_comments.rs`, `tests/c_style_comments_cli.rs` and the
  terminal unit suite execute this behavior.

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
  `SyntaxError("Unexpected character '|'")` at column 17. Since
  [NUC 59](docs/nuc-0059.md) made `?` a token, the nullable-looking
  `def f left: int?, right => left` fails the bare-header scan instead and
  gives `SyntaxError("Expected '(' but found identifier 'left'")` at column 7.
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
  gives `SyntaxError("Unexpected character '|'")` at line 1, column 17. Since
  [NUC 59](docs/nuc-0059.md) made `?` a token, `def f left: int?, right:`
  fails the bare-header scan instead and gives
  `SyntaxError("Expected '(' but found identifier 'left'")` at column 7.
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

## Reactive list flattening

- [NUC 103](docs/nuc-0103.md) adds the read-only list query `flat`,
  JavaScript's `Array.prototype.flat`, to ordinary lists and class
  populations, the receivers of `flatMap`, through the same shared
  list-query route. No syntax, keyword, error type or message is added.
- `list.flat()` is `list.flat(1)`, exactly `list.flatMap(v => v)`: each item
  that is a list contributes its immediate items and every other item
  contributes itself. Order, duplicates, deeper lists and object and function
  identities are kept, an empty list contributes nothing, a string is not
  split, and the receiver is never written.
- `list.flat(n)` for a whole number `n` of at least 1 is exactly `n`
  successive `.flatMap(v => v)` steps, `list.flat(0)` is a copy, exactly
  `list.map(v => v)`, and `list.flat(Number.POSITIVE_INFINITY)` flattens until
  no item is a list. `Infinity` is not a defined name, so `list.flat(Infinity)`
  keeps raising `ReferenceError("Infinity is not defined")` while its argument
  is evaluated, unless the program defines `Infinity`.
- **Depth.** A missing depth is 1. A supplied depth converts as the index of
  `at` does: the value converts to a number as arithmetic converts it, NaN
  becomes 0 and a fraction truncates toward zero. So `null`, `undefined`,
  `false`, `""`, `Number.NaN`, text that is not a number, objects and the
  undefined value of a missing property flatten nothing, `true` is 1, `"2"`
  and `[2]` are 2 and `2.9` is 2. A depth of 0 or less flattens nothing, and
  a depth at least as deep as the receiver's nesting, such as positive
  infinity, `"Infinity"` or `1e300`, flattens every level. Arguments after
  the depth are evaluated, left to right, and ignored, as `flatMap` ignores
  its extra arguments.
- **Reads.** A level is flattened by reading each of its items, as the
  identity callback reads its parameter. `list.flat(n)` reads the items of
  the first `n` levels, `list.flat(0)` the receiver's own items and an
  infinite depth every item, so a `null` item that is read makes the
  declaration holding the call settle to `null`, exactly as its `flatMap`
  spelling does: `cells = [[1], null].flat()` stores `null`, while
  `cells = [[1, null]].flat()` keeps the `null` as an item. No other read or
  dependency is recorded for the items.
- **Live.** The call reads its receiver and arguments as every list query
  does, so a declaration holding it follows later writes of its receiver:
  `push`, `pop`, `shift`, a replaced list or property, a changed population
  and a changed depth argument. A statement that writes nothing, such as
  `rows[0].push(9)`, which writes nothing through an index, changes nothing.
- **Populations.** A population's items are instances, which are not lists,
  so `Name.flat(n)` is the instance list in population order for every depth,
  read as a dependency on the population.
- **Rendering.** Statements, `why`, `affects` and graph keys show the call as
  written: `why` reports `cells = rows.flat()`. A `flat` call and its
  `flatMap` spelling are different statements and graph keys, as
  `sum of xs` and `xs.sum()` are.
- **Transactions.** Failed statements and transactions restore flattened
  declarations, their values and their dependencies like any other
  declaration, and later updates keep working.
- **Bounds.** Flattening walks an explicit worklist, visiting each item at
  most once and taking no stack for each level, so its cost is linear in the
  items it visits. Existing depth, propagation and memory bounds apply, and
  no flattening-specific limit is added.
- **Unchanged.** Only calls of `flat` on lists and class populations change;
  they raised `TypeError("<receiver>.flat is not a function")`. Reading
  `list.flat` or `Name.flat` without a call still gives `null`, objects and
  instances keep their own `flat` properties and methods, a `null` or
  undefined receiver keeps its deferred reading, and calls on strings,
  numbers, booleans, dates, regular expressions, functions and the built-in
  namespaces keep their accepted errors.
- The six entries beginning `Nucleoid flattens nested lists by one level`
  through `Nucleoid rolls back failed flattening updates` specify the
  executable behavior. `tests/list_flattening.rs` executes the rest.

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

## Depth-limited reasoning selections

- [NUC 82](docs/nuc-0082.md) adds the pipeline stage `SELECTION |> depth(LIMIT)`.
  It keeps the steps of `SELECTION` whose distance is at most `LIMIT`, in their
  order and with their `node`, `holds`, `rule`, `state` and `from` fields
  unchanged, and drops the others.
- A step's distance is the number of edges on the shortest path by which the
  most recent walking stage reached it from that stage's input selection:
  `why` follows dependencies and `affects` follows dependents, skipping
  reasoning observers as the walks do. Edges count whether or not the nodes
  along them are selected, so pending nodes, which no selection lists, are
  walked as before. Distances are shortest paths whatever order the walk
  lists the steps in.
- `why` keeps its input steps at distance 0, so `c |> why |> depth(1)` selects
  `c` and the nodes it reads directly. `affects` never selects a node by the
  empty path, so every step it selects is at distance 1 or more:
  `a |> affects |> depth(1)` selects the direct dependents of `a`,
  `a |> affects |> depth(0)` selects nothing, and an input that is also a
  dependent of another input takes its shortest non-empty path.
- Steps that no walk produced, the one-step selection of a reference and
  `model`, are at distance 0. `stated`, `derived` and `depth` keep the
  distances of the steps they keep, so `depth` and the state filters compose
  in either order, and a later walk measures distances from its own input.
  Prefix stages compose as before: `why c |> depth(1)` is
  `(why c) |> depth(1)`.
- `LIMIT` is one expression in parentheses, evaluated in place each time the
  selection is evaluated, after the source selection. A declaration therefore
  depends on what the limit reads, so `near = c |> why |> depth(n)` is
  evaluated again when `n` changes. It must be a non-negative integer number;
  `-0` is `0`, and a limit larger than every distance keeps every step.
- A limit that evaluates to `null` or to undefined makes the reasoning
  expression undefined: the stage keeps nothing that could be known, the
  stages after it are not applied and their limits are not evaluated. A
  declaration stores that undefined value as `null` until the limit is known,
  a quantifier over it tests no member and yields undefined, and reads from
  it, such as `.length` or `.map(...)`, give `null`, as they do from a `null`
  binding.
- Every other limit raises `TypeError("<limit> is not a non-negative integer")`,
  where `<limit>` is the limit expression's path or canonical source, named as
  quantifier errors name their collection, so `$Plan.depth` is `Plan.depth`.
  This covers negative numbers, fractions, `NaN`, infinities, booleans,
  strings, lists, objects, instances, classes, functions, dates and regular
  expressions. After `n = 1.5`, `c |> why |> depth(n)` raises
  `TypeError("n is not a non-negative integer")`, and `c |> why |> depth(-1)`
  raises `TypeError("-1 is not a non-negative integer")`. The error is
  positioned at the running top-level statement and rolls back its
  transaction.
- Only `|>` followed by the name `depth` and then a `(` token is the stage, so
  `|> depth (1)` is the stage too. `depth` remains an ordinary lexer
  identifier: variables, parameters, properties, keys, functions and methods
  named `depth` keep their accepted meanings, there is no prefix form
  `depth c`, and `depth(c, 1)` remains an ordinary call. A bare `|> depth`
  keeps the parse-time `TypeError("depth is not an operation")`, and so does
  `|> depth` followed by a line break and then `(` outside parentheses and
  brackets, where a line break is a token. `|> depth()` raises
  `SyntaxError("Unexpected ')'")` and `|> depth(1, 2)` raises
  `SyntaxError("Expected ')' but found ','")`, as the parenthesized
  expressions `()` and `(1, 2)` do.
- A local or parameter, state variable, function or class named `depth`
  shadows the stage, as the accepted `stated` and `derived` filters are
  shadowed: the stage then raises `TypeError("depth is not an operation")`
  before selecting its source. A binding declared after a standing selection
  makes the selection's next evaluation raise, which fails and rolls back the
  statement that triggered it, as it does for the state filters. The accepted
  `why` and `affects` shadowing behavior does not change.
- An invalid source keeps its existing diagnostic and is checked before the
  limit is evaluated: `1 |> depth(1)` raises `TypeError("Cannot explain 1")`
  and `missing |> depth(1)` raises
  `ReferenceError("missing is not defined")`.
- Like the state filters, `depth` preserves every value and structural
  observation that its source selection established, including those of the
  steps it drops, so a selection refreshes when a dropped ancestor's
  declaration, value or edges change. A declaration that contains the stage is
  a reasoning observer, whether it is a variable, a property, a class-level
  rule, a condition or a quantifier. Functions, methods and lambdas evaluate
  it in place. Propagation order, `.value` snapshots, `suppose` blocks and
  rollback behave as for the accepted stages, and a `.value` read in the limit
  is frozen when the declaration is stored.
- The stage renders as `SOURCE|>depth(LIMIT)` in graph keys, block keys,
  conditions and `rule` text, with its source and limit rendered once each.
  Programs without the stage select, walk and filter exactly as before and
  compute no distances.
- The eight appended entries from
  `Nucleoid narrows explanations to a reasoning depth` through
  `Nucleoid restores depth selections after failed changes` specify the
  executable behavior.

## Derivation path selections

- [NUC 88](docs/nuc-0088.md) adds the pipeline stage `SELECTION |> path(TARGET)`.
  It keeps the steps of `SELECTION` that lie on a dependency path between the
  walk that selected them and one target node, in their order and with their
  `node`, `holds`, `rule`, `state` and `from` fields unchanged, and drops the
  others.
- After `why`, a step is kept when it is the target or depends on the target
  transitively, so `total |> why |> path(price)` selects the steps by which the
  price reaches the total. They are the steps whose node is the target's or one
  that `affects TARGET` selects. After `affects`, a step is kept when it is the
  target or the target depends on it transitively, so
  `price |> affects |> path(total)` selects what the price moves on its way to
  the total: the steps whose node `why TARGET` selects.
- The direction is that of the most recent `why` or `affects` in the pipeline. A
  selection that no walk produced, the one-step selection of a reference and
  `model`, uses the direction of `why`, as `depth` puts its steps at distance 0:
  `price |> path(price)` keeps the price, and `model |> path(price)` is the
  price and every node that rests on it, in model order. `stated`, `derived`,
  `depth`, `diff` and `path` keep the direction and the steps' distances, and a
  later walk sets its own, so a path stage composes with each of them in either
  order.
- Reachability follows the edges the walks follow, dependencies after `why` and
  dependents after `affects`, through no reasoning observer. Edges count whether
  or not the nodes along them are selected, so a step whose only path runs
  through a step that an earlier filter dropped is still kept.
- With the walk's own input as the target, `c |> why |> path(c)` keeps only the
  step of `c`, and `a |> affects |> path(a)` keeps nothing, as `affects` never
  selects its input. A target that no step reaches makes the selection empty.
- `TARGET` is one expression in parentheses that names a node, resolved as the
  source of a pipeline is: a name, a property path such as `order.price`,
  `this.price` in a method, or `$Class.property` inside a class-level rule for
  the instance's property. It is resolved when the stage applies, after the
  source selection and the stages before it. A target that is not a node of the
  model, such as an absent property, a pending placeholder that rules still read
  after its name was deleted, or a reasoning observer, selects nothing, and the
  selection fills in when the target is defined.
- An invalid target keeps the diagnostics of a pipeline source. A literal, a
  property path whose base holds no object, and any other expression that is
  not a reference raise `TypeError("Cannot explain <target>")`, including a
  pipeline such as `path(price |> why)` and `$Class.property` outside a
  class-level rule. A name that nothing defines raises
  `ReferenceError("<name> is not defined")`, including `model` when no node has
  that name and an undefined first name of a property path. The error is
  positioned at the running top-level statement and rolls back its
  transaction.
- Only `|>` followed by the name `path` and then a `(` token is the stage, so
  `|> path (price)` is the stage too. `path` remains an ordinary lexer
  identifier: variables, parameters, properties, keys, functions and methods
  named `path` keep their accepted meanings, there is no prefix form `path c`,
  and `path(c)` remains an ordinary call. A bare `|> path` keeps the parse-time
  `TypeError("path is not an operation")`, and so does `|> path` followed by a
  line break and then `(` outside parentheses and brackets. `|> path()` raises
  `SyntaxError("Unexpected ')'")` and `|> path(a, b)` raises
  `SyntaxError("Expected ')' but found ','")`, as the parenthesized expressions
  `()` and `(a, b)` do.
- A local or parameter, state variable, function or class named `path` shadows
  the stage, as for `stated`, `derived`, `depth` and `diff`: the stage then
  raises `TypeError("path is not an operation")` before selecting its source. A
  binding declared after a standing selection makes its next evaluation raise,
  which fails and rolls back the statement that triggered it. Methods,
  properties and object keys named `path` are not bindings. The accepted `why`
  and `affects` shadowing behavior does not change.
- An invalid source keeps its diagnostic and is reported before the target is
  resolved: `1 |> path(price)` raises `TypeError("Cannot explain 1")` and
  `missing |> path(price)` raises `ReferenceError("missing is not defined")`.
- The stage keeps every value and structural observation its source made,
  including those of the steps it drops. It also observes the structure of
  every node it walks to decide reachability and of the target, so a standing
  selection refreshes when an edge on or toward a path appears or disappears and
  when the target is declared, replaced or deleted. A declaration, property,
  class-level rule, condition or quantifier that holds the stage is a reasoning
  observer. Functions, methods and lambdas evaluate it in place, and class-level
  rules give each instance its own selection. `.value` snapshots it, and
  propagation order, `suppose` blocks and rollback behave as for the accepted
  stages.
- The stage renders as `SOURCE|>path(TARGET)` in graph keys, block keys,
  conditions and `rule` text, with its source and target rendered once each. It
  walks once from the selection's steps over the nodes they reach in the walk
  direction, then once back from the target over the edges that walk found, in
  time linear in those nodes and edges. Programs without the stage do no work
  for it.
- The ten appended entries from
  `Nucleoid narrows an explanation to the paths from one source` through
  `Nucleoid restores path selections after failed changes and hypotheticals`
  specify the executable behavior.

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

## Reactive base two exponentiation

- Accepted [NUC 56](docs/nuc-0056.md) adds only unshadowed static
  `Math.exp2(value)`. The accepted `Math.pow(2, value)` remains supported
  unchanged; this names that operation rather than changing its numerics.
- Reuse Math's existing any-supplied-`Undefined` gate before its intrinsic
  first-argument `Value::to_number` conversion. Omission supplies numeric NaN,
  not Undefined. Only the first supplied value is numerically converted.
- Compute exactly `2.0_f64.powf(first)`, the same host operation used by
  accepted `Math.pow(2, value)`. Do not substitute native `f64::exp2`,
  exponent/log algebra, integer shortcuts, fused/rescaled math or higher
  precision. Every non-NaN result agrees by bits; NaNs agree by classification,
  without a payload guarantee. No accuracy or speed improvement is specified.
- Positive and negative zero exponents both yield one. Finite exponents,
  overflow, subnormal rounding and positive-zero underflow match accepted
  powf. Positive infinity yields positive infinity, negative infinity yields
  positive zero, and numeric NaN yields numeric NaN.
- Literal null converts to zero, booleans to zero/one, strings and
  empty/singleton lists keep intrinsic conversion, and dates contribute
  milliseconds. Other nonnumeric conversions produce numeric NaN, not new
  type errors. Literal-null conversion does not override inherited live
  assignment or parameter-null deferral.
- Evaluate all supplied argument expressions exactly once left to right per
  call, including ignored extras and expressions after Undefined. Extra values
  are not numerically converted, but retain reads, dependencies, effects,
  strict errors and the any-supplied-Undefined gate.
- Missing properties, deletion and later definition retain deferral and
  recovery. Independent class rules, functions, scopes, call frames,
  receiver lookup, graph subscriptions, transactions and rollback use the
  existing machinery unchanged.
- Intrinsic conversion and the fixed base never resolve or track shadowable
  `Number` or Math constants. State/local Math bindings and ordinary `.Math`
  members keep their lookup. Reading `Math.exp2` as a value remains
  unsupported/Undefined, like sibling static helpers, not a first-class method.
- Introduce no arity, conversion, numeric or range error. Strict unknown
  arguments, including extras and expressions after Undefined, retain
  `ReferenceError("<name> is not defined")`. Unknown static methods retain
  `TypeError("Math.<name> is not a function")`, and `Math.random()` retains
  `TypeError("Math.random is not supported")`, after their unchanged gate.
  Calling through numeric `Math = 7` retains the ordinary receiver error
  `TypeError("number.exp2 is not a function")`. Preserve error positions,
  caught values and transactional recovery.
- All sibling Math helpers, accepted syntax and inherited scope limitations
  remain unchanged. Add no alias, first-class method, parser/lexer change,
  public Rust API/type, dependency, version change or resource limit.
- The six appended entries titled `Nucleoid keeps base two exponentiation
  reactive`, `Nucleoid preserves base two exponential coercion and omissions`,
  `Nucleoid preserves base two exponential extremes and underflow`,
  `Nucleoid applies base two exponentiation to independent class rules`,
  `Nucleoid restores base two exponential relationships after failed
  transactions`, and `Nucleoid preserves ignored exponential reads and missing
  extras` preserve the caller's executed accepted-pow counterparts, ordered
  assertions and expected return. New RHS call ASTs intentionally differ;
  unchanged historical sources and their complete ASTs remain protected.

## Reactive cotangent

- The [NUC 100 candidate](docs/nuc-0100.md) adds only unshadowed static
  `Math.cot(value)`, exactly the numeric composition `1 / Math.tan(value)`.
  Inputs are radians, as for tangent; the accepted composition stays available.
- Apply Math's existing any-supplied-`Undefined` gate before intrinsic
  first-argument conversion, then compute exactly `1.0 / first.tan()`.
  Do not substitute cosine divided by sine, rearrange operations, use higher
  precision, normalize angles or special-case any input.
- Every non-NaN result agrees by bits with the accepted tangent followed by
  reciprocal, including finite extremes, subnormal inputs and signed-zero
  reciprocal infinities. Positive zero yields positive infinity, negative
  zero yields negative infinity, and infinite or NaN inputs yield numeric NaN.
  NaNs agree by classification, without a payload guarantee.
- Omission supplies numeric NaN. Literal null converts to zero, booleans to
  zero/one, strings and empty/singleton lists keep intrinsic conversion, and
  dates contribute milliseconds. Other nonnumeric conversions produce
  numeric NaN, not a new type error. Conversion never resolves a shadowable
  `Number` binding.
- Every supplied expression evaluates once left to right, including ignored
  extras and expressions after Undefined. Extras are not numerically converted
  but retain their effects, strict errors, reads, dependencies and the
  any-supplied-Undefined gate.
- Missing properties, deletion and later definition retain deferral and
  recovery. Literal-null numeric conversion does not override live-input
  assignment or parameter-null deferral. Class rules remain independent per
  instance; scopes, call frames, receivers, subscriptions and transactions,
  including rollback, use the existing machinery unchanged.
- State/local `Math` bindings still shadow the static namespace. Ordinary
  `.cot` members and methods keep their lookup. Reading `Math.cot` as a value
  remains unsupported/Undefined, not a first-class callable.
- Introduce no arity, conversion, numeric or range error. Strict unknown
  arguments retain `ReferenceError("<name> is not defined")`, including extras
  after Undefined. Other unknown static methods retain
  `TypeError("Math.<name> is not a function")`, and `Math.random()` retains
  `TypeError("Math.random is not supported")`, after their unchanged gate.
  Numeric `Math = 7` retains the ordinary receiver error
  `TypeError("number.cot is not a function")`. Preserve error positions and
  caught values. Former unknown unshadowed `Math.cot(...)` calls may succeed;
  other accepted behavior and diagnostics remain unchanged.
- Add no parser/lexer form, public Rust API/type, dependency, option or project
  version. All legacy Math helpers and arithmetic forms remain available.
- The appended spec entry `Nucleoid keeps cotangents reactive`, reference
  entry `Nucleoid preserves cotangent argument dependencies`, and example
  `Cotangents keep directional ratios live as angles change` retain the
  caller's executed accepted-tangent counterparts except for their single
  designated RHS. Canonical tangent assertions, order, returns and executed
  coverage remain unchanged.

## Read-only state versions

- A postfix `reference@0`, `@1`, ... reads a non-negative, oldest-first
  version of a variable or actual object property. `@first` means zero,
  `@last` means the newest entry, and `@previous` means the entry before it.
  Decimal digit literals must fit the host's `usize`; signs, decimal points,
  exponent spellings, expressions and unknown selector words are not selectors.
  The three words remain ordinary identifiers everywhere except in a selector
  after an operand; a prefix `@first` of [At variables](#at-variables) reads
  the variable `first`.
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
  nullish/falsy substitution. Missing fixed arguments without defaults still
  bind Null; fixed-only signatures still evaluate and ignore extras, and
  annotations remain non-enforcing. Final rest binding is specified below.
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

## Final rest parameters

- An explicitly parenthesized user-defined signature may end in `...name`
  or `...name: Type`, optionally followed by its ordinary trailing comma.
  The three dots must be adjacent; no lexer token or keyword is added.
  Named block/arrow functions, function expressions, parenthesized arrows,
  methods, class headers and adopted `init` signatures share this syntax.
- There is at most one rest formal, and it must be last. A following formal
  raises `SyntaxError("Rest parameter must be last")`. A rest default raises
  `SyntaxError("Rest parameter cannot have a default")`. Its name must differ
  from each fixed formal, otherwise
  `SyntaxError("Rest parameter '<name>' duplicates a fixed parameter")` is
  raised. Duplicate fixed names retain their accepted binding/history
  semantics, including in a new rest-bearing signature.
- Bind the fixed prefix exactly as before: absence uses its scalar default
  or Null, supplied Null/Undefined/falsy values remain supplied, required
  fixed slots may follow defaults, and annotations remain non-enforcing.
  Bind every remaining positional value, in source order, to a fresh List
  under the rest name. With no remaining values bind `[]`, never Null.
- List arguments remain single elements; there is no flattening or call-site
  spread. The fresh outer list has ordinary Value semantics: local mutation
  does not mutate a caller's list binding, while contained Object references
  retain their ordinary shared identity. No deep-copy boundary is added.
- Evaluate each argument once, left to right, per actual invocation, before
  binding. A method's receiver is not an argument and never enters its rest
  list. Existing higher-order operations pass exactly their existing
  arguments; no hidden index or receiver arguments are added.
- Constructor labels may address fixed formals only. Named-only calls leave
  an empty rest list; mixed calls fill leading fixed positions, then fixed
  labels, with positional extras collected only after the fixed prefix is
  full. Naming the rest formal raises
  `TypeError("Rest parameter '<name>' cannot be a named constructor argument for <class>")`.
  Validate every label, duplicate and positional-after-named restriction
  before evaluating any value, with existing diagnostics otherwise unchanged.
- Inherited constructor selection, `init` adoption, bodyless headers and
  `super` retain their existing rules. A bodyless rest header does not create
  properties or displace a parent's nearest nonempty constructor. Purely
  positional calls still select that constructor after argument evaluation;
  calls with labels pin its signature/body first. A named-rest diagnostic
  names the effective constructor class, not a bypassed subclass.
- Functions, aliases, callbacks, callable properties/list items, methods and
  stored Function Values share binding. Function/method argument assignments
  remain positional expressions, not labels. Bare signatures and Python-style
  lambdas gain no rest syntax; call-site spread, closures, built-in changes
  and new arity/type enforcement are not introduced.
- Initialize the local rest binding once: its version zero is the complete
  collected list, including `[]`. Parameter shadowing, fresh invocation
  frames, noncapture and nontransactional locals are unchanged. Caught
  failures restore State/graph, not edits to an enclosing local binding.
- Ordinary dependencies, standing re-evaluation, `.value` settlement and
  snapshots are unchanged. Live callers observe redefinitions; frozen and
  historical Function Values keep their own signature/body but continue to
  read ordinary live state. Failed updates restore stored definitions,
  class metadata, instances, histories and the graph through existing rollback.
- Every fixed-only signature, including runtime-failing accepted inputs,
  retains its AST Debug/display representation, canonical graph keys,
  diagnostics and behavior. Fixed-only calls still evaluate and ignore
  surplus arguments. The five appended entries beginning `Nucleoid collects
  surplus positional arguments in fresh rest lists` specify this addition.

## Initializing constructor parameters

- [NUC 87](docs/nuc-0087.md) lets a constructor parameter written
  `this.name` or `self.name` declare the parameter `name` and assign it to
  the new instance's property `name`, as Dart's initializing formals and
  TypeScript's parameter properties do. `class Point(this.x, this.y)`
  declares exactly what `class Point(x, y):` with the indented body
  `this.x = x` and `this.y = y` declares. No token, keyword, syntax tree
  node or runtime path is added.
- Only constructor parameter lists written in parentheses accept the
  prefix: the list of a class header, whether the header is bodyless, has a
  colon body on its own lines or on the same line, names its parent after a
  colon or with `extends`; and the list of a `def init(...)` written directly
  in a class's indented body, with a block, same-line or arrow body.
- A prefixed parameter is `this` or `self`, `.` and one identifier,
  directly followed by `,`, `)`, a type annotation `: Type` or a literal
  default `= literal`. The annotation, the literal default and, on the final
  parameter, the rest marker `...this.name` keep their meanings.
- The prefixed parameter is the ordinary parameter `name`. The statement
  `this.name = name` starts the constructor body, one per prefixed
  parameter in parameter order, before every written statement, including a
  leading `super(...)`. In a `def init`, the statements start the method's
  body, and an arrow body `=> expression` is the statement
  `return expression` after them.
- Binding is unchanged: positional, labelled and mixed arguments, missing
  arguments as `null`, literal defaults, rest lists and repeated names,
  where the later binding wins. So are dependencies, `why`, which explains
  `p.x` by the rule `$Point.x = x`, transactions, redeclaration, class-level
  rules, populations, `init` adoption, `super`, canonical rendering and graph
  keys.
- A header with a prefixed parameter has a nonempty constructor even when it
  has no body, so `class Shell(this.code: int)` is not an empty class, and
  a bodyless subclass header with a prefixed parameter is its own nearest
  nonempty constructor. A header whose parameters are all plain, including a
  rest header, still creates no properties and does not displace a parent's
  nearest nonempty constructor. Typed attributes in class bodies, such as
  `x: int`, are unchanged.
- Every other parameter list keeps its accepted diagnostic and position:
  named functions, methods other than `init`, a `def init` anywhere other
  than directly in a class's indented body, function literals, arrow and
  thin-arrow functions, Python-style lambdas and every bare signature. For
  example `def f(this.x) => 1` still raises
  `SyntaxError("Expected a parameter but found keyword 'this'")`, and
  `def init this.x:` in a class body still raises
  `SyntaxError("Expected '(' but found keyword 'this'")`.
- A `this` or `self` that does not begin a complete prefixed parameter keeps
  that accepted error, placed at the token after the keyword, as in
  `this.a.b`, `this["x"]`, a bare `this`, `this.class`, `this.5` and
  `this.x this.y`.
- Only inputs that were errors at the accepted revision change: they now
  parse, or report a later error. A complete prefixed parameter reports the
  error its plain spelling reports, so `class P(this.x: 5)` raises
  `SyntaxError("Expected a type but found number 5")`, and a statement the
  parser reads tentatively, such as a destructuring declaration, may now
  take the reading that failed before.
- The ten appended entries beginning
  `Nucleoid assigns the properties that a constructor header names with this`
  through `Nucleoid rolls back an instance whose initializing constructor fails`
  specify the executable behavior.

## Class-level state version anchors

- Extend NUC53 property histories inside per-instance class rules, without new
  syntax or a class-wide history. `$Obj.x@first` and `$Obj.x@0` select that
  concrete instance/property slot's first recorded value, not its value when
  a rule was declared or first activated. Constructor parameters do not
  implicitly create fields: store `this.orientation = orientation` explicitly.
- An ordinary class property rule may read its own fixed historical position
  with `@first` or any numeric `@n`. That fixed self-reference is exempt from
  static class self-cycle validation; **Fixed cross-property class anchors**
  below extends the exemption to cycles made only of fixed reads of the
  receiver's other properties. Live reads, moving history reads and class
  cycles through them retain their existing dependency checks. A live
  self-read alongside an anchor still raises
  `TypeError("Circular Dependency")`.
- Class conditionals may tag an orientation-180 receiver and assign its `x`
  and `y` from their own first values minus one. Each receiver, including
  inherited receivers and instances created after declaration, owns independent
  histories. Later guard/input changes recompute from the selected fixed value;
  re-evaluation never changes the meaning of `@first` or cumulatively subtracts.
- A fixed self-anchor whose slot/index has no recorded entry defers that
  assignment: return Null, retain any existing destination value, and do not
  write a placeholder or seed its own history. Other branch statements still
  follow ordinary execution. An actual stored Null is an available entry and
  keeps normal null propagation; it is not an absent history.
- History reads still track the actual property slot and receiver/key reads.
  Conditionals and non-self readers therefore recover after external writes,
  missing guards, slot deletion and recreation. A recreated slot begins again
  at zero. Ordinary instance-property overrides/deletion still replace/remove
  that instance's property rule; this feature does not silently reinstall it.
- `@last` and `@previous` move when history grows. Direct class property rules
  using moving self-history still raise `TypeError("Circular Dependency")`
  before instances exist. Imperative conditional bodies retain accepted
  same-slot moving reads, including helpers and computed receivers: later
  activations use the then-selected value and preserve actual history order.
  Unavailable moving entries keep ordinary deferred-Null assignment behavior,
  not fixed-anchor deferral. Non-self moving readers remain reactive, genuine
  graph cycles remain errors, and ordinary non-class variable/object
  self-assignments retain their accepted behavior.
- `.value` remains an explicit per-instance snapshot: its isolated reads are
  frozen before storing a rule, not turned into live self-feedback.
  A missing `$Empty.x@first.value` keeps
  `ReferenceError("$Empty.x@first is not defined")`.
  Selected destinations remain read-only:
  `TypeError("State versions are read-only")`. Non-slot selections keep
  `TypeError("State versions require a variable or object property")`;
  malformed selectors keep
  `SyntaxError("Expected a non-negative integer, 'first', 'last', or 'previous' after '@'")`.
- Preserve NUC53 recording, slot lifetimes, object identity/list-value semantics,
  rollback of histories with State/graph/class declarations, successful-run
  retention, ordinary class order/overrides, parser/generator forms and limits.
  The eight appended entries beginning `Nucleoid anchors class conditionals to
  each instances first property values` through `Nucleoid preserves moving
  class conditionals beside fixed first anchors` specify this extension.

## Fixed cross-property class anchors

- A class property rule may read another property of its own receiver at a
  fixed history position, written directly on its class: `$C.q@first`,
  `$C.q@n` with a literal non-negative `n`, `$C["q"]@first` or `$C["q"]@n`.
  Such a read is a fixed anchor, and `@0` is `@first`. No syntax is added.
- A class property cycle whose every edge is a fixed anchor, including fixed
  self-anchors, is valid. `$Box.width = $Box.height@first` with
  `$Box.height = $Box.width@first` swaps each instance's first recorded
  values, whether the rules are declared before or after the instances, and
  records one new entry per changed property.
- Such a cycle settles: a recorded position never changes during its slot's
  lifetime, and storing an equal value records no entry and wakes nothing in a
  cascade. The existing propagation limit still applies.
- A cycle with any other edge still raises `TypeError("Circular Dependency")`:
  a live read `$C.q`, a moving `@last` or `@previous` read, a compound
  self-assignment, an index or noun read, or a read not written directly on
  `$C`. A property read both as an anchor and in another way by the same rule
  is an ordinary read. The static class check raises the error before
  instances exist when it sees the cycle.
- Anchors are tracked like other non-self history readers. They wake on writes
  to the slot they read and on its deletion and recreation: a deleted slot
  reads null, and a recreated one is followed from its new first entry.
  Instance overrides and deletion still replace or remove that instance's
  rule.
- A missing entry of another property reads null, as every non-self history
  read does, and that null is an ordinary recorded value. Only a rule's own
  unavailable fixed self-anchor waits.
- Only direct reads on the rule's own receiver are anchors. Reads through
  helpers, `this`, other objects, aliases and populations, and any live or
  moving read of an anchored slot in the same evaluation, including one inside
  a helper, are ordinary dependencies with their existing checks. A cycle
  whose only ordinary read is hidden in a helper fails when an instance
  applies it.
- Propagation order is unchanged: upstream work runs first and ready rules
  keep declaration order. Only when every queued rule waits for another does
  a rule whose waits all run through fixed anchors go next, in declaration
  order. A rule that waits through any other edge, including the structural
  link of a reasoning selection, keeps waiting, so a queue that cannot go on
  still raises `TypeError("Circular Dependency")` as before. `.value` keeps
  its detached snapshot meaning.
- Only programs that raised `TypeError("Circular Dependency")` change. The six
  appended entries beginning `Nucleoid swaps two class properties through
  fixed cross anchors` through `Nucleoid keeps live and moving reads in class
  cycles as circular dependencies` specify this extension.

## Thin arrows and Python-style lambdas

- [NUC 57](docs/nuc-0057.md) adds two optional spellings of the existing
  anonymous and named arrow functions. `=>` stays canonical; every accepted
  form keeps its parsing, AST, canonical rendering and behavior.
- Adjacent `->` is one lexer token and an exact alternative to `=>` wherever
  an arrow is accepted: `value -> expression`, `(left, right) -> expression`,
  typed and literal-default parenthesized parameters, braced statement bodies,
  and every named header, including `def name(params) ->`, bare single,
  typed, multiple, typed/mixed and comma-free parameters, NUC32's indented
  expression body, methods and `init`. Both spellings build the identical
  AST. The native REPL buffers a named header ending in its only arrow of
  either spelling.
- `-` and `>` separated by whitespace remain two tokens. Arrow text inside
  strings, template text outside interpolations, regular expressions and
  comments is unchanged; an interpolation is an expression and accepts either
  arrow. A `/` after `->` starts a regular expression exactly as after `=>`.
- Return-annotation lookalikes are ordinary arrows: `def f() -> int` returns
  the value of `int`, `def f() -> int => 1` returns a function, and
  `def f() -> int:` followed by a body still fails at its colon.
- `lambda: expression`, `lambda name: expression` and
  `lambda first, second: expression` lower to exactly `() => (expression)`,
  `(name) => (expression)` and `(first, second) => (expression)`: untyped
  parameters and an expression body extended exactly like an anonymous
  arrow's. A leading brace in the body is an object literal. Annotations,
  defaults, trailing commas, parenthesized or starred parameters and statement
  bodies are not part of this spelling; the parenthesized arrow has them.
- `lambda` is not reserved. It forms a lambda only at an operand position,
  when a complete non-consuming scan finds nothing or comma-separated
  ordinary names and then a colon. It remains an ordinary identifier,
  keeping its accepted reading, at the top level of `if`, `else`, `elif` and
  `for` headers, subscript starts (the index or slice start inside `[...]`)
  and template interpolations; as a single
  parameter in an object-literal value, where `{first: lambda next: 2}` keeps
  NUC44's two comma-free properties; and as a single parameter whose colon
  starts a statement. Statement loops accept a statement directly after the
  expression that `lambda` ends, and exactly two statements begin with a name
  and a colon: a typed declaration, whose colon is followed by one name and
  a newline, semicolon, dedent or end of input, and NUC45's `assert:`
  statement. So `f = lambda x: int` stays an assignment followed by a typed
  declaration, and `x = lambda assert: x, 1` stays an assignment followed by
  an assertion statement. A statement-leading `lambda: Type` remains a
  declaration. Parentheses or call arguments select the lambda in each of
  these places, except that a single `assert` parameter never qualifies;
  `lambda assert, other: other` is still a lambda.
- Variables, object keys, shorthand, members, labels, parameters, annotations,
  catch and loop names, and arrow parameters named `lambda`, such as
  `lambda -> lambda * 2`, are unchanged.
- No new error type, message format, public AST node or runtime path is
  added. Diagnostics name the arrow as written, such as
  `SyntaxError("Expected '(' but found '->'")` for `def f -> 1`. The five
  appended entries beginning `Nucleoid accepts thin arrows wherever fat arrows
  are accepted` through `Nucleoid rolls back failed lambda and thin arrow
  updates` specify the executable behavior.

## List comprehensions

- [NUC 92](docs/nuc-0092.md) adds Python's list comprehensions. A list literal
  whose whole content is `EXPR for NAME in SOURCE` or `EXPR for NAME of SOURCE`,
  optionally followed by one `if COND`, is a comprehension. No token, keyword,
  syntax tree node or runtime path is added.
- `[EXPR for NAME in SOURCE]` is exactly `SOURCE.map(NAME => EXPR)`, and
  `[EXPR for NAME in SOURCE if COND]` is exactly
  `SOURCE.filter(NAME => COND).map(NAME => EXPR)`; `of` is the same as `in`.
  The parser builds the trees those arrow spellings build, so evaluation,
  errors, graph keys, liveness, `why`, `affects`, transactions and canonical
  rendering are theirs, and canonical source writes
  `SOURCE.filter(function()).map(function())`.
- SOURCE is evaluated once, as the receiver: a list or a class's population.
  The comprehension stays live on SOURCE, on the instances and properties that
  EXPR and COND read and on the variables they read. NAME is local to the
  comprehension, so a variable of the same name keeps its value.
- NAME is one plain identifier. EXPR is a full expression, including a
  Python-style conditional, `?:`, a lambda or another comprehension. SOURCE
  ends before an `if` at its top level, so a Python-style conditional as
  SOURCE needs parentheses, while `?:` and calls need none. COND is a full
  expression up to `]`.
- A comprehension has exactly one `for` clause and at most one `if` clause,
  with no other items and no trailing comma. Line breaks and indentation
  inside the brackets carry no layout, as in every list literal.
- A comprehension may stand wherever a list literal may: as a value, an
  argument, a return value, an object value, inside a template interpolation
  or another list, in a condition and at the start of a statement. Index and
  slice brackets, parenthesized generators such as `(x for x in xs)`, bare
  call arguments such as `f(x for x in xs)` and dict and set comprehensions
  keep their accepted errors.
- The limits of lambdas are inherited. A lambda reads variables and its own
  parameter but not the parameters of an enclosing function or lambda, so a
  comprehension in a function reads the function's parameters only in SOURCE,
  and a nested comprehension reads only its own names: reading `row` inside
  `[[cell + row.length for cell in row] for row in grid]` raises
  `ReferenceError("row is not defined")`. `this` in a method's comprehension
  raises `ReferenceError("this is not defined")`, and a string has no `map`,
  so `[c for c in "abc"]` raises `TypeError("abc.map is not a function")`.
- A NAME that is not an identifier raises
  `SyntaxError("Expected a name but found <token>")`, and a missing `in` or
  `of` raises `SyntaxError("Expected 'in' or 'of' but found <token>")`, both
  at that token. A tuple target such as `[a for a, b in pairs]` reports its
  comma that way. After SOURCE or COND, any token other than `]` raises the
  existing `SyntaxError("Expected ']' but found <token>")`, so a second `for`
  or `if` clause, a trailing comma and an `else` after COND report there.
- Nested comprehensions raise
  `SyntaxError("Expressions are nested too deeply")` where nested list
  literals do. A comprehension as an assignment target raises
  `SyntaxError("Cannot assign to <rendering>")`, such as
  `Cannot assign to xs.map(function())`, when the statement runs, as its
  arrow spelling does.
- The line-oriented REPL keeps its buffering rules: a comprehension on one
  line, or inside a header that it buffers, runs, and one split over lines
  fails at its first line, as a list literal split over lines does.
- Only former syntax errors change. A program, an expression or a template
  interpolation is parsed exactly as the accepted parser parses it first;
  only when that fails and its tokens include `for` or a template is it parsed
  again with comprehensions. So every input the accepted parser accepts keeps
  its parse, including the ignored tokens after an interpolation's
  expression. When the comprehension parse fails too, its error is
  reported. The two parses agree until a list literal's first item is
  followed by `for`, so the error differs from the accepted parser's only for
  such a source.
- The eight appended entries beginning
  `Nucleoid builds lists with comprehensions` through
  `Nucleoid keeps comprehensions identical to their map and filter spellings`
  specify the executable behavior.

## Property selector arguments

- [NUC 79](docs/nuc-0079.md) lets a call argument written `.name`, or a
  dotted path such as `.a.b`, stand for the one-parameter function that reads
  that path from its argument: `Bid.max(.amount)` means
  `Bid.max(bid -> bid.amount)`. No token, keyword, syntax tree node or error
  type is added.
- A selector is a whole argument of a parenthesized argument list: one or more
  `.NAME` pairs directly followed by `,` or `)`. The argument lists are those
  of calls, method calls, constructor calls, calls written with `new` and
  `super(...)`, at any argument position. Inside the parentheses line breaks,
  indentation and comments carry no layout, so `. a`, `.a .b` and a selector
  on its own line are selectors, and one trailing comma may follow.
- `NAME` is any identifier or keyword that member access accepts after `.`.
  A keyword reads and renders its canonical spelling, as in member access:
  `.self` reads `this`, `.elif` reads `else`, and `.None` and `.undefined`
  read `null`. A digit after `.` is a number token, so `.5` stays `0.5`.
- A selector is exactly the anonymous function `v -> v.a.b` whose parameter no
  source can name, so it reads only its argument. Called with an argument it
  returns what that member path returns; a missing property, or one read from
  `null` or from a value without properties, reads undefined, and strings,
  lists and classes answer `.length`. A missing argument is `null`, and
  arguments after the first are ignored, so `reduce` reads the accumulator's
  property and a `sort` comparator reads the item being placed. `typeof` gives
  `"Function"` and the value displays as `[function]`.
- Reads follow member access. A declaration that reads through a `null` value
  settles to `null`, and `.value` stays the snapshot read, so
  `[{value: 3}].map(.value)` yields the record. A call that receives a
  selector depends on whatever it reads for each item, so declarations,
  class-level rules, methods, lambda bodies, headers, conditional expressions,
  template interpolations and `suppose` blocks stay live and roll back as with
  the lambda.
- A selector renders as its written path without spaces, such as `.to.city`,
  in graph and block keys, `why` and `affects` rules and errors that quote an
  expression, so `why` explains `top = Bid.max(.amount)`. Anonymous lambdas
  keep rendering as `function()`, so a selector and its lambda spelling are
  different graph keys, and selectors of different paths stay different keys.
- Every changed input was a syntax error: `SyntaxError("Unexpected '.'")`
  at the selector, or the fallback error of a tentatively parsed statement
  whose value held one, as `first, second = xs.map(.a)` raised
  `SyntaxError("Unexpected ','")` and now destructures. An argument that
  starts with `.` but is not a complete selector keeps that error and its
  position, as in `f(.a + 1)`, `f(.a())`, `f(.)`, `f(..a)`, `f(...xs)`,
  `f(.a[0])` and `f(.a ?? 1)`. A complete selector earlier in the same
  argument list is a valid argument, so the later error is reported where it
  occurs: `f(.a, .b + 1)` reports `.b`, and `f(.a,,)` raises
  `SyntaxError("Unexpected ','")`. Outside parenthesized argument lists
  nothing changes: `x = .a`, `[.a]`, `xs[.a]`, `x -> .a`, `assert: .a, 1`,
  `x = a?.b` and `x = a?? .b` keep their errors, and a line that starts with
  `.` still raises its accepted error.
- Building a selector passes the nesting guard of any argument, so nested
  calls raise `SyntaxError("Expressions are nested too deeply")` where they
  raised it before. A selector names at most 127 properties; the 128th
  `.NAME` pair raises `SyntaxError("Expression has too many parts")` at its
  `.`. A `.value` read of an undefined value inside a selector raises
  `ReferenceError("<path> is not defined")` with the written path before
  `.value`, such as `.a`, and `ReferenceError(".value is not defined")` when
  the selector's own argument is undefined.
- The eight appended entries beginning
  `Nucleoid reads list item properties with selector arguments` through
  `Nucleoid tracks only the properties that selector arguments read` specify
  the executable behavior.

## Implicit receiver properties

- [NUC 102](docs/nuc-0102.md) lets the value of an assignment to a property
  read the other properties of the same object with a leading dot:
  `$Order.total = .price * .qty` is
  `$Order.total = $Order.price * $Order.qty`. No token, keyword, syntax tree
  node, error type or message is added.
- **Receivers.** The assignment forms are `=`, `:=`, `+=`, `-=`, `*=`,
  `**=`, `/=`, `%=`, `&&=` and `||=`. A target that is a plain path whose
  last link is a property, `RECEIVER.NAME`, rooted in a name, a class
  reference `$Class` or `every Class`, `this` or `self`, gives its value the
  receiver `RECEIVER`, the path without its last property. These are the
  paths that compound assignments accept, apart from a bare name. So
  `order.customer.label = .first` reads `order.customer.first`. A variable,
  a destructuring pattern, a path that ends in a state version and a
  computed target such as `xs[0].p` or `f().p` give no receiver.
- **Receiver reads.** In the value of an assignment with a receiver, a `.`
  that begins an operand and is directly followed by a name reads that
  property of the receiver. The name is read as member access reads it: an
  identifier, or a keyword in its canonical spelling, so `.self` reads
  `this` and `.None` reads `null`. The operand continues with every suffix
  that member access takes: `.a.b`, `.a[0]`, `.a[1:]`, `.a()` and
  `.a@first` read `RECEIVER.a.b`, `RECEIVER.a[0]`, `RECEIVER.a[1:]`,
  `RECEIVER.a()` and `RECEIVER.a@first`.
- A dot begins an operand only after a token in operand position, which the
  lexer also uses to tell a regular expression from a division: not after a
  name, a class reference, a literal, `)`, `]`, `}` or `this`. After such a
  token the dot keeps its accepted reading: `x .a` and `1 .a` are member
  access, as are `x` and `.a` on the next line inside brackets, while
  `(x is .a)` and `while .a: ...` keep their errors. A digit after `.` is a
  number, so `.5` stays `0.5`.
- So the comparison words differ: `is` is a name and `not` a keyword, and
  in the value of `o.p = ...`, `(1 is .a)` keeps its error while
  `(1 is not .a)` reads `o.a`, as `(1 is not o.a)`. Without grouping,
  `o.p = 1 is not .a` is the three statements `o.p = 1`, `is` and
  `not .a`, whose dot has no receiver and keeps its error.
- **The innermost assignment.** A dot reads the receiver of the innermost
  assignment whose value contains it, through every construct of that
  value: operands of operators, `?:` and Python-style conditionals with
  their conditions, lists, objects, subscripts, quantifiers, lambda, arrow
  and function bodies, the `if`, `for` and `while` headers and `suppose`
  premises in those bodies, and template interpolations. So
  `o.p = .a if .b else .c` reads all three. An assignment nested in the
  value has a receiver of its own: in `o.p = (q.r = .a)` the dot reads
  `q.a`, and in `o.p = (x = .a)` it keeps its error, as do the values of
  variables and destructuring declarations, whose targets are names. A
  property assignment nested in such a value reads its own receiver, so
  `{a} = (q.r = .x)` reads `q.x`.
- Lambda and function bodies read the receiver lexically. A read is
  evaluated where its written-out path would be, with the accepted scoping
  rules, so when a constructor holds `this.xs = [1, 2].map(i -> i * .w)`,
  creating an instance raises `ReferenceError("this is not defined")`, as
  `this.w` does in that lambda, while
  `$Cell.taller = Cell.filter(c -> c.h > .h).length` compares every cell
  with the instance that the rule computes.
- **Kept readings.** These dots keep their accepted readings and errors,
  although they begin an operand:
  - the first token of a call argument, which is a selector of
    [Property selector arguments](#property-selector-arguments) when the
    argument is exactly a property path, and otherwise its accepted error:
    `$O.p = f(.a)` passes `v -> v.a`, and `$O.p = f(.a + 1)` keeps its
    error. The first tokens of the two arguments of an `assert:` statement
    follow the same rule. A dot later in an argument reads the receiver, as
    in `f(1 + .a)`, `f(-.a)` and `f(x -> x + .a)`;
  - every dot in the value of an `is` assignment of
    [Is assignments](#is-assignments), including the values of assignments
    nested in it: `$Obj.h is .h * 2` and `o.p is .a` still run as two
    statements, and `$Obj.d is 2 * .h` keeps its error;
  - a dot that begins a statement or the target of an assignment, as in
    `.a = 1`, `.a.push(1)` in a function body and `o.p = (.a = 1)`;
  - every position outside the value of an assignment with a receiver:
    statements, conditions and headers, `return`, `throw` and `assert` at
    the top level, the values of variables, destructuring declarations and
    label arguments such as `P(n = .a)`, and `parse_expression` of an input
    without a property assignment.
- **The written-out path.** A read builds the member expression over a copy
  of the receiver path, the tree of `RECEIVER.NAME` written out. Evaluation,
  dependencies, class-level rules, `.value` snapshots, state versions,
  `why`, `affects`, rendering, graph and block keys, rollback and run-time
  errors are those of that path. Rendering shows the expanded path, as a
  compound assignment renders its lowering: `why` reports
  `$Order.total = $Order.price*$Order.qty`, and
  `$Cart.total = .total + 1` raises `TypeError("Circular Dependency")`.
- **Limits.** A read and its suffixes form one member chain, as
  `(RECEIVER).NAME` would, also after `new`, so the 128th link after the
  receiver raises `SyntaxError("Expression has too many parts")`. A target of more than 128
  links gives a receiver that no read may use: a dot that would read it
  raises `SyntaxError("Expression has too many parts")` at the dot.
  Expression and statement nesting keep their accepted limits.
- **Parse passes.** Every input first runs the accepted passes exactly as
  before, with no receiver reads: the plain parse, the parse with `is`
  comparisons when the tokens hold `is` or a template, and the parses with
  comprehensions of [List comprehensions](#list-comprehensions) when they
  hold `for` or a template. Only when all of them fail, and the tokens hold
  a `.` directly followed by a name after a token in operand position or a
  template, one final parse reads receivers, with `is` comparisons when the
  tokens hold `is` or a template and with comprehensions when they hold
  `for` or a template. An input is parsed at most five times, and at most
  three times when it holds neither `for` nor a template.
- When the final parse fails too, the input reports whichever of the
  accepted error and the final parse's error lies further into the source,
  by line and column, the accepted one on a tie. So `o.p = .a +` reports
  the missing operand at its end, and `x = .a` keeps its accepted error.
- **Compatibility.** Only inputs that were errors at the accepted revision
  change. Every input that an accepted pass parses, including
  `parse_expression` inputs and template interpolations whose trailing
  tokens are ignored, gets that pass's result before the final parse can
  run.
- The eight entries beginning
  `Nucleoid reads receiver properties in class level rules` through
  `Nucleoid rolls back failed receiver assignments` specify the executable
  behavior. `tests/implicit_receivers.rs` executes the rest.

## In-place list sorting

- [NUC 58](docs/nuc-0058.md) makes `list.sort()` and `list.sort(comparator)`
  reorder a writable receiver in place, as JavaScript's `Array.prototype.sort`
  does, and return the sorted list. This deliberately changes the accepted
  meaning of `sort` on a writable receiver, which returned a sorted copy and
  left the receiver unchanged. Parsing and every other list method are
  unchanged.
- Ordering is unchanged. A function argument is a comparator: items are placed
  in input order, each before the first already placed item for which
  `comparator(item, placed)` converts to a number below zero, so equal items
  keep their order. Without a function argument, including a non-function
  argument, items order stably by their string forms: `[10, 9, 1]` sorts to
  `[1, 10, 9]` and `["b", "a", "C"]` to `["C", "a", "b"]`.
- The receiver is read once, before the arguments are evaluated, and the
  complete order is computed before anything is written. A comparator error
  leaves the receiver unchanged and propagates through the existing
  transaction like any other error.
- Writable receivers are exactly those `push`, `pop` and `shift` write back
  to: variables, parameter and loop bindings, members of objects, including
  members reached through `this`, `self`, a class-level `$Class` receiver or
  a call result, and properties of an object selected through a state
  version, by name or by index. A member's object is evaluated once, so the
  write never evaluates the receiver again.
- A variable or property write appends a version only when the order changes,
  and re-runs the receiver's dependents. An equal list written outside a
  propagation cascade still re-runs them, like any explicit write; inside a
  cascade it wakes nothing.
- Receivers without a writable list place keep returning the sorted list and
  write nothing, exactly as before: list literals, call, operator and
  pipeline results such as `scores.slice().sort()`, other indexes such as
  `grid[0]` and `record["items"]`, `.value` snapshots, selected state versions
  such as `scores@first`, and class populations such as
  `Item.sort(comparator)`, which keep their creation order. `push`, `pop` and
  `shift` still write nothing through those indexes, and still raise
  `TypeError("State versions are read-only")` on a selected version; `sort`
  never writes into history.
- Lists are values. Sorting a parameter or loop binding reorders that binding
  only; the list it was copied from is unchanged.
- A `sort` in a declaration is a standing sort: `ranked = scores.sort(comparator)`
  sorts `scores` in place and re-runs whenever `scores`, the comparator or
  anything the comparator reads changes, including a reassignment of
  `scores`, so `scores` stays sorted. A statement never re-runs for its own
  write, so re-sorting does not loop, whatever the comparator returns.
  `scores = scores.sort()` reads its own target, so it sorts once.
- Two standing sorts of one list with different orders re-run each other. A
  later change to that list raises `TypeError("Propagation did not settle")`
  and rolls back.
- Replacing a standing sort keeps the accepted replacement order of
  declarations: the replaced rule is still filed while the replacement's
  right-hand side runs, so the replacement's write re-runs it once and the
  list keeps the replaced order until it next changes, exactly as a replaced
  `count = items.push(item)` re-runs once.
- No syntax, keyword, error type or message is added. The seven appended
  entries beginning `Nucleoid sorts a list in place like JavaScript` through
  `Nucleoid rolls back failed in place sorts` specify the executable behavior.

## Conditional expressions

- [NUC 59](docs/nuc-0059.md) adds conditional expressions in two optional
  spellings that build one expression: `condition ? consequent : alternate`
  and `consequent if condition else alternate`. Statement-level `if`, `else`
  and `elif` rules keep their grammar and behavior.
- The condition is evaluated once and tested with the existing truthiness of
  `if`, `not`, `&&` and `||`: null, an undefined property, `false`, `0`, NaN
  and the empty string are false; everything else, including empty lists and
  objects, is true. Exactly one branch is then evaluated and its value is the
  result. The unselected branch is not evaluated: it has no effects, raises
  nothing and adds no dependency.
- Reads follow the short-circuit reads of `&&` and `||`. A standing
  declaration depends on what its condition and its selected branch read,
  re-evaluates when either changes and switches branches when the condition
  changes, replacing obsolete dependencies. Variables, properties, class-level
  rules, function bodies and returns, arrow and lambda callbacks, list and
  object elements, template interpolations, call arguments and subscripts use
  a conditional as an ordinary operand.
- Existing assignment rules are unchanged. A stored declaration whose
  evaluation read a null binding or property, including in its condition,
  settles to `null`, as `x = flag || 2` does. A missing property read by the
  condition is false and stays subscribed, an undefined selected branch stores
  `null` until it is defined, and reading a deleted variable during its
  deletion's propagation defers to `null`. `.value` snapshots are taken in
  every operand when the declaration is stored, as in a skipped logical
  operand, so an unselected branch keeps the snapshot it had then.
- Class-level rule validation inspects every operand, as for `&&` and `||`:
  an undefined name in either branch raises
  `ReferenceError("<name> is not defined")` when the rule is declared, and a
  `$Class` property read in either branch takes part in
  `TypeError("Circular Dependency")` checks. Each instance evaluates only its
  selected branch. Unlike a class-level `if` statement, which stands aside
  while its condition reads something undefined, a conditional expression
  treats that condition as false and selects the alternate.
- In the value of an `is` assignment, a branch that is exactly one
  identifier token is a word instead, as
  [Conditional words](#conditional-words) describes, which class-level rule
  validation does not require to be defined.
- An error raised by the condition or the selected branch propagates through
  the existing transaction, so a failed standing re-evaluation rolls back the
  triggering change and keeps the earlier branch selection.
- Both spellings share one right-associative precedence level above
  assignment and below the reasoning pipeline `|>`. `x = c ? a : b` assigns
  the result, `a || b ? c : d` tests `a || b`, `x |> why ? a : b` tests the
  pipeline, and `a ? b : c ? d : e`, `a if b else c if d else e` and mixed
  chains group to the right. The consequent after `?` is a complete
  expression. The operand before `?` and both operands before `else` are
  coalescing-level, the pipeline level extended by NUC 60's `??`, so a nested
  conditional there needs parentheses; either alternate is conditional-level. Arrow and lambda bodies extend over a
  conditional, as in `items.map(x => x > 2 ? "big" : "small")`.
- `?` is one lexer token. `?.` and `?:` are not operators, `??` is
  [NUC 60](docs/nuc-0060.md)'s nullish coalescing operator, and a `/` after
  `?` starts a regular expression. `?` inside strings, template text
  outside interpolations, regular expressions and comments is unchanged. In
  direct-alternative and native REPL header scans, a top-level conditional's
  colon pairs with its `?`.
- The Python spelling qualifies only when `if` follows a complete operand and
  one condition, parsed like an `if` header's up to the coalescing level, is
  immediately followed by `else` or its existing lexical alias `elif`.
  Otherwise it consumes nothing, and `if` keeps its accepted reading as an
  adjacent statement with its original diagnostics: `x = 1 if ready:` with an
  indented suite, `z = 3 if lit: w = 4` and colon-free NUC49 headers are
  unchanged. An attempt is decided by its `if` position and nesting depth and
  reused when the same source is parsed again.
- `lambda` stays an ordinary name at the top level of a `?` consequent and of
  a Python condition, whose following `:` or `else` belongs to the
  conditional; parenthesize a lambda there. At the top level of a template
  interpolation, whose trailing tokens are ignored, the Python spelling is not
  recognized, so `` `${a if b else c}` `` still renders `a`; use the `?`
  spelling or parentheses, as in `` `${(a if b else c)}` ``.
- Both spellings render canonically as `(condition?consequent:alternate)`.
  The grouping keeps a conditional operand distinct from its enclosing
  operator in graph keys.
- Malformed forms use existing diagnostics, for example
  `SyntaxError("Expected ':' but found end of input")` for `x = c ? 1`,
  `SyntaxError("Unexpected '?'")` for `x = c ? ? 1 : 2` and
  `SyntaxError("Unexpected end of input")` for `x = 1 if c else`. The six
  appended entries beginning `Nucleoid selects values with conditional
  expressions` through `Nucleoid rolls back failed conditional expression
  updates` specify the executable behavior.

## Nullish coalescing

- [NUC 60](docs/nuc-0060.md) adds the nullish coalescing operator
  `left ?? right`. It yields `left` unless `left` is null, and otherwise
  evaluates and yields `right`. `&&`, `||` and conditional expressions keep
  their grammar and behavior.
- `left` is evaluated once and settled by the existing assignment rules: it is
  null when its value is null, when its value is undefined, as for a missing
  property, or when its evaluation read a null binding or property, exactly
  what a declaration of `left` alone would store. So `(n + 1) ?? 0` is `0`
  while `n` is null. Falsy values that are not null, such as `0`, `""`,
  `false`, NaN and empty lists and objects, are kept, unlike with `||`.
- When `left` is not null, `right` is not evaluated: it has no effects, raises
  nothing and adds no dependency. When `left` is null, the null and undefined
  reads that `left` made are discarded and `right` is evaluated. Its reads
  count as usual, so `a ?? b` with both null still stores `null`.
- A standing declaration depends on what `left` read and, when it was
  evaluated, on what `right` read. It re-evaluates when either changes and
  switches between the left value and the default as `left` becomes null or
  not, replacing obsolete dependencies. Variables, properties, class-level
  rules, function bodies and returns, arrow and lambda callbacks, list and
  object elements, template interpolations, call arguments, subscripts and
  conditional-expression operands use `??` as an ordinary operand.
- An undefined name is an error, not null: `missing ?? 5` raises
  `ReferenceError("missing is not defined")`. A variable read while its
  deletion propagates is undefined, so its dependents take their defaults and
  follow it again once it is defined. `.value` snapshots are taken in both
  operands when the declaration is stored, as in a skipped logical operand, so
  an unevaluated `right` keeps the snapshot it had then.
- Class-level rule validation inspects both operands, as for `||`: an
  undefined name in either raises `ReferenceError("<name> is not defined")`
  when the rule is declared, and a `$Class` property read in either takes part
  in `TypeError("Circular Dependency")` checks. Each instance evaluates its
  own operands. A class-level `if` statement still stands aside while its
  condition reads something undefined, but an undefined read that `??`
  replaced with its default does not count, so `if $Order.rush ?? false:`
  decides for an order without `rush`.
- An error raised by `left` or an evaluated `right` propagates through the
  existing transaction, so a failed standing re-evaluation rolls back the
  change that triggered it.
- `??` is one right-associative level between the conditional expression and
  the reasoning pipeline `|>`. It binds more loosely than `||`, `&&`,
  comparisons and arithmetic, as in C#: `a || b ?? c` is `(a || b) ?? c`,
  `a ?? b || c` is `a ?? (b || c)`, `a ?? b == c` is `a ?? (b == c)` and
  `a ?? b ?? c` is `a ?? (b ?? c)`, so write `(x ?? 0) > 5` to compare a
  default. `x |> why ?? b` defaults the pipeline. `a ?? b ? c : d` tests
  `a ?? b`, a `?` consequent and either alternate may contain `??`, and the
  operand before `?`, the Python consequent and the Python condition are
  coalescing-level. One chain holds at most 128 `??` operators.
- `??` is one lexer token. `? ?` stays two tokens and `???` is `??` followed by
  `?`. A `/` after `??` starts a regular expression, and `a??.5` reads `.5` as
  a number. `??` inside strings, template text outside interpolations,
  regular expressions and comments is unchanged. In direct-alternative and
  native REPL header scans, `??` pairs with no colon.
- It renders canonically as `(left??right)`. The grouping keeps
  `(x ?? 0) > 5` distinct from `x ?? 0 > 5` in graph keys.
- `??=`, optional chaining `?.` and Kotlin's Elvis `?:` are not added, and
  statement-level `if` and the null-read rule outside `??` are unchanged.
  Malformed forms use existing diagnostics, for example
  `SyntaxError("Unexpected ':'")` for `x = c ?? 1 : 2` and
  `SyntaxError("Unexpected '='")` for `x ??= 1`. The six appended entries
  beginning `Nucleoid defaults unknown values with nullish coalescing` through
  `Nucleoid rolls back failed nullish coalescing updates` specify the
  executable behavior.

## Is and is not comparisons

- [NUC 86](docs/nuc-0086.md) adds `is` and `is not` as spellings of `==` and
  `!=`. They compare numbers, strings, booleans, null, lists, objects and
  property reads with the loose equality, precedence and left associativity
  of `==` and `!=`, so `count is 3`, `order.status is "SHIPPED"`,
  `ready is true` and `a.x is b.x` mean `count == 3`,
  `order.status == "SHIPPED"`, `ready == true` and `a.x == b.x`. No token or
  keyword is added: `is` stays an ordinary identifier, and `is not` is `is`
  followed by the existing keyword `not`.
- A quoted right operand is an ordinary string in either quote, so
  `value is "EXACT"` and `value is 'EXACT'` are `value == "EXACT"`.
- When the whole right operand is one bare identifier, it is a word.
  `value is EXACT` reads `EXACT` as any name is read when a local, variable,
  class, function or built-in global of that name exists, and otherwise
  compares with the string of its spelling, `"EXACT"`. Keywords such as
  `true`, `false`, `null`, `None`, `undefined`, `this` and `self`, `$Class`
  references, paths such as `EXACT.x`, calls, subscripts, versions, operator
  expressions and parenthesized operands are not words and read as they do
  with `==`: `value is (EXACT)` raises `ReferenceError("EXACT is not defined")`
  while nothing defines `EXACT`. The left operand is never a word.
- The right operand may hold the `@name` of [At variables](#at-variables),
  which reads the variable and is never a word: `value is @EXACT` means
  `value == EXACT`.
- A word records a dependency on the variable of its name even while nothing
  defines it, so a standing declaration, `if` rule or class-level rule that
  compares with it re-evaluates when that variable is defined, changed or
  deleted, and deleting it returns the word to its text. A `while` condition
  or function body reads the word each time it runs. While undefined, the
  name is a pending graph node, as any name read before it is defined: `why`
  lists it among the sources, and `affects` of the pending name lists nothing
  until it is defined. A class-level rule does not require a word to be
  defined when it is declared.
- `is` is an operator only directly after a complete operand in an enclosed
  position, where the accepted grammar cannot continue with an identifier:
  inside parentheses, call arguments, lists, object-literal values and
  spreads, subscripts and slice bounds; in the condition of `if`, `elif`,
  `else if`, a direct alternative or `while`, a `for` source, a quantifier
  collection, the consequent after `?` and a Python-style condition; in the
  first operand of an `assert:` statement; and in an arrow, lambda or
  quantifier body or a conditional alternate that sits in one of these
  positions. In an object value, `is` directly followed by `:` stays the next
  comma-free property key.
- Everywhere else, apart from the statements of
  [Is assignments](#is-assignments) and the predicate sentences of
  [Every quantifiers, class references and predicate sentences](#every-quantifiers-class-references-and-predicate-sentences),
  `is` keeps its accepted reading as a
  name that begins the next juxtaposed statement: expression statements,
  assignment, destructuring
  and `:=` values, `return`, `throw` and `delete` operands, the operand before
  `?` and
  both operands around a Python-style condition at statement level, arrow,
  lambda and quantifier bodies at statement level, a named arrow body on its
  own line or the same line, which means exactly its `return` form, the
  second operand of `assert:` and the top level of a template interpolation.
  So `flag = value is "EXACT"` is still `flag = value`, `is` and `"EXACT"`;
  write `flag = (value is "EXACT")`.
- A program that parses at the accepted revision keeps that parse; the
  operator applies only when the whole program would otherwise fail to parse.
  Even then, a `while` header or a `suppose` premise that also parses through
  its colon as separate statements keeps that reading, so
  `while f() is x: T` stays `while`, `f()`, `is` and the declaration `x: T`.
- `is` and `is not` share one left-associative level with `==`, `!=`, `===`
  and `!==`, binding more tightly than `and` and `&&` and more loosely than
  `<`, `>`, `<=` and `>=`: `a is b == c` is `(a is b) == c`,
  `a is b < c` is `a is (b < c)`, and
  `not a is b` is `(not a) is b`, as `not a == b` is. The right operand of
  `is not` starts after `not`, so `value is not EXACT` applies the word rule
  to `EXACT`, and `a is not not b` is `a != (not b)`. One chain holds at most
  128 of these operators, as before.
- A word comparison renders as `(left is word)` or `(left is not word)`.
  Every other `is` renders as `==` or `!=`, so `if value is "EXACT":` and
  `if value == "EXACT":` are one standing rule, while `if value is EXACT:` is
  a different one.
- Malformed forms raise the diagnostics of their `==` and `!=` spellings in
  the same places, for example `SyntaxError("Unexpected ':'")` at line 1,
  column 12 for `if value is:` followed by an indented body, and
  `SyntaxError("Unexpected end of input")` at line 1, column 12 for
  `if value is`.
- A statement that begins with an assignable path directly followed by `is`
  and a value is an assignment under [Is assignments](#is-assignments), so
  `value is "EXACT"` assigns and `while v is x: T` is a loop; every enclosed
  comparison above keeps its reading.
- The five appended entries beginning
  `Nucleoid matches values with is and is not` through
  `Nucleoid keeps is as an ordinary name where a statement can follow`
  specify the executable behavior.

## Is assignments

- The statement `target is value` means `target = value`, for variables and
  properties. [NUC 93](docs/nuc-0093.md) introduced it behind an
  `--is-assignment` flag and a `SyntaxOptions` field; both were removed, and
  every source reads it. So `a is good` assigns, where
  [NUC 86](docs/nuc-0086.md) read the three juxtaposed statements `a`, `is`
  and `good`.
- It combines with C-style comments. `run`, `check`, `parse_program`, `parse`
  and `statement::compile` read it, and `clear()` and `.clear` keep reading
  it. File runs, the plain and redirected REPL, the terminal workspace,
  `--json` and `--graph` read it; REPL continuation and terminal indentation
  are lexical and unchanged.
- A statement is `target is value` when its leading
  expression is a target that `:=` accepts and the next two tokens are the
  identifier `is` and a token that can begin an expression. Such a target is
  a name or a dotted path rooted in a name, `this`, `self` or `$Class`, as for
  compound assignment, possibly grouped in parentheses, with at most 128
  links; a root `this`, `self` or `$Class` needs a property. A link may be a
  state version, so
  `a@first is 1` raises `TypeError("State versions are read-only")` as
  `a@first = 1` does. Nothing may separate the target from `is` but spaces
  or comments on the same line.
- A token can begin an expression when it is an identifier, a `$Class`
  reference, a number, a string in either quote, a template, a regular
  expression, `(`, `[`, `{`, `!`, `-`, `+`, or one of the keywords `true`,
  `false`, `null` (also spelled `None` and `undefined`), `this` (also `self`),
  `super`, `new`, `delete`, `function`, `not` and `typeof`. Any other token
  after `is` keeps the accepted reading, in which `is` is a name that begins
  the next statement: `:`, `=`, `:=`, a compound assignment such as `+=`,
  another operator or punctuation, a keyword such as `if`, `and` or `of`, or
  the end of the line or input. So `a is: T`, `a is = 5`, `a is += 1` and
  `a is` at the end of a line read as before. An `@` directly followed by a
  name other than `first`, `last` and `previous` also begins the value, as
  [At variables](#at-variables) describes; any other `@`, as in
  `a is @first`, keeps the accepted reading.
- A leading expression that `:=` rejects also keeps the accepted reading:
  `f() is 5`, `1 is 2`, `a[0] is 1`, `this is 5`, `$Order is 5`,
  `a + b is 5`, `a = b is c` and `x => x is 5` are juxtaposed statements as
  before, and destructuring declarations are unchanged. The one exception is
  a class reference without a property followed by `is` and one name, the
  predicate sentence of
  [Every quantifiers, class references and predicate sentences](#every-quantifiers-class-references-and-predicate-sentences):
  `$Human is mortal` states `$Human.mortal = true`. A statement that
  begins with `why`, `affects`, `stated` or `derived` followed by `is` keeps
  its accepted reading as a reasoning selection of the name `is`, so those
  variables are assigned with `=`.
- The lexer is unchanged, so a `/` directly after `is` is division, as after
  any name. A regular-expression value is written in parentheses:
  `pattern is (/colou?r/)`.
- The form applies in every statement position: the top level, after another
  statement on the same line, indented, same-line and braced blocks, the
  bodies of `if`, `elif`, `else`, `for`, `while`, `try`, `catch` and
  `suppose`, function, method, `init` and constructor bodies, one-statement
  class bodies, and the block bodies of function literals wherever they
  appear, including inside template interpolations. It never applies inside
  an expression: conditions, parentheses, arguments, lists, object values,
  named arrow bodies, the top level of a template interpolation and the top
  level of `parse_expression` keep their readings, including the
  enclosed `is` and `is not` comparisons of
  [Is and is not comparisons](#is-and-is-not-comparisons).
- Unless its value is a word or holds the word branches of
  [Conditional words](#conditional-words) or the item words of
  [Item words](#item-words), `target is value` builds exactly
  the syntax tree of `target = value`. The value is parsed as the value of `=` is at
  statement level, in the same context and at the same nesting depth, and
  ends where that value ends. So
  standing declarations and their dependencies, property assignments,
  `this.p` in constructors, class-level `$Class.p` rules for every instance,
  chains such as `a is b = c`, values such as `a is b += 1`, `a is suppose:`
  with a body, runtime errors and syntax errors all follow from `=`.
  `a is not b` assigns `not b`, and `a is b is c` is `a is b` followed by the
  juxtaposed statements `is` and `c`, as `a = b is c` is `a = b`, `is` and
  `c`.
- When the value is exactly one identifier token, it is a word, with the
  word rule of `is` comparisons. It reads the local, variable, class,
  function or built-in global of that name when one exists, so after
  `good = 2`, `a is good` makes `a` follow `good`. Otherwise its value is the
  string of its spelling, so `a is good` stores `"good"`.
- A word records a dependency on the variable of its name even while nothing
  defines it. The declaration re-evaluates when that variable is defined,
  changed or deleted, including inside a `suppose` block whose rollback
  restores it, and deleting the variable returns the word to its text. While
  undefined, the name is a pending graph node that `why` lists among the
  sources. Static class-rule validation does not require a word to be
  defined, so `$Order.state is OPEN` is accepted while `OPEN` is undefined.
- Keywords, `$Class` references, paths, calls, subscripts, versions, operator
  expressions and parenthesized operands are not words. `a is (good)` reads
  `good` as `a = good` does and raises `ReferenceError("good is not defined")`
  while nothing defines it, and `$Order.bad is (MISSING)` raises
  `ReferenceError("MISSING is not defined")` when declared. Quoted values
  `"good"` and `'good'` are ordinary strings.
- A word assignment renders as `target is word`: in graph keys such as
  `block(total is good)`, in `--graph` output and in the `rule` that `why` and
  `affects` report, as `a is good` and `$Order.state is OPEN`. Every other
  `is` assignment without word branches or item words renders as its `=`
  spelling, such as `a = 5`.
- No error type or message is added. A malformed value raises what its `=`
  spelling raises at the same place in the source, which is one column
  further right when that place follows `is` on its line:
  `a is (1 + )` raises `SyntaxError("Unexpected ')'")` at line 1, column 12,
  and `a is b := 1` raises `SyntaxError("Invalid value assignment target")`
  at line 1, column 8.
- These readings differ from those of NUC 86, because each contains such a
  statement: `value is "EXACT"` assigns `"EXACT"`;
  `{ a is b }` assigns the word `b`; `a is 1 ? 2 : 3` assigns `2`;
  `while v is x: T` is a loop whose condition is the comparison `v is x`;
  `suppose v is x: T` is a `suppose` block whose premise is `v is x`;
  `while v is suppose:` with a body is `while` followed by `v = suppose:`; and
  `a is x: T` and `while v is assert: a, b` are syntax errors.
- `tests/is_assignments.rs`, `tests/is_assignments_cli.rs` and the terminal
  unit suite execute this behavior.
- A branch of a conditional value can be a word as well, as
  [Conditional words](#conditional-words) describes, and so can an item of a
  list, a dictionary value or a named argument, as
  [Item words](#item-words) describes. An `@name` in the value reads the
  variable `name` instead, as [At variables](#at-variables) describes.

## Conditional words

- [NUC 94](docs/nuc-0094.md) lets a branch of a conditional expression in the
  value of an `is` assignment be a word. NUC 94 introduced it behind a
  `--conditional-words` flag and a `SyntaxOptions` field; both were removed,
  and every source reads it.
- It extends [Is assignments](#is-assignments) only and combines with C-style
  comments. `run`, `check`, `parse_program`, `parse`, `parse_expression` and
  `statement::compile` read it, and `clear()` and `.clear` keep reading it.
  File runs, the plain and redirected REPL, the terminal workspace, `--json`
  and `--graph` read it.
- The word positions of an `is` assignment's value are the value itself,
  which is a word position under NUC 93, the item positions of
  [Item words](#item-words), and both branches of a conditional
  expression that stands in a word position, in either spelling,
  `consequent if condition else alternate` or
  `condition ? consequent : alternate`. So `A`, `B` and `C` are in word
  positions in `a is A if c1 else B if c2 else C`,
  `a is c1 ? A : c2 ? B : C` and `a is c1 ? c2 ? A : B : C`, and `A` and `B`
  are in word positions in `a is [A if c else B]`.
- A branch in a word position is a word when it is exactly one identifier
  token, as a value is under NUC 93: `a is A if c else 1` makes `A` a word
  and keeps `1` a number. Keywords such as `true`, `null`, `this` and `self`,
  `$Class` references, quoted strings, paths, calls, subscripts, versions,
  operator expressions and parenthesized operands are not words, so
  `a is (A) if c else B` reads `A` as `a = A if c else B` does and makes only
  `B` a word.
- Nothing else is a word position. Conditions, including the `is`
  comparisons of [Is and is not comparisons](#is-and-is-not-comparisons) in
  them, keep their reading. So does a parenthesized conditional:
  `a is (A if c else B)` reads `A` and `B` as `a = (A if c else B)` does. So
  do the operands of other operators, positional call arguments, the
  `name=value` arguments that are not item positions, function, arrow and
  lambda bodies and template interpolations inside the value, and the value
  of an assignment inside the value, as in `a is b = A if c else B`. A conditional that an assignment operator
  follows is that assignment's target, not a result, so
  `a is X if c else Y = 5` keeps its tree and raises
  `SyntaxError("Cannot assign to (c?X:Y)")` at its statement.
- A word branch reads by the word rule of NUC 93: the local, variable,
  class, function or built-in global of that name when one exists, and
  otherwise the string of its spelling, recording a dependency on the
  variable of that name. The condition is evaluated once and then only the
  selected branch, so a word in the unselected branch is not read and adds
  no dependency.
- A standing declaration, property, constructor property or class-level
  rule re-evaluates when the variable named by its selected word is
  defined, changed or deleted, including inside a `suppose` block whose
  rollback restores it, and switches branches when its condition changes.
  Deleting the variable returns the word to its text. While undefined, the
  selected word is a pending graph node that `why` lists among the sources.
- Static class-rule validation does not require a branch word to be
  defined, as for the words of NUC 93. It still requires every other name,
  so `$Order.bad is (MISSING) if $Order.total > 0 else OTHER` raises
  `ReferenceError("MISSING is not defined")` when declared, while
  `$Order.ok is MISSING if $Order.total > 0 else OTHER` is accepted.
- An `is` assignment whose value holds a word renders as `target is value`
  in statement rendering, block graph keys such as `block(total is c?A:(B))`,
  `--graph` output and the `rule` that `why` and `affects` report. In such a
  value, a conditional that holds a word renders without its grouping
  parentheses, as in `a is c?A:B`, and a name in a word position that is not
  a word renders in parentheses, as in `c?A:(B)`, so the rendering reads back
  as the same statement. A value that also holds item words renders its item
  positions as [Item words](#item-words) describes. A value whose only words
  are word branches renders its item positions as before, so
  `a is [(A)] if c else B` renders as `a is c?[A]:B`, which reads back with
  the item word `A`. Every other `is` assignment renders as before, so
  `a is 1 if c else 2` renders as `a=(c?1:2)`.
- At statement level, the operand before `?` is not an enclosed position, so
  an `is` comparison there needs parentheses, as in
  `mark is (shape is SQUARE) ? CORNER : EDGE`; a Python-style condition is
  enclosed, as in `mark is CORNER if shape is SQUARE else EDGE`.
- No token, keyword or error type or message is added, and word branches
  never change whether a program parses or where a syntax error is raised.
  A malformed value raises what its `=` spelling raises, as for
  [Is assignments](#is-assignments), for example
  `SyntaxError("Unexpected end of input")` at line 1, column 17 for
  `a is A if c else`.
- `tests/conditional_words.rs`, `tests/conditional_words_cli.rs` and the
  terminal unit suite execute this behavior.

## Item words

- [NUC 95](docs/nuc-0095.md) lets an item of a list, the value of a
  dictionary entry and the value of a named argument inside the value of an
  `is` assignment be a word. No option or flag enables it: every source
  reads it, as every source reads [Is assignments](#is-assignments) and
  [Conditional words](#conditional-words).
- It extends [Is assignments](#is-assignments) only and combines with
  C-style comments and conditional words. `run`, `check`, `parse_program`,
  `parse`, `parse_expression` and `statement::compile` read it, and
  `clear()` and `.clear` keep reading it. File runs, the plain and
  redirected REPL, the terminal workspace, `--json` and `--graph` read it.
- These are the item positions of an `is` assignment's value:
  - each item of a list literal;
  - the value of each `key: value` entry of a dictionary literal;
  - the value of each named argument `name=value` of a call whose callee is
    a bare name, including `new Name(...)`, and of `super(...)`. These are
    exactly the arguments that [Default named constructors](#default-named-constructors)
    reads as constructor labels when the name is a class.
- The literal or call may stand anywhere in the value, at any depth: inside
  another list, dictionary or named argument, as the receiver of a member
  access, call or subscript, as an argument, an operand, a subscript, a
  slice bound or a pipeline stage argument, in a condition or a branch of a
  conditional, in a quantifier, in the source of a list comprehension and
  inside grouping parentheses. So
  `[[RED, GREEN], [BLUE, AMBER]].find(pair => pair.includes(color))`,
  `{small: DIM, large: BRIGHT}[size]`, `[RED, GREEN, BLUE][x]`,
  `X if [A, B].includes(c) else Y` and
  `Light(color=RED, parts=[GREEN, GREEN])` all hold item positions.
- No item position lies inside a function, arrow or lambda body, inside the
  element or filter condition of a list comprehension, which are the bodies
  of the functions it lowers to, inside a template interpolation, in the
  value of an assignment operator inside the value, or in the target of an
  assignment operator. A block body's own `is` assignments have their own
  item positions, as every `is` assignment does. A `name=value` argument of
  any other call, such as `o.m(k=A)`, is an assignment expression, so its
  value is not an item position.
- An item position whose expression is exactly one identifier token is a
  word. Keywords such as `true`, `null`, `this` and `self`, `$Class`
  references, quoted strings, numbers, paths, calls, subscripts, versions,
  operator expressions and parenthesized operands are not words, so
  `a is [A, (B), "C", D.x]` makes only `A` a word. Every other expression in
  an item position keeps its reading.
- Dictionary keys keep their reading: an identifier key is already its own
  text, so `{s7: C}` has the key `"s7"` whatever `s7` names. The shorthand
  entry `{A}` keeps reading the variable `A`, as [NUC 17](docs/nuc-0017.md)
  defines it. Spread sources such as `{...o}`, positional arguments,
  subscripts and slice bounds are not item positions, though literals inside
  them have their own. The accepted grammar has no list spread and no
  computed dictionary key.
- An item position is a word position of
  [Conditional words](#conditional-words) as well, so a conditional standing
  in one has word branches, recursively: `A` and `B` are words in
  `a is [A if c else B]` and in `a is Light(color=c ? A : B)`. Literals in
  the branches of a conditional value have item positions as everywhere in
  the value, so `a is [A] if c else {k: B}` makes `A` and `B` words.
- An item word reads by the word rule of [Is assignments](#is-assignments):
  the local, variable, class, function or built-in global of that name when
  one exists, read as an identifier is, and otherwise the string of its
  spelling, recording a dependency on the variable of that name. Because the
  read is an identifier read, a standing declaration whose item word names a
  variable that holds `null` stores `null`, as `xs = [D, 2]` does.
- Only items that are evaluated are read. A list literal evaluates every
  item in order and a dictionary literal every value in order. A constructor
  call validates its named arguments first and then evaluates every supplied
  value once, in source order, so a call with an unknown label reads none of
  them. A conditional reads only its selected branch, and `&&`, `||` and
  `??` skip their right operand when the left one decides, so an item word
  there is not read and adds no dependency.
- A standing declaration, property, constructor property or class-level rule
  re-evaluates when the variable named by an item word it read is defined,
  changed or deleted, including inside a `suppose` block whose rollback
  restores it. Deleting the variable returns the word to its text. While
  undefined, the word is a pending graph node that `why` lists among the
  sources. A named instance such as `first is Light(color=BLUE)` reads its
  constructor values once, when it is built, as every constructor value is,
  so defining `BLUE` later leaves `first.color` as `"BLUE"`; an anonymous
  construction inside a standing value, as in
  `lights is [Light(color=BLUE)]`, builds a new instance when the value
  re-evaluates.
- Static class-rule validation does not require an item word to be defined.
  It still requires every other name, so `$Light.bad is [(MISSING)]` raises
  `ReferenceError("MISSING is not defined")` when declared, while
  `$Light.ok is [MISSING]` is accepted.
- An `is` assignment whose value holds an item word renders as
  `target is value` in statement rendering, block graph keys, `--graph`
  output and the `rule` that `why` and `affects` report, and every item
  position in it renders as a word position: a name there that is not a word
  renders in parentheses, as in `a is [A,(B)]` and `Light(color=(A))`, and a
  shorthand entry renders as `{"A":(A)}`. Dictionary keys render quoted, as
  in `a is {"small":DIM}`. The rendering then reads back as the same
  statement. A value whose only words are word branches renders as
  [Conditional words](#conditional-words) describes, so
  `a is [(A)] if c else B` renders as `a is c?[A]:B`, which reads back with
  the item word `A`, and a block holding it has the same key as a block
  holding `a is [A] if c else B`.
- No token, keyword or error type or message is added, and item words never
  change whether a program parses or where a syntax error is raised. A
  malformed value raises what its `=` spelling raises, as for
  [Is assignments](#is-assignments), for example
  `SyntaxError("Unexpected end of input")` at line 1, column 10 for
  `a is [A, `. Nested lists, dictionaries and named arguments stop at the
  nesting limit where those of the `=` spelling stop.
- These readings differ from those before NUC 95, because each holds such an
  item: `z is [A, B]` stores `["A", "B"]`, where it raised
  `ReferenceError("A is not defined")`; `$Light.ok is [MISSING]` is accepted,
  where its declaration raised `ReferenceError("MISSING is not defined")`;
  and after `good = 3`, `xs is [1, good]` renders as `xs is [1,good]`, where
  it rendered as `xs=[1,good]`, and `delete good` makes `xs` `[1, "good"]`,
  where it made `xs[1]` undefined. `=` assignments keep the reference
  reading, so `z = [A, B]` still raises `ReferenceError("A is not defined")`.
- `tests/item_words.rs`, `tests/item_words_cli.rs` and the terminal unit
  suite execute this behavior.

## At variables

- [NUC 96](docs/nuc-0096.md) adds `@name`: the character `@` directly
  followed by a name reads the variable of that name where a bare name could
  be a word. No option or flag enables it, and no token, keyword or error
  message is added.
- `@name` is `@` with a name token immediately after it: nothing, not even a
  space, tab, comment or line break, stands between them. A name is an
  identifier token. Keywords such as `true`, `null`, `None`, `this`, `self`,
  `not`, `if`, `in`, `of` and `typeof`, `$Class` references, numbers,
  strings, `(` and another `@` are not names, so `@ a`, `@1`, `@"a"`,
  `@(a)`, `@$Order`, `@this` and `@@a` keep their accepted readings.
- `@name` may stand wherever an operand begins in two regions:
  - the value of an `is` assignment of [Is assignments](#is-assignments),
    through every construct where [item positions](#item-words) can lie: the
    whole value, conditions and branches, the operands of every operator,
    receivers, arguments and named-argument values, list items, dictionary
    values, subscripts and slice bounds, quantifiers, comprehension sources,
    pipeline stage arguments, spread sources and grouping parentheses;
  - the right operand of an `is` or `is not` comparison of
    [Is and is not comparisons](#is-and-is-not-comparisons), wherever that
    comparison stands, as in `if $Order.state is @OPEN:`.
- Neither region reaches into a function, arrow or lambda body, the element
  or filter condition of a list comprehension, a template interpolation, the
  value of an assignment operator inside the region, or the target of an
  assignment operator. Outside an `is` value a `name=value` argument is such
  an assignment, so `if x is Light(color=@C):` keeps the accepted error. A
  comparison inside one of these parts has its own right operand, and a
  block body's own `is` assignments have their own values.
- `@name` reads `name` as `(name)` does: the local, variable, class, function
  or built-in global of that name, recording a dependency on it, and
  `ReferenceError("<name> is not defined")` when nothing defines it, raised
  at the start of its statement as for `=`. It is never a word. So after
  `a is 1`, `b is @a + 2` makes `b` follow `a` and be `3`, and `b is @c + 3`
  raises `ReferenceError("c is not defined")` at line 1, column 1 while
  nothing defines `c`, as `b = c + 3` does.
- Where a bare name would be a word, `@name` reads the variable instead:
  `x is @A`, `x is @A if c else B`, `x is [@A, B]`, `x is {k: @A}` and
  `x is Light(color=@A)` read `A` as `(A)` does, and `if state is @OPEN:` is
  the comparison `state == OPEN`. Where a bare name is already a reference,
  as in `b is @a + 2`, the mark changes nothing. Static class-rule
  validation requires the name to be defined, so `$Order.state is @OPEN`
  raises `ReferenceError("OPEN is not defined")` when declared while nothing
  defines `OPEN`.
- Postfix operations follow `@name` as they follow any name: `@a.b`, `@a[0]`,
  `@a(1)`, `@Light(color=RED)`, a constructor call whose named arguments are
  item positions, and `@a@first`, the version `first` of the variable `a`.
- After a complete operand, `@` is the version selector of
  [Read-only state versions](#read-only-state-versions), unchanged, as in
  `x@first` and `x@3`. A reasoning name is such an operand, so in `why @a`
  the `@a` is an unknown selector word of `why` and still raises the
  selector error. In prefix position `first`, `last` and `previous` are
  names, so `x is [@first]` reads the variable `first`. The one exception is the start of an `is`
  assignment's value: `target is @first`, `@last`, `@previous` or `@` with
  digits keeps its accepted reading as the statement `target` followed by a
  version of the name `is`, so `b is @first` is `b` and `is@first`. Write
  `b is (@first)` to read the variable `first` there.
- A leading `@` outside both regions keeps
  `SyntaxError("Unexpected character '@'")` at the `@`, as in `b = @a`,
  `return @a`, `f(@a)`, `x == @a`, a statement that starts with `@a` and
  `x is () => @a`; so does the left operand of a comparison outside an `is`
  value, as in `(@a is x)`. An assignment target or comprehension element
  that holds `@name` raises the error the accepted reading raises, at the
  same place: `SyntaxError("Unexpected character '@'")` at its first `@`, as
  for `x is [@y] = 1` and `x is [@a for v in xs]`, or, when the target begins
  the value, as in `x is @y = 1`, which the accepted grammar reads as `x`
  followed by `is@y = 1`,
  `SyntaxError("Expected a non-negative integer, 'first', 'last', or 'previous' after '@'")`
  at the name.
- A `while` or `suppose` header owns its colon after `@lambda`, which reads
  the variable `lambda`, as it does after `(lambda)`; after the bare word
  `lambda` it still does not.
- `@name` builds exactly the syntax tree of `(name)`, so a value with at
  variables renders as that spelling: `b is @a` renders as `b=a`,
  `b is c ? @a : B` as `b is c?(a):B` and `b is [@A, B]` as `b is [(A),B]`,
  and each rendering reads back as the same statement.
- REPL continuation stays lexical. A colon-free `if` header waits for its
  indented body, and an `elif`, `else if` or direct `else` alternative
  indents its next line, when an `@` and a name follow the comparison word
  `is`, or `is not`, after an operand, as in `if x is @C`; any other `@`
  keeps the accepted rule that a version selector must follow.
- `tests/at_variables.rs`, `tests/at_variables_cli.rs` and the REPL
  continuation and terminal unit suites execute this behavior.

## Instance list ranges and list queries

- [NUC 61](docs/nuc-0061.md) adds ranges on class instance lists,
  `Name.list()` and `Name.all()`, and the list queries `at`, `lastIndexOf`,
  `findIndex`, `findLast` and `findLastIndex` on lists and instance lists. It
  specifies the existing `slice`, `includes`, `indexOf` and `concat` without
  changing their results. No syntax, keyword or error type is added.
- `Name[start:end]` on a declared class `Name` returns its instances in
  population order, with the bound rules of list slices: either bound may be
  omitted, a negative bound counts from the end, a bound is converted to a
  number and clamped to the population, and a start at or after the end gives
  `[]`. `Name[:]` lists every instance, and `Name[start:end]` equals
  `Name.slice(start, end)`. The population is read once, before the bounds
  are evaluated.
- A range depends on the population and on what its bounds read, unless a
  bound is frozen with `.value`, so creating or deleting an instance
  re-evaluates it. A class-level rule validates the references in both bounds
  when it is declared, and each instance evaluates its own bounds.
- Bracket indexing is unchanged: `Name[0]` and `Name["id"]` keep their
  accepted results. A range on anything other than a list, a string or a
  declared class, such as `Math[:]` or the global `Class[:]`, still yields
  undefined.
- `Name.list()` and `Name.all()` return every instance in population order,
  with the result and dependency of `Name[:]` and `Name.slice()`. Supplied
  arguments are evaluated once, left to right, and ignored. Plain lists have
  neither method and keep raising `TypeError("<list>.list is not a function")`
  and `TypeError("<list>.all is not a function")`.
- Every query below snapshots its receiver before its arguments, evaluates
  every supplied argument once, left to right, including unused extras, and
  never writes the receiver. A declaration that uses one depends on the list
  or population, on its argument expressions and on whatever its callback
  reads. Callback errors propagate through the existing transaction, so a
  failed standing re-evaluation rolls back the change that triggered it.
- `includes(value)`, `indexOf(value)` and `lastIndexOf(value)` compare items
  with the deep equality of `assert`: lists and objects match on their
  contents, `null` and undefined match each other, so `null` matches an
  omitted value, and NaN matches nothing. Every instance holds its own `id`,
  so distinct instances never match each other. `indexOf` and `lastIndexOf`
  return the first and the last matching position, or `-1`. Only the first
  argument is compared; a JavaScript `fromIndex` is evaluated and ignored.
- A comparison reads the objects it compares without subscribing to their
  properties: a declaration follows its list or population and its argument,
  not a property of a compared object.
- `at(index)` converts the index to a number, reads NaN, including an omitted
  index, as `0`, truncates it toward zero and counts a negative index from
  the end, as in JavaScript. It returns the item at that position. Outside the
  receiver it returns what bracket indexing returns there: undefined for a
  list, which a declaration stores as `null`, and `null` for an instance list.
  A negative index before the start is outside the receiver too: `at` never
  clamps an index into the receiver.
- `findIndex(callback)` returns the position of the first item the callback
  accepts, or `-1`. `findLast(callback)` and `findLastIndex(callback)` visit
  items from last to first and return the first accepted item or its
  position, or `null` and `-1`, as `find` returns `null`. The callback
  receives only the item and is not called after an item is accepted, so
  unvisited items add no dependency. A missing or non-function callback
  raises `TypeError("<name> expects a function")`, even on an empty receiver.
- `concat(...values)` returns a new list of the receiver's items followed by
  each argument: a list contributes its items one level deep, and any other
  value contributes itself. An instance list named by its class is one value,
  not spread; `Guest.concat(Host.list())` appends the instances of `Host`.
- `slice`, `reverse`, `shift` and the aliases `contains`, `any` and `append`
  keep their accepted behavior. The ten appended entries beginning
  `Nucleoid slices class instance lists with ranges` through
  `Nucleoid rolls back failed list query updates` specify the executable
  behavior.

## Quantifiers

- [NUC 62](docs/nuc-0062.md) adds the quantified expressions
  `all NAME in COLLECTION: CONDITION` and `any NAME in COLLECTION: CONDITION`.
  No keyword or lexer token is added. A quantifier starts only where the name
  `all`, `every` or `any`, a binder name and the existing `in` keyword follow
  each other; `every` spells `all`. Everywhere else `all` and `any` keep every
  accepted reading:
  variables, functions and calls, `Name.all()` and the `.any(callback)` list
  alias, object keys and values, parameters, loop variables and typed
  declarations. A variable named `all` or `any` does not disable quantifiers,
  so `all n in all: n > any` reads both variables.
- The collection is evaluated once. A declared class yields its instances in
  population order and depends on the population, as `Name.every(...)` does.
  A list, including a reasoning selection such as `why c` or
  `c |> why |> stated`, yields its items. A null or undefined collection tests
  no member and yields undefined, which a declaration stores as `null`. Any
  other value, such as a number, string, boolean, object, instance, function
  or built-in namespace, raises `TypeError("<collection> is not a collection")`
  naming the collection's path or canonical source.
- For each member in order, the binder is bound to it in a new innermost
  frame and the condition is tested with the truthiness of `if`. `all` is
  `false` at the first member whose condition is falsy and `true` otherwise,
  including for an empty collection. `any` is `true` at the first member whose
  condition is truthy and `false` otherwise, including for an empty
  collection. Members after the deciding one are not visited and add no
  dependency.
- Over a list, an instance list, null or undefined, and with a condition that
  reads only the binder and state names, results, reads, dependencies,
  short-circuiting, null settling and errors equal those of
  `COLLECTION.every(NAME => CONDITION)` and
  `COLLECTION.some(NAME => CONDITION)`. A stored declaration whose evaluation
  read a null binding or property settles to `null`, as `pending.every(...)`
  with a null `pending` and `C.every(c => c.passed)` with a null `passed` do.
- The condition is evaluated in place, not as a function call. It reads the
  enclosing scope: enclosing binders, so quantifiers nest, function
  parameters and locals, `this` or `self`, and the instance of a class-level
  rule. The binder shadows an outer name only inside the condition and is not
  defined after the quantifier. Functions and arrows called from the
  condition still start from their own scope and do not see the binder, so
  `all x in xs: ys.some(y => y > x)` raises
  `ReferenceError("x is not defined")`; nest a quantifier instead, as in
  `all x in xs: any y in ys: y > x`.
- `.value` adds no dependency in either part. In the collection it is frozen
  when the declaration is stored, exactly as in the receiver of `every`: the
  existing freezing replaces a scalar or object reading with its value, and a
  list, which has no literal form, is read again at each evaluation. In the
  condition, as in a callback body, `.value` is read each time the condition
  is tested.
- Class-level rule validation treats the binder as bound inside the
  condition. Any other undefined name in the collection or the condition
  raises `ReferenceError("<name> is not defined")` when the rule is declared,
  and a `$Class` property read in either takes part in
  `TypeError("Circular Dependency")` checks. Each instance evaluates its own
  quantifier.
- A quantifier whose collection or condition contains a reasoning operation is
  a reasoning observer, like `(why c).every(...)`: graph walks neither select
  nor traverse it, and `model` excludes it.
- An error raised by the collection or the condition propagates through the
  existing transaction, so a failed standing re-evaluation rolls back the
  change that triggered it.
- A quantifier is an operand whose condition is a complete expression, like
  an arrow or lambda body, so it extends as far right as possible:
  `x and all v in xs: v > 0` is `x and (all v in xs: (v > 0))`,
  `not all v in xs: v > 0` negates the quantifier, and
  `all v in xs: v > 0 and ready` tests `v > 0 and ready` for each member.
  Parenthesize a quantifier to combine it with what follows, as in
  `(all v in scores: v > 0) ? "clean" : "dirty"`.
- The collection is a complete expression that ends at the quantifier's own
  `:`, parsed like the source of a `for` header. It may hold `|>`, `??`,
  `and`, `or`, a prefix stage such as `why c`, a Python-style conditional and
  conditional expressions whose colons pair with their `?`; a top-level
  `lambda` there needs parentheses. A quantifier nested in the collection or
  the condition pairs with its own colon.
- A quantifier is one logical line. Wrap it in parentheses or brackets, where
  layout is suppressed, to continue it over several lines.
- Statement headers take quantifier conditions: `if all v in xs: v > 0:` with
  an indented or same-line suite, the colon-free `if all v in xs: v > 0` of
  [NUC 49](docs/nuc-0049.md), `else if` and `elif if` headers, and the indented
  direct alternatives `else any v in xs: v > 2` of
  [NUC 43](docs/nuc-0043.md). In direct-alternative and native REPL header
  scans, a top-level quantifier's colon pairs with its `all`, `every` or
  `any` prefix, as a conditional's colon pairs with its `?`.
- At the top level of a template interpolation, whose trailing tokens are
  ignored, a quantifier is not recognized, so `` `${all v in xs: v}` `` still
  renders the variable `all`; parenthesize it, as in
  `` `${(all v in xs: v)}` ``. Colon-led `assert:` statements, call arguments,
  list items, object values with or without commas, subscripts and arrow,
  lambda and named-function bodies take quantifiers as ordinary operands.
- Both forms render canonically as `(all NAME in COLLECTION:CONDITION)` and
  `(any NAME in COLLECTION:CONDITION)`. The grouping keeps a quantifier
  distinct from an enclosing operator in graph keys and `rule` text.
- Malformed forms use existing diagnostics, for example
  `SyntaxError("Expected ':' but found end of input")` for `ok = all v in xs`
  and `SyntaxError("Unexpected end of input")` for `ok = all v in xs:`. The
  eight appended entries beginning
  `Nucleoid quantifies over class instances with all and any` through
  `Nucleoid rolls back failed quantifier updates` specify the executable
  behavior.
- `every` is a third prefix that spells `all`, as
  [Every quantifiers, class references and predicate sentences](#every-quantifiers-class-references-and-predicate-sentences)
  describes.

## Every quantifiers, class references and predicate sentences

- [NUC 98](docs/nuc-0098.md) adds three spellings of universal claims. Each
  builds the syntax tree of an accepted form, so its results, dependencies,
  errors and rendering are that form's. No token, keyword, syntax-tree
  variant, error type or message, option or flag is added, and `every` stays
  an ordinary identifier.
- **Every quantifiers.** `every NAME in COLLECTION: CONDITION` is the
  quantifier `all NAME in COLLECTION: CONDITION` of
  [Quantifiers](#quantifiers). It starts only where the name `every`, a
  binder name and the keyword `in` follow each other, wherever `all` starts
  one, and it builds the `all` node, rendered as
  `(all NAME in COLLECTION:CONDITION)`. So it evaluates, nests, depends,
  validates class-level rules, rolls back and reports diagnostics as `all`
  does, and `every v in xs: v > 0` and `all v in xs: v > 0` are one standing
  rule. Statement headers take it, and the direct-alternative and REPL header
  scans pair its colon with its `every` as they pair it with `all`. At the
  top level of a template interpolation it is not recognized, as `all` is
  not, so `` `${every v in xs: v}` `` reads the variable `every`; parentheses
  select the quantifier.
- **Class references.** Where an operand begins, the name `every` directly
  followed by another name `NAME` is the class reference `$NAME`:
  `every Human.mortal = true` is `$Human.mortal = true`,
  `every Order.big = every Order.total > 10` is a class-level rule, and
  `if every Order.total > 10:` is the class-level conditional
  `if $Order.total > 10:`. Names stay exact, so `every human` is `$human`.
  It builds exactly the tree of `$NAME`, so evaluation, class-level rule
  validation, dependencies, errors such as
  `SyntaxError("Cannot assign to $Human")` for `every Human = 4` and the
  rendering `$NAME` follow from `$NAME`. A block comment may stand between
  `every` and `NAME`, while a line break outside parentheses and brackets
  separates them, so they keep their accepted reading.
- These keep their accepted readings although `every` and a name follow each
  other where an operand begins:
  - `every NAME in`, which begins an every quantifier;
  - `every is`, whose `is` assigns or compares a variable named `every`, as
    in `every is 3` and `if every is 3:`;
  - `every NAME:`, when the colon follows `NAME` outside a statement header,
    a subscript start, the consequent after `?`, a Python-style condition and
    a quantifier collection. `NAME:` then begins the next comma-free object
    property or a colon statement, as in `{k: every x: 1}`, `every x: int`
    and `every assert: a, b`. In those clauses the colon is the clause's, so
    `if every Ready:` is `if $Ready:`;
  - the top level of a template interpolation, whose trailing tokens are
    ignored, so `` `${every x}` `` and `` `${1 + every x}` `` read the
    variable `every`, while `` `${(every Human)}` `` reads `$Human`.
- Everywhere else `every` keeps every accepted reading: a variable,
  function, parameter, property, key, label, loop name or binder, as in
  `every = 3`, `every.x`, `every(1)`, `[every, x]`, `xs.every(v => v > 0)`,
  `x.every`, `def every(x) => x`, `def f every x => every + x`,
  `lambda every: every`, `{every: 1}`, `{every}`, `f(every=1)`,
  `{every} = o`, `every, x = [1, 2]`, `class every:`, `for every in xs:` and
  `all every in xs: every`, and `every` followed by anything but a name. A
  variable named `every` disables neither form: after `every = [1]`,
  `every n in every: n > 0` quantifies over it.
- `parse_expression` reads both forms and still ignores what follows them,
  so it returns the quantifier for `every v in xs: v` and `$Human` for
  `every Human` and for `every Human is mortal`.
- **Predicate sentences.** In every statement position of
  [Is assignments](#is-assignments), a statement whose leading expression is
  a class reference without a property, written `$Class` or `every Class`
  and possibly grouped in parentheses, followed by `is` and exactly one
  name `WORD`, states `$Class.WORD = true` and builds its tree. So
  `$Human is mortal` and `every Human is mortal` both state
  `$Human.mortal = true`, which holds for every instance, created before or
  after it. `WORD` names the property whatever a variable of that name
  holds.
- It applies only when the token after `WORD` ends a line statement: the
  end of the line, a `;`, the end of an indented or braced block, the end of
  the input, `else` or `catch`. Those tokens then always ran as the three
  one-token statements `$Class`, `is` and `WORD`. Any other token after
  `WORD` keeps the accepted reading, including another statement on the
  same line: `$Human is mortal x = 1` is still `$Human`, `is`, `mortal` and
  `x = 1`, and in `$Human is mortal $Human is wise` only the second sentence
  ends its line. Inside parentheses line breaks are not statement ends, so
  a sentence in the body of a function literal passed to a call needs a
  `;` or the end of the body after it.
- Every other statement keeps its accepted reading as juxtaposed statements
  or an `is` assignment:
  - a value that is not one name, as in `$Order is 5`, `$Human is true`,
    `$Human is "mortal"`, `$Human is (mortal)` and `$Human is not mortal`;
  - a name that the expression continues, as in
    `$Human is mortal and wise`, `$Human is mortal.x` and
    `$Human is mortal = 1`;
  - a name that begins a longer statement, as in `$Human is mortal := 1`,
    `$Human is mortal: int`, `$Human is mortal is wise`,
    `$Human is while c: x` and `$Human is suppose:` with a body, and
    `$Human is is wise`, whose `is is wise` assigns a variable named `is`;
  - a class reference with a property or a state version, which is an
    `is` assignment target: `$Human.kind is mortal` stores the word
    `"mortal"`;
  - any other subject: `socrates is mortal` still stores the word
    `"mortal"`, and `this is mortal` reads three statements.
- `is` inside an expression keeps its reading as the comparison of
  [Is and is not comparisons](#is-and-is-not-comparisons), so
  `if $Human is mortal:` and `r = ($Human is mortal)` compare.
- A predicate sentence renders as its `=` spelling, `$Human.mortal=true`, in
  statement rendering, block graph keys, `--graph` output and the `rule`
  that `why` and `affects` report. It is the same rule as
  `$Human.mortal = true`: either spelling replaces the other, an instance's
  own assignment overrides it, a subclass inherits it and a `suppose` block
  or a failed transaction rolls it back.
- Not added: `is not` sentences, which keep their accepted reading, and
  `every` in any other role.
- REPL continuation stays lexical. A colon-free `if every v in xs: v > 0`
  header waits for its indented body, and an `elif` or direct `else`
  alternative with that condition indents its next line, as with `all`.
- These readings changed, as the user accepted in advance; every other input
  keeps its parse and behavior:
  - `every NAME in` where an operand begins, outside the top level of a
    template interpolation, raised a `SyntaxError`, usually
    `Unexpected keyword 'in'`, and now begins a quantifier, and
    `parse_expression("every v in xs: v")` returned the name `every` and now
    returns the quantifier;
  - `every NAME` where an operand begins, outside the readings kept above,
    ended an expression with `every` and began the next statement or item
    with `NAME`, and now reads `$NAME`. So after `every = 3`,
    `every Human = 4` assigned `Human` and now raises
    `SyntaxError("Cannot assign to $Human")` when it runs, and
    `parse_expression("every x")` returned the name `every` and now returns
    `$x`;
  - a predicate sentence ran as the three statements `$Class`, `is` and
    `WORD`, so `$Human is mortal` raised
    `ReferenceError("is is not defined")` at its `is` unless a variable
    named `is` existed, and now states the rule.
- The five appended entries beginning
  `Nucleoid states class predicates with is` through
  `Nucleoid keeps other is statements on a class reference` specify the
  executable behavior. `tests/universal_claims.rs`,
  `tests/universal_claims_cli.rs` and the REPL continuation and terminal unit
  suites execute the rest.

## Exponentiation

- [NUC 64](docs/nuc-0064.md) adds the exponentiation operator
  `base ** exponent` and its compound assignment `target **= exponent`. Each
  is one lexer token. `*`, `*=`, `Math.pow` and every other operator keep
  their grammar and behavior.
- `a ** b` yields exactly the value of `Math.pow(a, b)`. Both operands are
  converted to numbers as for `*`: `null`, `false`, the empty string and an
  empty list are `0`, `true` is `1`, a numeric string or a one-item list is
  its number, a date is its time in milliseconds, and objects, classes,
  regular expressions, functions, other strings and longer lists are NaN. The
  same host floating-point power as `Math.pow` then
  gives NaN for a negative base with a non-integer exponent, infinities for a
  zero base with a negative exponent, and the signed zeros `Math.pow` gives.
  An undefined operand, such as a missing property, makes the result
  undefined, which a declaration stores as `null`.
- `**` does not read the name `Math`. After `Math = 1`, `Math.pow(2, 3)`
  raises `TypeError("number.pow is not a function")` while `2 ** 3` is still
  `8`.
- The left operand is evaluated before the right one, as `Math.pow` evaluates
  its arguments.
- A declaration that uses `**` is live like any arithmetic declaration: it
  depends on what its operands read, re-evaluates when they change and
  replaces obsolete dependencies. A stored declaration whose evaluation read
  a null binding or property settles to `null`, as `y = x * 2` and
  `y = Math.pow(x, 2)` do with a null `x`, while a literal `null` operand is
  `0`. `.value` operands are frozen when the declaration is stored.
  Class-level rules validate the names in both operands and take part in
  `TypeError("Circular Dependency")` checks when they are declared, and each
  instance evaluates its own power. Errors propagate through the existing
  transaction, so a failed standing re-evaluation rolls back the change that
  triggered it. Variables, properties, class-level rules, function and method
  bodies, arrow and lambda callbacks, list and object elements, template
  interpolations, call arguments and subscripts use `**` as an ordinary
  operand.
- `**` binds tighter than `*`, `/` and `%`, and so than every binary
  operator, and it is right-associative: `2 ** 3 ** 2` is `512`,
  `3 * x ** 2` is `3 * (x ** 2)` and `(2 ** 3) ** 2` is `64`. Member access,
  calls, subscripts and version selectors bind tighter than `**`, so
  `box.side ** 2` raises the property.
- Prefix operators follow Python. `-`, `+`, `!`, `not`, `typeof`, `delete`
  and prefix reasoning stages such as `why`, written before a base, apply to
  the whole power: `-x ** 2` is `-(x ** 2)`, `not x ** 0` is
  `not (x ** 0)` and `typeof x ** 2` is `"number"`. Write `(-x) ** 2` to
  raise a negative base. The exponent after `**` may carry prefix operators
  itself: `2 ** -1` is `0.5` and `2 ** -3 ** 2` is `2 ** (-(3 ** 2))`.
  `new` stays part of its operand, so `new Shape(2) ** 2` raises the
  instance.
- Each `**` takes one level of the existing 64-level expression nesting
  budget, as a prefix operator does. An assignment such as
  `x = 1 ** 1 ** ...` holds at most 61 `**` operators and a bare expression
  statement 62; one more raises
  `SyntaxError("Expressions are nested too deeply")` at the operand after it.
- `target **= exponent` is an optional single-token spelling of
  `target = target ** (exponent)` under the compound arithmetic assignment
  rules above: targets, validation and its 128-link bound, right-associative
  assignment precedence, standing relationships and self-reference, locals
  and receivers, rollback and canonical expanded rendering. Diagnostics name
  the operator `'**='`.
- `* *` stays two tokens, `***` is `**` followed by `*`, and `**==` is `**=`
  followed by `=`. A `/` after `**` or `**=` starts a regular expression, as
  after `*`. `**` inside strings, template text outside interpolations,
  regular expressions such as `/x**y/`, and comments is unchanged. Adjacent
  `**` or `**=` elsewhere was a syntax error before this operator, except in
  the trailing tokens of a template interpolation, which are still ignored:
  `` `${x y ** 2}` `` renders the value of `x`.
- `**` renders canonically as `base**exponent`. A base that is a prefix,
  binary, logical, assignment, `delete` or pipeline expression or a negative
  number, and an exponent that is a binary expression other than `**` or a
  logical, assignment or pipeline expression, are parenthesized, so the text
  reads back as the same expression: `(-x)**2`, `-x**2`, `(x**2)**3`,
  `x**2**3` and `x**(a+b)`. Other operators keep their rendering.
- Malformed forms use existing diagnostics, for example
  `SyntaxError("Unexpected '**'")` for `x = **2`,
  `SyntaxError("Unexpected end of input")` for `x = 2 **` and
  `SyntaxError("Invalid compound assignment target")` for `2 **= 3`.
  Applying `delete` or `why` to a power raises, for example,
  `SyntaxError("Cannot delete p.v**2")` and
  `TypeError("Cannot explain x**2")`. The eight appended entries beginning
  `Nucleoid raises numbers to powers with the exponentiation operator`
  through `Nucleoid rolls back failed exponentiation updates` specify the
  executable behavior.

## Hypotheticals

- [NUC 65](docs/nuc-0065.md) adds `suppose` blocks. `suppose PREMISE:`
  followed by a suite runs the premise statement and then the suite in a
  hypothetical world, and discards everything they did when the block ends.
  `suppose:` followed by a suite has no premise. `TARGET = suppose PREMISE:`
  and `TARGET = suppose:` assign the block's value. The suite is an indented
  block or statements on the same line separated by `;`, as for `if`. No
  keyword or lexer token is added.
- A statement is a `suppose` block only where the accepted grammar rejected it.
  The statement must be the name `suppose`, or a plain `=` assignment whose
  value is exactly the name `suppose`, and the tokens after that name must be
  `:` or one premise immediately followed by `:`. The premise is an expression
  or an assignment, parsed like an `if` header, so conditional expressions and
  quantifiers in it pair their own colons.
- Everywhere else `suppose` stays an ordinary name: a variable, function,
  parameter, property, object key, loop variable or template value. These
  spellings keep their accepted readings:
  - `suppose` followed by a statement and no colon, such as
    `suppose reading = 95` followed by an indented block or by `{ ... }`,
    which are juxtaposed statements;
  - `suppose x: int`, `suppose: int` and `x = suppose y: int`, which hold a
    typed declaration;
  - `suppose assert: a, b`, the name `suppose` followed by a colon-led
    assertion;
  - a premise whose last token before the colon is `lambda`, as in
    `suppose f = lambda: 1`, which is `suppose` followed by a lambda.
- A compound or chained assignment such as `x += suppose ...` or
  `a = b = suppose ...`, `return suppose ...`, and `suppose` inside
  parentheses, brackets or call arguments are not `suppose` blocks; they keep
  their accepted `SyntaxError`.
- The premise and the suite run in the current scope on a transaction
  savepoint. Rules propagate there exactly as they do outside, and the block's
  own propagation finishes inside it, even when the block runs while other
  changes are propagating; the statement being re-evaluated does not re-run
  itself.
- When the block ends, whether normally, by an error or a thrown value, or by
  `return`, everything it did is rolled back:
  - variable, property and object writes;
  - created and deleted instances;
  - class, function and class-level rule declarations;
  - graph nodes and dependency edges;
  - reasoning observers and their structural subscriptions;
  - state versions and queued propagation;
  - local bindings such as parameters and loop variables.
- Assertions that ran inside the block stay counted, and their failures stay
  recorded.
- A `suppose` block adds no dependency and no structural subscription to the
  declaration, block or call that runs it.
- The block's value is the value of its last statement when that statement is
  an expression statement other than an `assert` call, or a `suppose` block
  without a target whose own block has a value. Otherwise the block has no
  value.
- The block's value, a value thrown out of it and a value returned from it
  leave as plain data:
  - null, booleans, numbers, strings, dates, regular expressions and functions
    leave unchanged;
  - a list leaves with each item converted the same way;
  - every object the value reaches, instances and reasoning steps included,
    is copied into a new plain object that holds its converted properties as
    they were when the block ended, keeping shared references and cycles. A
    copy belongs to no class: it is not an instance, joins no population, has
    no methods and is skipped by `for`. An object that no longer existed when
    the block ended is copied as an empty object;
  - a class value leaves unchanged when a class or built-in namespace of that
    name exists after the rollback, and as `null` when the class was declared
    inside the block.
- A function declared inside the block loses its name with the rollback, but
  its value stays callable. The names in its body are resolved when it runs,
  so a body that reads a name declared only inside the block raises
  `ReferenceError("<name> is not defined")`.
- A reasoning selection that leaves the block is plain data: its steps keep
  their `holds` values and it no longer re-derives.
- Without a target, a `suppose` block is a statement whose value is the
  block's value, or `null` when the block has none. A program or native REPL
  input that ends with one shows that value.
- `TARGET = suppose ...` runs the block once and then assigns its value as
  `TARGET = LITERAL` does. LITERAL is a literal of the value, an object
  reference, a function, `Date(<milliseconds>)` for a date or the name of a
  class. The stored declaration has no dependency and never runs the block
  again. The target is resolved after the rollback, as for `TARGET = LITERAL`:
  a name, a local, a property path, `this.x` or `$Class.property`, where
  `$Class.property = suppose ...` declares a class-level rule whose value is
  the same snapshot for every instance.
- An assigned block that has no value raises
  `TypeError("suppose has no value")` when the statement runs, before the block
  runs.
- `return` inside the block leaves the block, then the enclosing function,
  method or program, with the returned value converted to plain data after the
  rollback.
- An error or a thrown value inside the block is raised to the enclosing
  statement after the rollback. A thrown value is converted to plain data like
  the block's value. A `try` that catches it then restores the state from
  before its own body, as for any failure, which removes those plain copies
  again. In the catch, scalars, lists and functions in the thrown value are
  intact, and an object in it reads every property as undefined, like an
  object created in a `try` body.
- Blocks nest. An inner block is rolled back before the outer one, and its
  value leaves into the outer hypothetical world.
- A `suppose` block runs each time the statement holding it runs: once at the
  top level, on each call of a function or method, on each iteration of a
  `for` body, and whenever an enclosing standing `if` branch or block runs
  again. A `suppose` block never makes the block or conditional around it a
  class-level rule. Inside a block or conditional that is a class-level rule,
  it runs once per instance, and `$Class` reads that instance.
- Graph keys fold the premise into the block, as in `suppose{reading=95;alarm}`
  and `peak=suppose{reading=95;alarm}`. Inside default-bearing declarations a
  block renders as `suppose:` or `peak=suppose:` with an indented body. Both
  forms parse back to the same block.
- The native REPL buffers a line ending in `:` until a blank line, so
  `suppose reading = 95:` and `peak = suppose reading = 95:` wait for their
  body, while `suppose reading = 95: alarm` runs at once and prints its value.
  In the terminal workspace, Ctrl-J indents after such a header, and
  `suppose` is a completion word.
- Malformed blocks use existing diagnostics, for example
  `SyntaxError("Unexpected end of input")` for `peak = suppose reading = 95:`
  and `SyntaxError("Expected an indented block but found end of input")` when
  only a newline follows the colon. The eight appended entries beginning
  `Nucleoid runs a suppose block in a hypothetical world` through
  `Nucleoid supposes a changed class level rule` specify the executable
  behavior.

## Hypothetical differences

- [NUC 83](docs/nuc-0083.md) adds the pipeline stage `SELECTION |> diff`.
  Inside a `suppose` block it keeps the steps of `SELECTION` whose node did
  not exist when the innermost running block began, its premise included, or
  whose value differs from the value the node held then. It keeps them in
  their order, with their `node`, `holds`, `rule`, `state` and `from` fields
  unchanged, and drops the others.
- `model |> diff` therefore lists every node that the hypothetical added or
  changed, `alarm |> why |> diff` the changed facts behind an alarm,
  `price |> affects |> diff` what changed among the nodes a price reaches,
  and `model |> diff |> affects` everything that rests on a change. A node
  removed inside the block cannot appear, because the stage only keeps steps
  that exist.
- A node existed when the block began if the model held it then. The
  placeholder of a name read before its definition and a reasoning observer,
  which no selection lists, did not exist, so a property first defined in the
  block under a rule that already read it is new, and so is a former
  reasoning declaration redeclared there as an ordinary one. A property that a
  statement declares inside the block is new, even when an object literal
  already held that value.
- The value a node held is what its step holds: the value of a variable or
  an instance, the value of a property, and `null` for classes, named
  functions, conditions and blocks, which therefore count only when they are
  new. Values compare as a write compares the value it stores with the one it
  replaces, which is what decides whether the write propagates:
  - numbers are the same when they are equal and have the same sign, and
    `NaN` is the same as `NaN`, so `-0` differs from `0`;
  - strings, booleans, `null`, dates and class values compare by value, and
    regular expressions by their pattern;
  - lists are the same when they have the same length and the same items in
    order under this rule, so a list mutated in place, as by `push`, differs
    and a list restored, as by `pop`, does not;
  - objects, instances and function values are the same only when they are
    the same object or function. An instance whose properties change is not
    listed, while each changed property is, and a new object with equal
    entries differs.
- An equal-valued write, a value restored before the stage runs, a rule
  rewritten to give the same value, a node deleted and declared again with
  the value it had, and a write undone by a caught failure are not
  differences.
- The innermost running block is the one whose premise or suite is running
  when the stage is evaluated: directly, in a function, method or lambda
  called there, or in a rule that propagation evaluates there. `try`
  savepoints inside the block do not change it, and a nested block compares
  with its own start. Each evaluation compares with the block running when it
  happens, so a standing selection declared in an outer block that
  re-derives inside a nested block compares with the nested block's start
  until that block rolls back.
- Evaluating the stage when no `suppose` block is running raises
  `TypeError("Cannot use 'diff' outside suppose")`, positioned at the running
  top-level statement, and rolls back its transaction. The error is raised
  when the stage applies, after its source is selected and the stages before
  it have applied. A declaration, class-level rule or function that holds the
  stage can therefore only be evaluated inside a block; outside one, its
  evaluation raises and rolls back the statement that triggered it.
- The stage keeps the distances of the steps it keeps, so it composes with
  `depth`, `stated`, `derived`, `why` and `affects` in any order:
  `c |> why |> diff |> depth(1)` and `c |> why |> depth(1) |> diff` select
  the same steps.
- Only `|>` followed by the name `diff` is the stage, without an argument,
  and further stages may follow it. `diff` remains an ordinary lexer
  identifier: variables, parameters, properties, keys, functions and methods
  named `diff` keep their accepted meanings, there is no prefix form, so
  `diff c` stays two juxtaposed statements, and `diff(c)` is an ordinary
  call. A bare `diff` stays a name, so `suppose a = 2: diff` still raises
  `ReferenceError("diff is not defined")`. `c |> diff(1)` is the stage
  followed by the juxtaposed statement `(1)`, as `c |> why(1)` is.
- A local or parameter, state variable, function or class named `diff`
  shadows the stage, as for `stated`, `derived` and `depth`: the stage then
  raises `TypeError("diff is not an operation")` before selecting its source.
  A binding declared after a standing selection makes its next evaluation
  raise, which fails and rolls back the statement that triggered it. The
  accepted `why` and `affects` shadowing behavior does not change.
- An invalid source keeps its diagnostic and is reported first:
  `1 |> diff` raises `TypeError("Cannot explain 1")` and `missing |> diff`
  raises `ReferenceError("missing is not defined")`, inside a block or not.
- Like the state filters, the stage preserves every value and structural
  observation its source made, including those of the steps it drops, so a
  standing selection inside a block re-derives when its source does, and a
  dropped step joins it once that node changes. A declaration, property,
  class-level rule, condition or quantifier that holds the stage is a
  reasoning observer. `.value` snapshots it as for the accepted stages, and a
  selection that leaves the block is plain data.
- The stage renders as `SOURCE|>diff` in graph keys, block keys, conditions
  and `rule` text. The values the block began with are read from the
  transaction's undo records since the block's savepoint, in time linear in
  those records and the selected steps; programs without the stage do no
  work for it beyond noting where each block begins and ends.
- The eight appended entries from
  `Nucleoid lists what a supposed premise changes` through
  `Nucleoid composes hypothetical differences with state filters` specify the
  executable behavior.

## Value assignment

- [NUC 66](docs/nuc-0066.md) adds the value assignment statement
  `TARGET := EXPR`, an action taken once beside the standing declarations
  that `=` makes. `:=` is one lexer token, described as `':='`. `=` and
  `.value` keep their meaning.
- `TARGET := EXPR` evaluates `EXPR` once, in the current scope, and assigns
  the value to `TARGET` exactly as `TARGET = LITERAL` would. LITERAL is a
  literal of the value, an object reference, a function,
  `Date(<milliseconds>)` for a date or the name of a class. The value is the
  one `TARGET = EXPR` would store at that moment, so an evaluation that reads
  a null binding or property, or yields an undefined value, stores `null`.
- The statement records no dependency and no structural subscription, for
  its target or for anything around it: an `if`, a block, a function call or
  a declaration that runs it. This covers every read made while evaluating
  `EXPR`, including reads inside the functions, methods, callbacks and
  reasoning selections it calls, and the reads that locate the target, such
  as `order` in `order.total := 5`. `.value` inside `EXPR` changes nothing.
- Wherever `=` files a declaration, as at the top level, the target becomes a
  stated fact. Its declaration is the literal with no dependencies, so it
  replaces any standing relationship the target had, and its dependents
  propagate as usual. For a variable,
  `why` shows the step `TARGET = LITERAL` with state `stated` and an empty
  `from`; a property keeps the `$Class.property = LITERAL` rendering and the
  `derived` state that every instance property has, with an empty `from`.
  Later changes to what `EXPR` read do not change the target.
- Wherever `=` is only carried out, in a `for` body, in the branch of a
  standing `if`, in a `{ }` block and in the functions these run, the
  statement is carried out once each time it runs, without being filed,
  exactly as `=` is there. A standing `if` runs it again whenever its branch
  runs, and the statement adds nothing to what the `if` depends on.
- Targets are names and plain dotted paths rooted in a name, `$Class`,
  `this` or `self`, under the compound assignment target rules: parentheses
  may group a path, and a path holds at most 128 dotted links. A local name,
  such as a parameter or a loop variable, is assigned locally. Any other name
  and every property path are assigned as with `=`, so a name a function does
  not bind is a state variable and a `:=` in a class body is a constructor
  statement.
- Outside a class-level rule, `$Class.property := EXPR`, and
  `Name.property := EXPR` where `Name` is a class, raise
  `TypeError("Cannot use ':=' on a class-level property")`. A class-level
  declaration also governs instances created later, so it is a law rather
  than an action. Inside a class-level `if` or block, `$Class` is the instance
  being processed, so `$Class.property := EXPR` assigns that instance's
  property once each time the rule runs for it. `$Class` inside `EXPR` reads
  exactly as it does on the right of `=` in the same place.
- `TARGET := Class(arguments)` creates the instance as `=` does, named after
  its target, and reads the arguments without dependency.
- The statement's value is the assigned value, as for `=`, so a program or a
  native REPL input that ends with it shows that value. A state version is
  recorded only when the value changes, as for `=`.
- Errors raised while evaluating `EXPR`, while assigning or during the
  propagation that follows roll back through the existing transaction.
- When the statement files a declaration, its literal is bounded by the
  runtime's 192-level expression depth limit. A list nested 192 levels or
  more raises `TypeError("Maximum expression depth exceeded")` at the
  statement, which rolls back. Nothing is filed in a `for` body, an `if`
  branch or a block, so no limit beyond those of `=` applies there.
- `:=` is a statement. Expression positions keep their existing diagnostics,
  which now name `':='`: `[x := 1]` raises
  `SyntaxError("Expected ']' but found ':='")`, `f(x := 1)` raises
  `SyntaxError("Expected ')' but found ':='")`, `if x := 1:` raises
  `SyntaxError("Expected ':' but found ':='")` and `return x := 1` raises
  `SyntaxError("Unexpected ':='")`. `parse_expression` still ignores trailing
  tokens.
- Other targets raise `SyntaxError("Invalid value assignment target")` at the
  `:=`: indexes, slices, calls, a bare `this` or `$Class`, literals and
  assignment expressions such as `y = x := 1`. A version selector passes this
  check and raises `TypeError("State versions are read-only")` when the
  statement runs, as `x@previous += 1` does. A missing object, a non-object
  base and the property name `value` raise the errors `=` raises.
- The lexer reads `:=` by longest match: `: =` stays two tokens, `:==` is
  `:=` followed by `=`, `:=>` is `:=` followed by `>`, `::=` is `:` followed
  by `:=`, and `?:=` is `?` followed by `:=`. A `/` after `:=` starts a regular
  expression. Strings, template text, regular expressions and comments are
  unchanged. No accepted program had `:` directly followed by `=` outside such
  text except in the trailing tokens of a template interpolation, which are
  still ignored: `` `${x:=2}` `` renders the value of `x`.
- Every other colon keeps its reading: typed declarations and fields,
  `assert:` statements, conditional expressions, quantifiers, lambdas,
  slices, object keys, class parents and suites. `x: =1` keeps
  `SyntaxError("Unexpected ':'")`. `lambda`, `assert` and `suppose` stay
  ordinary names, so `lambda := 3` assigns the name `lambda`.
- `TARGET := suppose ...` is not a `suppose` block: `x := suppose` reads the
  name `suppose`, and `x := suppose: 1` raises `SyntaxError("Unexpected ':'")`.
  `TARGET = suppose ...` already stores a one-shot snapshot. A `suppose`
  premise cannot be a value assignment. A value assignment inside a `suppose`
  suite is discarded with the block.
- Graph keys render the statement as `target:=value`, so `{ x := a }` and
  `{ x = a }` are different blocks.
- The native REPL runs a `:=` line at once. Its colon-free header scans and
  the parser's direct-alternative scan treat a top-level `:=` like `;`, so
  `if x := 1` is not held as a header. Completion words are unchanged.
- The eight appended entries beginning
  `Nucleoid applies a value assignment once` through
  `Nucleoid rolls back failed value assignments` specify the executable
  behavior.

## List and population sums

- [NUC 68](docs/nuc-0068.md) adds the read-only list query `sum` to lists and
  class instance lists. No syntax, keyword or error type is added.
- `RECEIVER.sum()` returns the total of the receiver's items and
  `RECEIVER.sum(selector)` the total of `selector(item)` for every item. The
  values are converted to numbers and added left to right, starting from `0`,
  in 64-bit floating point, so a list of numbers totals exactly what
  `reduce((total, item) => total + item, 0)` returns. An empty receiver gives
  `0`.
- The receiver is a list, however it is reached, or a class's instance list in
  population order, named by the class or by a variable that holds the class.
  On a class name, `sum` is the population query even when the class declares
  an instance method `sum`; its instances keep calling that method.
- Each added value is converted with the numeric conversion of `-`, `*` and
  the Math helpers: `null`, `false`, an empty or whitespace-only string and an
  empty list are `0`, `true` is `1`, a numeric string after trimming is its
  number, a one-item list is its item's number, a date is its time in
  milliseconds, and objects, classes, functions, regular expressions, other
  strings and longer lists are NaN. NaN and infinities add as 64-bit floating
  point.
- An undefined item or selector result makes the total undefined, which a
  declaration stores as `null`; every item is still visited. `sum()` reads
  items, not bindings, so a `null` item counts `0` without settling a
  declaration to `null`, while the reads inside a selector follow the existing
  rules.
- A selector is called once for every item, in order, with only the item;
  further parameters are `null`. A supplied first argument that is not a
  function, including `null`, raises `TypeError("sum expects a function")`,
  even on an empty receiver.
- `sum` follows the shared rules of the list queries: it reads its receiver
  before its arguments, evaluates every supplied argument once, left to right,
  including unused extras, and never writes the receiver. A declaration that
  uses it depends on the list or population, on its argument expressions and
  on whatever the selector reads, so list mutations, new and deleted
  instances and property changes re-evaluate it, in class-level rules too.
  Selector errors roll back through the existing transaction. A null or
  undefined receiver yields undefined without evaluating the arguments.
- Property reads such as `xs.sum`, bindings and object properties named
  `sum`, and instance methods named `sum` called on an instance keep their
  accepted results. The eight appended entries beginning
  `Nucleoid totals lists with sum` through
  `Nucleoid rolls back failed sum updates` specify the executable behavior.

## List and population extremes

- [NUC 69](docs/nuc-0069.md) adds the read-only list queries `max` and `min`
  to lists and class instance lists. No syntax, keyword or error type is
  added.
- `RECEIVER.max()` returns the largest of the receiver's items and
  `RECEIVER.max(selector)` the largest `selector(item)`; `min` returns the
  smallest. The result is `Math.max` or `Math.min` applied to those values in
  receiver order. An empty receiver gives `-Infinity` for `max` and
  `Infinity` for `min`, as `Math.max()` and `Math.min()` do.
- The result is a number, never the item. Each value is converted with the
  numeric conversion of `-`, `*` and the Math helpers, so `["10", "9"].max()`
  is `10`, `true` counts `1`, `null`, `false` and `""` count `0`, a date
  counts its time in milliseconds, and objects and other strings are NaN. A
  NaN value makes the result NaN, and infinities compare as 64-bit floating
  point.
- An undefined item or selector result makes the result undefined, which a
  declaration stores as `null`, even beside a NaN value; every item is still
  visited. `max()` and `min()` read items, not bindings, so a `null` item
  counts `0` without settling a declaration to `null`, while the reads inside
  a selector follow the existing rules.
- A tie between `0` and `-0` gives the same result as `Math.max` or
  `Math.min` on the same values in the same order. That parity is the
  contract, this specification, not JavaScript, decides, and the language
  does not otherwise fix the sign of a zero tie. The current x86-64 reference
  build keeps the later zero, so there `[0, -0].max()` is `-0` and
  `[-0, 0].min()` is `0`, where JavaScript's `Math.max` and `Math.min` give
  `0` and `-0`.
- The receiver is a list, however it is reached, or a class's instance list in
  population order, named by the class or by a variable that holds the class.
  On a class name, `max` and `min` are population queries even when the class
  declares instance methods `max` or `min`; its instances keep calling those
  methods.
- A selector is called once for every item, in order, with only the item;
  further parameters are `null`. A supplied first argument that is not a
  function, including `null`, raises `TypeError("max expects a function")` or
  `TypeError("min expects a function")`, even on an empty receiver.
- `max` and `min` follow the shared rules of the list queries: they read the
  receiver before their arguments, evaluate every supplied argument once, left
  to right, including unused extras, and never write the receiver. A
  declaration that uses one depends on the list or population, on its
  argument expressions and on whatever the selector reads, so list mutations,
  new and deleted instances and property changes re-evaluate it, in
  class-level rules too. Selector errors roll back through the existing
  transaction. A null or undefined receiver yields undefined without
  evaluating the arguments.
- Property reads such as `xs.max`, bindings and object properties named `max`
  or `min`, instance methods named `max` or `min` called on an instance, and
  `Math.max` and `Math.min` keep their accepted results. The eight appended
  entries beginning `Nucleoid finds list extremes with max and min` through
  `Nucleoid rolls back failed extreme updates` specify the executable
  behavior.

## List and population distinct values

- [NUC 71](docs/nuc-0071.md) adds the read-only list query `distinct` to lists
  and class instance lists. No syntax, keyword or error type is added.
- `RECEIVER.distinct()` returns a new list of the receiver's items without
  repeats, keeping the first occurrence of each in receiver order.
  `RECEIVER.distinct(selector)` returns the distinct values of
  `selector(item)` the same way, so it equals
  `RECEIVER.map(selector).distinct()`. An empty receiver gives `[]`.
- A value is kept when no value already kept matches it under the deep
  equality of `assert`, `includes` and `indexOf`, compared as
  `kept.includes(value)` compares, so `xs.distinct()` returns the list that
  `xs.reduce((kept, x) => kept.includes(x) ? kept : kept.concat([x]), [])`
  evaluates to. Kept values are the first occurrences, unchanged: an object
  stays the same object.
- Values are compared, never converted: `1`, `"1"` and `true` are three
  values, and a date matches the number of its milliseconds. `null` and
  undefined are one value, and an undefined item or selector result is kept
  like any other value. `0` and `-0` are one value, and the first is kept
  with its sign. NaN matches nothing, so every NaN is kept.
- Lists match when their items match in order. Objects match when they hold
  matching values for the same defined properties, in any order; every
  instance holds its own `id`, so distinct instances never match each other.
  Functions and regular expressions match nothing, not even themselves. The
  quirks of deep equality carry over exactly: after `a = {k: null, m: 1}`
  and `b = {k: {}.none, m: 1, z: 5}`, `[a, b].distinct()` keeps only `a` and
  `[b, a].distinct()` keeps both, as `[a].includes(b)` is `true` and
  `[b].includes(a)` is `false`; a record holding an instance's `id` and
  fields matches that instance; and a list nested more than 64 levels deep
  matches nothing, not even itself, while an object always matches itself.
- A comparison reads the objects it compares without subscribing to their
  properties. A declaration that uses `distinct` follows its list or
  population, its argument expressions and what the selector reads; changing
  a property of a compared object does not re-evaluate it.
- `distinct()` reads items, not bindings, so a `null` item is kept without
  settling a declaration to `null`, while the reads inside a selector follow
  the existing rules.
- The receiver is a list, however it is reached, or a class's instance list in
  population order, named by the class or by a variable that holds the class.
  On a class name, `distinct` is the population query even when the class
  declares an instance method `distinct`; its instances keep calling that
  method.
- A selector is called once for every item, in order, with only the item;
  further parameters are `null`. A supplied first argument that is not a
  function, including `null`, raises `TypeError("distinct expects a function")`,
  even on an empty receiver.
- `distinct` follows the shared rules of the list queries: it reads its
  receiver before its arguments, evaluates every supplied argument once, left
  to right, including unused extras, and never writes the receiver. List
  mutations, new and deleted instances and changes to properties that a
  selector reads re-evaluate a declaration that uses it, in class-level rules
  too. Selector errors roll back through the existing transaction, and a
  rolled-back list keeps its order. A null or undefined receiver yields
  undefined without evaluating the arguments.
- Each value is compared with deep equality only against kept values that
  share its structural hash. An object holding a property whose value is
  undefined is also compared with the kept objects that hold a `null`
  property and as many defined properties, the only values that can match it
  outside its hash. A call makes at most one comparison per pair of a value
  and a kept value.
- Property reads such as `xs.distinct`, bindings and object properties named
  `distinct`, and instance methods named `distinct` called on an instance keep
  their accepted results. The eight appended entries beginning
  `Nucleoid removes duplicate list items with distinct` through
  `Nucleoid rolls back failed distinct updates` specify the executable
  behavior.

## List and population predicate counts

- [NUC 73](docs/nuc-0073.md) adds the read-only shared query
  `RECEIVER.count(predicate)` to every list and class population receiver
  supported by `filter`, including class aliases. No syntax, keyword, alias,
  no-argument overload or public Rust API is added.
- For a function predicate, the query returns the Number that
  `RECEIVER.filter(predicate).length` returns, without allocating a filtered
  result list. A valid empty receiver returns `0`.
- The predicate is invoked once per item, in receiver order, with only the
  item, through the same invocation and truthiness rules as `filter`.
  Remaining parameters follow the normal missing-argument/default rules.
  Every item is visited unless an invocation fails; a truthy or falsy result
  never short-circuits the query.
- A missing or non-function first argument, including `null` or undefined,
  raises `TypeError("count expects a function")`, even on an empty list or
  population. There is no `count()` form for the receiver's length.
- The receiver is snapshotted before the arguments, and every supplied
  argument is evaluated once, left to right, including evaluated-and-ignored
  extras. The query never writes the receiver; callback side effects and
  errors keep their existing behavior. A null or undefined receiver yields
  undefined without evaluating the arguments, as for `filter`.
- A declaration follows its receiver, argument expressions and the reads
  made by every predicate invocation. List changes, population additions
  and removals, and changes to those reads re-evaluate it, including in
  class-level rules. `.value`, state-version receivers, nullish reads and
  deferred declarations work exactly as in `filter(predicate).length`;
  a numeric query result does not suppress missing-value settlement.
  Failures use the existing transaction and rollback behavior.
- Bindings, properties and instance methods named `count` are unchanged,
  as are property reads such as `items.count`. On a class or class alias,
  `Name.count(predicate)` queries the population even when its instances
  have a method named `count`; calls on an instance still invoke that method.
  All accepted `filter(predicate).length` forms remain available unchanged.
- The four appended entries beginning `Nucleoid counts matching list items`
  through `Nucleoid uses predicate counts beside ordinary count names`
  specify the executable behavior. Their original assertions, including the
  population `filter(...).length` assertions, are preserved.

## List and population modes

- [NUC 75](docs/nuc-0075.md) adds the read-only list query `mode` to lists
  and class instance lists. No syntax, keyword or error type is added.
- `RECEIVER.mode()` returns the receiver's most frequent item.
  `RECEIVER.mode(selector)` returns the most frequent value of
  `selector(item)` the same way, so it equals `RECEIVER.map(selector).mode()`.
  An empty receiver gives undefined, which a declaration stores as `null`.
- Items are counted against the values `distinct` keeps. An item that no
  value kept before it matches, under the deep equality of `assert`,
  `includes` and `indexOf` compared as `kept.includes(item)` compares, is
  kept and counts once for itself. Every other item counts once for the
  earliest kept value that matches it. The result is the kept value with the
  highest count, unchanged: an object stays the same object, and the mode of
  a nonempty receiver is always one of the values that `distinct` keeps.
- Among kept values with the same count, the one that comes first in
  receiver order wins, as Python's `statistics.mode` returns the first mode it
  meets: `[3, 4, 4, 3].mode()` is `3`.
- When deep equality sorts the items into groups, as it does for numbers
  other than NaN, strings, booleans, dates, `null`, undefined, classes and
  instances, and for lists and records built from them, nested at most 64
  levels, in which no record holds an undefined-valued property, the result
  is the item that `xs[counts.indexOf(counts.max())]` picks, where
  `counts[i]` is the number of items that match `xs[i]`.
- Values are compared, never converted: `1`, `"1"` and `true` are three
  values, and a date matches the number of its milliseconds. `null` and
  undefined are one value, and the first of them is the one returned. `0` and
  `-0` are one value, and the first is returned with its sign.
- NaN, functions and regular expressions match nothing, not even themselves,
  so each one is kept and counts once: `[0 / 0, 0 / 0, 1].mode()` is the
  first NaN, and `[1, 0 / 0, 1].mode()` is `1`.
- Records match on their contents in any key order, and instances only
  themselves. The quirks of deep equality carry over exactly: after
  `a = {k: null, m: 1}` and `b = {k: {}.none, m: 1, z: 5}`, `[a, b].mode()`
  is `a`, which `b` repeats, while `[b, a].mode()` is `b`, because
  `[a].includes(b)` is `true` and `[b].includes(a)` is `false`. An item that
  matches several kept values counts only for the earliest. A list nested
  more than 64 levels deep matches nothing, not even itself, while an object
  always matches itself.
- A comparison reads the objects it compares without subscribing to their
  properties. A declaration that uses `mode` follows its list or population,
  its argument expressions and what the selector reads; changing a property
  of a compared object does not re-evaluate it.
- `mode()` reads items, not bindings, so a `null` item is counted without
  settling a declaration to `null`, while the reads inside a selector follow
  the existing rules.
- The receiver is a list, however it is reached, or a class's instance list in
  population order, named by the class or by a variable that holds the class.
  On a class name, `mode` is the population query even when the class
  declares an instance method `mode`; its instances keep calling that method.
- A selector is called once for every item, in order, with only the item;
  further parameters are `null`. A supplied first argument that is not a
  function, including `null`, raises `TypeError("mode expects a function")`,
  even on an empty receiver.
- `mode` follows the shared rules of the list queries: it reads its receiver
  before its arguments, evaluates every supplied argument once, left to
  right, including unused extras, and never writes the receiver. List
  mutations, new and deleted instances and changes to properties that a
  selector reads re-evaluate a declaration that uses it, in class-level rules
  too. Selector errors roll back through the existing transaction. A null or
  undefined receiver yields undefined without evaluating the arguments.
- Each value is compared with deep equality only against kept values that
  share its structural hash, and, for an object holding a property whose value
  is undefined, against the kept objects that hold a `null` property and as
  many defined properties. These candidates are compared in receiver order
  until the first match. A call makes at most one comparison per pair of a
  value and a kept value, then reads the count of each kept value once.
- Property reads such as `xs.mode`, bindings and object properties named
  `mode`, and instance methods named `mode` called on an instance keep their
  accepted results. The eight appended entries beginning
  `Nucleoid finds the most frequent list item with mode` through
  `Nucleoid rolls back failed mode updates` specify the executable behavior.

## Noun list queries with of

- Revision 2 of [NUC 75](docs/nuc-0075.md) adds the prefix operator
  `NOUN of OPERAND` for the four list queries named by nouns: `sum`, `max`,
  `min` and `mode`. Queries named by verbs keep the method call, such as
  `xs.sort()` and `xs.push(x)`. `of` is already a keyword; no token, keyword
  or error type is added.
- `NOUN of OPERAND` means exactly the method call `OPERAND.NOUN()`. It accepts
  the same receivers, returns the same results, raises the same errors and
  evaluates the receiver once through the same route; a null or undefined
  receiver yields undefined. Declarations and class-level rules that use it
  depend on what the method call depends on, stay live the same way and roll
  back through the existing transaction.
- The noun always names the query, never a binding: `sum of xs` reads no
  variable, function or parameter named `sum`. The receiver decides as for the
  method call, so a record's function property or an instance's method named
  like the noun answers, and on a class name the population query answers.
- `NOUN of` is a prefix operator at the level of `-`, `not` and `typeof`. Its
  operand is a unary expression with its whole postfix chain:
  `sum of xs.map(f)` is `sum of (xs.map(f))`, and `sum of xs + 1` is
  `(sum of xs) + 1`. Like the other prefix operators it applies to a whole
  power: `sum of xs ** 2` is `sum of (xs ** 2)`, and `(sum of xs) ** 2` squares
  the total. `sum of max of xs` nests.
- A selector stays an argument of the method call, as in `xs.max(selector)`.
  For every function selector, `NOUN of xs.map(selector)` equals
  `xs.NOUN(selector)`.
- In a `for` header, `NAME of` still names the loop variable:
  `for mode of rows:` loops over `rows` with `mode` bound, and an iterable may
  itself be a query, as in `for item of mode of groups:`. At the top level of a
  template interpolation, which ignores trailing tokens, `${sum of xs}` still
  reads the name `sum`, while `${(sum of xs)}` reads the query.
- The query renders as written in graph keys, `why` rules and errors that
  quote an expression, so `top = max of prices` explains as
  `top = max of prices`. It is parenthesized where its reading would otherwise
  change: as the base of `**` or of a postfix chain, as in
  `(mode of words).length`, and as a whole template interpolation.
- The nine appended entries beginning
  `Nucleoid reads noun list queries with of` through
  `Nucleoid reads noun queries of snapshots versions and other receivers`
  specify the executable behavior.

## List and population extreme items

- [NUC 77](docs/nuc-0077.md) adds the read-only list queries `maxBy` and
  `minBy` to lists and class instance lists. No syntax, keyword or error type
  is added.
- `RECEIVER.maxBy(selector)` returns the first item of the receiver whose
  `selector(item)` is the largest, and `RECEIVER.minBy(selector)` the first
  item whose `selector(item)` is the smallest. The result is the item itself,
  unchanged: an object or instance stays the same object, and the selected
  key is not returned.
- Keys compare as `>` and `<` compare them: two strings as text, and every
  other pair as numbers with the numeric conversion of the comparisons, so
  dates compare by time and `true`, `false`, `null`, empty lists and one-item
  lists count as numbers. `["10", "9"].maxBy(code -> code)` is `"9"`, while
  `["10", "9"].max()` is `10`.
- The items are scanned in receiver order. The first item is kept, and a
  later item replaces the kept one only when its key is greater for `maxBy`,
  or smaller for `minBy`, so on a tie the item that comes first wins. A
  string that is not numeric is neither greater nor smaller than a number, so
  where such strings and other keys mix, the kept item stays.
- A key that cannot be compared with itself, one for which `key >= key` is
  `false`, makes the result undefined, which a declaration stores as `null`:
  undefined, NaN, objects, instances, classes, functions, regular expressions
  and lists that convert to NaN. Every item is still visited. An empty
  receiver gives undefined.
- When every key can be compared with itself, `xs.maxBy(f)` is the item that
  `xs.reduce((best, item) -> f(item) > f(best) ? item : best)` returns, and
  `xs.minBy(f)` the item that the same `reduce` with `<` returns, except that
  `f` runs once per item.
- When the keys are all strings, or none is a string, `xs.maxBy(f)` is the
  item that `xs.find(x -> all y in xs: f(x) >= f(y))` finds, and undefined
  where it finds none; `xs.minBy(f)` corresponds to the same `find` with
  `<=`.
- For keys that are not strings and a receiver that is not empty,
  `xs.maxBy(f)` is undefined exactly when `xs.max(f)` is undefined or NaN,
  and otherwise it is the first item whose key converts to `xs.max(f)`;
  `minBy` and `min` correspond the same way.
- A `null` key compares as `0`, as in `null > -1`. Reads inside a selector
  follow the existing rules, so a selector that reads a `null` binding or
  property settles a declaration to `null`.
- The receiver is a list, however it is reached, or a class's instance list in
  population order, named by the class or by a variable that holds the class.
  On a class name, `maxBy` and `minBy` are population queries even when the
  class declares instance methods `maxBy` or `minBy`; its instances keep
  calling those methods.
- The selector is required. It is called once for every item, in order, with
  only the item; further parameters are `null`. A missing first argument, or
  one that is not a function, including `null`, raises
  `TypeError("maxBy expects a function")` or
  `TypeError("minBy expects a function")`, even on an empty receiver.
- `maxBy` and `minBy` follow the shared rules of the list queries: they read
  the receiver before their arguments, evaluate every supplied argument once,
  left to right, including unused extras, and never write the receiver. A
  declaration that uses one depends on the list or population, on its
  argument expressions and on whatever the selector reads for every item, so
  list mutations, new and deleted instances and property changes re-evaluate
  it, in class-level rules too. Selector errors roll back through the
  existing transaction. A null or undefined receiver yields undefined without
  evaluating the arguments.
- A call selects each key once and compares it once with itself and at most
  once with the kept key, in one pass that neither sorts nor copies the
  receiver.
- Property reads such as `xs.maxBy`, bindings and object properties named
  `maxBy` or `minBy`, and instance methods named `maxBy` or `minBy` called on
  an instance keep their accepted results. The eight appended entries
  beginning `Nucleoid finds the extreme items of lists with maxBy and minBy`
  through `Nucleoid rolls back failed extreme item updates` specify the
  executable behavior.

## List and population averages

- [NUC 80](docs/nuc-0080.md) adds the read-only list query `average` to lists
  and class instance lists. No syntax, keyword or error type is added.
- `RECEIVER.average()` returns the arithmetic mean of the receiver's items and
  `RECEIVER.average(selector)` the mean of `selector(item)` for every item.
  The result is exactly `RECEIVER.sum(...) / RECEIVER.length` with the same
  arguments: the total of `sum`, divided by the number of items with the
  division of `/`, in 64-bit floating point. A list of numbers therefore
  averages to exactly what `xs.sum() / xs.length` gives, so
  `[0.1, 0.2, 0.3].average()` is `0.20000000000000004`.
- An empty receiver gives `0 / 0`, which is NaN. The result is a number, or
  undefined when any added value is undefined, which a declaration stores as
  `null`, because `/` gives undefined when an operand is undefined.
- Every item counts toward the number of items, including items that convert
  to `0` or NaN, so `[2, null].average()` is `1`.
- The receivers, the numeric conversion, the selector and the liveness are
  those of `sum`: a list however it is reached, or a class's instance list in
  population order, named by the class or by a variable that holds the class;
  `null`, `false`, empty strings and empty lists count `0`, `true` counts `1`,
  numeric strings and one-item lists count their number, and objects, other
  strings and longer lists are NaN. On a class name, `average` is the
  population query even when the class declares an instance method `average`;
  its instances keep calling that method.
- `average()` reads items, not bindings, so a `null` item counts `0` without
  settling a declaration to `null`, while the reads inside a selector follow
  the existing rules.
- A selector is called once for every item, in order, with only the item;
  further parameters are `null`. A supplied first argument that is not a
  function, including `null`, raises `TypeError("average expects a function")`,
  even on an empty receiver.
- `average` follows the shared rules of the list queries: it reads its
  receiver before its arguments, evaluates every supplied argument once, left
  to right, including unused extras, and never writes the receiver. A
  declaration that uses it depends on the list or population, on its argument
  expressions and on whatever the selector reads, so list mutations, new and
  deleted instances and property changes re-evaluate it, in class-level rules
  too. Selector errors roll back through the existing transaction. A null or
  undefined receiver yields undefined without evaluating the arguments.
- The call keeps its ordinary rendering, so `mean = temps.average()` explains
  as `mean = temps.average()`, and `why` lists the receiver among the
  declaration's dependencies.
- `average of xs` stays `SyntaxError("Unexpected keyword 'of'")`: `average`
  is not one of the noun queries. Property reads such as `xs.average`,
  bindings and object properties named `average`, `average(...)` without a
  definition, `Math.average` and instance methods named `average` called on
  an instance keep their accepted results. The eight appended
  entries beginning `Nucleoid averages lists with average` through
  `Nucleoid rolls back failed average updates` specify the executable
  behavior.

## List and population medians

- [NUC 84](docs/nuc-0084.md) adds the read-only list query `median` to lists
  and class instance lists. No syntax, keyword or error type is added.
- `RECEIVER.median()` returns the middle value of the receiver's items in
  ascending numeric order and `RECEIVER.median(selector)` the middle value of
  `selector(item)` for every item. With an odd number of values it is the
  middle value itself, unchanged. With an even number it is the mean of the
  two middle values, `(lower + upper) / 2` in 64-bit floating point, so
  `[4, 1, 3, 2].median()` is `2.5`.
- The values are converted to numbers and ordered by a stable ascending sort,
  so equal values such as `0` and `-0` keep their receiver order. The receiver
  is never sorted or written: after `waits = [12, 3, 45]`, `waits.median()` is
  `12` and `waits` keeps its order.
- An empty receiver gives NaN. A value that converts to NaN makes the median
  NaN, so `[1, "a", 3].median()` is NaN. An undefined value makes the median
  undefined, which a declaration stores as `null`; every item is still visited.
- The receivers, the numeric conversion and the selector are those of `sum`
  and `average`: a list however it is reached, or a class's instance list in
  population order, named by the class or by a variable that holds the class;
  `null`, `false`, empty strings and empty lists are `0`, `true` is `1`,
  numeric strings and one-item lists are their number, and objects, other
  strings and longer lists are NaN. On a class name, `median` is the
  population query even when the class declares an instance method `median`;
  its instances keep calling that method.
- A selector is called once for every item, in order, with only the item;
  further parameters are `null`. A supplied first argument that is not a
  function, including `null`, raises `TypeError("median expects a function")`,
  even on an empty receiver.
- `median` follows the shared rules of the list queries: it reads its receiver
  before its arguments, evaluates every supplied argument once, left to right,
  including unused extras, and never writes the receiver. A declaration that
  uses it depends on the list or population, on its argument expressions and
  on whatever the selector reads, so list mutations, new and deleted instances
  and property changes re-evaluate it, in class-level rules too. Selector
  errors roll back through the existing transaction. A null or undefined
  receiver yields undefined without evaluating the arguments.
- A call converts every value once and sorts one list of numbers, in
  O(n log n) time, without recursion.
- The call keeps its ordinary rendering, so `typical = waits.median()`
  explains as `typical = waits.median()`, and `why` lists the receiver among
  the declaration's dependencies.
- `median of xs` stays `SyntaxError("Unexpected keyword 'of'")`: `median` is
  not one of the noun queries. Property reads such as `xs.median`, bindings
  and object properties named `median`, `median(...)` without a definition,
  `Math.median` and instance methods named `median` called on an instance
  keep their accepted results. The eight appended entries beginning
  `Nucleoid finds the middle values of lists with median` through
  `Nucleoid rolls back failed median updates` specify the executable
  behavior.

## Object entry loops with ordered rollback

- [NUC 72](docs/nuc-0072.md) adds loops over an object's entries:
  `for KEY, VALUE of SOURCE:` and `for KEY, VALUE in SOURCE:`, and makes
  rollback restore the order of every object's entries. No keyword, token or
  error type is added.
- A header is a two-name header exactly when `for` is followed by a name, a
  comma, a name and `of` or `in`. `SOURCE` is any expression, and the body is
  any suite the single-name loop accepts: an indented block, statements on
  the same line separated by `;`, or a braced block after the colon. There is
  no colon-free, implicit, parenthesized or three-name form.
- `SOURCE` is evaluated once, before the names are bound, and must be an
  object: an object literal, an instance, or any other object value. Every
  other value raises `TypeError("<source> is not an object")`, with the source
  in canonical form, for example `TypeError("xs is not an object")` for a list
  named `xs`. An undefined name raises `ReferenceError("<name> is not defined")`
  as before. Both are positioned at the running top-level statement.
- The loop visits the object's own entries in insertion order: an object
  literal's keys as written, later properties appended when they are first
  assigned, deleted properties removed, and a property deleted and assigned
  again moved to the end. An instance lists `id` first, then the properties
  its constructor assigns, the properties its class-level rules derive and
  the properties assigned later, in the order they were first stored.
- An entry whose value is `null` is visited. A literal slot whose value is
  undefined, such as `a` in `{a: none.x}` when `none.x` was never set, is not
  an entry, as deep equality and JSON serialization already skip it. An empty
  object runs no body.
- Every entry, key and value, is snapshotted before the first body runs.
  Entries the body adds are not visited, entries it deletes are still visited
  with their snapshotted values, and an entry whose value it changes is still
  bound to the snapshotted value, while `SOURCE[KEY]` reads the current one.
  A later loop takes a fresh snapshot.
- Each iteration pushes a scope, declares `KEY` as the entry's key, a string,
  and `VALUE` as its value, and runs the body imperatively, as a single-name
  loop does. The names shadow and restore outer names, and when both names
  are the same, the value is bound.
- Explicit and bare returns leave the loop as from a single-name loop; a
  normally completed or empty loop yields `null`. Errors and thrown values
  roll back through the existing transaction.
- Taking the snapshot records no dependency of its own, as the class, list and
  string snapshots do; evaluating `SOURCE` records its reads as usual. A loop
  inside a standing `if` branch, a block or a function they call runs whenever
  the branch or block runs. It does not run again because an entry changed or
  was added, and it does when the binding that `SOURCE` reads is replaced.
  Reads made by the body, such as `SOURCE[KEY]`, are dependencies as usual.
- Loops run in functions and methods, where they see parameters and `this`
  as single-name loops do, in constructors, class-level rules, blocks,
  `suppose` suites and `try` bodies, and they nest. Inside a class-level `if`
  or block, a source that reads `$Class`, such as `$Box.parts`, fails when the
  block is declared, with `TypeError("$Box.parts is not an object")`, as the
  single-name loop fails with `TypeError("$Box.parts is not iterable")`.
- Rollback restores the order of every object's entries along with their
  values. When a transaction or a savepoint rolls back, through a caught
  failure in `try`, a failed top-level statement, the end of a `suppose` block
  or an error inside one, every object holds exactly the entries it held when
  the rolled-back work began, in the same order: a property the work deleted
  returns at the position it held, a property it added is removed, a property
  it overwrote keeps its position and regains its value, and a property it
  deleted and assigned again returns to its original position with its
  original value. An inner savepoint restores the order its own work changed,
  and an enclosing rollback restores the rest; instances keep `id` first.
- Rollback keeps its accepted behavior for everything else it restores,
  including the order of graph nodes and dependency edges that `model |> why`
  reflects. Before entry loops, the order of an object's entries had no
  observable channel, so restoring it changes no accepted program.
- Graph and block keys render the header as `for KEY,VALUE of SOURCE`, as they
  render `for NAME of SOURCE`, and default-bearing declarations render the
  suite form `for KEY,VALUE of SOURCE:` with its indented body. `in` renders as
  `of`. The suite form parses back to the same statement.
- Single-name loops are unchanged, including `TypeError("<name> is not
  iterable")` for objects and every other noniterable source. Inputs that are
  not two-name headers keep their diagnostics: `for values, other:` raises
  `SyntaxError("Expected 'of' in for statement")` at line 1, column 11, and
  `for t := "ab": pass` the same message at line 1, column 7. A malformed
  two-name header reports its error where it occurs, as a malformed
  single-name header does.
- The eight appended entries beginning
  `Nucleoid loops over object entries in insertion order` through
  `Nucleoid restores object entries in place when it rolls back` specify the
  executable behavior.

## Object spread entries

- [NUC 81](docs/nuc-0081.md) adds spread entries to object literals:
  `{...SOURCE, key: value}` copies the entries of the object `SOURCE` holds
  into the new object. No keyword, token or error type is added.
- A spread entry is three dots at consecutive columns of one line, followed
  by one expression, its source. It may begin wherever a key may: first in
  the braces or after a comma, with the line breaks a key allows before it.
  The source is parsed as an entry's value is, so it ends at a comma, at the
  closing brace, or before a following keyed entry written without a comma,
  as in `{...base qty: 1}`. Keyed, shorthand and spread entries mix in any
  order, and one trailing comma is allowed: `{tag, ...base stock: 0,}`.
- A spread that follows a value or another spread needs a comma before it:
  `{y: 2 ...a}` keeps `SyntaxError("Expected a property but found '.'")`,
  because the value reads the dots as a member access. Shorthand entries keep
  needing a comma or the closing brace after them.
- Spreads are entries of object literals written as expressions, in
  assignments, arguments, list items, returns, named-arrow and `lambda`
  bodies, parentheses, template interpolations, nested literals,
  constructors, methods, class-level rules and every suite. A brace that
  begins a statement is still a block, so `{...a, y: 2}` as a statement keeps
  `SyntaxError("Unexpected '.'")`, and an anonymous arrow body in braces is
  still a block; `({...a})`, `lambda p: {...p}` and `def f p => {...p}` write
  objects. `[...xs]`, `f(...xs)` and `{x, ...rest} = p` keep their syntax
  errors.
- The entries are evaluated once each, from left to right. A keyed entry
  stores its value. A spread evaluates its source once and copies the
  source's entries in their insertion order, the entries an entry loop
  visits: an instance's `id` first, then its other properties in the order
  they were stored; every property whose value is not undefined, `null`
  values included. A slot whose value is undefined is not copied. The result
  is a new object that belongs to no class, and a copied object or list is
  the same value in both records.
- An entry for a key the object already holds replaces its value and keeps
  the key's first position, as a repeated key does: with
  `point = {x: 1, y: 2}`, `{...point, x: 5}` lists `x` then `y` with `x` 5,
  and `{x: 9, ...point}` takes `x` from `point`.
- Copying an entry is not a read of a binding, so a copied `null` does not
  settle a declaration to `null`. The source's own reads follow the existing
  rules: after `owner = null`, `card = {...owner, seen: true}` is `null`,
  and it becomes a record when `owner` holds one. An undefined source, such
  as a property never set, copies nothing, and the literal `null` copies
  nothing. An undefined name raises `ReferenceError("<name> is not defined")`
  as before.
- Every other source raises `TypeError("<source> is not an object")`, with the
  source in canonical form, as entry loops do: a list, a string, a number, a
  boolean, a class outside a class-level rule, a function, a date and a
  regular expression. After `sizes = [1, 2]`, `{...sizes}` raises
  `TypeError("sizes is not an object")`. The error is positioned at the
  running top-level statement and rolls back its transaction.
- While a property declaration evaluates its value, a spread of the object
  that owns the declared property leaves that property out:
  `cart.copy = {...cart}` copies every other entry of `cart`, and the
  class-level rule `$Order.view = {...$Order}` every other entry of each
  order. This holds in
  every suite, in constructors, class-level rules and value assignments, and
  in the functions they call. An assignment used as an expression evaluates
  its value before its target and copies every entry.
- A declaration that spreads stays live. Its source's reads are dependencies
  as usual. It also depends on every entry it copies, except an entry that a
  later keyed entry of the same literal replaces, and it observes the set of
  entries of the source object. Changing a copied entry, deleting an entry,
  adding an entry and replacing the binding the source reads each evaluate it
  again, so a derived record stays equal to the explicit record that names
  every entry its source holds. Evaluating it again creates a new object, as
  every object literal declaration does. `{...a.value}` follows the entries
  of the object `a` held when the declaration was stored, and `b := {...a}`
  copies once.
- Entry-set observations are not graph nodes or value dependencies: `why`
  lists the same dependencies as the explicit record, `affects`, `model` and
  the reasoning stages are unchanged, and a declaration that spreads is an
  ordinary declaration, not a reasoning observer. The `--graph` listing shows
  each observation under `observes` as `{...<object id>}`. Programs without
  spreads record none and propagate exactly as before. Two class-level rules
  that spread the same receiver read each other and raise
  `TypeError("Circular Dependency")` when an instance applies them.
- Rollback restores spread records and observations with everything else it
  restores: a caught failure in `try`, a failed top-level statement, the end
  of a `suppose` block and an error inside one leave every record and every
  object's entries as they were, in order.
- Graph and block keys, conditions and `why` rules render a spread as `...`
  followed by its source between the quoted keyed entries, such as
  `view = {...lab,"unit":"C"}`. Each source renders once, and the rendering
  parses back to the same literal.
- Only inputs that raised `SyntaxError("Expected a key but found '.'")` at an
  object-literal entry beginning with three adjacent dots change. The eight
  appended entries beginning
  `Nucleoid copies object entries into records with spread entries` through
  `Nucleoid rolls back failed spread updates` specify the executable
  behavior.

## While loops

- [NUC 63](docs/nuc-0063.md) adds the condition-controlled loop
  `while CONDITION:` followed by a suite: an indented block, or statements on
  the same line separated by `;`. No keyword or lexer token is added, and
  `while` stays an ordinary name everywhere else.
- A statement is a loop when it starts with the name `while` and the tokens
  after it form one complete condition immediately followed by `:`, except
  in the four spellings below. The condition is parsed like an `if` header
  condition, so conditional expressions, quantifiers, `??`, `|>`, arrows and
  bracketed lambdas pair their own colons. At the accepted revision every
  loop statement was a `SyntaxError`.
- Four spellings keep their accepted readings, because accepted programs
  use them:
  - `while NAME: TYPE` before a newline, `;`, dedent or the end of input is
    the statement `while` followed by the typed declaration `NAME: TYPE`, so
    `while ready: tick` runs `while` and then declares `ready`. Executing it
    raises `ReferenceError("while is not defined")`, or
    `ReferenceError("Missing definition")` when `while` is defined. A loop
    whose same-line body is one bare name could only read that name, so no
    useful loop is lost; `while ready: tick()` and an indented `tick` body
    are loops.
  - `while assert: ACTUAL, EXPECTED` is `while` followed by the colon-led
    assertion statement `assert: ACTUAL, EXPECTED`, so with `while = 1`,
    `while assert: while, 1` runs that assertion.
  - A condition whose last token is a bare `lambda`, as in `while lambda: 1`
    or `while x + lambda: 1`, keeps its accepted reading: `while` followed by
    an expression that holds the zero-parameter lambda `lambda: 1`.
  - A condition that is the name `suppose`, or an assignment whose value is
    the name `suppose`, keeps its accepted reading, which is usually a
    `suppose` block ([NUC 65](docs/nuc-0065.md)): `while suppose: x` is
    `while` followed by the block `suppose: x`, and `while x = suppose: x` is
    `while` followed by the assigned block `x = suppose: x`. The test reads
    the parsed condition, as the `suppose` statement does, so
    `while x = (suppose): x` is an assigned block too, and
    `while (suppose): pass` keeps its accepted `SyntaxError("Unexpected ':'")`.
- `while CONDITION` followed by an indented block without a colon, and
  `while CONDITION { ... }`, keep their accepted readings as the juxtaposed
  statements `while`, `CONDITION` and the block. A variable, function,
  parameter or loop variable named `while`, including either name of a
  two-name `for` loop, an object key or property `while`, `while(...)`
  calls, the value assignment `while := VALUE` and the declaration
  `while: TYPE` keep working.
- Before each iteration the condition is evaluated in the current state and
  tested with the truthiness of `if`. While it holds the body runs again;
  when it is falsy the loop ends. A condition that is falsy at the start runs
  no iteration. A completed loop yields `null`.
- The loop is one-shot and imperative, like a `for` loop. Its body,
  including nested conditionals, blocks and loops, is carried out rather
  than filed as standing rules. Each iteration's assignments, and the
  propagation they cause, are visible to the rest of the body and to the next
  condition test. The loop files nothing in the graph and does not run again
  when its inputs change later.
- A loop inside the body of a standing rule, such as the branch of a
  top-level `if` or a function called from a declaration, runs again whenever
  that rule runs, as a `for` loop does there.
- The body runs in the enclosing scope, like an `if` branch. A `for` body
  pushes a new scope frame for each iteration to hold its loop variable; a
  `while` loop binds no variable and pushes no frame. This is the one place
  where a `while` body differs from a `for` body. Inside a function, loop or
  block body, a `for` iteration's frame turns a name that the enclosing body
  already assigned into a local of that iteration: a function that sets
  `sum = 0` and then runs `sum = sum + 1` in a `for` loop over two items
  returns `0`, which is accepted behavior and unchanged. The same accumulator
  in a `while` body updates the enclosing `sum`, so the function returns the
  accumulated total. Parameters, enclosing loop variables, locals, `this` and
  `self`, and `return` otherwise behave as in `for` bodies.
- `return` in the body leaves the enclosing function, method, block or loop,
  as from a `for` body. At the top level it ends the program with its value.
- A `while` statement is never kept on a class. Outside a class-level rule,
  `$Class` in its condition or body reads the declared class, as in a `for`
  statement, and an undeclared `$Name` raises
  `ReferenceError("Name is not defined")`. A body statement that is itself a
  class-level declaration, such as `$Item.flag = true`, is declared on the
  class when it runs, as in a `for` body. Inside the body of a class-level
  rule the loop runs for each instance, and `$Class` reads that instance.
- `.value` in the condition reads the current value at each test; it is not
  frozen.
- Termination is guaranteed. Each run of a `while` statement may run its body
  at most 100,000 times, the same bound as propagation steps. If the
  condition still holds after the 100,000th iteration, the loop raises
  `TypeError("Loop did not terminate")` positioned at the running top-level
  statement, and the transaction rolls back. Each run of a nested loop has
  its own budget, so every run terminates, but nested loops multiply and the
  bound is not a cap on total work.
- Errors raised by the condition or the body, and thrown values, propagate
  through the existing transaction unchanged. A caught failure restores the
  state from before the enclosing `try` body.
- Canonical rendering is `while(CONDITION){BODY}` in graph keys and
  `while CONDITION:` with an indented body inside default-bearing
  declarations.
- The native REPL already buffers a line that ends in `:` until a blank
  line, and the terminal workspace's indented newline indents after one, so
  `while CONDITION:` waits for its indented body.
- Malformed loops use existing diagnostics: `while n < 3:` at the end of
  input raises `SyntaxError("Unexpected end of input")`, and an unindented
  next line raises `SyntaxError("Expected an indented block but found ...")`.
  The eight appended entries beginning
  `Nucleoid repeats a while loop until its condition fails` through
  `Nucleoid rolls back a while loop that fails during an iteration` specify
  the executable behavior.

## Destructuring declarations

- [NUC 67](docs/nuc-0067.md) adds destructuring declarations: object patterns
  `{x, y} = point`, renamed targets `{key: name} = source` and Python-style
  unpacking `a, b = pair`. No token, syntax tree node or runtime path is
  added.
- Each target becomes exactly one ordinary declaration, in pattern order. An
  object entry with key `key` and target `name` becomes `name = value.key`;
  the comma-separated target at position `i`, counted from 0, becomes
  `name = value[i]`. The value is copied into every declaration, so
  `{x, y} = point` parses to exactly the statements of `x = point.x` and
  `y = point.y`.
- Each declaration is filed or carried out as it would be when written out:
  a standing declaration at the top level; carried out in a `for` body, a
  standing `if` branch and a block; a local for a parameter or loop variable;
  a constructor statement in a class body. A destructured name stays live: a
  later change to what its declaration read updates it. A missing key or
  position reads `null`, as `value.key` and `value[i]` do.
- The value is evaluated once per target, each time that target's
  declaration runs, so a call or instantiation in it runs once per target:
  `{x, y} = Point(1, 2)` creates two points. Name such a value first, as in
  `p = Point(1, 2)` and `{x, y} = p`, to evaluate it once.
- The declarations run in pattern order and each propagates before the next,
  as consecutive statements do, so a standing rule that reads two targets can
  run between them. A name that appears twice is declared twice and the last
  declaration stands. The statement's value is the value of its last target.
- `why` explains a destructured name as its declaration, such as
  `x = point.x` or `first = pair[0]`, and `affects` lists the declarations.
  Graph and block keys render the declarations; nothing records the pattern.
- An object pattern is `{` entries `}` with at least one entry. An entry is a
  shorthand `name`, followed by `,` or `}`, or a keyed `key: name`, followed
  by `,`, `}` or directly by the next keyed entry, as in object literals. One
  trailing comma is allowed. Keys and targets are ordinary identifiers.
- A comma-separated pattern is ordinary identifiers separated by commas, with
  at least one comma and an optional trailing comma: `a, b`, `a, b,` and
  `first,`. There is no arity check, and extra positions are ignored.
- The value is one expression accepted on the right of `=` that is not
  itself an assignment.
- A destructuring is a whole statement on one line. Its pattern begins the
  input or follows a line break, `;`, an indentation change, a block's `{` or
  a suite's `:`. Its value is followed by the end of the line or input, `;`,
  a block's `}`, or `else` or `catch`. In an inline suite, as in
  `if ready: low, high = pair`, all of its declarations belong to the suite.
- Recognition is an attempt that consumes nothing unless the whole statement
  qualifies. Every other input is parsed as before and keeps its accepted
  diagnostic and position. `[a, b] = pair` still parses and raises
  `SyntaxError("Cannot assign to [a,b]")` when it runs, `{ x }` is a block,
  and inputs such as `{ amount, label }`, `{x, y} == point`, `{x} += point`,
  `a, b = 1, 2`, `x, y = y, x`, `a, b = c = pair`, `(a, b) = pair`,
  `let {x, y} = point` and `const a, b = pair`, rest, star, default, nested,
  quoted and numeric entries, multi-line patterns, and property and index
  targets keep their syntax errors.
- A destructuring names at most 128 targets. The 129th target of an otherwise
  complete destructuring raises
  `SyntaxError("Destructuring has too many targets")` at that target.
- Only inputs that were syntax errors change. Patterns whose keys or targets
  are names that other statements read specially, such as
  `{assert: a, lambda: b, suppose: c} = p` and `{x: int} = point`, now
  destructure. A complete destructuring followed by a stray `else` or `catch`
  now reports that keyword: `a, b = pair else: x = 1` raises
  `SyntaxError("Unexpected keyword 'else'")`.
- The eight appended entries beginning
  `Nucleoid destructures object fields into standing declarations` through
  `Nucleoid rolls back failed destructuring` specify the executable behavior.

## Enum declarations

- [NUC 90](docs/nuc-0090.md) adds enum declarations, the categorical types
  that TypeScript, Java, C#, Kotlin, Swift, Rust and Python models write:
  `enum Status: PENDING, PAID, SHIPPED` on one line, or `enum Status:`
  followed by an indented block with one member per line. `enum` stays an
  ordinary name, and no token is added.
- A member is a name, optionally followed by `=` and a scalar literal: a
  finite number with an optional `-` or `+` sign, a string, `true`, `false`
  or `null`, the literals of literal parameter defaults. A member without a
  value holds its own name as a string, so `enum Status: PENDING, PAID`
  holds `{PENDING: "PENDING", PAID: "PAID"}` and
  `enum Level: LOW = 1, HIGH = 3` holds `{LOW: 1, HIGH: 3}`.
- The one-line form has at least two members separated by commas, with no
  trailing comma. Its last member is followed by the end of the line or
  input, `;`, a block's `}`, the end of an indented block, or `else` or
  `catch`. The block form ends its first line at the colon and writes each
  member on its own indented line, with at least one member; blank lines may
  separate members.
- A declaration starts wherever a statement may start: at the top level, in
  indented and same-line suites, in braced blocks and function bodies, after
  `;`, and after another statement on the same line. Inside a class body,
  including its methods and the suites in it, every `enum` spelling keeps its
  accepted reading.
- `enum NAME: ...` declares `NAME` exactly as `NAME = {...}` with the same
  members would: a standing declaration at the top level and in standing
  branches, a declaration carried out in a loop body, a local when `NAME` is
  a parameter or loop variable, and a declaration in the state when a
  function body runs it. It reads nothing, so `why` lists it as stated and no
  change re-evaluates it. Declaring the name again replaces it and
  re-evaluates its readers, and a failed transaction or a `suppose` block
  restores the earlier binding. Each run creates a new object.
- The value is an ordinary object whose properties are the members in
  declaration order. `Status.PAID` and `Status["PAID"]` read a member, a
  missing member reads `undefined` as a missing property does, `typeof`
  gives `"Object"`, and assertions, JSON, entry loops and spreads see the
  members as they see an object literal's properties. A spread such as
  `{...Status}` builds an ordinary object.
- The object is read-only. Assigning a member, adding a property or deleting
  one raises `TypeError("Enums are read-only")` through every spelling and
  every name that holds the object: property declarations, compound and value
  assignments, assignment expressions, `delete` and the list write-backs of
  `push`, `append`, `pop` and `shift`. The error is placed at the running
  top-level statement and rolls back its transaction, so the enum and its
  readers stay unchanged. A copy that a `suppose` block's value carries out
  of the block is read-only too. Binding the name to another value is not a
  write: a later declaration of `Status` replaces it, and `delete Status`
  follows the accepted rule for objects, raising
  `TypeError("Cannot delete object '<id>'")` while the object holds a
  property other than `id`.
- Canonical source writes an enum on one line, `enum Status:PENDING,PAID`
  or `enum Level:LOW=1,HIGH=3`, leaving out a value that equals the member's
  own name; an enum with one member is written `enum NAME:` with the member
  on an indented line. `why` rules show the same text, and it parses back to
  the same declaration.
- Only inputs that were syntax errors at the accepted revision change.
  `enum Status: PENDING`, one name after the colon before the end of the
  statement, keeps its accepted reading: the expression `enum` followed by
  the typed declaration `Status: PENDING`. So do `enum assert: ...`,
  `enum lambda: ...` and `enum suppose: ...`, whose colon statements parse,
  and every use of `enum` as a variable, property, key or parameter.
- A repeated member raises
  `SyntaxError("Enum '<name>' already has member '<member>'")` at the repeated
  member, and a one-line enum with a single member raises
  `SyntaxError("Enum '<name>' needs at least two members on one line")` where
  its second member should begin. A value that is not a scalar literal raises
  `SyntaxError("Expected a scalar literal enum member value")`, and a number
  that is not finite raises
  `SyntaxError("Enum member value must be a finite number")`, both at that
  token. Other malformed declarations raise
  `SyntaxError("Expected <what> but found <what>")` at the offending token.
- The eight appended entries beginning
  `Nucleoid declares enums whose members name themselves` through
  `Nucleoid keeps enum an ordinary name outside enum declarations` specify
  the executable behavior.

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
---
# Nucleoid anchors class conditionals to each instances first property values

class Obj(x: int, y: int, orientation: int):
    this.x = x
    this.y = y
    this.orientation = orientation
first = Obj(10, 20, 180)
first.x = 70
first.y = 80
if $Obj.orientation == 180:
    $Obj.tag = "l4214"
    $Obj.x = $Obj.x@first - 1
    $Obj.y = $Obj.y@first - 1
second = Obj(30, 40, 180)
waiting = Obj(50, 60, 0)
assert([first.x, first.y, first.tag], [9, 19, "l4214"])
assert([second.x, second.y, second.tag], [29, 39, "l4214"])
assert([waiting.x, waiting.y, waiting.tag], [50, 60, null])
waiting.x = 90
waiting.orientation = 180
assert([waiting.x, waiting.y, waiting.tag], [49, 59, "l4214"])
first.orientation = 0
first.orientation = 180
second.orientation = 180
assert([first.x, first.y, second.x, second.y], [9, 19, 29, 39])
assert(first.x@first, 10)
assert(first.x@0, 10)
assert(second.x@first, 30)
assert(second.x@2, null)
[first.x, first.y, second.x, second.y]
# return: [9,19,29,39]
---
# Nucleoid anchors ordinary class rules with fixed numeric and first selectors

class Counter(seed):
    this.x = seed
    this.x = seed + 10
    this.y = seed * 2
before = Counter(10)
step = 1
$Counter.x = $Counter.x@first - step
$Counter.y = $Counter["y"]@0 - step
$Counter.latest = $Counter.x@last
$Counter.prior = $Counter.x@previous
assert([before.x, before.y, before.latest, before.prior], [9, 19, 9, 20])
step = 2
assert([before.x, before.y, before.latest, before.prior], [8, 18, 8, 9])
$Counter.x = $Counter.x@1 - step
after = Counter(30)
assert([before.x, after.x, after.y], [18, 38, 58])
assert([after.x@first, after.x@0, after.x@1], [30, 30, 40])
step = 3
assert([before.x, after.x, before.y, after.y], [17, 37, 17, 57])
assert([before.latest, after.latest, before.prior, after.prior], [17, 37, 18, 38])
[before.x, after.x, before.y, after.y]
# return: [17,37,17,57]
---
# Nucleoid defers missing self anchors and recovers recreated property slots

class Pending(orientation):
    this.orientation = orientation
if $Pending.orientation == 180:
    $Pending.tag = "l4214"
    $Pending.x = $Pending.x@first - 1
item = Pending(180)
original = item.x@first
assert(item.tag, "l4214")
assert(item.x, null)
assert(original, null)
item.x = 10
assert(item.x, 9)
assert(item.x@first, 10)
assert(item.x@previous, 10)
assert(original, 10)
delete item.x
assert(item.x, null)
assert(item.x@first, null)
assert(original, null)
item.x = 40
assert(item.x, 39)
assert(item.x@first, 40)
assert(item.x@0, 40)
assert(original, 40)
item.orientation = 180
assert(item.x, 39)
assert(item.x@2, null)
[item.x, item.x@first, original]
# return: [39,40,40]
---
# Nucleoid waits for actual history entries before using numeric self anchors

class Sample(seed):
    this.x = seed
    this.enabled = true
if $Sample.enabled:
    $Sample.x = $Sample.x@1 - 1
item = Sample(10)
assert(item.x, 10)
assert(item.x@first, 10)
assert(item.x@1, null)
item.enabled = true
assert(item.x, 10)
assert(item.x@1, null)
item.x = 20
assert(item.x, 19)
assert(item.x@first, 10)
assert(item.x@1, 20)
item.enabled = false
item.enabled = true
assert(item.x, 19)
assert(item.x@3, null)
---
# Nucleoid anchors inherited class receivers independently

class Base(seed):
    this.x = seed
    this.orientation = 180
class Child: Base
    def init(seed):
        super(seed)
step = 1
$Base.x = $Base.x@0 - step
if $Base.orientation == 180:
    $Base.tag = "l4214"
    $Base.x = $Base.x@first - step
parent = Base(10)
child = Child(30)
assert([parent.x, child.x], [9, 29])
assert([parent.tag, child.tag], ["l4214", "l4214"])
assert([parent.x@first, child.x@first], [10, 30])
step = 2
assert([parent.x, child.x], [8, 28])
child.orientation = 0
child.orientation = 180
assert(child.x, 28)
assert(child.x@3, null)
---
# Nucleoid restores class anchors histories and declarations on caught rollback

class Sample(seed):
    this.x = seed
    this.enabled = true
if $Sample.enabled:
    $Sample.x = $Sample.x@first - 1
item = Sample(10)
original = item.x@first
try:
    delete item.x
    item.x = 50
    $Sample.extra = $Sample.x@first
    throw "RESTORE_ANCHOR"
catch error:
    assert(error, "RESTORE_ANCHOR")
assert(item.x, 9)
assert(item.x@first, 10)
assert(item.x@previous, 10)
assert(item.x@2, null)
assert(item.extra, null)
assert(original, 10)
item.enabled = true
assert(item.x, 9)
delete item.x
item.x = 30
assert(item.x, 29)
assert(item.x@first, 30)
assert(original, 30)
[item.x, original]
# return: [29,30]
---
# Nucleoid protects direct class history cycles and read only destinations

class Sample:
    this.x = 10
    this.items = [1, 2]
    this.enabled = true
item = Sample()
try:
    $Sample.x = $Sample.x@last - 1
catch error:
    assert(error, TypeError("Circular Dependency"))
try:
    $Sample.x = $Sample.x@previous - 1
catch error:
    assert(error, TypeError("Circular Dependency"))
try:
    $Sample.x = $Sample.x@first + $Sample.x
catch error:
    assert(error, TypeError("Circular Dependency"))
try:
    $Sample.x@first = 4
catch error:
    assert(error, TypeError("State versions are read-only"))
try:
    delete $Sample.x@first
catch error:
    assert(error, TypeError("State versions are read-only"))
try:
    if $Sample.enabled:
        $Sample.items@first.push(3)
catch error:
    assert(error, TypeError("State versions are read-only"))
try:
    $Sample.bad = Math.PI@first
catch error:
    assert(error, TypeError("State versions require a variable or object property"))
class Empty
$Empty.x = $Empty.x@first.value
try:
    absent = Empty()
catch error:
    assert(error, ReferenceError("$Empty.x@first is not defined"))
assert(item.x, 10)
assert(item.x@first, 10)
assert(item.x@1, null)
assert(item.items, [1, 2])
assert(item.items@1, null)
---
# Nucleoid preserves moving class conditionals beside fixed first anchors

class Counter(seed):
    this.x = seed
    this.enabled = false
if $Counter.enabled:
    $Counter.x = $Counter.x@last - 1
item = Counter(10)
item.enabled = true
assert(item.x, 9)
item.enabled = false
item.enabled = true
assert(item.x, 8)
assert(item.x@first, 10)
assert(item.x@previous, 9)
class Anchor(seed):
    this.x = seed
    this.y = seed
    this.enabled = false
if $Anchor.enabled:
    $Anchor.x = $Anchor.x@first - 1
    $Anchor.y = $Anchor.y@0 - 1
anchored = Anchor(10)
anchored.enabled = true
assert([anchored.x, anchored.y], [9, 9])
anchored.enabled = false
anchored.enabled = true
assert([anchored.x, anchored.y], [9, 9])
assert([anchored.x@first, anchored.y@first], [10, 10])
assert([anchored.x@2, anchored.y@2], [null, null])
class Previous(seed):
    this.x = seed
    this.x = seed + 10
    this.enabled = false
if $Previous.enabled:
    $Previous.x = $Previous.x@previous - 1
previous = Previous(10)
assert(previous.x@previous, 10)
previous.enabled = true
assert(previous.x, 9)
assert(previous.x@previous, 20)
previous.enabled = false
previous.enabled = true
assert(previous.x, 19)
assert(previous.x@first, 10)
assert(previous.x@1, 20)
assert(previous.x@2, 9)
assert(previous.x@previous, 9)
assert(previous.x@3, 19)
assert(previous.x@4, null)
[item.x, anchored.x, anchored.y, previous.x, previous.x@previous]
# return: [8,9,9,19,9]
---
# Nucleoid keeps base two exponentiation reactive

power = 2
converted = Math.exp2(power)
assert(converted, 4)
power = 3
assert(converted, 8)
power = -2
assert(converted, 0.25)
power = 0
assert(converted, 1)
power = -0
assert(converted, 1)
power = 4
assert(converted, 16)
converted
# return: 16
---
# Nucleoid preserves base two exponential coercion and omissions

text = " 3 "
converted = Math.exp2(text)
assert(converted, 8)
text = true
assert(converted, 2)
text = false
assert(converted, 1)
text = []
assert(converted, 1)
text = ["-1"]
assert(converted, 0.5)
text = Date(2)
assert(converted, 4)
text = [1, 2]
assert(Number.isNaN(converted), true)
text = "invalid"
assert(Number.isNaN(converted), true)
literal = Math.exp2(null)
assert(literal, 1)
omitted = Math.exp2()
assert(Number.isNaN(omitted), true)
---
# Nucleoid preserves base two exponential extremes and underflow

power = 1023
converted = Math.exp2(power)
assert(Number.isFinite(converted), true)
power = 1024
assert(converted, Number.POSITIVE_INFINITY)
power = -1074
assert(converted, 5e-324)
power = -1075
assert(converted, 0)
assert(1 / converted, Number.POSITIVE_INFINITY)
power = Number.POSITIVE_INFINITY
assert(converted, Number.POSITIVE_INFINITY)
power = Number.NEGATIVE_INFINITY
assert(converted, 0)
power = Number.NaN
assert(Number.isNaN(converted), true)
---
# Nucleoid applies base two exponentiation to independent class rules

class Exponent(power):
    this.power = power
$Exponent.converted = Math.exp2($Exponent.power)
first = Exponent(2)
second = Exponent(-1)
assert(first.converted, 4)
assert(second.converted, 0.5)
first.power = 3
assert(first.converted, 8)
assert(second.converted, 0.5)
delete first.power
assert(first.converted, null)
first.power = -2
assert(first.converted, 0.25)
second.power = 4
assert(second.converted, 16)
---
# Nucleoid restores base two exponential relationships after failed transactions

class Dial
dial = Dial()
dial.power = 2
converted = Math.exp2(dial.power)
failure = ""
try:
    dial.power = 3
    throw "EXP2_ROLLBACK"
catch error:
    failure = error
assert(failure, "EXP2_ROLLBACK")
assert(dial.power, 2)
assert(converted, 4)
dial.power = -1
assert(converted, 0.5)
---
# Nucleoid preserves ignored exponential reads and missing extras

class Probe
probe = Probe()
probe.power = 2
extra = 9
converted = Math.exp2(probe.power, extra)
withMissing = Math.exp2(probe.power, probe.missing)
assert(converted, 4)
assert(withMissing, null)
probe.missing = 7
assert(withMissing, 4)
extra = "ignored"
assert(converted, 4)
delete probe.power
assert(converted, null)
assert(withMissing, null)
probe.power = -1
assert(converted, 0.5)
assert(withMissing, 0.5)
delete probe.missing
assert(withMissing, null)
probe.missing = 2
assert(withMissing, 0.5)
---
# Nucleoid accepts thin arrows wherever fat arrows are accepted

offset = 1
items = [1, 2, 3]
doubled = items.map(item -> item * 2)
large = items.filter((item: int) -> item > offset)
found = items.find(item -> { return item == 2 })
def shift(value) -> value + offset
def double amount -> amount * 2
def increment amount: int -> amount + 1
def area width, height -> width * height
def mixed left: int, right -> left - right
def scaled left right factor -> (left + right) * factor
def tripled value ->
    value * 3
assert(doubled, [2, 4, 6])
assert(large, [2, 3])
assert(found, 2)
assert(shift(2), 3)
assert(double(4), 8)
assert(increment(4), 5)
assert(area(3, 4), 12)
assert(mixed(5, 2), 3)
assert(scaled(1, 2, 3), 9)
assert(tripled(2), 6)
offset = 2
assert(large, [3])
assert(shift(2), 4)
---
# Nucleoid keeps thin arrow methods and class queries reactive

class Gauge:
    def init amount -> this.amount = amount
    def reading() -> self.amount
    def scaled factor -> self.amount * factor
gauge = Gauge(4)
assert(gauge.reading(), 4)
assert(gauge.scaled(3), 12)
class Reading(level: int):
    this.level = level
Reading(4); Reading(9); Reading(15)
threshold = 5
high = Reading.filter(reading -> reading.level > threshold)
assert(high.length, 2)
threshold = 10
assert(high.length, 1)
Reading(20)
assert(high.length, 2)
---
# Nucleoid defines Python style lambdas

scale = 3
items = [1, 2, 3]
tripled = items.map(lambda item: item * scale)
total = items.reduce(lambda sum, item: sum + item, 0)
zero = lambda: 0
add = lambda left, right: left + right
pair = lambda left, right: {left: left, right: right}
assert(tripled, [3, 6, 9])
assert(total, 6)
assert(zero(), 0)
assert(add(2, 3), 5)
assert(pair(1, 2).right, 2)
assert((lambda value: value * 2)(4), 8)
scale = 2
assert(tripled, [2, 4, 6])
items.push(4)
assert(tripled, [2, 4, 6, 8])
assert(total, 10)
---
# Nucleoid keeps lambda an ordinary name where accepted syntax owns the colon

lambda = 1
values = [10, 20, 30]
assert(values[lambda:], [20, 30])
record = { first: lambda next: 2 }
assert(record.first, 1)
assert(record.next, 2)
keyed = { lambda: 5 }
assert(keyed.lambda, 5)
short = { lambda }
assert(short.lambda, 1)
if lambda:
    state = "on"
assert(state, "on")
alias = lambda assert: alias, 1
double = lambda -> lambda * 2
assert(double(4), 8)
lambda = 2
assert(values[lambda:], [30])
assert(record.first, 2)
assert(alias, 2)
---
# Nucleoid rolls back failed lambda and thin arrow updates

rate = 2
items = [1, 2]
scaled = items.map(lambda item: item * rate)
def fee amount -> amount * rate
charge = fee(10)
failure = ""
try:
    rate = 3
    throw "ROLLBACK"
catch error:
    failure = error
assert(failure, "ROLLBACK")
assert(rate, 2)
assert(scaled, [2, 4])
assert(charge, 20)
rate = 5
assert(scaled, [5, 10])
assert(charge, 50)
---
# Nucleoid sorts a list in place like JavaScript

scores = [30, 10, 20]
first = scores[0]
scores.sort((a, b) => a - b)
assert(scores, [10, 20, 30])
assert(first, 10)
assert(scores.sort((a, b) => b - a), [30, 20, 10])
assert(scores, [30, 20, 10])
assert(first, 30)
codes = [10, 9, 1]
codes.sort()
assert(codes, [1, 10, 9])
names = ["b", "a", "C"]
names.sort(null)
assert(names, ["C", "a", "b"])
people = [{ name: "a", age: 30 }, { name: "b", age: 20 }, { name: "c", age: 30 }, { name: "d", age: 20 }]
people.sort((x, y) => x.age - y.age)
assert(people.map(person => person.name), ["b", "d", "a", "c"])
scores
# return: [30,20,10]
---
# Nucleoid sorts properties and instance members in place

board = { entries: [3, 1, 2] }
board.entries.sort((a, b) => b - a)
assert(board.entries, [3, 2, 1])
calls = []
def pick():
    calls.push(1)
    return board
pick().entries.sort()
assert(calls.length, 1)
assert(board.entries, [1, 2, 3])
class Team:
    def init(scores) => this.scores = scores
    def ranked() => self.scores.sort((a, b) => b - a)
team = Team([2, 9, 4])
assert(team.ranked(), [9, 4, 2])
assert(team.scores, [9, 4, 2])
class Squad(scores):
    this.scores = scores
$Squad.best = $Squad.scores.sort((a, b) => b - a)[0]
red = Squad([3, 1, 2])
blue = Squad([5, 9])
assert(red.scores, [3, 2, 1])
assert(blue.scores, [9, 5])
red.scores.push(7)
assert(red.scores, [7, 3, 2, 1])
assert(red.best, 7)
older = { items: [3, 1, 2] }
newer = { items: [9, 8] }
picked = older
picked = newer
picked@first.items.sort()
assert(older.items, [1, 2, 3])
assert(newer.items, [9, 8])
---
# Nucleoid returns sorted copies from receivers without a list place

scores = [30, 10, 20]
ranked = scores.slice().sort((a, b) => a - b)
assert(ranked, [10, 20, 30])
assert(scores, [30, 10, 20])
assert([3, 1, 2].sort(), [1, 2, 3])
grid = [[3, 1], [2, 0]]
assert(grid[0].sort(), [1, 3])
assert(grid, [[3, 1], [2, 0]])
record = { items: [2, 1] }
assert(record["items"].sort(), [1, 2])
assert(record.items, [2, 1])
snapshot = scores.value.sort()
assert(snapshot, [10, 20, 30])
scores = [5, 4]
oldest = scores@first.sort()
assert(oldest, [10, 20, 30])
assert(scores@first, [30, 10, 20])
assert(scores@last.sort(), [4, 5])
assert(scores, [5, 4])
assert(ranked, [4, 5])
assert(snapshot, [10, 20, 30])
try:
    scores@first.push(1)
catch error:
    assert(error, TypeError("State versions are read-only"))
class Item(size):
    this.size = size
Item(2); Item(1); Item(3)
smallest = Item.sort((x, y) => x.size - y.size)[0].size
assert(smallest, 1)
assert(Item.map(item => item.size), [2, 1, 3])
Item(0)
assert(smallest, 0)
assert(Item[0].size, 2)
---
# Nucleoid sorts only the local binding of a copied list

def ordered(values):
    values.sort()
    return values
original = [3, 1, 2]
assert(ordered(original), [1, 2, 3])
assert(original, [3, 1, 2])
rows = [[3, 1], [2, 0]]
for row of rows:
    row.sort()
assert(rows, [[3, 1], [2, 0]])
---
# Nucleoid keeps a list sorted with a standing sort

class Player(name, score):
    this.name = name
    this.score = score
alice = Player("alice", 30)
bob = Player("bob", 10)
cara = Player("cara", 20)
players = [alice, bob, cara]
direction = 1
ranked = players.sort((a, b) => (a.score - b.score) * direction)
names = players.map(player => player.name)
assert(names, ["bob", "cara", "alice"])
assert(ranked, players)
dave = Player("dave", 5)
players.push(dave)
assert(names, ["dave", "bob", "cara", "alice"])
bob.score = 40
assert(names, ["dave", "cara", "alice", "bob"])
direction = -1
assert(names, ["bob", "alice", "cara", "dave"])
players = [cara, alice]
assert(names, ["alice", "cara"])
assert(ranked, players)
values = [3, 1, 2]
values = values.sort()
values.push(0)
assert(values, [1, 2, 3, 0])
names
# return: ["alice","cara"]
---
# Nucleoid records in place sorts as list versions

scores = [3, 1, 2]
log = []
watched = scores.length + log.push("run") * 0
assert(log.length, 1)
scores.sort()
assert(scores@0, [3, 1, 2])
assert(scores@1, [1, 2, 3])
assert(log.length, 2)
scores.sort()
assert(scores@2, null)
assert(log.length, 3)
assert(watched, 3)
---
# Nucleoid rolls back failed in place sorts

scores = [3, 1, 2]
failure = ""
try:
    scores.sort((a, b) => a - b)
    throw "SORT_ROLLBACK"
catch error:
    failure = error
assert(failure, "SORT_ROLLBACK")
assert(scores, [3, 1, 2])
assert(scores@1, null)
try:
    scores.sort((a, b) => { throw "BAD_COMPARATOR" })
catch error:
    failure = error
assert(failure, "BAD_COMPARATOR")
assert(scores, [3, 1, 2])
def compare(a, b):
    if a > 100:
        throw "TOO_LARGE"
    return a - b
ranked = scores.sort(compare)
assert(scores, [1, 2, 3])
try:
    scores.push(500)
catch error:
    failure = error
assert(failure, "TOO_LARGE")
assert(scores, [1, 2, 3])
assert(ranked, [1, 2, 3])
scores.push(0)
assert(scores, [0, 1, 2, 3])
assert(ranked, [0, 1, 2, 3])
---
# Nucleoid selects values with conditional expressions

total = 80
member = false
shipping = 0 if total > 100 else 5
discount = member ? 10 : 0
grade = total >= 90 ? "A" : total >= 70 ? "B" : "C"
level = "gold" if total > 150 else "silver" if total > 50 else "basic"
size = total > 100 ? "large" : "medium" if total > 50 else "small"
label = member || total > 100 ? "priority" : "standard"
charge = total + (member ? 0 : shipping)
vip = "yes" if member and total > 50 else "no"
assert(shipping, 5)
assert(discount, 0)
assert(grade, "B")
assert(level, "silver")
assert(size, "medium")
assert(label, "standard")
assert(charge, 85)
assert(vip, "no")
total = 120
assert(shipping, 0)
assert(grade, "A")
assert(level, "silver")
assert(size, "large")
assert(label, "priority")
assert(charge, 120)
member = true
assert(discount, 10)
assert(vip, "yes")
total = 40
assert(shipping, 5)
assert(grade, "C")
assert(level, "basic")
assert(size, "small")
assert(label, "priority")
assert(charge, 40)
assert(vip, "no")
---
# Nucleoid evaluates only the selected conditional branch

log = []
def note(tag):
    log.push(tag)
    return tag
flag = true
other = 1
pick = flag ? note("yes") : note(other)
assert(pick, "yes")
assert(log, ["yes"])
other = 2
assert(pick, "yes")
assert(log, ["yes"])
flag = false
assert(pick, 2)
assert(log, ["yes", 2])
other = 3
assert(pick, 3)
assert(log, ["yes", 2, 3])
checks = []
def check(value):
    checks.push(value)
    return value > 1
size = "big" if check(other) else "small"
assert(size, "big")
assert(checks, [3])
ready = true
safe = 1 if ready else missing
assert(safe, 1)
failure = ""
try:
    ready = false
catch error:
    failure = error
assert(failure, ReferenceError("missing is not defined"))
assert(ready, true)
assert(safe, 1)
missing = 7
assert(safe, 1)
ready = false
assert(safe, 7)
toggle = null
state = toggle ? "on" : "off"
assert(state, null)
toggle = false
assert(state, "off")
toggle = 1
assert(state, "on")
---
# Nucleoid applies conditional expressions in class level rules

class Order(total, color):
    this.total = total
    this.color = color
$Order.shipping = 0 if $Order.total > 100 else 5
$Order.tag = $Order.color == 1 ? "a" : "b"
$Order.rush = $Order.express ? 10 : 0
$Order.due = $Order.total + $Order.shipping + $Order.rush
small = Order(40, 1)
large = Order(250, 2)
assert(small.shipping, 5)
assert(large.shipping, 0)
assert(small.tag, "a")
assert(large.tag, "b")
assert(small.rush, 0)
assert(small.due, 45)
assert(large.due, 250)
small.total = 120
assert(small.shipping, 0)
assert(small.due, 120)
assert(large.shipping, 0)
large.color = 1
assert(large.tag, "a")
assert(small.tag, "a")
large.express = true
assert(large.rush, 10)
assert(small.rush, 0)
large.total = 30
assert(large.shipping, 5)
assert(large.due, 45)
failure = ""
try:
    $Order.note = $Order.total > 1000 ? vip : "regular"
catch error:
    failure = error
assert(failure, ReferenceError("vip is not defined"))
try:
    $Order.limit = $Order.express ? $Order.cap : 50
    $Order.cap = $Order.limit + 1
catch error:
    failure = error
assert(failure, TypeError("Circular Dependency"))
---
# Nucleoid uses conditional expressions in functions callbacks and literals

limit = 2
items = [1, 2, 3]
def size(value) => value > limit ? "big" : "small"
def sign(value):
    return "neg" if value < 0 else "pos"
labels = items.map(item => item > limit ? "big" : "small")
kinds = items.map(lambda item: "odd" if item % 2 else "even")
first = size(items[0])
summary = { count: items.length > 2 ? "many" : "few", signs: [sign(-1), sign(limit)] }
pick = items[limit > 2 ? 0 : 1]
banner = `${limit > 2 ? "wide" : "narrow"}`
scaled = Math.max(limit > 2 ? 10 : 20, 5)
assert(labels, ["small", "small", "big"])
assert(kinds, ["odd", "even", "odd"])
assert(first, "small")
assert(summary.count, "many")
assert(summary.signs, ["neg", "pos"])
assert(pick, 2)
assert(banner, "narrow")
assert(scaled, 20)
limit = 0
assert(labels, ["big", "big", "big"])
assert(first, "big")
assert(pick, 2)
limit = 5
assert(pick, 1)
assert(banner, "wide")
assert(scaled, 10)
items.push(6)
assert(labels, ["small", "small", "small", "big"])
assert(kinds, ["odd", "even", "odd", "even"])
---
# Nucleoid keeps if statements beside conditional expressions

ready = true
armed = true
lit = true
limit = 10
mode = "on" if ready else "off"
x = 1 if armed:
    y = 2
z = 3 if lit: w = 4
if limit > 5: a = 1 else: a = 2
b = 5 c = 6
if limit > (ready ? 5 : 20):
    band = "wide"
else:
    band = "narrow"
if armed and lit
    flow = "open"
elif limit > 5
    flow = "half"
else
    flow = "shut"
note = "ready? yes"
pattern = /colou?r/
assert(mode, "on")
assert([x, y, z, w, a, b, c], [1, 2, 3, 4, 1, 5, 6])
assert(band, "wide")
assert(flow, "open")
assert(note, "ready? yes")
assert(pattern.test("color"), true)
ready = false
armed = false
assert(mode, "off")
assert(band, "narrow")
assert(flow, "half")
assert([x, y, z, w, a], [1, 2, 3, 4, 1])
limit = 1
assert(a, 2)
assert(flow, "shut")
---
# Nucleoid rolls back failed conditional expression updates

def check(amount):
    if amount > 100:
        throw "TOO_LARGE"
    return amount
rate = 2
amount = 5
fee = amount > 10 ? check(amount) * rate : rate
assert(fee, 2)
failure = ""
try:
    amount = 500
catch error:
    failure = error
assert(failure, "TOO_LARGE")
assert(amount, 5)
assert(fee, 2)
try:
    amount = 20
    throw "ROLLBACK"
catch error:
    failure = error
assert(failure, "ROLLBACK")
assert(amount, 5)
assert(fee, 2)
amount = 20
assert(fee, 40)
rate = 3
assert(fee, 60)
base = 3
frozen = amount > 10 ? base.value : 0
spare = base.value if amount > 50 else 1
base = 4
assert(frozen, 3)
assert(spare, 1)
amount = 60
assert(spare, 3)
assert(fee, 180)
bonus = 7
reward = bonus if amount > 30 else 0
assert(reward, 7)
delete bonus
assert(reward, null)
bonus = 9
assert(reward, 9)
amount = 25
assert(reward, 0)
---
# Nucleoid defaults unknown values with nullish coalescing

stock = null
count = stock ?? 0
discount = 0
rate = discount ?? 10
page = { name: "", muted: false }
page.title = page.name ?? "untitled"
page.sound = page.muted ?? true
nickname = null
first = "Ada"
display = nickname ?? first ?? "guest"
other = null
fallback = other ?? nickname
plain = stock || 0
legacy = discount || 10
assert(count, 0)
assert(rate, 0)
assert(page.title, "")
assert(page.sound, false)
assert(display, "Ada")
assert(fallback, null)
assert(plain, null)
assert(legacy, 10)
stock = 5
discount = null
page.name = null
nickname = "Lady A"
assert(count, 5)
assert(rate, 10)
assert(page.title, "untitled")
assert(display, "Lady A")
assert(fallback, "Lady A")
assert(plain, 5)
assert(legacy, null)
stock = 0
first = null
nickname = null
assert(count, 0)
assert(display, "guest")
assert(fallback, null)
---
# Nucleoid evaluates the right operand of nullish coalescing only when needed

log = []
def note(tag):
    log.push(tag)
    return tag
saved = "draft"
other = 1
pick = saved ?? note(other)
assert(pick, "draft")
assert(log, [])
other = 2
assert(pick, "draft")
assert(log, [])
saved = null
assert(pick, 2)
assert(log, [2])
other = 3
assert(pick, 3)
assert(log, [2, 3])
saved = 0
other = 4
assert(pick, 0)
assert(log, [2, 3])
ready = 1
safe = ready ?? missing
assert(safe, 1)
failure = ""
try:
    ready = null
catch error:
    failure = error
assert(failure, ReferenceError("missing is not defined"))
assert(ready, 1)
assert(safe, 1)
missing = 7
ready = null
assert(safe, 7)
try:
    loose = absent ?? 5
catch error:
    failure = error
assert(failure, ReferenceError("absent is not defined"))
---
# Nucleoid applies nullish coalescing in class level rules

class Order(total, coupon):
    this.total = total
    this.coupon = coupon
$Order.discount = $Order.coupon ?? 0
$Order.due = $Order.total - $Order.discount
if $Order.rush ?? false:
    $Order.lane = "fast"
else:
    $Order.lane = "normal"
small = Order(40, null)
large = Order(250, 25)
savings = Order.map(order => order.coupon ?? 0)
assert(small.discount, 0)
assert(large.discount, 25)
assert(small.due, 40)
assert(large.due, 225)
assert(small.lane, "normal")
assert(large.lane, "normal")
assert(savings, [0, 25])
small.coupon = 0
assert(small.discount, 0)
small.coupon = 5
assert(small.discount, 5)
assert(small.due, 35)
large.coupon = null
assert(large.discount, 0)
assert(large.due, 250)
assert(savings, [5, 0])
large.rush = true
assert(large.lane, "fast")
assert(small.lane, "normal")
large.rush = null
assert(large.lane, "normal")
failure = ""
try:
    $Order.note = $Order.memo ?? vip
catch error:
    failure = error
assert(failure, ReferenceError("vip is not defined"))
try:
    $Order.limit = $Order.cap ?? 50
    $Order.cap = $Order.limit + 1
catch error:
    failure = error
assert(failure, TypeError("Circular Dependency"))
---
# Nucleoid uses nullish coalescing in functions callbacks and literals

settings = {}
items = [{ name: "pen", size: 2 }, { name: "cup", size: 0 }, { name: "box" }]
def width(item) => item.size ?? settings.size ?? 1
def label(item):
    return item.nick ?? item.name
sizes = items.map(item => item.size ?? -1)
names = items.map(lambda item: item.nick ?? item.name)
first = width(items[2])
last = label(items[2])
summary = { cap: settings.cap ?? 10, widths: [settings.size ?? 1, width(items[1])] }
pick = items[settings.index ?? 1].name
banner = `${settings.title ?? "untitled"}`
scaled = Math.max(settings.cap ?? 0, 5)
mode = settings.strict ? "strict" : settings.mode ?? "auto"
assert(sizes, [2, 0, -1])
assert(names, ["pen", "cup", "box"])
assert(first, 1)
assert(last, "box")
assert(summary.cap, 10)
assert(summary.widths, [1, 0])
assert(pick, "cup")
assert(banner, "untitled")
assert(scaled, 5)
assert(mode, "auto")
settings.size = 3
settings.index = 0
settings.title = "Sale"
settings.cap = 0
settings.mode = "manual"
assert(first, 3)
assert(summary.cap, 0)
assert(summary.widths, [3, 0])
assert(pick, "pen")
assert(banner, "Sale")
assert(scaled, 5)
assert(mode, "manual")
items.push({ name: "bag", nick: "tote" })
assert(sizes, [2, 0, -1, -1])
assert(names, ["pen", "cup", "box", "tote"])
---
# Nucleoid groups nullish coalescing between conditionals and logical operators

page = { base: 0, level: 3, visible: true }
state = page.custom ?? page.base ? "set" : "unset"
shown = page.visible ? page.label ?? "untitled" : "hidden"
spoken = page.label ?? "untitled" if page.visible else "hidden"
first = page.flag || page.backup ?? 9
second = page.custom ?? page.base || page.level
same = page.custom ?? page.level == 3
note = "why?? not"
pattern = /colou??r/
greeting = `who?? ${page.label ?? "you"}`
assert(state, "unset")
assert(shown, "untitled")
assert(spoken, "untitled")
assert(first, 9)
assert(second, 3)
assert(same, true)
assert(note, "why?? not")
assert(pattern.test("color"), true)
assert(greeting, "who?? you")
page.custom = 0
page.backup = 4
assert(state, "unset")
assert(first, 4)
assert(second, 0)
assert(same, 0)
page.custom = 2
page.label = "Main"
assert(state, "set")
assert(shown, "Main")
assert(spoken, "Main")
assert(second, 2)
assert(greeting, "who?? Main")
page.visible = false
assert(shown, "hidden")
assert(spoken, "hidden")
---
# Nucleoid rolls back failed nullish coalescing updates

def check(amount):
    if amount > 100:
        throw "TOO_LARGE"
    return amount
rate = 2
amount = 5
custom = null
fee = custom ?? check(amount) * rate
assert(fee, 10)
failure = ""
try:
    amount = 500
catch error:
    failure = error
assert(failure, "TOO_LARGE")
assert(amount, 5)
assert(fee, 10)
try:
    custom = 1
    throw "ROLLBACK"
catch error:
    failure = error
assert(failure, "ROLLBACK")
assert(custom, null)
assert(fee, 10)
custom = 3
assert(fee, 3)
amount = 500
assert(fee, 3)
rate = 4
amount = 20
custom = null
assert(fee, 80)
base = 3
config = { size: 2 }
spare = config.size ?? base.value
base = 4
assert(spare, 2)
delete config.size
assert(spare, 3)
bonus = 7
reward = bonus ?? 0
assert(reward, 7)
delete bonus
assert(reward, 0)
bonus = 9
assert(reward, 9)
---
# Nucleoid slices class instance lists with ranges

class User(name):
    this.name = name
ada = User("Ada")
bob = User("Bob")
cy = User("Cy")
assert(User[:], [ada, bob, cy])
assert(User[1:], [bob, cy])
assert(User[:2], [ada, bob])
assert(User[-2:], [bob, cy])
assert(User[1:-1], [bob])
assert(User[0:9], [ada, bob, cy])
assert(User[-9:1], [ada])
assert(User[2:1], [])
assert(User[1:], User.slice(1))
assert(User[-2:-1], User.slice(-2, -1))
assert(User[:].length, User.length)
start = 1
assert(User[start:start + 1], [bob])
class Empty
assert(Empty[:], [])
assert(User[0], ada)
assert(User["bob"], bob)
assert([1, 2, 3][1:], [2, 3])
assert("Nucleoid"[-4:], "eoid")
User[1:].map(user => user.name)
# return: ["Bob","Cy"]
---
# Nucleoid keeps instance list ranges live

class Task(title):
    this.title = title
first = Task("plan")
second = Task("build")
count = 1
recent = Task[-2:]
opening = Task[:count]
pinned = Task[:count.value]
titles = Task[1:].map(task => task.title)
assert(recent, [first, second])
assert(opening, [first])
assert(pinned, [first])
assert(titles, ["build"])
third = Task("ship")
assert(recent, [second, third])
assert(titles, ["build", "ship"])
count = 2
assert(opening, [first, second])
assert(pinned, [first])
second.title = "test"
assert(titles, ["test", "ship"])
class Ticket
older = Ticket()
newer = Ticket()
latest = Ticket[-1:]
extra = Ticket()
assert(latest, [extra])
delete extra
assert(latest, [newer])
assert(Ticket[:], [older, newer])
---
# Nucleoid lists every instance with list and all

class Member(name):
    this.name = name
ana = Member("Ana")
ben = Member("Ben")
everyone = Member.list()
roster = Member.all()
names = Member.list().map(member => member.name)
assert(everyone, [ana, ben])
assert(roster, everyone)
assert(everyone, Member[:])
assert(everyone, Member.slice())
cal = Member("Cal")
assert(everyone, [ana, ben, cal])
assert(roster, [ana, ben, cal])
assert(names, ["Ana", "Ben", "Cal"])
class Guest
dee = Guest()
party = Guest.concat(Member.list())
assert(party, [dee, ana, ben, cal])
assert(Guest.concat(Member[1:]), [dee, ben, cal])
calls = []
def note(tag):
    calls.push(tag)
    return tag
assert(Member.all(note(1), note(2)), [ana, ben, cal])
assert(calls, [1, 2])
plain = [1, 2]
failure = ""
try:
    plain.list()
catch error:
    failure = error
assert(failure, TypeError("plain.list is not a function"))
try:
    plain.all()
catch error:
    failure = error
assert(failure, TypeError("plain.all is not a function"))
---
# Nucleoid compares list members with deep equality

tags = ["red", "blue", "red"]
assert(tags.includes("blue"), true)
assert(tags.includes("green"), false)
assert(tags.indexOf("red"), 0)
assert(tags.indexOf("green"), -1)
assert(tags.indexOf("red", 1), 0)
assert(tags.includes("red", 5), true)
pairs = [[1, 2], { x: 1 }, null]
assert(pairs.indexOf([1, 2]), 0)
assert(pairs.includes({ x: 1 }), true)
assert(pairs.includes({ x: 2 }), false)
assert(pairs.indexOf(), 2)
assert(pairs.includes(), true)
assert([Number.NaN].includes(Number.NaN), false)
assert([Number.NaN].indexOf(Number.NaN), -1)
class Seat(row):
    this.row = row
front = Seat(1)
twin = Seat(1)
assert(Seat.indexOf(twin), 1)
assert(Seat.indexOf(front), 0)
assert(Seat.includes(twin), true)
assert(Seat.includes({ row: 1 }), false)
assert(Seat.includes({ id: "twin", row: 1 }), true)
assert(Seat.indexOf(5), -1)
where = tags.indexOf("green")
known = tags.includes("green")
assert(where, -1)
tags.push("green")
assert(where, 3)
assert(known, true)
back = Seat(2)
spot = Seat.indexOf(back)
assert(spot, 2)
---
# Nucleoid concatenates lists and instance lists into new lists

base = [1, 2]
more = base.concat(3, [4, [5]])
assert(more, [1, 2, 3, 4, [5]])
assert(base, [1, 2])
assert(base.concat(), [1, 2])
assert(base.concat(null), [1, 2, null])
assert(base.concat([]), [1, 2])
base.push(9)
assert(more, [1, 2, 9, 3, 4, [5]])
class Guest
amy = Guest()
ben = Guest()
class Host
cara = Host()
dan = Host()
party = Guest.concat([cara, dan])
assert(party, [amy, ben, cara, dan])
assert(Guest.concat(Host).length, 3)
assert(Guest.concat(Host)[2], Host)
eve = Guest()
assert(party, [amy, ben, eve, cara, dan])
assert(Guest.length, 3)
---
# Nucleoid reads positions from either end with at and lastIndexOf

scores = [10, 20, 30]
assert(scores.at(0), 10)
assert(scores.at(-1), 30)
assert(scores.at(1.9), 20)
assert(scores.at(-1.9), 30)
assert(scores.at(), 10)
assert(scores.at("2"), 30)
assert(scores.at(3), null)
assert(scores.at(-4), null)
assert(typeof scores.at(3), "undefined")
last = scores.at(-1)
scores.push(40)
assert(last, 40)
label = "top " + scores.at(9)
assert(label, null)
tags = ["red", "blue", "red"]
assert(tags.lastIndexOf("red"), 2)
assert(tags.lastIndexOf("green"), -1)
assert(tags.lastIndexOf("red", 0), 2)
assert([[1], [1]].lastIndexOf([1]), 1)
assert([null, 1].lastIndexOf(), 0)
assert([Number.NaN].lastIndexOf(Number.NaN), -1)
where = tags.lastIndexOf("blue")
tags.push("blue")
assert(where, 3)
class Lap(time):
    this.time = time
one = Lap(61)
two = Lap(58)
assert(Lap.at(0), one)
assert(Lap.at(-1), two)
assert(Lap.at(2), null)
assert(typeof Lap.at(2), "null")
assert("lap " + Lap.at(5), "lap null")
assert(Lap.lastIndexOf(one), 0)
latest = Lap.at(-1).time
three = Lap(57)
assert(latest, 57)
---
# Nucleoid searches from either end with findIndex findLast and findLastIndex

visits = []
def big(value):
    visits.push(value)
    return value > 15
readings = [10, 20, 30, 5]
assert(readings.findIndex(big), 1)
assert(visits, [10, 20])
visits = []
assert(readings.findLast(big), 30)
assert(visits, [5, 30])
visits = []
assert(readings.findLastIndex(big), 2)
assert(visits, [5, 30])
assert(readings.find(big), 20)
assert(readings.findIndex(value => value > 99), -1)
assert(readings.findLast(value => value > 99), null)
assert(readings.findLastIndex(value => value > 99), -1)
assert(readings.findLastIndex((value, position) => position == null), 3)
assert([].findLast(value => true), null)
failure = ""
try:
    readings.findIndex(15)
catch error:
    failure = error
assert(failure, TypeError("findIndex expects a function"))
try:
    [].findLast()
catch error:
    failure = error
assert(failure, TypeError("findLast expects a function"))
try:
    [].findLastIndex(null)
catch error:
    failure = error
assert(failure, TypeError("findLastIndex expects a function"))
---
# Nucleoid keeps list queries live

class Order(total, rush):
    this.total = total
    this.rush = rush
first = Order(20, false)
second = Order(45, true)
third = Order(30, false)
limit = 25
lastRush = Order.findLast(order => order.rush)
lastBig = Order.findLastIndex(order => order.total > limit)
firstBig = Order.findIndex(order => order.total > limit)
newest = Order.at(-1)
assert(lastRush, second)
assert(lastBig, 2)
assert(firstBig, 1)
assert(newest, third)
third.rush = true
assert(lastRush, third)
limit = 40
assert(lastBig, 1)
assert(firstBig, 1)
second.total = 10
assert(lastBig, -1)
assert(firstBig, -1)
fourth = Order(50, false)
assert(newest, fourth)
assert(lastBig, 3)
assert(firstBig, 3)
assert(lastRush, third)
queue = [3, 8, 1]
peak = queue.findLastIndex(value => value > limit / 10)
tail = queue.at(-1)
where = queue.lastIndexOf(8)
assert(peak, 1)
queue.push(9)
assert(peak, 3)
assert(tail, 9)
assert(where, 1)
queue.pop()
assert(tail, 1)
assert(peak, 1)
---
# Nucleoid applies instance list ranges and queries in class level rules

class Player(name):
    this.name = name
class Team(size):
    this.size = size
$Team.roster = Player[0:$Team.size]
$Player.rank = Player.indexOf($Player)
ana = Player("Ana")
ben = Player("Ben")
red = Team(1)
blue = Team(3)
assert(red.roster, [ana])
assert(blue.roster, [ana, ben])
assert(ana.rank, 0)
assert(ben.rank, 1)
cal = Player("Cal")
assert(blue.roster, [ana, ben, cal])
assert(cal.rank, 2)
red.size = 2
assert(red.roster, [ana, ben])
failure = ""
try:
    $Team.bench = Player[$Team.size:limit]
catch error:
    failure = error
assert(failure, ReferenceError("limit is not defined"))
---
# Nucleoid rolls back failed list query updates

def check(value):
    if value > 100:
        throw "TOO_LARGE"
    return value > 2
values = [1, 3]
spot = values.findLastIndex(check)
early = values.findIndex(check)
assert(spot, 1)
assert(early, 1)
failure = ""
try:
    values.push(500)
catch error:
    failure = error
assert(failure, "TOO_LARGE")
assert(values, [1, 3])
assert(spot, 1)
values.push(5)
assert(spot, 2)
assert(early, 1)
class Item(size):
    this.size = size
small = Item(1)
pick = Item.findLast(item => check(item.size))
assert(pick, null)
try:
    big = Item(500)
catch error:
    failure = error
assert(failure, "TOO_LARGE")
assert(Item.length, 1)
assert(pick, null)
try:
    mid = Item(3)
    throw "ROLLBACK"
catch error:
    failure = error
assert(failure, "ROLLBACK")
assert(Item.length, 1)
assert(pick, null)
mid = Item(3)
assert(pick, mid)
---
# Nucleoid quantifies over class instances with all and any

class Check
ready = all c in Check: c.passed
blocked = any c in Check: c.failed
assert(ready, true)
assert(blocked, false)
first = Check()
second = Check()
first.passed = true
second.passed = true
assert(ready, true)
second.passed = false
second.failed = true
assert(ready, false)
assert(blocked, true)
second.passed = true
second.failed = false
assert(ready, true)
assert(blocked, false)
third = Check()
assert(ready, false)
third.passed = true
assert(ready, true)
---
# Nucleoid quantifies over lists with all and any

numbers = [3, 5, 8]
limit = 2
large = all n in numbers: n > limit
even = any n in numbers: n % 2 == 0
assert(large, true)
assert(even, true)
limit = 4
assert(large, false)
numbers = [5, 7]
assert(large, true)
assert(even, false)
numbers.push(10)
assert(even, true)
empty = []
vacuous = all n in empty: n > 0
none = any n in empty: n > 0
assert(vacuous, true)
assert(none, false)
---
# Nucleoid quantifies over reasoning selections

a = 1
b = a + 2
c = b * 2
positive = all s in why c: s.holds > 0
grounded = all s in c |> why |> stated: s.holds > 0
mentions = any s in why c: s.node == "d"
assert(positive, true)
assert(grounded, true)
assert(mentions, false)
d = -5
b = a + d
assert(positive, false)
assert(grounded, false)
assert(mentions, true)
d = 5
assert(positive, true)
assert(grounded, true)
---
# Nucleoid nests quantifiers over related instances

class Task(skill):
    this.skill = skill
class Worker(skill):
    this.skill = skill
covered = all t in Task: any w in Worker: w.skill == t.skill
spare = any w in Worker: all t in Task: t.skill != w.skill
assert(covered, true)
assert(spare, false)
Task("rust")
Task("docs")
assert(covered, false)
Worker("rust")
writer = Worker("docs")
assert(covered, true)
assert(spare, false)
Worker("ops")
assert(spare, true)
writer.skill = "ops"
assert(covered, false)
writer.skill = "docs"
assert(covered, true)
---
# Nucleoid applies quantifiers in class level rules

class Line(sent):
    this.sent = sent
class Order(lines):
    this.lines = lines
$Order.complete = all line in $Order.lines: line.sent
$Order.started = any line in $Order.lines: line.sent
first = Line(true)
second = Line(false)
order = Order([first, second])
empty = Order([])
assert(order.complete, false)
assert(order.started, true)
assert(empty.complete, true)
assert(empty.started, false)
second.sent = true
assert(order.complete, true)
first.sent = false
second.sent = false
assert(order.complete, false)
assert(order.started, false)
---
# Nucleoid uses quantifiers in functions callbacks and conditionals

def positive(values) => all v in values: v > 0
scores = [2, 4]
ok = positive(scores)
label = (all v in scores: v > 0) ? "clean" : "dirty"
assert(ok, true)
assert(label, "clean")
scores = [2, -1]
assert(ok, false)
assert(label, "dirty")
groups = [[1, 2], [3, -4], []]
whole = groups.filter(g => all v in g: v > 0)
mixed = groups.map(g => any v in g: v < 0)
assert(whole, [[1, 2], []])
assert(mixed, [false, true, false])
groups = [[-1]]
assert(whole, [])
assert(mixed, [true])
---
# Nucleoid keeps all and any as ordinary names beside quantifiers

all = [1, 2, 3]
any = 0
large = all n in all: n > any
some = any n in all: n > 2
assert(large, true)
assert(some, true)
any = 2
assert(large, false)
assert(some, true)
n = 10
doubled = all n in all: n * 2 > n
assert(doubled, true)
assert(n, 10)
---
# Nucleoid rolls back failed quantifier updates

def checked(value):
    if value < 0:
        throw "NEGATIVE"
    return value > 0
class Item(qty):
    this.qty = qty
item = Item(2)
valid = all i in Item: checked(i.qty)
assert(valid, true)
failure = ""
try:
    item.qty = -1
catch error:
    failure = error
assert(failure, "NEGATIVE")
assert(item.qty, 2)
assert(valid, true)
failure = ""
try:
    Item(-3)
catch error:
    failure = error
assert(failure, "NEGATIVE")
assert(Item.length, 1)
item.qty = 0
assert(valid, false)
try:
    broken = all x in missing: x
catch error:
    failure = error
assert(failure, ReferenceError("missing is not defined"))
assert(valid, false)
---
# Nucleoid raises numbers to powers with the exponentiation operator

base = 2
exponent = 3
tower = base ** exponent ** 2
grouped = (base ** exponent) ** 2
scaled = 3 * base ** exponent + 1
assert(tower, 512)
assert(grouped, 64)
assert(scaled, 25)
base = 3
exponent = 2
assert(tower, 81)
assert(grouped, 81)
assert(scaled, 28)
base = 16
exponent = 0.5
assert(tower, 2)
assert(grouped, 16)
assert(scaled, 13)
tower
# return: 2
---
# Nucleoid binds prefix operators around the exponentiation operator

x = 2
n = 2
negated = -x ** n
grouped = (-x) ** n
inverse = x ** -n
assert(negated, -4)
assert(grouped, 4)
assert(inverse, 0.25)
n = 3
assert(negated, -8)
assert(grouped, -8)
assert(inverse, 0.125)
x = 0.5
assert(negated, -0.125)
assert(grouped, -0.125)
assert(inverse, 8)
---
# Nucleoid gives the exponentiation operator the results of Math.pow

base = "3"
power = 2
result = base ** power
literal = null ** power
assert(result, 9)
assert(literal, 0)
base = [2]
assert(result, 4)
base = true
power = 3
assert(result, 1)
base = "a"
assert(Number.isNaN(result), true)
base = 0
power = -1
assert(result, Number.POSITIVE_INFINITY)
assert(literal, Number.POSITIVE_INFINITY)
base = -0
assert(result, Number.NEGATIVE_INFINITY)
base = -8
power = 1 / 3
assert(Number.isNaN(result), true)
base = Number.NaN
power = 0
assert(result, 1)
assert(literal, 1)
base = null
assert(result, null)
---
# Nucleoid keeps exponentiation relationships live in class level rules

class Tank(side, depth):
    this.side = side
    this.depth = depth
$Tank.volume = $Tank.side ** 2 * $Tank.depth
$Tank.cube = $Tank.side ** 3
small = Tank(2, 3)
large = Tank(3, 1)
assert(small.volume, 12)
assert(large.volume, 9)
assert(small.cube, 8)
small.side = 4
assert(small.volume, 48)
assert(small.cube, 64)
assert(large.cube, 27)
small.depth = 0.5
assert(small.volume, 8)
delete small.side
assert(small.volume, null)
assert(small.cube, null)
---
# Nucleoid keeps compound exponentiation assignments equivalent

value = 4
step = 2
value **= step
assert(value, 16)
step = 0.5
assert(value, 4)
class Meter(reading, power):
    this.reading = reading
    this.power = power
    def root(degree):
        degree **= -1
        this.reading **= degree
        return this.reading
meter = Meter(2, 2)
meter.reading **= meter.power
assert(meter.reading, 4)
meter.power = 3
assert(meter.reading, 64)
assert(meter.root(2), 8)
assert(meter.reading, 8)
class Scale
scale = Scale()
$Scale.factor = 2
try:
    $Scale.factor **= 3
catch error:
    assert(error, TypeError("Circular Dependency"))
assert(scale.factor, 2)
---
# Nucleoid uses the exponentiation operator in functions callbacks and literals

def hypotenuse(a, b) => (a ** 2 + b ** 2) ** 0.5
def grow current rate => current **= rate
sides = [3, 4]
squares = sides.map(side -> side ** 2)
box = {volume: sides[1] ** 3}
label = `${sides[0]} cubed is ${sides[0] ** 3}`
assert(hypotenuse(3, 4), 5)
assert(hypotenuse(8, 15), 17)
assert(grow(4, 1.5), 8)
assert(grow(16, 0.25), 2)
assert(squares, [9, 16])
assert(box.volume, 64)
assert(label, "3 cubed is 27")
sides = [2, 5]
assert(squares, [4, 25])
assert(box.volume, 125)
assert(label, "2 cubed is 8")
---
# Nucleoid keeps double stars in strings and regular expressions beside the exponentiation operator

bits = 8
states = 2 ** bits
note = `2**${bits} is ${states}`
pattern = /x**y/ # x**y stays a pattern
assert(states, 256)
assert(note, "2**8 is 256")
assert(note.split("**"), ["2", "8 is 256"])
assert(pattern.test("xxy"), true)
assert(pattern.test("xx"), false)
bits = 10
assert(states, 1024)
assert(note, "2**10 is 1024")
---
# Nucleoid rolls back failed exponentiation updates

def checked(value):
    if value > 1000:
        throw "TOO_LARGE"
    return value
base = 2
power = 3
result = checked(base ** power)
assert(result, 8)
failure = ""
try:
    power = 11
catch error:
    failure = error
assert(failure, "TOO_LARGE")
assert(power, 3)
assert(result, 8)
base = 3
power = 5
assert(result, 243)
balance = 3
if balance > 100:
    throw "LIMIT"
try:
    balance **= power
catch error:
    failure = error
assert(failure, "LIMIT")
assert(balance, 3)
power = 2
assert(balance, 3)
assert(result, 9)
---
# Nucleoid runs a suppose block in a hypothetical world

threshold = 90
reading = 40
alarm = reading > threshold
status = alarm ? "ALERT" : "OK"
suppose reading = 95:
    assert: status, "ALERT"
    threshold = 99
    assert: alarm, false
    margin = threshold - reading
    assert: margin, 4
    assert: (affects reading).length, 3
assert: reading, 40
assert: threshold, 90
assert: status, "OK"
assert: (affects reading).length, 2
suppose threshold = 30: assert: status, "ALERT"
assert: threshold, 90
reading = 95
assert: status, "ALERT"
---
# Nucleoid discards instances created in a suppose block

class Account(balance): this.balance = balance
$Account.rich = $Account.balance > 100
total = Account.reduce(lambda sum, account: sum + account.balance, 0)
rich = Account.filter(account -> account.rich).length
checking = Account(50)
savings = Account(150)
suppose Account(500):
    assert: Account.length, 3
    assert: total, 700
    checking.balance = 300
    assert: checking.rich, true
    assert: rich, 3
assert: Account.length, 2
assert: total, 200
assert: rich, 1
assert: checking.rich, false
suppose savings.balance = 20: assert: rich, 0
Account(400)
assert: Account.length, 3
assert: total, 600
assert: rich, 2
---
# Nucleoid keeps the value of a suppose block as a snapshot

class Line(price, quantity):
    this.price = price
    this.quantity = quantity
$Line.cost = $Line.price * $Line.quantity
subtotal = Line.reduce(lambda sum, line: sum + line.cost, 0)
shipping = subtotal >= 100 ? 0 : 15
due = subtotal + shipping
pens = Line(2, 10)
ink = Line(30, 2)
bulk = suppose pens.quantity = 30: due
quote = suppose ink.price = 45: pens.quantity = 5; [subtotal, shipping]
assert: bulk, 120
assert: quote, [100, 0]
assert: due, 95
ink.quantity = 3
assert: due, 110
assert: bulk, 120
assert: quote, [100, 0]
---
# Nucleoid nests suppose blocks and discards the inner one first

class Seat(taken): this.taken = taken
free = Seat.filter(seat -> not seat.taken).length
full = not free
front = Seat(false)
back = Seat(true)
outer = suppose front.taken = true:
    assert: full, true
    inner = suppose back.taken = false: [free, full]
    assert: inner, [1, false]
    assert: free, 0
    [inner, full]
assert: outer, [[1, false], true]
assert: free, 1
back.taken = false
assert: free, 2
assert: outer, [[1, false], true]
---
# Nucleoid runs header-less suppose blocks over whole populations

class Account(balance):
    this.balance = balance
    def withdraw(amount): this.balance -= amount; return this.balance >= 0
$Account.interest = $Account.balance * 0.05
total = Account.reduce(lambda sum, account: sum + account.interest, 0)
cash = Account(100)
Account(300)
suppose:
    for account of Account: account.balance *= 2
    assert: total, 40
covered = suppose: cash.withdraw(150)
assert: covered, false
assert: cash.balance, 100
cash.withdraw(50)
assert: total, 17.5
assert: covered, false
---
# Nucleoid returns hypothetical answers from functions and methods

class Account(balance):
    this.balance = balance
    def projected(deposit): suppose this.balance += deposit: return this.tier
$Account.tier = $Account.balance >= 1000 ? "gold" : "basic"
def tier_at(account, balance): suppose account.balance = balance: return account.tier
acct = Account(600)
assert: acct.projected(500), "gold"
assert: acct.projected(100), "basic"
assert: tier_at(acct, 2000), "gold"
assert: acct.balance, 600
assert: acct.tier, "basic"
$Account.tier = $Account.balance >= 650 ? "silver" : "basic"
assert: acct.projected(100), "silver"
assert: tier_at(acct, 600), "basic"
---
# Nucleoid keeps suppose as an ordinary name beside suppose blocks

suppose = 2
reading = 40
scaled = reading * suppose
suppose reading = 95: assert: scaled, 190
suppose suppose = 3: assert: scaled, 120
assert: scaled, 80
def twice suppose => suppose * 2
assert: twice(suppose), 4
box = {suppose}
assert: box.suppose, 2
suppose reading = 50
assert: scaled, 100
---
# Nucleoid supposes a changed class level rule

class Order(amount): this.amount = amount
$Order.fee = $Order.amount * 0.02
Order(100)
large = Order(400)
fees = Order.reduce(lambda sum, order: sum + order.fee, 0)
floor = suppose $Order.fee = Math.max($Order.amount * 0.02, 5): fees
waived = suppose $Order.fee = 0: [fees, large.fee]
assert: floor, 13
assert: waived, [0, 0]
assert: fees, 10
assert: large.fee, 8
Order(500)
assert: fees, 20
assert: floor, 13
---
# Nucleoid applies a value assignment once

stock = 10
qty = 3
low = stock < 8
stock := stock - qty
assert: stock, 7
assert: low, true
qty = 5
assert: stock, 7
stock := stock - qty
assert: stock, 2
assert: stock@previous, 7
assert: (why stock)[0].rule, "stock = 2"
assert: (why stock)[0].state, "stated"
assert: (affects qty).length, 0
stock
# return: 2
---
# Nucleoid replaces a standing relationship with a value assignment

price = 4
count = 3
total = price * count
due = total + 1
assert: due, 13
total := price * 2
assert: total, 8
assert: due, 9
price = 10
count = 5
assert: total, 8
assert: due, 9
assert: (why total)[0].from, []
assert: (affects price).length, 0
box = {width: 2, height: 3}
box.area = box.width * box.height
box.area := box.width
box.width = 7
assert: box.area, 2
total = price * count
assert: due, 51
assert: (affects price).length, 2
total := price * count
# return: 50
---
# Nucleoid assigns instance properties once with value assignments

catalog = {price: 4}
class Order(qty):
    this.qty = qty
    this.price := catalog.price
    this.total = this.qty * this.price
    def reprice(): this.price := catalog.price
first = Order(3)
catalog.price = 9
second = Order(1)
assert: first.price, 4
assert: second.price, 9
assert: first.total, 12
first.qty = 5
assert: first.total, 20
first.reprice()
assert: first.total, 45
catalog.price = 2
assert: first.price, 9
assert: (why first.price)[0].from, []
second.price := catalog.price + second.qty
catalog.price = 6
assert: second.price, 3
assert: second.total, 3
---
# Nucleoid records no dependency through calls in a value assignment

rate = 2
def fee(amount) => amount * rate
class Item(price):
    this.price = price
    def taxed() => this.price * rate
pen = Item(3)
book = Item(5)
charged := fee(10)
quote := pen.taxed()
spent := Item.reduce(lambda sum, item: sum + item.price, 0)
count := Item.length
live = Item.length
rate = 3
pen.price = 4
Item(7)
assert: charged, 20
assert: quote, 6
assert: spent, 8
assert: count, 2
assert: live, 3
assert: fee(10), 30
assert: pen.taxed(), 12
---
# Nucleoid runs a value assignment whenever its conditional branch runs

paid = false
stock = 10
qty = 3
if paid:
    shipped := qty
    stock := stock - qty
paid = true
assert: stock, 7
assert: shipped, 3
qty = 4
stock = 20
assert: stock, 20
assert: shipped, 3
paid = false
paid = true
assert: stock, 16
assert: shipped, 4
class Order(qty, paid):
    this.qty = qty
    this.paid = paid
if $Order.paid:
    $Order.taken := $Order.qty
    stock := stock - $Order.qty
small = Order(2, true)
large = Order(5, false)
assert: stock, 14
large.paid = true
assert: stock, 9
large.qty = 6
assert: large.taken, 5
assert: stock, 9
assert: small.taken, 2
---
# Nucleoid uses value assignments in functions blocks and loops

bonus = 5
def award(points):
    granted := points * bonus
    return granted
prize = award(2)
class Player(name): this.name = name
ada = Player("Ada")
bo = Player("Bo")
def roster():
    for player of Player: last := player.name
    return last
final = roster()
a = 1
b = 2
{
    sum := a + b
    double = sum * 2
}
bonus = 7
bo.name = "Ben"
a = 10
assert: prize, 10
assert: granted, 10
assert: final, "Bo"
assert: double, 6
assert: sum, 3
assert: award(2), 14
assert: roster(), "Ben"
---
# Nucleoid keeps colon readings beside value assignments

class Meter:
    reading: int
    def init(reading): this.reading = reading
meter = Meter(4)
scale = 2
label := `${meter.reading:=scale}`
level := meter.reading > 3 ? "high" : "low"
box := {size: meter.reading * scale}
double = lambda value: value * scale
parts = [1, 2, 3][1:3]
ready := all part in parts: part > 1
if ready: total := double(meter.reading)
meter.reading = 1
scale = 5
parts = [0]
assert: label, "4"
assert: level, "high"
assert: box.size, 8
assert: ready, true
assert: total, 8
assert: double(2), 10
---
# Nucleoid rolls back failed value assignments

def checked(amount):
    if amount > 100: throw "TOO_LARGE"
    return amount
limit = 50
stock = 10
stock := checked(limit)
assert: stock, 50
failure = ""
try:
    stock := checked(limit * 3)
catch error:
    failure = error
assert: failure, "TOO_LARGE"
assert: stock, 50
balance = 3
if balance > 100: throw "LIMIT"
try:
    balance := stock * 4
catch error:
    failure = error
assert: failure, "LIMIT"
assert: balance, 3
try:
    missing.total := stock
catch error:
    failure = error
assert: failure, ReferenceError("missing is not defined")
limit = 1000
balance := stock + 40
stock = 70
assert: stock, 70
assert: balance, 90
---
# Nucleoid totals lists with sum

moves = [4, -6, 10]
net = moves.sum()
travelled = moves.sum(move -> Math.abs(move))
assert: net, 8
assert: travelled, 20
moves.push(-5)
assert: net, 3
assert: travelled, 25
moves.pop()
moves.pop()
assert: net, -2
assert: travelled, 10
moves = []
assert: net, 0
assert: travelled, 0
cart = {items: [0.1, 0.2]}
paid = cart.items.sum()
small = cart.items.filter(item -> item < 0.25).sum()
assert: paid, 0.30000000000000004
cart.items.push(0.3)
assert: paid, 0.6000000000000001
assert: small, 0.30000000000000004
---
# Nucleoid totals class instance populations with sum

class Order(total, rush):
    this.total = total
    this.rush = rush
first = Order(20, false)
second = Order(45, true)
revenue = Order.sum(order -> order.total)
rushed = Order.filter(order -> order.rush).sum(order -> order.total)
recent = Order[-2:].sum(order -> order.total)
assert: revenue, 65
assert: rushed, 45
assert: recent, 65
third = Order(30, true)
assert: revenue, 95
assert: rushed, 75
assert: recent, 75
first.total = 50
assert: revenue, 125
delete third.total
assert: revenue, null
delete third.rush
delete third
assert: revenue, 95
assert: rushed, 45
class Refund(amount): this.amount = amount
refunds = Refund.sum(refund -> refund.amount)
assert: refunds, 0
Refund(5)
assert: refunds, 5
---
# Nucleoid converts summed values to numbers

flags = [true, false, true]
count = flags.sum()
entries = [2, "3", " 4 ", "", null, [5], []]
total = entries.sum()
assert: count, 2
assert: total, 14
flags.push(true)
entries.push("x")
assert: count, 3
assert: Number.isNaN(total), true
entries.pop()
entries.push(Number.POSITIVE_INFINITY)
assert: total, Number.POSITIVE_INFINITY
class Item(price): this.price = price
pen = Item(3)
book = Item(12)
spent = Item.sum(item -> item.price)
assert: spent, 15
gift = Item(null)
assert: spent, null
gift.price = 5
assert: spent, 20
---
# Nucleoid calls the sum selector once per item with only the item

seen = []
def weigh(item, position):
    seen.push(position ?? item)
    return item * item
items = [3, 1, 2]
weighed := items.sum(weigh)
assert: weighed, 14
assert: seen, [3, 1, 2]
floor = 2
def price item => Math.max(item, floor)
cost = items.sum(price)
assert: cost, 7
floor = 3
assert: cost, 9
items.push(4)
assert: cost, 13
assert: seen, [3, 1, 2]
missing = null
skipped = missing.sum(weigh(9))
assert: skipped, null
assert: seen, [3, 1, 2]
---
# Nucleoid keeps sums live in class level rules

class Order(name): this.name = name
class Line(order, price, qty):
    this.order = order
    this.price = price
    this.qty = qty
$Line.cost = $Line.price * $Line.qty
$Order.total = Line.filter(line -> line.order == $Order).sum(line -> line.cost)
$Order.units = Line.filter(line -> line.order == $Order).sum(line -> line.qty)
north = Order("north")
south = Order("south")
Line(north, 2, 3)
pen = Line(south, 4, 2)
Line(north, 5, 1)
assert: north.total, 11
assert: south.total, 8
assert: north.units, 4
pen.qty = 5
assert: south.total, 20
assert: south.units, 5
grand = Order.sum(order -> order.total)
assert: grand, 31
west = Order("west")
assert: west.total, 0
Line(west, 1, 1)
assert: grand, 32
pen.price = 1
assert: grand, 17
---
# Nucleoid uses sums in functions methods and conditional expressions

class Team(name, scores):
    this.name = name
    this.scores = scores
    def points(): return this.scores.sum()
def squares(team) => team.scores.sum(score -> score * score)
red = Team("red", [3, 4])
blue = Team("blue", [5])
league = Team.sum(team -> team.points())
leader = red.points() > blue.points() ? red.name : blue.name
spread = squares(red)
assert: league, 12
assert: leader, "red"
assert: spread, 25
blue.scores.push(4)
red.scores.push(1)
Team("green", [2])
assert: league, 19
assert: leader, "blue"
assert: spread, 26
---
# Nucleoid keeps sum as an ordinary name beside the sum query

sum = 2
report = {sum: 5}
def double sum => sum * 2
class Bag(items):
    this.items = items
    def sum(): return this.items.sum() + 1
bag = Bag([1, 2])
total = bag.items.sum() + sum + report.sum
bags = Bag.sum(each -> each.sum())
assert: total, 10
assert: bag.sum(), 4
assert: bags, 4
assert: double(sum), 4
assert: bag.items.sum, null
bag.items.push(3)
sum = 4
Bag([5])
assert: total, 15
assert: bags, 13
---
# Nucleoid rolls back failed sum updates

def checked(value):
    if value > 100:
        throw "TOO_LARGE"
    return value
amounts = [10, 20]
total = amounts.sum(checked)
assert: total, 30
failure = ""
try:
    amounts.push(500)
catch error:
    failure = error
assert: failure, "TOO_LARGE"
assert: amounts, [10, 20]
assert: total, 30
amounts.push(5)
assert: total, 35
class Item(price): this.price = price
pen = Item(3)
spent = Item.sum(item -> checked(item.price))
failure = ""
try:
    Item(500)
catch error:
    failure = error
assert: failure, "TOO_LARGE"
assert: Item.length, 1
assert: spent, 3
Item(4)
assert: spent, 7
---
# Nucleoid finds list extremes with max and min

temps = [18, 24, 9]
high = temps.max()
low = temps.min()
assert: high, 24
assert: low, 9
temps.push(30)
assert: high, 30
temps.pop()
temps.pop()
assert: high, 24
assert: low, 18
temps = []
assert: high, Number.NEGATIVE_INFINITY
assert: low, Number.POSITIVE_INFINITY
trip = {legs: [{km: 5, mins: 10}, {km: 12, mins: 30}]}
fastest = trip.legs.max(leg -> leg.km / leg.mins)
shortest = trip.legs.min(leg -> leg.km)
assert: fastest, 0.5
assert: shortest, 5
trip.legs.push({km: 3, mins: 4})
assert: fastest, 0.75
assert: shortest, 3
---
# Nucleoid finds extremes of class instance populations with max and min

class Bid(amount, live):
    this.amount = amount
    this.live = live
highest = Bid.max(bid -> bid.amount)
lowest = Bid.min(bid -> bid.amount)
assert: highest, Number.NEGATIVE_INFINITY
assert: lowest, Number.POSITIVE_INFINITY
first = Bid(120, true)
second = Bid(90, false)
third = Bid(150, true)
open = Bid.filter(bid -> bid.live).min(bid -> bid.amount)
recent = Bid[-2:].min(bid -> bid.amount)
assert: highest, 150
assert: lowest, 90
assert: open, 120
assert: recent, 90
second.amount = 200
first.live = false
assert: highest, 200
assert: lowest, 120
assert: open, 150
assert: recent, 150
delete third.amount
assert: highest, null
delete third.live
delete third
assert: highest, 200
assert: open, Number.POSITIVE_INFINITY
assert: recent, 120
---
# Nucleoid compares extreme values as numbers

inputs = ["10", " 7 ", "9"]
largest = inputs.max()
smallest = inputs.min()
assert: largest, 10
assert: smallest, 7
inputs.push(true)
assert: smallest, 1
inputs.push(null)
assert: smallest, 0
assert: largest, 10
inputs.push("n/a")
assert: Number.isNaN(largest), true
assert: Number.isNaN(smallest), true
inputs.pop()
inputs.push(Number.POSITIVE_INFINITY)
assert: largest, Number.POSITIVE_INFINITY
assert: smallest, 0
class Gauge(level): this.level = level
pump = Gauge(55)
tank = Gauge(40)
peak = Gauge.max(gauge -> gauge.level)
assert: peak, 55
idle = Gauge(null)
assert: peak, null
idle.level = 70
assert: peak, 70
---
# Nucleoid calls the extreme selector once per item with only the item

seen = []
def weigh(item, position):
    seen.push(position ?? item)
    return item * item
items = [3, -4, 2]
def heaviest() => items.max(weigh)
assert: heaviest(), 16
assert: seen, [3, -4, 2]
items.push(5)
assert: heaviest(), 25
assert: seen, [3, -4, 2, 3, -4, 2, 5]
target = 4
def gap item => Math.abs(item - target)
spread = items.max(gap)
assert: spread, 8
target = -1
assert: spread, 6
missing = null
skipped = missing.min(weigh(9))
assert: skipped, null
assert: seen, [3, -4, 2, 3, -4, 2, 5]
---
# Nucleoid keeps extremes live in class level rules

class Room(name): this.name = name
class Sensor(room, temp):
    this.room = room
    this.temp = temp
$Room.high = Sensor.filter(sensor -> sensor.room == $Room).max(sensor -> sensor.temp)
$Room.low = Sensor.filter(sensor -> sensor.room == $Room).min(sensor -> sensor.temp)
$Room.swing = $Room.high - $Room.low
lab = Room("lab")
den = Room("den")
Sensor(lab, 21)
probe = Sensor(lab, 25)
Sensor(den, 19)
assert: lab.high, 25
assert: lab.swing, 4
assert: den.swing, 0
peak = Room.max(room -> room.high)
assert: peak, 25
probe.temp = 18
assert: lab.high, 21
assert: lab.low, 18
assert: lab.swing, 3
attic = Room("attic")
assert: attic.high, Number.NEGATIVE_INFINITY
Sensor(attic, 30)
assert: peak, 30
---
# Nucleoid uses extremes in functions methods and conditional expressions

class Team(name, scores):
    this.name = name
    this.scores = scores
    def best(): return this.scores.max()
def worst(team) => team.scores.min()
red = Team("red", [4, 7, 9])
blue = Team("blue", [12, 6, 8])
leader = red.best() > blue.best() ? red.name : blue.name
champion = Team.max(team -> team.best())
steady = Team.max(team -> team.scores.min())
floor = worst(red)
assert: leader, "blue"
assert: champion, 12
assert: steady, 6
assert: floor, 4
blue.scores.push(3)
red.scores = [9, 15]
Team("green", [5, 13])
assert: leader, "red"
assert: champion, 15
assert: steady, 9
assert: floor, 9
---
# Nucleoid keeps max and min as ordinary names beside the extreme queries

max = 3
min = 1
limits = {max: 10, min: 0}
def scale min => min * 2
class Gauge(readings):
    this.readings = readings
    def max(): return this.readings.max() + 1
gauge = Gauge([4, 6])
spare = Gauge([2])
top = gauge.readings.max() + max + limits.max
low = gauge.readings.min() - min - limits.min
peaks = Gauge.max(each -> each.max())
assert: top, 19
assert: low, 3
assert: gauge.max(), 7
assert: peaks, 7
assert: scale(min), 2
assert: gauge.readings.max, null
assert: Math.max(max, min), 3
gauge.readings = [8, 5, 2]
max = 5
Gauge([12])
assert: top, 23
assert: low, 1
assert: gauge.max(), 9
assert: peaks, 13
---
# Nucleoid rolls back failed extreme updates

def checked(value):
    if value > 100:
        throw "TOO_HOT"
    return value
temps = [35, 20]
hottest = temps.max(checked)
assert: hottest, 35
failure = ""
try:
    temps.push(500)
catch error:
    failure = error
assert: failure, "TOO_HOT"
assert: temps, [35, 20]
assert: hottest, 35
temps.push(40)
assert: hottest, 40
class Probe(temp): this.temp = temp
Probe(12)
Probe(30)
coldest = Probe.min(probe -> checked(probe.temp))
failure = ""
try:
    Probe(500)
catch error:
    failure = error
assert: failure, "TOO_HOT"
assert: Probe.length, 2
assert: coldest, 12
Probe(9)
Probe(4)
assert: coldest, 4
---
# Nucleoid removes duplicate list items with distinct

visits = ["lab", "den", "lab", "hall", "den"]
rooms = visits.distinct()
assert: rooms, ["lab", "den", "hall"]
visits.push("attic")
visits.push("hall")
assert: rooms, ["lab", "den", "hall", "attic"]
visits.shift()
assert: rooms, ["den", "lab", "hall", "attic"]
visits = []
assert: rooms, []
cart = {lines: [{sku: "pen", qty: 2}, {sku: "ink", qty: 1}, {sku: "pen", qty: 5}]}
skus = cart.lines.distinct(line -> line.sku)
bulk = cart.lines.filter(line -> line.qty > 1).distinct(line -> line.sku)
assert: skus, ["pen", "ink"]
assert: bulk, ["pen"]
cart.lines.push({sku: "cap", qty: 3})
assert: skus, ["pen", "ink", "cap"]
assert: bulk, ["pen", "cap"]
---
# Nucleoid lists the distinct values of class instance populations

class Order(region, rush):
    this.region = region
    this.rush = rush
regions = Order.distinct(order -> order.region)
assert: regions, []
first = Order("north", false)
second = Order("south", true)
third = Order("north", true)
rushed = Order.filter(order -> order.rush).distinct(order -> order.region)
recent = Order[-3:].distinct(order -> order.region)
assert: regions, ["north", "south"]
assert: rushed, ["south", "north"]
assert: recent, ["north", "south"]
first.region = "east"
Order("south", true)
assert: regions, ["east", "south", "north"]
assert: rushed, ["south", "north"]
assert: recent, ["south", "north"]
delete third.region
assert: regions, ["east", "south", null]
delete third.rush
delete third
assert: regions, ["east", "south"]
assert: rushed, ["south"]
assert: recent, ["east", "south"]
Order("east", false)
assert: recent, ["south", "east"]
---
# Nucleoid compares distinct values with deep equality

inputs = [2, "2", true, 2, null, "2", null]
kinds = inputs.distinct()
assert: kinds, [2, "2", true, null]
inputs.push(1)
inputs.push(-0)
inputs.push(0)
assert: kinds, [2, "2", true, null, 1, 0]
assert: String(1 / kinds[5]), "-Infinity"
inputs.push(0 / 0)
inputs.push(0 / 0)
assert: kinds.length, 8
places = [{x: 1, y: 2}, {x: 1, y: 2}, {x: 2, y: 1}]
spots = places.distinct()
assert: spots, [{x: 1, y: 2}, {x: 2, y: 1}]
places.push({y: 2, x: 1})
places.push({x: 1, y: 2, z: 0})
assert: spots.length, 3
paths = [[1, 2], [2, 1], [1, 2]]
routes = paths.distinct()
assert: routes, [[1, 2], [2, 1]]
paths.push([1, [2]])
paths.push([1, [2]])
assert: routes, [[1, 2], [2, 1], [1, [2]]]
class Robot(name): this.name = name
first = Robot("ada")
second = Robot("ada")
crew = [first, first]
team = crew.distinct()
assert: team.length, 1
crew.push(second)
assert: team.length, 2
---
# Nucleoid calls the distinct selector once per item with only the item

seen = []
def zone(stop, position):
    seen.push(position ?? stop.city)
    return stop.zone
stops = [{city: "Oslo", zone: 1}, {city: "Rome", zone: 3}, {city: "Bergen", zone: 1}]
def zones() => stops.distinct(zone)
assert: zones(), [1, 3]
assert: seen, ["Oslo", "Rome", "Bergen"]
stops.push({city: "Nice", zone: 2})
assert: zones(), [1, 3, 2]
assert: seen, ["Oslo", "Rome", "Bergen", "Oslo", "Rome", "Bergen", "Nice"]
limit = 2
def band stop => stop.zone > limit ? "far" : "near"
bands = stops.distinct(band)
assert: bands, ["near", "far"]
limit = 0
assert: bands, ["far"]
missing = null
skipped = missing.distinct(zone(9))
assert: skipped, null
assert: seen, ["Oslo", "Rome", "Bergen", "Oslo", "Rome", "Bergen", "Nice"]
---
# Nucleoid keeps distinct values live in class level rules

class Customer(name): this.name = name
class Order(customer, region, total):
    this.customer = customer
    this.region = region
    this.total = total
$Order.size = $Order.total > 100 ? "large" : "small"
$Customer.regions = Order.filter(order -> order.customer == $Customer).distinct(order -> order.region)
$Customer.sizes = Order.filter(order -> order.customer == $Customer).distinct(order -> order.size)
$Customer.spread = $Customer.regions.length
ada = Customer("ada")
bo = Customer("bo")
Order(ada, "north", 50)
big = Order(ada, "south", 150)
Order(bo, "north", 20)
Order(ada, "north", 80)
sizes = Order.distinct(order -> order.size)
assert: ada.regions, ["north", "south"]
assert: ada.sizes, ["small", "large"]
assert: bo.spread, 1
assert: sizes, ["small", "large"]
big.total = 90
assert: ada.sizes, ["small"]
assert: sizes, ["small"]
cy = Customer("cy")
assert: cy.regions, []
assert: cy.spread, 0
Order(cy, "east", 300)
Order(cy, "east", 60)
assert: cy.regions, ["east"]
assert: sizes, ["small", "large"]
---
# Nucleoid uses distinct values in functions methods and conditional expressions

class Post(author, tags):
    this.author = author
    this.tags = tags
    def topics(): return this.tags.distinct()
def sizes(post) => post.tags.distinct(tag -> tag.length)
first = Post("ada", ["rust", "logic", "rust"])
second = Post("bo", ["logic", "go"])
inuse = Post.flatMap(post -> post.topics()).distinct()
broader = first.topics().length > second.topics().length ? first.author : second.author
lengths = sizes(first)
assert: inuse, ["rust", "logic", "go"]
assert: broader, "bo"
assert: lengths, [4, 5]
second.tags.pop()
first.tags.push("graphs")
Post("cy", ["go", "sql"])
assert: inuse, ["rust", "logic", "graphs", "go", "sql"]
assert: broader, "ada"
assert: lengths, [4, 5, 6]
---
# Nucleoid keeps distinct as an ordinary name beside the distinct query

distinct = 2
report = {distinct: 3}
def twice distinct => distinct * 2
class Shelf(items):
    this.items = items
    def distinct(): return this.items.distinct().length
shelf = Shelf(["pen", "ink", "pen"])
spare = Shelf(["cap", "pen"])
kinds = shelf.items.distinct()
count = shelf.distinct() + distinct + report.distinct
shelves = Shelf.distinct(each -> each.distinct())
assert: kinds, ["pen", "ink"]
assert: count, 7
assert: shelves, [2]
assert: twice(distinct), 4
assert: shelf.items.distinct, null
shelf.items.push("cap")
distinct = 4
Shelf(["ink", "ink"])
assert: kinds, ["pen", "ink", "cap"]
assert: count, 10
assert: shelves, [3, 2, 1]
---
# Nucleoid rolls back failed distinct updates

def checked(zone):
    if zone == "void":
        throw "BAD_ZONE"
    return zone
zones = ["a", "b", "a"]
seen = zones.distinct(checked)
assert: seen, ["a", "b"]
failure = ""
try:
    zones.shift()
    zones.push("void")
catch error:
    failure = error
assert: failure, "BAD_ZONE"
assert: zones, ["a", "b", "a"]
assert: seen, ["a", "b"]
zones.push("c")
assert: seen, ["a", "b", "c"]
class Parcel(zone): this.zone = zone
Parcel("b")
Parcel("a")
routes = Parcel.distinct(parcel -> checked(parcel.zone))
failure = ""
try:
    Parcel("void")
catch error:
    failure = error
assert: failure, "BAD_ZONE"
assert: Parcel.length, 2
assert: routes, ["b", "a"]
Parcel("a")
Parcel("c")
assert: routes, ["b", "a", "c"]
---
# Nucleoid loops over object entries in insertion order

class Line(shelf, item, count):
    this.shelf = shelf
    this.item = item
    this.count = count
class Shelf(name):
    this.name = name
    this.units = 0
    def stock(items):
        for item, count in items:
            Line(this.name, item, count)
            this.units += count ?? 0
        return this.units
none = {}
dry = {rice: 4, beans: 1, oil: null, tea: none.tea}
dry.flour = 6
delete dry.beans
dry.beans = 2
pantry = Shelf("pantry")
assert: [pantry.stock(dry), Shelf("fridge").stock({milk: 2, eggs: 12}), Shelf("cellar").stock({wine: 3, cider: 1})], [12, 14, 4]
assert: Line.map(line -> line.item).join(" "), "rice oil flour beans milk eggs wine cider"
assert: Line.filter(line -> line.count == null).map(line -> line.shelf + ":" + line.item), ["pantry:oil"]
---
# Nucleoid loops over the entries an instance holds

class Room(name, width, depth):
    this.name = name
    this.width = width
    this.depth = depth
$Room.area = $Room.width * $Room.depth
class Lamp(watts): this.watts = watts
class Fact(subject, field, data):
    this.subject = subject
    this.field = field
    this.data = data
def record(thing):
    for field, value of thing: Fact(thing.id, field, value)
    return Fact.length
lab = Room("lab", 3, 4)
den = Room("den", 2, 2)
desk = Lamp(60)
desk.on = true
lab.width = 5
delete lab.name
lab.name = "studio"
try:
    delete den.width
    den.width = 7
    throw "BUSY"
catch error:
    pass
assert: [record(lab), record(den), record(desk)], [5, 10, 13]
assert: Fact.filter(fact -> fact.subject == "lab").map(fact -> fact.field).join(" "), "id width depth area name"
assert: Fact.filter(fact -> fact.subject == "den").map(fact -> fact.field + "=" + fact.data).join(" "), "id=den name=den width=2 depth=2 area=4"
assert: Fact.filter(fact -> fact.data == 20 or fact.data == true).map(fact -> fact.subject + "." + fact.field), ["lab.area", "desk.on"]
---
# Nucleoid snapshots object entries before loop bodies

bills = {rent: 900, food: 300, travel: 200, gym: 50}
seen = []
for bill, amount of bills:
    seen.push(bill + ":" + amount + ":" + (bills[bill] ?? "gone"))
    bills.food = 0
    delete bills.travel
    bills.fee = (bills.fee ?? 0) + 5
assert: seen, ["rent:900:900", "food:300:0", "travel:200:gone", "gym:50:50"]
assert: [bills.food, bills.travel, bills.fee], [0, null, 20]
owed = {total: 0}
for bill, amount of bills: owed.total += amount
assert: owed.total, 970
---
# Nucleoid nests object entry loops over nested records

class Store(region, city, units):
    this.region = region
    this.city = city
    this.units = units
class Region(name, stores):
    this.name = name
    this.stores = stores
regions = {north: {oslo: 4, bergen: 2}, south: {rome: 5, bari: 1, napoli: 3}, east: {riga: 6, vilnius: 2}}
for region, cities of regions:
    for city, units of cities: Store(region, city, units)
    Region(region, Store.length)
regions.south.bari = 7
assert: Store.map(store -> store.city).join(" "), "oslo bergen rome bari napoli riga vilnius"
assert: Region.map(each -> each.name + each.stores).join(" "), "north2 south5 east7"
assert: [Store.filter(store -> store.region == "south").sum(store -> store.units), regions.south.bari], [9, 7]
---
# Nucleoid returns from object entry loops in functions and methods

class Pick(item, qty):
    this.item = item
    this.qty = qty
class Store(stock):
    this.stock = stock
    def pick(order):
        for item, qty of order:
            if (this.stock[item] ?? 0) < qty:
                return item
            Pick(item, qty)
        return "done"
def load(truck, crates):
    for crate, kg of crates:
        if truck.kg + kg > truck.limit:
            return
        truck.kg += kg
shop = Store({nuts: 5, bolts: 2, gears: 7})
assert: shop.pick({nuts: 3, gears: 2}), "done"
assert: shop.pick({gears: 1, bolts: 4, nuts: 1}), "bolts"
assert: Pick.map(pick -> pick.item).join(" "), "nuts gears gears"
van = {kg: 0, limit: 10}
load(van, {tools: 4, tiles: 9, paint: 2})
assert: van.kg, 4
load(van, {paint: 2, rope: 3})
assert: van.kg, 9
---
# Nucleoid keeps object entry loops one shot

class Lamp(room, watts):
    this.room = room
    this.watts = watts
def light(rooms):
    for room, watts of rooms: Lamp(room, watts)
    return Lamp.length
load = Lamp.sum(lamp -> lamp.watts)
lights = {hall: 40, desk: 60, porch: 25}
night = false
if night: light(lights)
assert: Lamp.length, 0
night = true
lights.desk = 100
lights.shed = 30
assert: [Lamp.length, load], [3, 125]
lights = {shed: 30, gate: 10, pool: 20, yard: 5}
assert: [Lamp.length, load], [7, 190]
bill = {watts: 0}
for room, watts of lights: bill.watts += watts
lights.gate = 50
assert: [Lamp.map(lamp -> lamp.room).join(" "), bill.watts], ["hall desk porch shed gate pool yard", 65]
---
# Nucleoid restores object entries at nested savepoints

class Seat(row, guest, price):
    this.row = row
    this.guest = guest
    this.price = price
def seat(row, guests):
    for guest, price of guests:
        if price > 50: throw guest + " is over budget"
        Seat(row, guest, price)
    return Seat.length
party = {ann: 30, bob: 40, cy: 20}
try:
    delete party.ann
    try:
        delete party.bob
        party.bob = 60
        seat("a", party)
    catch error:
        assert: [error, Seat.length], ["bob is over budget", 0]
    party.ann = 35
    assert: seat("b", party), 3
    assert: Seat.map(each -> each.guest + each.price).join(" "), "bob40 cy20 ann35"
    throw "CANCELLED"
catch error:
    assert: [error, Seat.length], ["CANCELLED", 0]
assert: seat("c", party), 3
assert: Seat.map(each -> each.row + each.guest + each.price).join(" "), "cann30 cbob40 ccy20"
---
# Nucleoid restores object entries in place when it rolls back

class Line(order, item, qty):
    this.order = order
    this.item = item
    this.qty = qty
class Order(name):
    this.name = name
    this.items = []
    def fill(stock):
        for item, qty of stock:
            if qty > 5: throw item + " is short"
            Line(this.name, item, qty)
            this.items.push(item + qty)
        return this.items
basket = {tea: 4, jam: 3, oats: 2}
first = Order("first")
try:
    basket.tea = 9
    delete basket.jam
    delete basket.tea
    basket.tea = 6
    first.fill(basket)
catch error:
    assert: [error, first.items, Line.length], ["tea is short", [], 0]
second = Order("second")
assert: second.fill(basket), ["tea4", "jam3", "oats2"]
plan = suppose:
    delete basket.jam
    basket.rye = 5
    Order("third").fill(basket)
assert: plan, ["tea4", "oats2", "rye5"]
basket.salt = 1
fourth = Order("fourth")
assert: fourth.fill(basket), ["tea4", "jam3", "oats2", "salt1"]
assert: Line.map(line -> line.order + ":" + line.item).join(" "), "second:tea second:jam second:oats fourth:tea fourth:jam fourth:oats fourth:salt"
---
# Nucleoid repeats a while loop until its condition fails

level = 0
full = level >= 10
steps = 0
while not full:
    level = level + 3
    steps = steps + 1
assert(level, 12)
assert(steps, 4)
assert(full, true)
level = 4
assert(full, false)
assert(steps, 4)
---
# Nucleoid skips a while loop whose condition starts falsy

class Counter(count):
    this.count = count
    this.ticks = 0
busy = Counter(3)
idle = Counter(0)
blank = Counter("")
unset = Counter(null)
for counter of Counter:
    while counter.count:
        counter.count = counter.count - 1
        counter.ticks = counter.ticks + 1
assert(busy.count, 0)
assert(busy.ticks, 3)
assert(idle.ticks, 0)
assert(blank.ticks, 0)
assert(unset.ticks, 0)
assert(blank.count, "")
---
# Nucleoid runs while loops in methods with parameters and return

class Tank(level):
    this.level = level
    this.cycles = 0
    def drain(rate):
        while this.level > 0:
            if this.level < rate:
                return "partial"
            this.level = this.level - rate
            this.cycles = this.cycles + 1
        return "empty"
big = Tank(9)
small = Tank(7)
assert(big.drain(3), "empty")
assert(big.level, 0)
assert(big.cycles, 3)
assert(small.drain(3), "partial")
assert(small.level, 1)
assert(small.cycles, 2)
assert(Tank(0).drain(3), "empty")
---
# Nucleoid nests while loops with conditionals and for loops

class Shelf(stock):
    this.stock = stock
a = Shelf(0)
b = Shelf(3)
low = Shelf.filter(s => s.stock < 3).length
rounds = 0
moves = 0
while low > 0:
    for s of Shelf:
        if s.stock < 3:
            while s.stock < 3:
                s.stock = s.stock + 2
                moves = moves + 1
        else:
            s.stock = s.stock - 1
    rounds = rounds + 1
assert(rounds, 2)
assert(moves, 3)
assert(a.stock, 3)
assert(b.stock, 4)
assert(low, 0)
a.stock = 0
assert(low, 1)
assert(moves, 3)
---
# Nucleoid creates instances in a while loop until a class rule holds

class Cell(size):
    this.size = size
$Cell.big = $Cell.size > 4
grown = Cell.filter(c => c.big).length
while grown < 2:
    Cell(Cell.length * 2)
assert(Cell.length, 5)
assert(grown, 2)
assert(Cell.map(c => c.size), [0, 2, 4, 6, 8])
assert(Cell.map(c => c.big), [false, false, false, true, true])
---
# Nucleoid stops a runaway while loop and rolls it back

level = 0
while level < 12:
    level = level + 3
assert(level, 12)
try:
    while level != 20: level = level + 3
catch error:
    failure = error
assert(failure, TypeError("Loop did not terminate"))
assert(level, 12)
---
# Nucleoid keeps while as an ordinary name beside while loops

while = 3
count = 0
while while > 0:
    while = while - 1
    count = count + 1
assert(while, 0)
assert(count, 3)
def twice(while) => while * 2
assert(twice(4), 8)
point = {while: 1}
assert(point.while, 1)
---
# Nucleoid rolls back a while loop that fails during an iteration

class Parcel
stock = 16
shipped = 0
left = stock - shipped
try:
    while shipped < 20:
        if left < 3:
            throw "SHORT"
        Parcel()
        shipped = shipped + 3
catch error:
    failure = error
assert(failure, "SHORT")
assert(shipped, 0)
assert(left, 16)
assert(Parcel.length, 0)
try:
    while missing > 0:
        shipped = 1
catch error:
    failure = error
assert(failure, ReferenceError("missing is not defined"))
assert(shipped, 0)
---
# Nucleoid destructures object fields into standing declarations

point = {x: 3, y: 4, z: 0}
{x, y, z} = point
assert: [x, y, z], [3, 4, 0]
point.x = 6
point.z = 2
assert: [x, z], [6, 2]
point = {x: 1, y: 2}
assert: [x, y, z], [1, 2, null]
size = {w: 5}
{w, h,} = size
assert: [w, h], [5, null]
size.h = 4
size.w = 3
assert: [w, h], [3, 4]
x + y + w * h
# return: 15
---
# Nucleoid destructures renamed fields of instances

class Room(name, width, depth):
    this.name = name
    this.width = width
    this.depth = depth
$Room.area = $Room.width * $Room.depth
hall = Room("hall", 2, 5)
den = Room("den", 4, 4)
{name, area, width} = hall
{name: other area: larger, depth} = den
assert: [name, area, width], ["hall", 10, 2]
assert: [other, larger, depth], ["den", 16, 4]
hall.depth = 9
den.width = 5
assert: [area, larger], [18, 20]
hall.width = 3
den.depth = 2
hall.name = "atrium"
assert: [name, width, depth, area, larger], ["atrium", 3, 2, 27, 10]
Room.map(room -> room.area)
# return: [27,10]
---
# Nucleoid unpacks lists and strings into comma separated names

size = [4, 3]
width, height = size
area = width * height
assert: area, 12
size = [5, 2]
assert: [width, height, area], [5, 2, 10]
origin = {at: [0, 3]}
row, column, = origin.at
origin.at = [4, 1]
assert: [row, column], [4, 1]
word = "ok"
first, = word
word = "no"
assert: first, "n"
dims = [2, 7]
deep, wide = dims
dims = [6]
assert: [deep, wide], [6, null]
area + row
# return: 14
---
# Nucleoid destructures inside functions blocks and loops

class Leg(span):
    this.span = span
Leg([0, 4])
Leg([4, 10])
def length(leg):
    start, end = leg.span
    return end - start
covered = 0
for leg of Leg:
    covered = covered + length(leg)
assert: [covered, start, end], [10, 4, 10]
point = {x: 1, y: 2, z: 3}
{
    {x, y, z} = point
    norm = x * x + y * y + z * z
}
assert: [x, y, z, norm], [1, 2, 3, 14]
point.z = 5
assert: [z, norm], [5, 30]
---
# Nucleoid destructures in conditional branches and hypotheticals

pair = [3, 4]
ready = false
if ready: low, high = pair
ready = true
assert: [low, high], [3, 4]
pair = [5, 6]
assert: [low, high], [5, 6]
ready = false
pair = [7, 8]
assert: [low, high], [5, 6]
spread = suppose ready = true: high - low
assert: [spread, low, high], [1, 5, 6]
point = {x: 1, y: 2}
suppose point.y = 9:
    {x, y} = point
    assert: [x, y], [1, 9]
assert: point.y, 2
---
# Nucleoid explains destructured names as their declarations

point = {x: 3, y: 4}
pair = [5, 6]
{x, y} = point
first, second = pair
assert: (affects point).map(step -> step.rule), ["x = point.x", "y = point.y"]
assert: (affects pair).map(step -> step.rule), ["first = pair[0]", "second = pair[1]"]
assert: (why second)[0].from, ["pair"]
class Point(x, y):
    this.x = x
    this.y = y
{x, y} = Point(1, 2)
assert: [x, y, Point.length], [1, 2, 2]
assert: (why y)[0].rule, "y = Point(1,2).y"
assert: (affects point).length, 0
---
# Nucleoid keeps block object and list readings beside destructuring

point = {x: 1, y: 2}
{x, y} = point
{ total = x + y }
box = {x, y}
record = {left: x right: y}
pair = [3, 4]
low, high = pair
add = lambda a, b: a + b
positive = all v in pair: v > 0
failure = ""
try:
    [a, b] = pair
catch error:
    failure = error
assert: failure, SyntaxError("Cannot assign to [a,b]")
assert: [total, box, record], [3, {x: 1, y: 2}, {left: 1, right: 2}]
assert: [add(low, high), positive], [7, true]
point.x = 5
pair = [-1, 6]
assert: [total, box.x, record.left, low, high, positive], [7, 5, 5, -1, 6, false]
{ x }
# return: 5
---
# Nucleoid rolls back failed destructuring

limits = [4, 50]
low, high = limits
if high - low > 100: throw "WIDE"
widened = ""
try: limits = [0, 500]
catch error: widened = error
wide = [9, 300]
moved = ""
try: low, high = wide
catch error: moved = error
assert: [widened, moved], ["WIDE", "WIDE"]
assert: [low, high, limits], [4, 50, [4, 50]]
order = {qty: 2, price: 5}
{qty, price} = order
if qty * price > 50: throw "COSTLY"
costly = ""
try: order.price = 40
catch error: costly = error
assert: [costly, order.price, qty, price], ["COSTLY", 5, 2, 5]
order.qty = 3
order.price = 7
limits = [3, 60]
assert: [low, high, qty, price], [3, 60, 3, 7]
---
# Nucleoid counts matching list items

readings = [2, 5, 8, 11]
threshold = 6
high = readings.count(reading -> reading > threshold)
assert: high, 2
readings.push(14)
assert: high, 3
threshold = 10
assert: high, 2
readings.shift()
assert: high, 2
readings = []
assert: high, 0
cart = {items: [0, 1, 2, 0, 3]}
nonzero = cart.items.count(item -> item != 0)
assert: nonzero, 3
cart.items.push(4)
assert: nonzero, 4
---
# Nucleoid counts matching class instances

class Ticket(open, priority):
    this.open = open
    this.priority = priority
pending = Ticket.count(ticket -> ticket.open)
assert: pending, 0
first = Ticket(true, 2)
second = Ticket(false, 5)
third = Ticket(true, 9)
important = Ticket.count(ticket -> ticket.open and ticket.priority >= 5)
assert: pending, 2
assert: important, 1
second.open = true
assert: pending, 3
assert: important, 2
third.priority = 1
assert: important, 1
delete first.open
delete first.priority
delete first
assert: pending, 2
third.open = false
assert: pending, 1
assert: important, 1
---
# Nucleoid keeps predicate counts live in class level rules

class Team(name): this.name = name
class Task(team, done):
    this.team = team
    this.done = done
$Team.pending = Task.count(task -> task.team == $Team and not task.done)
alpha = Team("alpha")
beta = Team("beta")
assert: alpha.pending, 0
assert: beta.pending, 0
first = Task(alpha, false)
second = Task(alpha, true)
third = Task(beta, false)
assert: alpha.pending, 1
assert: beta.pending, 1
first.team = beta
assert: alpha.pending, 0
assert: beta.pending, 2
second.done = false
assert: alpha.pending, 1
delete third.team
delete third.done
delete third
assert: beta.pending, 1
---
# Nucleoid uses predicate counts beside ordinary count names

cutoff = 3
count = 7
report = {count: 9}
class Bag(items):
    this.items = items
    def count(): return this.items.count(item -> item > cutoff)
bag = Bag([1, 4, 8])
def matching(items) => items.count(item -> item > cutoff)
assert: bag.count(), 2
assert: matching([2, 5, 9]), 2
assert: Bag.filter(each -> each.count() > 1).length, 1
kind = Bag
assert: kind.filter(each -> each.count() > 1).length, 1
assert: [count, report.count], [7, 9]
cutoff = 6
assert: bag.count(), 1
assert: matching([2, 5, 9]), 1
assert: Bag.filter(each -> each.count() > 1).length, 0
---
# Nucleoid collects surplus positional arguments in fresh rest lists

def collect(head = 1, ...tail) => [head, tail]
empty = collect()
one = collect(0)
many = collect(2, 3, 4)
nested = collect(false, "", [5, 6])
assert(empty, [1, []])
assert(one, [0, []])
assert(many, [2, [3, 4]])
assert(nested, [false, ["", [5, 6]]])
assert(collect(null).value, [null, []])
many
# return: [2,[3,4]]
---
# Nucleoid shares rest binding across anonymous functions callbacks and methods

arrow = (...items) -> items
blocked = function(head, ...tail) { return [head, tail] }
def collect(...items) => items
alias = collect
record = {call: collect}
callbacks = [collect]
assert(arrow(1, 2), [1, 2])
assert(blocked(1, 2, 3), [1, [2, 3]])
assert(alias(), [])
assert(record.call(3, 4), [3, 4])
assert(callbacks[0](5), [5])
assert([2, 3].map((item, ...extras) => [item, extras]), [[2, []], [3, []]])
class Meter(seed):
    this.seed = seed
    def read(...items) => [self.seed, items]
meter = Meter(7)
assert(meter.read(8, 9), [7, [8, 9]])
assert(meter.read(), [7, []])
---
# Nucleoid keeps fixed constructor labels separate from positional rest values

class Packet(head = 1, ...tail):
    this.head = head
    this.tail = tail
empty = Packet()
many = Packet(2, 3, 4)
named = Packet(head=5)
nested = Packet(6, [7, 8])
assert(empty.tail, [])
assert(many.tail, [3, 4])
assert(named.head, 5)
assert(named.tail, [])
assert(nested.tail, [[7, 8]])
class Pair(first = 1, second = 2, ...tail):
    this.values = [first, second, tail]
mixed = Pair(4, second=7)
assert(mixed.values, [4, 7, []])
try:
    rejected = Packet(tail=[9])
catch error:
    assert(error, TypeError("Rest parameter 'tail' cannot be a named constructor argument for Packet"))
class Base:
    def init(...items):
        this.items = items
class Inherited: Base
    pass
class Child: Base
    def init(...items):
        super(1, 2)
        this.own = items
inherited = Inherited(3, 4)
child = Child(5, 6)
assert(inherited.items, [3, 4])
assert(child.items, [1, 2])
assert(child.own, [5, 6])
---
# Nucleoid initializes rest histories without capture or shared list mutation

tail = [100]
def history(...tail):
    tail.push(9)
    return [tail@0, tail@last]
one = history(1, 2)
two = history()
three = history()
assert(one, [[1, 2], [1, 2, 9]])
assert(two, [[], [9]])
assert(three, [[], [9]])
assert(tail, [100])
def factory(...tail) => () => tail
callback = factory(7, 8)
assert(callback(), [100])
state = 1
def attempt(...tail):
    try:
        tail.push(8)
        state = 2
        throw "LOCAL_REST"
    catch error:
        state = state
    return [tail@0, tail, state]
assert(attempt(1), [[1], [1, 8], 1])
assert(state, 1)
---
# Nucleoid restores variadic definitions and their reactive callers on rollback

unit = 2
def total(...items) => items.sum() * unit
live = total
frozen = total.value
observed = live(1, 2)
assert(observed, 6)
def total(head = 10, ...tail) => (head + tail.length) * unit
assert(observed, 4)
assert(frozen(1, 2), 6)
try:
    def total(...items) => 99
    assert(observed, 99)
    throw "RESTORE_REST"
catch error:
    assert(error, "RESTORE_REST")
assert(observed, 4)
unit = 3
assert(observed, 6)
assert(frozen(1, 2), 9)
observed
# return: 6
---
# Nucleoid finds the most frequent list item with mode

sizes = ["M", "S", "M", "L"]
top = sizes.mode()
assert: top, "M"
sizes.push("L")
sizes.push("L")
assert: top, "L"
sizes.shift()
assert: top, "L"
sizes = []
assert: top, null
sizes = [3, 4, 4, 3]
assert: top, 3
cart = {lines: [{sku: "pen", qty: 2}, {sku: "ink", qty: 1}, {sku: "ink", qty: 5}]}
common = cart.lines.mode(line -> line.sku)
bulk = cart.lines.filter(line -> line.qty > 1).mode(line -> line.sku)
assert: common, "ink"
assert: bulk, "pen"
cart.lines.push({sku: "ink", qty: 4})
assert: bulk, "ink"
cart.lines.push({sku: "pen", qty: 6})
cart.lines.push({sku: "pen", qty: 7})
assert: common, "pen"
assert: bulk, "pen"
---
# Nucleoid finds the most frequent value of class instance populations

class Vote(choice, round):
    this.choice = choice
    this.round = round
favorite = Vote.mode(vote -> vote.choice)
assert: favorite, null
first = Vote("tea", 1)
Vote("coffee", 1)
second = Vote("coffee", 2)
late = Vote.filter(vote -> vote.round > 1).mode(vote -> vote.choice)
recent = Vote[-2:].mode(vote -> vote.choice)
assert: favorite, "coffee"
assert: late, "coffee"
assert: recent, "coffee"
second.choice = "tea"
assert: favorite, "tea"
assert: late, "tea"
assert: recent, "coffee"
Vote("juice", 2)
Vote("juice", 3)
assert: favorite, "tea"
assert: late, "juice"
assert: recent, "juice"
delete second.choice
delete second.round
delete second
assert: favorite, "juice"
kind = Vote
assert: kind.mode(vote -> vote.round), 1
---
# Nucleoid compares mode items with deep equality

inputs = [2, "2", true, "2", 2]
common = inputs.mode()
assert: common, 2
inputs.push("2")
assert: common, "2"
inputs.push(null)
inputs.push(inputs[99])
inputs.push(null)
inputs.push(null)
assert: common, null
signs = [-0, 1, 0, 1]
zero = signs.mode()
assert: String(1 / zero), "-Infinity"
readings = [0 / 0, 0 / 0, 5]
level = readings.mode()
assert: String(level), "NaN"
readings.push(5)
assert: level, 5
first = {x: 1, y: 2}
places = [{x: 2, y: 1}, first, {y: 2, x: 1}]
spot = places.mode()
assert: spot == first, true
held = {k: null, m: 1}
gap = {k: {}.none, m: 1, z: 5}
records = [held, gap]
kept = records.mode()
assert: kept == held, true
records = [gap, held]
assert: kept == gap, true
class Robot(name): this.name = name
ada = Robot("ada")
twin = Robot("ada")
crew = [ada, twin, twin]
lead = crew.mode()
assert: lead == twin, true
crew.push(ada)
assert: lead == ada, true
---
# Nucleoid calls the mode selector once per item with only the item

seen = []
def size(order, position):
    seen.push(position ?? order.id)
    return order.size
orders = [{id: 1, size: "M"}, {id: 2, size: "S"}, {id: 3, size: "S"}]
def common() => orders.mode(size)
assert: common(), "S"
assert: seen, [1, 2, 3]
orders.push({id: 4, size: "M"})
assert: common(), "M"
assert: seen, [1, 2, 3, 1, 2, 3, 4]
limit = 2
def band order => order.id > limit ? "late" : "early"
phase = orders.mode(band)
assert: phase, "early"
limit = 0
assert: phase, "late"
missing = null
skipped = missing.mode(size(9))
assert: skipped, null
assert: seen, [1, 2, 3, 1, 2, 3, 4]
---
# Nucleoid keeps modes live in class level rules

class Team(name): this.name = name
class Vote(team, choice):
    this.team = team
    this.choice = choice
$Team.favorite = Vote.filter(vote -> vote.team == $Team).mode(vote -> vote.choice)
red = Team("red")
blue = Team("blue")
Vote(red, "tea")
swing = Vote(red, "coffee")
Vote(red, "coffee")
Vote(blue, "juice")
overall = Vote.mode(vote -> vote.choice)
assert: red.favorite, "coffee"
assert: blue.favorite, "juice"
assert: overall, "coffee"
swing.choice = "tea"
assert: red.favorite, "tea"
assert: overall, "tea"
green = Team("green")
assert: green.favorite, null
Vote(green, "juice")
Vote(green, "juice")
assert: green.favorite, "juice"
assert: overall, "juice"
---
# Nucleoid uses modes in functions methods and conditional expressions

class Shop(name, sales):
    this.name = name
    this.sales = sales
    def bestseller(): return this.sales.mode()
def typical(shop) => shop.sales.mode(sale -> sale.length)
north = Shop("north", ["tea", "cake", "tea"])
south = Shop("south", ["pie", "cake", "cake"])
hit = north.bestseller() == south.bestseller() ? "shared" : north.bestseller()
size = typical(north)
trend = Shop.mode(shop -> shop.bestseller())
assert: hit, "tea"
assert: size, 3
assert: trend, "tea"
north.sales.push("cake")
north.sales.push("cake")
assert: hit, "shared"
assert: size, 4
assert: trend, "cake"
---
# Nucleoid keeps mode as an ordinary name beside the mode query

mode = 2
report = {mode: 3}
def twice mode => mode * 2
class Shelf(items):
    this.items = items
    def mode(): return this.items.mode()
shelf = Shelf(["pen", "ink", "pen"])
spare = Shelf(["cap", "cap"])
common = shelf.items.mode()
label = shelf.mode() + "/" + spare.mode()
shelves = Shelf.mode(each -> each.mode())
assert: common, "pen"
assert: label, "pen/cap"
assert: shelves, "pen"
assert: [mode, report.mode, twice(mode)], [2, 3, 4]
assert: shelf.items.mode, null
assert: Shelf.mode() == shelf, true
spare.items.push("ink")
Shelf(["cap"])
assert: shelves, "cap"
shelf.items.push("ink")
shelf.items.push("ink")
assert: common, "ink"
assert: label, "ink/cap"
---
# Nucleoid rolls back failed mode updates

def checked(zone):
    if zone == "void":
        throw "BAD_ZONE"
    return zone
zones = ["a", "b", "b"]
busiest = zones.mode(checked)
assert: busiest, "b"
failure = ""
try:
    zones.shift()
    zones.push("void")
catch error:
    failure = error
assert: failure, "BAD_ZONE"
assert: zones, ["a", "b", "b"]
assert: busiest, "b"
zones.push("a")
zones.push("a")
assert: busiest, "a"
class Parcel(zone): this.zone = zone
Parcel("b")
Parcel("a")
Parcel("a")
routes = Parcel.mode(parcel -> checked(parcel.zone))
failure = ""
try:
    Parcel("void")
catch error:
    failure = error
assert: failure, "BAD_ZONE"
assert: Parcel.length, 3
assert: routes, "a"
Parcel("b")
Parcel("b")
assert: routes, "b"
---
# Nucleoid reads noun list queries with of

prices = [4, 9, 2, 9]
total = sum of prices
top = max of prices
low = min of prices
common = mode of prices
assert: [total, top, low, common], [24, 9, 2, 9]
prices.push(2)
prices.push(2)
assert: [total, top, low, common], [28, 9, 2, 2]
prices.shift()
assert: [total, common], [24, 2]
prices = [1, 3, 3]
assert: [total, top, low, common], [7, 3, 1, 3]
assert: sum of prices, prices.sum()
assert: max of prices, prices.max()
assert: min of prices, prices.min()
cart = {lines: [{sku: "pen", qty: 2}, {sku: "ink", qty: 5}]}
units = sum of cart.lines.map(line -> line.qty)
favorite = mode of cart.lines.map(line -> line.sku)
assert: units, cart.lines.sum(line -> line.qty)
assert: favorite, "pen"
cart.lines.push({sku: "ink", qty: 1})
assert: [units, favorite], [8, "ink"]
---
# Nucleoid reads noun queries of class instance populations with of

class Ticket(queue, minutes):
    this.queue = queue
    this.minutes = minutes
busiest = mode of Ticket.map(ticket -> ticket.queue)
longest = max of Ticket.map(ticket -> ticket.minutes)
assert: busiest, null
assert: longest, Number.NEGATIVE_INFINITY
first = Ticket("billing", 30)
Ticket("support", 5)
second = Ticket("support", 12)
assert: [busiest, longest], ["support", 30]
recent = min of Ticket[1:].map(ticket -> ticket.minutes)
kind = Ticket
workload = sum of kind.map(ticket -> ticket.minutes)
listed = sum of Ticket.list().map(ticket -> ticket.minutes)
assert: [recent, workload, listed], [5, 47, 47]
first.minutes = 8
second.queue = "billing"
assert: [busiest, longest, workload], ["billing", 12, 25]
assert: mode of Ticket, first
delete second.queue
delete second.minutes
delete second
assert: [busiest, longest, recent, workload], ["billing", 8, 5, 13]
---
# Nucleoid keeps noun queries live in class level rules

class Obj(parts): this.parts = parts
$Obj.fill_color = mode of $Obj.parts
$Obj.brightest = max of $Obj.parts
first = Obj([3, 1, 3])
second = Obj([2, 2, 5])
assert: [first.fill_color, first.brightest], [3, 3]
assert: [second.fill_color, second.brightest], [2, 5]
first.parts.push(1)
first.parts.push(1)
second.parts = [7]
assert: [first.fill_color, second.fill_color, second.brightest], [1, 7, 7]
class Team(name): this.name = name
class Job(team, hours):
    this.team = team
    this.hours = hours
$Team.load = sum of Job.filter(job -> job.team == $Team).map(job -> job.hours)
red = Team("red")
blue = Team("blue")
Job(red, 3)
long = Job(blue, 8)
Job(red, 2)
assert: [red.load, blue.load], [5, 8]
long.team = red
assert: [red.load, blue.load], [13, 0]
long.hours = 1
assert: red.load, 6
---
# Nucleoid binds noun queries as prefix operators

xs = [1, 2, 3]
assert: sum of xs + 1, 7
assert: 1 + sum of xs * 2, 13
assert: sum of xs.map(x -> x * 10), 60
assert: (sum of xs) ** 2, 36
assert: 2 ** max of xs, 8
assert: -min of xs, -1
assert: typeof mode of xs, "number"
assert: max of xs > min of xs, true
rows = [[1, 2], [1, 2], [5]]
assert: sum of mode of rows, 3
assert: max of rows.map(row -> sum of row), 5
missing = null
assert: sum of missing ?? 0, 0
ready = false
assert: sum of xs if ready else max of xs, 3
failure = ""
try:
    sum of xs ** 2
catch error:
    failure = error
assert: failure, TypeError("number.sum is not a function")
---
# Nucleoid keeps names and loop variables named like noun queries

def sum(values):
    return 0
max = 9
records = [{mode: 4}, {mode: 4}, {mode: 6}]
assert: sum([1, 2]), 0
assert: sum of [1, 2], 3
assert: max, 9
assert: max of [1, 2], 2
assert: mode of records.map(record -> record.mode), 4
class Item(size): this.size = size
Item(4)
Item(6)
sizes = []
for mode of Item:
    sizes.push(mode.size)
assert: sizes, [4, 6]
for min in Item: sizes.push(min.size * 10)
assert: sizes, [4, 6, 40, 60]
total = [1, 2, 3].reduce(lambda sum, x: sum + x, 0)
assert: total, 6
label = `${max of [1, 2]}`
grouped = `${(max of [1, 2])}`
assert: [label, grouped], ["9", "2"]
---
# Nucleoid reads noun queries in headers and nested expressions

class Item(size): this.size = size
small = Item(1)
large = Item(2)
groups = [[small, large], [small, large], [large]]
sizes = []
for item of mode of groups:
    sizes.push(item.size)
assert: sizes, [1, 2]
queue = [1]
while sum of queue < 4:
    queue.push(1)
assert: queue.length, 4
alert = false
if max of queue > 0:
    alert = true
assert: alert, true
assert: all group in groups: sum of group.map(item -> item.size) > 1, true
def spread(values) => max of values - min of values
assert: spread([4, 9, 1]), 8
pick = values -> mode of values
assert: pick(["a", "b", "b"]), "b"
report = {total: sum of queue, top: max of queue}
assert: [report.total, report.top], [4, 1]
assert: [max of queue, min of queue], [1, 1]
assert: String(sum of queue), "4"
ready = true
assert: ready ? sum of queue : 0, 4
assert: [10, 20, 30][max of queue], 20
---
# Nucleoid explains noun queries in their written form

prices = [4, 9, 9]
top = max of prices
words = ["ab", "c", "ab"]
width = (mode of words).length
squared = (sum of prices) ** 2
assert: (why top)[0].rule, "top = max of prices"
assert: (why top)[0].from, ["prices"]
assert: (why width)[0].rule, "width = (mode of words).length"
assert: (why squared)[0].rule, "squared = (sum of prices)**2"
assert: [top, width, squared], [9, 2, 484]
---
# Nucleoid rolls back failed noun query updates

def checked(zone):
    if zone == "void":
        throw "BAD_ZONE"
    return zone
zones = ["a", "b", "b"]
busiest = mode of zones.map(checked)
assert: busiest, "b"
failure = ""
try:
    zones.shift()
    zones.push("void")
catch error:
    failure = error
assert: failure, "BAD_ZONE"
assert: zones, ["a", "b", "b"]
assert: busiest, "b"
zones.push("a")
zones.push("a")
assert: busiest, "a"
---
# Nucleoid reads noun queries of snapshots versions and other receivers

history = [1]
history.push(4)
first = sum of history@first
frozen = sum of history.value
history.push(5)
assert: [first, frozen, sum of history], [1, 5, 10]
nothing = null
assert: sum of nothing, null
assert: mode of [], null
assert: [max of [], min of []], [Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY]
class Shelf:
    def mode(): return 9
shelf = Shelf()
assert: mode of shelf, 9
assert: mode of Shelf, shelf
record = {sum: () -> 5}
assert: sum of record, 5
failure = ""
try:
    sum of "abc"
catch error:
    failure = error
assert: failure, TypeError("abc.sum is not a function")
---
# Nucleoid finds the extreme items of lists with maxBy and minBy

readings = [{at: 9, c: 18}, {at: 12, c: 24}, {at: 15, c: 24}, {at: 18, c: 9}]
warmest = readings.maxBy(reading -> reading.c)
coldest = readings.minBy(reading -> reading.c)
assert: warmest.at, 12
assert: coldest.at, 18
assert: warmest == readings[1], true
readings.push({at: 21, c: 30})
readings.shift()
assert: warmest.at, 21
assert: coldest.at, 18
readings.pop()
readings.pop()
assert: warmest.at, 12
assert: coldest.at, 12
readings = []
assert: warmest, null
assert: coldest, null
trip = {legs: [{to: "oslo", km: 410, mins: 300}, {to: "voss", km: 110, mins: 75}]}
fastest = trip.legs.maxBy(leg -> leg.km / leg.mins)
shortest = trip.legs.minBy(leg -> leg.km)
assert: fastest.to, "voss"
assert: shortest.to, "voss"
trip.legs.push({to: "lom", km: 60, mins: 90})
trip.legs.push({to: "alta", km: 900, mins: 480})
trip.legs.push({to: "bodo", km: 300, mins: 400})
assert: fastest.to, "alta"
assert: shortest.to, "lom"
---
# Nucleoid finds the extreme instances of class populations with maxBy and minBy

class Bid(bidder, amount, open):
    this.bidder = bidder
    this.amount = amount
    this.open = open
best = Bid.maxBy(bid -> bid.amount)
cheapest = Bid.minBy(bid -> bid.amount)
assert: best, null
assert: cheapest, null
ada = Bid("ada", 120, true)
bo = Bid("bo", 90, false)
cy = Bid("cy", 150, true)
leading = Bid.filter(bid -> bid.open).minBy(bid -> bid.amount)
recent = Bid[-2:].maxBy(bid -> bid.amount)
assert: best, cy
assert: cheapest, bo
assert: leading, ada
assert: recent, cy
bo.amount = 200
ada.open = false
assert: best, bo
assert: cheapest, ada
assert: leading, cy
assert: recent, bo
delete cy.amount
assert: best, null
assert: leading, null
delete cy.open
delete cy.bidder
delete cy
assert: best, bo
assert: cheapest, ada
assert: leading, null
assert: recent, bo
---
# Nucleoid compares maxBy and minBy keys as the comparison operators do

class Job(name, due, kg):
    this.name = name
    this.due = due
    this.kg = kg
alpha = Job.minBy(job -> job.name)
soonest = Job.minBy(job -> job.due)
latest = Job.maxBy(job -> job.due)
heaviest = Job.maxBy(job -> job.kg)
press = Job("press", new Date("2026-12-01"), 40)
coat = Job("Coat", new Date("2026-11-20"), "9")
weld = Job("weld", new Date("2026-11-28"), 40)
assert: alpha, coat
assert: soonest, coat
assert: latest, press
assert: heaviest, press
coat.kg = "75"
weld.due = new Date("2026-11-09")
Job("Anvil", new Date("2026-12-24"), "100")
assert: alpha.name, "Anvil"
assert: soonest, weld
assert: latest.name, "Anvil"
assert: heaviest, coat
press.kg = 0 / 0
assert: heaviest, null
press.kg = 300
delete weld.due
assert: heaviest, press
assert: soonest, null
assert: latest, null
---
# Nucleoid calls the extreme item selector once per item with only the item

seen = []
def weigh(parcel, position):
    seen.push(position ?? parcel.id)
    return parcel.kg
parcels = [{id: 1, kg: 4}, {id: 2, kg: 9}, {id: 3, kg: 2}]
def heaviest() => parcels.maxBy(weigh)
assert: heaviest().id, 2
assert: seen, [1, 2, 3]
parcels.push({id: 4, kg: 12})
assert: heaviest().id, 4
assert: seen, [1, 2, 3, 1, 2, 3, 4]
depot = 5
def gap parcel => Math.abs(parcel.kg - depot)
closest = parcels.minBy(gap)
assert: closest.id, 1
depot = 10
assert: closest.id, 2
missing = null
skipped = missing.minBy(weigh(9))
assert: skipped, null
assert: seen, [1, 2, 3, 1, 2, 3, 4]
---
# Nucleoid keeps extreme items live in class level rules

class Team(name, target):
    this.name = name
    this.target = target
class Player(team, name, goals):
    this.team = team
    this.name = name
    this.goals = goals
$Team.star = Player.filter(player -> player.team == $Team.name).maxBy(player -> player.goals)
$Team.total = Player.filter(player -> player.team == $Team.name).sum(player -> player.goals)
$Team.fit = Player.minBy(player -> Math.abs(player.goals - $Team.target))
reds = Team("reds", 4)
blues = Team("blues", 10)
leader = Team.maxBy(team -> team.total)
ana = Player("reds", "ana", 7)
Player("reds", "ben", 3)
cal = Player("blues", "cal", 5)
assert: reds.star, ana
assert: blues.star, cal
assert: reds.fit.name, "ben"
assert: blues.fit, ana
assert: leader, reds
cal.goals = 12
assert: blues.star, cal
assert: blues.fit, cal
assert: leader, blues
cal.team = "reds"
assert: reds.star, cal
assert: blues.star, null
assert: leader, reds
greens = Team("greens", 6)
assert: greens.star, null
assert: greens.fit, ana
---
# Nucleoid uses extreme items in functions methods and conditional expressions

class Shelf(name, items):
    this.name = name
    this.items = items
    def priciest(): return this.items.maxBy(item -> item.price)
def cheapest(shelf) => shelf.items.minBy(item -> item.price)
left = Shelf("left", [{sku: "pen", price: 2}, {sku: "ink", price: 9}])
right = Shelf("right", [{sku: "pad", price: 4}, {sku: "cap", price: 1}])
dearer = left.priciest().price > right.priciest().price ? left.name : right.name
top = Shelf.maxBy(shelf -> shelf.priciest().price)
bargain = Shelf.minBy(shelf -> cheapest(shelf).price)
assert: dearer, "left"
assert: top, left
assert: bargain, right
assert: cheapest(left).sku, "pen"
assert: right.priciest().sku, "pad"
right.items.push({sku: "tape", price: 15})
left.items = [{sku: "clip", price: 0.5}]
assert: dearer, "right"
assert: top, right
assert: bargain, left
assert: cheapest(left).sku, "clip"
---
# Nucleoid keeps maxBy and minBy as ordinary names beside the extreme item queries

maxBy = 3
minBy = 1
limits = {maxBy: size -> size * 10, minBy: 2}
def scale minBy => minBy * 2
class Crate(items):
    this.items = items
    def maxBy(key): return key(this.items.length)
crate = Crate([{kg: 6}, {kg: 4}])
spare = Crate([{kg: 9}])
heavy = crate.items.maxBy(box -> box.kg)
fullest = Crate.maxBy(each -> each.maxBy(count -> count))
lightest = Crate.minBy(each -> each.items.min(box -> box.kg))
assert: heavy.kg, 6
assert: fullest, crate
assert: lightest, crate
assert: crate.maxBy(count -> count + maxBy), 5
assert: limits.maxBy(minBy), 10
assert: scale(minBy), 2
assert: crate.items.maxBy, null
spare.items = [{kg: 1}, {kg: 2}, {kg: 3}, {kg: 9}]
crate.items.push({kg: 7})
maxBy = 5
assert: heavy.kg, 7
assert: fullest, spare
assert: lightest, spare
assert: crate.maxBy(count -> count + maxBy), 8
---
# Nucleoid rolls back failed extreme item updates

def checked(value):
    if value > 100:
        throw "TOO_MANY"
    return value
orders = [{id: 1, qty: 30}, {id: 2, qty: 50}]
largest = orders.maxBy(order -> checked(order.qty))
assert: largest.id, 2
failure = ""
try:
    orders.push({id: 3, qty: 500})
catch error:
    failure = error
assert: failure, "TOO_MANY"
assert: orders.length, 2
assert: largest.id, 2
orders.push({id: 4, qty: 80})
assert: largest.id, 4
class Probe(name, temp):
    this.name = name
    this.temp = temp
Probe("a", 12)
Probe("b", 30)
coldest = Probe.minBy(probe -> checked(probe.temp))
failure = ""
try:
    Probe("c", 500)
catch error:
    failure = error
assert: failure, "TOO_MANY"
assert: Probe.length, 2
assert: coldest.name, "a"
Probe("d", 9)
assert: coldest.name, "d"
---
# Nucleoid declares subclasses with extends clauses

class Animal(name):
    this.name = name
$Animal.legs = 4
class Dog(name, breed) extends Animal:
    this.name = name
    this.breed = breed
    def speak():
        return this.name + " barks"
class Cat extends Animal:
    def speak() => this.name + " meows"
rex = Dog("Rex", "lab")
tom = Cat("Tom")
assert(rex.breed, "lab")
assert(rex.speak(), "Rex barks")
assert(tom.name, "Tom")
assert(tom.speak(), "Tom meows")
assert(rex.legs, 4)
assert(tom.legs, 4)
assert(Animal.length, 0)
assert(Dog.length, 1)
assert(Cat.length, 1)
$Animal.legs = 3
assert(rex.legs, 3)
assert(tom.legs, 3)
---
# Nucleoid declares bodyless subclasses with extends clauses

class Shape(sides):
    this.sides = sides
$Shape.polygon = $Shape.sides > 2
class Square extends Shape
square = Square(4)
class Triangle(sides) extends Shape; triangle = Triangle(3)
{ class Segment extends Shape }
segment = Segment(1)
assert(square.sides, 4)
assert(triangle.sides, 3)
assert(square.polygon, true)
assert(segment.polygon, false)
assert(Shape.length, 0)
assert(Square.length, 1)
assert(Triangle.length, 1)
assert(Segment.length, 1)
segment.sides = 5
assert(segment.polygon, true)
---
# Nucleoid keeps extends subclasses equivalent to colon subclasses

class Account(owner, balance):
    this.owner = owner
    this.balance = balance
$Account.overdrawn = $Account.balance < 0
class Savings: Account
    def init(owner, balance, rate):
        super(owner, balance)
        this.rate = rate
class Deposit extends Account:
    def init(owner, balance, rate):
        super(owner, balance)
        this.rate = rate
saving = Savings("ada", 100, 2)
deposit = Deposit(rate=2, owner="ada", balance=100)
assert(deposit.owner, saving.owner)
assert(deposit.rate, saving.rate)
assert(deposit.overdrawn, false)
deposit.balance = -5
saving.balance = -5
assert(deposit.overdrawn, true)
assert(saving.overdrawn, true)
assert(Account.length, 0)
assert(Deposit.length, Savings.length)
---
# Nucleoid resolves super and inherited rules through extends chains

class Base:
    def init():
        this.trace = "A"
class Middle extends Base:
    def init():
        super
        this.trace = this.trace + "B"
class Leaf: Middle
    def init():
        super
        this.trace = this.trace + "C"
class Passive extends Leaf
$Base.ready = true
leaf = Leaf()
passive = Passive()
assert(leaf.trace, "ABC")
assert(passive.trace, "ABC")
assert(passive.ready, true)
assert(Base.length, 0)
assert(Middle.length, 0)
assert(Leaf.length, 1)
assert(Passive.length, 1)
---
# Nucleoid keeps extends an ordinary name

extends = 2
limits = {extends: 3}
def scale(extends) => extends * 10
class Gate:
    def extends(step) => step + 1
gate = Gate()
class extends
class Probe extends extends
$extends.ready = true
probe = Probe()
assert(extends + limits.extends, 5)
assert(scale(extends), 20)
assert(gate.extends(4), 5)
assert(probe.ready, true)
assert(Probe.length, 1)
extends = 4
assert(scale(extends), 40)
---
# Nucleoid rolls back extends class declarations

class Base
try:
    class Loop extends Loop
catch error:
    assert(error, TypeError("Circular Inheritance"))
class Child extends Base
try:
    class Base extends Child
catch error:
    assert(error, TypeError("Circular Inheritance"))
assert(Class.length, 2)
class Limited extends Base:
    def init(amount):
        if amount > 10:
            throw "LIMIT"
        this.amount = amount
small = Limited(5)
try:
    large = Limited(20)
catch error:
    assert(error, "LIMIT")
assert(Limited.length, 1)
assert(small.amount, 5)
---
# Nucleoid reads list item properties with selector arguments

pen = {item: "pen", qty: 2, paid: true}
ink = {item: "ink", qty: 5, paid: false}
orders = [pen, ink]
items = orders.map(.item)
units = orders.sum(.qty)
settled = orders.count(.paid)
largest = orders.maxBy(.qty)
assert: [items, units, settled, largest], [["pen", "ink"], 7, 1, ink]
ink.paid = true
pen.qty = 8
orders.push({item: "cap", qty: 1, paid: false})
assert: [items, units, settled, largest], [["pen", "ink", "cap"], 14, 2, pen]
orders.shift()
assert: [items, units, settled, largest], [["ink", "cap"], 6, 1, ink]
---
# Nucleoid reads nested property paths with selector arguments

oslo = {to: {city: "Oslo", zip: "0150"}, tags: ["cold", "fragile"], "class": "air"}
bergen = {to: {city: "Bergen"}, tags: [], "class": "sea"}
shipments = [oslo, bergen]
cities = shipments.map(.to.city)
zips = shipments.map(.to.zip)
counts = shipments.map(.tags.length)
modes = shipments.map(.class)
assert: [cities, zips, counts, modes], [["Oslo", "Bergen"], ["0150", null], [2, 0], ["air", "sea"]]
bergen.to.zip = "5003"
bergen.tags.push("bulk")
shipments.push({tags: [], "class": "rail"})
assert: [cities, zips, counts, modes], [["Oslo", "Bergen", null], ["0150", "5003", null], [2, 1, 0], ["air", "sea", "rail"]]
oslo.to = null
assert: [cities, zips, counts], [null, null, [2, 1, 0]]
oslo.to = {city: "Tromso"}
assert: [cities, zips], [["Tromso", "Bergen", null], [null, "5003", null]]
---
# Nucleoid reads class instance properties with selector arguments

class Bid(bidder, amount, open):
    this.bidder = bidder
    this.amount = amount
    this.open = open
low = Bid.min(.amount)
leader = Bid.maxBy(.amount)
bidding = Bid.filter(.open).map(.bidder)
recent = Bid[-1:].map(.bidder)
assert: [low, leader, bidding, recent], [Number.POSITIVE_INFINITY, null, [], []]
ada = Bid("ada", 120, true)
bo = Bid("bo", 90, false)
assert: [low, leader, bidding, recent], [90, ada, ["ada"], ["bo"]]
bo.amount = 150
bo.open = true
assert: [low, leader, bidding], [120, bo, ["ada", "bo"]]
delete bo.open
delete bo.bidder
delete bo.amount
delete bo
assert: [low, leader, bidding, recent], [120, ada, ["ada"], ["ada"]]
---
# Nucleoid keeps selector arguments live in class level rules

class Order(lines): this.lines = lines
$Order.units = $Order.lines.sum(.qty)
$Order.skus = $Order.lines.map(.sku)
$Order.bulky = $Order.lines.count(.bulk)
small = Order([{sku: "pen", qty: 2, bulk: false}])
large = Order([{sku: "ink", qty: 50, bulk: true}, {sku: "cap", qty: 1, bulk: false}])
busiest = Order.maxBy(.units)
assert: [small.units, small.skus, small.bulky, large.units, large.bulky], [2, ["pen"], 0, 51, 1]
assert: busiest, large
small.lines.push({sku: "pad", qty: 80, bulk: true})
assert: [small.units, small.skus, small.bulky, busiest], [82, ["pen", "pad"], 1, small]
large.lines = [{sku: "box", qty: 99, bulk: true}]
assert: [large.units, large.skus, large.bulky, busiest], [99, ["box"], 1, large]
---
# Nucleoid passes selector arguments as ordinary functions

def spread(list, key) => list.max(key) - list.min(key)
def column(list, key) => list.map(key).join("/")
boxes = [{w: 2, h: 9, tag: "a"}, {w: 5, h: 4, tag: "b"}]
wide = spread(boxes, .w)
tall = spread(boxes, .h)
widths = column(boxes, .w)
tags = column(
    boxes,
    .tag
)
chain = {at: 1, next: {at: 2, next: {at: 3, next: null}}}
steps = ["x", "y"]
hop = steps.reduce(.next, chain)
assert: [wide, tall, widths, tags, hop.at], [3, 5, "2/5", "a/b", 3]
boxes.push({w: 11, h: 1, tag: "c"})
steps.pop()
assert: [wide, tall, widths, tags, hop.at], [9, 8, "2/5/11", "a/b/c", 2]
---
# Nucleoid uses selector arguments in headers methods lambdas and templates

class Part(sku, kg, fragile, cost):
    this.sku = sku
    this.kg = kg
    this.fragile = fragile
    this.cost = cost
class Shelf(parts):
    this.parts = parts
    def weight(): return this.parts.sum(.kg)
pen = Part("pen", 2, false, 4)
ink = Part("ink", 5, true, 9)
left = Shelf([pen, ink])
care = left.parts.some(.fragile) ? "careful" : "plain"
size = "big" if left.parts.max(.kg) > 4 else "small"
note = `${left.parts.map(.sku).join("+")}`
price = shelf -> shelf.parts.sum(.cost)
wrapped = []
for part of Part.filter(.fragile):
    wrapped.push(part.sku)
pending = [{open: true}, {open: true}, {open: false}]
while pending.count(.open) > 1:
    pending.shift()
assert: [left.weight(), care, size, note, price(left)], [7, "careful", "big", "pen+ink", 13]
assert: [wrapped, pending.length], [["ink"], 2]
ink.fragile = false
ink.kg = 3
assert: [left.weight(), care, size], [5, "plain", "small"]
left.parts.push(Part("cap", 2, true, 1))
assert: [left.weight(), care, note, price(left)], [7, "careful", "pen+ink+cap", 14]
---
# Nucleoid rolls back failed selector argument updates

def checked(total):
    if total > 100:
        throw "TOO_MANY"
    return total
cart = [{sku: "pen", qty: 2, price: 3}, {sku: "ink", qty: 5, price: 9}]
units = cart.sum(.qty)
skus = cart.map(.sku)
dearest = cart.maxBy(.price)
cheapest = cart.minBy(.price)
cap = checked(units)
failure = ""
try:
    cart.push({sku: "box", qty: 500, price: 1})
catch error:
    failure = error
assert: [failure, units, skus, cap, cart.length], ["TOO_MANY", 7, ["pen", "ink"], 7, 2]
suppose:
    cart.push({sku: "cap", qty: 30, price: 12})
    assert: [units, skus, dearest.sku, cap], [37, ["pen", "ink", "cap"], "cap", 37]
assert: [units, skus, dearest.sku, cheapest.sku, cart.length], [7, ["pen", "ink"], "ink", "pen", 2]
cart.push({sku: "pad", qty: 9, price: 1})
assert: [units, skus, dearest.sku, cheapest.sku, cap], [16, ["pen", "ink", "pad"], "ink", "pad", 16]
---
# Nucleoid tracks only the properties that selector arguments read

class Sensor(room, temp):
    this.room = room
    this.temp = temp
hottest = Sensor.maxBy(.temp)
rooms = Sensor.map(.room)
lab = Sensor("lab", 21)
hall = Sensor("hall", 25)
readers = (affects hall.temp).map(.node)
listers = (affects hall.room).map(.node)
assert: [readers, listers, hottest, rooms], [["hottest"], ["rooms"], hall, ["lab", "hall"]]
coolest = Sensor.minBy(.temp)
assert: [readers, listers, coolest], [["hottest", "coolest"], ["rooms"], lab]
lab.temp = 30
hall.room = "foyer"
assert: [hottest, coolest, rooms], [lab, hall, ["lab", "foyer"]]
---
# Nucleoid averages lists with average

temps = [18, 24, 9]
mean = temps.average()
gap = temps.average(temp -> Math.abs(temp - 20))
assert: [mean, gap, (why mean)[0].from], [17, 5.666666666666667, ["temps"]]
temps.push(29)
assert: [mean, gap], [20, 6.5]
temps.pop()
temps.pop()
assert: [mean, gap], [21, 3]
temps = []
assert: [Number.isNaN(mean), Number.isNaN(gap)], [true, true]
cart = {items: [0.1, 0.2]}
paid = cart.items.average()
small = cart.items.filter(item -> item < 0.25).average()
cart.items.push(0.3)
assert: [paid, small], [0.20000000000000004, 0.15000000000000002]
---
# Nucleoid averages class instance populations with average

class Trip(km, hours):
    this.km = km
    this.hours = hours
def speed trip => trip.km / trip.hours
first = Trip(120, 2)
second = Trip(45, 1)
pace = Trip.average(speed)
far = Trip.filter(trip -> trip.km > 100).average(speed)
recent = Trip[-2:].average(trip -> trip.hours * 60)
assert: [pace, far, recent], [52.5, 60, 90]
third = Trip(300, 4)
assert: [pace, far, recent], [60, 67.5, 150]
first.hours = 3
assert: [pace, far], [53.333333333333336, 57.5]
delete third.hours
assert: [pace, far, recent], [null, null, null]
delete third.km
delete third
assert: [pace, far, recent], [42.5, 40, 120]
class Refund(cents): this.cents = cents
refunds = Refund.average(refund -> refund.cents / 100)
assert: Number.isNaN(refunds), true
Refund(250)
assert: refunds, 2.5
---
# Nucleoid converts averaged values to numbers

flags = [true, false, true, true]
share = flags.average()
entries = [2, "3", " 4 ", "", null, [5], []]
mean = entries.average()
assert: [share, mean], [0.75, 2]
flags.push(false)
entries.push("x")
assert: [share, Number.isNaN(mean)], [0.6, true]
entries.pop()
entries.push(Number.POSITIVE_INFINITY)
assert: mean, Number.POSITIVE_INFINITY
class Line(price, qty):
    this.price = price
    this.qty = qty
pen = Line(3, 2)
book = Line(12, 1)
typical = Line.average(line -> line.price * line.qty)
assert: typical, 9
gift = Line(null, 1)
assert: typical, null
gift.price = 6
assert: typical, 8
---
# Nucleoid calls the average selector once per item with only the item

seen = []
def weigh(item, position):
    seen.push(position ?? item)
    return item * item
items = [3, 1, 2]
def spread() => items.average(weigh)
assert: [spread(), seen], [4.666666666666667, [3, 1, 2]]
floor = 2
def price item => Math.max(item, floor)
cost = items.average(price)
assert: cost, 2.3333333333333335
floor = 3
items.push(5)
assert: [cost, seen], [3.5, [3, 1, 2]]
missing = null
skipped = missing.average(weigh(9))
assert: [skipped, seen], [null, [3, 1, 2]]
---
# Nucleoid keeps averages live in class level rules

class Room(name): this.name = name
class Sensor(room, temp):
    this.room = room
    this.temp = temp
$Room.mean = Sensor.filter(sensor -> sensor.room == $Room).average(sensor -> sensor.temp)
$Room.warm = $Room.mean > 21
lab = Room("lab")
hall = Room("hall")
Sensor(lab, 20)
probe = Sensor(lab, 25)
Sensor(hall, 18)
comfort = Room.average(room -> Math.abs(room.mean - 21))
assert: [lab.mean, lab.warm, hall.mean, hall.warm, comfort], [22.5, true, 18, false, 2.25]
probe.temp = 21
assert: [lab.mean, lab.warm, comfort], [20.5, false, 1.75]
probe.room = hall
assert: [lab.mean, hall.mean, comfort], [20, 19.5, 1.25]
attic = Room("attic")
assert: [Number.isNaN(attic.mean), attic.warm, Number.isNaN(comfort)], [true, false, true]
---
# Nucleoid uses averages in functions methods and conditional expressions

class Team(name, scores):
    this.name = name
    this.scores = scores
    def mean(): return this.scores.average()
def variance(team) => team.scores.average(score -> score * score) - team.mean() ** 2
red = Team("red", [3, 5])
blue = Team("blue", [6])
league = Team.average(team -> team.mean())
leader = red.mean() > blue.mean() ? red.name : blue.name
spread = variance(red)
assert: [league, leader, spread], [5, "blue", 1]
blue.scores.push(2)
red.scores.push(7)
Team("green", [1])
assert: [league, leader, spread], [3.3333333333333335, "red", 2.666666666666668]
---
# Nucleoid keeps average as an ordinary name beside the average query

average = 2
report = {average: 5}
def double average => average * 2
class Bag(items):
    this.items = items
    def average(): return this.items.average(item -> item * item)
bag = Bag([1, 3])
level = bag.items.average() + average + report.average
bags = Bag.average(each -> each.average())
assert: [level, bag.average(), bags, double(average)], [9, 5, 5, 4]
assert: bag.items.average, null
bag.items.push(5)
average = 4
Bag([6])
assert: [level, bags], [12, 23.833333333333332]
---
# Nucleoid rolls back failed average updates

def checked(value):
    if value > 100:
        throw "TOO_LARGE"
    return value
amounts = [10, 20]
mean = amounts.average(checked)
failure = ""
try:
    amounts.push(500)
catch error:
    failure = error
assert: [failure, amounts, mean], ["TOO_LARGE", [10, 20], 15]
amounts.push(30)
assert: mean, 20
class Item(price): this.price = price
pen = Item(3)
typical = Item.average(item -> checked(item.price))
failure = ""
try:
    Item(500)
catch error:
    failure = error
assert: [failure, Item.length, typical], ["TOO_LARGE", 1, 3]
suppose:
    Item(5)
    assert: typical, 4
assert: [Item.length, typical], [1, 3]
---
# Nucleoid copies object entries into records with spread entries

base = {name: "tea", price: 4, stock: 10}
sale = {...base, price: 3, tag: "sale"}
floor = {price: 9, ...base}
assert: [sale, floor], [{name: "tea", price: 3, stock: 10, tag: "sale"}, {price: 4, name: "tea", stock: 10}]
assert: [sale == base, floor == base, base.tag], [false, false, null]
base.name = "oolong"
base.price = 6
base.size = 2
delete base.stock
assert: [sale, floor], [{name: "oolong", price: 3, size: 2, tag: "sale"}, {price: 6, name: "oolong", size: 2}]
tag = "new"
fresh = {tag, ...base price: 5,}
assert: fresh, {tag: "new", name: "oolong", price: 5, size: 2}
---
# Nucleoid keeps spread entries live as their sources change

base = {name: "tea", price: 4}
item = {...base, qty: 2}
line = {...item, total: item.price * item.qty}
assert: line, {name: "tea", price: 4, qty: 2, total: 8}
base.price = 5
base.size = "large"
delete base.name
assert: line, {price: 5, size: "large", qty: 2, total: 10}
base = {name: "jam", price: 3}
assert: [item, line], [{name: "jam", price: 3, qty: 2}, {name: "jam", price: 3, qty: 2, total: 6}]
---
# Nucleoid merges records with several spread entries

defaults = {color: "red", size: 2, gift: false}
custom = {size: 3}
order = {...defaults, ...custom, id: 7}
assert: order, {color: "red", size: 3, gift: false, id: 7}
custom.gift = true
custom.note = "rush"
assert: order, {color: "red", size: 3, gift: true, note: "rush", id: 7}
delete custom.size
defaults.color = "blue"
assert: order, {color: "blue", size: 2, gift: true, note: "rush", id: 7}
---
# Nucleoid spreads instances into records and class level rules

class Room(name, temp):
    this.name = name
    this.temp = temp
$Room.card = {...$Room, unit: "C"}
lab = Room("lab", 20)
lab.temp = 25
lab.floor = 2
assert: lab.card, {id: "lab", name: "lab", temp: 25, floor: 2, unit: "C"}
hall = Room("hall", 18)
copy = {...hall, temp: 19}
assert: (why copy)[0].from, ["hall", "hall.id", "hall.name", "hall.card"]
hall.name = "foyer"
assert: copy, {id: "hall", name: "foyer", temp: 19, card: {id: "hall", name: "foyer", temp: 18, unit: "C"}}
---
# Nucleoid spreads entries in functions methods and lambdas

class Point(x, y):
    this.x = x
    this.y = y
    def shifted dx => {...this, x: this.x + dx}
def flat p => {...p, z: 0}
p = Point(1, 2)
q = Point(3, 4)
up = flat(p)
far = p.shifted(10)
grid = [p, q].map(lambda pt: {...pt, area: pt.x * pt.y})
assert: [up, far], [{id: "p", x: 1, y: 2, z: 0}, {id: "p", x: 11, y: 2}]
p.y = 5
assert: [up, far, grid], [{id: "p", x: 1, y: 5, z: 0}, {id: "p", x: 11, y: 5}, [{id: "p", x: 1, y: 5, area: 5}, {id: "q", x: 3, y: 4, area: 12}]]
---
# Nucleoid spreads only objects and evaluates each source once

owner = null
card = {...owner, seen: true}
assert: card, null
owner = {name: "ada", role: "dev"}
assert: card, {name: "ada", role: "dev", seen: true}
box = {}
packed = {...box.lid, open: false}
assert: packed, {open: false}
box.lid = {color: "tan", size: 3}
assert: packed, {color: "tan", size: 3, open: false}
class Tag(label, uses):
    this.label = label
    this.uses = uses
made = {...Tag("new", 0), qty: 1}
assert: [made.label, made.uses, made.qty, Tag.length], ["new", 0, 1, 1]
sizes = [1, 2]
try:
    broken = {...sizes}
catch error:
    assert: error, TypeError("sizes is not an object")
---
# Nucleoid leaves the declared property out of spreads of its own object

class Order(total, items):
    this.total = total
    this.items = items
$Order.view = {...$Order, paid: false}
order = Order(10, 2)
order.total = 12
order.total = 14
assert: order.view, {id: "order", total: 14, items: 2, paid: false}
cart = {tea: 4, jam: 2}
cart.copy = {...cart, n: 1}
cart.tea = 5
cart.oat = 1
assert: cart.copy, {tea: 5, jam: 2, oat: 1, n: 1}
---
# Nucleoid rolls back failed spread updates

def checked(count):
    if count > 9:
        throw "TOO_MANY"
    return count
cart = {tea: 4, jam: 3}
order = {...cart, count: checked(cart.tea)}
failure = ""
try:
    cart.tea = 50
catch error:
    failure = error
assert: [failure, cart.tea, order], ["TOO_MANY", 4, {tea: 4, jam: 3, count: 4}]
try:
    cart.oats = 2
    delete cart.jam
    assert: order, {tea: 4, oats: 2, count: 4}
    throw "DECLINED"
catch error:
    pass
assert: order, {tea: 4, jam: 3, count: 4}
plan = suppose:
    cart.rye = 5
    order.rye
assert: [plan, order.rye], [5, null]
cart.rye = 1
assert: order, {tea: 4, jam: 3, rye: 1, count: 4}
---
# Nucleoid narrows explanations to a reasoning depth

base = 1
seed = base + 1
step = seed + 1
next = step * 2
result = next + step
own = result |> why |> depth(0)
near = result |> why |> depth(1)
wide = result |> why |> depth(2)
assert(own.map(s -> s.node), ["result"])
assert(near.map(s -> s.node), ["result", "next", "step"])
assert(near[0].from, ["next", "step"])
assert(wide.map(s -> s.node), ["result", "next", "step", "seed"])
base = 5
step = base * 3
assert(near.map(s -> s.holds), [45, 30, 15])
assert(wide.map(s -> s.node), ["result", "next", "step", "base"])
result = 7
assert(own.map(s -> s.state), ["stated"])
---
# Nucleoid measures reasoning depth along the shortest path

rate = 2
fee = rate + 1
charge = fee * 2
base = rate * 3
total = base + charge
two = total |> why |> depth(2)
assert(two.map(s -> s.node), ["total", "base", "charge", "fee", "rate"])
base = 10
assert(two.map(s -> s.node), ["total", "base", "charge", "fee"])
assert((why total).length, 5)
rate = 4
assert(two.map(s -> s.holds), [20, 10, 10, 5])
---
# Nucleoid narrows impact selections to direct dependents

price = 10
tax = price * 2
total = price + tax
label = total * 3
direct = price |> affects |> depth(1)
again = tax |> why |> affects |> depth(1)
assert((affects price).map(s -> s.node), ["tax", "total", "label"])
assert(direct.map(s -> s.node), ["tax", "total"])
assert(again.map(s -> s.node), ["tax", "total"])
label = price + 1
assert(direct.map(s -> s.node), ["tax", "total", "label"])
assert(again.map(s -> s.node), ["tax", "total", "label"])
price = 4
assert(direct.map(s -> s.holds), [8, 12, 5])
---
# Nucleoid keeps depth selections live as limits and rules change

base = 1
mid = base + 1
top = mid * 2
limit = 1
near = top |> why |> depth(limit)
assert(near.map(s -> s.node), ["top", "mid"])
limit = 2
assert(near.map(s -> s.node), ["top", "mid", "base"])
extra = 5
mid = extra + base
assert(near.map(s -> s.node), ["top", "mid", "extra", "base"])
assert(near[0].holds, 12)
limit = 0
assert(near.map(s -> s.node), ["top"])
top = 3
limit = 1
assert(near.map(s -> s.state), ["stated"])
---
# Nucleoid composes depth with state filters and later walks

base = 2
rate = base * 2
fee = 1
cost = rate * 4
price = cost + fee
facts = price |> why |> depth(2) |> stated
touched = price |> why |> stated |> affects |> depth(1)
alone = price |> depth(2)
assert(facts.map(s -> s.node), ["fee"])
assert(touched.map(s -> s.node), ["rate", "price"])
assert(alone.map(s -> s.node), ["price"])
fee = base + 1
assert(facts.map(s -> s.node), ["base"])
assert(touched.map(s -> s.node), ["rate", "fee"])
price = 9
assert(alone.map(s -> s.state), ["stated"])
---
# Nucleoid waits for unknown depth limits

cause = 4
effect = cause * 2
class Plan
plan = Plan()
known = null
pending = effect |> why |> depth(plan.depth)
waiting = effect |> why |> depth(known)
ready = all s in waiting: s.holds > 0
assert(pending, null)
assert(waiting, null)
assert(ready, null)
plan.depth = 0
known = 1
assert(pending.map(s -> s.node), ["effect"])
assert(waiting.map(s -> s.node), ["effect", "cause"])
assert(ready, true)
cause = -1
assert(ready, false)
---
# Nucleoid applies depth in class level rules methods and quantifiers

class Order(qty, price):
    this.qty = qty
    this.price = price
    def direct() => this.total |> why |> depth(1)
$Order.units = $Order.qty * 10
$Order.total = $Order.units * $Order.price
$Order.causes = $Order.total |> why |> depth(1)
pen = Order(2, 3)
ok = all s in pen.total |> why |> depth(1): s.holds > 5
assert(pen.causes.map(s -> s.node), ["pen.total", "pen.units", "pen.price"])
assert(pen.direct().map(s -> s.holds), [60, 20, 3])
assert(ok, false)
pen.price = 7
assert(pen.causes[0].holds, 140)
assert(ok, true)
cup = Order(1, 9)
assert(cup.causes.map(s -> s.holds), [90, 10, 9])
---
# Nucleoid restores depth selections after failed changes

seed = 1
mid = seed + 1
top = mid * 2
limit = 1
near = top |> why |> depth(limit)
frozen = near.value
try:
    limit = 2
    mid = 5
    throw "UNDO"
catch error:
    assert(error, "UNDO")
assert(near.map(s -> s.node), ["top", "mid"])
plan = suppose:
    limit = 2
    near.length
assert(plan, 3)
assert(near.length, 2)
limit = 2
seed = 4
assert(near.map(s -> s.holds), [10, 5, 4])
assert(frozen.map(s -> s.holds), [4, 2])
---
# Nucleoid lists what a supposed premise changes

threshold = 90
reading = 40
alarm = reading > threshold
level = reading > 10
suppose reading = 95:
    moved = model |> diff
    assert(moved.map(s -> s.node), ["reading", "alarm"])
    assert(moved.map(s -> s.holds), [95, true])
    assert(moved[1].from, ["reading", "threshold"])
    threshold = 99
    assert(moved.map(s -> s.node), ["threshold", "reading"])
assert(alarm, false)
---
# Nucleoid lists what changed in an explanation or an impact

base = 10
rate = 2
fee = 3
cost = base * rate
total = cost + fee
label = total > 40
charge = fee * 2
suppose rate = 3:
    causes = total |> why |> diff
    effects = base |> affects |> diff
    reach = model |> diff |> affects
    assert(causes.map(s -> s.node), ["total", "cost", "rate"])
    assert(effects.map(s -> s.node), ["cost", "total"])
    assert(reach.map(s -> s.node).sort(), ["cost", "label", "total"])
    fee = 30
    assert(causes.map(s -> s.node), ["total", "cost", "fee", "rate"])
    assert(effects.map(s -> s.node), ["cost", "total", "label"])
    assert(reach.map(s -> s.node).sort(), ["charge", "cost", "label", "total"])
---
# Nucleoid compares instances and class level rules in a hypothetical

class Item
$Item.premium = $Item.price > 10
pen = Item()
pen.price = 5
book = Item()
book.price = 20
suppose pen.price = 20:
    moved = model |> diff
    assert(moved.map(s -> s.node), ["pen.premium", "pen.price"])
    cup = Item()
    cup.price = 3
    assert(moved.map(s -> s.node), ["pen.premium", "pen.price", "cup", "cup.premium", "cup.price"])
    $Item.premium = $Item.price > 25
    assert(book.premium, false)
    assert(moved.map(s -> s.node), ["pen.price", "book.premium", "cup", "cup.premium", "cup.price"])
assert(book.premium, true)
---
# Nucleoid compares lists and objects in a hypothetical

tags = ["pen"]
count = tags.length
limits = {max: 3}
full = count >= limits.max
suppose tags.push("ink"):
    moved = model |> diff
    assert(moved.map(s -> s.node), ["tags", "count"])
    assert(moved[0].holds, ["pen", "ink"])
    tags.pop()
    assert(moved, [])
    limits = {max: 3}
    assert(moved.map(s -> s.node), ["limits"])
assert(tags, ["pen"])
---
# Nucleoid leaves equal values and undone writes out of a diff

a = 1
b = a + 1
c = b * 2
suppose a = 1:
    moved = model |> diff
    assert(moved, [])
    b = 1 + a
    delete c
    c = b * 2
    assert(moved, [])
    try:
        a = 5
        throw "UNDO"
    catch error:
        assert(error, "UNDO")
    assert(moved, [])
    a = 3
    assert(moved.map(s -> s.node), ["a", "b", "c"])
    assert(moved[1].rule, "b = 1+a")
---
# Nucleoid compares nested hypotheticals with the innermost block

price = 10
qty = 2
total = price * qty
suppose price = 12:
    outer = model |> diff
    assert(outer.map(s -> s.node), ["price", "total"])
    inner = suppose qty = 3:
        nested = model |> diff
        assert(nested.map(s -> s.node), ["qty", "total"])
        price = 10
        assert(nested.map(s -> s.node), ["price", "qty", "total"])
        nested.map(s -> s.holds)
    assert(inner, [10, 3, 30])
    assert(outer.map(s -> s.node), ["price", "total", "inner"])
assert(total, 20)
---
# Nucleoid finds hypothetical changes in functions methods and quantifiers

class Tank(level):
    this.level = level
    def changed() => (model |> diff).map(s -> s.node)
low = 10
tank = Tank(5)
empty = tank.level < low
def touches(name) => any s in model |> diff: s.node == name
suppose tank.level = 20:
    stable = all s in model |> diff: s.node != "low"
    assert(tank.changed(), ["tank.level", "empty"])
    assert(touches("empty"), true)
    assert(stable, true)
    low = 30
    assert(stable, false)
    assert(touches("empty"), false)
    assert(tank.changed(), ["low", "tank.level"])
---
# Nucleoid composes hypothetical differences with state filters

price = 10
discount = 0
net = price - discount
taxed = net * 2
suppose discount = 3:
    facts = model |> diff |> stated
    effects = model |> derived |> diff
    assert(facts.map(s -> s.node), ["discount"])
    assert(effects.map(s -> s.node), ["net", "taxed"])
    assert(effects[0].from, ["price", "discount"])
    price = 13
    assert(facts.map(s -> s.node), ["price", "discount"])
    assert(effects, [])
---
# Nucleoid finds the middle values of lists with median

waits = [12, 3, 45]
typical = waits.median()
assert: [typical, waits, (why typical)[0].from], [12, [12, 3, 45], ["waits"]]
waits.push(7)
assert: typical, 9.5
waits.push(300)
assert: [typical, waits.average()], [12, 73.4]
waits.pop()
waits.shift()
assert: typical, 7
courier = {stops: [12, 2, 45, 31]}
usual = courier.stops.median()
short = courier.stops.filter(stop -> stop < 30).median()
assert: [usual, short], [21.5, 7]
courier.stops.push(5)
assert: [usual, short], [12, 5]
---
# Nucleoid finds the medians of class instance populations

class Listing(area, price):
    this.area = area
    this.price = price
def perMeter listing => listing.price / listing.area
Listing(50, 200000)
loft = Listing(40, 400000)
typical = Listing.median(perMeter)
roomy = Listing.filter(listing -> listing.area > 45).median(perMeter)
recent = Listing[-3:].median(listing -> listing.area)
assert: [typical, roomy, recent], [7000, 4000, 45]
Listing(80, 72000)
assert: [typical, roomy, recent], [4000, 2450, 50]
Listing(100, 450000)
assert: [typical, roomy, recent], [4250, 4000, 80]
loft.price = 120000
assert: [typical, roomy], [3500, 4000]
---
# Nucleoid orders values as numbers to find medians

north = [21, 9]
south = [-3, 7.5]
middle = north.concat(south).median()
warm = north.filter(temp -> temp > 0).median()
assert: [middle, warm], [8.25, 15]
north.push(-8)
assert: [middle, warm, north], [7.5, 15, [21, 9, -8]]
north.push(4)
assert: [middle, warm], [5.75, 9]
---
# Nucleoid calls the median selector once per item with only the item

seen = []
def weigh(item, position):
    seen.push(position ?? item)
    return item * item
items = [4, 1]
def spread() => items.median(weigh)
assert: [spread(), seen], [8.5, [4, 1]]
floor = 2
def price item => Math.max(item, floor)
cost = items.median(price)
assert: cost, 3
floor = 3
items.push(12)
assert: [cost, spread(), seen], [4, 16, [4, 1, 4, 1, 12]]
missing = null
skipped = missing.median(weigh(9))
assert: [skipped, seen], [null, [4, 1, 4, 1, 12]]
---
# Nucleoid keeps medians live in class level rules

class Room(name): this.name = name
class Sensor(room, temp):
    this.room = room
    this.temp = temp
$Room.mid = Sensor.filter(sensor -> sensor.room == $Room).median(sensor -> sensor.temp)
$Room.warm = $Room.mid > 21
lab = Room("lab")
hall = Room("hall")
Sensor(lab, 8)
probe = Sensor(lab, 25)
Sensor(lab, 30)
Sensor(hall, 18)
Sensor(hall, 23)
spread = Room.median(room -> room.mid)
assert: [lab.mid, lab.warm, hall.mid, hall.warm, spread], [25, true, 20.5, false, 22.75]
probe.temp = 21
assert: [lab.mid, lab.warm, spread], [21, false, 20.75]
probe.room = hall
porch = Room("porch")
Sensor(porch, 5)
assert: [lab.mid, hall.mid, porch.mid, spread], [19, 21, 5, 19]
---
# Nucleoid uses medians in functions methods and conditional expressions

class Team(name, scores):
    this.name = name
    this.scores = scores
    def middle(): return this.scores.median()
def lift(team) => team.scores.max() - team.scores.median()
red = Team("red", [3, 9])
blue = Team("blue", [8, 5, 7])
league = Team.median(team -> team.middle())
leader = red.middle() > blue.middle() ? red.name : blue.name
assert: [league, leader, lift(red)], [6.5, "blue", 3]
red.scores.push(10)
blue.scores.push(2)
Team("green", [13])
assert: [league, leader, lift(red)], [9, "red", 1]
---
# Nucleoid keeps median as an ordinary name beside the median query

median = 2
report = {median: 5}
def double median => median * 2
class Bag(items):
    this.items = items
    def median(): return this.items.median(item -> item * item)
bag = Bag([1, 3])
Bag([4])
level = bag.items.median() + median + report.median
bags = Bag.median(each -> each.median())
assert: [level, bag.median(), bags, double(median)], [9, 5, 10.5, 4]
assert: bag.items.median, null
bag.items.push(10)
median = 4
Bag([6])
assert: [level, bag.median(), bags], [12, 9, 16]
---
# Nucleoid rolls back failed median updates

def checked(value):
    if value > 100:
        throw "TOO_LARGE"
    return value
amounts = [10, 40]
mid = amounts.median(checked)
failure = ""
try:
    amounts.push(500)
catch error:
    failure = error
assert: [failure, amounts, mid], ["TOO_LARGE", [10, 40], 25]
amounts.push(5)
assert: mid, 10
class Item(price): this.price = price
Item(3)
Item(8)
typical = Item.median(item -> checked(item.price))
failure = ""
try:
    Item(500)
catch error:
    failure = error
assert: [failure, Item.length, typical], ["TOO_LARGE", 2, 5.5]
suppose:
    Item(12)
    assert: typical, 8
assert: [Item.length, typical], [2, 5.5]
---
# Nucleoid swaps two class properties through fixed cross anchors

class Box(width, height):
    this.width = width
    this.height = height
early = Box(4, 1)
$Box.width = $Box.height@first
$Box.height = $Box.width@first
box = Box(3, 2)
assert([box.width, box.height], [2, 3])
assert([early.width, early.height], [1, 4])
assert([box.width@first, box.height@first], [3, 2])
assert([box.width@1, box.height@1, box.width@2], [2, 3, null])
assert([early.width@1, early.height@1, early.width@2], [1, 4, null])
[box.width, box.height, early.width, early.height]
# return: [2,3,1,4]
---
# Nucleoid exchanges coordinates and turns rectangles through fixed cross anchors

class Point(x, y):
    this.x = x
    this.y = y
$Point.x = $Point.y@first
$Point.y = $Point.x@first
p = Point(1, 5)
q = Point(-2, 4)
assert([p.x, p.y, q.x, q.y], [5, 1, 4, -2])
size = 10
class Rect(x, y, width, height):
    this.x = x
    this.y = y
    this.width = width
    this.height = height
$Rect.x = $Rect.y@first
$Rect.y = size - $Rect.x@first - $Rect.width@first
$Rect.width = $Rect.height@first
$Rect.height = $Rect.width@first
r = Rect(1, 2, 4, 3)
assert([r.x, r.y, r.width, r.height], [2, 5, 3, 4])
size = 20
assert([r.x, r.y, r.width, r.height], [2, 15, 3, 4])
assert([r.x@first, r.y@first, r.width@first, r.height@first], [1, 2, 4, 3])
[p.x, p.y, r.x, r.y]
# return: [5,1,2,15]
---
# Nucleoid rotates class properties with numeric first and indexed anchors

class Triple(a, b, c):
    this.a = a
    this.b = b
    this.c = c
$Triple.a = $Triple.b@0
$Triple.b = $Triple["c"]@first
$Triple.c = $Triple.a@0
t = Triple(1, 2, 3)
assert([t.a, t.b, t.c], [2, 3, 1])
class Pair(a, b):
    this.a = a
    this.b = b
$Pair.a = $Pair.a@first + $Pair.b@first
$Pair.b = $Pair.a@first
pair = Pair(1, 2)
assert([pair.a, pair.b], [3, 1])
class Base(a, b):
    this.a = a
    this.b = b
class Child: Base
    def init(a, b):
        super(a, b)
$Base.a = $Base.b@first
$Child.b = $Child.a@first
child = Child(1, 2)
parent = Base(5, 6)
assert([child.a, child.b], [2, 1])
assert([parent.a, parent.b], [6, 6])
[t.a, t.b, t.c]
# return: [2,3,1]
---
# Nucleoid keeps fixed cross anchors tracked through slot deletion and recreation

class Box(width, height):
    this.width = width
    this.height = height
$Box.width = $Box.height@first
$Box.height = $Box.width@first
class Frozen(width, height):
    this.width = width
    this.height = height
$Frozen.width = $Frozen.height@first.value
$Frozen.height = $Frozen.width@first.value
box = Box(3, 2)
frozen = Frozen(3, 2)
assert([box.width, box.height, frozen.width, frozen.height], [2, 3, 2, 3])
delete box.height
delete frozen.height
assert(box.width, null)
assert(frozen.width, 2)
box.height = 7
frozen.height = 7
assert([box.width, box.height], [7, 7])
assert([frozen.width, frozen.height], [2, 7])
assert([box.width@first, box.width@1, box.width@2, box.width@3], [3, 2, null, 7])
assert(box.height@first, 7)
[box.width, frozen.width]
# return: [7,2]
---
# Nucleoid settles fixed cross anchor cycles beside live inputs and readers

k = 1
class Pair(a, b):
    this.a = a
    this.b = b
$Pair.a = $Pair.b@first + k
$Pair.b = $Pair.a@first + k
$Pair.sum = $Pair.a + $Pair.b
pair = Pair(1, 2)
assert([pair.a, pair.b, pair.sum], [3, 2, 5])
k = 10
assert([pair.a, pair.b, pair.sum], [12, 11, 23])
assert([pair.a@first, pair.b@first, pair.a@3], [1, 2, null])
class Late(a, b):
    this.a = a
    this.b = b
$Late.a = $Late.b@1 ?? -1
$Late.b = $Late.a@first
late = Late(1, 2)
assert([late.a, late.b], [1, 1])
assert([late.a@1, late.a@2, late.b@1], [-1, 1, 1])
class Missing(width):
    this.width = width
$Missing.width = $Missing.height@first
$Missing.height = $Missing.width@first
missing = Missing(3)
assert([missing.width, missing.height], [3, 3])
assert([missing.width@1, missing.width@2], [null, 3])
[pair.a, pair.b, pair.sum]
# return: [12,11,23]
---
# Nucleoid keeps live and moving reads in class cycles as circular dependencies

class Sample(x, y):
    this.x = x
    this.y = y
def latest(sample) => sample.y@last
item = Sample(1, 2)
try:
    $Sample.x = $Sample.y@first
    $Sample.y = $Sample.x + 1
catch error:
    assert(error, TypeError("Circular Dependency"))
try:
    $Sample.x = $Sample.x@first + $Sample.y
    $Sample.y = $Sample.x
catch error:
    assert(error, TypeError("Circular Dependency"))
try:
    $Sample.x = $Sample.y@first
    $Sample.y = $Sample.x@last
catch error:
    assert(error, TypeError("Circular Dependency"))
try:
    $Sample.x = $Sample.y@previous
    $Sample.y = $Sample.x@first
catch error:
    assert(error, TypeError("Circular Dependency"))
try:
    $Sample.x = $Sample.y@first + $Sample.y
    $Sample.y = $Sample.x@first
catch error:
    assert(error, TypeError("Circular Dependency"))
try:
    $Sample.x = $Sample.y@first + latest($Sample) * 0
    $Sample.y = $Sample.x@first
catch error:
    assert(error, TypeError("Circular Dependency"))
try:
    $Sample.x = $Sample.y@first
    $Sample.y = $Sample.x@first
    $Sample.y += 1
catch error:
    assert(error, TypeError("Circular Dependency"))
assert([item.x, item.y], [1, 2])
assert([item.x@1, item.y@1], [null, null])
$Sample.x = $Sample.y@first
$Sample.y = $Sample.x@first
assert([item.x, item.y], [2, 1])
[item.x, item.y]
# return: [2,1]
---
# Nucleoid matches values with is and is not

value = "EXACT"
if value is "EXACT":
    double = "match"
if value is 'EXACT':
    single = "match"
if value is EXACT:
    bare = "match"
if value is not OTHER:
    other = "differs"
assert(double, "match")
assert(single, "match")
assert(bare, "match")
assert(other, "differs")
count = 3
ready = true
order = { status: "SHIPPED", total: 30 }
checks = [count is 3, count is "3", count is not 4, ready is true, ready is not false, order.status is SHIPPED, order.total is 30, order.status is not PENDING]
assert(checks, [true, true, true, true, true, true, true, true])
order.status = "PENDING"
assert(checks, [true, true, true, true, true, false, true, false])
status = "PENDING"
if status is SHIPPED:
    stage = "done"
else:
    stage = "open"
assert(stage, "open")
status = "SHIPPED"
assert(stage, "done")
stage
# return: "done"
---
# Nucleoid reads a bare word after is as its variable or else as its text

value = "EXACT"
flag = (value is EXACT)
miss = (value is not EXACT)
assert(flag, true)
assert(miss, false)
EXACT = "OTHER"
assert(flag, false)
assert(miss, true)
value = "OTHER"
assert(flag, true)
delete EXACT
assert(flag, false)
assert(miss, true)
value = "EXACT"
assert(flag, true)
if value is not EXACT:
    note = "changed"
else:
    note = "same"
assert(note, "same")
EXACT = "NEW"
assert(note, "changed")
delete EXACT
assert(note, "same")
kind = "Math"
named = [(kind is Math), (kind is "Math"), (kind is MATH)]
assert(named, [false, true, false])
def pick(EXACT, candidate) => (candidate is EXACT)
assert(pick("A", "A"), true)
assert(pick("A", "EXACT"), false)
flag
# return: true
---
# Nucleoid applies is comparisons in class level rules

class Order(status, total):
    this.status = status
    this.total = total
$Order.shipped = ($Order.status is SHIPPED)
$Order.open = ($Order.status is not SHIPPED)
if $Order.total is 0:
    $Order.kind = "free"
else:
    $Order.kind = "paid"
first = Order("SHIPPED", 0)
second = Order("PENDING", 30)
assert([first.shipped, second.shipped], [true, false])
assert([first.open, second.open], [false, true])
assert([first.kind, second.kind], ["free", "paid"])
second.status = "SHIPPED"
assert(second.shipped, true)
SHIPPED = "PENDING"
assert([first.shipped, second.shipped], [false, false])
first.status = "PENDING"
assert(first.shipped, true)
delete SHIPPED
assert([first.shipped, second.shipped], [false, true])
second.total = 0
assert(second.kind, "free")
second.shipped
# return: true
---
# Nucleoid uses is comparisons in callbacks literals conditions and loops

statuses = ["SHIPPED", "PENDING", "SHIPPED"]
shipped = statuses.filter(s => s is SHIPPED)
pending = statuses.filter(lambda s: s is not SHIPPED)
assert(shipped.length, 2)
assert(pending, ["PENDING"])
mode = "FAST"
summary = { fast: mode is FAST, slow: mode is SLOW }
flags = [mode is FAST, mode is not FAST]
speed = 10 if mode is FAST else 1
label = (mode is FAST) ? "fast" : "slow"
pick = ["slow", "fast"][mode is FAST ? 1 : 0]
everyone = (all s in statuses: s is SHIPPED)
banner = `${(mode is FAST)}`
assert(summary, { fast: true, slow: false })
assert(flags, [true, false])
assert(speed, 10)
assert(label, "fast")
assert(pick, "fast")
assert(everyone, false)
assert(banner, "true")
if any s in statuses: s is PENDING:
    waiting = "yes"
else:
    waiting = "no"
assert(waiting, "yes")
mode = "SLOW"
assert(summary, { fast: false, slow: true })
assert(speed, 1)
assert(label, "slow")
assert(banner, "false")
countdown = 3
steps = 0
while countdown is not 0:
    countdown -= 1
    steps += 1
assert(steps, 3)
def same(value) =>
    (value is EXACT)
assert(same("EXACT"), true)
assert(same("OTHER"), false)
statuses.push("PENDING")
assert(pending, ["PENDING", "PENDING"])
steps
# return: 3
---
# Nucleoid keeps is as an ordinary name where a statement can follow

is = 4
value = 1
result = value is 4
assert(result, 1)
assert(is, 4)
def increment(is) => is + 1
assert(increment(1), 2)
record = { k: value is: 3 }
assert(record.k, 1)
assert(record.is, 3)
text = `${value is 4}`
assert(text, "1")
same = (value is 1)
assert(same, true)
named = (value is is)
assert(named, false)
failure = ""
try:
    while value() is x: T
catch error:
    failure = error
assert(failure, ReferenceError("while is not defined"))
named
# return: false
---
# Nucleoid assigns the properties that a constructor header names with this

class Point(this.x, this.y)
origin = Point(0, 0)
corner = Point(3, 4)
assert(corner, {"id": "corner", "x": 3, "y": 4})
assert([typeof corner, Point.length], [Object, 2])
$Point.far = $Point.x + $Point.y > 5
assert([origin.far, corner.far], [false, true])
corner.x = 1
assert(corner.far, false)
[origin.x, corner.x]
# return: [0,1]
---
# Nucleoid initializes self parameters before the constructor body reads them

class Rectangle(self.width, self.height):
    this.area = width * height
    this.square = this.width == this.height
box = Rectangle(2, 5)
tile = Rectangle(3, 3)
assert([box.width, box.height, box.area, box.square], [2, 5, 10, false])
assert([tile.area, tile.square, Rectangle.length], [9, true, 2])
box.width = 5
assert([box.area, box.square], [10, true])
box.square
# return: true
---
# Nucleoid fills typed initializing parameters from literal defaults and labels

class Account(this.owner: str, this.balance: int = 0, this.currency = "EUR")
plain = Account("ana")
funded = Account("bo", 250)
named = Account(currency="USD", owner="cy")
mixed = Account("di", currency="GBP")
assert([plain.owner, plain.balance, plain.currency], ["ana", 0, "EUR"])
assert([funded.owner, funded.balance, funded.currency], ["bo", 250, "EUR"])
assert([named.owner, named.balance, named.currency], ["cy", 0, "USD"])
assert([mixed.owner, mixed.balance, mixed.currency], ["di", 0, "GBP"])
Account.length
# return: 4
---
# Nucleoid leaves missing initializing parameters null

class Contact(this.name, this.email, this.phone)
$Contact.reach = $Contact.email ?? $Contact.phone ?? "unreachable"
ana = Contact("ana", "ana@example.com")
bo = Contact(phone="555 0100", name="bo")
cy = Contact("cy")
assert([ana.phone, bo.email, cy.email, cy.phone], [null, null, null, null])
assert([ana.reach, bo.reach, cy.reach], ["ana@example.com", "555 0100", "unreachable"])
cy.phone = "555 0199"
assert(cy.reach, "555 0199")
Contact.length
# return: 3
---
# Nucleoid collects final rest arguments into an initializing parameter

class Route(this.name, ...this.stops)
$Route.count = $Route.stops.length
north = Route("north", "oslo", "lillehammer", "trondheim")
loop = Route("loop")
assert(north.stops, ["oslo", "lillehammer", "trondheim"])
assert([north.name, north.count, loop.stops, loop.count], ["north", 3, [], 0])
north.stops.push("bodo")
assert(north.count, 4)
north.count
# return: 4
---
# Nucleoid initializes properties from the parameter list of init

class Sensor:
    unit: str
    def init(this.room, this.reading, this.unit = "C"):
        this.alarm = reading > 30
lab = Sensor("lab", 21)
attic = Sensor("attic", 34, "F")
assert([lab.room, lab.reading, lab.unit, lab.alarm], ["lab", 21, "C", false])
assert([attic.room, attic.reading, attic.unit, attic.alarm], ["attic", 34, "F", true])
Sensor.length
# return: 2
---
# Nucleoid initializes subclass parameters before the parent constructor runs

class Vehicle(this.make, this.wheels = 4)
class Truck(this.payload): Vehicle
    super("volvo", 6)
class Van extends Vehicle
class Racer(this.wheels) extends Vehicle:
    super("kart", wheels * 2)
truck = Truck(12)
van = Van("ford")
racer = Racer(3)
assert([truck.make, truck.wheels, truck.payload], ["volvo", 6, 12])
assert([van.make, van.wheels], ["ford", 4])
assert([racer.make, racer.wheels], ["kart", 6])
assert([Vehicle.length, Truck.length, Van.length, Racer.length], [0, 1, 1, 1])
racer.wheels
# return: 6
---
# Nucleoid applies class level rules to initialized properties

class Order(this.item, this.qty, this.price)
$Order.total = $Order.qty * $Order.price
$Order.bulk = $Order.qty >= 10
tea = Order("tea", 2, 4)
nuts = Order("nuts", 12, 1)
revenue = Order.sum(order -> order.total)
assert([tea.total, tea.bulk, nuts.total, nuts.bulk, revenue], [8, false, 12, true, 20])
tea.qty = 10
assert([tea.total, tea.bulk, revenue], [40, true, 52])
revenue
# return: 52
---
# Nucleoid explains initialized properties by their constructor assignments

class Shipment(this.origin, this.weight)
$Shipment.heavy = $Shipment.weight > 10
parcel = Shipment("oslo", 3)
assert((why parcel.weight).map(step -> step.rule), ["$Shipment.weight = weight"])
assert((why parcel.heavy).map(step -> step.rule), ["$Shipment.heavy = $Shipment.weight>10", "$Shipment.weight = weight"])
parcel.weight = 12
assert([parcel.heavy, (why parcel.weight)[0].rule], [true, "$Shipment.weight = 12"])
parcel.heavy
# return: true
---
# Nucleoid rolls back an instance whose initializing constructor fails

class Ticket(this.code, this.seat):
    if seat > 100:
        throw "no such seat"
first = Ticket("A1", 4)
try:
    Ticket("Z9", 400)
catch error:
    reason = error
assert([Ticket.length, reason], [1, "no such seat"])
assert([first.code, first.seat], ["A1", 4])
Ticket.length
# return: 1
---
# Nucleoid narrows an explanation to the paths from one source

price = 10
qty = 3
shipping = 5
subtotal = price * qty
total = subtotal + shipping
link = total |> why |> path(price)
assert(link.map(s -> s.node), ["total", "subtotal", "price"])
assert(link[0].from, ["subtotal", "shipping"])
assert(link[1].rule, "subtotal = price*qty")
assert((why total).length, 5)
shipping = price + 1
assert(link.map(s -> s.node), ["total", "subtotal", "shipping", "price"])
price = 4
assert(link.map(s -> s.holds), [17, 12, 5, 4])
---
# Nucleoid narrows an impact to the paths toward one target

price = 10
fee = 2
tax = price * 2
net = price + fee
total = tax + net
label = total * 3
report = price - 1
reach = price |> affects |> path(total)
assert((affects price).map(s -> s.node), ["tax", "net", "report", "total", "label"])
assert(reach.map(s -> s.node), ["tax", "net", "total"])
assert(reach[2].from, ["tax", "net"])
total = tax + net + report
assert(reach.map(s -> s.node), ["tax", "net", "report", "total"])
price = 4
assert(reach.map(s -> s.holds), [8, 6, 3, 17])
---
# Nucleoid keeps every path through a diamond in walk order

seed = 2
left = seed + 1
right = seed * 3
spare = 7
joined = left + right + spare
mixed = joined * 2
paths = mixed |> why |> path(seed)
assert((why mixed).map(s -> s.node), ["mixed", "joined", "left", "right", "spare", "seed"])
assert(paths.map(s -> s.node), ["mixed", "joined", "left", "right", "seed"])
left = 4
assert(paths.map(s -> s.node), ["mixed", "joined", "right", "seed"])
spare = seed - 1
assert(paths.map(s -> s.node), ["mixed", "joined", "right", "spare", "seed"])
seed = 3
assert(paths.map(s -> s.holds), [30, 15, 9, 2, 3])
---
# Nucleoid selects no step until a path reaches the target

a = 1
b = a + 1
c = 5
d = b * c
z = 9
gap = d |> why |> path(z)
own = d |> why |> path(d)
rest = model |> path(z)
assert([gap, rest.map(s -> s.node)], [[], ["z"]])
assert(own.map(s -> s.node), ["d"])
c = z + 1
assert(gap.map(s -> s.node), ["d", "c", "z"])
assert(gap.map(s -> s.holds), [20, 10, 9])
b = c * 2
assert(gap.map(s -> s.node), ["d", "b", "c", "z"])
assert(rest.map(s -> s.node), ["b", "c", "d", "z"])
assert(own[0].holds, 200)
---
# Nucleoid keeps path selections live as rules and targets change

base = 2
rate = base * 3
fee = 4
cost = rate + fee
via = cost |> why |> path(base)
assert(via.map(s -> s.node), ["cost", "rate", "base"])
rate = 6
assert(via, [])
fee = base + 1
assert(via.map(s -> s.node), ["cost", "fee", "base"])
delete base
assert([via, fee], [[], null])
base = 5
assert(via.map(s -> s.holds), [12, 6, 5])
---
# Nucleoid finds derivation paths in class level rules

class Item(price, qty):
    this.price = price
    this.qty = qty
$Item.subtotal = $Item.price * $Item.qty
$Item.total = $Item.subtotal + 1
$Item.via = $Item.total |> why |> path($Item.price)
pen = Item(2, 5)
cup = Item(3, 1)
assert(pen.via.map(s -> s.node), ["pen.total", "pen.subtotal", "pen.price"])
assert(cup.via.map(s -> s.holds), [4, 3, 3])
assert(pen.via[2].rule, "$Item.price = price")
pen.qty = 7
assert(pen.via.map(s -> s.holds), [15, 14, 2])
$Item.total = $Item.qty + 1
assert([pen.via, cup.via], [[], []])
---
# Nucleoid composes path selections with state filters

base = 2
rate = base * 2
fee = 1
cost = rate * 4
price = cost + fee
chain = price |> why |> path(base) |> derived
facts = price |> why |> stated |> path(base)
assert(chain.map(s -> s.node), ["price", "cost", "rate"])
assert(facts.map(s -> s.node), ["base"])
fee = base + 1
assert(chain.map(s -> s.node), ["price", "cost", "fee", "rate"])
base = 5
assert(chain.map(s -> s.holds), [46, 40, 6, 10])
assert(facts[0].holds, 5)
---
# Nucleoid uses path selections in functions and quantifiers

class Order(qty, price):
    this.qty = qty
    this.price = price
$Order.subtotal = $Order.qty * $Order.price
$Order.total = $Order.subtotal + 1
$Order.big = any s in $Order.total |> why |> path($Order.qty): s.holds > 10
pen = Order(2, 3)
cup = Order(4, 3)
def link() => (pen.total |> why |> path(pen.qty)).map(s -> s.node)
bulk = all s in pen.total |> why |> path(pen.qty): s.holds > 1
assert(link(), ["pen.total", "pen.subtotal", "pen.qty"])
assert([pen.big, cup.big, bulk], [false, true, true])
pen.qty = 5
assert([pen.big, cup.big, bulk], [true, true, true])
pen.qty = 1
assert([pen.big, cup.big, bulk], [false, true, false])
$Order.total = $Order.price + 1
assert(link(), [])
assert([pen.big, cup.big, bulk], [false, false, true])
---
# Nucleoid keeps path an ordinary name beside path stages

class Route(stops):
    this.stops = stops
    def path() => this.stops.join(">")
def hops(path) => path.length - 1
trip = Route(["oslo", "bergen"])
plan = {path: "coast"}
legs = hops(trip.stops)
fare = legs * 40
via = fare |> why |> path(trip.stops)
assert([trip.path(), plan.path, legs], ["oslo>bergen", "coast", 1])
assert(via.map(s -> s.node), ["fare", "legs", "trip.stops"])
trip.stops.push("tromso")
assert(via.map(s -> s.holds), [80, 2, ["oslo", "bergen", "tromso"]])
---
# Nucleoid restores path selections after failed changes and hypotheticals

seed = 1
mid = seed + 1
top = mid * 2
side = 3
via = top |> why |> path(side)
frozen = (top |> why |> path(seed)).value
assert(via, [])
try:
    mid = seed + side
    assert(via.map(s -> s.node), ["top", "mid", "side"])
    throw "UNDO"
catch error:
    assert(error, "UNDO")
assert(via, [])
plan = suppose mid = side * 2: via.map(s -> s.holds)
assert([plan, via], [[12, 6, 3], []])
mid = side + 1
seed = 7
assert(via.map(s -> s.holds), [8, 4, 3])
assert(frozen.map(s -> s.node), ["top", "mid", "seed"])
assert(frozen.map(s -> s.holds), [4, 2, 1])
---
# Nucleoid declares named functions with the function keyword

function double(amount) {
    return amount * 2
}
function area(width, height) { return width * height }
function describe(size) {
    if size > 10:
        return "large"
    return "small"
}
room = area(3, 4)
assert(double(room), 24)
assert(describe(room), "large")
assert(describe(double(2)), "small")
room = area(2, 2)
assert([room, describe(room)], [4, "small"])
double(5)
# return: 10
---
# Nucleoid keeps function declarations equivalent to def declarations

def fee(amount) => amount / 10
base = 200
charge = fee(base)
assert(charge, 20)
function fee(amount) { return amount / 4 }
assert(charge, 50)
function label(name, count: int = 1, ...tags) { return [name, count, tags] }
assert(label("a"), ["a", 1, []])
assert(label("b", 2, "x", "y"), ["b", 2, ["x", "y"]])
def fee(amount) => amount / 2
base = 300
charge
# return: 150
---
# Nucleoid declares methods and init with the function keyword

class Account:
    owner: str
    function init(owner, balance = 0) {
        this.owner = owner
        this.balance = balance
    }
    function deposit(amount) { return this.balance + amount }
    def withdraw(amount) => this.balance - amount
$Account.rich = $Account.balance > 100
ana = Account("ana", 80)
bo = Account("bo")
assert([ana.owner, ana.balance, bo.balance], ["ana", 80, 0])
assert([ana.deposit(30), ana.withdraw(30)], [110, 50])
assert([ana.rich, bo.rich], [false, false])
ana.balance = 150
assert(ana.rich, true)
Account.length
# return: 2
---
# Nucleoid declares functions wherever a statement may start

ready = true
if ready:
    function greet(name) { return "hi " + name }
{ function shout(name) { return name + "!" } }
function outer(x) {
    function inner(y) { return y * 10 }
    return inner(x) + 1
}
function twice(x) { return x * 2 } total = twice(3)
assert(greet("ana"), "hi ana")
assert(shout("bo"), "bo!")
assert(outer(2), 21)
assert(total, 6)
---
# Nucleoid keeps function declarations live without hoisting

try:
    early = rate(2)
catch error:
    assert(error, ReferenceError("rate is not defined"))
function rate(x) { return x * 2 }
base = 3
charge = rate(base)
frozen = rate.value
function rate(x) { return x * 5 }
assert(charge, 15)
assert(frozen(3), 6)
base = 4
assert(charge, 20)
assert((why charge)[0].from, ["rate", "base"])
charge
# return: 20
---
# Nucleoid rolls back failed function declarations

function fee(amount) { return amount + 1 }
total = fee(10)
try:
    function fee(amount) { return amount * 100 }
    assert(total, 1000)
    throw "UNDO"
catch error:
    assert(error, "UNDO")
assert(total, 11)
assert(fee(1), 2)
failure = ""
try:
    function fee(amount) { return missing + amount }
catch error:
    failure = error
assert(failure, ReferenceError("missing is not defined"))
assert([total, fee(2)], [11, 3])
---
# Nucleoid declares enums whose members name themselves

enum Status: PENDING, PAID, SHIPPED
assert(Status.PAID, "PAID")
assert(Status["SHIPPED"], "SHIPPED")
assert(Status, {PENDING: "PENDING", PAID: "PAID", SHIPPED: "SHIPPED"})
assert(Status.REFUNDED, null)
label = Status.PENDING + " order"
assert(label, "PENDING order")
typeof Status
# return: "Object"
---
# Nucleoid gives enum members explicit scalar values

enum Level: LOW = 1, HIGH = 10
enum Size: LARGE = "XL", SMALL, MEDIUM, CUSTOM = null
enum Move: BACK = -1, AHEAD = 1
enum Flag: ON = true, OFF = false
assert([Level.LOW, Level.HIGH], [1, 10])
assert([Size.SMALL, Size.LARGE, Size.CUSTOM], ["SMALL", "XL", null])
assert(Size, {LARGE: "XL", SMALL: "SMALL", MEDIUM: "MEDIUM", CUSTOM: null})
assert(Move.BACK + Move.AHEAD, 0)
assert(Flag.ON && !Flag.OFF, true)
steps = Level.HIGH * 2 + Move.BACK
steps
# return: 19
---
# Nucleoid declares enums in indented blocks

enum Color:
    RED
    GREEN
    BLUE
    BLACK = "#000"
enum Mode:
    AUTO
pen = {color: Color.GREEN, mode: Mode.AUTO}
assert(pen, {color: "GREEN", mode: "AUTO"})
assert([Color.RED, Color.BLACK], ["RED", "#000"])
enum Mode: AUTO = "auto", MANUAL
assert(pen.mode, "auto")
pen.color
# return: "GREEN"
---
# Nucleoid keeps class level rules over enum members live

enum Status: PENDING, PAID, SHIPPED
class Order(total, status):
    this.total = total
    this.status = status
$Order.billable = $Order.status != Status.PENDING
first = Order(30, Status.PENDING)
second = Order(45, Status.PAID)
waiting = Order.filter(o -> o.status == Status.PENDING)
assert([first.billable, second.billable, waiting.length], [false, true, 1])
first.status = Status.SHIPPED
assert([first.billable, waiting.length], [true, 0])
second.status
# return: "PAID"
---
# Nucleoid updates readers when an enum is declared again

enum Plan: FREE, PRO
choice = Plan.TEAM
assert(choice, null)
price = Plan.PRO == "PRO" ? 20 : 0
enum Plan: FREE, PRO, TEAM
assert([choice, price], ["TEAM", 20])
enum Plan: PRO = "PREMIUM", FREE, TEAM
assert([choice, price], ["TEAM", 0])
assert((why choice).map(s -> s.node), ["choice", "Plan"])
(affects Plan).map(s -> s.node)
# return: ["choice","price"]
---
# Nucleoid declares enums wherever a statement may start

def kinds():
    enum Kind: BOOK, GAME
    return Kind.GAME
assert([kinds(), Kind.BOOK], ["GAME", "BOOK"])
Pick = "outer"
def pick(Pick):
    enum Pick: LEFT, RIGHT
    return Pick.RIGHT
assert([pick(null), Pick], ["RIGHT", "outer"])
ready = true
if ready: enum Mood: CALM, BUSY
{ enum Tier: GOLD, SILVER }
count = 2 enum Pair: ONE, TWO
assert([Mood.BUSY, Tier.GOLD, Pair.TWO, count], ["BUSY", "GOLD", "TWO", 2])
---
# Nucleoid rolls back enum declarations with their transactions

enum Status: OPEN, CLOSED
view = Status.CLOSED
try:
    enum Status: OPEN, DONE
    assert(view, null)
    throw "UNDO"
catch error:
    assert(error, "UNDO")
assert(view, "CLOSED")
failure = ""
try:
    enum Status: CLOSED = "SHUT", OPEN
    missing + 1
catch error:
    failure = error
assert(failure, ReferenceError("missing is not defined"))
seen = suppose: enum Status: OPEN, DONE; view
assert([seen, view], [null, "CLOSED"])
assert(Status, {OPEN: "OPEN", CLOSED: "CLOSED"})
---
# Nucleoid keeps enum an ordinary name outside enum declarations

enum = 5
assert(enum * 2, 10)
try:
    enum Size: SMALL
catch error:
    assert(error, ReferenceError("Missing definition"))
enum Size: SMALL, LARGE
enum Unit:
    ONLY
record = {enum: enum, size: Size.LARGE, unit: Unit.ONLY}
assert(record, {enum: 5, size: "LARGE", unit: "ONLY"})
enum = 7
assert(record.enum, 7)
---
# Nucleoid builds lists with comprehensions

prices = [12, 30, 8]
doubled = [price * 2 for price in prices]
labels = ["#" + price for price in prices]
assert(doubled, [24, 60, 16])
assert(labels, ["#12", "#30", "#8"])
prices.push(5)
assert(doubled, [24, 60, 16, 10])
prices = [1, 2]
assert(doubled, [2, 4])
rate = 3
scaled = [price * rate for price in prices]
assert(scaled, [3, 6])
rate = 10
scaled
# return: [10,20]
---
# Nucleoid filters comprehensions with an if clause

scores = [40, 75, 90, 55]
passing = [score for score in scores if score >= 50]
assert(passing, [75, 90, 55])
bar = 60
curved = [score + 5 for score in scores if score >= bar]
assert(curved, [80, 95])
bar = 50
assert(curved, [80, 95, 60])
scores.push(70)
assert(curved, [80, 95, 60, 75])
grades = ["high" if score > 80 else "mid" for score in scores if score > 50]
assert(grades, ["mid", "high", "mid", "mid"])
curved
# return: [80,95,60,75]
---
# Nucleoid keeps comprehensions over class populations live

class Order(item, qty):
    this.item = item
    this.qty = qty
minimum = 2
bulk = [order.item for order in Order if order.qty >= minimum]
Order("pen", 1)
ink = Order("ink", 3)
assert(bulk, ["ink"])
Order("cap", 5)
assert(bulk, ["ink", "cap"])
minimum = 4
assert(bulk, ["cap"])
ink.qty = 9
assert(bulk, ["ink", "cap"])
bulk
# return: ["ink","cap"]
---
# Nucleoid accepts of in comprehensions over lists and populations

class Room(name, temp):
    this.name = name
    this.temp = temp
target = 20
names = [room.name for room of Room]
cold = [room.name for room of Room if room.temp < target]
lab = Room("lab", 18)
Room("hall", 22)
readings = [17, 21]
offsets = [reading - target for reading of readings]
assert(names, ["lab", "hall"])
assert(cold, ["lab"])
assert(offsets, [-3, 1])
assert(offsets, [reading - target for reading in readings])
target = 23
assert(cold, ["lab", "hall"])
lab.temp = 25
cold
# return: ["hall"]
---
# Nucleoid writes comprehensions across lines

readings = [18, 25, 31, 22]
warm = [
    reading - 20
    for reading in readings
    if reading > 20
]
assert(warm, [5, 11, 2])
pairs = [[reading, reading * 2]
         for reading of readings
         if reading < 20]
assert(pairs, [[18, 36]])
readings.push(28)
assert(warm, [5, 11, 2, 8])
warm
# return: [5,11,2,8]
---
# Nucleoid nests comprehensions that read only their own names

grid = [[1, 2], [3, 4]]
scaled = [[cell * 10 for cell in row] for row in grid]
assert(scaled, [[10, 20], [30, 40]])
large = [[cell for cell in row if cell > 1] for row in grid]
assert(large, [[2], [3, 4]])
sizes = [row.length for row in [[cell for cell in row if cell > 2] for row in grid]]
assert(sizes, [0, 2])
grid.push([5, 6])
assert(scaled, [[10, 20], [30, 40], [50, 60]])
large
# return: [[2],[3,4],[5,6]]
---
# Nucleoid writes comprehensions wherever a list may start

stock = [3, 0, 7, 2]
def doubled(values):
    return [value * 2 for value in values]
def count(values) => values.length
summary = {low: [qty for qty in stock if qty < 3], total: count([qty for qty of stock if qty > 0])}
note = `restock ${[qty + 10 for qty in stock if qty == 0]}`
assert(doubled(stock), [6, 0, 14, 4])
assert(summary, {low: [0, 2], total: 3})
assert(note, "restock 10")
status = [qty for qty in stock if qty > 5].length > 0 ? "order" : "wait"
assert(status, "order")
if [qty for qty in stock if qty > 9].length == 0:
    quiet = true
assert(quiet, true)
[qty - 1 for qty in stock if qty > 1]
# return: [2,6,1]
---
# Nucleoid keeps comprehensions identical to their map and filter spellings

stock = [3, 0, 7]
limit = 1
listed = [qty * 2 for qty in stock if qty > limit]
spelled = stock.filter(qty => qty > limit).map(qty => qty * 2)
plain = [qty + 1 for qty of stock]
assert(listed, spelled)
assert(plain, stock.map(qty => qty + 1))
assert((why listed).map(step -> step.rule), ["listed = stock.filter(function()).map(function())", "stock = [3,0,7]", "limit = 1"])
assert((why plain).map(step -> step.rule), ["plain = stock.map(function())", "stock = [3,0,7]"])
assert((affects limit).map(step -> step.node), ["listed", "spelled"])
try:
    limit = 5
    assert(listed, [14])
    throw "UNDO"
catch error:
    assert(error, "UNDO")
assert([listed, spelled], [[6, 14], [6, 14]])
listed
# return: [6,14]
---
# Nucleoid keeps cotangents reactive

angle = 0
ratio = Math.cot(angle)
assert(ratio, Number.POSITIVE_INFINITY)
angle = 1
assert(ratio, 1 / Math.tan(1))
angle = -0
assert(ratio, Number.NEGATIVE_INFINITY)
angle = -1
assert(ratio, 1 / Math.tan(-1))
angle = Number.NaN
assert(Number.isNaN(ratio), true)
angle = 0.5
assert(ratio, 1 / Math.tan(0.5))
---
# Nucleoid reads receiver properties in class level rules

class Order(price, qty):
    this.price = price
    this.qty = qty
$Order.total = .price * .qty
first = Order(4, 5)
second = Order(2, 10)
assert([first.total, second.total], [20, 20])
first.qty = 6
assert([first.total, second.total], [24, 20])
$Order.change = .total - .total@first
second.price = 3
assert([first.change, second.change], [4, 10])
assert((why first.total)[0].rule, "$Order.total = $Order.price*$Order.qty")
first.total
# return: 24
---
# Nucleoid reads receiver properties through nested targets

person = {first: "Ada", last: "Lovelace"}
order = {customer: person, qty: 2, price: 5}
order.total = .qty * .price
order.customer.label = .first + " " + .last
assert([order.total, person.label], [10, "Ada Lovelace"])
person.first = "Augusta"
person.last = "King"
order.qty = 3
order.price = 4
[order.total, order.customer.label]
# return: [12,"Augusta King"]
---
# Nucleoid reads receiver properties through this and self

class Rect(w, h):
    this.w = w
    this.h = h
    this.area = .w * .h
    def square():
        self.w = .h
        return self.area
r = Rect(2, 3)
assert(r.area, 6)
r.w = 5
assert(r.area, 15)
assert(r.square(), 9)
r.h = 4
[r.w, r.area]
# return: [4,16]
---
# Nucleoid reads receiver properties in compound assignments

account = {balance: 100, deposit: 20, fee: 5}
account.balance += .deposit
assert(account.balance, 120)
account.deposit = 30
assert(account.balance, 150)
account.balance -= .fee
assert(account.balance, 145)
account.fee = 10
assert(account.balance, 135)
account.deposit = 0
account.balance
# return: 135
---
# Nucleoid reads receiver properties inside lambdas of class level rules

class Cell(h):
    this.h = h
$Cell.taller = Cell.filter(other -> other.h > .h).length
low = Cell(1)
high = Cell(3)
mid = Cell(2)
assert([low.taller, high.taller, mid.taller], [2, 0, 1])
high.h = 0
[low.taller, high.taller, mid.taller]
# return: [1,2,0]
---
# Nucleoid reads receiver properties in conditionals objects and templates

item = {name: "lamp", stock: 3, price: 20}
item.status = .stock > 0 ? "open" : "sold"
item.card = {title: .name, cost: .price * 2}
item.label = `${.name}: ${.stock} left`
assert([item.status, item.card, item.label], ["open", {title: "lamp", cost: 40}, "lamp: 3 left"])
item.stock = 0
item.name = "desk"
item.price = 15
[item.status, item.card, item.label]
# return: ["sold",{"title":"desk","cost":30},"desk: 0 left"]
---
# Nucleoid continues receiver reads with members indexes and calls

team = {members: ["ada", "grace"], lead: {name: "ada"}}
team.size = .members.length
team.first = .members[0]
team.title = .lead.name.toUpperCase()
assert([team.size, team.first, team.title], [2, "ada", "ADA"])
team.members = ["linus", "ada", "grace"]
team.lead.name = "grace"
[team.size, team.first, team.title]
# return: [3,"linus","GRACE"]
---
# Nucleoid rolls back failed receiver assignments

class Cart(qty, price):
    this.qty = qty
    this.price = price
$Cart.total = .qty * .price
cart = Cart(2, 5)
try:
    cart.price = 7
    assert(cart.total, 14)
    throw "UNDO"
catch error:
    assert(error, "UNDO")
assert([cart.price, cart.total], [5, 10])
try:
    $Cart.total = .total + 1
catch error:
    assert(error, TypeError("Circular Dependency"))
assert(cart.total, 10)
cart.qty = 3
cart.total
# return: 15
---
# Nucleoid flattens nested lists by one level

nested = [[1, 2], 2, [[3, [4]]], [], "ab"]
once = nested.flat()
twice = once.flat(1)
assert(once, [1, 2, 2, [3, [4]], "ab"])
assert(twice, [1, 2, 2, 3, [4], "ab"])
assert(nested, [[1, 2], 2, [[3, [4]]], [], "ab"])
nested.push([[5]], 6)
assert(once, [1, 2, 2, [3, [4]], "ab", [5], 6])
assert(twice, [1, 2, 2, 3, [4], "ab", 5, 6])
twice.length
# return: 8
---
# Nucleoid flattens lists to a given depth

deep = [1, [2, [3, [4, [5]]]], [[6]]]
two = deep.flat(2)
three = deep.flat(3)
same = deep.flat(0)
assert(two, [1, 2, 3, [4, [5]], 6])
assert(three, [1, 2, 3, 4, [5], 6])
assert(same, [1, [2, [3, [4, [5]]]], [[6]]])
deep.push([[7, [8]]])
assert(two, [1, 2, 3, [4, [5]], 6, 7, [8]])
assert(three, [1, 2, 3, 4, [5], 6, 7, 8])
assert(same, deep)
three.length
# return: 8
---
# Nucleoid keeps flattened lists live

shifts = [["ana", "bo"], ["cy"]]
staff = shifts.flat()
size = staff.length
assert([staff, size], [["ana", "bo", "cy"], 3])
shifts.push(["di"])
assert([staff, size], [["ana", "bo", "cy", "di"], 4])
shifts = [["eve"], [], ["fay", "gus"]]
assert([staff, size], [["eve", "fay", "gus"], 3])
shifts.shift()
assert([staff, size], [["fay", "gus"], 2])
size
# return: 2
---
# Nucleoid flattens lists in class level rules

class Team(squads):
    this.squads = squads
$Team.members = $Team.squads.flat()
red = Team([["ana", "bo"], ["cy"]])
blue = Team([["di"]])
assert([red.members, blue.members], [["ana", "bo", "cy"], ["di"]])
red.squads.push(["ed"])
blue.squads = [["fay"], ["gus", "hal"]]
assert([red.members, blue.members], [["ana", "bo", "cy", "ed"], ["fay", "gus", "hal"]])
red.members.length
# return: 4
---
# Nucleoid queries the cells of flattened grids

grid = [[1, 0, 2], [0, 3, 2]]
cells = grid.flat()
total = cells.sum()
colors = cells.distinct()
filled = cells.count(cell => cell > 0)
assert([total, colors, filled], [8, [1, 0, 2, 3], 4])
grid.push([5, 0, 0])
assert([total, colors, filled], [13, [1, 0, 2, 3, 5], 5])
grid = [[4, 4], [0]]
[total, colors, filled]
# return: [8,[4,0],2]
---
# Nucleoid rolls back failed flattening updates

rows = [[1, [2]], [3]]
cells = rows.flat()
try:
    rows.push([4])
    assert(cells, [1, [2], 3, 4])
    throw "UNDO"
catch error:
    assert(error, "UNDO")
assert([rows, cells], [[[1, [2]], [3]], [1, [2], 3]])
try:
    cells = rows.flat(2)
    assert(cells, [1, 2, 3])
    throw "UNDO"
catch error:
    assert(error, "UNDO")
rows.push([[5]])
cells
# return: [1,[2],3,[5]]
---
# Nucleoid states class predicates with is

class Human(name: str):
    this.name = name
plato = Human("Plato")
every Human is mortal
socrates = Human("Socrates")
assert([plato.mortal, socrates.mortal], [true, true])
$Human is wise
assert(socrates.wise, true)
assert((why socrates.mortal)[0].rule, "$Human.mortal = true")
mortal = false
assert(socrates.mortal, true)
plato.mortal = false
assert([plato.mortal, socrates.mortal], [false, true])
socrates.mortal
# return: true
---
# Nucleoid spells class references with every

class Order(total):
    this.total = total
every Order.big = every Order.total > 10
if every Order.total > 100:
    every Order.vip = true
small = Order(5)
large = Order(200)
assert([small.big, large.big], [false, true])
assert([small.vip, large.vip], [null, true])
small.total = 50
assert(small.big, true)
every Order.tier is "gold"
assert(large.tier, "gold")
small.big
# return: true
---
# Nucleoid quantifies with every as with all

class Check
ready = every check in Check: check.passed
assert(ready, true)
first = Check()
first.passed = true
second = Check()
assert(ready, false)
second.passed = true
assert(ready, true)
scores = [3, 4]
assert(every score in scores: score > 2, all score in scores: score > 2)
assert(every score in scores: score > 3, false)
assert((every score in scores: score > 2) ? "pass" : "fail", "pass")
ready
# return: true
---
# Nucleoid keeps every an ordinary name where no name follows it

every = [1, 2]
total = every.length
assert(total, 2)
assert(every.every(v => v > 0), true)
pairs = {every: 3, other: every[0]}
assert(pairs, {every: 3, other: 1})
def count(every) => every + 1
assert(count(4), 5)
every is [5]
assert(total, 1)
assert(every n in every: n > 4, true)
every
# return: [5]
---
# Nucleoid keeps other is statements on a class reference

class Order(total):
    this.total = total
order = Order(20)
$Order.size is big
assert(order.size, "big")
try:
    $Order is 5
catch error:
    assert(error, ReferenceError("is is not defined"))
assert(order.big, null)
$Order is big
assert(order.big, true)
order.size
# return: "big"
```

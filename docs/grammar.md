# Nucleoid Grammar

This is the complete grammar of Nucleoid source text as the Rust
implementation accepts it, taken from the lexer in `src/lang/ast/lexer/` and
the recursive-descent parser in `src/lang/ast/parser/`. Rules name the
syntax-tree node they build. `nucleoid.spec.md` is normative for behaviour, and
the [Syntax Summary](README.md#14-syntax-summary) lists the same forms
informally, with the NUC that introduced each one.

## Contents

1. [Notation](#1-notation)
2. [Lexical Structure](#2-lexical-structure)
3. [Programs and Suites](#3-programs-and-suites)
4. [Statements](#4-statements)
5. [Functions and Classes](#5-functions-and-classes)
6. [Expressions](#6-expressions)
7. [Parser Behaviour](#7-parser-behaviour)

## 1. Notation

The grammar uses PEG notation in the style of Python's
[grammar specification](https://docs.python.org/3/reference/grammar.html).

```
rule ::= e        definition
e1 e2             sequence
e1 | e2           ordered choice: the first alternative that matches is taken
e?  e*  e+        optional, zero or more, one or more
( e )             grouping
&e                lookahead: e must come next; nothing is consumed
!e                negative lookahead: e must not come next; nothing is consumed
'if'  '=>'        a keyword or operator token
"while"           a contextual word: a NAME token with exactly this spelling
NAME  NEWLINE     a token class produced by the lexer
# text            a comment: the syntax-tree node built, or a note
```

- The parser chooses an alternative by looking ahead at its leading tokens and
  at any lookahead the grammar shows, then commits to it: a later mismatch is a
  syntax error, not a reason to try the next alternative.
- Rules marked *tentative* are the exceptions. The parser tries them in full,
  and when they do not match it consumes nothing and continues with the next
  alternative.
- Lexical rules (section 2) use regular-expression character classes.

## 2. Lexical Structure

### 2.1 Characters, Whitespace and Comments

- Tokens are ASCII. Other characters may appear only inside strings, templates,
  regular expressions and comments; anywhere else they are the syntax error
  `Unexpected character`.
- Space, tab, form feed and carriage return separate tokens. A line feed ends a
  line.
- `#` starts a comment that runs to the end of the line.
- The C-style comments option (`--c-style-comments`, or
  `SyntaxOptions { c_style_comments: true }`) adds `//` line comments and
  `/* ... */` block comments. A block comment ends at the first `*/`, does not
  nest, and is an error when unterminated. Without the option `/` keeps its
  operator and regular-expression readings, so `//` is an empty regular
  expression.

### 2.2 Tokens

```
NAME          ::= [A-Za-z_][A-Za-z0-9_]*           # that is not a reserved keyword
CLASS_REF     ::= '$' [A-Za-z_][A-Za-z0-9_]*       # $Person
NUMBER        ::= [0-9]+ ('.' [0-9]+)? EXPONENT?
                | '.' [0-9]+ EXPONENT?
EXPONENT      ::= [eE] [+-]? [0-9]+
VERSION_INDEX ::= [0-9]+                           # only directly after '@'
STRING        ::= '"' ([^"\\] | ESCAPE)* '"'
                | "'" ([^'\\] | ESCAPE)* "'"
TEMPLATE      ::= '`' ([^`\\] | ESCAPE)* '`'
ESCAPE        ::= '\' [^\n]
REGEX         ::= '/' REGEX_CHAR* '/' [A-Za-z]*     # only in operand position
REGEX_CHAR    ::= [^/\\[\n] | '\' . | '[' ([^\]\\\n] | '\' .)* ']'
```

- Each token is the longest match at its position, so `===`, `**=` and `??`
  are single tokens.
- In a STRING, `\n`, `\t`, `\r` and `\0` stand for line feed, tab, carriage
  return and NUL, and any other escaped character stands for itself. Strings
  may contain raw line breaks.
- In a TEMPLATE, `\n` and `\t` are escapes and any other escaped character
  stands for itself, so `\${` writes a literal `${`. `${ ... }` holds an
  interpolated expression (section 6.6). Templates may contain raw line breaks.
- Numbers have no sign, radix prefix or digit separator: `-1` is unary minus
  applied to `1`.
- A number directly after `@` must be plain digits with no letter, digit or
  underscore after them. It becomes VERSION_INDEX; anything else is the error
  `Expected a non-negative integer, 'first', 'last', or 'previous' after '@'`.
- `/` starts a REGEX only in operand position, that is, when the previous token
  is not a NAME, CLASS_REF, NUMBER, VERSION_INDEX, STRING, TEMPLATE, REGEX,
  `)`, `]`, `}` or `this`, and only when the literal closes on the same line.
  Flags after the closing slash are accepted and ignored. Elsewhere `/` and
  `/=` are operators.

### 2.3 Keywords and Contextual Words

Reserved keywords cannot be used as names:

```
and       catch     class     def       delete    else      false
for       function  if        in        new       not       null
of        or        pass      return    super     this      throw
true      try       typeof
```

In the grammar, `KEYWORD` stands for any of these tokens. Four aliases lex to
the same token as a keyword:

| Alias | Token |
| --- | --- |
| `elif` | `else` |
| `None`, `undefined` | `null` |
| `self` | `this` |

Because an alias is the same token, it is accepted wherever its keyword is:
`elif condition:` is an `else` with a direct condition (section 4.3), and
`obj.self` names the property `this`. Keywords may follow `.` as property
names (`obj.class`), but they cannot be object-literal keys or declared names.

Contextual words are ordinary names except in these positions:

| Word | Position |
| --- | --- |
| `while` | a loop at the start of a statement |
| `enum` | an enum declaration at the start of a statement, outside class bodies |
| `extends` | a parent class after a class header |
| `assert` | an assertion statement, `assert:` |
| `suppose` | a hypothetical block |
| `lambda` | a lambda expression |
| `is` | an equality operator in enclosed positions |
| `all`, `any` | a quantifier, before `NAME in` |
| `sum`, `max`, `min`, `mode` | a noun query, before `of` |
| `why`, `affects`, `stated`, `derived` | a reasoning prefix before a name or class reference; pipeline stages |
| `diff`, `depth`, `path` | pipeline stages |
| `model` | the current model, directly before `\|>` |
| `first`, `last`, `previous` | version selectors after `@` |
| `init` | a constructor method, which accepts `this.` parameters |

The parser also supplies two names: `it`, the item of `for source:`, and
`error`, the default binding of `catch`.

### 2.4 Operators and Delimiters

```
(    )    [    ]    {    }    ,    :    ;    .    @    ?    ??
=>   ->   =    :=   +=   -=   *=   **=  /=   %=   &&=  ||=
==   ===  !=   !==  <    <=   >    >=
+    -    *    **   /    %    !    &&   ||   |>
```

- `=>` and `->` are interchangeable wherever an arrow is accepted (`ARROW`).
- Three `.` tokens written with no space between them (`...`) mark a rest
  parameter or a spread entry.

### 2.5 Layout

A layout pass turns line structure into the tokens NEWLINE, INDENT and DEDENT,
followed by a final EOF.

1. Inside parentheses and brackets, line breaks and indentation produce
   nothing. Braces do not suspend layout.
2. Elsewhere, at the first token of a line, the pass emits NEWLINE if any token
   came before it. It then measures the line's indentation, counting a tab as
   4, any other whitespace character as 1 and comments as 0. A deeper line
   opens a level and emits INDENT. A shallower line closes every deeper level,
   emitting one DEDENT for each; a width between two open levels closes back to
   the shallower one without error.
3. Blank lines and comment-only lines produce nothing.
4. At the end of input the pass emits any pending NEWLINE, one DEDENT for each
   open level, and EOF.

As a result, an expression may continue onto following lines only inside
`( )` or `[ ]`. Inside `{ }` a line break is a NEWLINE: an object literal
accepts line breaks between entries, but an indented continuation line
produces an INDENT that the literal rejects. Write such lines unindented, or
put the literal in parentheses.

## 3. Programs and Suites

```
program        ::= SEP* (statement SEP*)* EOF
SEP            ::= NEWLINE | ';' | INDENT | DEDENT
LINE_SEP       ::= NEWLINE | ';'
stmt_end       ::= NEWLINE | ';' | DEDENT | '}' | EOF
line_end       ::= stmt_end | 'else' | 'catch'

suite          ::= ':' suite_body
suite_body     ::= &(NEWLINE | INDENT) indented_block
                 | statement (';' (statement | &(NEWLINE | EOF)))* !';'
indented_block ::= NEWLINE? LINE_SEP* INDENT LINE_SEP* (statement LINE_SEP*)* (DEDENT | &EOF)
body           ::= suite
                 | &indented_body indented_block       # colon left out before a real indented block
indented_body  ::= INDENT | NEWLINE LINE_SEP* INDENT   # used only as a lookahead
block          ::= '{' SEP* (statement SEP*)* '}'
```

- Statements need no separator. A statement ends where its grammar ends, and
  the next one starts at the following token: `a = 1 b = 2` is two
  assignments, and `let x = 1` is the name `let` followed by `x = 1`.
- At the top level and inside braces, INDENT and DEDENT count as separators,
  so stray indentation is ignored there. Inside an indented block, a line
  indented deeper than its neighbours is a syntax error unless a header before
  it opens a suite.
- An inline suite runs to the end of its line, with its statements separated
  by `;`: `if x: a = 1; b = 2` puts both assignments in the body. After a `;`
  only another statement or the end of the line may follow, so `else` and
  `catch` can follow an inline suite on the same line only without a `;`
  (`if x: a else: b`).
- A `{` at the start of a statement opens a block or a destructuring pattern,
  never an object literal.

## 4. Statements

```
statement ::= destructuring                     # tentative
            | while_stmt                        # tentative
            | enum_stmt
            | if_stmt
            | for_stmt
            | class_stmt
            | function_decl
            | try_stmt
            | 'pass'                            # Stmt::Pass
            | return_stmt
            | 'throw' expression                # Stmt::Throw
            | 'delete' expression               # Stmt::Delete
            | block                             # Stmt::Block
            | type_declaration
            | assert_stmt
            | suppose_stmt
            | value_assignment
            | expression                        # Stmt::Assign for '=', otherwise Stmt::Expression
```

### 4.1 Simple Statements

```
return_stmt      ::= 'return' (!stmt_end expression)?                    # Stmt::Return
type_declaration ::= NAME ':' NAME &(NEWLINE | ';' | DEDENT | EOF)       # Stmt::Declaration
assert_stmt      ::= "assert" ':' expression ',' expression !','         # assert(actual, expected)
suppose_stmt     ::= (conditional '=')? "suppose" (premise &':')? suite  # Stmt::Suppose
premise          ::= !colon_statement clause_expression
colon_statement  ::= type_declaration
                   | "assert" ':'
value_assignment ::= path ':=' expression                                # Stmt::ValueAssign
```

- A type declaration takes exactly one type name followed by the end of the
  statement, so `a: int` declares while `a: b + 1` is a syntax error.
- The parser reads an expression statement's expression first. A following
  `:=` makes it a value assignment, which requires a path (section 6.1). An
  expression that is the name `suppose`, or a plain assignment of it, makes a
  `suppose` block when a premise or the colon follows; otherwise `suppose` is
  an ordinary name.
- A premise is tentative. It must end exactly at the colon, and that colon
  must not directly follow the word `lambda`. Its statement runs first in the
  block.

### 4.2 Destructuring and Enums

```
destructuring     ::= (object_pattern | unpacking_pattern) '=' conditional &line_end
object_pattern    ::= '{' pattern_entry+ '}'
pattern_entry     ::= NAME ':' NAME (',' | &'}' | &(NAME ':'))   # key: name -> name = value.key
                    | NAME (',' | &'}')                          # name      -> name = value.name
unpacking_pattern ::= NAME ',' (NAME (',' NAME)* ','?)?          # a, b      -> a = value[0]; b = value[1]
```

- Destructuring is tried only at the start of a statement: after NEWLINE, `;`,
  INDENT, DEDENT, `{` or `:`, or at the beginning of the source. It is
  tentative: when the value is not a `conditional` followed by a line end, the
  statement is read again as an ordinary statement.
- It lowers to one `Stmt::Assign` per target, each reading the value
  expression again. A pattern may have at most 128 targets.

```
enum_stmt      ::= "enum" !type_declaration enum_name ':' (&(NEWLINE | INDENT) enum_block | enum_line)
enum_name      ::= !("assert" | "lambda" | "suppose") NAME
enum_line      ::= enum_member (',' enum_member)+ &line_end
enum_block     ::= NEWLINE* INDENT NEWLINE* (enum_member (NEWLINE+ | &DEDENT | &EOF))* (DEDENT | &EOF)
enum_member    ::= NAME ('=' scalar_literal)?            # a bare member holds its own name
scalar_literal ::= ('-' | '+')? NUMBER | STRING | 'true' | 'false' | 'null'
```

- An enum lowers to `NAME = <enum>`, a `Stmt::Assign` of `Expr::Enum`. It is
  not recognized anywhere inside a class body.
- The line form needs at least two members. The block form takes one member
  per line, and `;` does not separate members.
- Member names must be distinct, and numbers must be finite.

### 4.3 Conditionals, Loops and Exceptions

```
if_stmt          ::= 'if' condition_clause                                 # Stmt::If
condition_clause ::= clause_expression body else_clause?
else_clause      ::= LINE_SEP* 'else' ( if_stmt                            # else if
                                       | &direct_condition condition_clause
                                       | body )                            # Stmt::Block
direct_condition ::= # lookahead only; see below

for_stmt         ::= 'for' NAME ',' NAME ('of' | 'in') clause_expression suite   # Stmt::ForEntries
                   | 'for' NAME &(':' indented_body) suite                        # Stmt::For over NAME, item "it"
                   | 'for' NAME ('of' | 'in') clause_expression suite             # Stmt::For
while_stmt       ::= "while" !colon_statement clause_expression &':' suite       # Stmt::While
try_stmt         ::= 'try' body LINE_SEP* 'catch' ('(' NAME ')' | NAME)? body    # Stmt::Try
```

- A direct condition is a non-empty header, balanced in `( )`, `[ ]` and
  `{ }`, that ends at a top-level `:` followed by an indented block, or at a
  top-level line break followed by an indented block. A top-level `:` that
  closes a `?` conditional or a quantifier stays in the header. A top-level
  `;`, `:=`, DEDENT or EOF rules the header out. So `else x > 1:` and
  `elif x > 1:` take an indented body, and the same-line form is a syntax
  error.
- `if`, `else`, `try` and `catch` may leave out the colon before a real
  indented block (`body`). `for` and `while` always need it.
- Bodies are suites, not braces: `if (x) { ... }` is a syntax error, while
  `if x: { ... }` is an inline suite holding a block.
- `of` and `in` are interchangeable in `for` headers.
- `while` is tentative. The condition must end exactly at the colon, that
  colon must not directly follow the word `lambda`, and the condition must not
  be `suppose` or an assignment of it. Otherwise `while` is an ordinary name,
  so `while (x) { ... }` is a call of `while` followed by a block.
- `catch` without a name binds `error`.

## 5. Functions and Classes

### 5.1 Function Declarations

```
function_decl  ::= 'function' NAME parameters block                         # Stmt::Function
                 | 'def' NAME def_parameters def_body
def_parameters ::= NAME NAME+ &(ARROW | ':' indented_body)                  # untyped, without commas
                 | bare_parameter (',' bare_parameter)+ &(ARROW | ':' indented_body)
                 | NAME (':' NAME)? &ARROW
                 | NAME ':' NAME &(':' indented_body)
                 | NAME &(':' indented_body)
                 | parameters
bare_parameter ::= NAME (':' NAME)?
def_body       ::= ARROW &indented_body LINE_SEP* INDENT LINE_SEP* expression LINE_SEP* DEDENT
                 | ARROW expression
                 | suite
```

- An arrow body is one expression and becomes `return expression`.
- Parameters without parentheses need an arrow on the same line, or a colon
  followed by a real indented block. A function without parameters needs
  `()`.
- `function NAME` declares a function. `function` followed by `(` is an
  anonymous function expression (section 6.3).

### 5.2 Parameter Lists

```
parameters             ::= '(' (parameter (',' parameter)* ','?)? ')'
parameter              ::= REST? NAME (':' NAME)? ('=' scalar_literal &(',' | ')'))?
constructor_parameters ::= '(' (constructor_parameter (',' constructor_parameter)* ','?)? ')'
constructor_parameter  ::= REST? ('this' '.')? NAME (':' NAME)? ('=' scalar_literal &(',' | ')'))?
REST                   ::= '.' '.' '.'                    # adjacent, with no space between them
```

- A type annotation is a single NAME. A default is a scalar literal, and a
  number default must be finite.
- A rest parameter collects the remaining positional arguments. It must come
  last (a trailing comma may follow it), takes no default, and cannot reuse a
  fixed parameter's name.
- Constructor parameters appear in class headers and `init` methods. There,
  `this.NAME` (or `self.NAME`) also assigns the argument to that property at
  the start of the constructor or `init` body.

### 5.3 Classes

```
class_stmt   ::= 'class' NAME constructor_parameters? class_tail           # Stmt::Class
class_tail   ::= "extends" NAME (bodyless_end | ':' class_body)
               | bodyless_end
               | ':' (NAME &(NEWLINE | INDENT | EOF))? class_body          # NAME is the parent
bodyless_end ::= &stmt_end !(LINE_SEP* INDENT)
class_body   ::= LINE_SEP* ( INDENT LINE_SEP* (class_member LINE_SEP*)* (DEDENT | &EOF)
                           | &EOF
                           | statement )
class_member ::= 'pass'
               | method_decl                                               # ClassDecl::methods
               | type_declaration                                          # ClassDecl::fields
               | statement                                                 # ClassDecl::constructor
method_decl  ::= 'def' "init" constructor_parameters def_body
               | 'function' "init" constructor_parameters block
               | function_decl
```

- `class Name: Parent` and `class Name extends Parent` both name the parent.
  Naming a parent both ways is an error.
- When no indented block follows the colon (and parent), the parser skips line
  breaks and takes the next statement as the whole body, even when that
  statement is on a following line at the same indentation. Write an empty
  class with `pass`, or without a colon (`class Name`).
- Statements in a class body other than methods and typed fields run in the
  constructor.

## 6. Expressions

Some positions read an expression as a *clause*: `if`, `else`, `for` and
`while` headers, `suppose` premises, a subscript's index or slice start, the
consequent after `?`, the condition of a Python-style conditional, and a
quantifier's collection. A top-level `:` after a clause belongs to the
enclosing construct, so `lambda` is an ordinary name inside a clause.

```
clause_expression ::= expression                # read as a clause
```

*Enclosed positions* are clauses, parenthesized expressions, list items,
comprehension parts, object-literal values, call arguments, slice ends and
stage arguments. A nested expression, such as an arrow body, inherits the
position it appears in. Statement level, including an assignment's value
there, is not enclosed.

### 6.1 Assignment, Conditionals and Operators

```
expression  ::= conditional (ASSIGN_OP expression)?                   # Expr::Assign, right-associative
ASSIGN_OP   ::= '=' | '+=' | '-=' | '*=' | '**=' | '/=' | '%=' | '&&=' | '||='
path        ::= NAME (member | version)*
              | (CLASS_REF | 'this') (member | version)+

conditional ::= coalesce ( '?' clause_expression ':' conditional      # Expr::Conditional
                         | 'if' coalesce 'else' conditional )?        # the same node; tentative
coalesce    ::= pipeline ('??' pipeline)*                             # Expr::Logical, right-associative
pipeline    ::= logical_or ('|>' stage)*
stage       ::= "why" | "affects" | "stated" | "derived" | "diff"     # Expr::Reason
              | "depth" '(' expression ')'                            # Expr::Depth
              | "path" '(' expression ')'                             # Expr::Path
logical_or  ::= logical_and (('||' | 'or') logical_and)*              # Expr::Logical
logical_and ::= equality (('&&' | 'and') equality)*                   # Expr::Logical
equality    ::= relational (('==' | '!=' | '===' | '!==' | "is" 'not'?) relational)*   # Expr::Binary
relational  ::= additive (('<' | '<=' | '>' | '>=') additive)*        # Expr::Binary
additive    ::= term (('+' | '-') term)*                              # Expr::Binary
term        ::= unary (('*' | '/' | '%') unary)*                      # Expr::Binary
unary       ::= ("why" | "affects" | "stated" | "derived") &(NAME | CLASS_REF) unary   # Expr::Reason
              | ("sum" | "max" | "min" | "mode") 'of' unary           # Expr::Of
              | ('!' | 'not' | '-' | '+' | 'typeof') unary            # Expr::Unary
              | power
power       ::= postfix ('**' unary)?                                 # Expr::Binary, right-associative
```

- Compound assignment and `:=` need a `path` target. `a op= b` lowers to
  `a = a op b`, and `&&=` and `||=` use the logical operators.
- The Python-style conditional `a if c else b` applies only when `else`
  directly follows the condition. Otherwise nothing is consumed and the `if`
  starts the next statement. It is not read in template interpolations or
  comprehension sources.
- `model` directly before `|>` is the current model (`Expr::Model`). An
  unknown stage name is the type error `NAME is not an operation`.
- `is` and `is not` compare only in enclosed positions, and only in the parse
  pass that enables them (section 7.1). When the right operand is a single
  NAME, the comparison is a word comparison (`BinaryOp::Is` or `IsNot`);
  otherwise it is `==` or `!=`.
- In an object-literal value, a following `NAME:` begins the next property:
  `is:`, `stated NAME:` and `derived NAME:` keep that reading instead of
  forming an operator.
- Noun queries are not read in template interpolations.

Precedence, lowest first:

| Operators | Associativity |
| --- | --- |
| `=` and compound assignments | right |
| `? :`, `if else` | right |
| `??` | right |
| `\|>` | left |
| `\|\|`, `or` | left |
| `&&`, `and` | left |
| `==`, `!=`, `===`, `!==`, `is`, `is not` | left |
| `<`, `<=`, `>`, `>=` | left |
| `+`, `-` | left |
| `*`, `/`, `%` | left |
| prefix `!`, `not`, `-`, `+`, `typeof`, reasoning prefixes, `NOUN of` | prefix |
| `**` | right |
| `.`, `[ ]`, `( )`, `@` | postfix |

Comparisons do not chain (`a < b < c` is `(a < b) < c`), and a prefix
operator applies to a whole power (`-x ** 2` is `-(x ** 2)`).

### 6.2 Postfix Forms

```
postfix   ::= primary (version | member | subscript | call)*
version   ::= '@' (VERSION_INDEX | "first" | "last" | "previous")    # Expr::Version
member    ::= '.' property                                           # Expr::Member
property  ::= NAME | KEYWORD                                         # an alias reads as its keyword
subscript ::= '[' (':' expression? | clause_expression (':' expression?)?) ']'   # Expr::Index, Expr::Slice
call      ::= '(' (argument (',' argument)* ','?)? ')'               # Expr::Call
argument  ::= selector &(',' | ')')                                  # f(.a.b) means f(v => v.a.b)
            | expression
selector  ::= ('.' property)+
```

- `@` cannot start an operand.
- Slices have no step.
- When the callee is a plain name, an argument written `NAME = value` is a
  label (a named constructor argument). Any other assignment-shaped argument,
  such as `(NAME = value)`, stays positional (`UnaryOp::Positional`).

### 6.3 Primary Expressions

```
primary          ::= quantified                                      # not in template interpolations
                   | NAME ARROW arrow_body                           # Expr::Function
                   | lambda
                   | &arrow_parameters parameters ARROW arrow_body
                   | NUMBER | STRING | REGEX | TEMPLATE | NAME | CLASS_REF
                   | 'true' | 'false' | 'null' | 'this'
                   | 'new' primary                                   # new X(...) means X(...)
                   | 'delete' unary                                  # Expr::Delete
                   | 'super' call?                                   # Expr::Super
                   | 'function' parameters block                     # an anonymous function
                   | '(' expression ')'
                   | list_display
                   | object_display
ARROW            ::= '=>' | '->'
arrow_body       ::= block | expression
arrow_parameters ::= # lookahead only: a balanced '( ... )' directly followed by ARROW
quantified       ::= ("all" | "any") NAME 'in' clause_expression ':' expression   # Expr::Quantified
lambda           ::= "lambda" (NAME (',' NAME)*)? ':' expression                   # (names) => expression
```

- Braces after an arrow always open a block, so `x => {a: 1}` is a syntax
  error. A lambda's body is an expression, so `lambda: {a: 1}` returns an
  object.
- `lambda` is read only outside clauses and template interpolations. A
  single-name `lambda x:` also stays a name in an object-literal value, where
  `x:` begins the next property, and where `x:` would begin a type
  declaration or an assertion statement.
- The `(` of `super(...)` must be the next token.
- A quantifier's condition extends as far as an expression can; parenthesize
  the quantifier to combine it with what follows.

### 6.4 Lists and Comprehensions

```
list_display  ::= '[' (expression (comprehension | (',' expression)* ','?))? ']'   # Expr::List
comprehension ::= 'for' NAME ('in' | 'of') expression ('if' expression)?
```

- `[e for x in s]` lowers to `s.map(x => e)`, and `[e for x in s if c]` to
  `s.filter(x => c).map(x => e)`.
- The source is not read as a Python-style conditional, so its `if` is the
  filter. A comprehension takes one `for` and at most one `if`.
- Comprehensions are read only in a later parse pass (section 7.1).

### 6.5 Objects

```
object_display ::= '{' LINE_SEP* (object_entry LINE_SEP* (',' LINE_SEP* | &'}' | &(object_key ':')))* '}'
object_entry   ::= REST expression                     # spread
                 | NAME &(LINE_SEP* (',' | '}'))       # shorthand: {a} means {a: a}
                 | object_key ':' expression
object_key     ::= NAME | STRING | NUMBER              # a number key becomes its text
```

- Entries are separated by `,`, by line breaks or `;`, or by nothing when the
  next entry is written `key:` (`{a: 1 b: 2}`).
- A literal with a spread entry is an `Expr::SpreadObject`, in which later
  entries win; any other literal is an `Expr::ObjectLiteral`.
- Values are read in object-value position (section 6.1).

### 6.6 Template Literals

```
template      ::= (TEMPLATE_TEXT | ESCAPE | '${' interpolation '}')*   # Expr::Template
interpolation ::= expression TOKEN*                                    # the tokens after the expression are ignored
```

- `template` is the text between a TEMPLATE token's backticks.
  TEMPLATE_TEXT is a run of characters other than `\` and the opening `${`,
  and TOKEN is any token.
- An interpolation ends at the `}` that balances its `${`. With C-style
  comments enabled, the end is found by lexing, so braces inside strings and
  comments do not count.
- The interpolation is parsed as a separate expression. Quantifiers, noun
  queries, `lambda` and the Python-style `if` are not read there, and the
  tokens after the expression are ignored: `${a if b else c}` reads `a`, and
  `${sum of xs}` reads `sum`.

## 7. Parser Behaviour

### 7.1 Parse Passes

The parser reads a program up to four times and keeps the first reading that
succeeds:

1. without `is` comparisons or list comprehensions;
2. with `is` comparisons, when the source contains the name `is` or a template
   literal;
3. with list comprehensions, when the source contains `for` or a template
   literal;
4. with both, under the conditions of 2 and 3.

A later reading applies only to a program that failed the earlier ones, so it
never changes a program that parses without it.

### 7.2 Entry Points

| Function | Start rule |
| --- | --- |
| `parse`, `parse_program` and their `_with_options` forms | `program` |
| `parse_expression` and `parse_expression_with_options` | `expression`, after skipping leading separators; the tokens after it are ignored |

### 7.3 Limits

| Constant | Bounds | Error |
| --- | --- | --- |
| `MAX_NESTING` (64) | statement and expression nesting, counted together | `Statements are nested too deeply`, `Expressions are nested too deeply` |
| `MAX_CHAIN` (128) | operators in one binary, `??` or `\|>` chain | `Expression has too many operands` |
| `MAX_CHAIN` (128) | parts of a postfix chain, selector or assignment path | `Expression has too many parts` |
| `MAX_DESTRUCTURING_TARGETS` (128) | destructuring targets | `Destructuring has too many targets` |

### 7.4 Lowerings

These forms parse to the syntax tree of their longer spelling:

| Written | Parsed as |
| --- | --- |
| `a += b`, also `-=`, `*=`, `**=`, `/=`, `%=` | `a = a + b` |
| `a &&= b`, `a \|\|= b` | `a = a && b`, `a = a \|\| b` |
| `{a, b: c} = v` | `a = v.a`, `c = v.b` |
| `a, b = v` | `a = v[0]`, `b = v[1]` |
| `enum E: A, B = 1` | `E = <enum object>` |
| `assert: a, b` | `assert(a, b)` |
| `def f(x) => e` | `def f(x): return e` |
| `class C(this.x)` | `class C(x)` with `this.x = x` in its constructor |
| `lambda a, b: e` | `(a, b) => e` |
| `f(.a.b)` | `f(v => v.a.b)` |
| `[e for x in s if c]` | `s.filter(x => c).map(x => e)` |
| `a if c else b` | `c ? a : b` |
| `new X(...)` | `X(...)` |
| `for xs:` | `for it of xs:` |
| `catch:` | `catch error:` |

### 7.5 Source Map

| Grammar | Source |
| --- | --- |
| Tokens and keywords | `src/lang/ast/lexer/raw.rs`, `src/lang/ast/lexer/token.rs` |
| Regular expressions, C-style comments, version indexes | `src/lang/ast/lexer/scanner.rs` |
| Layout | `src/lang/ast/lexer/indentation.rs` |
| Parse passes and limits | `src/lang/ast/parser.rs` |
| Programs, suites and simple statements | `src/lang/ast/parser/statement.rs` |
| `if`, `for`, `while` and `try` | `src/lang/ast/parser/control.rs` |
| Destructuring | `src/lang/ast/parser/destructuring.rs` |
| Enums | `src/lang/ast/parser/enumeration.rs` |
| Functions and parameters | `src/lang/ast/parser/function.rs` |
| Classes | `src/lang/ast/parser/class.rs` |
| Assignment, conditionals, `??` and `\|>` | `src/lang/ast/parser/expression.rs` |
| Binary and prefix operators | `src/lang/ast/parser/precedence.rs` |
| Postfix forms | `src/lang/ast/parser/postfix.rs` |
| Primary expressions, arrows, lambdas, quantifiers and objects | `src/lang/ast/parser/primary.rs` |
| Comprehensions | `src/lang/ast/parser/comprehension.rs` |
| Template literals | `src/lang/ast/parser/template.rs` |

---

## Copyright

Copyright 2020 Nucleoid

This document is licensed under the Apache License, Version 2.0. See
[LICENSE](../LICENSE) for the full text.

// Copyright 2020 Nucleoid. Licensed under Apache-2.0; see ../LICENSE.

grammar nucleoid;

/*
 * Action-free ANTLR4 syntax reference for the Rust lexer and parser.
 * nucleoid.spec.md remains normative for behavior.
 *
 * A generated parser is not a replacement for the Rust implementation:
 * its token source must supply INDENT, DEDENT, VERSION_INDEX and REGEX.
 * ANTLR prediction does not enforce the Rust parser's committed lookahead.
 * Contextual decisions, tentative parses, lowerings and limits described
 * below remain part of the implementation contract, not ANTLR actions.
 *
 * Token-source contract:
 * - Tokens outside literals and comments are ASCII.
 * - Scan C-style comments and operand-position regular expressions before
 *   ordinary tokenization. A regular expression must close on its
 *   line; otherwise '/' and '/=' keep their operator readings. Its flags
 *   are accepted and ignored. Operand position excludes a preceding name,
 *   class reference, literal, ')', ']', '}' or 'this' token.
 * - C-style comments win over regular expressions and division. Line
 *   comments start with '//'; block comments end at the first star-slash, do
 *   not nest, and fail when unterminated. So '//' is a comment, never an
 *   empty regular expression.
 * - Directly after '@', plain adjacent digits become VERSION_INDEX. Reject
 *   a following letter, digit or underscore and reject other number forms.
 * - Replace physical NEWLINEs with logical layout tokens. Parentheses and
 *   brackets suppress layout; braces do not. Ignore blank/comment-only
 *   lines. Before each subsequent logical line emit NEWLINE, then INDENT
 *   for an increased width or DEDENT for each closed level. A tab has width
 *   4; other whitespace has width 1 and comments have width 0. A width
 *   between open levels closes to the shallower level without error.
 * - Finish with the pending NEWLINE, all remaining DEDENTs, then EOF.
 *
 * WHITESPACE and COMMENT stay on the hidden channel so a token-source
 * adapter can inspect layout. REGEX_LITERAL records the scanner's pattern;
 * it is a fragment, deliberately not an unconditional '/' lexer rule.
 */

tokens { INDENT, DEDENT, VERSION_INDEX, REGEX }

program
    : separator* (statement separator*)* EOF
    ;

// parse_expression skips leading separators and ignores trailing tokens.
// Its top level forms no membership test, as an interpolation's does not.
expressionInput
    : separator* expression remainingTokens EOF
    ;

separator
    : NEWLINE
    | SEMI
    | INDENT
    | DEDENT
    ;

lineSeparator
    : NEWLINE
    | SEMI
    ;

suite
    : COLON suiteBody
    ;

suiteBody
    : indentedBlock
    | inlineSuite
    ;

// A trailing ';' in an inline suite is valid only before NEWLINE or EOF,
// never immediately before 'else' or 'catch'.
inlineSuite
    : statement (SEMI statement)* SEMI?
    ;

indentedStart
    : INDENT
    | NEWLINE lineSeparator* INDENT
    ;

indentedBlock
    : indentedStart lineSeparator* (statement lineSeparator*)* blockEnd
    ;

blockEnd
    : DEDENT
    | EOF
    ;

body
    : suite
    | indentedBlock
    ;

block
    : LBRACE separator* (statement separator*)* RBRACE
    ;

// Statements need no separator where the preceding production ends.
// At statement start, '{' is a block or pattern, never an object literal.
// Destructuring, whileStmt and repeatStmt are tentative; other recognized
// statement headers commit even if a later token fails to match, as
// repeatUntilStmt commits at 'REPEAT COLON' outside a typeDeclaration.
statement
    : destructuring
    | whileStmt
    | repeatUntilStmt
    | repeatStmt
    | enumStmt
    | ifStmt
    | forStmt
    | classStmt
    | functionDecl
    | tryStmt
    | PASS                              // Stmt::Pass
    | returnStmt
    | THROW expression                  // Stmt::Throw
    | DELETE expression                 // Stmt::Delete
    | block                             // Stmt::Block
    | typeDeclaration
    | assertStmt
    | supposeStmt
    | valueAssignment
    | isAssignment
    | predicateSentence
    | expression                        // Stmt::Assign or Stmt::Expression
    ;

returnStmt
    : RETURN expression?                // Stmt::Return
    ;

// The following token must be NEWLINE, ';', DEDENT or EOF.
typeDeclaration
    : name COLON name                   // Stmt::Declaration
    ;

// A third comma is an error.
assertStmt
    : ASSERT COLON expression COMMA expression
    ;

// A premise is tentative: it ends exactly at ':' and cannot be a colon
// statement or end in the word 'lambda'; an atVariable named lambda is not
// that word. Otherwise 'suppose' stays a name.
// The 'path IS' head follows the isAssignment contract.
supposeStmt
    : (conditional ASSIGN | path IS)? SUPPOSE premise? suite
    ;

premise
    : clauseExpression
    ;

valueAssignment
    : path VALUE_ASSIGN expression      // Stmt::ValueAssign
    ;

/*
 * Is-assignment contract (NUC 93, NUC 94, NUC 95, NUC 96):
 * - An isAssignment path may be parenthesized and has at most 128 links. No NEWLINE separates it from
 *   'is', and the token after 'is' must begin an expression: a name token,
 *   CLASS_REF, NUMBER, STRING, TEMPLATE, REGEX, LPAREN, LBRACK, LBRACE,
 *   BANG, MINUS, PLUS, TRUE, FALSE, NULL, THIS, SUPER, NEW, DELETE,
 *   FUNCTION, NOT or TYPEOF, or the AT of an atVariable whose name is not
 *   FIRST, LAST or PREVIOUS. Otherwise 'is' stays a name that begins the
 *   next statement, and 'why', 'affects', 'stated' or 'derived' before
 *   'is' keep their reasoning-selection reading.
 * - The value is ASSIGN's statement-level value and builds the tree of
 *   'path = expression' apart from its word markers. A following ':=' or a
 *   'suppose' value is handled as after ASSIGN.
 * - Word positions of the value: the whole value, its item positions, and
 *   both branches of a conditional standing in a word position, in either
 *   spelling. A word position holding exactly one name token is a word,
 *   UnaryOp::Word, which reads that name when something defines it and
 *   otherwise its own text.
 * - Item positions: each expression of a listDisplay without a
 *   comprehension, the expression of each 'objectKey COLON expression'
 *   entry, and the expression after ASSIGN of each 'name ASSIGN expression'
 *   argument of a call whose callee is a plain name, of a 'NEW primary' call
 *   and of 'SUPER call'. They lie anywhere in the value, through every
 *   construct, including conditions, branches, grouping parentheses,
 *   subscripts, slice bounds and steps, stage arguments, quantifiers and
 *   comprehension sources, except inside function, lambda and arrow
 *   bodies, comprehension elements and filters, template interpolations,
 *   the value of an inner assignment operator and the target of an
 *   assignment operator.
 * - Conditions, grouping parentheses, operands of other operators,
 *   positional call arguments, the 'name ASSIGN expression' arguments of
 *   other calls, objectKey, shorthand 'name' entries, spread sources,
 *   function, lambda and arrow bodies, template interpolations, the value of
 *   an inner assignment and the target of an assignment operator are not
 *   word positions themselves; an item position inside a condition, group,
 *   operand, positional argument or spread source stays an item position.
 */
isAssignment
    : path IS expression                // Stmt::Assign, as path = expression
    ;

/*
 * Predicate-sentence contract (NUC 98, NUC 104):
 * - The leading expression is a class reference without a property: a
 *   CLASS_REF or a spelledClass, possibly grouped in parentheses. A path
 *   with a member or version is an isAssignment instead.
 * - IS and one name token follow it, and the token after the name is
 *   NEWLINE, ';', DEDENT, '}', EOF, ELSE or CATCH, so the accepted grammar
 *   read 'CLASS_REF', 'IS' and 'name' as three one-token statements. Any
 *   other token after the name, including the start of another statement
 *   on the same line, keeps the accepted reading.
 * - It builds the tree of 'CLASS_REF DOT name ASSIGN TRUE' and renders as
 *   it; the name is the property, never a word.
 */
predicateSentence
    : classReference IS name            // Stmt::Assign, as $Class.name = true
    ;

classReference
    : CLASS_REF
    | spelledClass
    | LPAREN classReference RPAREN
    ;

// Only at a statement boundary; require NEWLINE, ';', DEDENT, '}', EOF,
// 'else' or 'catch' after the value. At most 128 targets are accepted.
destructuring
    : (objectPattern | unpackingPattern) ASSIGN conditional
    ;

objectPattern
    : LBRACE patternEntries RBRACE
    ;

patternEntries
    : namedPatternEntries
    | name (COMMA patternEntries?)?
    ;

namedPatternEntries
    : name COLON name (COMMA patternEntries? | namedPatternEntries)?
    ;

unpackingPattern
    : name COMMA (name (COMMA name)* COMMA?)?
    ;

// Enum declarations are disabled inside class bodies. Their line form
// requires a line end; names are distinct and numeric members are finite.
enumStmt
    : ENUM enumName COLON (enumBlock | enumLine)
    ;

enumLine
    : enumMember (COMMA enumMember)+
    ;

enumBlock
    : NEWLINE* INDENT NEWLINE* (enumMember NEWLINE+)* enumMember? blockEnd
    ;

enumMember
    : name (ASSIGN scalarLiteral)?      // Expr::Enum
    ;

scalarLiteral
    : (MINUS | PLUS)? NUMBER
    | STRING
    | TRUE
    | FALSE
    | NULL
    ;

ifStmt
    : IF conditionClause                // Stmt::If
    ;

conditionClause
    : clauseExpression body elseClause?
    ;

elseClause
    : lineSeparator* ELSE (ifStmt | directCondition | body)
    ;

// A direct else/elif condition needs a real indented body. Its balanced
// header rejects top-level ';', ':=', DEDENT and EOF; ':' belonging to a
// conditional or quantifier stays in the header.
directCondition
    : clauseExpression COLON? indentedBlock elseClause?
    ;

// The implicit alternative requires exactly one Rust Identifier and a real
// indented body, with or without its colon. Contextual spellings such as
// WHILE, ENUM, MODEL, REPEAT and UNTIL normalize through name/enumName.
// Only NEWLINE or INDENT may immediately follow that name or its colon;
// a header-line ';', inline/braced body or wider source never qualifies.
// Both explicit alternatives retain their colon-required suite.
forStmt
    : FOR name COMMA name (OF | IN) clauseExpression suite  // Stmt::ForEntries
    | FOR name COLON? indentedBlock                         // Stmt::For, item "it"
    | FOR name (OF | IN) clauseExpression suite             // Stmt::For
    ;

// The tentative condition must end exactly at ':', cannot be a colon
// statement or suppose expression, and cannot end in the word 'lambda';
// an atVariable named lambda is not that word.
whileStmt
    : WHILE clauseExpression suite      // Stmt::While
    ;

// The tentative count follows the whileStmt rules above, through one shared
// header test. It is evaluated once, before the first iteration.
repeatStmt
    : REPEAT clauseExpression suite     // Stmt::Repeat
    ;

// REPEAT COLON starts a repeatUntilStmt, except where a typeDeclaration
// 'REPEAT COLON name' stands before NEWLINE, ';', DEDENT or EOF, which keeps
// its reading. UNTIL starts the logical line after the body, at the
// repeatUntilStmt's layout level: only NEWLINEs follow the body, and the
// token before UNTIL is the NEWLINE that ends an inline suite or the DEDENT
// that closes an indented block, so the UNTIL line after an inline suite
// nested in another inline suite closes the innermost one. UNTIL on the
// body's line, after ';', or inside '(' and '[', which suppress layout,
// keeps its name reading, and the statement fails with
// SyntaxError("Expected 'until' after 'repeat'") where UNTIL should stand.
// The condition must be followed by NEWLINE, ';', DEDENT, EOF or the '}' that
// closes an enclosing block, which the loop leaves to that block, as
// returnStmt does.
repeatUntilStmt
    : REPEAT suite NEWLINE* UNTIL clauseExpression  // Stmt::RepeatUntil
    ;

// Without finallyClause a tryStmt is Stmt::Try, with it Stmt::TryFinally.
tryStmt
    : TRY body lineSeparator* (catchClause | exceptClause) finallyClause?
    ;

catchClause
    : CATCH (LPAREN name RPAREN | name)? body
    ;

// Python's catch-all headers build the tree of their catch spelling (NUC 122):
// 'EXCEPT COLON' and 'EXCEPT EXCEPTION COLON' are 'CATCH COLON', binding the
// name error, and 'EXCEPT EXCEPTION AS name COLON' is 'CATCH name COLON'. The
// whole header, through its COLON, is matched before any of it is consumed,
// and its suite is the explicit-colon suite of CATCH. Any other EXCEPT where
// CATCH is expected, a header without its COLON included, keeps
// SyntaxError("Expected 'catch' after 'try'") at the EXCEPT. On an inline
// body's line, EXCEPT after ';' begins another statement of the body, and a
// destructuring, enumLine or predicateSentence ends only before NEWLINE, ';',
// DEDENT, '}', EOF, ELSE or CATCH, never before EXCEPT.
exceptClause
    : EXCEPT (EXCEPTION (AS name)?)? suite
    ;

// FINALLY starts the logical line after the catch or except body, at the
// tryStmt's layout level: only NEWLINEs follow that body, and the token before
// FINALLY is a NEWLINE or the DEDENT that closes it. FINALLY on that
// body's line, after ';', or inside '(' and '[', which suppress layout, keeps
// its name reading, and so does 'FINALLY COLON name' before NEWLINE, ';',
// DEDENT or EOF, which stays a typeDeclaration.
finallyClause
    : NEWLINE* FINALLY suite
    ;

functionDecl
    : FUNCTION name parameters block    // Stmt::Function
    | DEF name defParameters defBody
    ;

// Bare parameters require a same-line arrow or ':' followed by a real
// indented block. A zero-parameter declaration requires parentheses.
defParameters
    : name name+
    | bareParameter (COMMA bareParameter)+
    | bareParameter
    | parameters
    ;

bareParameter
    : name (COLON name)?
    ;

defBody
    : arrow indentedExpression
    | arrow expression
    | suite
    ;

indentedExpression
    : indentedStart lineSeparator* expression lineSeparator* DEDENT
    ;

parameters
    : LPAREN (parameter (COMMA parameter)* COMMA?)? RPAREN
    ;

// Defaults are finite scalar literals. A rest parameter is last, has no
// default, and cannot duplicate a fixed parameter's name.
parameter
    : rest? name (COLON name)? (ASSIGN scalarLiteral)?
    ;

constructorParameters
    : LPAREN (constructorParameter (COMMA constructorParameter)* COMMA?)? RPAREN
    ;

constructorParameter
    : rest? (THIS DOT)? name (COLON name)? (ASSIGN scalarLiteral)?
    ;

// These three DOT tokens must be adjacent in the source.
rest
    : DOT DOT DOT
    ;

classStmt
    : CLASS name constructorParameters? classTail          // Stmt::Class
    ;

// A bodyless header requires a statement end with no following indented
// block. A parent after ':' is recognized only before NEWLINE, INDENT or
// EOF. Giving a parent both after 'extends' and after ':' is an error.
classTail
    : (EXTENDS name (COLON classBody)? | COLON name? classBody)?
    ;

// Without indentation, the next statement is the entire class body even
// across a line break. Use 'pass' or a bodyless header for an empty class.
classBody
    : lineSeparator*
      (INDENT lineSeparator* (classMember lineSeparator*)* blockEnd
      | EOF
      | statement)
    ;

classMember
    : PASS
    | methodDecl                        // ClassDecl::methods
    | typeDeclaration                   // ClassDecl::fields
    | statement                         // ClassDecl::constructor
    ;

methodDecl
    : DEF INIT constructorParameters defBody
    | DEF (INIT | PYTHON_INIT) receiverConstructorParameters defBody
    | DEF name receiverParameters defBody
    | FUNCTION INIT constructorParameters block
    | functionDecl
    ;

/*
 * Receiver-parameter contract (NUC 107):
 * - The receiverParameters and receiverConstructorParameters alternatives
 *   of methodDecl apply only in the receiver parse of the parse-pass
 *   contract, and only to a DEF written directly in a class's indented
 *   body. Every other parameter list, including those of a same-line class
 *   body, keeps its accepted reading and error.
 * - The receiver is THIS ('this' or 'self') directly after LPAREN and
 *   directly followed by COMMA or RPAREN. It is dropped: the method has
 *   exactly the parameters after it and binds no receiver name.
 * - DEF INIT or DEF '__init__' with a receiver declares the method 'init',
 *   whose list keeps constructorParameter prefixes. DEF '__init__' without
 *   one stays an ordinary method named '__init__'.
 * - The tree is that of the spelling without the receiver, with 'init' for
 *   '__init__'; no syntax-tree node records the receiver.
 */
receiverParameters
    : LPAREN receiver (COMMA (parameter (COMMA parameter)* COMMA?)?)? RPAREN
    ;

receiverConstructorParameters
    : LPAREN receiver (COMMA (constructorParameter (COMMA constructorParameter)* COMMA?)?)? RPAREN
    ;

receiver
    : THIS
    ;

/*
 * Expression-context contract:
 * - Clauses are headers, suppose premises, index/slice starts, the
 *   consequent of '?', Python-style conditions and quantifier collections.
 *   Their top-level ':' belongs to the enclosing construct; 'lambda'
 *   therefore stays an ordinary name there.
 * - Enclosed positions additionally include grouping, lists, comprehension
 *   parts, object values, call/stage arguments, slice ends and slice steps.
 *   Nested expressions inherit context. Statement-level assignment values
 *   are not enclosed.
 * - 'is'/'is not' compare only in enclosed positions and in an enabled
 *   parse pass. A single-name right operand is a word comparison;
 *   otherwise they mean '=='/'!='. The right operand is an atVariable
 *   region, and an atVariable there is not a single name.
 * - In an object value, a following name ':' starts the next property.
 *   'is:', 'stated name:' and 'derived name:' keep that interpretation.
 * - A Python conditional is tentative and needs 'else' directly after its
 *   condition. It is disabled in interpolation and comprehension sources.
 * - Interpolation also disables quantifiers, spelled class references, noun
 *   queries, lambdas and membership tests. The top level of an
 *   expressionInput disables membership tests only.
 *   Object values retain 'lambda name:' as a property/declaration boundary
 *   when that boundary has the accepted reading.
 */

clauseExpression
    : expression
    ;

expression
    : conditional (assignmentOperator expression)?         // Expr::Assign, right
    ;

assignmentOperator
    : ASSIGN
    | PLUS_ASSIGN
    | MINUS_ASSIGN
    | STAR_ASSIGN
    | POWER_ASSIGN
    | SLASH_ASSIGN
    | PERCENT_ASSIGN
    | AND_ASSIGN
    | OR_ASSIGN
    ;

// Compound assignments and ':=' require a path, not an arbitrary expression.
path
    : name (member | version)*
    | (CLASS_REF | THIS) (member | version)+
    ;

conditional
    : coalesce
      (QUESTION clauseExpression COLON conditional
      | IF coalesce ELSE conditional)?                     // Expr::Conditional, right
    ;

coalesce
    : pipeline (COALESCE coalesce)?                        // Expr::Logical, right
    ;

pipeline
    : logicalOr (PIPE stage)*
    ;

stage
    : WHY
    | AFFECTS
    | STATED
    | DERIVED
    | DIFF
    | NODES
    | HOLDS
    | ORIGINS
    | OUTCOMES                                            // Expr::Reason
    | DEPTH LPAREN expression RPAREN                       // Expr::Depth
    | PATH LPAREN expression RPAREN                        // Expr::Path
    | (INTERSECT | EXCEPT) LPAREN expression RPAREN        // Expr::Combine
    | UNION LPAREN expression RPAREN                       // Expr::Union
    ;

logicalOr
    : logicalAnd ((OR_OR | OR) logicalAnd)*                 // Expr::Logical, left
    ;

logicalAnd
    : equality ((AND_AND | AND) equality)*                 // Expr::Logical, left
    ;

equality
    : relational (equalityOperator relational)*            // Expr::Binary, left
    ;

equalityOperator
    : EQUAL
    | NOT_EQUAL
    | STRICT_EQUAL
    | STRICT_NOT_EQUAL
    | IS NOT?
    ;

/*
 * Membership contract (NUC 97):
 * - IN, and NOT directly followed by IN, after an additive operand build
 *   BinaryOp::In and BinaryOp::NotIn on the left-associative level of
 *   LESS, LESS_EQUAL, GREATER and GREATER_EQUAL, sharing its MAX_CHAIN
 *   operand limit. NOT before an operand stays a prefix, so
 *   'NOT a IN b' is '(NOT a) IN b'.
 * - Not at the top level of an interpolation or of an expressionInput,
 *   whose trailing tokens are ignored: IN stays their first ignored token.
 * - Not when COLON directly follows the right operand outside a clause:
 *   such a colon can only close a malformed quantifier, so IN keeps its
 *   accepted reading and the error "Unexpected keyword 'in'".
 */
relational
    : additive (relationalOperator additive)*              // Expr::Binary, left
    ;

relationalOperator
    : LESS
    | LESS_EQUAL
    | GREATER
    | GREATER_EQUAL
    | IN                                                  // BinaryOp::In
    | NOT IN                                              // BinaryOp::NotIn
    ;

additive
    : term ((PLUS | MINUS) term)*                          // Expr::Binary, left
    ;

term
    : unary ((STAR | SLASH | PERCENT) unary)*               // Expr::Binary, left
    ;

// A reasoning prefix is recognized only before a name or CLASS_REF.
unary
    : reasoningPrefix unary                               // Expr::Reason
    | noun OF unary                                       // Expr::Of
    | (BANG | NOT | MINUS | PLUS | TYPEOF) unary            // Expr::Unary
    | power
    ;

reasoningPrefix
    : WHY
    | AFFECTS
    | STATED
    | DERIVED
    ;

noun
    : SUM
    | MAX
    | MIN
    | MODE
    ;

power
    : postfix (POWER unary)?                              // Expr::Binary, right
    ;

postfix
    : primary (version | member | subscript | call)*
    ;

version
    : AT (VERSION_INDEX | FIRST | LAST | PREVIOUS)          // Expr::Version
    ;

member
    : DOT property                                        // Expr::Member
    ;

property
    : name
    | keyword
    ;

/*
 * Subscript contract (NUC 125):
 * - The start is a clause and the stop and the step are enclosed
 *   expressions, so a COLON that a conditional, a quantifier or a lambda
 *   inside a part consumes stays that part's: 'xs[c ? 1 : 2 : 4]' is the
 *   slice from 'c ? 1 : 2' to 4.
 * - Only a COLON left after the first COLON or after the stop starts a
 *   step, building Expr::SteppedSlice, whose omitted step stays omitted:
 *   'xs[1:5:]' keeps its second COLON. A third COLON is an error.
 */
subscript
    : LBRACK
      (COLON sliceTail | clauseExpression (COLON sliceTail)?)
      RBRACK                                              // Expr::Index, Expr::Slice or Expr::SteppedSlice
    ;

sliceTail
    : expression? (COLON expression?)?                    // a second COLON: Expr::SteppedSlice
    ;

call
    : LPAREN (argument (COMMA argument)* COMMA?)? RPAREN    // Expr::Call
    ;

argument
    : selector
    | section
    | expression
    ;

selector
    : (DOT property)+
    ;

/*
 * Section contract (NUC 109):
 * - It applies only in the section run of the parse-pass contract, to a
 *   whole positional argument of a call that begins with a
 *   sectionOperator. Grouping parentheses, list items, object values,
 *   subscripts, stage arguments, assertStmt arguments, parameter lists,
 *   arrow and lambda bodies and the values of named arguments keep their
 *   accepted errors.
 * - The tokens after the operator are not a separate expression: they
 *   continue an expression whose first operand is NUC 79's unnamed
 *   parameter, with the usual precedence, so '* 2 + 1' is '(v * 2) + 1' and
 *   '> 4 ? 1 : 0' is '(v > 4) ? 1 : 0'. The section ends at the COMMA or
 *   RPAREN that ends the argument.
 * - It builds the Expr::Function of 'v ARROW v sectionOperator ...' over
 *   that parameter, inside the nesting guards of an argument and of an
 *   arrowBody, and renders as 'function()'.
 */
section
    : sectionOperator expression        // Expr::Function over the unnamed parameter
    ;

sectionOperator
    : EQUAL
    | NOT_EQUAL
    | STRICT_EQUAL
    | STRICT_NOT_EQUAL
    | LESS
    | LESS_EQUAL
    | GREATER
    | GREATER_EQUAL
    | STAR
    | PERCENT
    | POWER
    | AND_AND
    | OR_OR
    | COALESCE
    ;

// A plain-name callee treats bare name=value arguments as constructor
// labels; grouping or another assignment-shaped target stays positional.
primary
    : receiverRead
    | atVariable
    | quantified
    | spelledClass
    | name arrow arrowBody
    | lambdaExpression
    | parameters arrow arrowBody
    | NUMBER
    | STRING
    | REGEX
    | templateLiteral
    | name
    | CLASS_REF
    | TRUE
    | FALSE
    | NULL
    | THIS
    | NEW primary
    | DELETE unary                                        // Expr::Delete
    | SUPER call?                                         // Expr::Super
    | FUNCTION parameters block
    | LPAREN expression RPAREN
    | listDisplay
    | objectDisplay
    ;

/*
 * At-variable contract (NUC 96):
 * - AT and the name are adjacent: the name token starts in the column
 *   after AT on the same line, so no whitespace, comment or NEWLINE
 *   separates them. A keyword, CLASS_REF, NUMBER, VERSION_INDEX, STRING or
 *   LPAREN after AT is not a name.
 * - The rule applies only where an operand begins inside an at-variable
 *   region: an isAssignment value, wherever its item positions can lie, and
 *   the right operand of an 'is'/'is not' comparison. Neither region
 *   reaches into function, lambda and arrow bodies, comprehension elements
 *   and filters, template interpolations, the value of an inner assignment
 *   operator or the target of an assignment operator; a comparison inside
 *   such a part has its own right operand. Elsewhere AT where an operand
 *   begins is "Unexpected character '@'" at the AT.
 * - An atVariable builds Expr::Identifier(name), the tree of
 *   'LPAREN name RPAREN': it spans two tokens, so it is never a word.
 *   FIRST, LAST and PREVIOUS are names here; after an operand, AT starts a
 *   version, and 'path IS AT (FIRST | LAST | PREVIOUS | VERSION_INDEX)'
 *   keeps the reading 'path' followed by the statement 'IS version'.
 * - An atVariable in an assignment target or a comprehension element is
 *   rejected with the accepted error: "Unexpected character '@'" at its
 *   AT, or, for a target that begins an isAssignment value, "Expected a
 *   non-negative integer, 'first', 'last', or 'previous' after '@'" at its
 *   name, which the accepted reading 'path' then 'IS version' reports.
 */
atVariable
    : AT name                                             // Expr::Identifier
    ;

/*
 * Receiver-read contract (NUC 102):
 * - It applies only in the final parse of the parse-pass contract, in the
 *   value of an assignment with a receiver. ASSIGN, VALUE_ASSIGN and the
 *   compound operators give their value the receiver of their target: a
 *   path whose last link is a member, rooted in a name, CLASS_REF,
 *   spelledClass or THIS, without that last member. A variable, a
 *   destructuring pattern, a computed target and a path ending in a
 *   version give none. A target of more than 128 links gives a receiver
 *   that a read rejects with "Expression has too many parts" at its DOT.
 * - The innermost assignment whose value holds the DOT decides, including
 *   assignments nested in the value and the statements of function, lambda
 *   and arrow bodies written in it; their reads are lexical.
 * - DOT begins a receiverRead only after a token in operand position, as
 *   the token-source contract defines it, or at the start of the input or
 *   of an interpolation. Never at the start of a statement, of a call
 *   argument or of an assertStmt argument, where selector and the accepted
 *   errors keep their readings, and never in an isAssignment value,
 *   including assignments nested in it.
 * - An assignment whose target begins with a receiverRead keeps the
 *   accepted error "Unexpected '.'" at its property.
 * - It builds Expr::Member over a copy of the receiver path, the tree of
 *   'receiver DOT property'. Postfix suffixes continue it as one chain, as
 *   after '(' receiver ')' DOT property.
 * Instance-read contract (NUC 113):
 * - In the same final parse, the code of a classBody has the receiver THIS
 *   wherever no assignment value gives it another: its methodDecls in every
 *   parameter form, INIT and PYTHON_INIT included, and its constructor
 *   statements, in subclasses too. A receiverRead there builds
 *   Expr::Member over Expr::This, the tree of 'THIS DOT property'.
 * - An assignment to a member path gives its value the receiver above, or
 *   none for a computed or versioned target; an assignment to a name, a
 *   label argument and a destructuring declaration keep THIS for their
 *   values.
 * - The bodies of lambdaExpression, arrowBody, FUNCTION parameters block,
 *   sections, nested function declarations and the element and condition
 *   of a comprehension have no THIS receiver, so their reads keep the
 *   accepted error; a comprehension's source and a quantified condition
 *   keep THIS. A classBody inside such a body has THIS again.
 * - The NUC 93 probes of whileStmt, repeatStmt and supposeStmt headers read
 *   THIS receivers but no assignment receivers, as the accepted probes read
 *   the written-out 'THIS DOT property'.
 */
receiverRead
    : DOT property                      // Expr::Member over the receiver path
    ;

arrow
    : ARROW
    | THIN_ARROW
    ;

// Braces directly after an arrow are always a block, not an object literal.
arrowBody
    : block
    | expression
    ;

// EVERY builds the ALL node and renders as 'all' (NUC 98).
quantified
    : (ALL | EVERY | ANY) name IN clauseExpression COLON expression
    ;

/*
 * Spelled-class contract (NUC 98, NUC 104):
 * - EVERY, ANY or ALL and the name token after it, where an operand begins
 *   outside the top level of an interpolation, build Expr::ClassRef(name),
 *   the tree of the CLASS_REF '$name'. A block comment may stand between
 *   them; NEWLINE outside parentheses and brackets separates them.
 * - Not when IN follows the name, which is quantified and keeps its
 *   priority; not when the name is IS, so 'every is v', 'any is v' and
 *   'all is v' stay an isAssignment or comparison of the first name; and
 *   not when COLON follows the name outside a clause, so the name keeps
 *   starting the next comma-free property or colon statement.
 * - The three words build the same node; none of them quantifies here.
 * - Inside an ifStmt, from its IF to the end of its last body, with every
 *   statement, block, function, lambda and class body nested there and the
 *   interpolations of its templates, the EVERY alternative is rejected with
 *   "Use all instead of every inside if" at the EVERY (NUC 98 revision 2).
 *   The other two words are unchanged there. The rule decides no parse: it
 *   is checked on the reading the parser keeps, so a tentative
 *   destructuring value, whileStmt condition, repeatStmt count, premise or
 *   Python conditional that holds such an EVERY and is dropped, or a parse
 *   pass that fails, leaves every decision as before; see the parse-pass
 *   contract.
 */
spelledClass
    : (ANY | ALL) name                  // Expr::ClassRef
    | EVERY name                        // Expr::ClassRef, outside an ifStmt only
    ;

lambdaExpression
    : LAMBDA (name (COMMA name)*)? COLON expression
    ;

listDisplay
    : LBRACK
      (expression (comprehension | (COMMA expression)* COMMA?))?
      RBRACK                                              // Expr::List
    ;

// The source's 'if' belongs to the filter, not a Python conditional.
// Recognition is enabled only in the appropriate later parse pass.
comprehension
    : FOR name (IN | OF) expression (IF expression)?
    ;

objectDisplay
    : LBRACE lineSeparator* objectEntries? RBRACE
    ;

objectEntries
    : explicitObjectEntry lineSeparator*
      (COMMA lineSeparator* objectEntries? | adjacentObjectEntries)?
    | name lineSeparator* (COMMA lineSeparator* objectEntries?)?
    ;

explicitObjectEntry
    : rest expression                                     // spread
    | objectKey COLON expression
    ;

// Only an explicit key:value entry may follow an entry without a comma.
adjacentObjectEntries
    : objectKey COLON expression lineSeparator*
      (COMMA lineSeparator* objectEntries? | adjacentObjectEntries)?
    ;

objectKey
    : name
    | STRING
    | NUMBER
    ;

// Decode TEMPLATE as text/escapes/${interpolation}. An interpolation ends
// at its balancing '}', ignoring braces inside strings and C-style
// comments, and is parsed separately in interpolation context.
templateLiteral
    : TEMPLATE                                            // Expr::Template
    ;

interpolation
    : expression remainingTokens EOF
    ;

remainingTokens
    : .*?
    ;

// Contextual spellings have distinct token types here, but remain valid
// ordinary names. 'assert', 'lambda' and 'suppose' cannot name an enum.
name
    : enumName
    | ASSERT
    | LAMBDA
    | SUPPOSE
    ;

enumName
    : NAME
    | WHILE
    | REPEAT
    | UNTIL
    | FINALLY
    | ENUM
    | EXTENDS
    | IS
    | ALL
    | EVERY
    | ANY
    | SUM
    | MAX
    | MIN
    | MODE
    | WHY
    | AFFECTS
    | STATED
    | DERIVED
    | DIFF
    | DEPTH
    | PATH
    | INTERSECT
    | EXCEPT
    | EXCEPTION
    | AS
    | UNION
    | NODES
    | HOLDS
    | ORIGINS
    | OUTCOMES
    | MODEL
    | FIRST
    | LAST
    | PREVIOUS
    | INIT
    | PYTHON_INIT
    ;

// Reserved words may follow '.' as properties, but are not declared names
// or object-literal keys. Aliases identify the canonical token/property.
keyword
    : AND
    | CATCH
    | CLASS
    | DEF
    | DELETE
    | ELSE
    | FALSE
    | FOR
    | FUNCTION
    | IF
    | IN
    | NEW
    | NOT
    | NULL
    | OF
    | OR
    | PASS
    | RETURN
    | SUPER
    | THIS
    | THROW
    | TRUE
    | TRY
    | TYPEOF
    ;

AND              : 'and';
CATCH            : 'catch';
CLASS            : 'class';
DEF              : 'def';
DELETE           : 'delete';
ELSE             : 'else' | 'elif';
FALSE            : 'false';
FOR              : 'for';
FUNCTION         : 'function';
IF               : 'if';
IN               : 'in';
NEW              : 'new';
NOT              : 'not';
NULL             : 'null' | 'None' | 'undefined';
OF               : 'of';
OR               : 'or';
PASS             : 'pass';
RETURN           : 'return';
SUPER            : 'super';
THIS             : 'this' | 'self';
THROW            : 'throw';
TRUE             : 'true';
TRY              : 'try';
TYPEOF           : 'typeof';

WHILE            : 'while';
REPEAT           : 'repeat';
UNTIL            : 'until';
FINALLY          : 'finally';
ENUM             : 'enum';
EXTENDS          : 'extends';
ASSERT           : 'assert';
SUPPOSE          : 'suppose';
LAMBDA           : 'lambda';
IS               : 'is';
ALL              : 'all';
EVERY            : 'every';
ANY              : 'any';
SUM              : 'sum';
MAX              : 'max';
MIN              : 'min';
MODE             : 'mode';
WHY              : 'why';
AFFECTS          : 'affects';
STATED           : 'stated';
DERIVED          : 'derived';
DIFF             : 'diff';
DEPTH            : 'depth';
PATH             : 'path';
INTERSECT        : 'intersect';
EXCEPT           : 'except';
EXCEPTION        : 'Exception';
AS               : 'as';
UNION            : 'union';
NODES            : 'nodes';
HOLDS            : 'holds';
ORIGINS          : 'origins';
OUTCOMES         : 'outcomes';
MODEL            : 'model';
FIRST            : 'first';
LAST             : 'last';
PREVIOUS         : 'previous';
INIT             : 'init';
PYTHON_INIT      : '__init__';

STRICT_EQUAL     : '===';
STRICT_NOT_EQUAL : '!==';
POWER_ASSIGN     : '**=';
AND_ASSIGN       : '&&=';
OR_ASSIGN        : '||=';
ARROW            : '=>';
THIN_ARROW       : '->';
VALUE_ASSIGN     : ':=';
PLUS_ASSIGN      : '+=';
MINUS_ASSIGN     : '-=';
STAR_ASSIGN      : '*=';
SLASH_ASSIGN     : '/=';
PERCENT_ASSIGN   : '%=';
EQUAL            : '==';
NOT_EQUAL        : '!=';
LESS_EQUAL       : '<=';
GREATER_EQUAL    : '>=';
POWER            : '**';
AND_AND          : '&&';
OR_OR            : '||';
COALESCE         : '??';
PIPE             : '|>';
LPAREN           : '(';
RPAREN           : ')';
LBRACK           : '[';
RBRACK           : ']';
LBRACE           : '{';
RBRACE           : '}';
COMMA            : ',';
COLON            : ':';
SEMI             : ';';
DOT              : '.';
AT               : '@';
QUESTION         : '?';
ASSIGN           : '=';
LESS             : '<';
GREATER          : '>';
PLUS             : '+';
MINUS            : '-';
STAR             : '*';
SLASH            : '/';
PERCENT          : '%';
BANG             : '!';

CLASS_REF
    : '$' [A-Za-z_] [A-Za-z0-9_]*
    ;

NUMBER
    : DIGITS ('.' DIGITS)? EXPONENT?
    | '.' DIGITS EXPONENT?
    ;

// Strings decode \n, \t, \r and \0; other escapes stand for the character.
STRING
    : '"' (ESCAPE | ~["\\])* '"'
    | '\'' (ESCAPE | ~['\\])* '\''
    ;

// Templates decode \n and \t; other escapes stand for the character.
TEMPLATE
    : '`' (ESCAPE | ~[`\\])* '`'
    ;

NAME
    : [A-Za-z_] [A-Za-z0-9_]*
    ;

NEWLINE
    : '\n'
    ;

WHITESPACE
    : [ \t\f\r]+ -> channel(HIDDEN)
    ;

COMMENT
    : '#' ~[\n]* -> channel(HIDDEN)
    ;

fragment DIGITS
    : [0-9]+
    ;

fragment EXPONENT
    : [eE] [+-]? DIGITS
    ;

fragment ESCAPE
    : '\\' ~[\n]
    ;

fragment REGEX_LITERAL
    : '/' REGEX_CHAR* '/' [A-Za-z]*
    ;

fragment REGEX_CHAR
    : ~('/' | '\\' | '[' | '\n')
    | '\\' ~[\n]
    | '[' (~(']' | '\\' | '\n') | '\\' ~[\n])* ']'
    ;

/*
 * Parse-pass contract:
 * Try the complete program without 'is' comparisons or comprehensions,
 * then with 'is', then with comprehensions, then with both. 'is' passes
 * require an 'is' name or template in the source; comprehension passes
 * require 'for' or a template. Keep the first successful complete parse.
 * A later pass never changes a source accepted by an earlier pass.
 * Every pass reads isAssignment, and its word positions never change
 * whether or where a pass succeeds or fails. Every pass reads atVariable in
 * isAssignment values, and the passes that read 'is' comparisons also read
 * it in their right operands; it applies only where the accepted grammar
 * fails at the AT, so it never changes a source that parses without it.
 * Only when all of these passes fail, and the tokens hold a receiver dot
 * (DOT, then a name or keyword, after a token in operand position or at the
 * start) or a template, one final parse reads receiverRead, with 'is'
 * comparisons when the source holds 'is' or a template and with
 * comprehensions when it holds 'for' or a template, and with the THIS
 * receivers of the instance-read contract. When it fails too, its
 * error replaces the accepted one only when it lies further into the source.
 * Passes run and succeed or fail exactly as without the EVERY rule of the
 * spelled-class contract. When the kept pass read an EVERY spelledClass
 * inside an ifStmt, the parse fails with that rule's error at the first
 * such EVERY instead. When every pass fails, the error chosen above is
 * reported, as before.
 * Only when this whole sequence fails, and the tokens hold DEF, a name,
 * LPAREN, THIS and then COMMA or RPAREN, or a template whose text holds
 * 'def' and 'self' or 'this', the same sequence runs once more reading the
 * receiverParameters of the receiver-parameter contract. Every pass, probe,
 * tentative statement and interpolation of that run reads them. When it
 * fails too, the error of the first sequence is reported unchanged.
 * Only when all of that fails, and the tokens hold LPAREN or COMMA directly
 * followed by a sectionOperator, or a template, the whole sequence above,
 * the receiver-parameter run included, runs once more from a fresh parser
 * reading section arguments of the section contract. Every pass, probe,
 * tentative statement and interpolation of that run reads them. When it
 * fails too, its error replaces the error above only when it lies further
 * into the source, by line and column; an interpolation maps its errors to
 * program positions before they are compared.
 *
 * Limits:
 * MAX_NESTING = 64, shared by statement/expression nesting:
 *   Statements are nested too deeply / Expressions are nested too deeply.
 * MAX_CHAIN = 128, operator chains:
 *   Expression has too many operands.
 * MAX_CHAIN = 128, postfix/selector/assignment paths:
 *   Expression has too many parts.
 * MAX_DESTRUCTURING_TARGETS = 128:
 *   Destructuring has too many targets.
 *
 * Lowerings and contextual values:
 * a op= b                  -> a = a op b, including &&= and ||=.
 * {a, b: c} = v            -> a = v.a; c = v.b.
 * a, b = v                 -> a = v[0]; b = v[1].
 * enum E: A, B = 1         -> E = an Expr::Enum; bare A stores "A".
 * assert: a, b             -> assert(a, b).
 * def f(x) => e            -> a function body returning e.
 * class C(this.x)          -> C(x) with this.x = x at constructor start.
 * def m(self, a): s        -> def m(a): s, in a class body's receiver parse.
 * def __init__(self, a): s -> def init(a): s, also with 'this'.
 * lambda a, b: e           -> (a, b) => e.
 * f(.a.b)                  -> f(v => v.a.b).
 * f(> a + 1)               -> f(v => v > a + 1), in the section run.
 * [e for x in s]           -> s.map(x => e).
 * [e for x in s if c]      -> s.filter(x => c).map(x => e).
 * a if c else b            -> c ? a : b.
 * p is v                   -> p = v.
 * p is w                   -> p = UnaryOp::Word(w) for a single name w.
 * p is A if c else B       -> p = c ? Word(A) : Word(B).
 * p is [A, {k: B}, X(n=C)] -> p = [Word(A), {k: Word(B)}, X(n=Word(C))].
 * p is @a + 2              -> p = a + 2, the tree of p is (a) + 2.
 * p is [@A, B]             -> p = [A, Word(B)].
 * every n in s: c          -> all n in s: c.
 * every C, any C, all C    -> $C; every C inside an ifStmt is an error.
 * $C is w                  -> $C.w = true, also spelled every C is w,
 *                             any C is w and all C is w.
 * o.p = .a + .b            -> o.p = o.a + o.b, in the final parse.
 * def m() => .a + .b       -> def m() => this.a + this.b, in a classBody's
 *                             final parse.
 * new X(...)               -> X(...).
 * for xs:                  -> iteration with the item name "it".
 * catch without a name     -> the binding "error".
 * except:                  -> catch:, also except Exception:.
 * except Exception as e:   -> catch e:.
 * model directly before |> -> Expr::Model.
 * |> intersect(model)      -> its other selection is Expr::Model; also except
 *                             and union.
 * s |> union(o)            -> Expr::Union: the steps of s, then the steps of o
 *                             whose node s does not select, in o's order. A
 *                             binding named union shadows it at run time, and
 *                             a bare |> union stays the parse-time
 *                             TypeError("union is not an operation").
 * s |> nodes               -> Expr::Reason with Stage::Nodes, the node names of
 *                             s's selection. It ends the pipeline: a stage
 *                             after it, or an intersect, except, union or path
 *                             argument that ends with it, parses and then
 *                             raises TypeError("Cannot explain <source>").
 * s |> holds               -> Expr::Reason with Stage::Holds, the values that
 *                             s's steps hold. It ends the pipeline as nodes
 *                             does, and a binding named holds shadows it at
 *                             run time.
 * s |> origins             -> Expr::Reason with Stage::Origins, the steps of s
 *                             that rest on no step of s, and
 *                             s |> outcomes -> Stage::Outcomes, the steps that
 *                             no step of s rests on; a step on a dependency
 *                             cycle rests on itself. Both keep a
 *                             selection, and a binding of either name
 *                             shadows its stage at run time.
 * An object containing spread uses Expr::SpreadObject, with later entries
 * winning; otherwise it uses Expr::ObjectLiteral. Numeric keys become text.
 *
 * Source map:
 * Tokens/keywords: src\lang\ast\lexer\raw.rs and lexer\token.rs.
 * Regex/comments/version indexes: src\lang\ast\lexer\scanner.rs.
 * Layout: src\lang\ast\lexer\indentation.rs.
 * Passes/limits: src\lang\ast\parser.rs.
 * Programs/suites/statements: src\lang\ast\parser\statement.rs.
 * Conditions/loops/exceptions: src\lang\ast\parser\control.rs.
 * Patterns/enums: parser\destructuring.rs and parser\enumeration.rs.
 * Functions/classes: parser\function.rs and parser\class.rs.
 * Operators/postfix/primary: parser\expression.rs, parser\precedence.rs,
 * parser\postfix.rs and parser\primary.rs.
 * Comprehensions/templates: parser\comprehension.rs and parser\template.rs.
 * Is assignments and their words: parser\is_assignment.rs, with the
 * conditional word positions in parser\expression.rs and the item
 * positions in parser\primary.rs, parser\postfix.rs and
 * parser\comprehension.rs.
 * At variables: parser\at_variable.rs, with their regions in
 * parser\is_assignment.rs and parser\precedence.rs, their rejection in
 * parser\expression.rs and parser\comprehension.rs and the 'lambda' header
 * colon in parser\statement.rs.
 * Every quantifiers, spelled class references and predicate sentences:
 * Quantifier::starting in src\lang\ast\mod.rs, spelledClass in
 * parser\primary.rs and predicateSentence in parser\predicate.rs, which
 * parser\statement.rs consults after an expression statement's head.
 * EVERY inside an ifStmt: the ifStmt count in parser\control.rs, the error
 * and its passes in src\lang\ast\parser.rs, and the tentative readings that
 * keep or drop it in parser\expression.rs, parser\control.rs,
 * parser\statement.rs, parser\destructuring.rs and parser\template.rs.
 * Implicit receivers: parser\receiver.rs, with the final parse in
 * src\lang\ast\parser.rs, receivers set in parser\expression.rs and
 * parser\statement.rs, reads in parser\primary.rs and parser\postfix.rs, and
 * the operand-position rule in src\lang\ast\lexer\token.rs.
 * Instance receivers: parser\receiver.rs, set for class bodies in
 * parser\class.rs, kept for variables in parser\destructuring.rs, left out
 * of nested functions in parser\primary.rs, parser\function.rs,
 * parser\section.rs and parser\comprehension.rs, and carried into the
 * accepted probe in parser\statement.rs.
 * Membership tests: relationalOperator in parser\precedence.rs, the
 * expressionInput context in parser.rs and their evaluation in
 * src\lang\ast\operator.rs.
 * Self parameters: parser\self_parameter.rs, with the receiver parse in
 * src\lang\ast\parser.rs, the method in parser\class.rs, the shared list
 * loop in parser\function.rs, and the mode carried into the accepted probe
 * in parser\statement.rs and into interpolations in parser\template.rs.
 * Operator sections: parser\section.rs, with the section run started in
 * src\lang\ast\parser.rs, the argument and the pending left operand in
 * parser\postfix.rs, the mode carried into interpolations in
 * parser\template.rs and the function in src\lang\ast\function.rs.
 */

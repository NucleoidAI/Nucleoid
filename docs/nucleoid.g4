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
// Destructuring and whileStmt are tentative; other recognized statement
// headers commit even if a later token fails to match.
statement
    : destructuring
    | whileStmt
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
 *   subscripts, slice bounds, stage arguments, quantifiers and comprehension
 *   sources, except inside function, lambda and arrow bodies, comprehension
 *   elements and filters, template interpolations, the value of an inner
 *   assignment operator and the target of an assignment operator.
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
 * Predicate-sentence contract (NUC 98):
 * - The leading expression is a class reference without a property: a
 *   CLASS_REF or an everyClass, possibly grouped in parentheses. A path
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
    | everyClass
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

forStmt
    : FOR name COMMA name (OF | IN) clauseExpression suite  // Stmt::ForEntries
    | FOR name COLON indentedBlock                          // Stmt::For, item "it"
    | FOR name (OF | IN) clauseExpression suite             // Stmt::For
    ;

// The tentative condition must end exactly at ':', cannot be a colon
// statement or suppose expression, and cannot end in the word 'lambda';
// an atVariable named lambda is not that word.
whileStmt
    : WHILE clauseExpression suite      // Stmt::While
    ;

tryStmt
    : TRY body lineSeparator* CATCH (LPAREN name RPAREN | name)? body
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
    | FUNCTION INIT constructorParameters block
    | functionDecl
    ;

/*
 * Expression-context contract:
 * - Clauses are headers, suppose premises, index/slice starts, the
 *   consequent of '?', Python-style conditions and quantifier collections.
 *   Their top-level ':' belongs to the enclosing construct; 'lambda'
 *   therefore stays an ordinary name there.
 * - Enclosed positions additionally include grouping, lists, comprehension
 *   parts, object values, call/stage arguments and slice ends. Nested
 *   expressions inherit context. Statement-level assignment values are
 *   not enclosed.
 * - 'is'/'is not' compare only in enclosed positions and in an enabled
 *   parse pass. A single-name right operand is a word comparison;
 *   otherwise they mean '=='/'!='. The right operand is an atVariable
 *   region, and an atVariable there is not a single name.
 * - In an object value, a following name ':' starts the next property.
 *   'is:', 'stated name:' and 'derived name:' keep that interpretation.
 * - A Python conditional is tentative and needs 'else' directly after its
 *   condition. It is disabled in interpolation and comprehension sources.
 * - Interpolation also disables quantifiers, every class references, noun
 *   queries and lambdas.
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
    | DIFF                                                // Expr::Reason
    | DEPTH LPAREN expression RPAREN                       // Expr::Depth
    | PATH LPAREN expression RPAREN                        // Expr::Path
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

relational
    : additive ((LESS | LESS_EQUAL | GREATER | GREATER_EQUAL) additive)*
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

subscript
    : LBRACK
      (COLON expression? | clauseExpression (COLON expression?)?)
      RBRACK                                              // Expr::Index or Expr::Slice
    ;

call
    : LPAREN (argument (COMMA argument)* COMMA?)? RPAREN    // Expr::Call
    ;

argument
    : selector
    | expression
    ;

selector
    : (DOT property)+
    ;

// A plain-name callee treats bare name=value arguments as constructor
// labels; grouping or another assignment-shaped target stays positional.
primary
    : receiverRead
    | atVariable
    | quantified
    | everyClass
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
 *   everyClass or THIS, without that last member. A variable, a
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
 * Every-class contract (NUC 98):
 * - EVERY and the name token after it, where an operand begins outside the
 *   top level of an interpolation, build Expr::ClassRef(name), the tree of
 *   the CLASS_REF '$name'. A block comment may stand between them; NEWLINE
 *   outside parentheses and brackets separates them.
 * - Not when IN follows the name, which is quantified; not when the name is
 *   IS, so 'every is v' stays an isAssignment or comparison of the name
 *   'every'; and not when COLON follows the name outside a clause, so the
 *   name keeps starting the next comma-free property or colon statement.
 */
everyClass
    : EVERY name                        // Expr::ClassRef
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
    | MODEL
    | FIRST
    | LAST
    | PREVIOUS
    | INIT
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
MODEL            : 'model';
FIRST            : 'first';
LAST             : 'last';
PREVIOUS         : 'previous';
INIT             : 'init';

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
 * comprehensions when it holds 'for' or a template. When it fails too, its
 * error replaces the accepted one only when it lies further into the source.
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
 * lambda a, b: e           -> (a, b) => e.
 * f(.a.b)                  -> f(v => v.a.b).
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
 * every C                  -> $C.
 * $C is w                  -> $C.w = true, also spelled every C is w.
 * o.p = .a + .b            -> o.p = o.a + o.b, in the final parse.
 * new X(...)               -> X(...).
 * for xs:                  -> iteration with the item name "it".
 * catch without a name     -> the binding "error".
 * model directly before |> -> Expr::Model.
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
 * Every quantifiers, class references and predicate sentences:
 * Quantifier::starting in src\lang\ast\mod.rs, everyClass in
 * parser\primary.rs and predicateSentence in parser\predicate.rs, which
 * parser\statement.rs consults after an expression statement's head.
 * Implicit receivers: parser\receiver.rs, with the final parse in
 * src\lang\ast\parser.rs, receivers set in parser\expression.rs and
 * parser\statement.rs, reads in parser\primary.rs and parser\postfix.rs, and
 * the operand-position rule in src\lang\ast\lexer\token.rs.
 */

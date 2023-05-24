const Token = require("../../lib/token");
const $INSTANCE = require("../$nuc/$INSTANCE");
const $ASSIGNMENT = require("../$nuc/$ASSIGNMENT");
const $EXP = require("../$nuc/$EXPRESSION");
const Id = require("../../lib/identifier");

function JS$ASSIGNMENT(string, offset) {
  let context = Token.next(string, offset);
  let left = context.token;
  let bracket;

  let check = Token.next(string, context.offset);

  if (check?.token === "[") {
    context = Token.nextBracket(string, check.offset);
    const { statement } = $EXP(context.bracket);
    bracket = statement;
  }

  let standard = (context = Token.next(string, context.offset));
  let point = Token.next(string, context.offset);

  standard: if (point.token === "new") {
    context = Token.next(string, point.offset);

    // eslint-disable-next-line no-eval
    if (eval("typeof " + context.token) === "function") {
      context = standard;
      break standard;
    }

    let parts = Id.splitLast(left);
    let instance;

    if (parts.length > 1) {
      instance = $INSTANCE(context.token, parts[0], parts[1]);
    } else {
      instance = $INSTANCE(context.token, left);
    }

    context = Token.next(string, context.offset);
    if (context === null || context.token === ";")
      throw SyntaxError("Missing parentheses");

    if (context.token !== "(")
      throw SyntaxError(`Unexpected token ${context.token}`);

    let check = Token.next(string, context.offset);
    if (check === null || check.token === ";")
      throw SyntaxError("Missing parenthesis");

    context = Token.nextArgs(string, context.offset);
    instance.args = context.args;

    return { statement: instance, offset: context.offset };
  }

  context = $EXP(string, context.offset);
  let right = context.statement;
  return {
    statement: $ASSIGNMENT(left, right, bracket),
    offset: context.offset,
  };
}

module.exports = JS$ASSIGNMENT;

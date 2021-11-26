const $VAR = require("../$/$variable");
const Token = require("../../token");
const $INSTANCE = require("../$/$instance");
const $EXP = require("../$/$expression");

module.exports = function ES6$VARIABLE(string, offset) {
  let context = Token.next(string, offset);

  if (context.token === "new") {
    context = Token.next(string, context.offset);
    let instance = $INSTANCE(context.token);
    context = Token.next(string, context.offset);
    context = Token.nextArgs(string, context.offset);
    instance.args = context.args;
    return { statement: instance, offset: context.offset };
  }

  context = Token.next(string, context.offset);
  let name = context.token;
  let standard = (context = Token.next(string, context.offset));

  if (context === null || context.token === ";")
    throw SyntaxError("Missing definition");

  let check = Token.next(string, context.offset);

  standard: if (check.token === "new") {
    context = Token.next(string, check.offset);

    // eslint-disable-next-line no-eval
    if (eval("typeof " + context.token) === "function") {
      context = standard;
      break standard;
    }

    let instance = $INSTANCE(context.token);
    context = Token.next(string, context.offset);
    if (context === null || context.token === ";")
      throw SyntaxError("Missing parentheses");

    if (context.token !== "(")
      throw SyntaxError(`Unexpected token ${context.token}`);

    context = Token.next(string, context.offset);
    if (context === null || context.token === ";")
      throw SyntaxError("Missing parenthesis");

    if (context.token !== ")")
      throw SyntaxError(`Unexpected token ${context.token}`);

    return { statement: $VAR(name, instance), offset: context.offset };
  }

  context = $EXP(string, context.offset);
  let expression = context.statement;
  return { statement: $VAR(name, expression), offset: context.offset };
};

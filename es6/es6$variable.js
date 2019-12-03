var $VAR = require("../$variable");
var Token = require("../token");
var $INSTANCE = require("../$instance");
var $VALUE = require("../$value");

module.exports = function ES6$VARIABLE(string, offset) {
  let context = Token.next(string, offset);

  if (context.token === "new") {
    context = Token.next(string, context.offset);
    let instance = $INSTANCE(context.token);
    context = Token.next(string, context.offset);
    context = Token.next(string, context.offset);
    return { statement: instance, offset: context.offset };
  }

  context = Token.next(string, context.offset);
  let name = context.token;
  let standard = (context = Token.next(string, context.offset));
  let check = Token.next(string, context.offset);

  standard: if (check.token === "new") {
    context = Token.next(string, check.offset);

    if (eval("typeof " + context.token) === "function") {
      context = standard;
      break standard;
    }

    let instance = $INSTANCE(context.token);
    context = Token.next(string, context.offset);
    if (context === null || context.token === ";")
      throw new SyntaxError("Missing parentheses");

    if (context.token !== "(")
      throw new SyntaxError(`Unexpected token ${context.token}`);

    context = Token.next(string, context.offset);
    if (context === null || context.token === ";")
      throw new SyntaxError("Missing parenthesis");

    if (context.token !== ")")
      throw new SyntaxError(`Unexpected token ${context.token}`);

    return { statement: $VAR(name, instance), offset: context.offset };
  }

  context = $VALUE(string, context.offset);
  let expression = context.statement;
  return { statement: $VAR(name, expression), offset: context.offset };
};

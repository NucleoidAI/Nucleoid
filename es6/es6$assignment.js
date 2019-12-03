var Token = require("../token");
var $INSTANCE = require("../$instance");
var $ASSIGNMENT = require("../$assignment");
var $VALUE = require("../$value");
var Identifier = require("../identifier");

module.exports = function ES6$ASSIGNMENT(string, offset) {
  let context = Token.next(string, offset);
  let left = context.token;

  let standard = (context = Token.next(string, context.offset));
  let point = Token.next(string, context.offset);

  standard: if (point.token === "new") {
    context = Token.next(string, point.offset);

    if (eval("typeof " + context.token) === "function") {
      context = standard;
      break standard;
    }

    let parts = Identifier.splitLast(left);
    let instance;

    if (parts.length > 1) {
      instance = $INSTANCE(context.token, parts[0], parts[1]);
    } else {
      instance = $INSTANCE(context.token, left);
    }

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

    return { statement: instance, offset: context.offset };
  }

  context = $VALUE(string, context.offset);
  let right = context.statement;
  return { statement: $ASSIGNMENT(left, right), offset: context.offset };
};

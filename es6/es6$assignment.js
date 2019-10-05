var Token = require("../token");
var $INSTANCE = require("../$instance");
var $ASSIGNMENT = require("../$assignment");
var $EXP = require("../$expression");

module.exports = function ES6$ASSIGNMENT(string, offset) {
  let context = Token.next(string, offset);
  let left = context.token;

  let standard = (context = Token.next(string, context.offset));
  let point = Token.next(string, context.offset);

  standard: if (point.token == "new") {
    context = Token.next(string, point.offset);

    if (eval("typeof " + context.token) == "function") {
      context = standard;
      break standard;
    }

    let instance = $INSTANCE(context.token);
    context = Token.next(string, context.offset);
    context = Token.next(string, context.offset);
    return { statement: $ASSIGNMENT(left, instance), offset: context.offset };
  }

  context = $EXP(string, context.offset);
  let right = context.statement;
  return { statement: $ASSIGNMENT(left, right), offset: context.offset };
};

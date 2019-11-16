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
  context = Token.next(string, context.offset);

  let check = Token.next(string, context.offset);
  if (check.token === "new") {
    context = Token.next(string, check.offset);
    let instance = $INSTANCE(context.token);
    context = Token.next(string, context.offset);
    context = Token.next(string, context.offset);
    return { statement: $VAR(name, instance), offset: context.offset };
  } else {
    context = $VALUE(string, context.offset);
    let expression = context.statement;
    return { statement: $VAR(name, expression), offset: context.offset };
  }
};

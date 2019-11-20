var Token = require("../token");
var $LET = require("../$let");
var $VALUE = require("../$value");
var $INSTANCE = require("../$instance");

module.exports = function ES6$LET(string, offset) {
  let context = Token.next(string, offset);

  context = Token.next(string, context.offset);
  let name = context.token;
  context = Token.next(string, context.offset);

  let check = Token.next(string, context.offset);
  if (check.token === "new") {
    context = Token.next(string, check.offset);
    let instance = $INSTANCE(context.token);
    context = Token.next(string, context.offset);
    context = Token.next(string, context.offset);
    return { statement: $LET(name, instance), offset: context.offset };
  } else {
    context = $VALUE(string, context.offset);
    let expression = context.statement;
    return { statement: $LET(name, expression), offset: context.offset };
  }
};

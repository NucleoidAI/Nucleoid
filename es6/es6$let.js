var Token = require("../token");
var $LET = require("../$let");
var $EXP = require("../$expression");
var $INSTANCE = require("../$instance");

module.exports = function ES6$LET(string, offset) {
  let context = Token.next(string, offset);

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
    context = Token.next(string, context.offset);
    return { statement: $LET(name, instance), offset: context.offset };
  }

  context = $EXP(string, context.offset);
  let expression = context.statement;
  return { statement: $LET(name, expression), offset: context.offset };
};

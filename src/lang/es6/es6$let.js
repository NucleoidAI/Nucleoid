const Token = require("../../token");
const $LET = require("../$/$let");
const $EXP = require("../$/$expression");
const $INSTANCE = require("../$/$instance");

module.exports = function ES6$LET(string, offset) {
  let context = Token.next(string, offset);

  context = Token.next(string, context.offset);
  let name = context.token;
  let standard = (context = Token.next(string, context.offset));
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
    context = Token.nextArgs(string, context.offset);
    instance.args = context.args;
    return { statement: $LET(name, instance), offset: context.offset };
  }

  context = $EXP(string, context.offset);
  let expression = context.statement;
  return { statement: $LET(name, expression), offset: context.offset };
};

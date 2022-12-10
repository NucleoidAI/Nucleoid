const Token = require("../../utils/token");
const $LET = require("../$nuc/$let");
const $EXP = require("../$nuc/$expression");
const $INSTANCE = require("../$nuc/$instance");

function ES6$LET(string, offset, constant) {
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
    return {
      statement: $LET(name, instance, constant),
      offset: context.offset,
    };
  }

  context = $EXP(string, context.offset);
  let expression = context.statement;
  return {
    statement: $LET(name, expression, constant),
    offset: context.offset,
  };
}

module.exports = ES6$LET;

const Token = require("../../lib/token");
const $FUNCTION = require("../$nuc/$FUNCTION");
let JS$BLOCK = require("./JS$BLOCK");

function JS$FUNCTION(string, offset) {
  let context = Token.next(string, offset);

  context = Token.next(string, context.offset);
  let name = context.token;

  context = Token.next(string, context.offset);

  context = Token.nextArgs(string, context.offset);
  let args = context.args;

  context = JS$BLOCK(string, context.offset);

  return {
    statement: $FUNCTION(name, args, context.statement),
    offset: context.offset,
  };
}

module.exports = JS$FUNCTION;

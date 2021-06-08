const Token = require("../token");
const $FUNCTION = require("../$function");

module.exports = function ES6$FUNCTION(string, offset) {
  let context = Token.next(string, offset);

  context = Token.next(string, context.offset);
  let name = context.token;

  context = Token.next(string, context.offset);

  context = Token.nextArgs(string, context.offset);
  let args = context.args;

  context = Token.next(string, context.offset);
  context = Token.nextBlock(string, context.offset);

  return {
    statement: $FUNCTION(name, args, context.block),
    offset: context.offset,
  };
};

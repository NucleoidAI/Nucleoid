const Token = require("../../token");
const $FUNCTION = require("../$/$function");
let ES6$BLOCK = require("./es6$block");

module.exports = function ES6$FUNCTION(string, offset) {
  let context = Token.next(string, offset);

  context = Token.next(string, context.offset);
  let name = context.token;

  context = Token.next(string, context.offset);

  context = Token.nextArgs(string, context.offset);
  let args = context.args;

  context = ES6$BLOCK(string, context.offset);

  return {
    statement: $FUNCTION(name, args, context.statement),
    offset: context.offset,
  };
};

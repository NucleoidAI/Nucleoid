var Token = require("../token");
var ES6 = require("./es6");
var $FOR = require("../$for");

module.exports = function ES6$FOR(string, offset) {
  let context = Token.next(string, offset);
  context = Token.next(string, context.offset);
  context = Token.next(string, context.offset);
  let variable = context.token;

  context = Token.next(string, context.offset);
  context = Token.next(string, context.offset);
  let array = context.token;

  context = Token.next(string, context.offset);
  context = Token.next(string, context.offset);
  context = Token.nextBlock(string, context.offset);
  let statements = ES6.compile(context.block, 0);

  return { statement: $FOR(variable, array, statements) };
};

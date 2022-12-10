const Token = require("../../utils/token");
const JS = require("./js");
const $FOR = require("../$nuc/$for");

function JS$FOR(string, offset) {
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
  const { statements } = JS.compile(context.block);

  return { statement: $FOR(variable, array, statements) };
}

module.exports = JS$FOR;

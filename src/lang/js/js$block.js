const Token = require("../../utils/token");
const JS = require("./js");
const $BLOCK = require("../$nuc/$block");

module.exports = function JS$BLOCK(string, offset) {
  let context = Token.next(string, offset);
  context = Token.nextBlock(string, context.offset);
  const { statements } = JS.compile(context.block);
  return { statement: $BLOCK(statements), offset: context.offset };
};

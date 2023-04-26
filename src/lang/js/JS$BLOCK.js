const Token = require("../../lib/token");
const JS = require("./JS");
const $BLOCK = require("../$nuc/$BLOCK");

function JS$BLOCK(string, offset) {
  let context = Token.next(string, offset);
  context = Token.nextBlock(string, context.offset);
  const { statements } = JS.compile(context.block);
  return { statement: $BLOCK(statements), offset: context.offset };
}

module.exports = JS$BLOCK;

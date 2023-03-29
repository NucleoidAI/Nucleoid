const Token = require("../../lib/token");
const $DELETE = require("../$nuc/$delete");

function JS$DELETE(string, offset) {
  let context = Token.next(string, offset);
  context = Token.nextStatement(string, context.offset);
  return { statement: $DELETE(context.statement), offset: context.offset };
}

module.exports = JS$DELETE;

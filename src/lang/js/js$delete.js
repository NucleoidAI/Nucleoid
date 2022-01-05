const Token = require("../../utils/token");
const $DELETE = require("../$/$delete");

module.exports = function JS$DELETE(string, offset) {
  let context = Token.next(string, offset);
  context = Token.nextStatement(string, context.offset);
  return { statement: $DELETE(context.statement), offset: context.offset };
};

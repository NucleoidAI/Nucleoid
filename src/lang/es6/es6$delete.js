const Token = require("../../token");
const $DELETE = require("../$/$delete");

module.exports = function ES6$DELETE(string, offset) {
  let context = Token.next(string, offset);
  context = Token.next(string, context.offset);
  return { statement: $DELETE(context.token), offset: context.offset };
};

var Token = require("../token");
var $CLASS = require("../$class");

module.exports = function ES6$CLASS(string, offset) {
  let context = Token.next(string, offset);
  context = Token.next(string, context.offset);
  let name = context.token;
  context = Token.next(string, context.offset);
  context = Token.nextBlock(string, context.offset);
  return { statement: $CLASS(name), offset: context.offset };
};

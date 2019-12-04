var Token = require("../token");
var $CLASS = require("../$class");

module.exports = function ES6$CLASS(string, offset) {
  let context = Token.next(string, offset);
  context = Token.next(string, context.offset);
  let name = context.token;
  context = Token.next(string, context.offset);

  if (context === null || context.token === ";")
    throw new SyntaxError("Missing parentheses");

  if (context.token !== "{")
    throw new SyntaxError(`Unexpected token ${context.token}`);

  context = Token.nextBlock(string, context.offset);

  if (context.block.trim() !== "")
    throw new SyntaxError("Methods are not supported.");

  return { statement: $CLASS(name), offset: context.offset };
};

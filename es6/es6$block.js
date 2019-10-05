var Token = require("../token");
var ES6 = require("./es6");
var $BLOCK = require("../$block");

module.exports = function ES6$BLOCK(string, offset) {
  let context = Token.next(string, offset);
  context = Token.nextBlock(string, context.offset);
  let statements = ES6.compile(context.block, 0);
  return { statement: $BLOCK(statements), offset: context.offset };
};

var Token = require("./token");
var Statement = require("./statement");
var BLOCK = require("./block");

module.exports = function(string, offset) {
  let context = Token.next(string, offset);
  context = Token.nextBlock(string, context.offset);

  let statements = Statement.compile(context.block, 0);

  let statement = new BLOCK();
  statement.statements = statements;
  return { statement: statement, offset: context.offset };
};

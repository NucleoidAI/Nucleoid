var Token = require("./token");
var Statement = require("./statement");
var BLOCK = require("./block");
var BLOCK$CLASS = require("./block$class");
var $CLASS = require("./$class").$CLASS;

module.exports = function(string, offset) {
  let context = Token.next(string, offset);
  context = Token.nextBlock(string, context.offset);

  let statements = Statement.compile(context.block, 0);
  let dependent = statements[0];
  let statement = new BLOCK();

  if (dependent instanceof $CLASS) {
    statement = new BLOCK$CLASS();
    statement.class = dependent.class;
  }

  statement.statements = statements;
  return { statement: statement, offset: context.offset };
};

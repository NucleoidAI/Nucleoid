var Token = require("./token");
var STATEMENT = require("./statement");
var $EXP = require("./expression");

module.exports = function(string, offset) {
  let context = Token.next(string, offset);

  if (context && context.token == "if") {
    context = Token.next(string, context.offset);
  }

  if (context && context.token == "(") {
    context = $EXP(string, context.offset);

    let statement = new IF();
    statement.expression = context.statement.expression;
    statement.dependencies = context.statement.dependencies;

    context = Token.next(string, context.offset);
    context = Token.nextBlock(string, context.offset);
    statement.true = context.block;
    return { statement: statement, offset: context.offset };
  }
};

class IF extends STATEMENT {}
module.exports.IF = IF;

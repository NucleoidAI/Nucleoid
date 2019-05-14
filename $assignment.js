var Token = require("./token");
var $EXP = require("./$expression");
var ASSIGNMENT = require("./assignment");

module.exports = function(string, offset) {
  let context = Token.next(string, offset);
  let variable = context.token;

  context = Token.next(string, context.offset);

  if (context && context.token == "=") {
    context = $EXP(string, context.offset);

    let statement = new ASSIGNMENT();
    statement.variable = variable;
    statement.expression = context.statement;
    return { statement: statement, offset: context.offset };
  }
};

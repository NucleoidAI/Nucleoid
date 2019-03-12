var Token = require("./token");
var STATEMENT = require("./statement");
var $EXP = require("./expression");

module.exports = function(string, offset) {
  let context = Token.next(string, offset);
  let variable = context.token;

  context = Token.next(string, context.offset);

  if (context && context.token == "=") {
    context = $EXP(string, context.offset);

    let statement = new ASSIGNMENT(context.statement);
    statement.variable = variable;
    statement.assignment = "state." + variable + "=" + statement.expression;

    return { statement: statement, offset: context.offset };
  }
};

class ASSIGNMENT extends STATEMENT {
  constructor(statement) {
    super();
    this.expression = statement.expression;
    this.dependencies = statement.dependencies;
  }
}
module.exports.ASSIGNMENT = ASSIGNMENT;

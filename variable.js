var graph = require("./graph");
var Token = require("./token");
var STATEMENT = require("./statement");
var $ASSIGN = require("./assignment");

module.exports = function(string, offset) {
  let context = Token.next(string, offset);

  if (context.token == "var") {
    offset = context.offset;
    context = Token.next(string, context.offset);
  }

  if (context && !graph.node[context.token]) {
    context = $ASSIGN(string, offset);
    return {
      statement: new VARIABLE(context.statement),
      offset: context.offset
    };
  }
};

class VARIABLE extends STATEMENT {
  constructor(statement) {
    super();
    this.variable = statement.variable;
    this.assignment = "state." + statement.assignment;
    this.dependencies = statement.dependencies;
  }
}
module.exports.VARIABLE = VARIABLE;

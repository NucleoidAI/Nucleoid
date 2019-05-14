var graph = require("./graph");
var Token = require("./token");
var $EXP = require("./$expression");
var CLASS = require("./class");
var ASSIGNMENT = require("./assignment");
var ASSIGNMENT$CLASS = require("./assignment$class");

module.exports = function(string, offset) {
  let context = Token.next(string, offset);
  let variable;

  if (context.token == "var") {
    context = Token.next(string, context.offset);
    variable = context.token;
  } else {
    variable = context.token;
  }

  context = Token.next(string, context.offset);

  if (context && !graph.node[context.token]) {
    context = $EXP(string, context.offset);

    let prefix = variable.split(".")[0];

    let statement;

    if (graph.node[prefix] && graph.node[prefix].statement instanceof CLASS) {
      statement = new ASSIGNMENT$CLASS();
      statement.class = prefix;
      statement.variable = variable;
      statement.expression = context.statement;
    } else {
      statement = new ASSIGNMENT();
      statement.variable = variable;
      statement.expression = context.statement;
    }

    return {
      statement: statement,
      offset: context.offset
    };
  }
};

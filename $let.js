var graph = require("./graph");
var Token = require("./token");
var $EXP = require("./$expression");
var CLASS = require("./class");
var LET = require("./let");
var LET$CLASS = require("./let$class");

module.exports = function(string, offset) {
  let context = Token.next(string, offset);
  let variable;

  if (context.token == "let") {
    context = Token.next(string, context.offset);
    variable = context.token;

    context = Token.next(string, context.offset);
    context = $EXP(string, context.offset);

    let statement = new LET();
    statement.variable = variable;
    statement.expression = context.statement;

    for (let token of context.statement.tokens) {
      let prefix = token.split(".")[0];

      if (graph.node[prefix] && graph.node[prefix].statement instanceof CLASS) {
        statement = new LET$CLASS();
        statement.variable = variable;
        statement.expression = context.statement;
        statement.class = prefix;
        break;
      }
    }

    return {
      statement: statement,
      offset: context.offset
    };
  }
};

var graph = require("./graph");
var Token = require("./token");
var $EXP = require("./$expression");
var $BLOCK = require("./$block");
var IF = require("./if");
var CLASS = require("./class");
var IF$CLASS = require("./if$class");
var BLOCK$CLASS = require("./block$class");

module.exports = function(string, offset) {
  let context = Token.next(string, offset);

  if (context && context.token == "if")
    context = Token.next(string, context.offset);

  if (context && context.token == "(") {
    context = $EXP(string, context.offset);

    let point = context.offset;

    const condition = context.statement;

    for (let token of condition.tokens) {
      let prefix = token.split(".")[0];

      if (graph.node[prefix] && graph.node[prefix] instanceof CLASS) {
        let statement = new IF$CLASS();

        statement.class = graph.node[prefix];
        statement.condition = condition;

        context = $BLOCK(string, context.offset);

        let block = new BLOCK$CLASS();
        block.statements = context.statement.statements;
        statement.true = block;
        return { statement: statement, offset: context.offset };
      }
    }

    let statement = new IF();
    statement.condition = context.statement;
    context = $BLOCK(string, point);
    statement.true = context.statement;
    return { statement: statement, offset: context.offset };
  }
};

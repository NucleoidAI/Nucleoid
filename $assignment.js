var graph = require("./graph");
var Token = require("./token");
var $EXP = require("./$expression");
var CLASS = require("./class");
var INSTANCE = require("./instance");
var ASSIGNMENT$CLASS = require("./assignment$class");
var ASSIGNMENT$PROPERTY = require("./assignment$property");
var ASSIGNMENT$VARIABLE = require("./assignment$variable");

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
  let expression = context;

  if (context && !graph.node[context.token]) {
    let check = Token.next(string, context.offset);

    instance: if (check && check.token == "new") {
      let statement = new INSTANCE();
      statement.variable = variable;

      context = check;
      context = Token.next(string, context.offset);

      if (graph.node[context.token] instanceof CLASS) {
        statement.class = graph.node[context.token];
        context = Token.next(string, context.offset);
        context = Token.next(string, context.offset);
      } else {
        context = expression;
        break instance;
      }

      return {
        statement: statement,
        offset: context.offset
      };
    }

    context = $EXP(string, context.offset);

    let prefix = variable.split(".")[0];

    let statement;

    if (graph.node[prefix] && graph.node[prefix] instanceof CLASS) {
      statement = new ASSIGNMENT$CLASS();
      statement.class = graph.node[prefix];
      statement.expression = context.statement;

      let parts = variable.split(".");
      parts.shift();
      statement.property = parts.join(".");
    } else if (graph.node[prefix] && graph.node[prefix] instanceof INSTANCE) {
      statement = new ASSIGNMENT$PROPERTY();
      statement.instance = graph.node[prefix];
      statement.expression = context.statement;

      let parts = variable.split(".");
      parts.shift();
      statement.property = parts.join(".");
    } else {
      statement = new ASSIGNMENT$VARIABLE();
      statement.variable = variable;
      statement.expression = context.statement;
    }

    return {
      statement: statement,
      offset: context.offset
    };
  }
};

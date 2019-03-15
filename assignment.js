var state = require("./state"); // eslint-disable-line no-unused-vars
var graph = require("./graph");
var Node = require("./node");
var Token = require("./token");
var $EXP = require("./expression");
var CLASS = require("./class").CLASS;

module.exports = function(string, offset) {
  let context = Token.next(string, offset);
  let variable = context.token;

  context = Token.next(string, context.offset);

  if (context && context.token == "=") {
    context = $EXP(string, context.offset);

    let statement = new ASSIGNMENT(context.statement);
    statement.variable = variable;

    let prefix = variable.split(".")[0];

    if (graph.node[prefix] && graph.node[prefix].statement instanceof CLASS) {
      statement.class = prefix;
    }

    return { statement: statement, offset: context.offset };
  }
};

class ASSIGNMENT {
  constructor(expression) {
    this.expression = expression;
  }

  run() {
    let variable = this.variable;
    let expression = this.expression;

    if (!graph.node[variable]) {
      graph.node[variable] = new Node(this);
    }

    for (let i = 0; i < expression.tokens.length; i++) {
      let token = expression.tokens[i];

      if (graph.node[token]) {
        graph.node[token].edge[variable] = graph.node[variable];
      }

      if (graph.node[token.split(".")[0]]) {
        expression.tokens[i] = "state." + token;
      }
    }

    eval("state." + variable + "=" + expression.tokens.join(""));
  }
}
module.exports.ASSIGNMENT = ASSIGNMENT;

var state = require("./state"); // eslint-disable-line no-unused-vars
var graph = require("./graph");
var Node = require("./node");
var Token = require("./token");
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

class VARIABLE {
  constructor(assignment) {
    this.expression = assignment.expression;
    this.variable = assignment.variable;
    this.class = assignment.class;
  }

  run() {
    if (this.function) {
      this.function(state);
    }

    let variable = this.variable;
    let expression = this.expression;

    // Align with instance

    if (this.class && this.instance) {
      let parts = variable.split(".");

      if (parts[0] == this.class) {
        parts[0] = this.instance;
        variable = parts.join(".");
      }

      for (let i = 0; i < expression.tokens.length; i++) {
        let token = expression.tokens[i];
        let parts = token.split(".");

        if (parts[0] == this.class) {
          parts[0] = this.instance;
        }

        expression.tokens[i] = parts.join(".");
      }
    }

    // Adjust graph

    if (!graph.node[variable]) {
      graph.node[variable] = new Node(this);
    }

    for (let i = 0; i < expression.tokens.length; i++) {
      let token = expression.tokens[i];

      if (graph.node[token]) {
        graph.node[token].edge[variable] = graph.node[variable];
      }
    }

    // Apply to instances

    if (this.class && graph.node[this.class] && !this.instance) {
      let list = [];

      for (let e in graph.node[this.class].edge) {
        let statement = graph.node[this.class].edge[e].statement;

        if (statement.variable) {
          let v = new VARIABLE({ expression: expression, variable: variable });
          v.class = this.class;
          v.instance = statement.variable;
          list.push(v);
        }
      }

      return list;
    }

    // Compile into function

    let list = [];

    for (let i = 0; i < expression.tokens.length; i++) {
      let token = expression.tokens[i];

      if (graph.node[token.split(".")[0]]) {
        list.push("state." + token);
      } else {
        list.push(token);
      }
    }

    this.function = new Function(
      "state",
      "state." + variable + "=" + list.join("")
    );
    this.function(state);
  }
}
module.exports.VARIABLE = VARIABLE;

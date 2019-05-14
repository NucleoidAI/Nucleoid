var state = require("./state"); // eslint-disable-line no-unused-vars
var graph = require("./graph");
var Node = require("./node");

module.exports = class ASSIGNMENT {
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
};

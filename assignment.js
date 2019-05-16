var state = require("./state"); // eslint-disable-line no-unused-vars
var graph = require("./graph");
var Node = require("./node");

module.exports = class ASSIGNMENT {
  run(local) {
    let variable = this.variable;
    let expression = this.expression;

    let list = expression.tokens.map(token => {
      if (graph.node[token.split(".")[0]]) {
        return "state." + token;
      } else if (local[token]) {
        return local[token];
      } else {
        return token;
      }
    });

    eval("state." + variable + "=" + list.join(""));
  }

  graph() {
    let variable = this.variable;
    let expression = this.expression;

    if (graph.index[variable]) {
      graph.index[variable].forEach(i => delete graph.node[i].edge[variable]);

      graph.node[variable].statement = this;
      graph.index[variable] = [];
    } else {
      graph.node[variable] = new Node(this);
      graph.index[variable] = [];
    }

    expression.tokens.forEach(token => {
      if (graph.node[token]) {
        graph.node[token].edge[variable] = graph.node[variable];
        graph.index[variable].push(token);
      }
    });
  }
};

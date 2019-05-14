var graph = require("./graph");
var Node = require("./node");
var ASSIGNMENT$INSTANCE = require("./assignment$instance");

module.exports = class ASSIGNMENT$CLASS {
  run() {
    let list = [];

    for (let e in graph.node[this.class].edge) {
      let statement = graph.node[this.class].edge[e].statement;

      let instance = new ASSIGNMENT$INSTANCE();
      instance.expression = this.expression;
      instance.variable = this.variable;
      instance.class = this.class;
      instance.instance = statement.variable;

      list.push(instance);
    }

    return list;
  }

  graph() {
    const variable = this.variable;
    const expression = this.expression;

    graph.node[variable] = new Node(this);

    expression.tokens.forEach(token => {
      if (graph.node[token])
        graph.node[token].edge[variable] = graph.node[variable];
    });
  }
};

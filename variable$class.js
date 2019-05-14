var graph = require("./graph");
var Node = require("./node");
var VARIABLE$INSTANCE = require("./variable$instance");

module.exports = class VARIABLE$CLASS {
  run() {
    let variable = this.variable;
    let expression = this.expression;

    graph.node[variable] = new Node(this);

    expression.tokens.forEach(token => {
      if (graph.node[token])
        graph.node[token].edge[variable] = graph.node[variable];
    });

    let list = [];

    for (let e in graph.node[this.class].edge) {
      let statement = graph.node[this.class].edge[e].statement;

      let instance = new VARIABLE$INSTANCE();
      instance.expression = this.expression;
      instance.variable = variable;
      instance.class = this.class;
      instance.instance = statement.variable;

      list.push(instance);
    }

    return list;
  }
};

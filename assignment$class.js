var graph = require("./graph");
var Node = require("./node");
var ASSIGNMENT$INSTANCE = require("./assignment$instance");
var $CLASS = require("./$class").$CLASS;

module.exports = class ASSIGNMENT$CLASS extends $CLASS {
  run() {
    let list = [];

    if (this.instance) {
      let instance = new ASSIGNMENT$INSTANCE();
      instance.expression = this.expression;
      instance.variable = this.variable;
      instance.class = this.class;
      instance.instance = this.instance;
      return instance;
    }

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

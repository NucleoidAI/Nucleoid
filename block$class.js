var graph = require("./graph");
var Node = require("./node");
var BLOCK$INSTANCE = require("./block$instance");
var $CLASS = require("./$class").$CLASS;

module.exports = class BLOCK$CLASS extends $CLASS {
  constructor() {
    super();
    this.statements = [];
  }
  run() {
    let list = [];

    if (this.instance) {
      let instance = new BLOCK$INSTANCE();
      instance.class = this.class;
      instance.statements = this.statements;
      return instance;
    }

    for (let e in graph.node[this.class].edge) {
      let statement = graph.node[this.class].edge[e].statement;
      let instance = new BLOCK$INSTANCE();
      instance.class = this.class;
      instance.instance = statement.variable;
      instance.statements = this.statements;
      list.push(instance);
    }

    return list;
  }

  graph() {
    let statement = this.statements[0];
    let expression = statement.expression;

    const id = Date.now();
    graph.node[id] = new Node(this);

    expression.tokens.forEach(token => {
      if (graph.node[token]) {
        graph.node[token].edge[id] = graph.node[id];
      }
    });
  }
};

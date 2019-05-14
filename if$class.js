var graph = require("./graph");
var BLOCK$INSTANCE = require("./block$instance");
var IF$INSTANCE = require("./if$instance");

module.exports = class IF$CLASS {
  run() {
    let list = [];

    for (let e in graph.node[this.class].edge) {
      const statement = graph.node[this.class].edge[e].statement;
      let instance = new IF$INSTANCE();
      instance.condition = this.condition;
      instance.class = this.class;
      instance.instance = statement.variable;

      instance.true = new BLOCK$INSTANCE();
      this.true.statements.forEach(e => instance.true.statements.push(e));
      instance.true.class = this.class;
      instance.true.instance = statement.variable;
      list.push(instance);
    }

    return list;
  }

  graph() {}
};

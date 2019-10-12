var Node = require("./node");
var BLOCK$INSTANCE = require("./block$instance");
var IF$INSTANCE = require("./if$instance");

module.exports = class IF$CLASS extends Node {
  run() {
    let list = [];

    for (let node in this.class.instance) {
      let statement = new IF$INSTANCE();
      statement.class = this.class;
      statement.instance = this.class.instance[node];
      statement.condition = this.condition;

      statement.true = new BLOCK$INSTANCE();
      statement.true.statements = this.true.statements;
      statement.true.class = this.class;
      statement.true.instance = statement.instance;
      list.push(statement);
    }

    return list;
  }
};

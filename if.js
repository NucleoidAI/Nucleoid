var graph = require("./graph");
var Node = require("./node");

module.exports = class IF extends Node {
  run(local) {
    if (this.condition.run(local)) {
      return this.true;
    }
  }

  graph() {
    let key = "if(" + this.condition.tokens.join("") + ")";
    graph.node[key] = this;

    this.condition.tokens.forEach(token => {
      if (graph.node[token]) Node.direct(token, key, this);
    });
  }
};

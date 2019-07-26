var graph = require("./graph");
var Node = require("./node");

module.exports = class IF extends Node {
  run(scope) {
    if (this.condition.run(scope.local)) {
      return this.true;
    } else {
      return this.false;
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

var graph = require("./graph");
var Node = require("./node");

module.exports = class IF extends Node {
  run(scope) {
    if (this.condition.run(scope)) {
      return this.true;
    } else if (this.false && this.false instanceof IF) {
      return this.false.run(scope);
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

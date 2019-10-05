var state = require("./state");
var graph = require("./graph");
var Node = require("./node");

module.exports = class VARIABLE extends Node {
  run(scope) {
    state[this.name] = this.value.run(scope);
  }

  graph() {
    if (graph.node[this.name]) Node.replace(this.name, this);
    else graph.node[this.name] = this;

    this.value.tokens.forEach(token => {
      if (graph.node[token]) Node.direct(token, this.name, this);
    });
  }
};

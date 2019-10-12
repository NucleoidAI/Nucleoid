var state = require("./state");
var graph = require("./graph");
var Node = require("./node");

module.exports = class ASSIGNMENT$VARIABLE extends Node {
  run(local) {
    state[this.variable] = this.expression.run(local);
  }

  graph() {
    if (graph.node[this.variable]) Node.replace(this.variable, this);
    else graph.node[this.variable] = this;

    this.expression.tokens.forEach(token => {
      if (graph.node[token]) Node.direct(token, this.variable, this);
    });
  }
};

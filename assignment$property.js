var state = require("./state");
var graph = require("./graph");
var Node = require("./node");

module.exports = class ASSIGNMENT$PROPERTY extends Node {
  run(local) {
    state[this.instance.variable][this.property] = this.expression.run(local);
  }

  graph() {
    let key = this.instance.variable + "." + this.property;

    if (graph.node[key]) Node.replace(key, this);
    else graph.node[key] = this;

    this.expression.tokens.forEach(token => {
      if (graph.node[token]) Node.direct(token, key, this);
      else if (graph.node[token.split(".")[0]]) {
        graph.node[token] = new Node();
        Node.direct(token, key, this);
      }
    });
  }
};

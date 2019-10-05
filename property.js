var state = require("./state");
var graph = require("./graph");
var Node = require("./node");

module.exports = class PROPERTY extends Node {
  run(scope) {
    let instance = state[this.instance.name];
    instance[this.name] = this.value.run(scope);
  }

  graph() {
    let key = this.instance.name + "." + this.name;

    if (graph.node[key]) Node.replace(key, this);
    else graph.node[key] = this;

    this.value.tokens.forEach(token => {
      if (graph.node[token]) Node.direct(token, key, this);
      else if (graph.node[token.split(".")[0]]) {
        graph.node[token] = new Node();
        Node.direct(token, key, this);
      }
    });
  }
};
